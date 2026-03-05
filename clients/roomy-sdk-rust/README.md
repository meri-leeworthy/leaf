# Roomy SDK for Rust

Rust SDK for [Roomy](https://github.com/muni-town/roomy) - a federated collaboration platform built on AT Protocol.

## Overview

This SDK provides a Rust interface for interacting with Roomy, including:

- ✅ ATProto authentication (app password login)
- ✅ Leaf server connection and authentication
- ✅ Event streaming and subscription
- ✅ Message operations (create, edit, delete)
- ✅ Space queries and management
- ✅ Blob uploads

## Status

🚧 **Under active development** - This is a work in progress. Not ready for production use.

## Architecture

The SDK is built on top of:

- **[jacquard](https://github.com/rsform/jacquard)** - AT Protocol client implementation
- **[leaf-client-rust](../leaf-client-rust/)** - Leaf WebSocket client

### Module Structure

```
roomy-sdk-rust/
├── atproto/       # Wrapper around jacquard for ATProto operations
├── events/        # Roomy event types and CBOR codec
├── client/        # High-level RoomyClient
└── connection/    # ConnectedSpace for event streaming
```

## Quick Start

```rust
use roomy_sdk_rust::{RoomyClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let client = RoomyClient::create(
        "https://bsky.social",
        "your-did",
        "your-app-password",
        "https://leaf.example.com",
        "did:web:leaf.example.com",
    ).await?;

    // Connect to personal stream
    let personal_space = client.connect_personal_space().await?;

    // Subscribe to events
    let mut event_rx = personal_space.subscribe().await?;

    while let Some(event) = event_rx.recv().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}
```

## Development

### Building

```bash
cd clients/roomy-sdk-rust
cargo build
```

### Running Tests

```bash
cargo test
```

### Examples

See the `examples/` directory for usage examples:

- `basic_client.rs` - Basic client setup and authentication
- `send_message.rs` - Sending a message to a room
- `subscribe_events.rs` - Subscribing to event streams

## Dependencies

- `jacquard` - AT Protocol client
- `leaf-client-rust` - Leaf WebSocket client
- `tokio` - Async runtime
- `serde` - Serialization
- `ciborium` - CBOR encoding

## License

MPL-2.0

## Related

- [Roomy Monorepo](https://github.com/muni-town/roomy)
- [Leaf Server](https://github.com/muni-town/leaf)
- [AT Protocol](https://atproto.com)
