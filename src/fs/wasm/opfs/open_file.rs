use std::{io, path::Path};

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetFileOptions,
    FileSystemSyncAccessHandle,
};

use super::{
    super::File,
    OpenDirType,
    error::OpfsError,
    open_dir,
    options::{CreateFileMode, CreateSyncAccessHandleOptions, SyncAccessMode},
    root::root,
    virtualize,
};

#[tracing::instrument(level = "trace", fields(path = %path.as_ref().to_string_lossy()))]
pub(crate) async fn open_file(
    path: impl AsRef<Path>,
    create: CreateFileMode,
    mode: SyncAccessMode,
    truncate: bool,
) -> io::Result<File> {
    let virt = virtualize::virtualize(&path)?;

    let parent = virt.parent();

    let name = match virt.file_name() {
        Some(os_str) => Ok(os_str.to_string_lossy()),
        None => Err(io::Error::from(io::ErrorKind::InvalidFilename)),
    }?;

    let dir_entry = match parent {
        Some(parent_path) => open_dir(parent_path, OpenDirType::NotCreate).await?,
        None => root().await?,
    };

    let sync_access_handle = match create {
        CreateFileMode::Create => get_file_handle(&name, &dir_entry, mode, true, truncate).await?,
        CreateFileMode::CreateNew => {
            match get_file_handle(&name, &dir_entry, mode, false, truncate).await {
                Ok(_) => {
                    return Err(io::Error::from(io::ErrorKind::AlreadyExists));
                }
                Err(_) => get_file_handle(&name, &dir_entry, mode, true, truncate).await?,
            }
        }
        CreateFileMode::NotCreate => {
            get_file_handle(&name, &dir_entry, mode, false, truncate).await?
        }
    };
    Ok(File {
        sync_access_handle,
        pos: None,
    })
}

async fn get_file_handle(
    name: &str,
    dir_entry: &FileSystemDirectoryHandle,
    mode: SyncAccessMode,
    create: bool,
    truncate: bool,
) -> Result<FileSystemSyncAccessHandle, io::Error> {
    let option = FileSystemGetFileOptions::new();
    option.set_create(create);
    let file_handle = JsFuture::from(dir_entry.get_file_handle_with_options(name, &option))
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<FileSystemFileHandle>();

    let file_handle_js_value = JsValue::from(file_handle);

    let promise = Reflect::get(&file_handle_js_value, &"createSyncAccessHandle".into())
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Function>()
        .call1(
            &file_handle_js_value,
            &CreateSyncAccessHandleOptions::from(mode).into(),
        )
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<Promise>();

    let sync_access_handle = JsFuture::from(promise)
        .await
        .map_err(|err| OpfsError::from(err).into_io_err())?
        .unchecked_into::<FileSystemSyncAccessHandle>();

    if truncate {
        sync_access_handle
            .truncate_with_u32(0)
            .map_err(|err| OpfsError::from(err).into_io_err())?;
    }
    Ok(sync_access_handle)
}
