#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => (
        web_sys::console::log_1(&wasm_bindgen::JsValue::from(&format!($($arg)*)))
    )
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        web_sys::console::error_1(&wasm_bindgen::JsValue::from(&format!($($arg)*)))
    )
}

pub use error;
pub use log;
