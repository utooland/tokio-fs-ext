use std::io;

use js_sys::Object;
use send_wrapper::SendWrapper;
use tokio::sync::OnceCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DomException, FileSystemDirectoryHandle, window};

static FS_ROOT: OnceCell<SendWrapper<FileSystemDirectoryHandle>> = OnceCell::const_new();

pub async fn fs_root() -> io::Result<FileSystemDirectoryHandle> {
    let root = FS_ROOT
        .get_or_try_init(|| async {
            io::Result::Ok(SendWrapper::new(
                JsFuture::from(window().unwrap().navigator().storage().get_directory())
                    .await
                    .map_err(|err| io::Error::from(OpfsError::from(err)))?
                    .dyn_into::<FileSystemDirectoryHandle>()
                    .map_err(|err| io::Error::from(OpfsError::from(err)))?,
            ))
        })
        .await?;
    Ok(root.clone().take())
}

pub struct OpfsError {
    js_err: JsValue,
}

impl From<JsValue> for OpfsError {
    fn from(js_err: JsValue) -> Self {
        Self { js_err }
    }
}

impl From<OpfsError> for io::Error {
    fn from(opfs_err: OpfsError) -> Self {
        match opfs_err.js_err.clone().dyn_into::<DomException>() {
            Ok(e) => match e.name().as_str() {
                "NotFoundError" => io::Error::from(io::ErrorKind::NotFound),
                "NotAllowedError" => io::Error::from(io::ErrorKind::PermissionDenied),
                "TypeMismatchError" => io::Error::other("type mismatch"),
                msg => io::Error::other(msg),
            },
            Err(_) => io::Error::other(format!("{}", Object::from(opfs_err.js_err).to_string())),
        }
    }
}
