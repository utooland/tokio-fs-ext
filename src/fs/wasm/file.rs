use std::{
    cell::RefCell,
    cmp,
    collections::VecDeque,
    fs::Metadata,
    io::{self, Read, SeekFrom, Write},
    path::Path,
    pin::Pin,
    task::{Context, Poll, ready},
};

use send_wrapper::SendWrapper;
use std::future::Future;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{Mutex, oneshot},
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemFileHandle, FileSystemGetFileOptions, FileSystemReadWriteOptions,
    FileSystemSyncAccessHandle,
};

use crate::fs::{
    OpenOptions,
    buf::{Buf, DEFAULT_MAX_BUF_SIZE},
    opfs::{OpfsError, fs_root},
};

#[derive(Debug)]
pub struct File {
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
    inner: Mutex<Inner>,
    max_buf_size: usize,
}

#[derive(Debug)]
struct Inner {
    state: State,
    last_write_err: Option<io::ErrorKind>,
    pos: u64,
}

#[derive(Debug)]
pub(super) struct JoinHandle<T> {
    rx: oneshot::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::task::Poll;

        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(Ok(v)),
            Poll::Ready(Err(e)) => panic!("error = {e:?}"),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
enum Operation {
    Read(io::Result<usize>),
    Write(io::Result<()>),
    Seek(io::Result<u64>),
}

#[derive(Debug)]
enum State {
    Idle(Option<Buf>),
    Busy(JoinHandle<(Operation, Buf)>),
}

thread_local! {
    static QUEUE: RefCell<VecDeque<Box<dyn FnOnce() + Send>>> = RefCell::new(VecDeque::new())
}

pub(super) fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let task = Box::new(move || {
        let _ = tx.send(f());
    });

    QUEUE.with(|cell| cell.borrow_mut().push_back(task));

    JoinHandle { rx }
}

pub(super) fn spawn_mandatory_blocking<F, R>(f: F) -> Option<JoinHandle<R>>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let task = Box::new(move || {
        let _ = tx.send(f());
    });

    QUEUE.with(|cell| cell.borrow_mut().push_back(task));

    Some(JoinHandle { rx })
}

impl File {
    pub async fn open(path: impl AsRef<Path>) -> io::Result<File> {
        open_file(path, false, false).await
    }

    pub async fn create(path: impl AsRef<Path>) -> io::Result<File> {
        let mut open_options = OpenOptions::new();
        open_options.create(true);
        open_file(path, true, true).await
    }

    pub async fn create_new<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        if (open_file(&path, true, false).await).is_ok() {
            return io::Result::Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        File::create(path).await
    }

    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        todo!()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        self.sync_access_handle
            .flush()
            .expect("Failed to flush opfs sync access handle");
        self.sync_access_handle.close();
    }
}

pub(super) async fn open_file(
    path: impl AsRef<Path>,
    create: bool,
    truncate: bool,
) -> io::Result<File> {
    let name = path.as_ref().to_string_lossy();
    let root = fs_root().await?;
    let option = FileSystemGetFileOptions::new();
    option.set_create(create);
    let file_handle = JsFuture::from(root.get_file_handle_with_options(&name, &option))
        .await
        .map_err(|err| io::Error::from(OpfsError::from(err)))?
        .dyn_into::<FileSystemFileHandle>()
        .map_err(|err| io::Error::from(OpfsError::from(err)))?;
    let sync_access_handle = JsFuture::from(file_handle.create_sync_access_handle())
        .await
        .map_err(|err| io::Error::from(OpfsError::from(err)))?
        .dyn_into::<FileSystemSyncAccessHandle>()
        .map_err(|err| io::Error::from(OpfsError::from(err)))?;

    if truncate {
        sync_access_handle
            .truncate_with_u32(0)
            .map_err(|err| io::Error::from(OpfsError::from(err)))?;
    }

    Ok(File {
        sync_access_handle,
        inner: Mutex::new(Inner {
            state: State::Idle(Some(Buf::with_capacity(0))),
            last_write_err: None,
            pos: 0,
        }),
        max_buf_size: DEFAULT_MAX_BUF_SIZE,
    })
}

