use std::{io, path::Path};

use super::Metadata;

pub async fn symlink_metadata(_path: impl AsRef<Path>) -> io::Result<Metadata> {
    todo!()
}
