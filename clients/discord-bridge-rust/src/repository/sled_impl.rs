//! Sled-based implementation of BridgeRepository

use super::{BridgeRepository, DiscordId, RoomyId};
use roomy_sdk_rust::Did;
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

/// Sled-based bridge repository
pub struct SledBridgeRepository {
    /// Sled database
    db: Arc<Db>,

    /// Discord → Roomy mappings
    discord_to_roomy: Arc<tree::Tree>,

    /// Roomy → Discord mappings
    roomy_to_discord: Arc<tree::Tree>,

    /// Cursor tracking per space
    cursors: Arc<tree::Tree>,

    /// Latest message per channel
    latest_messages: Arc<tree::Tree>,
}

impl SledBridgeRepository {
    /// Create a new repository for a guild-space pair
    ///
    /// # Arguments
    /// * `guild_id` - Discord guild ID
    /// * `space_did` - Roomy space DID
    /// * `db_path` - Base path for databases (optional, defaults to ./data)
    pub fn new(
        guild_id: Id<GuildMarker>,
        space_did: &Did,
        db_path: Option<&Path>,
    ) -> Result<Self, sled::Error> {
        let base_path = db_path.unwrap_or_else(|| Path::new("./data"));

        // Create guild-space specific database path
        let db_path = base_path.join(format!("bridge_{}_{}", guild_id, space_did.as_str().replace(":", "_")));

        // Open database
        let db = sled::open(db_path)?;
        let db = Arc::new(db);

        // Open trees
        let discord_to_roomy = db.open_tree("discord_to_roomy")?;
        let roomy_to_discord = db.open_tree("roomy_to_discord")?;
        let cursors = db.open_tree("cursors")?;
        let latest_messages = db.open_tree("latest_messages")?;

        Ok(Self {
            db,
            discord_to_roomy: Arc::new(discord_to_roomy),
            roomy_to_discord: Arc::new(roomy_to_discord),
            cursors: Arc::new(cursors),
            latest_messages: Arc::new(latest_messages),
        })
    }

    /// Build a key for Discord → Roomy mapping
    fn build_discord_key(prefix: &str, id: &DiscordId) -> Vec<u8> {
        format!("{}:{}", prefix, id).into_bytes()
    }

    /// Build a key for Roomy → Discord mapping
    fn build_roomy_key(prefix: &str, id: &RoomyId) -> Vec<u8> {
        format!("{}:{}", prefix, id.to_key()).into_bytes()
    }
}

impl BridgeRepository for SledBridgeRepository {
    fn get_roomy_id(&self, discord_id: &DiscordId) -> Result<Option<RoomyId>, sled::Error> {
        // Try all prefixes
        let prefixes = ["channel", "message", "user"];

        for prefix in prefixes {
            let key = Self::build_discord_key(prefix, discord_id);
            if let Some(value) = self.discord_to_roomy.get(key)? {
                if let Some(roomy_id) = RoomyId::from_key(&String::from_utf8_lossy(&value)) {
                    return Ok(Some(roomy_id));
                }
            }
        }

        Ok(None)
    }

    fn get_discord_id(&self, roomy_id: &RoomyId) -> Result<Option<DiscordId>, sled::Error> {
        // Try all prefixes
        let prefixes = ["room", "message", "user"];

        for prefix in prefixes {
            let key = Self::build_roomy_key(prefix, roomy_id);
            if let Some(value) = self.roomy_to_discord.get(key)? {
                return Ok(Some(String::from_utf8_lossy(&value).to_string()));
            }
        }

        Ok(None)
    }

    fn register_mapping(
        &self,
        discord_id: &DiscordId,
        roomy_id: &RoomyId,
    ) -> Result<(), sled::Error> {
        // Determine prefixes based on Roomy ID type
        let (discord_prefix, roomy_prefix) = match roomy_id {
            RoomyId::Ulid(_) => {
                // Could be channel, room, or message - need to infer from discord_id context
                // For now, store bidirectionally with generic prefixes
                ("id", "id")
            }
            RoomyId::Did(_) => ("user", "user"),
        };

        // Store Discord → Roomy
        let discord_key = Self::build_discord_key(discord_prefix, discord_id);
        self.discord_to_roomy.insert(discord_key, roomy_id.to_key().as_bytes())?;

        // Store Roomy → Discord
        let roomy_key = Self::build_roomy_key(roomy_prefix, roomy_id);
        self.roomy_to_discord.insert(roomy_key, discord_id.as_bytes())?;

        Ok(())
    }

    fn get_cursor(&self, space_did: &Did) -> Result<u64, sled::Error> {
        let key = space_did.as_str();
        match self.cursors.get(key)? {
            Some(value) => {
                let idx = String::from_utf8_lossy(&value);
                Ok(idx.parse().unwrap_or(0))
            }
            None => Ok(0),
        }
    }

    fn set_cursor(&self, space_did: &Did, idx: u64) -> Result<(), sled::Error> {
        let key = space_did.as_str();
        self.cursors.insert(key, idx.to_string().as_bytes())?;
        Ok(())
    }

    fn get_latest_message(&self, channel_id: &str) -> Result<Option<String>, sled::Error> {
        match self.latest_messages.get(channel_id)? {
            Some(value) => Ok(Some(String::from_utf8_lossy(&value).to_string())),
            None => Ok(None),
        }
    }

    fn set_latest_message(&self, channel_id: &str, message_id: &str) -> Result<(), sled::Error> {
        self.latest_messages.insert(channel_id, message_id.as_bytes())?;
        Ok(())
    }

    fn delete(&self) -> Result<(), sled::Error> {
        // Clear all trees
        self.discord_to_roomy.clear()?;
        self.roomy_to_discord.clear()?;
        self.cursors.clear()?;
        self.latest_messages.clear()?;
        Ok(())
    }
}

// Fix for tree::Tree type annotation
mod tree {
    pub use sled::Tree;
}
