use std::{io, path::Path};

use crate::fs::{OpenOptions, opfs::OpfsError};

pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await?;

    file.sync_access_handle
        .write_with_u8_array(contents.as_ref())
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}
