//! Minimal test to isolate the disconnect issue
//!
//! This test strips away all Leaf-specific logic to test raw Socket.IO behavior

use rust_socketio::asynchronous::ClientBuilder;
use futures_util::future::FutureExt;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_raw_socketio_connect_disconnect() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Test 1: Raw Socket.IO connect/disconnect with no handlers");

    let url = "http://leaf-server:5530";

    // Build client with minimal config
    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected successfully");
    println!("   Socket connected");

    // Give it a moment to stabilize
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
        }
        Err(e) => {
            println!("❌ Disconnect failed with error:");
            println!("   Error type: {:?}", std::any::type_name::<rust_socketio::Error>());
            println!("   Error: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    }

    println!("✅ Test completed");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_socketio_with_event_handler() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Test 2: Socket.IO with event handler (like Leaf client)");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .on("error", |err, _socket| {
            futures::future::ready({
                eprintln!("Socket error event: {:?}", err);
            })
            .boxed()
        })
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected with error handler");

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔌 Attempting disconnect...");
    match socket.disconnect().await {
        Ok(_) => {
            println!("✅ Disconnect succeeded");
        }
        Err(e) => {
            println!("❌ Disconnect failed:");
            println!("   {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_multiple_connect_disconnect_cycles() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Test 3: Multiple connect/disconnect cycles");

    let url = "http://leaf-server:5530";

    for i in 1..=3 {
        println!("\n🔄 Cycle {} of 3", i);

        let socket = ClientBuilder::new(url)
            .connect()
            .await
            .map_err(|e| format!("Connection failed on cycle {}: {:?}", i, e))?;

        println!("✅ Cycle {}: Connected", i);
        tokio::time::sleep(Duration::from_millis(50)).await;

        match socket.disconnect().await {
            Ok(_) => println!("✅ Cycle {}: Disconnect succeeded", i),
            Err(e) => {
                println!("❌ Cycle {}: Disconnect failed: {:?}", i, e);
                // Continue to next cycle to see if it's consistent
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n✅ All cycles completed");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_disconnect_without_explicit_call() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Test 4: Let connection drop without explicit disconnect");

    let url = "http://leaf-server:5530";

    {
        let socket = ClientBuilder::new(url)
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {:?}", e))?;

        println!("✅ Connected, socket will go out of scope now");
        // No explicit disconnect - let Drop handle it
    }

    println!("✅ Socket dropped");
    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("✅ Test completed (no explicit disconnect)");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_double_disconnect() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Test 5: Call disconnect twice (should handle gracefully)");

    let url = "http://leaf-server:5530";

    let socket = ClientBuilder::new(url)
        .connect()
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    println!("✅ Connected");

    tokio::time::sleep(Duration::from_millis(50)).await;

    println!("🔌 First disconnect...");
    match socket.disconnect().await {
        Ok(_) => println!("✅ First disconnect succeeded"),
        Err(e) => println!("❌ First disconnect failed: {:?}", e),
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    println!("🔌 Second disconnect (should handle gracefully)...");
    match socket.disconnect().await {
        Ok(_) => println!("✅ Second disconnect succeeded (unexpected!)"),
        Err(e) => println!("⚠️  Second disconnect failed (expected): {:?}", e),
    }

    println!("✅ Test completed");
    Ok(())
}
