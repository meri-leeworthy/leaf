// Connected space for event streaming

use crate::{Error, Event};
use leaf_client_rust::{Did, LeafClient, SubscriptionId};
use std::sync::Arc;
use tokio::sync::mpsc;

/// A connected space stream
///
/// Represents a connection to a Roomy space stream,
/// allowing event subscription and sending.
pub struct ConnectedSpace {
    /// Stream DID
    pub stream_did: String,
    /// Subscription ID
    pub subscription_id: Option<SubscriptionId>,
    /// Leaf client
    pub leaf: Arc<LeafClient>,
    /// Event receiver
    pub event_rx: mpsc::Receiver<Event>,
}

impl ConnectedSpace {
    /// Connect to an existing space stream
    pub async fn connect(leaf: Arc<LeafClient>, stream_did: String) -> Result<Self, Error> {
        use leaf_client_rust::LeafQuery;

        // Convert to Did type
        let did = Did::from(stream_did.clone());

        // Subscribe to all events from the stream
        let query = LeafQuery {
            name: "get_events".to_string(),
            params: std::collections::HashMap::new(),
            start: None,
            limit: None,
        };

        let (tx, rx) = mpsc::channel(100);
        let tx_clone = tx.clone();

        // Subscribe to events
        let subscription_id = leaf.subscribe_events(
            &did,
            &query,
            move |events_resp| {
                for row in events_resp.rows {
                    if let Some(leaf_client_rust::SqlValue::Blob { value }) = row.get("payload") {
                        if let Ok(event) = ciborium::de::from_reader::<Event, _>(std::io::Cursor::new(value)) {
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

        Ok(Self {
            stream_did,
            subscription_id: Some(subscription_id),
            leaf,
            event_rx: rx,
        })
    }

    /// Subscribe to events from this space
    pub async fn subscribe(&mut self) -> Result<mpsc::Receiver<Event>, Error> {
        // Already subscribed in connect(), just return a new receiver
        let (tx, rx) = mpsc::channel(100);

        // Resubscribe to get a new subscription
        use leaf_client_rust::LeafQuery;
        let did = Did::from(self.stream_did.clone());

        let query = LeafQuery {
            name: "get_events".to_string(),
            params: std::collections::HashMap::new(),
            start: None,
            limit: None,
        };

        let tx_clone = tx.clone();

        let subscription_id = self.leaf.subscribe_events(
            &did,
            &query,
            move |events_resp| {
                for row in events_resp.rows {
                    if let Some(leaf_client_rust::SqlValue::Blob { value }) = row.get("payload") {
                        if let Ok(event) = ciborium::de::from_reader::<Event, _>(std::io::Cursor::new(value)) {
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

        self.subscription_id = Some(subscription_id);

        Ok(rx)
    }

    /// Send an event to this space
    pub async fn send_event(&self, event: Event) -> Result<(), Error> {
        // Encode the event as CBOR
        let mut encoded = Vec::new();
        ciborium::ser::into_writer(&event, &mut encoded)
            .map_err(|e| Error::Codec(format!("Failed to encode event: {}", e)))?;

        let did = Did::from(self.stream_did.clone());
        self.leaf.send_events(&did, &[encoded]).await?;
        Ok(())
    }

    /// Get the next event from the stream
    pub async fn recv(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }

    /// Get a stream of events
    pub fn stream(&mut self) -> mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel(100);

        // Forward events to the new channel
        let mut current_rx = std::mem::replace(&mut self.event_rx, mpsc::channel(1).1);

        tokio::spawn(async move {
            while let Some(event) = current_rx.recv().await {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        rx
    }

    /// Get the stream DID
    pub fn stream_did(&self) -> &str {
        &self.stream_did
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&mut self) -> Result<bool, Error> {
        if let Some(sub_id) = self.subscription_id.take() {
            self.leaf.unsubscribe_events(&sub_id).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_creation() {
        // Placeholder test
        assert!(true);
    }
}
