use std::{io, path::Path};

pub async fn symlink(_original: impl AsRef<Path>, _link: impl AsRef<Path>) -> io::Result<()> {
    todo!()
}
