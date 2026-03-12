//! socketio-rust: A parser-agnostic Socket.IO client for Rust
//!
//! This library provides a Socket.IO client implementation with support for
//! multiple parsers (JSON, MessagePack, CBOR), making it compatible with
//! servers using different encodings.
//!
//! # Features
//!
//! - **Parser support**: JSON, MessagePack, CBOR
//! - **Type-safe API**: Generic emit/ack operations
//! - **Async/await**: Built on tokio for async I/O
//! - **WebSocket transport**: Uses tokio-tungstenite
//!
//! # Example
//!
//! ```no_run
//! use socketio_rust::{SocketIoClient, Parser};
//! use socketio_rust::parser::JsonParser;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = SocketIoClient::connect("http://localhost:3000", JsonParser::new()).await?;
//!
//!     // Emit an event
//!     client.emit("greeting", serde_json::json!({"hello": "world"})).await?;
//!
//!     // Emit with acknowledgment
//!     let response = client.emit_with_ack::<serde_json::Value>("ping", serde_json::json!({})).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client_transport;
pub mod error;
pub mod engineio;
pub mod packet;
pub mod parser;
pub mod transport;

pub use client_transport::EngineTransport;
pub use error::{Error, Result};
pub use packet::{Packet, PacketType};
pub use parser::{ParseError, Parser};
pub use transport::Transport;

use bytes::Bytes;
use parser::Parser as ParserTrait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main Socket.IO client
pub struct SocketIoClient<P: ParserTrait> {
    parser: P,
    transport: Arc<Mutex<EngineTransport>>,
    namespace: String,
    next_ack_id: Arc<Mutex<i64>>,
    pending_acks: Arc<Mutex<std::collections::HashMap<i64, tokio::sync::oneshot::Sender<Bytes>>>>,
}

impl<P: ParserTrait> SocketIoClient<P> {
    /// Connect to a Socket.IO server
    ///
    /// # Arguments
    /// * `url` - Server URL (e.g., "http://localhost:3000")
    /// * `parser` - Parser implementation to use
    pub async fn connect(url: &str, parser: P) -> Result<Self> {
        let mut transport = EngineTransport::connect(url).await?;

        // Send Socket.IO CONNECT packet (wrapped in Engine.IO)
        let connect_packet = Packet::connect("/");
        let encoded = parser.encode(&connect_packet)?;
        transport.send(encoded).await?;

        Ok(Self {
            parser,
            transport: Arc::new(Mutex::new(transport)),
            namespace: "/".to_string(),
            next_ack_id: Arc::new(Mutex::new(0)),
            pending_acks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
    }

    /// Emit an event to the server
    ///
    /// # Arguments
    /// * `event` - Event name
    /// * `data` - Event data (must be serializable)
    pub async fn emit(&self, event: &str, data: serde_json::Value) -> Result<()> {
        let packet = Packet::event(event, Some(data), None);
        let encoded = self.parser.encode(&packet)?;
        self.transport.lock().await.send(encoded).await?;
        Ok(())
    }

    /// Emit an event and wait for acknowledgment
    ///
    /// # Type Parameters
    /// * `R` - Response type (must implement Deserialize)
    ///
    /// # Arguments
    /// * `event` - Event name
    /// * `data` - Event data (must be serializable)
    pub async fn emit_with_ack<R: serde::de::DeserializeOwned>(
        &self,
        event: &str,
        data: serde_json::Value,
    ) -> Result<R> {
        // Generate unique ack ID
        let ack_id = {
            let mut id = self.next_ack_id.lock().await;
            *id += 1;
            *id
        };

        // Create channel for response
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Register pending ack
        self.pending_acks.lock().await.insert(ack_id, tx);

        // Send packet
        let packet = Packet::event(event, Some(data), Some(ack_id));
        let encoded = self.parser.encode(&packet)?;
        self.transport.lock().await.send(encoded).await?;

        // Wait for response
        let response_bytes = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            rx,
        )
        .await
        .map_err(|_| Error::AckTimeout)?
        .map_err(|_| Error::AckCanceled)?;

        // Decode response
        let response_packet = self.parser.decode(&response_bytes)?;

        // Extract data from ack packet
        if let Some(data) = response_packet.data {
            let response: R = serde_json::from_value(data)
                .map_err(|e| Error::InvalidResponse(e.to_string()))?;
            Ok(response)
        } else {
            Err(Error::InvalidResponse("No data in ack packet".to_string()))
        }
    }

    /// Disconnect from the server
    pub async fn disconnect(self) -> Result<()> {
        let packet = Packet::disconnect(&self.namespace);
        let encoded = self.parser.encode(&packet)?;
        let mut transport = self.transport.lock().await;
        transport.send(encoded).await?;
        transport.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_packet() {
        let packet = Packet::event("test", Some(serde_json::json!({"key": "value"})), None);
        assert_eq!(packet.packet_type, PacketType::Event);
    }
}
