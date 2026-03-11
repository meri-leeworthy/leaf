//! Test immediate emit after connection

use leaf_client_rust::{LeafClient, types::*};
use std::collections::HashMap;

#[tokio::test]
#[ignore]
async fn test_immediate_emit_after_connect() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing immediate emit after connection...");

    // Connect
    let client = LeafClient::connect(
        "http://leaf-server:5530",
        None::<fn() -> futures::future::Ready<Result<String, leaf_client_rust::LeafClientError>>>,
    ).await?;
    println!("✅ Connected");

    // IMMEDIATELY upload - no delay
    let mut extra = HashMap::new();
    extra.insert("name".to_string(), serde_json::json!("test"));
    extra.insert("version".to_string(), serde_json::json!("1.0.0"));

    let module = ModuleCodec {
        type_name: "BasicModule".to_string(),
        extra,
    };

    println!("📤 Uploading module...");
    match tokio::time::timeout(std::time::Duration::from_millis(50), client.upload_module(&module)).await {
        Ok(Ok(result)) => {
            println!("✅ Upload succeeded! CID: {}", result.module_cid.link);
        }
        Ok(Err(e)) => {
            println!("❌ Upload failed: {:?}", e);
        }
        Err(_) => {
            println!("❌ Upload timed out after 50ms");
        }
    }

    println!("🔌 Disconnecting...");
    let _ = client.disconnect().await;
    println!("✅ Done");

    Ok(())
}
