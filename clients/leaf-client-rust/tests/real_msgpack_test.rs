//! Test using msgpack parser with actual Leaf server event

use rust_socketio::{asynchronous::ClientBuilder, Payload};
use socketioxide_parser_msgpack::MsgPackParser;
use socketioxide_core::parser::Parse;
use socketioxide_core::Value;
use futures_util::future::FutureExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_module_upload_with_msgpack_parser() {
    println!("🧪 Testing: Real module/upload with msgpack parser");

    let url = "http://leaf-server:5530";
    let parser = MsgPackParser;

    // Connect
    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .expect("Connection failed");

    println!("✅ Connected");

    // Create a simple module (CBOR encoded via ciborium)
    let test_module = serde_json::json!({
        "type_name": "BasicModule",
        "extra": {
            "name": "test",
            "version": "1.0.0"
        }
    });

    // Encode as CBOR first (what the server expects at application layer)
    let mut cbor_data = Vec::new();
    ciborium::ser::into_writer(&test_module, &mut cbor_data)
        .expect("CBOR encode failed");

    println!("📦 CBOR encoded: {} bytes", cbor_data.len());

    // Now encode the CBOR data as msgpack (for Socket.IO transport layer)
    match parser.encode_value(&cbor_data, Some("module/upload")) {
        Ok(Value::Bytes(msgpack_bytes)) => {
            println!("📦 Msgpack wrapped: {} bytes", msgpack_bytes.len());
            println!("   First 15 bytes: {:?}", &msgpack_bytes[..msgpack_bytes.len().min(15)]);

            // Send it
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            let callback = move |payload: Payload, _client| {
                println!("📨 Callback invoked!");
                if let Payload::Binary(data) = payload {
                    println!("   Received {} bytes", data.len());
                    let _ = tx.send(data.to_vec());
                }
                futures::future::ready(()).boxed()
            };

            println!("📤 Emitting module/upload...");
            match socket.emit_with_ack(
                "module/upload",
                Payload::Binary(msgpack_bytes.to_vec().into()),
                Duration::from_secs(2),
                callback,
            ).await {
                Ok(_) => println!("✅ Emit successful, waiting for ack..."),
                Err(e) => println!("❌ Emit failed: {:?}", e),
            }

            // Wait for response
            match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
                Ok(Some(data)) => {
                    println!("✅ Got response: {} bytes", data.len());
                    println!("   First 10 bytes: {:?}", &data[..data.len().min(10)]);
                }
                Ok(None) => println!("❌ Channel closed"),
                Err(_) => println!("❌ Timeout waiting for ack"),
            }
        }
        Ok(other) => {
            println!("❌ Msgpack encode got non-bytes: {:?}", other);
        }
        Err(e) => {
            println!("❌ Msgpack encode error: {:?}", e);
        }
    }

    println!("🔌 Disconnecting...");
    let _ = socket.disconnect().await;
}
