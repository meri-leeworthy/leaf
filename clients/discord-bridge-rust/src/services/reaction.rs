//! Reaction sync service

use crate::{repository::BridgeRepository, Error};
use roomy_sdk_rust::Did;
use twilight_model::{
    id::Id,
    id::marker::{GuildMarker, MessageMarker},
};

// Use the ReactionType from our bot events module
use crate::bot::events::ReactionType;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// Reaction sync service
///
/// Handles bidirectional synchronization of emoji reactions.
/// Uses database cache for idempotency tracking.
pub struct ReactionSyncService {
    repo: Arc<dyn BridgeRepository>,
    _space_id: Did,
    guild_id: Id<GuildMarker>,
}

impl ReactionSyncService {
    /// Create a new reaction sync service
    pub fn new(
        repo: Arc<dyn BridgeRepository>,
        space_id: Did,
        guild_id: Id<GuildMarker>,
    ) -> Self {
        Self {
            repo,
            _space_id: space_id,
            guild_id,
        }
    }

    /// Handle Discord reaction add event
    ///
    /// Converts Discord reaction to Roomy AddReaction event.
    #[instrument(skip(self, user_id, emoji), fields(guild_id = %self.guild_id, channel_id = %channel_id, message_id = %message_id))]
    pub async fn handle_reaction_add(
        &self,
        channel_id: Id<twilight_model::id::marker::ChannelMarker>,
        message_id: Id<MessageMarker>,
        user_id: Id<twilight_model::id::marker::UserMarker>,
        emoji: &ReactionType,
    ) -> Result<(), Error> {
        // Check if channel is synced
        if !self
            .repo
            .is_channel_synced(&channel_id.to_string())
            .await?
        {
            debug!("Channel not synced, ignoring reaction");
            return Ok(());
        }

        // Get Roomy message ULID
        let roomy_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_message_id(message_id))
            .await?
        {
            Some(crate::repository::RoomyId::Message(ulid)) => ulid,
            _ => {
                debug!("Unknown message, ignoring reaction");
                return Ok(());
            }
        };

        // Get user DID
        let user_did = self.get_user_did(user_id).await?;

        // Convert emoji to string
        let emoji_str = self.reaction_type_to_string(emoji);

        // Check idempotency cache
        let _cache_key = format!("{}:{}:{}", roomy_ulid, user_did, emoji_str);
        if self
            .repo
            .is_reaction_synced(&roomy_ulid.to_string(), &user_did.to_string(), &emoji_str)
            .await?
        {
            debug!("Reaction already synced, ignoring");
            return Ok(());
        }

        info!("Handling Discord reaction add");

        // Check if emoji is supported (Discord-specific custom emojis won't work on Roomy)
        if !self.is_emoji_supported(emoji) {
            debug!("Unsupported emoji type, skipping");
            return Ok(());
        }

        // TODO: Send AddReaction event to Roomy
        // let event = Event::AddReaction {
        //     entity_ulid: Ulid::new(),
        //     message_ulid: roomy_ulid,
        //     user_did,
        //     emoji: emoji_str,
        //     created_at: ...,
        // };

        // Mark as synced
        self.repo
            .mark_reaction_synced(&roomy_ulid.to_string(), &user_did.to_string(), &emoji_str)
            .await?;

        debug!("Synced reaction from Discord to Roomy");

        Ok(())
    }

    /// Handle Discord reaction remove event
    ///
    /// Converts Discord reaction removal to Roomy RemoveReaction event.
    #[instrument(skip(self, user_id, emoji), fields(guild_id = %self.guild_id, channel_id = %channel_id, message_id = %message_id))]
    pub async fn handle_reaction_remove(
        &self,
        channel_id: Id<twilight_model::id::marker::ChannelMarker>,
        message_id: Id<MessageMarker>,
        user_id: Id<twilight_model::id::marker::UserMarker>,
        emoji: &ReactionType,
    ) -> Result<(), Error> {
        // Check if channel is synced
        if !self
            .repo
            .is_channel_synced(&channel_id.to_string())
            .await?
        {
            return Ok(());
        }

        // Get Roomy message ULID
        let roomy_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_message_id(message_id))
            .await?
        {
            Some(crate::repository::RoomyId::Message(ulid)) => ulid,
            _ => {
                debug!("Unknown message, ignoring reaction removal");
                return Ok(());
            }
        };

        // Get user DID
        let user_did = self.get_user_did(user_id).await?;

        // Convert emoji to string
        let emoji_str = self.reaction_type_to_string(emoji);

        info!("Handling Discord reaction remove");

        // Check if we have this reaction cached
        if !self
            .repo
            .is_reaction_synced(&roomy_ulid.to_string(), &user_did.as_str().to_string(), &emoji_str)
            .await?
        {
            debug!("Reaction not in cache, may have been already removed");
            return Ok(());
        }

        // TODO: Send RemoveReaction event to Roomy

        // Remove from cache
        self.repo
            .remove_reaction_synced(&roomy_ulid.to_string(), &user_did.as_str().to_string(), &emoji_str)
            .await?;

        debug!("Synced reaction removal from Discord to Roomy");

        Ok(())
    }

    /// Handle Discord reaction clear (all reactions removed from message)
    #[instrument(skip(self), fields(guild_id = %self.guild_id, channel_id = %channel_id, message_id = %message_id))]
    pub async fn handle_reaction_clear(
        &self,
        channel_id: Id<twilight_model::id::marker::ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> Result<(), Error> {
        // Get Roomy message ULID
        let roomy_ulid = match self
            .repo
            .get_roomy_id(&crate::repository::discord_message_id(message_id))
            .await?
        {
            Some(crate::repository::RoomyId::Message(ulid)) => ulid,
            _ => return Ok(()),
        };

        info!("Handling Discord reaction clear");

        // TODO: Send event to clear all reactions on Roomy

        debug!("Cleared all reactions from Roomy message {}", roomy_ulid);

        Ok(())
    }

    /// Convert Twilight ReactionType to string for Roomy
    fn reaction_type_to_string(&self, emoji: &ReactionType) -> String {
        match emoji {
            ReactionType::Unicode { name } => name.clone(),
            ReactionType::Custom { id, .. } => {
                // Custom emoji - store as ID for now
                // Note: These won't render on Roomy, but we track them for consistency
                format!("custom:{}", id)
            }
        }
    }

    /// Check if emoji type is supported on Roomy
    fn is_emoji_supported(&self, emoji: &ReactionType) -> bool {
        match emoji {
            // Unicode emojis are universally supported
            ReactionType::Unicode { .. } => true,
            // Custom Discord emojis won't work on Roomy (no standard emoji set)
            ReactionType::Custom { .. } => {
                // TODO: Could we convert some popular custom emojis to unicode?
                false
            }
        }
    }

    /// Get user DID (create if needed)
    async fn get_user_did(&self, user_id: Id<twilight_model::id::marker::UserMarker>) -> Result<Did, Error> {
        // Check repository
        if let Some(roomy_id) = self
            .repo
            .get_roomy_id(&crate::repository::discord_user_id(user_id))
            .await?
        {
            if let crate::repository::RoomyId::User(did) = roomy_id {
                return Ok(did);
            }
        }

        // Create new DID
        let did = Did::new(format!("did:discord:{}", user_id.get()))?;

        // Store mapping
        self.repo
            .register_mapping(
                &crate::repository::discord_user_id(user_id),
                &crate::repository::RoomyId::User(did.clone()),
            )
            .await?;

        Ok(did)
    }
}
