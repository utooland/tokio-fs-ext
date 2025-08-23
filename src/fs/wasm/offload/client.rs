use std::{io, path::PathBuf};

use tokio::sync::{mpsc, oneshot};

use super::{FsTask, Metadata, ReadDir};

#[derive(Clone)]
pub struct Client {
    pub(super) sender: mpsc::Sender<FsTask>,
}

impl Client {
    pub async fn read(&self, path: PathBuf) -> io::Result<Vec<u8>> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::Read { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn write(&self, path: PathBuf, content: Vec<u8>) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::Write {
            path,
            sender,
            content,
        };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn read_dir(&self, path: PathBuf) -> io::Result<ReadDir> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::ReadDir { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn create_dir(&self, path: PathBuf) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::CreateDir { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn create_dir_all(&self, path: PathBuf) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::CreateDirAll { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn remove_file(&self, path: PathBuf) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::RemoveFile { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn remove_dir(&self, path: PathBuf) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::RemoveDir { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn remove_dir_all(&self, path: PathBuf) -> io::Result<()> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::RemoveDirAll { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }

    pub async fn metadata(&self, path: PathBuf) -> io::Result<Metadata> {
        let (sender, receiver) = oneshot::channel();

        let task = FsTask::Metadata { path, sender };

        self.sender
            .send(task)
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }
}
