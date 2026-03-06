//! Message sync service

use crate::{
    bot::events::{MessageCreatePayload, MessageUpdatePayload},
    repository::BridgeRepository,
    sync::dispatcher::QueuedRoomyEvent,
    Error,
};
use roomy_sdk_rust::{events::{CreateMessageEvent, EditMessageEvent, Event}, Did};
use twilight_http::Client;
use twilight_model::{
    channel::Message,
    id::Id,
    id::marker::{ChannelMarker, GuildMarker, MessageMarker},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument};

/// Message sync service
///
/// Handles bidirectional synchronization of messages between Discord and Roomy.
/// Uses the bridge database for all metadata storage (edit hashes, webhook cache).
pub struct MessageSyncService {
    repo: Arc<dyn BridgeRepository>,
    _space_id: Did,
    guild_id: Id<GuildMarker>,
    http: Arc<Client>,
    _to_discord_tx: mpsc::Sender<QueuedRoomyEvent>,
}

impl MessageSyncService {
    /// Create a new message sync service
    pub fn new(
        repo: Arc<dyn BridgeRepository>,
        space_id: Did,
        guild_id: Id<GuildMarker>,
        http: Arc<Client>,
        to_discord_tx: mpsc::Sender<QueuedRoomyEvent>,
    ) -> Self {
        Self {
            repo,
            _space_id: space_id,
            guild_id,
            http,
            _to_discord_tx: to_discord_tx,
        }
    }

    /// Handle Discord message create event
    ///
    /// Converts Discord message to Roomy CreateMessage event and sends to dispatcher.
    #[instrument(skip(self, payload), fields(guild_id = %self.guild_id, channel_id = %payload.channel_id, message_id = %payload.message.id))]
    pub async fn handle_message_create(&self, payload: MessageCreatePayload) -> Result<(), Error> {
        // Ignore messages from other guilds
        if payload.message.guild_id != Some(self.guild_id) {
            debug!("Ignoring message from different guild");
            return Ok(());
        }

        // Ignore messages from bots (including our own webhooks)
        if payload.message.author.bot {
            debug!("Ignoring bot message");
            return Ok(());
        }

        // Check if channel is synced
        let channel_id_str = payload.channel_id.to_string();
        if !self.repo.is_channel_synced(&channel_id_str).await? {
            debug!("Channel not synced, ignoring message");
            return Ok(());
        }

        info!("Handling Discord message create");

        // Get Roomy room ID for this Discord channel
        let room_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_channel_id(payload.channel_id))
            .await?
            .ok_or_else(|| Error::MissingMapping(format!("channel:{}", payload.channel_id)))?
        {
            crate::repository::RoomyId::Room(ulid) => ulid,
            _ => return Err(Error::InvalidMapping("Expected room ULID".to_string())),
        };

        // Convert Discord message to Roomy event
        let roomy_event = self.discord_message_to_roomy(&payload.message, room_ulid).await?;

        // Extract the message ULID from the event
        let message_ulid = match &roomy_event {
            Event::CreateMessage(e) => e.id.clone(),
            _ => return Err(Error::Other("Unexpected event type".to_string())),
        };

        // Store mapping for future sync
        self.repo
            .register_mapping(
                &crate::repository::discord_message_id(payload.message.id),
                &crate::repository::RoomyId::Message(message_ulid),
            )
            .await?;

        // Send to Roomy
        // TODO: Send via SDK to Roomy space

