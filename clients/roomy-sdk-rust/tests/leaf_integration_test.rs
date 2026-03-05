// Integration test for Leaf server connection

use roomy_sdk_rust::{RoomyClient, RoomyClientConfig};

/// Test connecting to the Leaf server
///
/// This test requires a running Leaf server at http://localhost:5530
/// and valid ATProto credentials in the .env file.
#[tokio::test]
#[ignore]  // Run with cargo test -- --ignored
async fn test_leaf_connection() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get credentials from environment
    let handle = std::env::var("VITE_TESTING_HANDLE")
        .expect("VITE_TESTING_HANDLE must be set");
    let password = std::env::var("VITE_TESTING_APP_PASSWORD")
        .expect("VITE_TESTING_APP_PASSWORD must be set");

    // Create client config
    let config = RoomyClientConfig {
        atproto_url: "https://bsky.social".to_string(),
        identifier: handle,
        password,
        leaf_url: "http://localhost:5530".to_string(),
        leaf_did: "did:web:leaf.example.com".to_string(),
    };

    // Create client (authenticates with ATProto and connects to Leaf)
    let mut client = RoomyClient::create(config).await?;

    println!("✓ Connected to Leaf server");
    println!("  DID: {}", client.did().unwrap());

    // Connect to personal space
    let mut personal_space = client.connect_personal_space().await?;

    println!("✓ Connected to personal space");
    println!("  Stream DID: {}", personal_space.stream_did);

    // Wait for a few seconds to receive events
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Try to receive an event
    if let Some(event) = personal_space.recv().await {
        println!("✓ Received event: {:?}", event.id());
    } else {
        println!("  No events received yet (this is expected for a new stream)");
    }

    // Unsubscribe
    personal_space.unsubscribe().await?;
    println!("✓ Unsubscribed from personal space");

    Ok(())
}
