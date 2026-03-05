//! Message sync service

use crate::repository::BridgeRepository;
use roomy_sdk_rust::{Did, Event, Ulid};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use std::sync::Arc;

/// Message sync service
pub struct MessageSyncService {
    repo: Arc<dyn BridgeRepository>,
    space_id: Did,
    guild_id: Id<GuildMarker>,
}

impl MessageSyncService {
    /// Create a new message sync service
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
