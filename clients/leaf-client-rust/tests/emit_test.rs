//! Test emit_with_ack behavior

use rust_socketio::{asynchronous::ClientBuilder, Payload};
use futures_util::future::FutureExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_emit_with_ack_immediate() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔔 Test emit_with_ack immediately after connection");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .connect()
        .await?;

    println!("✅ Connected");

    // Try to emit immediately
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    let callback = move |payload: Payload, _client| {
        if let Payload::Binary(data) = payload {
            println!("📨 Received ack: {} bytes", data.len());
            let _ = tx.send(data.to_vec());
        }
        futures::future::ready(()).boxed()
    };

    println!("📤 Emitting with ack...");
    match socket.emit_with_ack(
        "test",
        Payload::Binary(vec![1, 2, 3].into()),
        Duration::from_secs(5),
        callback,
    ).await {
        Ok(_) => println!("✅ Emit successful"),
        Err(e) => println!("❌ Emit failed: {:?}", e),
    }

    println!("⏳ Waiting for response...");
    match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
        Ok(Some(data)) => {
            println!("✅ Got response: {} bytes", data.len());
            Ok(())
        }
        Ok(None) => {
            println!("❌ Channel closed");
            Err("Channel closed".into())
        }
        Err(_) => {
            println!("❌ Timeout waiting for response");
            Err("Timeout".into())
        }
    }
}
