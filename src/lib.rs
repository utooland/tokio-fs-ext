#![feature(io_error_uncategorized)]
#![feature(const_pathbuf_osstring_new)]

mod fs;

pub use fs::*;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod console;
