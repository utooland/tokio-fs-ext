use std::{io, path::PathBuf};

use tokio::sync::oneshot;

#[cfg(feature = "opfs_watch")]
use super::super::opfs::watch::event;
use super::{FsOffload, Metadata, ReadDir};

#[cfg(feature = "opfs_watch")]
use tokio::sync::mpsc as watch_mpsc;

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
    Copy {
        from: PathBuf,
        to: PathBuf,
        sender: oneshot::Sender<io::Result<u64>>,
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
    #[cfg(feature = "opfs_watch")]
    WatchDir {
        path: PathBuf,
        recursive: bool,
        cb: Box<dyn Fn(event::Event) + Send + Sync + 'static>,
        /// Sender to return the stop signal sender to client
        sender: oneshot::Sender<io::Result<watch_mpsc::Sender<()>>>,
    },
}

macro_rules! impl_fs_task_execute {
    (
        $offload_trait:ident,
        $task_enum:ident,
        [ $( $(#[$attr:meta])* ($variant:ident, $method:ident, ( $( $arg:ident : $arg_type:ty ),* ) ) ),* ]
    ) => {
        impl $task_enum {
            pub(super) async fn execute(self, offload: &impl $offload_trait) {
                match self {
                    $(
                        $(#[$attr])*
                        $task_enum::$variant { $( $arg, )* sender } => {
                            let _ = sender.send(offload.$method( $( $arg ),* ).await);
                        }
                    )*
                    #[cfg(feature = "opfs_watch")]
                    $task_enum::WatchDir { .. } => {
                        // WatchDir is handled separately in server.rs
                        unreachable!("WatchDir should be handled separately")
                    }
                }
            }
        }
    };
}

impl_fs_task_execute!(
    FsOffload,
    FsTask,
    [
        (Read, read, (path: PathBuf)),
        (Write, write, (path: PathBuf, content: Vec<u8>)),
        (Copy, copy, (from: PathBuf, to: PathBuf)),
        (ReadDir, read_dir, (path: PathBuf)),
        (CreateDir, create_dir, (path: PathBuf)),
        (CreateDirAll, create_dir_all, (path: PathBuf)),
        (RemoveFile, remove_file, (path: PathBuf)),
        (RemoveDir, remove_dir, (path: PathBuf)),
        (RemoveDirAll, remove_dir_all, (path: PathBuf)),
        (Metadata, metadata, (path: PathBuf))
    ]
);

#[cfg(feature = "opfs_watch")]
impl FsTask {
    /// Execute WatchDir task separately since it needs special handling
    pub(super) async fn execute_watch(
        path: std::path::PathBuf,
        recursive: bool,
        cb: Box<dyn Fn(event::Event) + Send + Sync + 'static>,
        sender: oneshot::Sender<io::Result<watch_mpsc::Sender<()>>>,
        offload: &impl FsOffload,
    ) {
        match offload.watch_dir(path, recursive, cb).await {
            Ok(handle) => {
                // Create a channel for stop signal
                let (stop_tx, mut stop_rx) = watch_mpsc::channel::<()>(1);
                
                // Send the stop sender back to client
                let _ = sender.send(Ok(stop_tx));
                
                // Wait for stop signal, then drop the handle
                let _ = stop_rx.recv().await;
                handle.stop();
            }
            Err(e) => {
                let _ = sender.send(Err(e));
            }
        }
    }
}
