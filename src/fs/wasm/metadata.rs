use std::{io, path::Path};

use web_sys::FileSystemHandleKind;

use super::opfs::{opfs_err, open_dir};

/// Symlink is not supported.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    #[default]
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

impl From<&FileSystemHandleKind> for FileType {
    fn from(handle: &FileSystemHandleKind) -> Self {
        match handle {
            FileSystemHandleKind::File => FileType::File,
            FileSystemHandleKind::Directory => FileType::Directory,
            _ => todo!(),
        }
    }
}

impl From<FileSystemHandleKind> for FileType {
    fn from(handle: FileSystemHandleKind) -> Self {
        (&handle).into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Metadata {
    pub(crate) file_type: FileType,
    pub(crate) file_size: u64,
}

impl Metadata {
    pub fn new(file_type: FileType, file_size: u64) -> Self {
        Self {
            file_type,
            file_size,
        }
    }
}

impl Metadata {
    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.file_size
    }
}

pub async fn metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
    let path = path.as_ref();

    match super::opfs::get_fs_handle(path, super::opfs::CreateFileMode::NotCreate).await {
        Ok(handle) => {
            let file_val = wasm_bindgen_futures::JsFuture::from(handle.get_file())
                .await
                .map_err(opfs_err)?;

            let size = js_sys::Reflect::get(&file_val, &"size".into())
                .map_err(opfs_err)?
                .as_f64()
                .unwrap_or(0.0) as u64;

            Ok(Metadata {
                file_type: FileType::File,
                file_size: size,
            })
        }
        Err(_) => Ok(open_dir(path, super::opfs::OpenDirType::NotCreate)
            .await
            .map(|_| Metadata {
                file_type: FileType::Directory,
                file_size: 0,
            })?),
    }
}
