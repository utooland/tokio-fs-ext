use std::{io, path::Path};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions};

use crate::fs::wasm::opfs::{fs_root, map_opfs_err};

pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(true);
    futures::future::try_join_all(
        path.as_ref()
            .to_string_lossy()
            .split('/')
            .map(|name| async {
                let root = fs_root().await?;

                JsFuture::from(root.get_directory_handle_with_options(name, &options))
                    .await
                    .map_err(map_opfs_err)?
                    .dyn_into::<FileSystemDirectoryHandle>()
                    .map_err(map_opfs_err)?;
                io::Result::Ok(())
            }),
    )
    .await?;
    Ok(())
}
