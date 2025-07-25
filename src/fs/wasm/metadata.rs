use std::{fs::Metadata, io, path::Path};

use web_sys::FileSystemHandleKind;

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

pub async fn metadata(_path: impl AsRef<Path>) -> io::Result<Metadata> {
    todo!()
}
