use std::{io, path::Path};

use crate::fs::{OpenOptions, opfs::OpfsError};

pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let file = OpenOptions::new().write(true).open(path).await?;

    file.sync_access_handle
        .write_with_u8_array(contents.as_ref())
        .map_err(|err| io::Error::from(OpfsError::from(err)))?;

    Ok(())
}
