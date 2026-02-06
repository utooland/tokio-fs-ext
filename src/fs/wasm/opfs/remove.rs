use std::{io, path::Path};

use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemRemoveOptions;

use super::{
    OpenDirType, dir_handle_cache::remove_cached_dir_handle, get_fs_handle, open_dir, opfs_err,
    options::CreateFileMode, root::root, virtualize,
};

#[cfg_attr(feature = "opfs_tracing", tracing::instrument(level = "trace", fields(path = %path.as_ref().to_string_lossy())))]
pub(crate) async fn remove(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    let path = path.as_ref();
    let virt = virtualize::virtualize(path)?;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy().to_string()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(parent) => open_dir(parent, OpenDirType::NotCreate).await?,
        None => root().await?,
    };

    // Only lock if it's potentially a file to avoid "Operation would block" errors with SyncAccessHandle.
    //
    // Potential risk:
    // 1. Recursive removal: Removing a directory recursively does not acquire locks for its children.
    //    If a file inside the directory is currently open with a SyncAccessHandle, the removal might
    //    fail or leave the handle in an inconsistent state.
    // 2. Race condition: The entry could change between `get_file_handle` and `remove_entry`.
    let _lock = match get_fs_handle(path, CreateFileMode::NotCreate).await {
        Ok((_, lock)) => Some(lock),
        Err(_) => None,
    };

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(recursive);

    JsFuture::from(dir_entry.remove_entry_with_options(&name, &options))
        .await
        .map_err(opfs_err)?;

    remove_cached_dir_handle(&virt, recursive);

    Ok(())
}
