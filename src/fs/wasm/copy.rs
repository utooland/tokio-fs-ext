use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, FileSystemWritableFileStream};

use super::opfs::{get_fs_handle, CreateFileMode, opfs_err, lock_path, virtualize};

/// Files larger than this threshold will be copied in chunks (10 MB)
const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024;
/// Chunk size for copying large files (2 MB)
const CHUNK_SIZE: u64 = 2 * 1024 * 1024;

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64, io::Error> {
    let from_virt = virtualize(&from)?;
    let to_virt = virtualize(&to)?;
    let _lock_from = lock_path(&from_virt).await;
    let _lock_to = lock_path(&to_virt).await;

    if from.as_ref() == to.as_ref() {
        return Ok(0);
    }

    let from_handle = get_fs_handle(&from, CreateFileMode::NotCreate).await?;
    let to_handle = get_fs_handle(&to, CreateFileMode::Create).await?;

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

    let result = if size <= LARGE_FILE_THRESHOLD {
        copy_small(&stream, &file).await
    } else {
        #[cfg(feature = "opfs_tracing")]
        tracing::debug!(
            from = %from.as_ref().display(),
            to = %to.as_ref().display(),
            size_mb = size / 1024 / 1024,
            "copying large file in chunks"
        );
        copy_large(&stream, &file, size).await
    };

    // Always try to close the stream
    let close_result = JsFuture::from(stream.close()).await.map_err(opfs_err);

    result?;
    close_result?;

    Ok(size)
}

/// Copy small files by writing the entire Blob at once
async fn copy_small(stream: &FileSystemWritableFileStream, file: &web_sys::File) -> io::Result<()> {
    let write_method = Reflect::get(stream, &"write".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>();

    let promise = write_method
        .call1(stream, file)
        .map_err(opfs_err)?
        .unchecked_into::<Promise>();

    JsFuture::from(promise).await.map_err(opfs_err)?;
    Ok(())
}

/// Copy large files in chunks using Blob.slice() to reduce memory pressure
async fn copy_large(
    stream: &FileSystemWritableFileStream,
    file: &web_sys::File,
    size: u64,
) -> io::Result<()> {
    let write_method = Reflect::get(stream, &"write".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>();

    let slice_method: Function = Reflect::get(file, &"slice".into())
        .map_err(opfs_err)?
        .unchecked_into();

    let mut offset: u64 = 0;

    while offset < size {
        let end = (offset + CHUNK_SIZE).min(size);

        let chunk_blob: Blob = slice_method
            .call2(file, &(offset as f64).into(), &(end as f64).into())
            .map_err(opfs_err)?
            .unchecked_into();

        let promise = write_method
            .call1(stream, &chunk_blob)
            .map_err(opfs_err)?
            .unchecked_into::<Promise>();

        JsFuture::from(promise).await.map_err(opfs_err)?;

        offset = end;
    }

    Ok(())
}
