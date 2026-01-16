use std::{io, path::Path};

use js_sys::{Function, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File};

use super::opfs::{CreateFileMode, OpfsError, get_fs_handle};

/// Files larger than this threshold will be read in chunks (10 MB)
const LARGE_FILE_THRESHOLD: usize = 10 * 1024 * 1024;
/// Chunk size for reading large files (2 MB)
const CHUNK_SIZE: usize = 2 * 1024 * 1024;

/// Helper to convert OpfsError to io::Error
#[inline]
fn opfs_err(err: wasm_bindgen::JsValue) -> io::Error {
    OpfsError::from(err).into_io_err()
}

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let handle = get_fs_handle(&path, CreateFileMode::NotCreate).await?;

    let file: File = JsFuture::from(handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let size = file.size() as usize;

    if size == 0 {
        return Ok(Vec::new());
    }

    // For small files, read directly without chunking
    if size <= LARGE_FILE_THRESHOLD {
        return read_small_file(&file, size).await;
    }

    // For large files, read in chunks to avoid JS memory spike
    crate::console::warning!(
        "Reading large file: {}, size: {} MB",
        path.as_ref().display(),
        size / 1024 / 1024
    );

    read_large_file(&file, size).await
}

/// Read small files directly in one operation
async fn read_small_file(file: &File, size: usize) -> io::Result<Vec<u8>> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(opfs_err)?;

    let uint8_array = Uint8Array::new(&array_buffer);
    let len = uint8_array.length() as usize;

    let mut output = Vec::<u8>::new();
    output
        .try_reserve_exact(size)
        .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))?;

    // SAFETY: We reserved enough capacity and copy the exact length
    unsafe {
        uint8_array.raw_copy_to_ptr(output.as_mut_ptr());
        output.set_len(len);
    }
    Ok(output)
}

/// Read large files in chunks to reduce memory pressure
async fn read_large_file(file: &File, size: usize) -> io::Result<Vec<u8>> {
    // Pre-allocate total memory, handling OOM gracefully
    let mut output = Vec::<u8>::new();
    output
        .try_reserve_exact(size)
        .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))?;

    // Get Blob.slice method via Reflect for >2GB file support
    let slice_method: Function = Reflect::get(file, &"slice".into())
        .map_err(opfs_err)?
        .unchecked_into();

    let mut offset = 0;
    while offset < size {
        let end = (offset + CHUNK_SIZE).min(size);

        let chunk_blob: Blob = slice_method
            .call2(file, &offset.into(), &end.into())
            .map_err(opfs_err)?
            .unchecked_into();

        let array_buffer = JsFuture::from(chunk_blob.array_buffer())
            .await
            .map_err(opfs_err)?;

        let uint8_array = Uint8Array::new(&array_buffer);
        let chunk_len = uint8_array.length() as usize;

        // SAFETY: We pre-allocated `size` bytes and write to the correct offset
        unsafe {
            uint8_array.raw_copy_to_ptr(output.as_mut_ptr().add(offset));
            output.set_len(offset + chunk_len);
        }

        offset += chunk_len;
    }

    Ok(output)
}
