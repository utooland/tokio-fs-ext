use std::{io, path::Path};

use tokio::sync::mpsc;

#[cfg(feature = "opfs_watch")]
use super::opfs::watch::{WatchStream, event, watch_dir};
use super::{
    Metadata, ReadDir, copy, create_dir, create_dir_all, metadata, read, read_dir, remove_dir,
    remove_dir_all, remove_file, write,
};

mod client;
mod server;
mod task;

pub use self::{client::Client, server::Server, task::FsTask};

pub fn split() -> (Server, Client) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (Server { receiver }, Client { sender })
}

#[allow(async_fn_in_trait)]
pub trait FsOffload {
    async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>>;
    async fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()>;
    async fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64>;
    async fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<ReadDir>;
    async fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_dir(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn metadata(&self, path: impl AsRef<Path>) -> io::Result<Metadata>;
    #[cfg(feature = "opfs_watch")]
    async fn watch_dir(&self, path: impl AsRef<Path>, recursive: bool) -> io::Result<WatchStream>;
    #[cfg(feature = "opfs_watch")]
    async fn watch_file(&self, path: impl AsRef<Path>) -> io::Result<WatchStream>;
}

// Separate trait or methods for Client to return MPSC receiver
#[allow(async_fn_in_trait)]
#[cfg(feature = "opfs_watch")]
pub trait FsOffloadWatchStream {
    async fn watch_dir_stream(
        &self,
        path: impl AsRef<Path>,
        recursive: bool,
    ) -> io::Result<EventStream>;
    async fn watch_file_stream(&self, path: impl AsRef<Path>) -> io::Result<EventStream>;
}

#[cfg(feature = "opfs_watch")]
pub struct EventStream {
    receiver: mpsc::UnboundedReceiver<event::Event>,
}

#[cfg(feature = "opfs_watch")]
impl futures::Stream for EventStream {
    type Item = event::Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

pub struct FsOffloadDefault;

impl FsOffload for FsOffloadDefault {
    async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        read(path).await
    }

    async fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        write(path, content).await
    }

    async fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
        copy(from, to).await
    }

    async fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<ReadDir> {
        read_dir(path).await
    }

    async fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        create_dir(path).await
    }

    async fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        create_dir_all(path).await
    }

    async fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        remove_file(path).await
    }

    async fn remove_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        remove_dir(path).await
    }

    async fn remove_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        remove_dir_all(path).await
    }

    async fn metadata(&self, path: impl AsRef<Path>) -> io::Result<Metadata> {
        metadata(path).await
    }

    #[cfg(feature = "opfs_watch")]
    async fn watch_dir(&self, path: impl AsRef<Path>, recursive: bool) -> io::Result<WatchStream> {
        watch_dir(path, recursive).await
    }

    #[cfg(feature = "opfs_watch")]
    async fn watch_file(&self, path: impl AsRef<Path>) -> io::Result<WatchStream> {
        super::opfs::watch::watch_file(path).await
    }
}

#[cfg(feature = "opfs_watch")]
impl FsOffloadWatchStream for Client {
    async fn watch_dir_stream(
        &self,
        path: impl AsRef<Path>,
        recursive: bool,
    ) -> io::Result<EventStream> {
        self.watch_dir(path, recursive).await
    }

    async fn watch_file_stream(&self, path: impl AsRef<Path>) -> io::Result<EventStream> {
        self.watch_file(path).await
    }
}
