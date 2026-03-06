//! Structure sync service

use crate::{
    bot::events::{ChannelCreatePayload, ThreadCreatePayload},
    repository::{BridgeRepository, RoomyId},
    Error,
};
use roomy_sdk_rust::Did;
use twilight_http::Client;
use twilight_model::{
    channel::{Channel, ChannelType},
    id::Id,
    id::marker::{ChannelMarker, GuildMarker},
};
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Structure sync service
///
/// Handles synchronization of Discord channels/threads to Roomy rooms.
/// All configuration (which channels to sync) is stored in the bridge database.
pub struct StructureSyncService {
    repo: Arc<dyn BridgeRepository>,
    _space_id: Did,
    guild_id: Id<GuildMarker>,
    http: Arc<Client>,
}

impl StructureSyncService {
    /// Create a new structure sync service
    pub fn new(
        repo: Arc<dyn BridgeRepository>,
        space_id: Did,
        guild_id: Id<GuildMarker>,
        http: Arc<Client>,
    ) -> Self {
        Self {
            repo,
            _space_id: space_id,
            guild_id,
            http,
        }
    }

    /// Handle Discord channel create event
    ///
    /// Creates corresponding Roomy room for synced channels.
    #[instrument(skip(self, payload), fields(guild_id = %self.guild_id, channel_id = %payload.channel.id))]
    pub async fn handle_channel_create(&self, payload: ChannelCreatePayload) -> Result<(), Error> {
        // Ignore channels from other guilds
        if payload.channel.guild_id != Some(self.guild_id) {
            return Ok(());
        }

        let channel = &payload.channel;

        // Check if channel type is supported
        if !self.is_supported_channel_type(channel) {
            debug!("Ignoring unsupported channel type: {:?}", channel.kind);
            return Ok(());
        }

        // Check if channel should be synced based on bridge mode
        let bridge_config = self.repo.get_bridge_config().await?;
        if !self.should_sync_channel(channel, &bridge_config).await? {
            debug!("Channel not configured for sync, ignoring");
            return Ok(());
        }

        info!("Handling Discord channel create: {}", channel.name.as_ref().unwrap_or(&"<unnamed>".to_string()));

        // Create Roomy room
        let room_ulid = self.create_roomy_room(channel).await?;

        // Store mapping
        self.repo
            .register_mapping(
                &crate::repository::discord_channel_id(channel.id),
                &RoomyId::Room(room_ulid.clone()),
            )
            .await?;

        // Mark channel as synced
        self.repo
            .set_channel_synced(&channel.id.to_string(), true)
            .await?;

        // Store default channel settings
        let settings = crate::repository::ChannelSettings {
            channel_id: channel.id,
            auto_thread: false,
            sync_reactions: true,
            sync_edits: true,
            // allowed_mentions: twilight_model::channel::AllowedMentions { // TODO: Not available in twilight 0.15
            //     parse: vec![],
            //     users: vec![],
            //     roles: vec![],
            //     replied_user: false,
            // },
        };
        self.repo
            .set_channel_settings(&channel.id.to_string(), &settings)
            .await?;

        // Create webhook for Roomy → Discord sync
        self.create_webhook(channel.id).await?;

        info!(
            "Created Roomy room {} for Discord channel {}",
            room_ulid, channel.id
        );

        Ok(())
    }

    /// Handle Discord thread create event
    ///
    /// Creates corresponding Roomy room or thread as child of parent channel.
    #[instrument(skip(self, payload), fields(guild_id = %self.guild_id, thread_id = %payload.channel.id))]
    pub async fn handle_thread_create(&self, payload: ThreadCreatePayload) -> Result<(), Error> {
        let thread = &payload.channel;

        // Check if parent channel is synced
        let parent_id = payload.parent_id;

        let parent_room_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_channel_id(parent_id))
            .await?
        {
            Some(RoomyId::Room(ulid)) => ulid,
            _ => {
                debug!("Parent channel not synced, ignoring thread");
                return Ok(());
            }
        };

        info!("Handling Discord thread create: {}", thread.name.as_ref().unwrap_or(&"<unnamed>".to_string()));

        // Create Roomy room as child of parent
        let room_ulid = self.create_roomy_thread(thread, parent_room_ulid).await?;

        // Store mapping
        self.repo
            .register_mapping(&crate::repository::discord_channel_id(thread.id), &RoomyId::Room(room_ulid.clone()))
            .await?;

        // Mark thread as synced
        self.repo
            .set_channel_synced(&thread.id.to_string(), true)
            .await?;

        // Create webhook for the thread
        self.create_webhook(thread.id).await?;

        info!(
            "Created Roomy room {} for Discord thread {}",
            room_ulid, thread.id
        );

