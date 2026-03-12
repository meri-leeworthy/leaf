//! Parser trait and implementations for Socket.IO packet encoding/decoding
//!
//! This module defines the parser abstraction that allows socketio-rust to work
//! with different data encodings (JSON, MessagePack, CBOR, etc.)

use crate::packet::Packet;
use async_trait::async_trait;
use bytes::Bytes;

/// Errors that can occur during parsing
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON encoding error: {0}")]
    JsonEncode(String),

    #[error("JSON decoding error: {0}")]
    JsonDecode(String),

    #[error("MessagePack encoding error: {0}")]
    MsgPackEncode(String),

    #[error("MessagePack decoding error: {0}")]
    MsgPackDecode(String),

    #[error("CBOR encoding error: {0}")]
    CborEncode(String),

    #[error("CBOR decoding error: {0}")]
    CborDecode(String),

    #[error("Unsupported packet type: {0}")]
    UnsupportedPacketType(String),

    #[error("Invalid packet format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

/// A parser for encoding and decoding Socket.IO packets
///
/// Socket.IO supports pluggable parsers to change how data is encoded.
/// This trait allows socketio-rust to support different parsers.
#[async_trait]
pub trait Parser: Send + Sync {
    /// Encode a packet into bytes for transmission
    ///
    /// The format depends on the parser:
    /// - JSON: UTF-8 string representation
    /// - MessagePack: Binary msgpack encoding
    /// - CBOR: Binary CBOR encoding
    fn encode(&self, packet: &Packet) -> Result<Bytes>;

    /// Decode a packet from received bytes
    fn decode(&self, bytes: &[u8]) -> Result<Packet>;

    /// Get the parser type identifier
    fn parser_type(&self) -> &'static str;
}

/// JSON parser (default Socket.IO parser)
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for JsonParser {
    fn encode(&self, packet: &Packet) -> Result<Bytes> {
        // JSON parser encodes packets as JSON strings
        let json = serde_json::to_string(packet)
            .map_err(|e| ParseError::JsonEncode(e.to_string()))?;
        Ok(Bytes::from(json))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Packet> {
        let packet: Packet = serde_json::from_slice(bytes)
            .map_err(|e| ParseError::JsonDecode(e.to_string()))?;
        Ok(packet)
    }

    fn parser_type(&self) -> &'static str {
        "json"
    }
}

/// MessagePack parser (for servers using msgpack encoding)
#[cfg(feature = "msgpack-parser")]
pub struct MsgPackParser;

#[cfg(feature = "msgpack-parser")]
impl MsgPackParser {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "msgpack-parser")]
impl Default for MsgPackParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "msgpack-parser")]
#[async_trait]
impl Parser for MsgPackParser {
    fn encode(&self, packet: &Packet) -> Result<Bytes> {
        use rmp_serde::encode::Serializer;
        use serde::Serialize;

        let mut buf = Vec::new();
        let mut se = Serializer::new(&mut buf);
        packet.serialize(&mut se)
            .map_err(|e| ParseError::MsgPackEncode(e.to_string()))?;
        Ok(Bytes::from(buf))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Packet> {
        use rmp_serde::from_slice;

        let packet: Packet = from_slice(bytes)
            .map_err(|e| ParseError::MsgPackDecode(e.to_string()))?;
        Ok(packet)
    }

    fn parser_type(&self) -> &'static str {
        "msgpack"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PacketType;

    #[test]
    fn test_json_parser_encode_connect_packet() {
        let parser = JsonParser::new();
        let packet = Packet::connect("/");

        let encoded = parser.encode(&packet).unwrap();
        let json_str = std::str::from_utf8(&encoded).unwrap();

        assert!(json_str.contains("\"type\":0"));
        assert!(json_str.contains("\"nsp\":\"/\""));
    }

    #[test]
    fn test_json_parser_decode_connect_packet() {
        let parser = JsonParser::new();
        let json = r#"{"type":0,"nsp":"/","data":null}"#;

        let decoded = parser.decode(json.as_bytes()).unwrap();

        assert_eq!(decoded.packet_type, PacketType::Connect);
        assert_eq!(decoded.nsp, "/");
    }

    #[test]
    fn test_json_parser_roundtrip() {
        let parser = JsonParser::new();
        let original = Packet::event("test", Some(serde_json::json!({"key":"value"})), None);

        let encoded = parser.encode(&original).unwrap();
        let decoded = parser.decode(&encoded).unwrap();

        assert_eq!(decoded.packet_type, original.packet_type);
        assert_eq!(decoded.nsp, original.nsp);
    }
}
