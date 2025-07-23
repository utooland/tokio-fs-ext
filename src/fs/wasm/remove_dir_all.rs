use std::{io, path::Path};

use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemRemoveOptions;

use crate::fs::wasm::opfs::{fs_root, map_opfs_err};

pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let name = path.as_ref().to_string_lossy();

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(true);

    let root = fs_root().await?;
    JsFuture::from(root.remove_entry_with_options(&name, &options))
        .await
        .map_err(map_opfs_err)?;

    Ok(())
}
