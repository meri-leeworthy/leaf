//! Reaction sync service

use crate::repository::BridgeRepository;
use roomy_sdk_rust::Did;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use std::sync::Arc;

/// Reaction sync service
pub struct ReactionSyncService {
    repo: Arc<dyn BridgeRepository>,
    space_id: Did,
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
            space_id,
            guild_id,
        }
    }
}
