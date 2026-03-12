//! Engine.IO protocol implementation
//!
//! Engine.IO is the transport layer that Socket.IO is built on top of.
//! This module handles the Engine.IO handshake and packet encoding/decoding.

use crate::error::{Error, Result};
use bytes::Bytes;

/// Engine.IO packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnginePacketType {
    /// Open (0) - Server opens connection
    Open = 0,
    /// Close (1) - Close connection
    Close = 1,
    /// Ping (2) - Ping packet
    Ping = 2,
    /// Pong (3) - Pong packet
    Pong = 3,
    /// Message (4) - Actual message (contains Socket.IO packet)
    Message = 4,
    /// Upgrade (5) - Upgrade to WebSocket
    Upgrade = 5,
    /// Noop (6) - No-op
    Noop = 6,
}

impl EnginePacketType {
    /// Parse from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(EnginePacketType::Open),
            1 => Some(EnginePacketType::Close),
            2 => Some(EnginePacketType::Ping),
            3 => Some(EnginePacketType::Pong),
            4 => Some(EnginePacketType::Message),
            5 => Some(EnginePacketType::Upgrade),
            6 => Some(EnginePacketType::Noop),
            _ => None,
        }
    }
}

/// Engine.IO handshake response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HandshakeResponse {
    /// Session ID
    pub sid: String,
    /// Available upgrades
    pub upgrades: Vec<String>,
    /// Ping interval in milliseconds
    #[serde(rename = "pingInterval")]
    pub ping_interval: u64,
    /// Ping timeout in milliseconds
    #[serde(rename = "pingTimeout")]
    pub ping_timeout: u64,
    /// Maximum payload size
    #[serde(rename = "maxPayload")]
    pub max_payload: usize,
}

/// Engine.IO packet
#[derive(Debug, Clone)]
pub struct EnginePacket {
    /// Packet type
    pub packet_type: EnginePacketType,
    /// Packet data (for Message type, contains Socket.IO packet)
    pub data: Option<Bytes>,
}

impl EnginePacket {
    /// Create an Engine.IO packet from Socket.IO packet data
    pub fn message(data: Bytes) -> Self {
        EnginePacket {
            packet_type: EnginePacketType::Message,
            data: Some(data),
        }
    }

    /// Create a ping packet
    pub fn ping() -> Self {
        EnginePacket {
            packet_type: EnginePacketType::Ping,
            data: None,
        }
    }

    /// Create a pong packet
    pub fn pong() -> Self {
        EnginePacket {
            packet_type: EnginePacketType::Pong,
            data: None,
        }
    }

    /// Encode Engine.IO packet for transmission
    ///
    /// Format: `<packet_type><data?>`
    /// For text packets: "4<message>"
    /// For binary packets with length: "<length>:<packet_type><data>"
    pub fn encode(&self) -> Result<Bytes> {
        match self.packet_type {
            EnginePacketType::Message => {
                // Message packets contain Socket.IO data
                if let Some(ref data) = self.data {
                    // Check if data is binary (contains non-UTF8 bytes)
                    let is_binary = data.iter().any(|&b| b > 127);

                    if is_binary {
                        // Binary format: "<length>:<type><data>"
                        let total_len = 1 + data.len();
                        let mut encoded = format!("{}:{}", total_len, u8::from(self.packet_type)).into_bytes();
                        encoded.extend_from_slice(data);
                        Ok(Bytes::from(encoded))
                    } else {
                        // Text format: "4<socket.io packet>"
                        let mut encoded = vec![b'4'];
                        encoded.extend_from_slice(data);
                        Ok(Bytes::from(encoded))
                    }
                } else {
                    Ok(Bytes::from("4"))
                }
            }
            EnginePacketType::Ping => Ok(Bytes::from("2")),
            EnginePacketType::Pong => Ok(Bytes::from("3")),
            _ => Ok(Bytes::from(format!("{}", u8::from(self.packet_type)))),
        }
    }

