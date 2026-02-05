use std::{io, path::Path};

use super::{read, write};

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    if from.as_ref() == to.as_ref() {
        return Ok(0);
    }
    let contents = read(from).await?;
    write(to, &contents).await?;
    Ok(contents.len() as u64)
}
