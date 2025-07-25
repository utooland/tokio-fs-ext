use std::io;

use crate::fs::{dir::open_dir, file::open_file};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    Ok(open_file(&path, false, false).await.is_ok() || open_dir(path, false, true).await.is_ok())
}
