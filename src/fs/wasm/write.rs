use std::{io, path::Path};

use crate::fs::{OpenOptions, wasm::opfs::map_opfs_err};

pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let file = OpenOptions::new().write(true).open(path).await?;

    file.sync_access_handle
        .write_with_u8_array(contents.as_ref())
        .map_err(map_opfs_err)?;

    Ok(())
}
