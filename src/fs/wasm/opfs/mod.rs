use std::io::{Error, ErrorKind, Result};

use js_sys::Object;
use send_wrapper::SendWrapper;
use tokio::sync::OnceCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DomException, FileSystemDirectoryHandle, window};

static FS_ROOT: OnceCell<SendWrapper<FileSystemDirectoryHandle>> = OnceCell::const_new();

pub async fn fs_root() -> Result<FileSystemDirectoryHandle> {
    let root = FS_ROOT
        .get_or_try_init(|| async {
            Result::Ok(SendWrapper::new(
                JsFuture::from(window().unwrap().navigator().storage().get_directory())
                    .await
                    .map_err(map_opfs_err)?
                    .dyn_into::<FileSystemDirectoryHandle>()
                    .map_err(map_opfs_err)?,
            ))
        })
        .await?;
    Ok(root.clone().take())
}

pub fn map_opfs_err(js_err: JsValue) -> Error {
    match js_err.clone().dyn_into::<DomException>() {
        Ok(e) => match e.name().as_str() {
            "NotFoundError" => Error::from(ErrorKind::NotFound),
            "NotAllowedError" => Error::from(ErrorKind::PermissionDenied),
            msg => Error::other(msg),
        },
        Err(_) => Error::other(format!("{}", Object::from(js_err).to_string())),
    }
}
