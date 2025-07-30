#![feature(io_error_uncategorized)]

mod fs;

pub use fs::*;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod console;
