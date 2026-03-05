//! Integration tests for the Leaf client
//!
//! These tests require a running Leaf server. Start the server with:
//! ```bash
//! cd leaf
//! cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token
//! ```
//!
//! Then run tests with:
//! ```bash
//! cargo test -p leaf-client-rust --test integration_test
//! ```

use leaf_client_rust::{LeafClient, types::*};
use std::collections::HashMap;
use std::time::Duration;

/// Server URL for testing
const TEST_SERVER_URL: &str = "http://leaf-server:5530";

/// Unsafe auth token for testing (when server started with --unsafe-auth-token)
const TEST_AUTH_TOKEN: &str = "test-token";

/// Test DID for creating streams
const TEST_STREAM_DID: &str = "did:web:localhost:test";

/// Helper function to create a test client
async fn create_test_client() -> Result<LeafClient, Box<dyn std::error::Error>> {
    let client = LeafClient::connect(
        TEST_SERVER_URL,
        None::<fn() -> futures::future::Ready<Result<String, leaf_client_rust::LeafClientError>>>,
    ).await?;
    Ok(client)
}

/// Helper function to create a simple module for testing
fn create_test_module() -> ModuleCodec {
    let mut extra = HashMap::new();
    extra.insert("name".to_string(), serde_json::json!("test_module"));
    extra.insert("version".to_string(), serde_json::json!("1.0.0"));

    ModuleCodec {
        type_name: "BasicModule".to_string(),
        extra,
    }
}

