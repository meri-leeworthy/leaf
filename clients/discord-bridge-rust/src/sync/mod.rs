//! # Bridge Synchronization Orchestration
//!
//! This module coordinates the synchronization between Discord and Roomy.
//!
//! ## Components
//!
//! - [`BridgeOrchestrator`]: High-level coordinator that manages sync services
//! - [`Bridge`]: Per-bridge state machine connecting Discord guild to Roomy space
//! - [`EventDispatcher`]: Sends events to Roomy with queuing and retry logic
//!
//! ## Architecture
//!
//! 1. **Orchestrator** receives Discord events from the bot
//! 2. **Bridge** delegates to appropriate sync service (message, reaction, etc.)
//! 3. **Service** processes event and generates Roomy event
//! 4. **Dispatcher** queues and sends event to Leaf server
//! 5. **Repository** stores mappings and state in local database

pub mod dispatcher;
mod bridge;
mod orchestrator;

pub use dispatcher::EventDispatcher;
pub use bridge::Bridge;
pub use orchestrator::BridgeOrchestrator;
