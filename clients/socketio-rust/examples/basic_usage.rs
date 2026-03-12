//! Basic usage example for socketio-rust
//!
//! This example demonstrates the intended API for the socketio-rust client.
//! Note: This won't actually connect until Engine.IO handshake is implemented.

use socketio_rust::{Parser, parser::JsonParser};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 socketio-rust Example");
    println!("========================\n");

    // Create a parser
    let parser = JsonParser::new();
    println!("✅ Created JSON parser");

    // Note: The following won't work yet because we need Engine.IO handshake
    // But this demonstrates the intended API

    println!("\n📝 Intended API Usage:");
    println!("   let client = SocketIoClient::connect(\"http://localhost:3000\", parser).await?;");
    println!("   client.emit(\"greeting\", json!({{\"hello\": \"world\"}})).await?;");
    println!("   let response: serde_json::Value = client.emit_with_ack(\"ping\", json!({{}})).await?;");
    println!("   client.disconnect().await?;\n");

    // For now, let's just demonstrate packet creation
    use socketio_rust::Packet;

    let packet = Packet::connect("/");
    println!("✅ Created CONNECT packet: {:?}", packet);

    let event_packet = Packet::event(
        "test_event",
        Some(serde_json::json!({"key": "value"})),
        Some(42),
    );
    println!("✅ Created EVENT packet: {:?}", event_packet);

    // Encode and decode to demonstrate parser works
    let encoded = parser.encode(&packet)?;
    println!("✅ Encoded packet: {} bytes", encoded.len());

    let decoded = parser.decode(&encoded)?;
    println!("✅ Decoded packet: type={:?}, nsp={}", decoded.packet_type, decoded.nsp);

    println!("\n✨ Example completed successfully!");
    println!("\n📚 Next steps:");
    println!("   1. Implement Engine.IO handshake");
    println!("   2. Add MessagePack parser for Leaf server compatibility");
    println!("   3. Test with real Socket.IO/Leaf server");

    Ok(())
}
