#![feature(cfg_select)]

cfg_select! {
    all(target_family = "wasm", target_os = "unknown") => {
        mod wasm;
        pub use wasm::{fs, io};
    }
    not(all(target_family = "wasm", target_os = "unknown")) => {
        pub use tokio::{fs, io}
    }
}
