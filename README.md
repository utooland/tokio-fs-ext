# tokio-fs-ext

[![Crates.io](https://img.shields.io/crates/v/tokio-fs-ext.svg)](https://crates.io/crates/tokio-fs-ext)
[![Docs.rs](https://docs.rs/tokio-fs-ext/badge.svg)](https://docs.rs/tokio-fs-ext)

Tokio-fs-ext is a Rust library that provides a `tokio::fs` compatible API for both native and WebAssembly environments on web browsers.

## Overview

The standard `tokio::fs` module in the Tokio runtime is a powerful tool for asynchronous file system operations. However, it relies on `syscalls` and I/O operations that are executed on a dedicated thread pool. This design is not suitable for WebAssembly environments where threading and direct file system access are restricted.

This library aims to bridge that gap by offering an API that is compatible with `tokio::fs` but works seamlessly in WebAssembly. It provides a consistent interface for file system operations, regardless of the target platform.

## Features

- A `tokio::fs`-like API.
- Re-export `tokio::fs` on native platforms, and use implementations by [`OPFS`](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system) on `wasm32-unknown-unknown` platform.
- Implemented [futures::io::traits](https://docs.rs/futures/0.3.31/futures/io/index.html#traits).
- Asynchronous file operations for non-blocking applications.

## WASM Concurrency Model

On WASM platforms (via OPFS), this library implements a sophisticated **State-Aware Locking & Caching** model to overcome the limitation where only one `SyncAccessHandle` can be active per file:

### 1. Unified Locking System
We use a custom asynchronous lock manager to coordinate access:
- **Shared Access (`Some(Readonly)`)**: Allows multiple concurrent readers.
- **Promotable Access (`Some(Readwrite)`)**: Allows multiple concurrent readers and writers by sharing a single physical `SyncAccessHandle`.
- **Exclusive Access (`None`)**: Blocks all other access for atomic filesystem operations like `remove_file` or `rename`.

### 2. Intelligent Handle Caching
To maintain a `tokio::fs` compatible API where multiple `File` objects can coexist:
- **Reference Counting**: The underlying `SyncAccessHandle` is cached and shared. It is only physically closed when the last `File` object or active I/O operation on that path is dropped.
- **On-Demand Permission Upgrades**: If a file is currently cached in `Readonly` mode and a new task requests `Readwrite` access, the system will wait for current readers to yield, close the RO handle, and transparently reopen it in `Readwrite` mode for all subsequent users.

### 3. Hybrid I/O Strategy (The "Fast Path")
We optimize for performance by choosing the best Web API for the task:
- **Native Async Fast-Path**: Atomic operations like `fs::read` and `fs::write` prefer truly non-blocking Web APIs (`getFile().array_buffer()` and `createWritable()`).
- **Cache-Aware Persistence**: If a `SyncAccessHandle` is already active (e.g., a `File` object is open), atomic operations will automatically detect and "join" the existing handle to avoid the high overhead of creating new handles.
- **Minimal Blocking**: By using native async APIs where possible, we prevent the "Head-of-Line Blocking" common in OPFS implementations that rely solely on synchronous handles.

## Offload Design (Thread-Safety & Responsiveness)

A major challenge in WASM is that `wasm-bindgen` generated types (like `FileSystemSyncAccessHandle`) are **not thread-safe** (`!Send` and `!Sync`). This makes them incompatible with multi-threaded Rust/Tokio environments where tasks can migrate between threads.

To solve this, we implement a **Thread-Isolated Offload Server** architecture:

- **Dedicated I/O Thread**: The "Offload Server" runs in a single, dedicated worker thread where all native JavaScript handles reside. This ensures that `!Send` handles are never moved across thread boundaries.
- **Thread-Local Affinity**: Since `SyncAccessHandle` is bound to the thread that created it, the server acts as the sole custodian of these handles.
- **Any-Thread Client**: "Clients" can be invoked from any thread/worker. They communicate with the Offload Server via message passing (or shared memory buffers), allowing the rest of your application to remain multi-threaded and agnostic of OPFS's threading restrictions.
- **Non-Blocking Scheduler**: Beyond thread-safety, the server split large I/O requests into chunks and yields control back to the browser's event loop between chunks. This prevents a large file read from freezing the I/O thread and maintains overall system responsiveness.

## File System Watching

This library provides experimental support for file system monitoring via the [`FileSystemObserver`](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemObserver) API:

- **Recursive Watch**: Monitor entire directory trees for changes.
- **Cross-context Events**: Changes made in one Worker/Tab are reflected in others.
- **Native Efficiency**: Uses the browser's native file observer rather than polling, ensuring minimal CPU overhead.

> Note: `FileSystemObserver` is currently an experimental feature in modern browsers (e.g., Chrome with Experimental Web Platform features enabled).

## Usage

```rust
use tokio_fs_ext as fs;
use std::io;
use futures::io::AsyncReadExt;

async fn foo() -> io::Result<()> {
    fs::write("hello.txt", "Hello").await?;

    {
        let mut file = fs::File::open("hello.txt").await?;
    
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).await?;
    }

    fs::remove_file("hello.txt").await?;

    Ok(())
}
```

## Clarification

- The implements for WebAssembly can only be used in [`DedicatedWorkerGlobalScope`](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope).

## Contributing

## Testing

```bash
# test native
cargo test

# test wasm
brew install --cask chromedriver
CHROMEDRIVER=$(which chromedriver) cargo test --target wasm32-unknown-unknown

# test wasm in interactive mode
brew install wasm-pack
wasm-pack test --chrome
```
