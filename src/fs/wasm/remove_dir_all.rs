use std::{io, path::Path};

use super::opfs::remove;

pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    remove(path, true).await
}
