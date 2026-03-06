//! Sled-based implementation of BridgeRepository

use super::{BridgeRepository, BridgeConfig, ChannelSettings, DiscordId, RoomyId};
use roomy_sdk_rust::Did;
use sled::Db;
use std::path::Path;
use std::sync::Arc;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

/// Sled-based bridge repository
pub struct SledBridgeRepository {
    /// Sled database
    _db: Arc<Db>,

    /// Discord → Roomy mappings
    discord_to_roomy: Arc<tree::Tree>,

    /// Roomy → Discord mappings
    roomy_to_discord: Arc<tree::Tree>,

    /// Cursor tracking per space
    cursors: Arc<tree::Tree>,

    /// Latest message per channel
    latest_messages: Arc<tree::Tree>,

    /// Bridge configuration
    config: Arc<tree::Tree>,

    /// Channel sync status
    channel_synced: Arc<tree::Tree>,

    /// Channel settings
    channel_settings: Arc<tree::Tree>,

    /// Webhook URLs
    webhooks: Arc<tree::Tree>,

    /// Message content hashes (for edit detection)
    message_hashes: Arc<tree::Tree>,

    /// Reaction sync cache (for idempotency)
    reaction_synced: Arc<tree::Tree>,
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
        let config = db.open_tree("config")?;
        let channel_synced = db.open_tree("channel_synced")?;
        let channel_settings = db.open_tree("channel_settings")?;
        let webhooks = db.open_tree("webhooks")?;
        let message_hashes = db.open_tree("message_hashes")?;
        let reaction_synced = db.open_tree("reaction_synced")?;

