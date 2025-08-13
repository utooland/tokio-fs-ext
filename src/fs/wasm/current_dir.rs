use std::{
    io,
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
};

static CURRENT_DIR: LazyLock<RwLock<PathBuf>> = LazyLock::new(|| RwLock::new(PathBuf::from("/")));

pub fn current_dir() -> io::Result<PathBuf> {
    let cwd = CURRENT_DIR
        .read()
        .map_err(|_| io::Error::from(io::ErrorKind::Deadlock))?;

    Ok(cwd.clone())
}

pub fn set_current_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut cwd = CURRENT_DIR
        .write()
        .map_err(|_| io::Error::from(io::ErrorKind::Deadlock))?;

    *cwd = path.as_ref().into();

    Ok(())
}
