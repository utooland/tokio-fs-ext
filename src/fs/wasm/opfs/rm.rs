use std::{io, path::Path};

use send_wrapper::SendWrapper;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemRemoveOptions;

use super::{OpenDirType, OpfsError, open_dir, root::opfs_root, virtualize};

pub(crate) async fn rm(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    let virt = virtualize::virtualize(path)?;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(path) => {
            if path.to_string_lossy().is_empty() {
                opfs_root().await?
            } else {
                open_dir(path, OpenDirType::NotCreate).await?
            }
        }
        None => opfs_root().await?,
    };

    let options = SendWrapper::new(FileSystemRemoveOptions::new());
    options.set_recursive(recursive);

    SendWrapper::new(JsFuture::from(
        dir_entry.remove_entry_with_options(&name, &options),
    ))
    .await
    .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}
