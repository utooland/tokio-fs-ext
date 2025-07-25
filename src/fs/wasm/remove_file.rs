use std::{io, path::Path};

use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemRemoveOptions;

use crate::fs::opfs::{OpfsError, fs_root};

pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    let name = path.as_ref().to_string_lossy();

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(false);

    let root = fs_root().await?;
    JsFuture::from(root.remove_entry_with_options(&name, &options))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}
