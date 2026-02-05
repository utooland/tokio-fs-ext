use std::{io, path::Path};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::File as JsFile;

use super::opfs::{CreateFileMode, get_fs_handle, opfs_err, virtualize};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let virt_path = virtualize(&path)?;
    let handle = get_fs_handle(&virt_path, CreateFileMode::NotCreate).await?;

    let file: JsFile = JsFuture::from(handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(opfs_err)?;

    Ok(Uint8Array::new(&array_buffer).to_vec())
}
