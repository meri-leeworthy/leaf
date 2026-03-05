# WASM Support Analysis for Rust Leaf Client

## Current Status: 🔴 Not WASM-Compatible

The Rust Leaf client currently **cannot target WASM** due to several blocking dependencies.

## Blocking Issues

### 1. 🔴 **Tokio Runtime** (Major Blocker)

**Problem**: Tokio doesn't work on WASM

```toml
tokio = { version = "1.43", features = ["full", "macros"] }
```

**Why it blocks**:
- Tokio uses thread-based async runtime
- WASM is single-threaded (mostly)
- Tokio's I/O primitives don't work in browser

**Impact**: Entire client needs async runtime rewrite

### 2. 🔴 **rust_socketio** (Major Blocker)

**Problem**: rust_socketio depends on native-tls and OpenSSL

```toml
rust_socketio = { version = "0.6.0", features = ["async"] }
```

**Dependencies tree**:
```
rust_socketio
├── native-tls
│   └── openssl (❌ native C library)
└── tokio (❌ not WASM-compatible)
```

**Why it blocks**:
- OpenSSL is a native C library
- Native TLS doesn't work in browsers
- WebSocket handling depends on tokio

**Impact**: Need different WebSocket/Socket.IO solution

### 3. 🟡 **Native TLS/OpenSSL** (Cannot Work)

**Problem**: Browser has its own TLS, doesn't use OpenSSL

```toml
native-tls v0.2.18
└── openssl v0.10.75
    └── openssl-sys v0.9.111 (❌ native C bindings)
```

**Why it blocks**:
- Browsers provide Fetch API and WebSockets
- Cannot use native crypto libraries
- All I/O must go through browser APIs

## Path to WASM Support

### Option 1: Full Rewrite for WASM (Recommended for Web) 🌐

**Approach**: Create WASM-specific version using browser APIs

**Required Changes**:

#### 1. Replace Async Runtime
```toml
# Remove tokio
-tokio = { version = "1.43", features = ["full", "macros"] }

# Add WASM-compatible runtime
+wasm-bindgen-futures = "0.4"
```

#### 2. Replace Socket.IO Client
```toml
# Remove rust_socketio
-rust_socketio = { version = "0.6.0", features = ["async"] }

# Option A: Use raw WebSockets via wasm-bindgen
+wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
+js-sys = "0.3"
+web-sys = { version = "0.3", features = ["WebSocket", "MessageEvent"] }

# Option B: Use socket.io-client (JavaScript) via FFI
+# Need to call JavaScript from Rust

# Option C: Write custom Socket.IO protocol implementation
+More work but pure Rust
```

#### 3. Update Client Code

**Current (Tokio-based)**:
```rust
use tokio::sync::Mutex;
use rust_socketio::asynchronous::{Client, ClientBuilder};

pub struct LeafClient {
    socket: Client,
    subscriptions: Arc<Mutex<HashMap<...>>>,
}
```

**WASM-compatible**:
```rust
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;
use std::sync::Mutex;  // Not Arc, since WASM is single-threaded

#[wasm_bindgen]
pub struct LeafClient {
    socket: WebSocket,
    subscriptions: Mutex<HashMap<...>>,
}
```

#### 4. Add WASM Build Support

**Create `Cargo.toml` additions**:
```toml
[lib]
crate-type = ["cdylib", "rlib"]  # Both WASM and native

[dependencies]
wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "WebSocket",
    "MessageEvent",
    "CloseEvent",
    "ErrorEvent",
    "BinaryType",
]}

# Make tokio and rust_socketio platform-specific
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1.43", features = ["full", "macros"] }
rust_socketio = { version = "0.6.0", features = ["async"] }
native-tls = "0.2"
```

**Create `.cargo/config.toml`**:
```toml
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"
```

#### 5. Build Setup

**Install tools**:
```bash
cargo install wasm-pack
cargo install wasm-bindgen-cli
```

**Build for WASM**:
```bash
wasm-pack build --target web
```

**Use in JavaScript**:
```javascript
import init, { LeafClient } from './leaf_client_rust.js';

async function run() {
  await init();

  const client = new LeafClient();
  await client.connect("http://localhost:5530");
}
```

### Option 2: Conditional Compilation (Hybrid Approach) 🔀

**Approach**: Single codebase with native and WASM variants

**Pros**:
- Share type definitions and codec
- Separate implementations per platform
- Can optimize for each platform

**Cons**:
- More complex build setup
- Need to maintain two implementations

**Example Structure**:
```rust
// src/lib.rs
#[cfg(target_arch = "wasm32")]
mod wasm_client;
#[cfg(not(target_arch = "wasm32"))]
mod native_client;

// src/wasm_client.rs
use web_sys::WebSocket;
pub struct LeafClient { /* WASM implementation */ }

// src/native_client.rs
use rust_socketio::asynchronous::Client;
pub struct LeafClient { /* Native implementation */ }
```

### Option 3: Separate WASM Package (Cleanest) 📦

**Approach**: Create new `leaf-client-rust-wasm` package

