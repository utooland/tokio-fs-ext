use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::opfs::{get_fs_handle, CreateFileMode, OpfsError};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let handle = get_fs_handle(path, CreateFileMode::NotCreate).await?;

    let file_val = JsFuture::from(handle.get_file())
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    let array_buffer_method = Reflect::get(&file_val, &"arrayBuffer".into())
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Function>();

    let promise = array_buffer_method.call0(&file_val)
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Promise>();

    let array_buffer = JsFuture::from(promise)
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    let uint8_array = Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}