        Ok(())
    }

    /// Backfill all channels for the guild
    ///
    /// Fetches all Discord channels and creates corresponding Roomy rooms.
    #[instrument(skip(self), fields(guild_id = %self.guild_id))]
    pub async fn backfill_channels(&self) -> Result<u64, Error> {
        info!("Starting channel backfill for guild {}", self.guild_id);

        let bridge_config = self.repo.get_bridge_config().await?;
        let mut rooms_created = 0u64;

        // Fetch all channels from Discord
        let channels = self
            .http
            .guild_channels(self.guild_id)
            .await?
            .model()
            .await?;

        for channel in channels {
            // Skip unsupported channel types
            if !self.is_supported_channel_type(&channel) {
                continue;
            }

            // Check if channel should be synced
            if !self.should_sync_channel(&channel, &bridge_config).await? {
                continue;
            }

            // Skip if already synced
            if self
                .repo
                .is_channel_synced(&channel.id.to_string())
                .await?
            {
                debug!("Channel {} already synced, skipping", channel.id);
                continue;
            }

            // Create Roomy room
            let room_ulid = self.create_roomy_room(&channel).await?;

            // Store mapping
            self.repo
                .register_mapping(&crate::repository::discord_channel_id(channel.id), &RoomyId::Room(room_ulid))
                .await?;

            // Mark as synced
            self.repo
                .set_channel_synced(&channel.id.to_string(), true)
                .await?;

            // Create webhook
            self.create_webhook(channel.id).await?;

            rooms_created += 1;
        }

        info!(
            "Backfilled {} channels for guild {}",
            rooms_created, self.guild_id
        );

        Ok(rooms_created)
    }

    /// Create Roomy room from Discord channel
    async fn create_roomy_room(&self, channel: &Channel) -> Result<String, Error> {
        let room_ulid = ulid::Ulid::new().to_string();

        // Determine room kind based on Discord channel type
        let kind = match channel.kind {
            ChannelType::GuildText => "space.roomy.channel",
            ChannelType::GuildVoice => "space.roomy.voice",
            ChannelType::GuildForum => "space.roomy.forum",
            ChannelType::GuildAnnouncement => "space.roomy.announcement",
            _ => "space.roomy.channel", // fallback
        };

        // TODO: Send CreateRoom event to Roomy
        // let event = Event::CreateRoom {
        //     entity_ulid: room_ulid,
        //     kind: kind.to_string(),
        //     space_ulid: self.space_id,
        //     // ... other fields
        // };

        debug!(
            "Would create Roomy room {} with kind {}",
            room_ulid, kind
        );

        Ok(room_ulid)
    }

    /// Create Roomy room from Discord thread
    async fn create_roomy_thread(
        &self,
        _thread: &Channel,
        parent_ulid: String,
    ) -> Result<String, Error> {
        let room_ulid = ulid::Ulid::new().to_string();

        // TODO: Send CreateRoom event to Roomy with parent relationship

        debug!(
            "Would create Roomy room {} as child of {}",
            room_ulid, parent_ulid
        );

        Ok(room_ulid)
    }

    /// Create webhook for Roomy → Discord messaging
    async fn create_webhook(&self, channel_id: Id<ChannelMarker>) -> Result<(), Error> {
        // Check if webhook already exists
        if let Some(_url) = self
            .repo
            .get_webhook_url(&channel_id.to_string())
            .await?
        {
            debug!("Webhook already exists for channel {}", channel_id);
            return Ok(());
        }

        // Create webhook on Discord
        // Note: We need a webhook name that won't conflict
        let webhook = self
            .http
            .create_webhook(channel_id, "Roomy Bridge")
            .map_err(|e| Error::Other(format!("Validation error: {}", e)))?
            .await?
            .model()
            .await?;

        // Store webhook URL in cache
        let webhook_url = format!(
            "https://discord.com/api/webhooks/{}/{}",
            webhook.id,
            webhook.token.as_ref().ok_or_else(|| Error::Other("Webhook token missing".to_string()))?
        );

        self.repo
            .set_webhook_url(&channel_id.to_string(), &webhook_url)
            .await?;

        debug!("Created webhook for channel {}", channel_id);

        Ok(())
    }

    /// Check if channel type is supported for syncing
    fn is_supported_channel_type(&self, channel: &Channel) -> bool {
        matches!(
            channel.kind,
            ChannelType::GuildText
                | ChannelType::GuildVoice
                | ChannelType::GuildForum
                | ChannelType::GuildAnnouncement
                | ChannelType::PublicThread
                | ChannelType::PrivateThread
        )
    }

    /// Check if channel should be synced based on bridge mode
    async fn should_sync_channel(
        &self,
        channel: &Channel,
        config: &crate::repository::BridgeConfig,
    ) -> Result<bool, Error> {
        match &config.mode {
            crate::repository::BridgeMode::Full => {
                // Sync all public channels
                Ok(!channel.kind.is_thread())
            }
            crate::repository::BridgeMode::Subset { channels } => {
                // Sync only specified channels
                Ok(channels.iter().any(|id| *id == channel.id))
            }
            crate::repository::BridgeMode::PermissionBased { role_id } => {
                // Sync channels where bridge role has permission
                if let Some(_role_id) = role_id {
                    // TODO: Check permissions using Discord API
                    // For now, sync all channels
                    Ok(true)
                } else {
                    Ok(true)
                }
            }
        }
    }
}
