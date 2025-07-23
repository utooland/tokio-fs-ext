use std::{
    io,
    path::{Path, PathBuf},
};

use path_absolutize::Absolutize;

pub async fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    path.as_ref().absolutize().map(Into::into)
}
