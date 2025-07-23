use std::{io, path::Path};

use crate::fs::{OpenOptions, wasm::opfs::map_opfs_err};

pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let file = OpenOptions::new().read(true).open(path).await?;

    let file_size = file.sync_access_handle.get_size().map_err(map_opfs_err)?;

    let mut buf = Vec::with_capacity(file_size as usize);

    file.sync_access_handle
        .read_with_u8_array(&mut buf)
        .map_err(map_opfs_err)?;

    Ok(buf)
}
