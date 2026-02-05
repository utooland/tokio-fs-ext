use std::{io, path::Path};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{CreateFileMode, get_fs_handle, opfs_err, virtualize};

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let virt_path = virtualize(&path)?;
    let handle = get_fs_handle(&virt_path, CreateFileMode::Create).await?;

    let stream: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    // Copy data to a non-shared Uint8Array (WASM linear memory is shared)
    let content = content.as_ref();
    let uint8_array = Uint8Array::new_with_length(content.len() as u32);
    uint8_array.copy_from(content);

    let promise = stream
        .write_with_js_u8_array(&uint8_array)
        .map_err(opfs_err)?;

    JsFuture::from(promise).await.map_err(opfs_err)?;
    JsFuture::from(stream.close()).await.map_err(opfs_err)?;

    Ok(())
}
