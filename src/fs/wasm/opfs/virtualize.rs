use std::{
    io,
    path::{Component, MAIN_SEPARATOR_STR, Path, PathBuf},
};

use crate::fs::wasm::current_dir;

pub(crate) fn virtualize(path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    // TODO: should handle symlink here

    let path = if path.as_ref().starts_with(MAIN_SEPARATOR_STR) {
        path.as_ref().into()
    } else {
        current_dir()?.join(path)
    };

    let mut out = Vec::new();

    for comp in path.components() {
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
        Ok(out.iter().collect())
    } else {
        Ok(PathBuf::from("."))
    }
}
