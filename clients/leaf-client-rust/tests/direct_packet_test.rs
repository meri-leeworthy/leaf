//! Test: Can we manually construct a Packet with msgpack data?

use rust_socketio::{asynchronous::{Client, ClientBuilder}, Payload, packet::Packet, packet::PacketId};
use bytes::Bytes;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_manually_construct_packet() {
    println!("🧪 Testing: Manually construct Packet with msgpack data");

    // Connect
    let client = ClientBuilder::new("http://leaf-server:5530")
        .connect()
        .await
        .expect("Connection failed");

    println!("✅ Connected");

    // Try to create a custom packet directly
    // The server expects msgpack, so let's create that
    let test_data = vec![1, 2, 3, 4, 5];

    // In rust_socketio, BinaryEvent packets have:
    // - data: JSON string with event name
    // - attachments: Vec<Bytes> with binary data

    let packet = Packet {
        packet_type: PacketId::BinaryEvent,
        nsp: "/".to_string(),
        data: Some("\"test\"".to_string()),  // Event name as JSON
        id: Some(1),  // Ack ID
        attachment_count: 1,
        attachments: Some(vec![Bytes::from(test_data)]),
    };

    println!("📦 Created manual packet: {:?}", packet);

    // Try to send it
    match client.send(packet).await {
        Ok(_) => println!("✅ Packet sent successfully"),
        Err(e) => println!("❌ Send failed: {:?}", e),
    }

    // Wait a bit for response
    tokio::time::sleep(Duration::from_millis(500)).await;

    let _ = client.disconnect().await;
}
