cfg_select! {
    all(target_family = "wasm", target_os = "unknown") => {
        mod wasm;
        pub(crate) use wasm::opfs;
        pub use wasm::{
            canonicalize,
            copy,
            create_dir,
            create_dir_all,
            File,
            metadata,
            OpenOptions,
            read, DirEntry, ReadDir,
            read_dir,
            read_link,
            read_to_string,
            remove_dir,
            remove_dir_all,
            remove_file,
            rename,
            symlink,
            try_exists,
            write,

        };
    }
    not(all(target_family = "wasm", target_os = "unknown")) => {
        pub use tokio::fs::{
            canonicalize,
            copy,
            create_dir,
            create_dir_all,
            File,
            metadata,
            OpenOptions,
            read, DirEntry, ReadDir,
            read_dir,
            read_link,
            read_to_string,
            remove_dir,
            remove_dir_all,
            remove_file,
            rename,
            try_exists,
            write,
        };
    }
    target_family = "unix" => {
        pub use tokio::fs::symlink;
    }
}
