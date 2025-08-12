use std::{
    io,
    path::{Path, PathBuf},
};

use super::opfs::virtualize;

pub async fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    virtualize(path)
}
