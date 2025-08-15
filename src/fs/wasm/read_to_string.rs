use std::{io, path::Path};

use super::read;

pub async fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    Ok(unsafe { String::from_utf8_unchecked(read(path).await?) })
}
