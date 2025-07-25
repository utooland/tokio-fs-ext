use std::{io, path::Path};

use crate::fs::{OpenOptions, opfs::OpfsError};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let file = OpenOptions::new().read(true).open(path).await?;

    let file_size = file
        .sync_access_handle
        .get_size()
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    let mut buf = vec![0; file_size as usize];

    file.sync_access_handle
        .read_with_u8_array(&mut buf)
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(buf)
}
