use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{get_fs_handle, CreateFileMode, opfs_err};

/// Files larger than this threshold will be written in chunks (10 MB)
const LARGE_FILE_THRESHOLD: usize = 10 * 1024 * 1024;
/// Chunk size for writing large files (2 MB)
const CHUNK_SIZE: usize = 2 * 1024 * 1024;

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    let content = content.as_ref();

    #[cfg(feature = "opfs_tracing")]
    tracing::debug!(path = %path.display(), size = content.len(), "writing file");

    let handle = get_fs_handle(path, CreateFileMode::Create).await?;

    let stream: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let result = if content.len() <= LARGE_FILE_THRESHOLD {
        write_small(&stream, content).await
    } else {
        #[cfg(feature = "opfs_tracing")]
        tracing::debug!(
            path = %path.display(),
            size_mb = content.len() / 1024 / 1024,
            "writing large file in chunks"
        );
        write_large(&stream, content).await
    };

    // Always try to close the stream, even if write failed
    let close_result = JsFuture::from(stream.close()).await.map_err(opfs_err);

    // Return the first error if any
    result?;
    close_result?;

    #[cfg(feature = "opfs_tracing")]
    tracing::debug!(path = %path.display(), "file written successfully");

    Ok(())
}

/// Write small files in a single operation
async fn write_small(stream: &FileSystemWritableFileStream, content: &[u8]) -> io::Result<()> {
    let uint8_array = Uint8Array::from(content);

    let write_method = Reflect::get(stream, &"write".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>();

    let promise = write_method
        .call1(stream, &uint8_array)
        .map_err(opfs_err)?
        .unchecked_into::<Promise>();

    JsFuture::from(promise).await.map_err(opfs_err)?;
    Ok(())
}

/// Write large files in chunks to reduce memory pressure
async fn write_large(stream: &FileSystemWritableFileStream, content: &[u8]) -> io::Result<()> {
    let write_method = Reflect::get(stream, &"write".into())
        .map_err(opfs_err)?
        .unchecked_into::<Function>();

    let total = content.len();
    let mut offset = 0;

    while offset < total {
        let end = (offset + CHUNK_SIZE).min(total);
        let chunk = &content[offset..end];

        let uint8_array = Uint8Array::from(chunk);

        let promise = write_method
            .call1(stream, &uint8_array)
            .map_err(opfs_err)?
            .unchecked_into::<Promise>();

        JsFuture::from(promise).await.map_err(opfs_err)?;

        offset = end;
    }

    Ok(())
}
