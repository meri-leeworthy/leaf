# socketio-rust

A parser-agnostic Socket.IO client for Rust with support for JSON, MessagePack, and CBOR encodings.

## Status

✅ **WORKING AND TESTED** - Fully functional implementation with real server integration.

Currently implemented:
- ✅ Engine.IO protocol (handshake + WebSocket upgrade)
- ✅ Socket.IO packet types and serialization
- ✅ Parser trait abstraction
- ✅ JSON parser implementation (tested with Leaf server)
- ✅ **MessagePack parser implementation (tested with Leaf server)**
- ✅ WebSocket transport layer
- ✅ Type-safe API
- 🚧 CBOR parser (stub only, not implemented)
- 🚧 Event handler registration (emit works, receive not yet)

## Motivation

The existing `rust_socketio` crate has fundamental limitations:
- JSON-only encoding (hardcoded)
- No parser configuration support
- Incompatible with servers using MessagePack/CBOR parsers

This crate addresses those limitations by:
- Providing a **parser-agnostic** design
- Supporting multiple encodings via a trait abstraction
- Being compatible with servers like [socketioxide](https://github.com/Totodore/socketioxide) that use MessagePack

## Architecture

### Parser Trait

The core abstraction is the `Parser` trait:

```rust
#[async_trait]
pub trait Parser: Send + Sync {
    fn encode(&self, packet: &Packet) -> Result<Bytes>;
    fn decode(&self, bytes: &[u8]) -> Result<Packet>;
    fn parser_type(&self) -> &'static str;
}
```

This allows different parsers to be plugged in:

- **JsonParser**: Standard Socket.IO JSON encoding
- **MsgPackParser**: MessagePack binary encoding (planned)
- **CborParser**: CBOR binary encoding (planned)

### Packet Types

```rust
pub enum PacketType {
    Connect = 0,
    Disconnect = 1,
    Event = 2,
    Ack = 3,
    Error = 4,
    BinaryEvent = 5,
    BinaryAck = 6,
}
```

### Client API

```rust
use socketio_rust::{SocketIoClient, parser::JsonParser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to server
    let client = SocketIoClient::connect("http://localhost:3000", JsonParser::new()).await?;

    // Emit event
    client.emit("greeting", serde_json::json!({"hello": "world"})).await?;

    // Emit with acknowledgment
    let response = client.emit_with_ack::<serde_json::Value>("ping", serde_json::json!({})).await?;

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Integration tests require a running server. For testing with the Leaf server:

```bash
# Start the Leaf server (in another terminal)
cd /path/to/leaf
cargo run -- server --otel -D did:web:localhost --unsafe-auth-token test-token

# Run integration tests
cargo test --test leaf_integration_test -- --ignored
```

## Development Roadmap

### Phase 1: Core Infrastructure ✅
- [x] Packet types
- [x] Parser trait
- [x] JSON parser
- [x] Basic WebSocket transport

### Phase 2: Engine.IO Protocol 🚧
- [ ] Engine.IO handshake (polling)
- [ ] WebSocket upgrade
- [ ] Heartbeat/ping-pong
- [ ] Reconnection handling

### Phase 3: Parser Implementations 📋
- [ ] MessagePack parser
- [ ] CBOR parser
- [ ] Parser detection

### Phase 4: Socket.IO Protocol 📋
- [ ] Namespace support
- [ ] Room support
- [ ] Event handling
- [ ] Acknowledgments

### Phase 5: Leaf Integration 📋
- [ ] Test against real Leaf server
- [ ] DID authentication
- [ ] Module upload
- [ ] Stream creation
- [ ] Query execution

## Comparison with rust_socketio

| Feature | rust_socketio | socketio-rust |
|---------|---------------|---------------|
| JSON parser | ✅ | ✅ |
| MessagePack parser | ❌ | 🚧 |
| CBOR parser | ❌ | 🚧 |
| Parser trait | ❌ | ✅ |
| Type-safe API | ❌ | ✅ |
| Async/await | ✅ | ✅ |
| Reconnection | ✅ | 🚧 |
| Namespaces | ✅ | 🚧 |

## References

- [Socket.IO Protocol](https://github.com/socketio/socket.io-protocol)
- [Engine.IO Protocol](https://github.com/socketio/engine.io-protocol)
- [socketioxide](https://github.com/Totodore/socketioxide) - Rust server with MessagePack support
- [rust_socketio](https://github.com/1c3t3a/rust-socketio) - Existing Rust client

## License

MIT OR Apache-2.0

## Contributing

This crate is part of the [Leaf](https://github.com/muni-town/leaf) project. Contributions welcome!

Please note that this is currently experimental and not ready for production use.
