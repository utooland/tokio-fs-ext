use std::{io, path::Path};

use tokio::sync::{mpsc, oneshot};

#[cfg(feature = "opfs_watch")]
use super::super::opfs::watch::event;
use super::{FsTask, Metadata, ReadDir};

#[cfg(feature = "opfs_watch")]
use tokio::sync::mpsc as watch_mpsc;

/// Handle to stop watching in offload mode. Call `stop()` to cancel the watch.
#[cfg(feature = "opfs_watch")]
pub struct OffloadWatchHandle {
    stop_sender: Option<watch_mpsc::Sender<()>>,
}

// Ensure OffloadWatchHandle is Send + Sync for multi-threaded WASM
#[cfg(feature = "opfs_watch")]
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OffloadWatchHandle>();
};

#[cfg(feature = "opfs_watch")]
impl OffloadWatchHandle {
    /// Stop watching and release resources on the server side
    pub async fn stop(mut self) {
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(()).await;
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub(super) sender: mpsc::UnboundedSender<FsTask>,
}

impl Client {
    pub async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::Read { path, sender }).await
    }

    pub async fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        let path = path.as_ref().into();
        let content = content.as_ref().to_vec();
        self.dispatch(|sender| FsTask::Write {
            path,
            sender,
            content,
        })
        .await
    }

    pub async fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<ReadDir> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::ReadDir { path, sender })
            .await
    }

    pub async fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::CreateDir { path, sender })
            .await
    }

    pub async fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::CreateDirAll { path, sender })
            .await
    }

    pub async fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::RemoveFile { path, sender })
            .await
    }

    pub async fn remove_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::RemoveDir { path, sender })
            .await
    }

    pub async fn remove_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::RemoveDirAll { path, sender })
            .await
    }

    pub async fn metadata(&self, path: impl AsRef<Path>) -> io::Result<Metadata> {
        let path = path.as_ref().into();
        self.dispatch(|sender| FsTask::Metadata { path, sender })
            .await
    }

    #[cfg(feature = "opfs_watch")]
    pub async fn watch_dir(
        &self,
        path: impl AsRef<Path>,
        recursive: bool,
        cb: impl Fn(event::Event) + Send + Sync + 'static,
    ) -> io::Result<OffloadWatchHandle> {
        let path = path.as_ref().into();
        let stop_sender = self
            .dispatch(|sender| FsTask::WatchDir {
                path,
                recursive,
                cb: Box::new(cb),
                sender,
            })
            .await?;
        Ok(OffloadWatchHandle {
            stop_sender: Some(stop_sender),
        })
    }

    async fn dispatch<T, F>(&self, create_task: F) -> io::Result<T>
    where
        F: FnOnce(oneshot::Sender<io::Result<T>>) -> FsTask,
    {
        let (sender, receiver) = oneshot::channel();

        let task = create_task(sender);

        self.sender
            .send(task)
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?;

        receiver
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::ConnectionAborted))?
    }
}
