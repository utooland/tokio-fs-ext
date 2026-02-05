use std::{io, path::Path};

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::File as JsFile;

use super::opfs::{CreateFileMode, SyncAccessMode, open_file, opfs_err};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let file = open_file(path, CreateFileMode::NotCreate, SyncAccessMode::Readonly, false).await?;

    let js_file: JsFile = JsFuture::from(file.handle.get_file())
        .await
        .map_err(opfs_err)?
        .unchecked_into();

    let array_buffer = JsFuture::from(js_file.array_buffer())
        .await
        .map_err(opfs_err)?;

    Ok(Uint8Array::new(&array_buffer).to_vec())
}
