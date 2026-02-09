use std::{
    borrow::Cow,
    io,
    path::{Component, Path, PathBuf},
};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileSystemDirectoryHandle, FileSystemGetDirectoryOptions};

use crate::fs::wasm::current_dir::current_dir;

use super::{
    dir_handle_cache::{get_cached_dir_handle, set_cached_dir_handle},
    opfs_err,
    options::OpenDirType,
    root::root,
    virtualize,
};

#[cfg_attr(feature = "opfs_tracing", tracing::instrument(level = "trace", fields(path = %path.as_ref().to_string_lossy())))]
pub(crate) async fn open_dir(
    path: impl AsRef<Path>,
    r#type: OpenDirType,
) -> io::Result<FileSystemDirectoryHandle> {
    let virt = virtualize::virtualize(path)?;

    if let Some(handle) = get_cached_dir_handle(&virt) {
        return Ok(handle);
    }

    let components: Vec<Cow<'_, str>> = virt
        .components()
        .filter_map(|c| match c {
            Component::Normal(c) => Some(c.to_string_lossy()),
            _ => None,
        })
        .collect();

    let total_depth = components.len();

    let mut dir_handle = root().await?;

    let mut cur_virt = PathBuf::from("/");
    for (i, c) in components.iter().enumerate() {
        cur_virt = cur_virt.join(c.as_ref());
        dir_handle = if let Some(handle) = get_cached_dir_handle(&cur_virt) {
            handle
        } else {
            let is_last = i == total_depth - 1;
            let create = match r#type {
                OpenDirType::Create => is_last,
                OpenDirType::CreateRecursive => true,
                _ => {
                    // CWD needs to be checked when performing fs operations under cwd
                    if let Ok(cwd) = current_dir() {
                        cwd.starts_with(&cur_virt)
                    } else {
                        false
                    }
                }
            };

            let dir_handle = get_dir_handle(&dir_handle, c, create).await?;

            set_cached_dir_handle(cur_virt.clone(), dir_handle.clone());
            dir_handle
        };
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

    let dir_handle = JsFuture::from(parent.get_directory_handle_with_options(path, &options))
        .await
        .map_err(opfs_err)?
        .unchecked_into::<FileSystemDirectoryHandle>();
    Ok(dir_handle)
}
