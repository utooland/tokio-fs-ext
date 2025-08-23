use tokio::sync::mpsc;

use super::{Client, FsOffload, FsTask};

pub struct Server {
    receiver: mpsc::Receiver<FsTask>,
}

impl Server {
    pub(super) fn new_pair() -> (Server, Client) {
        let (sender, receiver) = mpsc::channel(32);
        (Server { receiver }, Client { sender })
    }

    pub async fn bind(mut self, offload: impl FsOffload) {
        while let Some(task) = self.receiver.recv().await {
            match task {
                FsTask::Read { path, sender } => {
                    let _ = sender.send(offload.read(path).await);
                }
                FsTask::Write {
                    path,
                    content,
                    sender,
                } => {
                    let _ = sender.send(offload.write(path, content).await);
                }
                FsTask::ReadDir { path, sender } => {
                    let _ = sender.send(offload.read_dir(path).await);
                }
                FsTask::CreateDir { path, sender } => {
                    let _ = sender.send(offload.create_dir(path).await);
                }
                FsTask::CreateDirAll { path, sender } => {
                    let _ = sender.send(offload.create_dir_all(path).await);
                }
                FsTask::RemoveFile { path, sender } => {
                    let _ = sender.send(offload.remove_file(path).await);
                }
                FsTask::RemoveDir { path, sender } => {
                    let _ = sender.send(offload.remove_dir(path).await);
                }
                FsTask::RemoveDirAll { path, sender } => {
                    let _ = sender.send(offload.remove_dir_all(path).await);
                }
                FsTask::Metadata { path, sender } => {
                    let _ = sender.send(offload.metadata(path).await);
                }
            }
        }
    }
}
