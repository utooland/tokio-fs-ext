use std::{
    io,
    path::{Path, PathBuf},
};

use crate::fs::opfs::virtualize;

pub async fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    virtualize(path)
}