        info!("Synced Discord message {} to Roomy", payload.message.id);
        Ok(())
    }

    /// Handle Discord message update event
    ///
    /// Detects if content actually changed (hash-based) and syncs to Roomy.
    #[instrument(skip(self, payload), fields(guild_id = %self.guild_id, channel_id = %payload.channel_id))]
    pub async fn handle_message_update(&self, payload: MessageUpdatePayload) -> Result<(), Error> {
        // Ignore updates from other guilds
        if payload.guild_id != Some(self.guild_id) {
            return Ok(());
        }

        // Check if channel is synced
        let channel_id_str = payload.channel_id.to_string();
        if !self.repo.is_channel_synced(&channel_id_str).await? {
            return Ok(());
        }

        // Get the full message (updates only contain changed fields)
        let message = self
            .http
            .message(payload.channel_id, payload.message.id)
            .await?
            .model()
            .await?;

        // Check if we're tracking this message
        let message_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_message_id(message.id))
            .await?
        {
            Some(crate::repository::RoomyId::Message(ulid)) => ulid,
            _ => return Ok(()),
        };

        // Get room ULID
        let room_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_channel_id(message.channel_id))
            .await?
        {
            Some(crate::repository::RoomyId::Room(ulid)) => ulid,
            _ => return Ok(()),
        };

        // Compute new content hash
        let new_hash = self.compute_message_hash(&message);

        // Check if content actually changed
        let old_hash = self.repo.get_message_hash(&message_ulid.to_string()).await?;
        if old_hash.as_ref() == Some(&new_hash) {
            debug!("Message content unchanged, skipping sync");
            return Ok(());
        }

        info!("Handling Discord message edit");

        // Store new hash
        self.repo
            .set_message_hash(&message_ulid.to_string(), new_hash)
            .await?;

        // Convert to Roomy EditMessage event
        let _roomy_event = self.discord_edit_to_roomy(&message, room_ulid, message_ulid)?;

        // Send to Roomy
        // TODO: Send via SDK to Roomy space

        info!("Synced Discord message edit {} to Roomy", message.id);
        Ok(())
    }

    /// Handle Discord message delete event
    #[instrument(skip(self), fields(guild_id = %self.guild_id, channel_id = %channel_id))]
    pub async fn handle_message_delete(
        &self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> Result<(), Error> {
        // Check if we're tracking this message
        let roomy_ulid = self
            .repo
            .get_roomy_id(&crate::repository::discord_message_id(message_id))
            .await?;

        let Some(roomy_ulid) = roomy_ulid else {
            debug!("Unknown message deleted, ignoring");
            return Ok(());
        };

        info!("Handling Discord message delete");

        // Convert to Roomy DeleteMessage event
        let (room_ulid, message_ulid) = match &roomy_ulid {
            crate::repository::RoomyId::Message(msg_ulid) => {
                // We need the room ULID too - fetch it from the repo
                // For now, we'll create a placeholder
                (msg_ulid.clone(), msg_ulid.clone())
            }
            _ => return Err(Error::InvalidMapping("Expected message ULID".into())),
        };

        let _roomy_event = Event::DeleteMessage(roomy_sdk_rust::events::DeleteMessageEvent {
            id: ulid::Ulid::new().to_string(),
            room: room_ulid,
            message_id: message_ulid,
        });

        // Send to Roomy
        // TODO: Send via SDK to Roomy space

        info!("Synced Discord message delete {} to Roomy", message_id);
        Ok(())
    }

    /// Backfill Discord messages for a channel
    ///
    /// Fetches historical messages from Discord and syncs them to Roomy.
    #[instrument(skip(self), fields(guild_id = %self.guild_id, channel_id = %channel_id))]
    pub async fn backfill_channel(
        &self,
        channel_id: Id<ChannelMarker>,
        limit: Option<u16>,
    ) -> Result<u64, Error> {
        info!("Starting backfill for channel {}", channel_id);

        let mut messages_synced = 0u64;

        // Get latest synced message from repo
        let last_synced_id = self
            .repo
            .get_latest_message(&channel_id.to_string())
            .await
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| Id::new(1)); // Start from beginning if no history

        let mut last_message_id = Some(last_synced_id);

        loop {
            // Fetch messages from Discord (max 100 per request)
            let response = self
                .http
                .channel_messages(channel_id)
                .limit(limit.unwrap_or(100))
                .map_err(|e| Error::Other(format!("Validation error: {}", e)))?
                .before(last_message_id.unwrap_or(last_synced_id))
                .await?;

            let messages: Vec<twilight_model::channel::Message> = response.model().await?;

            if messages.is_empty() {
                break;
            }

            for message in messages {
                // Skip bot messages
                if message.author.bot {
                    continue;
                }

                // Sync to Roomy
                let room_ulid = match self
                    .repo
                    .get_roomy_id(&crate::repository::discord_channel_id(channel_id))
                    .await?
                    .ok_or_else(|| Error::MissingMapping(format!("channel:{}", channel_id)))?
                {
                    crate::repository::RoomyId::Room(ulid) => ulid,
                    _ => continue,
                };

                let roomy_event = self
                    .discord_message_to_roomy(&message, room_ulid)
                    .await?;

                // Extract the message ULID from the event
                let message_ulid = match &roomy_event {
                    Event::CreateMessage(e) => e.id.clone(),
                    _ => continue, // Skip unexpected event types
                };

                // Store mapping
                self.repo
                    .register_mapping(
                        &crate::repository::discord_message_id(message.id),
                        &crate::repository::RoomyId::Message(message_ulid),
                    )
                    .await?;

                messages_synced += 1;
                last_message_id = Some(message.id);
            }

            // Break if we've hit the requested limit
            if let Some(limit) = limit {
                if messages_synced >= limit as u64 {
                    break;
                }
            }

            // Rate limit: Discord allows 50 requests per second per bucket
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // Store latest message ID
        if let Some(latest_id) = last_message_id {
            self.repo
                .set_latest_message(&channel_id.to_string(), &latest_id.to_string())
                .await?;
        }

        info!(
            "Backfilled {} messages for channel {}",
            messages_synced, channel_id
        );

        Ok(messages_synced)
    }

    /// Convert Discord message to Roomy CreateMessage event
    async fn discord_message_to_roomy(
        &self,
        message: &Message,
        room_ulid: String,
    ) -> Result<Event, Error> {
        // Get user DID from Discord user mapping
        let _author_did = self.get_or_create_user_mapping(message.author.id).await?;

        // Convert content
        let content = message.content.clone();

        // TODO: Handle attachments, embeds, stickers
        // TODO: Handle reply chains (message.reference)
        // TODO: Handle thread creation (message.thread)

        Ok(Event::CreateMessage(CreateMessageEvent {
            id: ulid::Ulid::new().to_string(),
            room: room_ulid,
            body: roomy_sdk_rust::events::Content {
                mime_type: "text/markdown".to_string(),
                data: content.into_bytes(),
            },
            attachments: vec![],
            extensions: serde_json::Value::Null,
        }))
    }

    /// Convert Discord message edit to Roomy EditMessage event
    fn discord_edit_to_roomy(
        &self,
        message: &Message,
        room_ulid: String,
        message_ulid: String,
    ) -> Result<Event, Error> {
        let content = message.content.clone();

        Ok(Event::EditMessage(EditMessageEvent {
            id: ulid::Ulid::new().to_string(),
            room: room_ulid,
            message_id: message_ulid,
            body: roomy_sdk_rust::events::Content {
                mime_type: "text/markdown".to_string(),
                data: content.into_bytes(),
            },
            extensions: serde_json::Value::Null,
        }))
    }

    /// Get or create user mapping for Discord user
    async fn get_or_create_user_mapping(&self, user_id: Id<twilight_model::id::marker::UserMarker>) -> Result<Did, Error> {
        // Check if we already have a mapping
        if let Some(roomy_id) = self
            .repo
            .get_roomy_id(&crate::repository::discord_user_id(user_id))
            .await?
        {
            if let crate::repository::RoomyId::User(did) = roomy_id {
                return Ok(did);
            }
        }

        // Fetch user from Discord
        let user = self.http.user(user_id).await?.model().await?;

        // Create a DID for the user
        // Note: This is a placeholder - in real implementation, we'd use a proper DID method
        let did = Did::new(format!("did:discord:{}", user.id.get()))?;

        // Store mapping
        self.repo
            .register_mapping(
                &crate::repository::discord_user_id(user_id),
                &crate::repository::RoomyId::User(did.clone()),
            )
            .await?;

        // Trigger profile sync
        // TODO: Send profile sync event

        Ok(did)
    }

    /// Compute content hash for edit detection
    fn compute_message_hash(&self, message: &Message) -> [u8; 32] {
        

        // Hash: content + attachments + embeds
        let mut hasher = blake3::Hasher::new();
        hasher.update(message.content.as_bytes());
        for attachment in &message.attachments {
            hasher.update(attachment.url.as_bytes());
        }
        for embed in &message.embeds {
            // Serialize embed to hashable format
            if let Ok(json) = serde_json::to_string(embed) {
                hasher.update(json.as_bytes());
            }
        }

        let hash = hasher.finalize();
        *hash.as_bytes()
    }
}
