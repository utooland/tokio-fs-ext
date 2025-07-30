#![feature(io_error_uncategorized)]

pub mod fs;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod console;
