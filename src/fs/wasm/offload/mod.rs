use std::{io, path::Path};

use tokio::sync::mpsc;

use super::{
    Metadata, ReadDir, create_dir, create_dir_all, metadata, read, read_dir, remove_dir,
    remove_dir_all, remove_file, write,
};

mod client;
mod server;
mod task;

pub use self::{client::Client, server::Server, task::FsTask};

pub fn split() -> (Server, Client) {
    let (sender, receiver) = mpsc::channel(32);
    (Server { receiver }, Client { sender })
}

#[allow(async_fn_in_trait)]
pub trait FsOffload {
    async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>>;
    async fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()>;
    async fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<ReadDir>;
    async fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_file(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_dir(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn remove_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()>;
    async fn metadata(&self, path: impl AsRef<Path>) -> io::Result<Metadata>;
}

pub struct FsOffloadDefault;

impl FsOffload for FsOffloadDefault {
    async fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        read(path).await
    }

    async fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        write(path, content).await
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
}