    /// Decode Engine.IO packet from received data
    ///
    /// Handles both text format (e.g., "4hello") and binary format with length prefix
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::InvalidResponse("Empty Engine.IO packet".to_string()));
        }

        // Check if this is a length-prefixed binary packet
        if let Some(colon_pos) = data.iter().position(|&b| b == b':') {
            // Format: "<length>:<type><data>"
            let length_str = std::str::from_utf8(&data[..colon_pos])
                .map_err(|e| Error::InvalidResponse(format!("Invalid length prefix: {}", e)))?;

            let expected_length: usize = length_str.parse()
                .map_err(|_| Error::InvalidResponse(format!("Invalid length: {}", length_str)))?;

            if data.len() < colon_pos + 1 + expected_length {
                return Err(Error::InvalidResponse("Incomplete packet".to_string()));
            }

            let packet_type_byte = data[colon_pos + 1];
            let packet_type = EnginePacketType::from_u8(packet_type_byte)
                .ok_or_else(|| Error::InvalidResponse(format!("Invalid Engine.IO packet type: {}", packet_type_byte)))?;

            let packet_data = if expected_length > 1 {
                Some(Bytes::copy_from_slice(&data[colon_pos + 2..colon_pos + 1 + expected_length]))
            } else {
                None
            };

            return Ok(EnginePacket {
                packet_type,
                data: packet_data,
            });
        }

        // Simple format: just packet type
        let first_byte = data[0];
        if first_byte < b'0' || first_byte > b'9' {
            // Binary data without length prefix (WebSocket frame)
            return Ok(EnginePacket {
                packet_type: EnginePacketType::from_u8(first_byte)
                    .ok_or_else(|| Error::InvalidResponse(format!("Invalid Engine.IO packet type: {}", first_byte)))?,
                data: if data.len() > 1 { Some(Bytes::copy_from_slice(&data[1..])) } else { None },
            });
        }

        // Text format
        let packet_type = EnginePacketType::from_u8(first_byte - b'0')
            .ok_or_else(|| Error::InvalidResponse(format!("Invalid Engine.IO packet type: {}", first_byte)))?;

        let packet_data = if data.len() > 1 {
            Some(Bytes::copy_from_slice(&data[1..]))
        } else {
            None
        };

        Ok(EnginePacket {
            packet_type,
            data: packet_data,
        })
    }
}

impl From<EnginePacketType> for u8 {
    fn from(pt: EnginePacketType) -> Self {
        pt as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_packet_type_from_u8() {
        assert_eq!(EnginePacketType::from_u8(0), Some(EnginePacketType::Open));
        assert_eq!(EnginePacketType::from_u8(4), Some(EnginePacketType::Message));
        assert_eq!(EnginePacketType::from_u8(99), None);
    }

    #[test]
    fn test_encode_message_packet_text() {
        let data = Bytes::from("test message");
        let packet = EnginePacket::message(data);
        let encoded = packet.encode().unwrap();

        // Should start with '4' (Message type)
        assert_eq!(encoded[0], b'4');
        assert_eq!(&encoded[1..], b"test message");
    }

    #[test]
    fn test_encode_ping_packet() {
        let packet = EnginePacket::ping();
        let encoded = packet.encode().unwrap();

        assert_eq!(&encoded[..], b"2");
    }

    #[test]
    fn test_encode_pong_packet() {
        let packet = EnginePacket::pong();
        let encoded = packet.encode().unwrap();

        assert_eq!(&encoded[..], b"3");
    }

    #[test]
    fn test_decode_text_message() {
        let data = b"4test message";
        let packet = EnginePacket::decode(data).unwrap();

        assert_eq!(packet.packet_type, EnginePacketType::Message);
        assert_eq!(packet.data, Some(Bytes::copy_from_slice(b"test message")));
    }

    #[test]
    fn test_decode_ping() {
        let data = b"2";
        let packet = EnginePacket::decode(data).unwrap();

        assert_eq!(packet.packet_type, EnginePacketType::Ping);
        assert!(packet.data.is_none());
    }

    #[test]
    fn test_decode_binary_with_length() {
        // Format: "<length>:<type><data>"
        // Type is sent as ASCII digit '4' in text format
        // Length includes packet type byte (1) + data length (3) = 4
        let data = b"4:\x04\0\x01\x02";
        let packet = EnginePacket::decode(data).unwrap();

        assert_eq!(packet.packet_type, EnginePacketType::Message);
        assert_eq!(packet.data, Some(Bytes::copy_from_slice(&[0u8, 1, 2][..])));
    }
}
