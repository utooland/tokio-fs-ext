use std::{io, path::Path};

use crate::fs::opfs::remove;

pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    remove(path, false).await
}
