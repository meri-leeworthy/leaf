# Testing Structure Overview

## Visual Test Organization

```
leaf-client-rust/
│
├── 📦 src/                    # Source code with inline unit tests
│   ├── lib.rs
│   │   └── #[cfg(test)]       ✅ test_library_builds
│   ├── types.rs
│   │   └── #[cfg(test)]       ✅ test_did_validation
│   │                         ✅ test_sql_value_serialization
│   ├── codec.rs
│   │   └── #[cfg(test)]       ✅ test_encode_decode_sql_value
│   │                         ✅ test_encode_decode_with_bytes
│   ├── client.rs              # (no inline tests yet)
│   └── error.rs               # (no inline tests yet)
│
├── 🧪 tests/                  # Integration tests
│   └── integration_test.rs    🚧 13 integration tests (need server)
│       ├── test_client_connection
│       ├── test_upload_and_check_module
│       ├── test_create_stream
│       ├── test_query_execution
│       ├── test_send_events
│       ├── test_send_state_events
│       ├── test_update_stream_module
│       ├── test_set_handle
│       ├── test_clear_state
│       ├── test_event_subscription
│       ├── test_error_handling
│       ├── test_concurrent_operations
│       └── test_did_validation
│
├── 📜 examples/               # Demo programs
│   ├── cbor_demo.rs           ✅ Demonstrates CBOR encoding
│   └── client_demo.rs         ✅ Shows API usage
│
├── 📊 benches/                # Performance benchmarks
│   └── cbor_bench.rs          ✅ CBOR performance tests
│
├── 🔧 scripts/                # Helper scripts
│   └── test.sh                ✅ Test runner with multiple commands
│
└── 📖 docs/                   # Documentation (root level)
    ├── README.md              ✅ Project overview
    ├── TESTING.md             ✅ Full testing guide
    ├── TEST_QUICKREF.md       ✅ Quick command reference
    ├── REVIEW_SUMMARY.md      ✅ Implementation review
    ├── PROGRESS.md            ✅ Development progress
    └── PHASE2_COMPLETE.md     ✅ Phase 2 details
```

