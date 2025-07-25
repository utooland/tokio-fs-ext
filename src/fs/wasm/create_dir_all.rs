use std::{io, path::Path};

use crate::fs::opfs::open_dir;

pub async fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    open_dir(path, true, true).await?;
    Ok(())
}
