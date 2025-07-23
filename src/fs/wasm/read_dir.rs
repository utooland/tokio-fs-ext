use std::{
    ffi::OsString,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    task::{Context, Poll},
};

const CHUNK_SIZE: usize = 32;

pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    todo!()
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct ReadDir {
    // TODO:
}

impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        todo!()
    }

    pub fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<DirEntry>>> {
        todo!()
    }
}

#[derive(Debug)]
pub struct DirEntry {
    // TODO:
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        todo!()
    }

    pub fn file_name(&self) -> OsString {
        todo!()
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}
