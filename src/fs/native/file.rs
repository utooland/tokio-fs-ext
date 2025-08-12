use std::{
    fs::Metadata,
    io,
    path::Path,
    pin::Pin,
    task::{Context, Poll, ready},
};

use pin_project_lite::pin_project;
use tokio::{fs::OpenOptions, io::AsyncSeek};

pin_project! {
    #[derive(Debug)]
    pub struct File {
        #[pin]
        pub(crate) inner: tokio::fs::File,
        pub(crate) seek_pos: Option<io::SeekFrom>,
    }
}

impl File {
    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        Ok(File {
            inner: tokio::fs::File::create(path).await?,
            seek_pos: None,
        })
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        Ok(File {
            inner: tokio::fs::File::create_new(path).await?,
            seek_pos: None,
        })
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        self.inner.metadata().await
    }

    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        Ok(File {
            inner: tokio::fs::File::open(path).await?,
            seek_pos: None,
        })
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub async fn sync_all(&self) -> io::Result<()> {
        self.inner.sync_all().await
    }

    pub async fn sync_data(&self) -> io::Result<()> {
        self.inner.sync_data().await
    }
}

impl futures::io::AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut buf = tokio::io::ReadBuf::new(buf);
        ready!(tokio::io::AsyncRead::poll_read(
            self.project().inner,
            cx,
            &mut buf
        ))?;
        Poll::Ready(Ok(buf.filled().len()))
    }
}

impl futures::io::AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        tokio::io::AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        tokio::io::AsyncWrite::poll_shutdown(self.project().inner, cx)
    }
}

impl futures::io::AsyncSeek for File {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: io::SeekFrom,
    ) -> Poll<io::Result<u64>> {
        if self.seek_pos != Some(pos) {
            ready!(self.as_mut().project().inner.poll_complete(cx))?;
            self.as_mut().project().inner.start_seek(pos)?;
            *self.as_mut().project().seek_pos = Some(pos);
        }
        let res = ready!(self.as_mut().project().inner.poll_complete(cx));
        *self.as_mut().project().seek_pos = None;
        Poll::Ready(res)
    }
}
