use std::{io, path::Path};

use crate::fs::opfs::rm;

pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    rm(path, false).await
}
