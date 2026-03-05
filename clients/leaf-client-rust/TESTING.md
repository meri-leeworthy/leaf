# Testing Guide for Rust Leaf Client

## Prerequisites

1. **Rust toolchain installed**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Leaf server running**:
   ```bash
   cd leaf
   cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token
   ```

   Or using Docker:
   ```bash
   docker run -it --rm -p 5530:5530 -v $(pwd)/data:/data leaf-server
   ```

## Test Suite Overview

### Unit Tests

Location: `src/*.rs` (inline in each module)

Run with:
```bash
cd leaf
cargo test -p leaf-client-rust
```

**What they test**:
- DID validation
- CBOR encoding/decoding round-trips
- Type serialization
- Basic API structure

**Current status**: ✅ 6/6 passing

### Integration Tests

Location: `tests/integration_test.rs`

Run with:
```bash
cd leaf
cargo test -p leaf-client-rust --test integration_test
```

Run specific test:
```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection
```

Run ignored tests (requires server):
```bash
cargo test -p leaf-client-rust --test integration_test -- --ignored
```

**What they test**:
- `test_client_connection` - Basic connection and disconnect
- `test_upload_and_check_module` - Module upload and existence check
- `test_create_stream` - Stream creation with module
- `test_query_execution` - Query execution protocol
- `test_send_events` - Sending events to streams
- `test_send_state_events` - Sending state events
- `test_update_stream_module` - Updating stream's module
- `test_set_handle` - Setting and clearing handles
- `test_clear_state` - Clearing stream state
- `test_event_subscription` - Subscribing to query events
- `test_error_handling` - Error scenarios
- `test_concurrent_operations` - Concurrent stream creation
- `test_did_validation` - DID format validation

**Current status**: 🚧 Need running server

## Running Tests

### 1. Run All Unit Tests (No Server Required)

```bash
cd leaf
cargo test -p leaf-client-rust --lib
```

Expected output:
```
running 6 tests
test types::tests::test_did_validation ... ok
test types::tests::test_sql_value_serialization ... ok
test codec::tests::test_encode_decode_sql_value ... ok
test codec::tests::test_encode_decode_with_bytes ... ok
test tests::test_library_builds ... ok
test lib::tests::test_library_builds ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 2. Run Integration Tests (Requires Server)

First, start the server in one terminal:
```bash
cd leaf
cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token
```

Then in another terminal:
```bash
cd leaf
cargo test -p leaf-client-rust --test integration_test -- --ignored
```

### 3. Run Specific Test

```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored
```

### 4. Run with Output

```bash
cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

## Test Markers

Tests are marked with `#[ignore]` by default to prevent them from running without a server.

To run only non-ignored tests (unit tests only):
```bash
cargo test -p leaf-client-rust
```

To run only integration tests (ignored tests):
```bash
cargo test -p leaf-client-rust -- --ignored
```

## Debugging Failed Tests

### Enable Logging

Set environment variable for debug output:
```bash
RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

### Single-Step Execution

Run one test at a time:
```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored --nocapture
```

### GDB Support

For low-level debugging:
```bash
cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Test Rust Leaf Client

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test -p leaf-client-rust --lib
      - name: Start Leaf server
        run: |
          cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token &
          sleep 10
      - name: Run integration tests
        run: cargo test -p leaf-client-rust --test integration_test -- --ignored
```

## Benchmarks

Run performance benchmarks:
```bash
cd leaf
cargo bench -p leaf-client-rust
```

Benchmark results:
- CBOR encoding/decoding speed
- Query serialization performance
- Large payload handling

## Manual Testing

### Interactive Demo

Run the CBOR demo:
```bash
cd leaf
cargo run -p leaf-client-rust --example cbor_demo
```

Run the API demo:
```bash
cd leaf
cargo run -p leaf-client-rust --example client_demo
```

### Custom Test Script

Create a custom test script in `examples/test_custom.rs`:

```rust
use leaf_client_rust::{LeafClient, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LeafClient::connect(
        "http://leaf-server:5530",
        None,
    ).await?;

    // Your custom test code here

    client.disconnect().await?;
    Ok(())
}
```

Run with:
```bash
cargo run -p leaf-client-rust --example test_custom
```

## Test Coverage

Install tarpaulin:
```bash
cargo install cargo-tarpaulin
```

Generate coverage report:
```bash
cd leaf
cargo tarpaulin -p leaf-client-rust --out Html
```

## Common Issues

### "Connection Refused"

**Problem**: Test can't connect to server

**Solution**:
- Ensure server is running: `curl leaf-server:5530`
- Check server URL in `TEST_SERVER_URL`
- Verify firewall settings

### "Authentication Failed"

**Problem**: Server rejects connection

**Solution**:
- Start server with `--unsafe-auth-token test-token`
- Or implement proper authentication in client

### "Timeout"

**Problem**: Test takes too long

**Solution**:
- Increase timeout in client
- Check server logs for errors
- Verify network connectivity

### "CBOR Decode Error"

**Problem**: Can't parse server response

**Solution**:
- Check server version matches expected protocol
- Verify CBOR encoding matches TypeScript client
- Add debug logging to see raw bytes

## Next Steps

1. ✅ Unit tests - Complete
2. 🚧 Integration tests - Need running server
3. 📋 Performance benchmarks - Ready to run
4. 📋 Property-based tests - Not implemented
5. 📋 Fuzzing tests - Not implemented

## Contributing

When adding new features:

1. Write unit tests first (TDD)
2. Add integration test for server interaction
3. Update this guide with new test instructions
4. Ensure all tests pass before committing

Example TDD workflow:
```bash
# 1. Write failing test
cargo test -p leaf-client-rust test_new_feature

# 2. Implement feature

# 3. Test passes
cargo test -p leaf-client-rust test_new_feature

# 4. Run all tests to ensure no regression
cargo test -p leaf-client-rust
```

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Leaf Protocol Docs](../../docs/)
- [TypeScript Client Tests](../typescript/)
