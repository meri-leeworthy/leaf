//! # Discord Bridge for Roomy
//!
//! This crate provides bidirectional synchronization between Discord guilds and Roomy spaces.
//!
//! ## Architecture
//!
//! The bridge consists of three main layers:
//!
//! - **Bot Layer** ([`bot`]): Discord bot client using Twilight library, handles WebSocket events
//!   and HTTP API calls
//! - **Services Layer** ([`services`]): Business logic for syncing messages, reactions, structures,
//!   and user profiles
//! - **Repository Layer** ([`repository`]): Persistent storage using Sled database for mapping
//!   Discord IDs to Roomy IDs
//!
//! ## Key Components
//!
//! - [`DiscordBot`]: Main bot client that connects to Discord and emits events
//! - [`BridgeOrchestrator`]: Coordinates synchronization between Discord and Roomy
//! - [`BridgeRepository`]: Manages ID mappings and sync metadata
//!
//! ## Data Flow
//!
//! 1. Discord events are received via WebSocket connection
//! 2. Events are processed by service layer to create corresponding Roomy events
//! 3. ID mappings are stored in local Sled database for correlation
//! 4. Roomy events are sent to the Leaf server
//! 5. Sync state is tracked for resumability

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

pub mod bot;
pub mod sync;
pub mod services;
pub mod repository;

// Re-export commonly used types
pub use bot::{DiscordBot, DiscordEvent};
pub use sync::{BridgeOrchestrator, Bridge};
pub use repository::BridgeRepository;

/// Result type alias for Discord bridge operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for Discord bridge
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Discord API errors
    #[error("Discord error: {0}")]
    Discord(String),

    /// Roomy SDK errors
    #[error("Roomy SDK error: {0}")]
    Roomy(#[from] roomy_sdk_rust::Error),

    /// Leaf client errors
    #[error("Leaf client error: {0}")]
    LeafClient(#[from] leaf_client_rust::error::LeafClientError),

    /// Repository errors
    #[error("Repository error: {0}")]
    Repository(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Event encoding/decoding errors
    #[error("Codec error: {0}")]
    Codec(String),

    /// Rate limited
    #[error("Rate limited: retry after {0:?}")]
    RateLimited(std::time::Duration),

    /// Missing mapping
    #[error("Missing mapping: {0}")]
    MissingMapping(String),

    /// Invalid mapping (e.g., expected message ULID but got DID)
    #[error("Invalid mapping: {0}")]
    InvalidMapping(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Task join errors
    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    /// Twilight HTTP errors
    #[error("HTTP error: {0}")]
    Http(#[from] twilight_http::Error),

    /// Twilight response body errors
    #[error("Response error: {0}")]
    ResponseBody(#[from] twilight_http::response::DeserializeBodyError),

    /// Sled errors
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
