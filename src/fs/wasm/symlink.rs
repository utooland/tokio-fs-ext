use std::{io, path::Path};

pub async fn symlink(_original: impl AsRef<Path>, _link: impl AsRef<Path>) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Symbolic links are not supported on OPFS",
    ))
}
