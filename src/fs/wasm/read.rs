use std::{io, path::Path};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

use super::opfs::{CreateFileMode, opfs_err};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    // Use Shared lock to allow concurrent reads and wait for exclusive writers.
    let (guard, _sync_handle, file_handle) = super::opfs::lock_and_handle(
        &path,
        Some(super::opfs::SyncAccessMode::Readonly),
        CreateFileMode::NotCreate,
    )
    .await?;

    let file: File = JsFuture::from(file_handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(opfs_err)?;

    let uint8_array = Uint8Array::new(&array_buffer);
    let vec = uint8_array.to_vec();

    drop(guard);
    Ok(vec)
}
