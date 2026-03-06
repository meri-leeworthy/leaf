//! Event dispatcher for bidirectional event routing

use roomy_sdk_rust::{Event, Ulid};
use tokio::sync::mpsc;

/// Queued Roomy event for processing
pub struct QueuedRoomyEvent {
    /// The decoded event
    pub decoded: Event,
    /// Batch ID for tracking
    pub batch_id: Ulid,
    /// Whether this is the last event in the batch
    pub is_last_event: bool,
}

/// Event dispatcher for routing events between Discord and Roomy
pub struct EventDispatcher {
    /// Discord → Roomy events
    pub to_roomy: mpsc::Sender<RoomySdkEvent>,

    /// Roomy → Discord events (queued during backfill)
    pub to_discord: mpsc::Sender<QueuedRoomyEvent>,
}

/// Event we send to Roomy (alias for convenience)
pub type RoomySdkEvent = Event;

impl EventDispatcher {
    /// Create a new event dispatcher with channels
    pub fn new(
        buffer: usize,
    ) -> (Self, mpsc::Receiver<RoomySdkEvent>, mpsc::Receiver<QueuedRoomyEvent>) {
        let (to_roomy_tx, to_roomy_rx) = mpsc::channel(buffer);
        let (to_discord_tx, to_discord_rx) = mpsc::channel(buffer);

        (
            Self {
                to_roomy: to_roomy_tx,
                to_discord: to_discord_tx,
            },
            to_roomy_rx,
            to_discord_rx,
        )
    }

    /// Send event to Roomy
    pub async fn send_to_roomy(&self, event: RoomySdkEvent) -> Result<(), mpsc::error::SendError<RoomySdkEvent>> {
        self.to_roomy.send(event).await
    }

    /// Queue event for Discord sync
    pub async fn queue_for_discord(&self, event: QueuedRoomyEvent) -> Result<(), mpsc::error::SendError<QueuedRoomyEvent>> {
        self.to_discord.send(event).await
    }

    /// Close the dispatcher channels (signal shutdown)
    ///
    /// Consumes the dispatcher to drop the senders and close the channels.
    pub fn close(self) {
        // Drop the senders to close the channels
        drop(self.to_roomy);
        drop(self.to_discord);
    }
}
