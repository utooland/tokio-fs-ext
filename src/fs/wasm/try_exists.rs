use std::io;

use crate::fs::{file::open_file, read_dir};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    Ok(open_file(&path, false, false).await.is_ok() || read_dir(path).await.is_ok())
}
