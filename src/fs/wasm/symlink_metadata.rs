use std::{io, path::Path};

use super::Metadata;

pub async fn symlink_metadata(_path: impl AsRef<Path>) -> io::Result<Metadata> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Symbolic links are not supported on OPFS",
    ))
}
