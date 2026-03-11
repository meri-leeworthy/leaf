# Leaf Client Rust - Disconnect Issue Investigation

## Executive Summary

**Issue**: The Rust Socket.IO client fails to disconnect cleanly with error:
```
IncompleteResponseFromEngineIo(WebsocketError(AlreadyClosed))
```

**Root Cause**: The WebSocket connection is being closed prematurely by the transport layer, likely due to a mismatch in ping/pong heartbeat configuration between the client and server.

**Impact**: Integration tests fail on `disconnect()`, but the actual client operations (connect, upload modules, create streams, etc.) work correctly.

## Investigation Results

### Test Matrix

| Test | Result | Key Finding |
|------|--------|-------------|
| Immediate disconnect (< 1ms) | ✅ Pass | Disconnect succeeds if called immediately |
| Disconnect after 100ms | ❌ Fail | WebSocket already closed |
| Disconnect after 2s | ❌ Fail | WebSocket already closed |
| Multiple cycles | ⚠️ Intermittent | Non-deterministic: Fail/Pass/Fail pattern |
| Explicit WebSocket transport | ❌ Fail | `SendAfterClosing` error |
| With event handler | ❌ Fail | Same `AlreadyClosed` error |

### Error Analysis

**Primary Error**:
```
IncompleteResponseFromEngineIo(WebsocketError(AlreadyClosed))
```

**Secondary Error** (with explicit WebSocket transport):
```
IncompleteResponseFromEngineIo(WebsocketError(Protocol(SendAfterClosing)))
```

**Interpretation**:
1. The WebSocket transport layer closes the connection after a short timeout
2. rust_socketio tries to send a proper disconnect message via the already-closed socket
3. This fails because you can't send data on a closed connection

## Hypothesis Generation

### Hypothesis 1: Heartbeat/Ping Timeout Mismatch ⭐ (Most Likely)

**Theory**: The server expects regular ping/pong heartbeats to keep the connection alive. rust_socketio might not be sending these, or the interval is misconfigured.

**Evidence**:
- Immediate disconnect works (no time for heartbeat)
- Delayed disconnect fails (heartbeat timeout triggered)
- Non-deterministic pattern suggests timing-dependent behavior

**Confidence**: 75%

**Test Plan**:
- [ ] Check socketioxide server default ping interval/timeout
- [ ] Check rust_socketio client default ping interval/timeout
- [ ] Configure rust_socketio to match server's expected heartbeat
- [ ] Test with explicit ping configuration

### Hypothesis 2: WebSocket vs Polling Transport Negotiation

**Theory**: The connection is being downgraded from WebSocket to polling, causing issues.

**Evidence**:
- Explicit WebSocket transport shows different error (`SendAfterClosing`)
- Default behavior (transport auto-detection) shows `AlreadyClosed`

**Confidence**: 40%

**Test Plan**:
- [ ] Force polling transport to see if issue persists
- [ ] Monitor which transport is actually being used
- [ ] Test with polling-only configuration

### Hypothesis 3: Server-Side Connection Cleanup

**Theory**: The Leaf server is proactively closing idle connections.

**Evidence**:
- 100ms delay is enough to trigger the issue
- The timeout is very short (< 1 second)

**Confidence**: 20%

**Test Plan**:
- [ ] Check Leaf server connection lifecycle management
- [ ] Look for connection timeout/keepalive settings
- [ ] Review socketioxide configuration

### Hypothesis 4: rust_socketio Bug

**Theory**: rust_socketio v0.6.0 has a bug in disconnect handling.

**Evidence**:
- TypeScript client has no issues (fire-and-forget disconnect)
- Error indicates trying to send after close

**Confidence**: 60%

**Test Plan**:
- [ ] Check rust_socketio GitHub issues for similar reports
- [ ] Try upgrading to latest version (if available)
- [ ] Review rust_socketio disconnect implementation

## Recommended Solutions

### Option 1: Make Disconnect Lenient (Quick Fix)

**Approach**: Change `disconnect()` to not fail on websocket errors.

```rust
pub async fn disconnect(self) -> Result<()> {
    match self.socket.disconnect().await {
        Ok(_) => Ok(()),
        Err LeafClientError::Socket(ref msg) if msg.contains("AlreadyClosed") => {
            // Already disconnected - not really an error
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

**Pros**:
- Quick to implement
- Matches TypeScript behavior (fire-and-forget)
- Tests will pass

**Cons**:
- Doesn't fix root cause
- Masks potential real issues

### Option 2: Fix Heartbeat Configuration (Proper Fix)

**Approach**: Configure rust_socketio to send proper ping/pong heartbeats.

```rust
let socket = ClientBuilder::new(url)
    .ping_interval(25000)  // Send ping every 25s
    .ping_timeout(60000)   // Wait 60s for pong response
    .connect()
    .await?;
```

**Pros**:
- Fixes root cause
- Robust connection handling
- Prevents other heartbeat-related issues

**Cons**:
- Requires research into correct values
- May need server-side changes too

### Option 3: Investigate Server Configuration (Collaborative Fix)

**Approach**: Work with Leaf server to ensure compatibility.

**Pros**:
- Benefits all clients
- Addresses architectural issue

**Cons**:
- Higher coordination overhead
- May not be under our control

## Next Steps

1. **Immediate**: Implement Option 1 (lenient disconnect) to unblock testing
2. **Short-term**: Research and implement Option 2 (heartbeat config)
3. **Long-term**: Collaborate with Leaf server team on Option 3 if needed

## Related Issues

- TypeScript client has had "problematic behaviour around connection/disconnection"
- This suggests the Leaf server's connection management may need improvement
- Rust client has opportunity to handle this more robustly

## Files Referenced

- `clients/leaf-client-rust/src/client.rs` - LeafClient implementation
- `clients/leaf-client-rust/tests/disconnect_minimal.rs` - Minimal reproduction tests
- `clients/leaf-client-rust/tests/transport_analysis.rs` - Transport-specific tests
- `leaf-server/src/http.rs` - Server Socket.IO setup
- `clients/typescript/src/index.ts` - TypeScript client reference implementation
