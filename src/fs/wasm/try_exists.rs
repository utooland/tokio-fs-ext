use std::io;

use crate::fs::{read_dir, wasm::file::open_file};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    Ok(open_file(&path, false, false).await.is_ok() || read_dir(path).await.is_ok())
}
