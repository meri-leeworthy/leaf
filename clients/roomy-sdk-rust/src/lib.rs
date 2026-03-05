// Roomy SDK for Rust
//
// This SDK provides a Rust interface for interacting with Roomy,
// a federated collaboration platform built on AT Protocol.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

pub mod atproto;
pub mod client;
pub mod connection;
pub mod events;

// Re-export commonly used types
pub use atproto::RoomyAtpClient;
pub use client::{RoomyClient, RoomyClientConfig};
pub use connection::ConnectedSpace;
pub use events::{Event, EventType};

/// Result type alias for Roomy SDK operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for Roomy SDK
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// ATProto-related errors
    #[error("ATProto error: {0}")]
    Atproto(String),

    /// Leaf client errors
    #[error("Leaf error: {0}")]
    Leaf(#[from] leaf_client_rust::LeafClientError),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Event encoding/decoding errors
    #[error("Event codec error: {0}")]
    Codec(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid DID
    #[error("Invalid DID: {0}")]
    InvalidDid(String),

    /// Invalid handle
    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    /// Not connected
    #[error("Not connected to space")]
    NotConnected,

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
