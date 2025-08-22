use std::{io, path::Path};

use futures::io::AsyncSeekExt;

use super::OpenOptions;

pub async fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await?;

    file.seek(io::SeekFrom::Start(0)).await?;

    file.write_with_buf(content.as_ref())?;

    Ok(())
}