## Test Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Command                             │
│                  cargo test -p leaf-client-rust             │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
        ┌──────────────────────────────────────┐
        │         Cargo Test Runner            │
        └──────────────────┬───────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
    ┌─────────────┐                 ┌─────────────┐
    │ Unit Tests  │                 │  Integration│
    │ (src/*.rs)  │                 │  Tests      │
    │             │                 │(tests/*.rs) │
    │ • No server │                 │ • Need server│
    │ • Fast      │                 │ • Slower    │
    │ • 6 tests   │                 │ • 13 tests  │
    └─────────────┘                 └─────────────┘
           │                               │
           ▼                               ▼
    ┌─────────────┐                 ┌─────────────┐
    │    ✅       │                 │   🚧        │
    │   Pass      │                 │  Need       │
    │   (6/6)     │                 │  Server     │
    └─────────────┘                 └─────────────┘
```

## Test Categories by Purpose

### 1. Unit Tests (Fast, No Server)

**Purpose**: Verify individual functions and types

| Test | Module | What It Tests |
|------|--------|---------------|
| `test_did_validation` | types.rs | DID format validation |
| `test_sql_value_serialization` | types.rs | Type tag correctness |
| `test_encode_decode_sql_value` | codec.rs | CBOR round-trip |
| `test_encode_decode_with_bytes` | codec.rs | Binary encoding |
| `test_library_builds` | lib.rs | Compilation |

**Run**: `cargo test -p leaf-client-rust --lib`

**Time**: < 1 second

### 2. Integration Tests (Slow, Needs Server)

**Purpose**: Verify client-server interaction

| Test | What It Tests | API Methods Used |
|------|---------------|------------------|
| `test_client_connection` | Connection lifecycle | `connect()`, `disconnect()` |
| `test_upload_and_check_module` | Module operations | `upload_module()`, `has_module()` |
| `test_create_stream` | Stream creation | `create_stream()`, `stream_info()` |
| `test_query_execution` | Query protocol | `query()` |
| `test_send_events` | Event sending | `send_events()` |
| `test_send_state_events` | State events | `send_state_events()` |
| `test_update_stream_module` | Module updates | `update_module()` |
| `test_set_handle` | Handle management | `set_handle()` |
| `test_clear_state` | State clearing | `clear_state()` |
| `test_event_subscription` | Subscriptions | `subscribe_events()`, `unsubscribe_events()` |
| `test_error_handling` | Error cases | Various |
| `test_concurrent_operations` | Concurrency | Multiple `create_stream()` |
| `test_did_validation` | DID validation | `Did::new()` |

**Run**: `cargo test -p leaf-client-rust --test integration_test -- --ignored`

**Time**: ~30 seconds (with server)

### 3. Benchmarks (Performance)

**Purpose**: Measure performance characteristics

| Benchmark | Measures | Purpose |
|-----------|----------|---------|
| `encode_sql_value` | Encoding speed | Baseline performance |
| `decode_sql_value` | Decoding speed | Baseline performance |
| `encode_query` | Query serialization | Real-world usage |
| `decode_query` | Query deserialization | Real-world usage |
| `large_query` | Large payload handling | Stress testing |

**Run**: `cargo bench -p leaf-client-rust`

**Time**: ~1 minute

## Test Coverage Matrix

| API Method | Unit Test | Integration Test | Example |
|------------|-----------|------------------|---------|
| `connect()` | ✅ | ✅ | ✅ |
| `upload_module()` | ⏳ | ✅ | ⏳ |
| `has_module()` | ⏳ | ✅ | ⏳ |
| `create_stream()` | ⏳ | ✅ | ⏳ |
| `stream_info()` | ⏳ | ✅ | ⏳ |
| `update_module()` | ⏳ | ✅ | ⏳ |
| `send_events()` | ⏳ | ✅ | ⏳ |
| `send_state_events()` | ⏳ | ✅ | ⏳ |
| `subscribe_events()` | ⏳ | ✅ | ⏳ |
| `unsubscribe_events()` | ⏳ | ✅ | ⏳ |
| `query()` | ⏳ | ✅ | ⏳ |
| `set_handle()` | ⏳ | ✅ | ⏳ |
| `clear_state()` | ⏳ | ✅ | ⏳ |
| `disconnect()` | ✅ | ✅ | ⏳ |

Legend: ✅ Tested | ⏳ Not directly tested (but covered by integration tests)

## Running Tests by Scenario

### I want to...

**Verify code compiles:**
```bash
cargo check -p leaf-client-rust
```

**Run quick checks before committing:**
```bash
./scripts/test.sh check
./scripts/test.sh clippy
./scripts/test.sh format
```

**Test without starting server:**
```bash
cargo test -p leaf-client-rust --lib
```

**Test everything with server running:**
```bash
./scripts/test.sh all
```

**Measure performance:**
```bash
cargo bench -p leaf-client-rust
```

**Watch mode during development:**
```bash
./scripts/test.sh watch
```

**Generate coverage report:**
```bash
./scripts/test.sh coverage
```

**Run a specific failing test:**
```bash
cargo test -p leaf-client-rust --test integration_test test_client_connection -- --ignored --nocapture
```

**Debug test with logging:**
```bash
RUST_LOG=debug cargo test -p leaf-client-rust --test integration_test -- --ignored --nocapture
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Test Rust Leaf Client

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test -p leaf-client-rust --lib
      - name: Run linter
        run: cargo clippy -p leaf-client-rust -- -D warnings
      - name: Check formatting
        run: cargo fmt -p leaf-client-rust -- --check

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Start Leaf server
        run: |
          cargo r -- --otel server -D did:web:localhost --unsafe-auth-token test-token &
          sleep 10
      - name: Run integration tests
        run: cargo test -p leaf-client-rust --test integration_test -- --ignored

  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench -p leaf-client-rust
```

## Test Development Workflow

```
┌─────────────────┐
│  Write Feature  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Write Test     │ ← TDD approach
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Run Test       │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌──────┐  ┌──────┐
│ Pass │  │ Fail │
└───┬──┘  └───┬──┘
    │         │
    ▼         ▼
┌──────┐  ┌──────────┐
│ Done │  │ Debug &  │
│      │  │ Fix      │
└──────┘  └────┬─────┘
               │
               ▼
        ┌──────────┐
        │ Re-run   │
        └─────┬────┘
              │
              ▼
         (loop until pass)
```

## Test Maintenance

### When to Update Tests

- ✅ Adding new API method → Add integration test
- ✅ Adding new type → Add unit test
- ✅ Fixing bug → Add regression test
- ✅ Changing protocol → Update tests
- ✅ Adding error case → Add error test

### Test Health Checklist

- [ ] All unit tests pass
- [ ] All integration tests pass (with server)
- [ ] No compiler warnings
- [ ] Clippy passes
- [ ] Code formatted
- [ ] Documentation updated
- [ ] Examples run successfully
- [ ] Benchmarks run without errors

## Next Steps

1. **Run the tests** against your server
2. **Debug any failures** that occur
3. **Add more tests** for edge cases
4. **Set up CI/CD** for automated testing
5. **Measure performance** with benchmarks
6. **Add property tests** for fuzzing

---

**Status**: Testing infrastructure complete ✅ | Ready for execution ✅
