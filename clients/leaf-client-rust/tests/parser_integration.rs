//! Test using socketioxide-parser-msgpack with rust_socketio

use rust_socketio::{asynchronous::ClientBuilder, Payload};
use socketioxide_parser_msgpack::MsgPackParser;
use socketioxide_core::parser::Parse;
use socketioxide_core::Value;
use futures_util::future::FutureExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_send_msgpack_encoded_data() {
    println!("🧪 Testing: Send msgpack-encoded data via rust_socketio");

    let url = "http://leaf-server:5530";
    let parser = MsgPackParser;

    // Connect
    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .expect("Connection failed");

    println!("✅ Connected");

    // Encode some test data as msgpack (simulating CBOR data)
    let test_cbor = vec![1, 2, 3, 4, 5];

    match parser.encode_value(&test_cbor, Some("test_event")) {
        Ok(Value::Bytes(msgpack_bytes)) => {
            println!("📦 Encoded CBOR as msgpack: {} bytes", msgpack_bytes.len());

            // Try to send it
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            let callback = move |payload: Payload, _client| {
                if let Payload::Binary(data) = payload {
                    println!("📨 Received {} bytes", data.len());
                    let _ = tx.send(data.to_vec());
                }
                futures::future::ready(()).boxed()
            };

            match socket.emit_with_ack(
                "test",
                Payload::Binary(msgpack_bytes.to_vec().into()),
                Duration::from_secs(2),
                callback,
            ).await {
                Ok(_) => println!("✅ Emit successful"),
                Err(e) => println!("❌ Emit failed: {:?}", e),
            }

            // Wait for response
            match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
                Ok(Some(data)) => {
                    println!("✅ Got response: {} bytes", data.len());
                    // Try to decode it
                    let mut value = Value::Bytes(data.into());
                    match parser.decode_value::<Vec<u8>>(&mut value, true) {
                        Ok(decoded) => println!("✅ Decoded: {:?}", decoded),
                        Err(e) => println!("⚠️  Decode error: {:?}", e),
                    }
                }
                Ok(None) => println!("❌ Channel closed"),
                Err(_) => println!("❌ Timeout"),
            }
        }
        Ok(other) => {
            println!("❌ Not bytes: {:?}", other);
        }
        Err(e) => {
            println!("❌ Encode error: {:?}", e);
        }
    }

    // Disconnect
    let _ = socket.disconnect().await;
}
