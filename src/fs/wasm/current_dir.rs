use std::{
    io,
    path::{Path, PathBuf},
    sync::RwLock,
};

static CWD: RwLock<PathBuf> = RwLock::new(PathBuf::new());

pub fn current_dir() -> io::Result<PathBuf> {
    let cwd = CWD
        .read()
        .map_err(|_| io::Error::from(io::ErrorKind::Deadlock))?;

    Ok(cwd.clone())
}

pub fn set_current_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut cwd = CWD
        .write()
        .map_err(|_| io::Error::from(io::ErrorKind::Deadlock))?;

    *cwd = path.as_ref().into();

    Ok(())
}