#[tokio::test]
#[ignore] // Ignore by default, requires running server
async fn test_client_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Testing client connection...");

    let client = create_test_client().await?;

    println!("✅ Successfully connected to Leaf server at {}", TEST_SERVER_URL);

    // Give connection a moment to stabilize
    tokio::time::sleep(Duration::from_millis(500)).await;

    client.disconnect().await?;
    println!("✅ Successfully disconnected");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_upload_and_check_module() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Testing module upload...");

    let client = create_test_client().await?;

    // Create a test module
    let module = create_test_module();

    // Upload the module
    let upload_result = client.upload_module(&module).await?;
    println!("✅ Module uploaded with CID: {}", upload_result.module_cid.link);

    // Check if module exists
    let module_cid = upload_result.module_cid.link.clone();
    let exists = client.has_module(&module_cid).await?;
    println!("✅ Module exists check: {}", exists);
    assert!(exists, "Module should exist after upload");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_create_stream() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Testing stream creation...");

    let client = create_test_client().await?;

    // First, upload a module
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let module_cid = upload_result.module_cid.link;

    // Create a stream with the module
    let stream_did = client.create_stream(&module_cid).await?;
    println!("✅ Stream created with DID: {}", stream_did.as_str());

    // Verify stream info
    let stream_module_cid = client.stream_info(&stream_did).await?;
    assert_eq!(stream_module_cid, Some(module_cid));
    println!("✅ Stream info matches uploaded module CID");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_query_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing query execution...");

    let client = create_test_client().await?;

    // Create a stream first
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Create a test query
    let query = LeafQuery {
        name: "test_query".to_string(),
        params: HashMap::new(),
        start: Some(0),
        limit: Some(10),
    };

    // Execute query (this will fail if the query doesn't exist, but tests the protocol)
    match client.query(&stream_did, &query).await {
        Ok(rows) => {
            println!("✅ Query executed successfully, returned {} rows", rows.len());
        }
        Err(e) => {
            println!("⚠️  Query returned error (expected if query not defined): {:?}", e);
            // This is okay - we're testing the protocol, not the query itself
        }
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_send_events() -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Testing event sending...");

    let client = create_test_client().await?;

    // Create a stream
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Create some test events (CBOR-encoded data)
    let events = vec![
        b"{\"test\": \"event1\"}".to_vec(),
        b"{\"test\": \"event2\"}".to_vec(),
    ];

    // Send events
    client.send_events(&stream_did, &events).await?;
    println!("✅ Events sent successfully");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_send_state_events() -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Testing state event sending...");

    let client = create_test_client().await?;

    // Create a stream
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Create test state events
    let state_events = vec![
        b"{\"type\": \"read\", \"position\": 100}".to_vec(),
    ];

    // Send state events
    client.send_state_events(&stream_did, &state_events).await?;
    println!("✅ State events sent successfully");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_update_stream_module() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Testing stream module update...");

    let client = create_test_client().await?;

    // Create initial stream
    let module1 = create_test_module();
    let upload1 = client.upload_module(&module1).await?;
    let stream_did = client.create_stream(&upload1.module_cid.link).await?;

    // Upload a second module
    let mut module2_extra = HashMap::new();
    module2_extra.insert("name".to_string(), serde_json::json!("test_module_v2"));
    module2_extra.insert("version".to_string(), serde_json::json!("2.0.0"));

    let module2 = ModuleCodec {
        type_name: "BasicModule".to_string(),
        extra: module2_extra,
    };
    let upload2 = client.upload_module(&module2).await?;

    // Update stream to use new module
    client.update_module(&stream_did, &upload2.module_cid.link).await?;
    println!("✅ Stream module updated successfully");

    // Verify the update
    let stream_info = client.stream_info(&stream_did).await?;
    assert_eq!(stream_info, Some(upload2.module_cid.link));
    println!("✅ Stream info reflects updated module");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_set_handle() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏷️  Testing handle setting...");

    let client = create_test_client().await?;

    // Create a stream
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Set a handle
    let test_handle = Some("test-handle");
    client.set_handle(&stream_did, test_handle).await?;
    println!("✅ Handle set to: {:?}", test_handle);

    // Clear the handle
    client.set_handle(&stream_did, None).await?;
    println!("✅ Handle cleared");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_clear_state() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 Testing state clearing...");

    let client = create_test_client().await?;

    // Create a stream
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Send some state events
    let state_events = vec![b"{\"test\": \"data\"}".to_vec()];
    client.send_state_events(&stream_did, &state_events).await?;

    // Clear state
    client.clear_state(&stream_did).await?;
    println!("✅ State cleared successfully");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_event_subscription() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔔 Testing event subscription...");

    let client = create_test_client().await?;

    // Create a stream
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let stream_did = client.create_stream(&upload_result.module_cid.link).await?;

    // Create a query to subscribe to
    let query = LeafQuery {
        name: "test_query".to_string(),
        params: HashMap::new(),
        start: Some(0),
        limit: Some(10),
    };

    // Use a channel to capture subscription events
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let subscription_id = client.subscribe_events(&stream_did, &query, move |resp| {
        let _ = tx.send(resp);
        Ok(())
    }).await?;
    println!("✅ Subscribed with ID: {:?}", subscription_id.as_str());

    // Wait a bit for any initial events
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check if we received any events
    match rx.try_recv() {
        Ok(resp) => {
            println!("✅ Received subscription response with {} rows, has_more: {}",
                     resp.rows.len(), resp.has_more);
        }
        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
            println!("⚠️  No events received (expected if query has no results)");
        }
        Err(e) => {
            println!("❌ Error receiving events: {:?}", e);
        }
    }

    // Unsubscribe
    let was_subscribed = client.unsubscribe_events(&subscription_id).await?;
    println!("✅ Unsubscribed (was_subscribed: {})", was_subscribed);
    assert!(was_subscribed, "Should have been subscribed");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("❌ Testing error handling...");

    let client = create_test_client().await?;

    // Try to query a non-existent stream
    let fake_did = Did::new("did:web:nonexistent:stream".to_string())?;
    let query = LeafQuery {
        name: "test".to_string(),
        params: HashMap::new(),
        start: None,
        limit: None,
    };

    match client.query(&fake_did, &query).await {
        Ok(_) => {
            println!("⚠️  Query succeeded unexpectedly");
        }
        Err(e) => {
            println!("✅ Got expected error: {:?}", e);
        }
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Testing concurrent operations...");

    let client = std::sync::Arc::new(create_test_client().await?);

    // Create multiple streams concurrently
    let module = create_test_module();
    let upload_result = client.upload_module(&module).await?;
    let module_cid = upload_result.module_cid.link;

    let mut handles = vec![];
    for _i in 0..5 {
        let client_ref = client.clone();
        let cid = module_cid.clone();
        let handle = tokio::spawn(async move {
            client_ref.create_stream(&cid).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await??;
        results.push(result);
    }

    println!("✅ Created {} streams concurrently", results.len());

    // Extract the client from Arc to disconnect
    match std::sync::Arc::try_unwrap(client) {
        Ok(client) => client.disconnect().await?,
        Err(_) => println!("⚠️  Could not fully disconnect client (Arc still has references)"),
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_did_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🆔 Testing DID validation...");

    // Valid DIDs
    let valid_dids = vec![
        "did:web:example.com",
        "did:plc:abcd1234",
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    ];

    for did_str in valid_dids {
        let did = Did::new(did_str.to_string())?;
        println!("✅ Valid DID: {}", did.as_str());
    }

    // Invalid DID
    let invalid_did = Did::new("not-a-did".to_string());
    assert!(invalid_did.is_err(), "Should reject invalid DID");
    println!("✅ Correctly rejected invalid DID");

    Ok(())
}
