use std::pin::Pin;
use std::future::Future;

use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::mpsc;

use super::{FsOffload, FsOffloadDefault, FsTask};

pub struct Server {
    pub(super) receiver: mpsc::UnboundedReceiver<FsTask>,
}

impl Server {
    pub async fn serve(&mut self, offload: impl FsOffload) {
        let mut tasks: FuturesUnordered<Pin<Box<dyn Future<Output = ()>>>> = FuturesUnordered::new();
        let offload = &offload;

        loop {
            tokio::select! {
                res = self.receiver.recv() => {
                    match res {
                        Some(task) => {
                            #[cfg(feature = "opfs_watch")]
                            if let FsTask::WatchDir { path, recursive, cb, sender } = task {
                                // WatchDir needs special handling - keep WatchHandle on server side
                                tasks.push(Box::pin(async move {
                                    FsTask::execute_watch(path, recursive, cb, sender, offload).await;
                                }));
                                continue;
                            }
                            
                            tasks.push(Box::pin(async move {
                                task.execute(offload).await;
                            }));
                        }
                        None => {
                            while (tasks.next().await).is_some() {}
                            break;
                        }
                    }
                }
                Some(_) = tasks.next() => {}
            }
        }
    }

    pub async fn serve_default(&mut self) {
        self.serve(FsOffloadDefault).await
    }
}