        Ok(Self {
            _db: db,
            discord_to_roomy: Arc::new(discord_to_roomy),
            roomy_to_discord: Arc::new(roomy_to_discord),
            cursors: Arc::new(cursors),
            latest_messages: Arc::new(latest_messages),
            config: Arc::new(config),
            channel_synced: Arc::new(channel_synced),
            channel_settings: Arc::new(channel_settings),
            webhooks: Arc::new(webhooks),
            message_hashes: Arc::new(message_hashes),
            reaction_synced: Arc::new(reaction_synced),
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

#[async_trait::async_trait]
impl BridgeRepository for SledBridgeRepository {
    async fn get_roomy_id(&self, discord_id: &DiscordId) -> Result<Option<RoomyId>, crate::Error> {
        let discord_id = discord_id.clone();
        let tree = self.discord_to_roomy.clone();

        let result = tokio::task::spawn_blocking(move || {
            // Try all prefixes
            let prefixes = ["channel", "message", "user"];

            for prefix in prefixes {
                let key = Self::build_discord_key(prefix, &discord_id);
                if let Some(value) = tree.get(key)? {
                    if let Some(roomy_id) = RoomyId::from_key(&String::from_utf8_lossy(&value)) {
                        return Ok(Some(roomy_id));
                    }
                }
            }

            Ok(None) as Result<Option<RoomyId>, sled::Error>
        })
        .await??;

        Ok(result)
    }

    async fn get_discord_id(&self, roomy_id: &RoomyId) -> Result<Option<DiscordId>, crate::Error> {
        let roomy_id = roomy_id.clone();
        let tree = self.roomy_to_discord.clone();

        let result = tokio::task::spawn_blocking(move || {
            // Try all prefixes
            let prefixes = ["room", "message", "user"];

            for prefix in prefixes {
                let key = Self::build_roomy_key(prefix, &roomy_id);
                if let Some(value) = tree.get(key)? {
                    return Ok(Some(String::from_utf8_lossy(&value).to_string()));
                }
            }

            Ok(None) as Result<Option<DiscordId>, sled::Error>
        })
        .await??;

        Ok(result)
    }

    async fn register_mapping(
        &self,
        discord_id: &DiscordId,
        roomy_id: &RoomyId,
    ) -> Result<(), crate::Error> {
        let discord_id = discord_id.clone();
        let roomy_id = roomy_id.clone();
        let discord_tree = self.discord_to_roomy.clone();
        let roomy_tree = self.roomy_to_discord.clone();

        tokio::task::spawn_blocking(move || {
            // Determine prefixes based on Roomy ID type
            let (discord_prefix, roomy_prefix) = match &roomy_id {
                RoomyId::Ulid(_) | RoomyId::Room(_) | RoomyId::Message(_) => {
                    // Could be channel, room, or message - need to infer from discord_id context
                    // For now, store bidirectionally with generic prefixes
                    ("id", "id")
                }
                RoomyId::Did(_) | RoomyId::User(_) => ("user", "user"),
            };

            // Store Discord → Roomy
            let discord_key = Self::build_discord_key(discord_prefix, &discord_id);
            discord_tree.insert(discord_key, roomy_id.to_key().as_bytes())?;

            // Store Roomy → Discord
            let roomy_key = Self::build_roomy_key(roomy_prefix, &roomy_id);
            roomy_tree.insert(roomy_key, discord_id.as_bytes())?;

            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn get_cursor(&self, space_did: &Did) -> Result<u64, crate::Error> {
        let key = space_did.as_str().to_string();
        let tree = self.cursors.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(value) => {
                    let idx = String::from_utf8_lossy(&value);
                    Ok::<u64, sled::Error>(idx.parse().unwrap_or(0))
                }
                None => Ok::<u64, sled::Error>(0),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_cursor(&self, space_did: &Did, idx: u64) -> Result<(), crate::Error> {
        let key = space_did.as_str().to_string();
        let tree = self.cursors.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, idx.to_string().as_bytes())?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn get_latest_message(&self, channel_id: &str) -> Result<Option<String>, crate::Error> {
        let key = channel_id.to_string();
        let tree = self.latest_messages.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(value) => Ok::<Option<String>, sled::Error>(Some(String::from_utf8_lossy(&value).to_string())),
                None => Ok::<Option<String>, sled::Error>(None),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_latest_message(&self, channel_id: &str, message_id: &str) -> Result<(), crate::Error> {
        let key = channel_id.to_string();
        let value = message_id.to_string();
        let tree = self.latest_messages.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, value.as_bytes())?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn get_bridge_config(&self) -> Result<BridgeConfig, crate::Error> {
        let tree = self.config.clone();

        let config_bytes = tokio::task::spawn_blocking(move || {
            tree.get("config")
        })
        .await??;

        match config_bytes {
            Some(bytes) => {
                let config: BridgeConfig = serde_json::from_slice(&bytes)?;
                Ok(config)
            }
            None => Err(crate::Error::Config("Bridge config not found".to_string())),
        }
    }

    async fn set_bridge_config(&self, config: &BridgeConfig) -> Result<(), crate::Error> {
        let config = config.clone();
        let tree = self.config.clone();

        tokio::task::spawn_blocking(move || {
            let bytes = serde_json::to_vec(&config)
                .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            tree.insert("config", bytes)?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn is_channel_synced(&self, channel_id: &str) -> Result<bool, crate::Error> {
        let key = channel_id.to_string();
        let tree = self.channel_synced.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(value) => {
                    let synced = String::from_utf8_lossy(&value);
                    Ok::<bool, sled::Error>(synced == "1")
                }
                None => Ok::<bool, sled::Error>(false),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_channel_synced(&self, channel_id: &str, synced: bool) -> Result<(), crate::Error> {
        let key = channel_id.to_string();
        let value = if synced { "1" } else { "0" };
        let tree = self.channel_synced.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, value.as_bytes())?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn list_synced_channels(&self) -> Result<Vec<String>, crate::Error> {
        let tree = self.channel_synced.clone();

        let channels = tokio::task::spawn_blocking(move || {
            let mut result = Vec::new();
            for item in tree.iter() {
                let (key, value) = item?;
                let key_str = String::from_utf8_lossy(&key);
                let value_str = String::from_utf8_lossy(&value);
                if value_str == "1" {
                    result.push(key_str.to_string());
                }
            }
            Ok(result) as Result<Vec<String>, sled::Error>
        })
        .await??;

        Ok(channels)
    }

    async fn get_channel_settings(&self, channel_id: &str) -> Result<Option<ChannelSettings>, crate::Error> {
        let key = channel_id.to_string();
        let tree = self.channel_settings.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(bytes) => {
                    let settings: ChannelSettings = serde_json::from_slice(&bytes)
                        .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
                    Ok::<Option<ChannelSettings>, sled::Error>(Some(settings))
                }
                None => Ok::<Option<ChannelSettings>, sled::Error>(None),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_channel_settings(&self, channel_id: &str, settings: &ChannelSettings) -> Result<(), crate::Error> {
        let key = channel_id.to_string();
        let settings = settings.clone();
        let tree = self.channel_settings.clone();

        tokio::task::spawn_blocking(move || {
            let bytes = serde_json::to_vec(&settings)
                .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            tree.insert(key, bytes)?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn get_webhook_url(&self, channel_id: &str) -> Result<Option<String>, crate::Error> {
        let key = channel_id.to_string();
        let tree = self.webhooks.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(value) => Ok::<Option<String>, sled::Error>(Some(String::from_utf8_lossy(&value).to_string())),
                None => Ok::<Option<String>, sled::Error>(None),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_webhook_url(&self, channel_id: &str, url: &str) -> Result<(), crate::Error> {
        let key = channel_id.to_string();
        let value = url.to_string();
        let tree = self.webhooks.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, value.as_bytes())?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn get_message_hash(&self, message_ulid: &str) -> Result<Option<[u8; 32]>, crate::Error> {
        let key = message_ulid.to_string();
        let tree = self.message_hashes.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(value) => {
                    if value.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&value);
                        Ok::<Option<[u8; 32]>, sled::Error>(Some(arr))
                    } else {
                        Ok::<Option<[u8; 32]>, sled::Error>(None)
                    }
                }
                None => Ok::<Option<[u8; 32]>, sled::Error>(None),
            }
        })
        .await??;

        Ok(result)
    }

    async fn set_message_hash(&self, message_ulid: &str, hash: [u8; 32]) -> Result<(), crate::Error> {
        let key = message_ulid.to_string();
        let tree = self.message_hashes.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, hash.to_vec())?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn is_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<bool, crate::Error> {
        let key = format!("{}:{}:{}", message_ulid, user_did, emoji);
        let tree = self.reaction_synced.clone();

        let result = tokio::task::spawn_blocking(move || {
            match tree.get(key)? {
                Some(_) => Ok::<bool, sled::Error>(true),
                None => Ok::<bool, sled::Error>(false),
            }
        })
        .await??;

        Ok(result)
    }

    async fn mark_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<(), crate::Error> {
        let key = format!("{}:{}:{}", message_ulid, user_did, emoji);
        let tree = self.reaction_synced.clone();

        tokio::task::spawn_blocking(move || {
            tree.insert(key, b"1")?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn remove_reaction_synced(&self, message_ulid: &str, user_did: &str, emoji: &str) -> Result<(), crate::Error> {
        let key = format!("{}:{}:{}", message_ulid, user_did, emoji);
        let tree = self.reaction_synced.clone();

        tokio::task::spawn_blocking(move || {
            tree.remove(key)?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }

    async fn delete(&self) -> Result<(), crate::Error> {
        let discord_tree = self.discord_to_roomy.clone();
        let roomy_tree = self.roomy_to_discord.clone();
        let cursors = self.cursors.clone();
        let latest_messages = self.latest_messages.clone();
        let config = self.config.clone();
        let channel_synced = self.channel_synced.clone();
        let channel_settings = self.channel_settings.clone();
        let webhooks = self.webhooks.clone();
        let message_hashes = self.message_hashes.clone();
        let reaction_synced = self.reaction_synced.clone();

        tokio::task::spawn_blocking(move || {
            // Clear all trees
            discord_tree.clear()?;
            roomy_tree.clear()?;
            cursors.clear()?;
            latest_messages.clear()?;
            config.clear()?;
            channel_synced.clear()?;
            channel_settings.clear()?;
            webhooks.clear()?;
            message_hashes.clear()?;
            reaction_synced.clear()?;
            Ok(()) as Result<(), sled::Error>
        })
        .await??;

        Ok(())
    }
}

// Fix for tree::Tree type annotation
mod tree {
    pub use sled::Tree;
}
