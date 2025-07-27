use std::{io, path::Path};

use tokio::io::AsyncSeekExt;

use crate::fs::OpenOptions;

pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await?;

    file.seek(io::SeekFrom::Start(0)).await?;

    file.write_with_buf(contents.as_ref())?;

    Ok(())
}
