use tokio::sync::mpsc;

use super::{FsOffload, FsTask};

pub struct Server {
    pub(super) receiver: mpsc::Receiver<FsTask>,
}

impl Server {
    pub async fn serve(&mut self, offload: impl FsOffload) {
        while let Some(task) = self.receiver.recv().await {
            task.execute(&offload).await;
        }
    }
}
