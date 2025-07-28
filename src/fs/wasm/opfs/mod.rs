use std::{
    io,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

use js_sys::{Function, Object, Promise, Reflect};
use send_wrapper::SendWrapper;
use tokio::sync::OnceCell;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    DedicatedWorkerGlobalScope, DomException, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemRemoveOptions,
    FileSystemSyncAccessHandle,
};

use crate::fs::File;

static FS_ROOT: OnceCell<SendWrapper<FileSystemDirectoryHandle>> = OnceCell::const_new();

async fn opfs_root() -> io::Result<FileSystemDirectoryHandle> {
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
                "NotAllowedError" | "NoModificationAllowedError" => {
                    io::Error::from(io::ErrorKind::PermissionDenied)
                }
                "TypeMismatchError" => io::Error::other("type mismatch"),
                msg => io::Error::other(msg),
            },
            Err(_) => io::Error::other(format!("{}", Object::from(opfs_err.js_err).to_string())),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAccessMode {
    Readonly = "read-only",
    Readwrite = "readwrite",
    ReadwriteUnsafe = "readwrite-unsafe",
}

// The file mode is still experimental:
// https://developer.mozilla.org/en-US/docs/Web/API/FileSystemFileHandle/createSyncAccessHandle#options
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateSyncAccessHandleOptions {
    mode: SyncAccessMode,
}

impl From<SyncAccessMode> for CreateSyncAccessHandleOptions {
    fn from(mode: SyncAccessMode) -> Self {
        Self { mode }
    }
}

pub(super) async fn open_file(
    path: impl AsRef<Path>,
    create: bool,
    truncate: bool,
    mode: SyncAccessMode,
) -> io::Result<File> {
    let virt = virtualize(&path)?;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry: FileSystemDirectoryHandle = match parent {
        Some(parent_path) => {
            if parent_path.to_string_lossy().is_empty() {
                opfs_root().await?
            } else {
                open_dir(parent_path, OpenDirType::NotCreate).await?
            }
        }
        None => opfs_root().await?,
    };

    let option = FileSystemGetFileOptions::new();
    option.set_create(create);
    let file_handle = JsFuture::from(dir_entry.get_file_handle_with_options(&name, &option))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<FileSystemFileHandle>()
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    let file_handle_js_value = JsValue::from(file_handle);

    let promise = Reflect::get(&file_handle_js_value, &"createSyncAccessHandle".into())
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<Function>()
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .call1(
            &file_handle_js_value,
            &CreateSyncAccessHandleOptions::from(mode).into(),
        )
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<Promise>()
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    let sync_access_handle = JsFuture::from(promise)
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<FileSystemSyncAccessHandle>()
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    if truncate {
        sync_access_handle
            .truncate_with_u32(0)
            .map_err(|err| OpfsError::from(err).into_io_err())?;
    }

    Ok(File {
        sync_access_handle,
        pos: Mutex::new(0),
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum OpenDirType {
    Create,
    CreateRecursive,
    NotCreate,
}

pub(crate) async fn open_dir(
    path: impl AsRef<Path>,
    r#type: OpenDirType,
) -> io::Result<FileSystemDirectoryHandle> {
    let virt = virtualize(path)?;

    let components = virt
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();

    let total_depth = components.len();

    if total_depth == 0 {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }

    let root = opfs_root().await?;

    if total_depth == 1 {
        return get_dir_handle(
            &root,
            &components[0],
            matches!(r#type, OpenDirType::Create | OpenDirType::CreateRecursive),
        )
        .await;
    }

    let mut dir_handle = get_dir_handle(
        &root,
        &components[0],
        matches!(r#type, OpenDirType::CreateRecursive),
    )
    .await?;

    let mut depth = 1_usize;

    for c in components.iter().skip(1) {
        dir_handle = get_dir_handle(
            &dir_handle,
            c,
            matches!(r#type, OpenDirType::Create | OpenDirType::CreateRecursive),
        )
        .await?;
        depth += 1;
    }

    if depth != total_depth {
        return Err(io::Error::from(io::ErrorKind::NotFound));
    }

    Ok(dir_handle)
}

async fn get_dir_handle(
    parent: &FileSystemDirectoryHandle,
    path: &str,
    create: bool,
) -> io::Result<FileSystemDirectoryHandle> {
    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(create);

    JsFuture::from(parent.get_directory_handle_with_options(path, &options))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .dyn_into::<FileSystemDirectoryHandle>()
        .map_err(|err| OpfsError::from(err).into_io_err())
}

pub(crate) async fn rm(path: impl AsRef<Path>, recursive: bool) -> io::Result<()> {
    let virt = virtualize(path)?;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry: FileSystemDirectoryHandle = match parent {
        Some(path) => {
            if path.to_string_lossy().is_empty() {
                opfs_root().await?
            } else {
                open_dir(path, OpenDirType::NotCreate).await?
            }
        }
        None => opfs_root().await?,
    };

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(recursive);

    JsFuture::from(dir_entry.remove_entry_with_options(&name, &options))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?;

    Ok(())
}

pub(crate) fn virtualize(path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    // TODO: should handle symlink here
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
