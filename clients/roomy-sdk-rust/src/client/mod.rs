// High-level Roomy client

use crate::atproto::RoomyAtpClient;
use crate::connection::ConnectedSpace;
use crate::Error;
use leaf_client_rust::LeafClient;

/// Configuration for creating a RoomyClient
#[derive(Debug, Clone)]
pub struct RoomyClientConfig {
    /// ATProto PDS URL (e.g., "https://bsky.social")
    pub atproto_url: String,

    /// User's DID or handle
    pub identifier: String,

    /// App password for authentication
    pub password: String,

    /// Leaf server URL
    pub leaf_url: String,

    /// Leaf server DID
    pub leaf_did: String,
}

/// High-level Roomy client
///
/// This client wraps both the ATProto client and Leaf client,
/// providing a unified interface for Roomy operations.
pub struct RoomyClient {
    atproto: RoomyAtpClient,
    leaf: std::sync::Arc<LeafClient>,
    personal_stream: Option<ConnectedSpace>,
}

impl RoomyClient {
    /// Create a new RoomyClient and authenticate
    ///
    /// This will:
    /// 1. Authenticate with ATProto using app password
    /// 2. Connect to the Leaf server
    /// 3. Return a ready-to-use client
    ///
    /// # Example
    /// ```no_run
    /// use roomy_sdk_rust::{RoomyClient, RoomyClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = RoomyClientConfig {
    ///         atproto_url: "https://bsky.social".to_string(),
    ///         identifier: "user.bsky.social".to_string(),
    ///         password: "password-xxx-xxx".to_string(),
    ///         leaf_url: "http://localhost:5530".to_string(),
    ///         leaf_did: "did:web:leaf.example.com".to_string(),
    ///     };
    ///
    ///     let client = RoomyClient::create(config).await?;
    ///     println!("Connected as DID: {}", client.did().unwrap());
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(config: RoomyClientConfig) -> Result<Self, Error> {
        // Authenticate with ATProto
        let mut atproto = RoomyAtpClient::new(&config.atproto_url);
        atproto.login(&config.identifier, &config.password).await?;

        // Connect to Leaf server without authenticator
        // (Leaf authentication will happen when subscribing to streams)
        type AuthFn = fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<String, leaf_client_rust::LeafClientError>> + Send>
        >;
        let leaf = std::sync::Arc::new(
            leaf_client_rust::LeafClient::connect::<AuthFn, _>(&config.leaf_url, None).await?
        );

        Ok(Self {
            atproto,
            leaf,
            personal_stream: None,
        })
    }

    /// Connect to the user's personal stream
    ///
    /// This will create or connect to the personal stream,
    /// which is required for most Roomy operations.
    ///
    /// The personal stream is stored at `space.roomy.space.personal.dev/self`
    /// in the user's ATProto repository.
    pub async fn connect_personal_space(&mut self) -> Result<ConnectedSpace, Error> {
        let did = self.did().ok_or(Error::Auth("Not authenticated".to_string()))?;

        // The personal stream DID is the user's DID
        let stream_did = leaf_client_rust::Did::from(did.to_string());

        // Subscribe to all events from the personal stream
        // Query: SELECT * FROM events ORDER BY idx
        let query = leaf_client_rust::LeafQuery {
            name: "get_events".to_string(),
            params: std::collections::HashMap::new(),
            start: None,
            limit: None,
        };

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Clone for the subscription callback
        let tx_clone = tx.clone();
        let leaf_clone = self.leaf.clone();

        // Subscribe to events
        let subscription_id = self.leaf.subscribe_events(
            &stream_did,
            &query,
            move |events_resp| {
                // Handle incoming events
                for row in events_resp.rows {
                    // Try to decode the event payload
                    if let Some(leaf_client_rust::SqlValue::Blob { value }) = row.get("payload") {
                        if let Ok(event) = ciborium::de::from_reader::<crate::events::Event, _>(std::io::Cursor::new(value)) {
                            // Send the event to the channel
                            if let Err(e) = tx_clone.blocking_send(event) {
                                tracing::error!("Failed to send event to channel: {}", e);
                            }
                        } else {
                            tracing::warn!("Failed to decode event from CBOR");
                        }
                    }
                }
                Ok(())
            }
        ).await?;

        tracing::info!("Subscribed to personal stream: {}", stream_did.as_str());

        Ok(ConnectedSpace {
            stream_did: stream_did.as_str().to_string(),
            subscription_id: Some(subscription_id),
            leaf: leaf_clone,
            event_rx: rx,
        })
    }

    /// Get a reference to the ATProto client
    pub fn atproto(&self) -> &RoomyAtpClient {
        &self.atproto
    }

    /// Get a reference to the Leaf client
    pub fn leaf(&self) -> &LeafClient {
        &self.leaf
    }

    /// Get the user's DID
    pub fn did(&self) -> Option<&str> {
        self.atproto.did()
    }

    /// Get the ATProto access token
    ///
    /// Note: This currently returns None
    pub fn access_token(&self) -> Option<String> {
        self.atproto.access_token()
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.atproto.is_authenticated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = RoomyClientConfig {
            atproto_url: "https://bsky.social".to_string(),
            identifier: "did:plc:test".to_string(),
            password: "test-password".to_string(),
            leaf_url: "https://leaf.example.com".to_string(),
            leaf_did: "did:web:leaf.example.com".to_string(),
        };

        assert_eq!(config.atproto_url, "https://bsky.social");
    }
}
