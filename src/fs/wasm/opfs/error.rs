use std::io;

use js_sys::Object;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::DomException;

pub struct OpfsError {
    pub(crate) js_err: JsValue,
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
            Some(e) => match e.name().as_str() {
                "NotFoundError" => io::Error::from(io::ErrorKind::NotFound),
                "NotAllowedError" | "NoModificationAllowedError" => {
                    io::Error::from(io::ErrorKind::PermissionDenied)
                }
                "TypeMismatchError" => io::Error::other("type mismatch"),
                msg => io::Error::other(msg),
            },
            None => io::Error::other(format!("{}", Object::from(opfs_err.js_err).to_string())),
        }
    }
}
