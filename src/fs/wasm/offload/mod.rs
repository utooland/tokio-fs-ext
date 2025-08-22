use std::{io, path::PathBuf};

use tokio::sync::{mpsc, oneshot};

use super::{
    Metadata, ReadDir, create_dir, create_dir_all, metadata, read, read_dir, remove_dir,
    remove_dir_all, remove_file, write,
};

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

#[allow(async_fn_in_trait)]
pub trait FsOffload {
    async fn read(&self, path: PathBuf) -> io::Result<Vec<u8>>;
    async fn write(&self, path: PathBuf, content: Vec<u8>) -> io::Result<()>;
    async fn read_dir(&self, path: PathBuf) -> io::Result<ReadDir>;
    async fn create_dir(&self, path: PathBuf) -> io::Result<()>;
    async fn create_dir_all(&self, path: PathBuf) -> io::Result<()>;
    async fn remove_file(&self, path: PathBuf) -> io::Result<()>;
    async fn remove_dir(&self, path: PathBuf) -> io::Result<()>;
    async fn remove_dir_all(&self, path: PathBuf) -> io::Result<()>;
    async fn metadata(&self, path: PathBuf) -> io::Result<Metadata>;
}

#[derive(Clone)]
pub struct FsActorHandle {
    sender: mpsc::Sender<FsTask>,
}

impl FsOffload for FsActorHandle {
    async fn read(&self, path: PathBuf) -> io::Result<Vec<u8>> {
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

    async fn write(&self, path: PathBuf, content: Vec<u8>) -> io::Result<()> {
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

    async fn read_dir(&self, path: PathBuf) -> io::Result<ReadDir> {
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

    async fn create_dir(&self, path: PathBuf) -> io::Result<()> {
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

    async fn create_dir_all(&self, path: PathBuf) -> io::Result<()> {
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

    async fn remove_file(&self, path: PathBuf) -> io::Result<()> {
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

    async fn remove_dir(&self, path: PathBuf) -> io::Result<()> {
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

    async fn remove_dir_all(&self, path: PathBuf) -> io::Result<()> {
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

    async fn metadata(&self, path: PathBuf) -> io::Result<Metadata> {
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

pub struct FsActor {
    receiver: mpsc::Receiver<FsTask>,
}

impl FsActor {
    pub fn create() -> (FsActor, FsActorHandle) {
        let (sender, receiver) = mpsc::channel(32);
        (FsActor { receiver }, FsActorHandle { sender })
    }

    pub async fn run(mut self) {
        while let Some(task) = self.receiver.recv().await {
            match task {
                FsTask::Read { path, sender } => {
                    let _ = sender.send(read(path).await);
                }
                FsTask::Write {
                    path,
                    content,
                    sender,
                } => {
                    let _ = sender.send(write(path, content).await);
                }
                FsTask::ReadDir { path, sender } => {
                    let _ = sender.send(read_dir(path).await);
                }
                FsTask::CreateDir { path, sender } => {
                    let _ = sender.send(create_dir(path).await);
                }
                FsTask::CreateDirAll { path, sender } => {
                    let _ = sender.send(create_dir_all(path).await);
                }
                FsTask::RemoveFile { path, sender } => {
                    let _ = sender.send(remove_file(path).await);
                }
                FsTask::RemoveDir { path, sender } => {
                    let _ = sender.send(remove_dir(path).await);
                }
                FsTask::RemoveDirAll { path, sender } => {
                    let _ = sender.send(remove_dir_all(path).await);
                }
                FsTask::Metadata { path, sender } => {
                    let _ = sender.send(metadata(path).await);
                }
            }
        }
    }
}
