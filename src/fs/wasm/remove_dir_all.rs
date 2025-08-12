use std::{io, path::Path};

use crate::fs::opfs::remove;

pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    remove(path, true).await
}
