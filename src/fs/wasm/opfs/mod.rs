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
                    .map_err(opfs_error_to_std_error)?
                    .dyn_into::<FileSystemDirectoryHandle>()
                    .map_err(opfs_error_to_std_error)?,
            ))
        })
        .await?;
    Ok(root.clone().take())
}

pub fn opfs_error_to_std_error(v: JsValue) -> Error {
    match v.clone().dyn_into::<DomException>() {
        Ok(e) => match e.name().as_str() {
            "NotFoundError" => Error::from(ErrorKind::NotFound),
            "NotAllowedError" | "NoModificationAllowedError" => {
                Error::from(ErrorKind::PermissionDenied)
            }
            msg => Error::other(msg),
        },
        Err(_) => Error::other(format!("{}", Object::from(v).to_string())),
    }
}
