use std::{io, path::PathBuf};

use tokio::sync::oneshot;

use super::{Metadata, ReadDir};

pub enum FsTask {
    Read {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<Vec<u8>>>,
    },
    Write {
        path: PathBuf,
        content: Vec<u8>,
        sender: oneshot::Sender<io::Result<()>>,
    },
    ReadDir {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<ReadDir>>,
    },
    CreateDir {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<()>>,
    },
    CreateDirAll {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<()>>,
    },
    RemoveFile {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<()>>,
    },
    RemoveDir {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<()>>,
    },
    RemoveDirAll {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<()>>,
    },
    Metadata {
        path: PathBuf,
        sender: oneshot::Sender<io::Result<Metadata>>,
    },
}
