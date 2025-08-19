use std::{cell::RefCell, io};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DedicatedWorkerGlobalScope, FileSystemDirectoryHandle};

use super::OpfsError;

thread_local! {
    static CACHED_ROOT: RefCell<Option<FileSystemDirectoryHandle>> = const { RefCell::new(None) };
}

pub(super) async fn root() -> io::Result<FileSystemDirectoryHandle> {
    let cached = CACHED_ROOT.with(|cell| cell.borrow().clone());
    match cached {
        None => {
            let storage = DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()))
                .navigator()
                .storage();
            let root_handle = JsFuture::from(storage.get_directory())
                .await
                .map_err(|err| OpfsError::from(err).into_io_err())?
                .dyn_into::<FileSystemDirectoryHandle>()
                .map_err(|err| OpfsError::from(err).into_io_err())?;
            CACHED_ROOT.with(|cell| cell.replace(Some(root_handle.clone())));
            Ok(root_handle)
        }
        Some(root) => Ok(root.clone()),
    }
}
