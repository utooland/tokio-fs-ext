use std::{fs::Metadata, io, path::Path};

pub async fn metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
    todo!()
}
