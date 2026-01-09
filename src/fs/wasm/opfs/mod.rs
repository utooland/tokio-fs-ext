mod dir_handle_cache;
mod error;
mod open_dir;
mod open_file;
mod options;
mod remove;
mod root;
mod virtualize;
#[cfg(feature = "opfs_watch")]
pub mod watch;

pub(super) use error::OpfsError;
pub(super) use open_dir::open_dir;
pub(super) use open_file::{open_file, get_fs_handle};
pub(super) use options::{CreateFileMode, OpenDirType, SyncAccessMode};
pub(super) use remove::remove;
pub(super) use virtualize::virtualize;
