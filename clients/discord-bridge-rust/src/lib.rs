// Discord Bridge for Roomy
//
// Bidirectional synchronization between Discord guilds and Roomy spaces.

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

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
