use std::{io, path::Path};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{CreateFileMode, SyncAccessMode, open_file, opfs_err, virtualize};

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    let from_virt = virtualize(&from)?;
    let to_virt = virtualize(&to)?;

    if from_virt == to_virt {
        return Ok(0);
    }

    let from_file = open_file(&from_virt, CreateFileMode::NotCreate, SyncAccessMode::Readonly, false).await?;
    let to_file = open_file(&to_virt, CreateFileMode::Create, SyncAccessMode::ReadwriteUnsafe, true).await?;

    let file: web_sys::File = JsFuture::from(from_file.handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let size = file.size() as u64;

    if size == 0 {
        return Ok(0);
    }

    let stream: FileSystemWritableFileStream = JsFuture::from(to_file.handle.create_writable())
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
