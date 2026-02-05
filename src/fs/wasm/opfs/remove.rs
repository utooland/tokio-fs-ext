use std::{io, path::Path};

use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemRemoveOptions;

use super::{
    OpenDirType, dir_handle_cache::remove_cached_dir_handle, lock_path, open_dir, opfs_err,
    root::root, virtualize,
};

#[cfg_attr(feature = "opfs_tracing", tracing::instrument(level = "trace", fields(path = %path.as_ref().to_string_lossy())))]
pub(crate) async fn remove(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    let virt = virtualize::virtualize(&path)?;
    let _lock = lock_path(&virt).await;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(parent) => open_dir(parent, OpenDirType::NotCreate).await?,
        None => root().await?,
    };

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(recursive);

    JsFuture::from(dir_entry.remove_entry_with_options(&name, &options))
        .await
        .map_err(opfs_err)?;

    remove_cached_dir_handle(&virt, recursive);

    Ok(())
}
