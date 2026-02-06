mod canonicalize;
mod copy;
mod create_dir;
mod create_dir_all;
mod current_dir;
mod dir_builder;
mod file;
mod metadata;
mod open_options;
pub(crate) mod opfs;
mod read;
mod read_dir;
mod read_dir_stream;
mod read_link;
mod read_to_string;
mod remove_dir;
mod remove_dir_all;
mod remove_file;
mod rename;
mod symlink;
mod symlink_metadata;
mod try_exists;
mod write;

pub use canonicalize::canonicalize;
pub use copy::copy;
pub use create_dir::create_dir;
pub use create_dir_all::create_dir_all;
pub use current_dir::{current_dir, set_current_dir};
pub use dir_builder::DirBuilder;
pub use file::File;
pub use metadata::{FileType, Metadata, metadata};
pub use open_options::OpenOptions;
pub use read::read;
pub use read_dir::{DirEntry, ReadDir, read_dir};
pub use read_dir_stream::ReadDirStream;
pub use read_link::read_link;
pub use read_to_string::read_to_string;
pub use remove_dir::remove_dir;
pub use remove_dir_all::remove_dir_all;
pub use remove_file::remove_file;
pub use rename::rename;
pub use symlink::symlink;
pub use symlink_metadata::symlink_metadata;
pub use try_exists::try_exists;
pub use write::write;

#[cfg(feature = "opfs_offload")]
pub mod offload;

#[cfg(feature = "opfs_watch")]
pub use opfs::watch::{WatchStream, watch_dir, watch_file};
