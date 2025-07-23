use std::{fmt, fs::Metadata, io, path::Path};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::fs::fs::OpenOptions;

pub struct File {
    max_buf_size: usize,
    // todo
    opfs: (),
}

#[derive(Debug)]
enum Operation {
    Read(io::Result<usize>),
    Write(io::Result<()>),
    Seek(io::Result<u64>),
}

impl File {
    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        todo!()
    }

    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        todo!()
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        todo!()
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}

impl AsyncRead for File {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        todo!()
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        todo!()
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        todo!()
    }
}

impl fmt::Debug for File {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("tokio_fs_ext::wasm::fs::File").finish()
    }
}
