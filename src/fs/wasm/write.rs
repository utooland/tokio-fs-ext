use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use super::opfs::{get_fs_handle, CreateFileMode, OpfsError};

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let handle = get_fs_handle(path, CreateFileMode::Create).await?;

    let stream: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into();

    let content = content.as_ref();
    let uint8_array = Uint8Array::from(content);

    let write_method = Reflect::get(&stream, &"write".into())
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Function>();

    let promise = write_method
        .call1(&stream, &uint8_array)
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Promise>();

    JsFuture::from(promise)
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    JsFuture::from(stream.close())
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}
