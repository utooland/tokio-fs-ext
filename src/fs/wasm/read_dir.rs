use std::{
    ffi::OsString,
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    str::FromStr,
    task::{Context, Poll},
};

use futures::{TryStreamExt, stream::StreamExt};
use js_sys::{Array, JsString};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::stream::JsStream;
use web_sys::FileSystemHandle;

use super::{
    metadata::{FileType, Metadata},
    opfs::{OpfsError, open_dir},
};

pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    let dir_handle = open_dir(&path, super::opfs::OpenDirType::NotCreate).await?;
    let entries = JsStream::from(dir_handle.entries())
        .map(|handle| {
            handle.map_or_else(
                |err| Err(OpfsError::from(err).into_io_err()),
                |entry| {
                    Ok({
                        let js_array = Array::from(&entry);

                        let name = OsString::from_str(
                            JsString::from(js_array.get(0))
                                .as_string()
                                .ok_or(io::Error::from(io::ErrorKind::InvalidFilename))?
                                .as_str(),
                        )
                        .map_err(|_| io::Error::from(io::ErrorKind::InvalidFilename))?;

                        let path = path.as_ref().join(&name);

                        let file_type = js_array
                            .get(1)
                            .unchecked_into::<FileSystemHandle>()
                            .kind()
                            .into();

                        DirEntry {
                            file_type,
                            path,
                            name,
                        }
                    })
                },
            )
        })
        .try_collect()
        .await?;

    Ok(ReadDir { entries })
}

#[derive(Debug)]
pub struct ReadDir {
    entries: Vec<DirEntry>,
}

impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        Ok(self.entries.pop())
    }

    pub fn poll_next_entry(&mut self, _cx: &mut Context<'_>) -> Poll<io::Result<Option<DirEntry>>> {
        Poll::Ready(Ok(self.entries.pop()))
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.entries.pop().map(Result::Ok)
    }
}

impl FromIterator<DirEntry> for ReadDir {
    fn from_iter<T: IntoIterator<Item = DirEntry>>(iter: T) -> Self {
        ReadDir {
            entries: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug)]
pub struct DirEntry {
    file_type: FileType,
    name: OsString,
    path: PathBuf,
}

impl DirEntry {
    /// Create a new `DirEntry`.
    ///
    /// This is useful for constructing mock directory entries for testing.
    pub fn new(path: PathBuf, name: OsString, file_type: FileType) -> Self {
        Self {
            file_type,
            name,
            path,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.file_type)
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}
