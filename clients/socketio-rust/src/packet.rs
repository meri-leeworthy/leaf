//! Socket.IO packet types and structures
//!
//! This module defines the core packet types used in Socket.IO communication.

use serde::{Deserialize, Serialize};

/// Socket.IO packet types according to the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// Connect (0)
    Connect = 0,
    /// Disconnect (1)
    Disconnect = 1,
    /// Event (2)
    Event = 2,
    /// Ack (3)
    Ack = 3,
    /// Error (4)
    Error = 4,
    /// Binary Event (5)
    BinaryEvent = 5,
    /// Binary Ack (6)
    BinaryAck = 6,
}

impl Serialize for PacketType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for PacketType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        PacketType::from_u8(value).ok_or_else(|| {
            serde::de::Error::custom(format!("invalid packet type: {}", value))
        })
    }
}

impl PacketType {
    /// Convert from u8, returning None if invalid
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PacketType::Connect),
            1 => Some(PacketType::Disconnect),
            2 => Some(PacketType::Event),
            3 => Some(PacketType::Ack),
            4 => Some(PacketType::Error),
            5 => Some(PacketType::BinaryEvent),
            6 => Some(PacketType::BinaryAck),
            _ => None,
        }
    }
}

/// A Socket.IO packet
///
/// Packets are the fundamental unit of communication in Socket.IO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    /// Packet type
    #[serde(rename = "type")]
    pub packet_type: PacketType,

    /// Namespace (e.g., "/", "/chat")
    #[serde(default = "default_namespace")]
    pub nsp: String,

    /// Optional data payload
    /// For Event packets, this contains the event data
    /// For BinaryEvent, this contains the event name as a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Packet ID for acknowledgments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// Number of binary attachments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_count: Option<u8>,

    /// Binary attachments (sent separately)
    #[serde(skip_serializing)]
    #[serde(default = "default_attachments")]
    pub attachments: Vec<Vec<u8>>,
}

fn default_attachments() -> Vec<Vec<u8>> {
    Vec::new()
}

fn default_namespace() -> String {
    "/".to_string()
}

impl Packet {
    /// Create a CONNECT packet
    pub fn connect(nsp: &str) -> Self {
        Packet {
            packet_type: PacketType::Connect,
            nsp: nsp.to_string(),
            data: None,
            id: None,
            binary_count: None,
            attachments: Vec::new(),
        }
    }

    /// Create a DISCONNECT packet
    pub fn disconnect(nsp: &str) -> Self {
        Packet {
            packet_type: PacketType::Disconnect,
            nsp: nsp.to_string(),
            data: None,
            id: None,
            binary_count: None,
            attachments: Vec::new(),
        }
    }

    /// Create an EVENT packet
    pub fn event(event: &str, data: Option<serde_json::Value>, id: Option<i64>) -> Self {
        Packet {
            packet_type: PacketType::Event,
            nsp: "/".to_string(),
            data: Some(data.unwrap_or_else(|| {
                serde_json::json!([event])
            })),
            id,
            binary_count: None,
            attachments: Vec::new(),
        }
    }

    /// Create a BINARY_EVENT packet
    pub fn binary_event(
        event: &str,
        data: Vec<u8>,
        id: Option<i64>,
    ) -> Self {
        Packet {
            packet_type: PacketType::BinaryEvent,
            nsp: "/".to_string(),
            data: Some(serde_json::Value::String(event.to_string())),
            id,
            binary_count: Some(1),
            attachments: vec![data],
        }
    }

    /// Create an ACK packet
    pub fn ack(id: i64, data: serde_json::Value) -> Self {
        Packet {
            packet_type: PacketType::Ack,
            nsp: "/".to_string(),
            data: Some(data),
            id: Some(id),
            binary_count: None,
            attachments: Vec::new(),
        }
    }

    /// Create a BINARY_ACK packet
    pub fn binary_ack(id: i64, data: Vec<u8>) -> Self {
        Packet {
            packet_type: PacketType::BinaryAck,
            nsp: "/".to_string(),
            data: None,
            id: Some(id),
            binary_count: Some(1),
            attachments: vec![data],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_type_from_u8() {
        assert_eq!(PacketType::from_u8(0), Some(PacketType::Connect));
        assert_eq!(PacketType::from_u8(2), Some(PacketType::Event));
        assert_eq!(PacketType::from_u8(5), Some(PacketType::BinaryEvent));
        assert_eq!(PacketType::from_u8(99), None);
    }

    #[test]
    fn test_create_connect_packet() {
        let packet = Packet::connect("/");

        assert_eq!(packet.packet_type, PacketType::Connect);
        assert_eq!(packet.nsp, "/");
        assert!(packet.data.is_none());
        assert!(packet.id.is_none());
    }

    #[test]
    fn test_create_event_packet() {
        let packet = Packet::event("test", Some(serde_json::json!({"key":"value"})), Some(42));

        assert_eq!(packet.packet_type, PacketType::Event);
        assert_eq!(packet.id, Some(42));
        assert!(packet.data.is_some());
    }

    #[test]
    fn test_serialize_packet_to_json() {
        let packet = Packet::connect("/");

        let json = serde_json::to_string(&packet).unwrap();
        assert!(json.contains("\"type\":0"));
        assert!(json.contains("\"nsp\":\"/\""));
    }

    #[test]
    fn test_deserialize_packet_from_json() {
        let json = r#"{"type":0,"nsp":"/","data":null}"#;

        let packet: Packet = serde_json::from_str(json).unwrap();

        assert_eq!(packet.packet_type, PacketType::Connect);
        assert_eq!(packet.nsp, "/");
    }
}
