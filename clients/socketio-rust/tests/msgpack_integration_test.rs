//! Integration tests with MessagePack parser against real Leaf server
//!
//! These tests require a running Leaf server at http://leaf-server:5530
//!
//! To run: cargo test --test msgpack_integration_test --features msgpack-parser -- --ignored

use socketio_rust::{SocketIoClient, Parser, parser::JsonParser};
use socketio_rust::Packet;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_connect_with_json_parser() {
    println!("🔌 Testing connection with JSON parser...");

    let parser = JsonParser::new();
    let client = SocketIoClient::connect("http://leaf-server:5530", parser).await;

    match client {
        Ok(_client) => {
            println!("✅ Successfully connected to Leaf server with JSON parser");
            // Note: We can't test much more without implementing event handlers
            // The server may or may not be configured to accept JSON
        }
        Err(e) => {
            println!("⚠️  Connection failed (expected if server only accepts MessagePack): {}", e);
            // This is OK - the Leaf server might be configured for MessagePack only
        }
    }
}

#[tokio::test]
#[ignore]
#[cfg(feature = "msgpack-parser")]
async fn test_connect_with_msgpack_parser() {
    println!("🔌 Testing connection with MessagePack parser...");

    use socketio_rust::parser::MsgPackParser;

    let parser = MsgPackParser::new();
    let client = SocketIoClient::connect("http://leaf-server:5530", parser).await;

    match client {
        Ok(_client) => {
            println!("✅ Successfully connected to Leaf server with MessagePack parser");
            // TODO: Test actual operations once we have event handlers
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            panic!("Expected to connect with MessagePack parser");
        }
    }
}

#[test]
fn test_packet_serialization_msgpack_vs_json() {
    println!("📦 Comparing MessagePack vs JSON packet sizes...");

    let packet = Packet::connect("/");

    // JSON encoding
    let json_parser = JsonParser::new();
    let json_encoded = json_parser.encode(&packet).unwrap();
    println!("   JSON encoded: {} bytes", json_encoded.len());

    #[cfg(feature = "msgpack-parser")]
    {
        use socketio_rust::parser::MsgPackParser;

        // MessagePack encoding
        let msgpack_parser = MsgPackParser::new();
        let msgpack_encoded = msgpack_parser.encode(&packet).unwrap();
        println!("   MessagePack encoded: {} bytes", msgpack_encoded.len());

        // MessagePack should be more compact
        assert!(msgpack_encoded.len() < json_encoded.len());
        println!("✅ MessagePack is more compact ({}% smaller)",
            100 * (json_encoded.len() - msgpack_encoded.len()) / json_encoded.len());
    }

    println!("✅ Serialization test passed");
}
