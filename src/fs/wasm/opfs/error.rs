use std::io;

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

pub(crate) struct OpfsError {
    js_err: JsValue,
}

impl OpfsError {
    pub(crate) fn into_io_err(self) -> io::Error {
        self.into()
    }
}

impl From<JsValue> for OpfsError {
    fn from(js_err: JsValue) -> Self {
        Self { js_err }
    }
}

impl From<OpfsError> for io::Error {
    fn from(opfs_err: OpfsError) -> Self {
        match opfs_err.js_err.dyn_ref::<DomException>() {
            Some(e) => {
                let error_name = e.name();
                let error_message = e.message();
                match error_name.as_str() {
                    "NotFoundError" => io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("NotFoundError: {}", error_message),
                    ),
                    "NotAllowedError" | "NoModificationAllowedError" => io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("{}: {} (this may indicate a file handle is still open or locked)", error_name, error_message),
                    ),
                    "TypeMismatchError" => {
                        io::Error::other(format!("TypeMismatchError: {}", error_message))
                    }
                    _ => io::Error::other(format!("{}: {}", error_name, error_message)),
                }
            }
            None => io::Error::other(format!("{}", Object::from(opfs_err.js_err).to_string())),
        }
    }
}
