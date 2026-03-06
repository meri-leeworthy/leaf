//! Discord bot client wrapper

use twilight_gateway::{
    stream::ShardEventStream,
    Intents, Shard, ShardId,
};
use futures::StreamExt;
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
        let current_user = http.current_user().await?.model().await?;
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
        let mut shard = Shard::new(ShardId::ONE, token, intents);

        // Spawn gateway event handler
        let http_clone = Arc::clone(&http);
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            Self::run_gateway(&mut shard, http_clone, event_tx_clone).await;
        });

        Ok((Self {
            http,
            id: bot_id,
            event_rx,
        }, event_tx))
    }

    /// Run the gateway event loop
    async fn run_gateway(
        shard: &mut Shard,
        http: Arc<Client>,
        event_tx: mpsc::Sender<DiscordEvent>,
    ) {
        let mut stream = ShardEventStream::new(std::iter::once(shard));

        while let Some(item) = stream.next().await {
            let (_shard_ref, event_result) = item;

            let event = match event_result {
                Ok(event) => event,
                Err(source) => {
                    tracing::error!("error processing event: {:?}", source);
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

    /// Convert Twilight ReactionType to our custom ReactionType
    fn convert_reaction_type(emoji: &twilight_model::channel::message::ReactionType) -> crate::bot::events::ReactionType {
        match emoji {
            twilight_model::channel::message::ReactionType::Unicode { name } => {
                crate::bot::events::ReactionType::Unicode {
                    name: name.clone(),
                }
            }
            twilight_model::channel::message::ReactionType::Custom {
                id,
                name,
                animated,
            } => crate::bot::events::ReactionType::Custom {
                id: id.get(),
                name: name.clone(),
                animated: *animated,
            },
        }
    }

    /// Convert Twilight event to our DiscordEvent
    async fn convert_event(event: &Event, _http: &Client) -> Option<DiscordEvent> {
        match event {
            // Message created
            Event::MessageCreate(msg) => {
                let guild_id = msg.guild_id;
                Some(DiscordEvent::MessageCreate(
                    crate::bot::MessageCreatePayload {
                        guild_id,
                        channel_id: msg.channel_id,
                        message: msg.0.clone(),
                    },
                ))
            }

            // Message updated
            Event::MessageUpdate(msg) => {
                let _guild_id = msg.guild_id;
                // For updates, we need to fetch the full message
                // TODO: Implement message fetching
                None
            }

            // Message deleted
            Event::MessageDelete(msg) => {
                let guild_id = msg.guild_id;
                Some(DiscordEvent::MessageDelete(
                    crate::bot::MessageDeletePayload {
                        guild_id,
                        channel_id: msg.channel_id,
                        message_id: msg.id,
                    },
                ))
            }

            // Reaction added
            Event::ReactionAdd(reaction) => {
                let guild_id = reaction.guild_id;
                Some(DiscordEvent::ReactionAdd(
                    crate::bot::ReactionAddPayload {
                        guild_id,
                        channel_id: reaction.channel_id,
                        message_id: reaction.message_id,
                        user_id: reaction.user_id,
                        emoji: Self::convert_reaction_type(&reaction.emoji),
                    },
                ))
            }

            // Reaction removed
            Event::ReactionRemove(reaction) => {
                let guild_id = reaction.guild_id;
                Some(DiscordEvent::ReactionRemove(
                    crate::bot::ReactionRemovePayload {
                        guild_id,
                        channel_id: reaction.channel_id,
                        message_id: reaction.message_id,
                        user_id: reaction.user_id,
                        emoji: Self::convert_reaction_type(&reaction.emoji),
                    },
                ))
            }

            // Channel created
            Event::ChannelCreate(channel) => {
                let guild_id = channel.guild_id;
                Some(DiscordEvent::ChannelCreate(
                    crate::bot::ChannelCreatePayload {
                        guild_id,
                        channel: channel.0.clone(),
                    },
                ))
            }

            // Thread created
            Event::ThreadCreate(thread) => {
                let guild_id = thread.guild_id;
                let parent_id = thread.parent_id;
                Some(DiscordEvent::ThreadCreate(
                    crate::bot::ThreadCreatePayload {
                        guild_id,
                        channel: thread.0.clone(),
                        parent_id: parent_id.unwrap(),
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
