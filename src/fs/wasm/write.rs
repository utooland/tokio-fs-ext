use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{get_fs_handle, CreateFileMode, opfs_err, lock_path, virtualize};

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let virt_path = virtualize(&path)?;
    let _lock = lock_path(&virt_path).await;

    let handle = get_fs_handle(path, CreateFileMode::Create).await?;

    let stream: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let content = content.as_ref();
    let uint8_array = Uint8Array::from(content);

    let write_method = Reflect::get(&stream, &"write".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>();

    let promise = write_method
        .call1(&stream, &uint8_array)
        .map_err(opfs_err)?
        .unchecked_into::<Promise>();

    JsFuture::from(promise).await.map_err(opfs_err)?;

    JsFuture::from(stream.close()).await.map_err(opfs_err)?;

    Ok(())
}
