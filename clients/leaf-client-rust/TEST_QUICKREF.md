# Rust Leaf Client - Quick Testing Reference

## Quick Start

```bash
cd leaf

# Run unit tests (no server needed)
cargo test -p leaf-client-rust --lib

# Run integration tests (server required)
cargo test -p leaf-client-rust --test integration_test -- --ignored

# Run all tests
./clients/leaf-client-rust/scripts/test.sh all

# Run benchmarks
cargo bench -p leaf-client-rust
```

## Test Script Commands

The helper script at `scripts/test.sh` provides convenient commands:

```bash
cd leaf/clients/leaf-client-rust
./scripts/test.sh [command]
```

### Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `unit` | `u` | Run unit tests (no server) |
| `integration` | `i` | Run integration tests (needs server) |
| `all` | `a` | Run all tests |
| `bench` | `b` | Run benchmarks |
| `watch` | `w` | Watch mode - rerun on changes |
| `coverage` | `c` | Generate coverage report |
| `doc` | `d` | Run documentation tests |
| `examples` | `e` | Run example programs |
| `check` | `ch` | Check compilation |
| `clippy` | `l` | Run Clippy linter |
| `format` | `f` | Check formatting |
| `format-fix` | - | Fix formatting |

## Server Setup

### Start Server for Testing

**Option 1: Cargo**
```bash
cd leaf
cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token
```

**Option 2: Docker**
```bash
docker run -it --rm -p 5530:5530 -v $(pwd)/data:/data leaf-server
```

### Verify Server Running

```bash
curl leaf-server:5530
# Should output: "Leaf Server API"
```

## Individual Tests

### Run Specific Integration Test

```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored
```

### Run with Output

```bash
cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

### Run with Logging

```bash
RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

## Test List

### Unit Tests (No Server)

- `test_did_validation` - DID format validation
- `test_sql_value_serialization` - SQL value type tags
- `test_encode_decode_sql_value` - CBOR round-trip
- `test_encode_decode_with_bytes` - Binary data encoding
- `test_library_builds` - Compilation test

### Integration Tests (Needs Server)

- `test_client_connection` - Basic connection
- `test_upload_and_check_module` - Module operations
- `test_create_stream` - Stream creation
- `test_query_execution` - Query protocol
- `test_send_events` - Event sending
- `test_send_state_events` - State events
- `test_update_stream_module` - Module updates
- `test_set_handle` - Handle management
- `test_clear_state` - State clearing
- `test_event_subscription` - Subscription system
- `test_error_handling` - Error cases
- `test_concurrent_operations` - Concurrency
- `test_did_validation` - DID validation

## Examples

```bash
# CBOR encoding demo
cargo run -p leaf-client-rust --example cbor_demo

# API usage demo
cargo run -p leaf-client-rust --example client_demo
```

## Benchmarks

```bash
cargo bench -p leaf-client-rust
```

Measures:
- SQL value encoding/decoding
- Query serialization
- Large query performance

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run unit tests
  run: cargo test -p leaf-client-rust --lib

- name: Start server
  run: cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token &

- name: Run integration tests
  run: cargo test -p leaf-client-rust --test integration_test -- --ignored
```

## Troubleshooting

### "Connection Refused"
- Server not running
- Wrong URL (check `TEST_SERVER_URL` in test file)

### "Timeout"
- Server too slow
- Network issues
- Increase timeout in test

### "CBOR Decode Error"
- Protocol mismatch
- Server version incompatible
- Check TypeScript client for reference

### Tests Not Found
- Wrong package name: use `-p leaf-client-rust`
- Wrong test file: check `tests/` directory

## Development Workflow

```bash
# 1. Make changes
vim src/client.rs

# 2. Check compilation
cargo check -p leaf-client-rust

# 3. Run unit tests
cargo test -p leaf-client-rust --lib

# 4. Start server in another terminal
cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token

# 5. Run integration tests
cargo test -p leaf-client-rust --test integration_test -- --ignored

# 6. Check formatting
cargo fmt -p leaf-client-rust -- --check

# 7. Run linter
cargo clippy -p leaf-client-rust -- -D warnings

# 8. Run all checks
./clients/leaf-client-rust/scripts/test.sh all
```

## Performance Testing

Compare Rust vs TypeScript:

```bash
# Rust benchmarks
cargo bench -p leaf-client-rust

# Note: Need equivalent TypeScript benchmarks for comparison
```

Key metrics:
- Throughput (operations/sec)
- Latency (ms)
- Memory usage (MB)
- Binary size

## Documentation

- Full testing guide: `TESTING.md`
- API docs: `cargo doc -p leaf-client-rust --open`
- Progress: `PROGRESS.md`
- Phase 2 complete: `PHASE2_COMPLETE.md`
