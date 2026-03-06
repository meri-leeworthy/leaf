//! # Discord Bot Client
//!
//! This module provides the Discord bot client using the Twilight library.
//!
//! ## Components
//!
//! - [`DiscordBot`]: Main bot client that connects to Discord via WebSocket
//! - [`DiscordEvent`]: Enum representing all Discord events we handle
//! - Event payload types: Structured representations of Discord event data
//!
//! ## Usage
//!
//! The bot connects to Discord using a bot token, receives events via WebSocket,
//! and emits them through a channel for processing by the sync services.

mod client;
pub mod events;

pub use client::DiscordBot;
pub use events::{
    DiscordEvent,
    MessageCreatePayload,
    MessageUpdatePayload,
    MessageDeletePayload,
    ReactionAddPayload,
    ReactionRemovePayload,
    ChannelCreatePayload,
    ThreadCreatePayload,
};
