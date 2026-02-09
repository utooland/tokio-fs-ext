use std::{io, path::Path};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemReadWriteOptions, FileSystemWritableFileStream};

use super::opfs::{CreateFileMode, opfs_err};

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let (guard, sync_handle, file_handle) = super::opfs::lock_and_handle(
        path,
        Some(super::opfs::SyncAccessMode::Readwrite),
        CreateFileMode::Create,
    )
    .await?;
    let content_slice = content.as_ref();

    if let Some(sync_access) = sync_handle {
        // A File is already open (SyncAccessHandle is exclusive).
        // Reuse it to write data.

        // fs::write overwrites the entire file.
        sync_access.truncate_with_u32(0).map_err(opfs_err)?;

        if !content_slice.is_empty() {
            let options = FileSystemReadWriteOptions::new();
            options.set_at(0.0);
            sync_access
                .write_with_u8_array_and_options(content_slice, &options)
                .map_err(opfs_err)?;
        }

        sync_access.flush().map_err(opfs_err)?;

        drop(guard);
        Ok(())
    } else {
        let stream: FileSystemWritableFileStream = JsFuture::from(file_handle.create_writable())
            .await
            .map_err(opfs_err)?
            .unchecked_into();

        // Create a fresh Uint8Array tailored for the JS side.
        // This performs a copy, which ensures safety even if the source is backed by SharedArrayBuffer
        // (avoiding data races or detachment issues during the async write).
        let content_js = Uint8Array::from(content_slice);

        JsFuture::from(
            stream
                .write_with_buffer_source(&content_js)
                .map_err(opfs_err)?,
        )
        .await
        .map_err(opfs_err)?;

        JsFuture::from(stream.close()).await.map_err(opfs_err)?;

        drop(guard);
        Ok(())
    }
}
