use std::{future::Future, path::PathBuf};
use tokio_fs_ext::{create_dir_all, remove_dir_all};

// Mutex to protect global resources like CWD.
#[cfg(not(target_family = "wasm"))]
pub static CWD_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn run_test<F, Fut>(name: &str, test_fn: F)
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: Future<Output = ()>,
{
    let base_dir = if cfg!(target_family = "wasm") {
        PathBuf::from(format!("/test_{}", name))
    } else {
        static CWD: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        let cwd = CWD.get_or_init(|| std::env::current_dir().unwrap());
        cwd.join("target").join("tokio_fs_ext_test").join(name)
    };

    // Cleanup before test
    let _ = remove_dir_all(&base_dir).await;
    create_dir_all(&base_dir)
        .await
        .expect("failed to create test dir");

    test_fn(base_dir.clone()).await;

    // Cleanup after test
    let _ = remove_dir_all(&base_dir).await;
}
