use std::{
    io,
    path::{Path, PathBuf},
};

use path_absolutize::Absolutize;

pub async fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    // FIXME: resolve the real file or dir
    path.as_ref().absolutize().map(Into::into)
}
