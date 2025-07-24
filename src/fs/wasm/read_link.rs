use std::{
    io,
    path::{Path, PathBuf},
};

pub async fn read_link(_path: impl AsRef<Path>) -> io::Result<PathBuf> {
    todo!()
}
