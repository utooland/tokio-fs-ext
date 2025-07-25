cfg_select! {
    all(target_family = "wasm", target_os = "unknown") => {
        mod wasm;
        pub(crate) use wasm::file;
        pub(crate) use wasm::opfs;
        pub(crate) use wasm::dir;
        pub use wasm::{
            canonicalize::canonicalize,
            copy::copy,
            create_dir::create_dir,
            create_dir_all::create_dir_all,
            file::File,
            metadata::metadata,
            open_options::OpenOptions,
            read::read,
            read_dir::{DirEntry, ReadDir, read_dir},
            read_link::read_link,
            read_to_string::read_to_string,
            remove_dir::remove_dir,
            remove_dir_all::remove_dir_all,
            remove_file::remove_file,
            rename::rename,
            symlink::symlink,
            try_exists::try_exists,
            write::write,
        };
    }
    not(all(target_family = "wasm", target_os = "unknown")) => {
        pub use tokio::fs::{
           canonicalize::canonicalize,
            copy::copy,
            create_dir::create_dir,
            create_dir_all::create_dir_all,
            file::File,
            metadata::metadata,
            open_options::OpenOptions,
            read::read,
            read_dir::{DirEntry, ReadDir, read_dir},
            read_link::read_link,
            read_to_string::read_to_string,
            remove_dir::remove_dir,
            remove_dir_all::remove_dir_all,
            remove_file::remove_file,
            rename::rename,
            symlink::symlink,
            try_exists::try_exists,
            write::write,
        };
    }
}
