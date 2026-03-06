//! # Bridge Repository
//!
//! This module provides persistent storage for ID mappings and sync metadata.
//!
//! ## Purpose
//!
//! The bridge maintains a local database (Sled) to store:
//! - Discord ↔ Roomy ID mappings (correlation)
//! - Sync state and cursors (resumability)
//! - Webhook URLs (message forwarding)
//! - Message hashes (edit detection)
//! - Reaction cache (idempotency)
//! - Channel settings (configuration)
//!
//! ## Key Types
//!
//! - [`BridgeRepository`]: Async trait for all database operations
//! - [`SledBridgeRepository`]: Sled-based implementation
//! - [`RoomyId`]: Typed wrapper for Roomy IDs (ULID or DID)
//! - [`DiscordId`]: String wrapper for Discord snowflakes with prefixes
//!
//! ## ID Mapping Strategy
//!
//! Discord IDs are stored with prefixes (`channel:`, `message:`, `user:`) to
//! avoid collisions between different Discord entity types in the same keyspace.

mod sled_impl;

pub use sled_impl::SledBridgeRepository;

use roomy_sdk_rust::Did;
use serde::{Deserialize, Serialize};
// use twilight_model::channel::AllowedMentions; // TODO: Not available in twilight 0.15
use twilight_model::id::{Id, marker};

/// Discord ID (snowflake as string)
///
/// We use String with prefixes for Discord IDs to simplify serialization.
pub type DiscordId = String;

/// Helper to create Discord channel ID key
pub fn discord_channel_id(id: Id<marker::ChannelMarker>) -> DiscordId {
    format!("channel:{}", id)
}

/// Helper to create Discord message ID key
pub fn discord_message_id(id: Id<marker::MessageMarker>) -> DiscordId {
    format!("message:{}", id)
}

/// Helper to create Discord user ID key
pub fn discord_user_id(id: Id<marker::UserMarker>) -> DiscordId {
    format!("user:{}", id)
}

/// Roomy ID (ULID string or DID)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoomyId {
    /// ULID for messages, rooms, etc.
    Ulid(String),
    /// DID for users
    Did(Did),
    // Convenience constructors
    Room(String),
    Message(String),
    User(Did),
}

impl RoomyId {
    /// Convert to string for storage
    pub fn to_key(&self) -> String {
        match self {
            Self::Ulid(u) | Self::Room(u) | Self::Message(u) => u.clone(),
            Self::Did(d) | Self::User(d) => d.as_str().to_string(),
        }
    }

    /// Parse from string
    pub fn from_key(s: &str) -> Option<Self> {
        // Try ULID first (check if it's a 26-character base32 string)
        if s.len() == 26 {
            return Some(Self::Ulid(s.to_string()));
        }
        // Try DID
        if s.starts_with("did:") {
            return Some(Self::Did(Did::new(s.to_string()).ok()?));
        }
        None
    }
}

/// Bridge configuration mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeMode {
    /// Bridge all public channels in the guild
    Full,

    /// Bridge specific channels (user-selected)
    Subset { channels: Vec<Id<marker::ChannelMarker>> },

    /// Bridge channels with specific Discord permissions
    PermissionBased { role_id: Option<Id<marker::RoleMarker>> },
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub guild_id: String,
    pub space_did: Did,
    pub mode: BridgeMode,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Per-channel sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSettings {
    pub channel_id: Id<marker::ChannelMarker>,
    pub auto_thread: bool,
    pub sync_reactions: bool,
    pub sync_edits: bool,
    // pub allowed_mentions: AllowedMentions, // TODO: Not available in twilight 0.15
}

/// Trait for bridge repository operations
///
/// This trait provides async methods for all database operations.
/// Implementations must be thread-safe (Send + Sync).
#[async_trait::async_trait]
pub trait BridgeRepository: Send + Sync {
    // === ID Mapping ===
    /// Get Roomy ID for a Discord ID
    async fn get_roomy_id(&self, discord_id: &DiscordId) -> Result<Option<RoomyId>, crate::Error>;

    /// Get Discord ID for a Roomy ID
    async fn get_discord_id(&self, roomy_id: &RoomyId) -> Result<Option<DiscordId>, crate::Error>;

    /// Register a Discord ↔ Roomy mapping
    async fn register_mapping(
        &self,
        discord_id: &DiscordId,
        roomy_id: &RoomyId,
    ) -> Result<(), crate::Error>;

    // === Cursor Management ===
    /// Get the cursor position for a space stream
    async fn get_cursor(&self, space_did: &Did) -> Result<u64, crate::Error>;

    /// Set the cursor position for a space stream
    async fn set_cursor(&self, space_did: &Did, idx: u64) -> Result<(), crate::Error>;

    // === Backfill State ===
    /// Get the latest message ID for a Discord channel
    async fn get_latest_message(&self, channel_id: &str) -> Result<Option<String>, crate::Error>;

    /// Set the latest message ID for a Discord channel
    async fn set_latest_message(&self, channel_id: &str, message_id: &str) -> Result<(), crate::Error>;

    // === Configuration Metadata ===
    /// Get bridge configuration
    async fn get_bridge_config(&self) -> Result<BridgeConfig, crate::Error>;

    /// Set bridge configuration
    async fn set_bridge_config(&self, config: &BridgeConfig) -> Result<(), crate::Error>;

    /// Check if channel is synced
    async fn is_channel_synced(&self, channel_id: &str) -> Result<bool, crate::Error>;

    /// Set channel sync status
    async fn set_channel_synced(&self, channel_id: &str, synced: bool) -> Result<(), crate::Error>;

    /// List all synced channels
    async fn list_synced_channels(&self) -> Result<Vec<String>, crate::Error>;

    /// Get channel settings
    async fn get_channel_settings(&self, channel_id: &str) -> Result<Option<ChannelSettings>, crate::Error>;

    /// Set channel settings
    async fn set_channel_settings(&self, channel_id: &str, settings: &ChannelSettings) -> Result<(), crate::Error>;

    // === Webhook Cache ===
    /// Get webhook URL for channel
    async fn get_webhook_url(&self, channel_id: &str) -> Result<Option<String>, crate::Error>;

    /// Set webhook URL for channel
    async fn set_webhook_url(&self, channel_id: &str, url: &str) -> Result<(), crate::Error>;

    // === Message Hashes (for edit detection) ===
    /// Get message content hash
    async fn get_message_hash(&self, message_ulid: &str) -> Result<Option<[u8; 32]>, crate::Error>;

    /// Set message content hash
    async fn set_message_hash(&self, message_ulid: &str, hash: [u8; 32]) -> Result<(), crate::Error>;

    // === Reaction Cache (for idempotency) ===
    /// Check if reaction is synced
    async fn is_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<bool, crate::Error>;

    /// Mark reaction as synced
    async fn mark_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<(), crate::Error>;

    /// Remove reaction from cache
    async fn remove_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<(), crate::Error>;

    // === Cleanup ===
    /// Delete all data for this repository
    async fn delete(&self) -> Result<(), crate::Error>;
}
