use std::io;

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

/// Helper to convert JsValue to io::Error via OpfsError
#[inline]
pub(crate) fn opfs_err(err: JsValue) -> io::Error {
    OpfsError::from(err).into()
}

pub(crate) struct OpfsError {
    js_err: JsValue,
}

impl From<JsValue> for OpfsError {
    fn from(js_err: JsValue) -> Self {
        Self { js_err }
    }
}

impl From<OpfsError> for io::Error {
    fn from(opfs_err: OpfsError) -> Self {
        match opfs_err.js_err.dyn_ref::<DomException>() {
            Some(e) => match e.name().as_str() {
                "NotFoundError" => io::Error::new(io::ErrorKind::NotFound, e.message()),
                "NotAllowedError" => io::Error::new(io::ErrorKind::PermissionDenied, e.message()),
                // NoModificationAllowedError: file is locked by another SyncAccessHandle
                // Use WouldBlock to indicate the resource is temporarily unavailable
                "NoModificationAllowedError" => {
                    io::Error::new(io::ErrorKind::WouldBlock, e.message())
                }
                "TypeMismatchError" => io::Error::new(io::ErrorKind::InvalidData, e.message()),
                // QuotaExceededError: storage quota exceeded
                "QuotaExceededError" => io::Error::new(io::ErrorKind::StorageFull, e.message()),
                "InvalidStateError" => io::Error::new(io::ErrorKind::InvalidInput, e.message()),
                "SecurityError" => io::Error::new(io::ErrorKind::PermissionDenied, e.message()),
                "AbortError" => io::Error::new(io::ErrorKind::Interrupted, e.message()),
                _ => io::Error::other(format!("{}: {}", e.name(), e.message())),
            },
            None => {
                let msg = match js_sys::Reflect::get(&opfs_err.js_err, &"message".into()) {
                    Ok(m) => m.as_string().unwrap_or_else(|| {
                        Object::from(opfs_err.js_err.clone()).to_string().into()
                    }),
                    Err(_) => Object::from(opfs_err.js_err.clone()).to_string().into(),
                };
                io::Error::other(msg)
            }
        }
    }
}
