use std::{
    ffi::OsString,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    str::FromStr,
    task::{Context, Poll},
};

use futures::stream::StreamExt;
use js_sys::{Array, JsString, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, stream::JsStream};
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions};

use crate::fs::opfs::{OpfsError, fs_root};

pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    let name = path.as_ref().to_string_lossy();
    let root = fs_root().await?;
    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(false);
    let directory_handle = JsFuture::from(root.get_directory_handle_with_options(&name, &options))
        .await
        .map_err(|err| io::Error::from(OpfsError::from(err)))?
        .dyn_into::<FileSystemDirectoryHandle>()
        .map_err(|err| io::Error::from(OpfsError::from(err)))?;
    Ok(ReadDir {
        path: path.as_ref().into(),
        stream: JsStream::from(directory_handle.entries()),
    })
}

#[must_use = "streams do nothing unless polled"]
pub struct ReadDir {
    path: PathBuf,
    pub(super) stream: JsStream,
}

impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        self.stream
            .next()
            .await
            .map_or(io::Result::Ok(None), |entry| {
                self.process_opfs_dir_entry(entry)
            })
    }

    pub fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<Option<DirEntry>>> {
        match self.stream.poll_next_unpin(cx) {
            Poll::Ready(entry) => entry.map_or(Poll::Ready(io::Result::Ok(None)), |entry| {
                Poll::Ready(self.process_opfs_dir_entry(entry))
            }),
            Poll::Pending => Poll::Pending,
        }
    }

    fn process_opfs_dir_entry(
        &self,
        entry: Result<JsValue, JsValue>,
    ) -> Result<Option<DirEntry>, io::Error> {
        entry.map_or_else(
            |err| io::Result::Err(io::Error::from(OpfsError::from(err))),
            |entry| {
                let js_array = Array::from(&entry);
                let name = OsString::from_str(
                    JsString::from(js_array.get(0))
                        .as_string()
                        .unwrap()
                        .as_str(),
                )
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidFilename))?;

                let kind = Reflect::get(&js_array.get(1), &JsValue::from(JsString::from("kind")))
                    .map_err(|err| io::Error::from(OpfsError::from(err)))?;
                let file_type = if let Some(kind) = kind.as_string()
                    && kind == "directory"
                {
                    FileType::Directory
                } else {
                    FileType::File
                };

                Ok(Some(DirEntry {
                    file_type,
                    path: self.path.join(name.clone()),
                    name,
                }))
            },
        )
    }
}

/// Symlink is not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
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
