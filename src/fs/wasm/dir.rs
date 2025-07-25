use std::{
    io,
    path::{MAIN_SEPARATOR_STR, Path},
};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions};

use crate::fs::opfs::{OpfsError, fs_root};

pub(crate) async fn open_dir(
    path: impl AsRef<Path>,
    create: bool,
    recursive: bool,
) -> io::Result<FileSystemDirectoryHandle> {
    let name = path.as_ref().to_string_lossy();

    if !recursive && name.len() > 1 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }

    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(create);

    let root = fs_root().await?;

    let mut split = name.split(MAIN_SEPARATOR_STR);
    let mut dir_handle = JsFuture::from(
        root.get_directory_handle_with_options(
            split
                .next()
                .as_ref()
                .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
            &options,
        ),
    )
    .await
    .map_err(|err| io::Error::from(OpfsError::from(err)))?
    .dyn_into::<FileSystemDirectoryHandle>()
    .map_err(|err| io::Error::from(OpfsError::from(err)))?;

    for name in split.skip(1) {
        dir_handle = JsFuture::from(root.get_directory_handle_with_options(name, &options))
            .await
            .map_err(|err| io::Error::from(OpfsError::from(err)))?
            .dyn_into::<FileSystemDirectoryHandle>()
            .map_err(|err| io::Error::from(OpfsError::from(err)))?;
    }

    Ok(dir_handle)
}

