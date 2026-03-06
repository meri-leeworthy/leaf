//! # Sync Services
//!
//! This module contains the business logic for synchronizing different types of
//! data between Discord and Roomy.
//!
//! ## Services
//!
//! - [`MessageSyncService`]: Bidirectional message sync with edit tracking
//! - [`StructureSyncService`]: Channel, category, and thread synchronization
//! - [`ProfileSyncService`]: User profile and avatar sync with caching
//! - [`ReactionSyncService`]: Emoji reaction sync with idempotency
//!
//! ## Service Pattern
//!
//! Each service:
//! 1. Receives Discord event payloads
//! 2. Transforms them to Roomy events
//! 3. Stores ID mappings in the bridge database
//! 4. Sends events to Roomy via the dispatcher
//! 5. Handles idempotency and error recovery

mod message;
mod structure;
mod profile;
mod reaction;

pub use message::MessageSyncService;
pub use structure::StructureSyncService;
pub use profile::ProfileSyncService;
pub use reaction::ReactionSyncService;
