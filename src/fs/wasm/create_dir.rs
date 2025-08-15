use std::{io, path::Path};

use super::opfs::{OpenDirType, open_dir};

pub async fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
    open_dir(path, OpenDirType::Create).await?;
    Ok(())
}
