use std::io;

use super::opfs::{CreateFileMode, OpenDirType, open_dir, resolve_file_handle};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    Ok(resolve_file_handle(&path, CreateFileMode::NotCreate)
        .await
        .is_ok()
        || open_dir(path, OpenDirType::NotCreate).await.is_ok())
}
