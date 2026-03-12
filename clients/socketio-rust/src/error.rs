//! Error types for socketio-rust

use crate::parser::ParseError;

/// Errors that can occur in the Socket.IO client
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Acknowledgment timeout")]
    AckTimeout,

    #[error("Acknowledgment canceled")]
    AckCanceled,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not connected")]
    NotConnected,
}

pub type Result<T> = std::result::Result<T, Error>;
