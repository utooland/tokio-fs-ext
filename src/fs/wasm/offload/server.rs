use tokio::sync::mpsc;

use super::{FsOffload, FsOffloadDefault, FsTask};

pub struct Server {
    pub(super) receiver: mpsc::UnboundedReceiver<FsTask>,
}

impl Server {
    pub async fn serve(&mut self, offload: impl FsOffload) {
        while let Some(task) = self.receiver.recv().await {
            task.execute(&offload).await;
        }
    }

    pub async fn serve_default(&mut self) {
        self.serve(FsOffloadDefault).await
    }
}
