use std::io;

use crate::fs::opfs::{open_dir, open_file};

pub async fn try_exists(path: impl AsRef<std::path::Path>) -> io::Result<bool> {
    Ok(open_file(&path, false, false).await.is_ok()
        || open_dir(path, super::opfs::OpenDirType::NotCreate)
            .await
            .is_ok())
}
