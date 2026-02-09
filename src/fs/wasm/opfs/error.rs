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
                "NotFoundError" => io::Error::from(io::ErrorKind::NotFound),
                "NotAllowedError" => io::Error::from(io::ErrorKind::PermissionDenied),
                // NoModificationAllowedError: file is locked by another SyncAccessHandle
                // Use WouldBlock to indicate the resource is temporarily unavailable
                "NoModificationAllowedError" => io::Error::from(io::ErrorKind::WouldBlock),
                "TypeMismatchError" => io::Error::new(io::ErrorKind::InvalidData, "type mismatch"),
                // QuotaExceededError: storage quota exceeded
                "QuotaExceededError" => io::Error::from(io::ErrorKind::StorageFull),
                "InvalidStateError" => io::Error::new(io::ErrorKind::InvalidInput, "invalid state"),
                "SecurityError" => io::Error::from(io::ErrorKind::PermissionDenied),
                "AbortError" => io::Error::new(io::ErrorKind::Interrupted, "operation aborted"),
                msg => io::Error::other(msg),
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
