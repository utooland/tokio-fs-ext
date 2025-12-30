use std::{io, path::Path};

use futures::io::AsyncReadExt;

use super::OpenOptions;

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(path).await?;

    let mut buf = vec![];

    file.read_to_end(&mut buf).await?;

    Ok(buf)
}
