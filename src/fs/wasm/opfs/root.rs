use std::io;

use send_wrapper::SendWrapper;
use tokio::sync::OnceCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DedicatedWorkerGlobalScope, FileSystemDirectoryHandle};

use super::OpfsError;

static OPFS_ROOT: OnceCell<SendWrapper<FileSystemDirectoryHandle>> = OnceCell::const_new();

pub(super) async fn root() -> io::Result<SendWrapper<FileSystemDirectoryHandle>> {
    let root = OPFS_ROOT
        .get_or_try_init(|| async {
            let storage = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()))
                .navigator()
                .storage();
            let root_handle =
                SendWrapper::new(JsFuture::from(SendWrapper::new(storage).get_directory()))
                    .await
                    .map_err(|err| OpfsError::from(err).into_io_err())?
                    .dyn_into::<FileSystemDirectoryHandle>()
                    .map_err(|err| OpfsError::from(err).into_io_err())?;
            io::Result::Ok(SendWrapper::new(root_handle))
        })
        .await?;

    Ok(root.clone())
}
