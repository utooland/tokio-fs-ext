use std::{io, path::Path};

use crate::fs::{copy, remove_file};

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    // TODO: rename dir
    copy(&from, to).await?;
    remove_file(from).await?;
    Ok(())
}
