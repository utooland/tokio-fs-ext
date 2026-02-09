use std::{io, path::Path};

use super::opfs::remove;

pub async fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    let (mut _guard, _) = super::file::lock_file(path, None).await;
    remove(path, false).await
}
