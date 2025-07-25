use std::{
    ffi::OsString,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    str::FromStr,
    task::{Context, Poll},
};

use futures::stream::StreamExt;
use js_sys::{Array, JsString};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::stream::JsStream;
use web_sys::{FileSystemHandle, FileSystemHandleKind};

use crate::fs::opfs::{OpfsError, open_dir};

pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    let dir_handle = open_dir(&path, false, true).await?;
    Ok(ReadDir {
        path: path.as_ref().into(),
        stream: JsStream::from(dir_handle.entries()),
    })
}

#[must_use = "streams do nothing unless polled"]
pub struct ReadDir {
    path: PathBuf,
    pub(super) stream: JsStream,
}

impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        match self.stream.next().await {
            Some(next) => match next {
                Ok(next) => Ok(Some(self.process_entry(&next)?)),
                Err(err) => Err(OpfsError::from(err).into_io_err()),
            },
            None => io::Result::Ok(None),
        }
    }

    pub fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<DirEntry>>> {
        match self.stream.poll_next_unpin(cx) {
            Poll::Ready(next) => match next {
                Some(next) => match next {
                    Ok(next) => Poll::Ready(Ok(Some(self.process_entry(&next)?))),
                    Err(err) => Poll::Ready(Err(OpfsError::from(err).into_io_err())),
                },
                None => todo!(),
            },
            Poll::Pending => Poll::Pending,
        }
    }

    fn process_entry(&self, entry: &JsValue) -> io::Result<DirEntry> {
        let js_array = Array::from(entry);

        let name = OsString::from_str(
            JsString::from(js_array.get(0))
                .as_string()
                .ok_or(io::Error::from(io::ErrorKind::InvalidFilename))?
                .as_str(),
        )
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidFilename))?;

        let handle = js_array
            .get(1)
            .dyn_into::<FileSystemHandle>()
            .map_err(|err| OpfsError::from(err).into_io_err())?;

        io::Result::Ok(DirEntry {
            file_type: handle.kind().into(),
            path: self.path.join(name.clone()),
            name,
        })
    }
}

/// Symlink is not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    // TODO:
    Symlink,
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        *self == Self::Directory
    }
    pub fn is_file(&self) -> bool {
        *self == Self::File
    }
    pub fn is_symlink(&self) -> bool {
        *self == Self::Symlink
    }
}

impl From<FileSystemHandleKind> for FileType {
    fn from(handle: FileSystemHandleKind) -> Self {
        match handle {
            FileSystemHandleKind::File => FileType::File,
            FileSystemHandleKind::Directory => FileType::Directory,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    file_type: FileType,
    name: OsString,
    path: PathBuf,
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}
