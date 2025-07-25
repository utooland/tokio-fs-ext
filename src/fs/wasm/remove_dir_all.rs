use std::{io, path::Path};

use crate::fs::opfs::rm;

pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    rm(path, true).await
}
