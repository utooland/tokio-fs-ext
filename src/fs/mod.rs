#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod wasm;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub(crate) use wasm::opfs;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use wasm::{
    DirBuilder, DirEntry, File, OpenOptions, ReadDir, canonicalize, copy, create_dir,
    create_dir_all, metadata, read, read_dir, read_link, read_to_string, remove_dir,
    remove_dir_all, remove_file, rename, symlink, try_exists, write,
};

#[cfg(any(target_family = "unix", target_family = "windows"))]
pub use tokio::fs::{
    DirBuilder, DirEntry, File, OpenOptions, ReadDir, canonicalize, copy, create_dir,
    create_dir_all, metadata, read, read_dir, read_link, read_to_string, remove_dir,
    remove_dir_all, remove_file, rename, try_exists, write,
};

#[cfg(target_family = "unix")]
pub use tokio::fs::symlink;
