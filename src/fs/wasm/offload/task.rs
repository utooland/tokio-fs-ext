use std::{io, path::PathBuf};

use tokio::sync::oneshot;

use super::{FsOffload, Metadata, ReadDir};

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
}

macro_rules! impl_fs_task_execute {
    (
        $offload_trait:ident,
        $task_enum:ident,
        [ $( ($variant:ident, $method:ident, ( $( $arg:ident : $arg_type:ty ),* ) ) ),* ]
    ) => {
        impl $task_enum {
            pub(super) async fn execute(self, offload: &impl $offload_trait) {
                match self {
                    $(
                        $task_enum::$variant { $( $arg, )* sender } => {
                            let _ = sender.send(offload.$method( $( $arg ),* ).await);
                        }
                    )*
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
