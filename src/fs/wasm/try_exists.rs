use std::io;

use super::opfs::{CreateFileMode, OpenDirType, get_fs_handle, open_dir};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    let path = path.as_ref();
    Ok(get_fs_handle(path, CreateFileMode::NotCreate).await.is_ok()
        || open_dir(path, OpenDirType::NotCreate).await.is_ok())
}
