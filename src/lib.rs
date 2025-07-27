#![feature(cfg_select)]

pub mod fs;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod console;
