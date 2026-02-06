use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(all(target_family = "wasm", target_os = "unknown"))] {

        mod wasm;

        pub use wasm::{ File, OpenOptions, current_dir, set_current_dir };

        pub use wasm::{
            DirBuilder, DirEntry, ReadDir, canonicalize, copy, create_dir, create_dir_all,
            metadata, read, read_dir, read_link, read_to_string, remove_dir, remove_dir_all, remove_file,
            rename, symlink_metadata, try_exists, write,
        };

        pub use wasm::{Metadata, symlink};

        pub use wasm::ReadDirStream;

        pub use wasm::FileType;

        #[cfg(feature = "opfs_offload")]
        pub use wasm::offload;

        #[cfg(feature = "opfs_watch")]
        pub use wasm::{WatchStream, watch_dir, watch_file};

    } else if #[cfg(any(target_family = "unix", target_family = "windows"))] {

        mod native;

        pub use native::{ File, OpenOptions, current_dir, set_current_dir };

        pub use tokio::fs::{
            DirBuilder, DirEntry, ReadDir, canonicalize, copy, create_dir, create_dir_all,
            metadata, read, read_dir, read_link, read_to_string, remove_dir, remove_dir_all, remove_file,
            rename, symlink_metadata, try_exists, write,
        };

        pub use tokio_stream::wrappers::ReadDirStream;

        pub use std::fs::Metadata;

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
