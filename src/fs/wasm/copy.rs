use std::{io, path::Path};

use super::opfs::virtualize;
use super::{read, write};

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    let from_canonical = virtualize(from.as_ref())?;
    let to_canonical = virtualize(to.as_ref())?;
    if from_canonical == to_canonical {
        return Ok(0);
    }
    let contents = read(from_canonical).await?;
    write(to_canonical, &contents).await?;
    Ok(contents.len() as u64)
}
