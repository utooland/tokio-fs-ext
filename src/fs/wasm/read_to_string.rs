use std::{io, path::Path};

use super::read;

pub async fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    String::from_utf8(read(path).await?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
