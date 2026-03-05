//! Bridge repository for ID mapping and state persistence

mod sled_impl;

pub use sled_impl::SledBridgeRepository;

use roomy_sdk_rust::Did;
use ulid::Ulid;

/// Discord ID (snowflake as string)
pub type DiscordId = String;

/// Roomy ID (ULID or DID)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoomyId {
    /// ULID for messages, rooms, etc.
    Ulid(Ulid),
    /// DID for users
    Did(Did),
}

impl RoomyId {
    /// Convert to string for storage
    pub fn to_key(&self) -> String {
        match self {
            Self::Ulid(u) => u.to_string(),
            Self::Did(d) => d.as_str().to_string(),
        }
    }

    /// Parse from string
    pub fn from_key(s: &str) -> Option<Self> {
        // Try ULID first
        if let Ok(ulid) = Ulid::from_string(s) {
            return Some(Self::Ulid(ulid));
        }
        // Try DID
        if s.starts_with("did:") {
            return Some(Self::Did(Did::new(s.to_string())));
        }
        None
    }
}

/// Trait for bridge repository operations
pub trait BridgeRepository: Send + Sync {
    /// Get Roomy ID for a Discord ID
    fn get_roomy_id(&self, discord_id: &DiscordId) -> Result<Option<RoomyId>, sled::Error>;

    /// Get Discord ID for a Roomy ID
    fn get_discord_id(&self, roomy_id: &RoomyId) -> Result<Option<DiscordId>, sled::Error>;

    /// Register a Discord ↔ Roomy mapping
    fn register_mapping(
        &self,
        discord_id: &DiscordId,
        roomy_id: &RoomyId,
    ) -> Result<(), sled::Error>;

    /// Get the cursor position for a space stream
    fn get_cursor(&self, space_did: &Did) -> Result<u64, sled::Error>;

    /// Set the cursor position for a space stream
    fn set_cursor(&self, space_did: &Did, idx: u64) -> Result<(), sled::Error>;

    /// Get the latest message ID for a Discord channel
    fn get_latest_message(&self, channel_id: &str) -> Result<Option<String>, sled::Error>;

    /// Set the latest message ID for a Discord channel
    fn set_latest_message(&self, channel_id: &str, message_id: &str) -> Result<(), sled::Error>;

    /// Delete all data for this repository (cleanup)
    fn delete(&self) -> Result<(), sled::Error>;
}
