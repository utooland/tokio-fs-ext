use std::{io, path::Path};

use super::opfs::{OpenDirType, open_dir};

pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    open_dir(path, OpenDirType::CreateRecursive)
        .await
        .map(|_| ())
}
