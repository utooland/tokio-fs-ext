use std::{io, path::Path};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{CreateFileMode, get_fs_handle, opfs_err, virtualize};

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64, io::Error> {
    let from_virt = virtualize(&from)?;
    let to_virt = virtualize(&to)?;

    if from_virt == to_virt {
        return Ok(0);
    }

    let from_handle = get_fs_handle(&from_virt, CreateFileMode::NotCreate).await?;
    let to_handle = get_fs_handle(&to_virt, CreateFileMode::Create).await?;

    let file: web_sys::File = JsFuture::from(from_handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let size = file.size() as u64;

    if size == 0 {
        return Ok(0);
    }

    let stream: FileSystemWritableFileStream = JsFuture::from(to_handle.create_writable())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let promise = stream
        .write_with_blob(&file)
        .map_err(opfs_err)?;

    JsFuture::from(promise).await.map_err(opfs_err)?;
    JsFuture::from(stream.close()).await.map_err(opfs_err)?;

    Ok(size)
}
