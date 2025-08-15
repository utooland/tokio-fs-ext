use std::{io, path::Path};

use super::opfs::remove;

pub async fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
    remove(path, false).await
}
