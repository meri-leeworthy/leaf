//! Transport-specific tests to isolate the disconnect issue
//!
//! Hypothesis: The issue is specific to WebSocket transport

use rust_socketio::asynchronous::{Client, ClientBuilder};
use rust_socketio::TransportType;
use futures_util::future::FutureExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_with_websocket_transport() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Test with WebSocket transport (explicit)");

    let url = "http://leaf-server:5530";

    // Force WebSocket transport
    let socket = ClientBuilder::new(url)
        .transport_type(TransportType::Websocket)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected via WebSocket");
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
            Ok(())
        }
        Err(e) => {
            println!("❌ Disconnect failed: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_with_polling_transport() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Test with polling transport (explicit)");

    let url = "http://leaf-server:5530";

    // Force polling transport
    let socket = ClientBuilder::new(url)
        .transport_type(TransportType::Polling)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected via polling");
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
            Ok(())
        }
        Err(e) => {
            println!("❌ Disconnect failed: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_with_delay_before_disconnect() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏱️  Test with longer delay before disconnect");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected, waiting 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
            Ok(())
        }
        Err(e) => {
            println!("❌ Disconnect failed: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_disconnect_with_open_packet() -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Test: Send data before disconnect");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .on("message", |payload, _client| {
            println!("Received message: {:?}", payload);
            futures::future::ready(()).boxed()
        })
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected");

    // Emit a ping to keep connection alive
    use rust_socketio::Payload;
    match socket.emit("ping", Payload::Text(vec![])).await {
        Ok(_) => println!("✅ Sent ping"),
        Err(e) => println!("⚠️  Ping failed: {:?}", e),
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
            Ok(())
        }
        Err(e) => {
            println!("❌ Disconnect failed: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_immediate_disconnect() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Test: Disconnect immediately after connection");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected, disconnecting immediately (no delay)");

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
            Ok(())
        }
        Err(e) => {
            println!("❌ Disconnect failed: {:?}", e);
            Err(Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}
