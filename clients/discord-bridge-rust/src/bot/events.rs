//! Discord event types that we handle

use twilight_model::{
    channel::{Channel, Message},
    id::{marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker}, Id},
};

/// Simplified reaction type for our purposes
#[derive(Debug, Clone)]
pub enum ReactionType {
    Unicode { name: String },
    Custom { id: u64, name: Option<String>, animated: bool },
}

/// Discord events we handle
#[derive(Debug, Clone)]
pub enum DiscordEvent {
    /// New message created
    MessageCreate(MessageCreatePayload),

    /// Message edited
    MessageUpdate(MessageUpdatePayload),

    /// Message deleted
    MessageDelete(MessageDeletePayload),

    /// Reaction added
    ReactionAdd(ReactionAddPayload),

    /// Reaction removed
    ReactionRemove(ReactionRemovePayload),

    /// Channel created
    ChannelCreate(ChannelCreatePayload),

    /// Thread created
    ThreadCreate(ThreadCreatePayload),
}

impl DiscordEvent {
    /// Get the guild ID if present
    pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
        match self {
            Self::MessageCreate(p) => p.guild_id,
            Self::MessageUpdate(p) => p.guild_id,
            Self::MessageDelete(p) => p.guild_id,
            Self::ReactionAdd(p) => p.guild_id,
            Self::ReactionRemove(p) => p.guild_id,
            Self::ChannelCreate(p) => p.guild_id,
            Self::ThreadCreate(p) => p.guild_id,
        }
    }
}

/// Payload for MESSAGE_CREATE event
#[derive(Debug, Clone)]
pub struct MessageCreatePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message: Message,
}

/// Payload for MESSAGE_UPDATE event
#[derive(Debug, Clone)]
pub struct MessageUpdatePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message: Message,
}

/// Payload for MESSAGE_DELETE event
#[derive(Debug, Clone)]
pub struct MessageDeletePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
}

/// Payload for REACTION_ADD event
#[derive(Debug, Clone)]
pub struct ReactionAddPayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
    pub user_id: Id<UserMarker>,
    pub emoji: ReactionType,
}

/// Payload for REACTION_REMOVE event
#[derive(Debug, Clone)]
pub struct ReactionRemovePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
    pub user_id: Id<UserMarker>,
    pub emoji: ReactionType,
}

/// Payload for CHANNEL_CREATE event
#[derive(Debug, Clone)]
pub struct ChannelCreatePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel: Channel,
}

/// Payload for THREAD_CREATE event
#[derive(Debug, Clone)]
pub struct ThreadCreatePayload {
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel: Channel,
    pub parent_id: Id<ChannelMarker>,
}
