use std::{io, path::Path};

use tokio::io::AsyncSeekExt;

use crate::fs::OpenOptions;

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(path).await?;

    file.seek(io::SeekFrom::Start(0)).await?;

    let file_size = file.size()?;

    let mut buf = vec![0; file_size as usize];

    file.read_with_buf(&mut buf)?;

    Ok(buf)
}
