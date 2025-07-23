use std::{fs::Metadata, io, path::Path};

use tokio::io::{AsyncRead, AsyncWrite};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemFileHandle, FileSystemGetFileOptions, FileSystemSyncAccessHandle};

use crate::fs::{
    OpenOptions,
    wasm::opfs::{fs_root, map_opfs_err},
};

#[derive(Debug)]
pub struct File {
    #[allow(dead_code)]
    pub(super) file_handle: FileSystemFileHandle,
    pub(super) sync_access_handle: FileSystemSyncAccessHandle,
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
        .map_err(map_opfs_err)?
        .dyn_into::<FileSystemFileHandle>()
        .map_err(map_opfs_err)?;
    let sync_access_handle = JsFuture::from(file_handle.create_sync_access_handle())
        .await
        .map_err(map_opfs_err)?
        .dyn_into::<FileSystemSyncAccessHandle>()
        .map_err(map_opfs_err)?;

    if truncate {
        sync_access_handle
            .truncate_with_u32(0)
            .map_err(map_opfs_err)?;
    }

    Ok(File {
        file_handle,
        sync_access_handle,
    })
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
