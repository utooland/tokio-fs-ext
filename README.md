# tokio-fs-ext

[![Crates.io](https://img.shields.io/crates/v/tokio-fs-ext.svg)](https://crates.io/crates/tokio-fs-ext)
[![Docs.rs](https://docs.rs/tokio-fs-ext/badge.svg)](https://docs.rs/tokio-fs-ext)

Tokio-fs-ext is a Rust library that provides a `tokio::fs` compatible API for both native and WebAssembly environments on web browsers.

## Overview

The standard `tokio::fs` module in the Tokio runtime is a powerful tool for asynchronous file system operations. However, it relies on blocking `syscalls` an I/O operations that are executed on a dedicated thread pool. This design is not suitable for WebAssembly environments where threading and direct file system access are restricted.

This library aims to bridge that gap by offering an API that is compatible with `tokio::fs` but works seamlessly in WebAssembly. It provides a consistent interface for file system operations, regardless of the target platform.

## Features

- A `tokio::fs`-like API.
- Re-export `tokio::fs` on native platforms, and use implementations by [`OPFS`](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system) on `wasm32-unknown-unknown` platform.
- Implemented [futures::io::traits](https://docs.rs/futures/0.3.31/futures/io/index.html#traits).
- Asynchronous file operations for non-blocking applications.

## Usage

```rust
use tokio_fs_ext as fs;
use std::io;
use futures::io::{AsyncReadExt, AsyncWrite};

async fn foo() -> io::Result<()> {
    fs::write("hello.txt", "Hello").await?;

    {
        let mut file = fs::File::open("hello.txt").await?;
    
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).await?;
    }

    fs::remove_file("hello.txt").await?;

    io::Result::Ok(())
}
```

## Clarification

- The implements for WebAssembly can only be used in [`DedicatedWorkerGlobalScope`](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope).

## Contributing

## Testing

```bash
# test native
cargo test

# test native
brew install --cask chromedriver
cargo test --target wasm32-unknown-unknown

# test wasm in interactive mode
brew install wasm-pack
wasm-pack test --chrome
```
