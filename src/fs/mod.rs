use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_family = "wasm", target_os = "unknown"))] {
        mod wasm;
        pub(crate) use wasm::opfs;
        pub use wasm::{
            DirBuilder, DirEntry, OpenOptions, ReadDir, canonicalize, copy, create_dir, create_dir_all,
            metadata, read, read_dir, read_link, read_to_string, remove_dir, remove_dir_all, remove_file,
            rename, symlink_metadata, try_exists, write,
        };
        pub use wasm::{File, Metadata, symlink};
    } else if #[cfg(any(target_family = "unix", target_family = "windows"))] {
        pub use tokio::fs::{
            DirBuilder, DirEntry, OpenOptions, ReadDir, canonicalize, copy, create_dir, create_dir_all,
            metadata, read, read_dir, read_link, read_to_string, remove_dir, remove_dir_all, remove_file,
            rename, symlink_metadata, try_exists, write,
        };
        pub use std::fs::Metadata;
        /// Should use this File type along with [`tokio_util::compat`]: https://docs.rs/tokio-util/latest/tokio_util/compat/index.html
        pub use tokio::fs::File;

        pub use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

        // Specific symlink exports based on OS
        cfg_if! {
            if #[cfg(target_family = "unix")] {
                pub use tokio::fs::symlink;
            } else if #[cfg(target_family = "windows")] {
                pub use tokio::fs::{symlink_dir, symlink_file};
            }
        }
    }
}
