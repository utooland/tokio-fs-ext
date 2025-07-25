use std::{io, path::Path};

use crate::fs::opfs::rm_dir;

pub async fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
    rm_dir(path, false).await
}
