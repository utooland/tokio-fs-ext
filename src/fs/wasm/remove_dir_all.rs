use std::{io, path::Path};

use crate::fs::opfs::rm_dir;

pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    rm_dir(path, true).await
}
