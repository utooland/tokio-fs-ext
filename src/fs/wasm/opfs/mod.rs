use std::{
    io,
    path::{Component, Path, PathBuf},
};

use js_sys::Object;
use send_wrapper::SendWrapper;
use tokio::sync::OnceCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    DedicatedWorkerGlobalScope, DomException, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemRemoveOptions,
    FileSystemSyncAccessHandle,
};

use crate::fs::File;

static FS_ROOT: OnceCell<SendWrapper<FileSystemDirectoryHandle>> = OnceCell::const_new();

pub async fn fs_root() -> io::Result<FileSystemDirectoryHandle> {
    let root = FS_ROOT
        .get_or_try_init(|| async {
            io::Result::Ok(SendWrapper::new(
                JsFuture::from(
                    DedicatedWorkerGlobalScope::from(JsValue::from(js_sys::global()))
                        .navigator()
                        .storage()
                        .get_directory(),
                )
                .await
                .map_err(|err| OpfsError::from(err).into_io_err())?
                .dyn_into::<FileSystemDirectoryHandle>()
                .map_err(|err| OpfsError::from(err).into_io_err())?,
            ))
        })
        .await?;

    Ok(root.clone().take())
}

pub struct OpfsError {
    js_err: JsValue,
}

impl OpfsError {
    pub(crate) fn into_io_err(self) -> io::Error {
        self.into()
    }
}

impl From<JsValue> for OpfsError {
    fn from(js_err: JsValue) -> Self {
        Self { js_err }
    }
}

impl From<OpfsError> for io::Error {
    fn from(opfs_err: OpfsError) -> Self {
        match opfs_err.js_err.clone().dyn_into::<DomException>() {
            Ok(e) => match e.name().as_str() {
                "NotFoundError" => io::Error::from(io::ErrorKind::NotFound),
                "NotAllowedError" => io::Error::from(io::ErrorKind::PermissionDenied),
                "TypeMismatchError" => io::Error::other("type mismatch"),
                msg => io::Error::other(msg),
            },
            Err(_) => io::Error::other(format!("{}", Object::from(opfs_err.js_err).to_string())),
        }
    }
}

pub(super) async fn open_file(
    path: impl AsRef<Path>,
    create: bool,
    truncate_all: bool,
) -> io::Result<File> {
    let virt = virtualize(path)?;

    let name = virt.to_string_lossy();

    let root = fs_root().await?;
    let option = FileSystemGetFileOptions::new();
    option.set_create(create);
    let file_handle = JsFuture::from(root.get_file_handle_with_options(&name, &option))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<FileSystemFileHandle>()
        .map_err(|err| OpfsError::from(err).into_io_err())?;
    let sync_access_handle = JsFuture::from(file_handle.create_sync_access_handle())
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<FileSystemSyncAccessHandle>()
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    if truncate_all {
        sync_access_handle
            .truncate_with_u32(0)
            .map_err(|err| OpfsError::from(err).into_io_err())?;
    }

    Ok(File { sync_access_handle })
}

pub(crate) async fn open_dir(
    path: impl AsRef<Path>,
    create: bool,
    recursive: bool,
) -> io::Result<FileSystemDirectoryHandle> {
    let virt = virtualize(path)?;

    let components = virt
        .components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect::<Vec<_>>();

    if components.is_empty() || (!recursive && components.len() > 1) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }

    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(create);

    let root = fs_root().await?;

    let mut dir_handle =
        JsFuture::from(root.get_directory_handle_with_options(components[0], &options))
            .await
            .map_err(|err| OpfsError::from(err).into_io_err())?
            .dyn_into::<FileSystemDirectoryHandle>()
            .map_err(|err| OpfsError::from(err).into_io_err())?;

    let mut depth = 1_usize;

    for c in components.iter().skip(1) {
        dir_handle = JsFuture::from(dir_handle.get_directory_handle_with_options(c, &options))
            .await
            .map_err(|err| OpfsError::from(err).into_io_err())?
            .dyn_into::<FileSystemDirectoryHandle>()
            .map_err(|err| OpfsError::from(err).into_io_err())?;
        depth += 1;
    }

    if depth != components.len() {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }

    Ok(dir_handle)
}

pub(crate) async fn rm_dir(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    let virt = virtualize(path)?;

    let name = virt.to_string_lossy();

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(recursive);

    let root = fs_root().await?;
    JsFuture::from(root.remove_entry_with_options(&name, &options))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}

pub(crate) fn virtualize(path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    let mut out = Vec::new();

    for comp in path.as_ref().components() {
        match comp {
            Component::CurDir => (),
            Component::ParentDir => match out.last() {
                Some(Component::RootDir) => (),
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                None
                | Some(Component::CurDir)
                | Some(Component::ParentDir)
                | Some(Component::Prefix(_)) => out.push(comp),
            },
            comp => out.push(comp),
        }
    }

    if !out.is_empty() {
        Ok(out
            .iter()
            .filter(|c| !matches!(c, Component::RootDir))
            .collect())
    } else {
        Ok(PathBuf::from("."))
    }
}
