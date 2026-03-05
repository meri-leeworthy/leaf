//! Discord bot client wrapper

use twilight_gateway::{
    stream::{ShardEventStream, StreamExt},
    Intents, Shard,
};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::{DiscordEvent, Result};

/// Discord bot client
pub struct DiscordBot {
    /// HTTP client for API calls
    pub http: Arc<Client>,

    /// Bot's user ID
    pub id: u64,

    /// Event receiver
    event_rx: mpsc::Receiver<DiscordEvent>,
}

impl DiscordBot {
    /// Create a new Discord bot
    ///
    /// # Arguments
    /// * `token` - Discord bot token
    ///
    /// # Returns
    /// The bot instance and a channel sender for events
    pub async fn new(token: String) -> Result<(Self, mpsc::Sender<DiscordEvent>)> {
        // Create HTTP client
        let http = Arc::new(Client::new(token.clone()));

        // Get bot user info
        let current_user = http.current_user().exec().await?.model().await?;
        let bot_id = current_user.id.get();

        // Create event channel
        let (event_tx, event_rx) = mpsc::channel(100);

        // Configure intents - we need:
        // - Guilds: for channel updates
        // - GuildMessages: for message events
        // - MessageContent: to read message content
        // - GuildMessageReactions: for reaction events
        let intents = Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_MESSAGE_REACTIONS;

        // Create shard
        let shard = Shard::new(token, intents);

        // Spawn gateway event handler
        let http_clone = Arc::clone(&http);
        tokio::spawn(async move {
            Self::run_gateway(shard, http_clone, event_tx).await;
        });

        Ok(Self {
            http,
            id: bot_id,
            event_rx,
        })
    }

    /// Run the gateway event loop
    async fn run_gateway(
        mut shard: Shard,
        http: Arc<Client>,
        event_tx: mpsc::Sender<DiscordEvent>,
    ) {
        let mut stream = ShardEventStream::new(shard.info());

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(source) => {
                    tracing::error!("error receiving event: {:?}", source);
                    continue;
                }
            };

            // Convert Twilight event to our DiscordEvent type
            if let Some(discord_event) = Self::convert_event(&event, &http).await {
                if let Err(e) = event_tx.send(discord_event).await {
                    tracing::error!("error sending event to channel: {}", e);
                }
            }
        }
    }

    /// Convert Twilight event to our DiscordEvent
    async fn convert_event(event: &Event, _http: &Client) -> Option<DiscordEvent> {
        match event {
            // Message created
            Event::MessageCreate(msg) => {
                let guild_id = msg.0.guild_id;
                Some(DiscordEvent::MessageCreate(
                    crate::bot::MessageCreatePayload {
                        guild_id,
                        channel_id: msg.0.channel_id,
                        message: msg.0.clone(),
                    },
                ))
            }

            // Message updated
            Event::MessageUpdate(msg) => {
                let guild_id = msg.0.guild_id;
                // For updates, we need to fetch the full message
                // TODO: Implement message fetching
                None
            }

            // Message deleted
            Event::MessageDelete(msg) => {
                let guild_id = msg.0.guild_id;
                Some(DiscordEvent::MessageDelete(
                    crate::bot::MessageDeletePayload {
                        guild_id,
                        channel_id: msg.0.channel_id,
                        message_id: msg.0.id,
                    },
                ))
            }

            // Reaction added
            Event::ReactionAdd(reaction) => {
                let guild_id = reaction.0.guild_id;
                Some(DiscordEvent::ReactionAdd(
                    crate::bot::ReactionAddPayload {
                        guild_id,
                        channel_id: reaction.0.channel_id,
                        message_id: reaction.0.message_id,
                        user_id: reaction.0.user_id,
                        emoji: reaction.0.emoji.clone(),
                    },
                ))
            }

            // Reaction removed
            Event::ReactionRemove(reaction) => {
                let guild_id = reaction.0.guild_id;
                Some(DiscordEvent::ReactionRemove(
                    crate::bot::ReactionRemovePayload {
                        guild_id,
                        channel_id: reaction.0.channel_id,
                        message_id: reaction.0.message_id,
                        user_id: reaction.0.user_id,
                        emoji: reaction.0.emoji.clone(),
                    },
                ))
            }

            // Channel created
            Event::ChannelCreate(channel) => {
                let guild_id = channel.0.guild_id;
                Some(DiscordEvent::ChannelCreate(
                    crate::bot::ChannelCreatePayload {
                        guild_id,
                        channel: channel.0.clone(),
                    },
                ))
            }

            // Thread created
            Event::ThreadCreate(thread) => {
                let guild_id = thread.0.guild_id;
                let parent_id = thread.0.parent_id?;
                Some(DiscordEvent::ThreadCreate(
                    crate::bot::ThreadCreatePayload {
                        guild_id,
                        channel: thread.0.clone(),
                        parent_id,
                    },
                ))
            }

            _ => None,
        }
    }

    /// Get the event receiver
    pub fn event_receiver(&mut self) -> &mut mpsc::Receiver<DiscordEvent> {
        &mut self.event_rx
    }
}
