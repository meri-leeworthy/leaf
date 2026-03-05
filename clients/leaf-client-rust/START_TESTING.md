# 🚀 Quick Start: Running the Rust Leaf Client Tests

## Prerequisites Check

```bash
# 1. Verify Rust is installed
rustc --version
cargo --version

# If not installed:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Verify server is running
curl http://leaf-server:5530
# Should output: "Leaf Server API"
```

## One-Line Test Commands

### Unit Tests (No Server Required - Fast!)
```bash
cd leaf && cargo test -p leaf-client-rust --lib
```

### Integration Tests (With Server - Complete!)
```bash
cd leaf && cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

### All Tests Using Helper Script
```bash
cd leaf/clients/leaf-client-rust && ./scripts/test.sh all
```

## Step-by-Step Guide

### Option 1: Unit Tests Only (Fastest)

```bash
# Navigate to leaf directory
cd leaf

# Run unit tests
cargo test -p leaf-client-rust --lib

# Expected output:
# running 6 tests
# test types::tests::test_did_validation ... ok
# test types::tests::test_sql_value_serialization ... ok
# test codec::tests::test_encode_decode_sql_value ... ok
# test codec::tests::test_encode_decode_with_bytes ... ok
# test tests::test_library_builds ... ok
# test lib::tests::test_library_builds ... ok
#
# test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

**Time**: ~1 second
**Server needed**: No
**Tests**: 6

### Option 2: Full Test Suite (Recommended)

#### Step 1: Start the Server (Terminal 1)

```bash
# Using cargo
cd leaf
cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token

# Or using Docker
docker run -it --rm -p 5530:5530 -v $(pwd)/data:/data leaf-server
```

#### Step 2: Run Tests (Terminal 2)

```bash
# Using helper script (easiest)
cd leaf/clients/leaf-client-rust
./scripts/test.sh all

# Or manually
cd leaf
cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

**Time**: ~30 seconds
**Server needed**: Yes
**Tests**: 19 (6 unit + 13 integration)

### Option 3: Individual Test Categories

#### Unit Tests Only
```bash
cargo test -p leaf-client-rust --lib
```

#### Integration Tests Only
```bash
cargo test -p leaf-client-rust --test integration_test -- --ignored
```

#### Specific Integration Test
```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored
```

#### With Debug Output
```bash
RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

## Helper Script Commands

The `scripts/test.sh` script provides easy commands:

```bash
cd leaf/clients/leaf-client-rust

# Unit tests (no server)
./scripts/test.sh unit

# Integration tests (with server)
./scripts/test.sh integration

# All tests
./scripts/test.sh all

# Benchmarks
./scripts/test.sh bench

# Watch mode (rerun on changes)
./scripts/test.sh watch

# Coverage report
./scripts/test.sh coverage

# Run examples
./scripts/test.sh examples

# Check code
./scripts/test.sh check

# Run linter
./scripts/test.sh clippy

# Check formatting
./scripts/test.sh format

# Fix formatting
./scripts/test.sh format-fix
```

## Understanding Test Output

### Success Output
```
running 13 tests
test test_client_connection ... ok
test test_upload_and_check_module ... ok
test test_create_stream ... ok
...

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured
```

### Failure Output
```
running 13 tests
test test_client_connection ... FAILED

failures:

---- test_client_connection stdout ----
thread 'test_client_connection' panicked at 'assertion failed: `(left == right)`
...

test result: FAILED. 12 passed; 1 failed; 0 ignored
```

## Troubleshooting

### "Connection Refused"

**Problem**: Can't connect to server

**Solutions**:
```bash
# Check if server is running
curl http://leaf-server:5530

# If not, start it:
cd leaf
cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token
```

### "Cargo Not Found"

**Problem**: Rust toolchain not installed

**Solution**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Tests Timeout

**Problem**: Tests take too long

**Solutions**:
- Check server logs for errors
- Verify network connectivity
- Run individual test for debugging:
  ```bash
  cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored --nocapture
  ```

### "Module Not Found"

**Problem**: Wrong package name

**Solution**:
```bash
# Use -p flag with exact package name
cargo test -p leaf-client-rust  # Correct!
cargo test leaf-client-rust     # Wrong!
```

## Next Steps After Testing

### If Tests Pass ✅

1. **Check code quality**:
   ```bash
   ./scripts/test.sh clippy
   ./scripts/test.sh format
   ```

2. **Run benchmarks**:
   ```bash
   ./scripts/test.sh bench
   ```

3. **Generate coverage**:
   ```bash
   ./scripts/test.sh coverage
   ```

4. **Build examples**:
   ```bash
   ./scripts/test.sh examples
   ```

### If Tests Fail ❌

1. **Check server logs** for errors
2. **Run with debug output**:
   ```bash
   RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
   ```
3. **Run individual test** to isolate issue:
   ```bash
   cargo test -p leaf-client-rust --test integration_test test_failing_test -- --ignored --nocapture
   ```
4. **Check protocol compatibility** between client and server

## Expected Test Results

### Unit Tests (6 tests)
```
✅ test_did_validation              - DID format validation
✅ test_sql_value_serialization     - Type tags
✅ test_encode_decode_sql_value     - CBOR round-trip
✅ test_encode_decode_with_bytes    - Binary data
✅ test_library_builds (x2)         - Compilation
```

### Integration Tests (13 tests)
```
✅ test_client_connection           - Connection lifecycle
✅ test_upload_and_check_module     - Module operations
✅ test_create_stream               - Stream creation
✅ test_query_execution             - Query protocol
✅ test_send_events                 - Event sending
✅ test_send_state_events           - State events
✅ test_update_stream_module        - Module updates
✅ test_set_handle                  - Handle management
✅ test_clear_state                 - State clearing
✅ test_event_subscription          - Subscriptions
✅ test_error_handling              - Error scenarios
✅ test_concurrent_operations       - Concurrency
✅ test_did_validation              - DID validation
```

## Quick Reference Card

```bash
# Fastest: Unit tests only
cd leaf && cargo test -p leaf-client-rust --lib

# Complete: All tests (with server)
cd leaf/clients/leaf-client-rust && ./scripts/test.sh all

# Debug: With output
cd leaf && RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture

# Single test
cd leaf && cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored

# Watch mode
cd leaf/clients/leaf-client-rust && ./scripts/test.sh watch

# Benchmarks
cd leaf && cargo bench -p leaf-client-rust
```

## What's Being Tested

### Unit Tests Verify:
- ✅ DID validation logic
- ✅ CBOR encoding/decoding
- ✅ Type serialization
- ✅ Binary data handling
- ✅ Code compilation

### Integration Tests Verify:
- ✅ Client-server communication
- ✅ All 13 API methods
- ✅ Error handling
- ✅ Concurrent operations
- ✅ Event subscriptions
- ✅ State management

## Resources

- **Full Guide**: See `TESTING.md`
- **Quick Reference**: See `TEST_QUICKREF.md`
- **Test Structure**: See `TESTING_STRUCTURE.md`
- **Implementation**: See `REVIEW_SUMMARY.md`

---

**Ready to test?** Run this:

```bash
cd leaf/clients/leaf-client-rust && ./scripts/test.sh unit
```

Good luck! 🌿