**Structure**:
```
leaf/
├── clients/
│   ├── leaf-client-rust/          # Native (current)
│   │   └── Cargo.toml             # Tokio + rust_socketio
│   └── leaf-client-rust-wasm/     # WASM (new)
│       ├── Cargo.toml             # wasm-bindgen + web-sys
│       └── src/
│           ├── lib.rs             # WASM-specific client
│           └── socket/            # Custom Socket.IO impl
```

**Shared via workspace**:
```toml
[workspace]
members = ["shared-types", "native-client", "wasm-client"]

[dependencies]
leaf-client-types = { path = "shared-types" }
```

## Estimated Effort

### Option 1: Full Rewrite
- **Time**: 20-30 hours
- **Effort**: High
- **Risk**: Medium
- **Maintenance**: Medium

### Option 2: Conditional Compilation
- **Time**: 30-40 hours
- **Effort**: Very High
- **Risk**: High
- **Maintenance**: High

### Option 3: Separate WASM Package
- **Time**: 15-20 hours
- **Effort**: Medium
- **Risk**: Low
- **Maintenance**: Low

## Technical Challenges

### 1. Socket.IO Protocol Complexity 🔴

The Socket.IO protocol is complex:
- Transport upgrade (polling → WebSocket)
- Packet encoding (MessagePack, binary acks)
- Reconnection logic
- Room management

**Options**:
- Use `socket.io-client` JavaScript library via FFI (easiest)
- Implement protocol in Rust (harder)
- Use raw WebSocket + custom protocol (moderate)

### 2. Async Runtime Differences 🟡

**Tokio (native)**:
```rust
let socket = ClientBuilder::new(url).connect().await?;
```

**WASM-bindgen-futures (WASM)**:
```rust
let socket = WebSocket::new(url)?;
// Promise-based, not Future-based
```

### 3. Browser API Limitations 🟡

- No direct TCP/UDP access
- No native file system
- Limited threading (Web Workers)
- Crypto via Web Crypto API only

## Recommended Approach

### For Web/WASM Target 🌐

**Use Option 3: Separate WASM package**

1. **Create `leaf-client-rust-wasm`** package
2. **Share types/codec** from current package
3. **Use browser WebSocket API** via wasm-bindgen
4. **Leverage existing JS Socket.IO client** via FFI
5. **Minimal reimplementation** of client logic

### Architecture

```
leaf-client-types (shared)
├── src/types.rs      # Current types
├── src/codec.rs      # CBOR codec
└── src/error.rs      # Error types

leaf-client-rust (native) - Current package
└── src/client.rs     # Tokio + rust_socketio

leaf-client-rust-wasm (new)
├── src/
│   ├── client.rs     # WASM WebSocket client
│   └── ffi.rs        # JavaScript interop
└── pkg/              # Generated wasm-bindgen output
```

## Quick Start: WASM Prototype

### Step 1: Create WASM Package

```bash
cd leaf/clients
mkdir leaf-client-rust-wasm
cd leaf-client-rust-wasm
cargo init --lib
```

### Step 2: Add Dependencies

```toml
[package]
name = "leaf-client-rust-wasm"
version = "0.1.0"
edition = "2021"
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "WebSocket",
    "MessageEvent",
    "BinaryType",
    "ErrorEvent",
] }
serde = { version = "1.0", features = ["derive"] }
serde_bytes = "0.11"
ciborium = "0.2"
thiserror = "2.0"

# Share types with native package
leaf-client-rust = { path = "../leaf-client-rust" }
```

### Step 3: Simple WebSocket Client

```rust
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;
use leaf_client_rust::{codec, types::*};

#[wasm_bindgen]
pub struct WasmLeafClient {
    socket: Option<WebSocket>,
    url: String,
}

#[wasm_bindgen]
impl WasmLeafClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String) -> Self {
        Self { socket: None, url }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        let socket = WebSocket::new(&self.url)?;
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        self.socket = Some(socket);
        Ok(())
    }

    // Add more methods...
}
```

### Step 4: Build

```bash
wasm-pack build --target web
```

### Step 5: Use in JavaScript

```html
<script type="module">
  import init, { WasmLeafClient } from './pkg/leaf_client_rust_wasm.js';

  async function main() {
    await init();
    const client = new WasmLeafClient("ws://localhost:5530");
    await client.connect();
  }

  main();
</script>
```

## Conclusion

### Distance to WASM: 🔴 FAR (15-40 hours of work)

**Current State**:
- ✅ Types and codec are WASM-compatible
- ❌ Client implementation needs complete rewrite
- ❌ Socket handling needs browser-specific implementation
- ❌ Async runtime needs replacement

**Minimum Viable WASM Support**: 15-20 hours
- Create separate WASM package
- Implement WebSocket client
- Use existing JS Socket.IO via FFI
- Port connection logic

**Full WASM Parity**: 30-40 hours
- Custom Socket.IO implementation in Rust
- Full feature parity with native client
- Optimized WASM bundle size
- Comprehensive testing

**Recommendation**: Start with separate WASM package (Option 3) for clean separation and faster development.

---

**TL;DR**: The client is **not close to WASM support**. It would require **15-40 hours of work** depending on approach. The types and codec are reusable, but the entire client layer needs to be rewritten for browser APIs.