impl AsyncRead for File {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        dst: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let sync_access_handle = self.sync_access_handle.clone();
        let me = self.get_mut();
        let inner = me.inner.get_mut();
        loop {
            match inner.state {
                State::Idle(ref mut buf_cell) => {
                    let mut buf = buf_cell.take().unwrap();

                    if !buf.is_empty() || dst.remaining() == 0 {
                        buf.copy_to(dst);
                        *buf_cell = Some(buf);
                        return Poll::Ready(Ok(()));
                    }

                    let max_buf_size = cmp::min(dst.remaining(), me.max_buf_size);

                    struct OpfsFileReader {
                        options: SendWrapper<FileSystemReadWriteOptions>,
                        sync_access_handle: SendWrapper<FileSystemSyncAccessHandle>,
                    }
                    impl Read for OpfsFileReader {
                        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                            self.sync_access_handle
                                .read_with_u8_array_and_options(buf, &self.options)
                                .map_or_else(
                                    |err| io::Result::Err(io::Error::from(OpfsError::from(err))),
                                    |size| io::Result::Ok(size as usize),
                                )
                        }
                    }

                    let options = FileSystemReadWriteOptions::new();
                    options.set_at(inner.pos as f64);
                    let mut opfs_file_reader = OpfsFileReader {
                        options: SendWrapper::new(options),
                        sync_access_handle: SendWrapper::new(sync_access_handle.clone()),
                    };
                    inner.state = State::Busy(spawn_blocking(move || {
                        let res = unsafe { buf.read_from(&mut opfs_file_reader, max_buf_size) };
                        (Operation::Read(res), buf)
                    }));
                }
                State::Busy(ref mut rx) => {
                    let (op, mut buf) = ready!(Pin::new(rx).poll(cx))?;
                    match op {
                        Operation::Read(Ok(_)) => {
                            buf.copy_to(dst);
                            inner.state = State::Idle(Some(buf));
                            return Poll::Ready(Ok(()));
                        }
                        Operation::Read(Err(e)) => {
                            assert!(buf.is_empty());
                            let kind = e.kind();
                            inner.state = State::Idle(Some(buf));
                            return Poll::Ready(io::Result::Err(io::Error::from(kind)));
                        }
                        Operation::Write(Ok(())) => {
                            assert!(buf.is_empty());
                            inner.state = State::Idle(Some(buf));
                            continue;
                        }
                        Operation::Write(Err(e)) => {
                            assert!(inner.last_write_err.is_none());
                            inner.last_write_err = Some(e.kind());
                            inner.state = State::Idle(Some(buf));
                        }
                        Operation::Seek(result) => {
                            assert!(buf.is_empty());
                            if let Ok(pos) = result {
                                inner.pos = pos;
                            }
                            inner.state = State::Idle(Some(buf));
                            continue;
                        }
                    }
                }
            }
        }
    }
}

impl AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        src: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let sync_access_handle = self.sync_access_handle.clone();
        let me = self.get_mut();
        let inner = me.inner.get_mut();
        if let Some(e) = inner.last_write_err.take() {
            return Poll::Ready(Err(e.into()));
        }

        loop {
            match inner.state {
                State::Idle(ref mut buf_cell) => {
                    let mut buf = buf_cell.take().unwrap();

                    let seek = if !buf.is_empty() {
                        Some(SeekFrom::Current(buf.discard_read()))
                    } else {
                        None
                    };

                    let n = buf.copy_from(src, me.max_buf_size);

                    struct OpfsFileWriter {
                        options: SendWrapper<FileSystemReadWriteOptions>,
                        sync_access_handle: SendWrapper<FileSystemSyncAccessHandle>,
                    }
                    impl Write for OpfsFileWriter {
                        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                            self.sync_access_handle
                                .write_with_u8_array_and_options(buf, &self.options)
                                .map_or_else(
                                    |err| io::Result::Err(io::Error::from(OpfsError::from(err))),
                                    |size| io::Result::Ok(size as usize),
                                )
                        }

                        fn flush(&mut self) -> io::Result<()> {
                            todo!()
                        }
                    }

                    let options = FileSystemReadWriteOptions::new();
                    options.set_at(inner.pos as f64);
                    let mut opfs_file_writer = OpfsFileWriter {
                        options: SendWrapper::new(options),
                        sync_access_handle: SendWrapper::new(sync_access_handle.clone()),
                    };

                    let blocking_task_join_handle = spawn_mandatory_blocking(move || {
                        let res = buf.write_to(&mut opfs_file_writer);
                        (Operation::Write(res), buf)
                    })
                    .ok_or_else(|| io::Error::other("background task failed"))?;

                    inner.state = State::Busy(blocking_task_join_handle);

                    return Poll::Ready(Ok(n));
                }
                State::Busy(ref mut rx) => {
                    let (op, buf) = ready!(Pin::new(rx).poll(cx))?;
                    inner.state = State::Idle(Some(buf));

                    match op {
                        Operation::Read(_) => {
                            // We don't care about the result here. The fact
                            // that the cursor has advanced will be reflected in
                            // the next iteration of the loop
                            continue;
                        }
                        Operation::Write(res) => {
                            // If the previous write was successful, continue.
                            // Otherwise, error.
                            res?;
                            continue;
                        }
                        Operation::Seek(_) => {
                            // Ignore the seek
                            continue;
                        }
                    }
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}
