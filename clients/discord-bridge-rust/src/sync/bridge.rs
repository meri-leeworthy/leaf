//! Single guild-space bridge

use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use roomy_sdk_rust::Did;

/// Bridge for a single guild-space pair
pub struct Bridge {
    guild_id: Id<GuildMarker>,
    space_did: Did,
}

impl Bridge {
    /// Connect to a guild-space pair
    pub async fn connect(
        guild_id: Id<GuildMarker>,
        space_did: Did,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            guild_id,
            space_did,
        })
    }
}
