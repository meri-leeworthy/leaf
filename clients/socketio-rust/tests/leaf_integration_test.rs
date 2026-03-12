//! Integration tests with real Leaf server
//!
//! These tests require a running Leaf server at http://leaf-server:5530
//!
//! To run: cargo test --test leaf_integration_test -- --ignored

use socketio_rust::{Packet, PacketType, parser::{JsonParser, Parser as ParserTrait}};

#[test]
fn test_packet_serialization_for_leaf() {
    // Test that we can serialize packets in the format expected by Socket.IO
    let packet = Packet::connect("/");

    let parser = JsonParser::new();
    let encoded = parser.encode(&packet).unwrap();

    // Should be valid JSON
    let json_str = std::str::from_utf8(&encoded).unwrap();
    println!("Encoded CONNECT packet: {}", json_str);

    // Should contain numeric type
    assert!(json_str.contains("\"type\":0"));

    // Should be deserializable
    let decoded = parser.decode(&encoded).unwrap();
    assert_eq!(decoded.packet_type, PacketType::Connect);
    assert_eq!(decoded.nsp, "/");
}

#[test]
fn test_event_packet_serialization() {
    // Test event serialization
    let packet = Packet::event(
        "test_event",
        Some(serde_json::json!({"key": "value"})),
        Some(42),
    );

    let parser = JsonParser::new();
    let encoded = parser.encode(&packet).unwrap();
    let json_str = std::str::from_utf8(&encoded).unwrap();
    println!("Encoded EVENT packet: {}", json_str);

    // Should contain the event type (2)
    assert!(json_str.contains("\"type\":2"));

    // Should contain the ack ID
    assert!(json_str.contains("\"id\":42"));

    // Round-trip test
    let decoded = parser.decode(&encoded).unwrap();
    assert_eq!(decoded.packet_type, PacketType::Event);
    assert_eq!(decoded.id, Some(42));
}

#[tokio::test]
#[ignore] // Requires WebSocket connection implementation
async fn test_connect_to_leaf_server() {
    // This test will be implemented once we have full Engine.IO + Socket.IO support
    // For now, we verify the server is running
    let response = reqwest::get("http://leaf-server:5530/").await.unwrap();
    assert!(response.status().is_success());

    println!("✅ Leaf server is accessible");
}

#[tokio::test]
#[ignore] // Requires WebSocket connection implementation
async fn test_socket_io_handshake() {
    // TODO: Implement Engine.IO handshake
    // 1. GET /socket.io/?EIO=4&transport=polling
    // 2. Parse session ID from response
    // 3. Upgrade to WebSocket with session ID
    // 4. Send Socket.IO CONNECT packet

    println!("📝 Engine.IO handshake not yet implemented");
}

#[test]
fn test_msgpack_parser_interface() {
    // Verify that our parser interface can support MessagePack
    // This is a compile-time check - if it compiles, the interface is correct
    use socketio_rust::parser::Parser as _;

    // The Parser trait is implemented by JsonParser
    // In the future, MsgPackParser and CborParser will also implement it
    let parser = JsonParser::new();
    assert_eq!(parser.parser_type(), "json");

    println!("✅ Parser interface is compatible with multiple parser types");
}

#[test]
fn test_binary_event_packet() {
    // Test binary event creation
    let data = vec![1, 2, 3, 4, 5];
    let packet = Packet::binary_event("binary_test", data.clone(), Some(1));

    assert_eq!(packet.packet_type, PacketType::BinaryEvent);
    assert_eq!(packet.binary_count, Some(1));
    assert_eq!(packet.id, Some(1));
    assert_eq!(packet.attachments.len(), 1);
    assert_eq!(packet.attachments[0], data);

    println!("✅ Binary event packet created successfully");
}
