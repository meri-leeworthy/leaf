//! Test basic server connectivity

use leaf_client_rust::LeafClient;
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn test_server_is_alive() {
    println!("🔌 Testing if server is alive...");

    match tokio::time::timeout(Duration::from_secs(3), async {
        LeafClient::connect(
            "http://leaf-server:5530",
            None::<fn() -> futures::future::Ready<Result<String, leaf_client_rust::LeafClientError>>>,
        ).await
    }).await {
        Ok(Ok(client)) => {
            println!("✅ Connected to server successfully!");
            // Try to disconnect immediately
            let _ = client.disconnect().await;
            println!("✅ Disconnected");
        }
        Ok(Err(e)) => {
            println!("❌ Connection failed: {:?}", e);
            panic!("Connection failed");
        }
        Err(_) => {
            println!("❌ Connection timed out after 3 seconds");
            panic!("Connection timeout");
        }
    }
}
