#![cfg(all(target_family = "wasm", target_os = "unknown"))]

#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test_configure;

#[cfg(test)]
wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg(test)]
use wasm_bindgen_test::*;

#[cfg(test)]
use tokio_fs_ext::fs::*;

use std::io;

#[wasm_bindgen_test]
async fn test_dir() {
    let _ = remove_dir_all("/1").await;

    assert!(!try_exists("/1").await.unwrap());

    create_dir("/1").await.unwrap();

    assert!(try_exists("/1").await.unwrap());

    let _ = remove_dir_all("/1").await;

    assert!(!try_exists("/1/2").await.unwrap());

    assert!(!try_exists("/1").await.unwrap());

    create_dir_all("/1/2/3").await.unwrap();
    assert!(try_exists("/1/2/3").await.unwrap());

    write("/1/2/f_0", "f_0").await.unwrap();
    let mut rd = read_dir("1/2").await.unwrap();

    let mut entries = vec![
        {
            let e = rd.next_entry().await.unwrap().unwrap();
            (
                e.file_type().is_dir(),
                e.file_name().to_string_lossy().to_string(),
            )
        },
        {
            let e = rd.next_entry().await.unwrap().unwrap();
            (
                e.file_type().is_dir(),
                e.file_name().to_string_lossy().to_string(),
            )
        },
    ];

    entries.sort_by_key(|e| e.0);

    assert_eq!(
        entries,
        vec![(false, "f_0".to_string()), (true, "3".to_string())]
    );

    assert!(rd.next_entry().await.unwrap().is_none());

    assert!(!try_exists("/1/2/3/x").await.unwrap());
}

#[wasm_bindgen_test]
async fn test_file() {
    let path = "/1/2/hello";
    let data = "world";

    create_dir_all("/1/2").await.unwrap();

    let _ = remove_file(path).await;

    assert!(!try_exists(path).await.unwrap());

    write(path, data.as_bytes()).await.unwrap();

    assert!(try_exists(path).await.unwrap());

    assert_eq!(read(path).await.unwrap(), data.as_bytes());

    let copy_path = &format!("{path}_copy");
    copy(path, copy_path).await.unwrap();
    assert_eq!(read(copy_path).await.unwrap(), data.as_bytes());

    let rename_path = &format!("{path}_rename");
    rename(path, rename_path).await.unwrap();
    assert_eq!(read(rename_path).await.unwrap(), data.as_bytes());

    assert_eq!(read_to_string(rename_path).await.unwrap(), data);

    remove_file(rename_path).await.unwrap();

    assert!(!try_exists(rename_path).await.unwrap());
}

#[wasm_bindgen_test]
async fn test_open_options() {
    // TODO:
    let should_not_create = "should_not_create";
    let _ = remove_file(should_not_create).await;
    assert_eq!(
        OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .truncate(true)
            .create_new(true)
            .open(should_not_create)
            .await
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );

    // TODO:
    let should_create = "should_create";
    let _ = remove_file(should_create).await;
    assert!(
        OpenOptions::new()
            .create(true)
            .open(should_create)
            .await
            .is_ok()
    );

    // TODO:
    let _readonly = OpenOptions::new()
        .read(true)
        .create(true)
        .open("reaonly")
        .await
        .unwrap();

    // TODO:
    let _writeonly = OpenOptions::new()
        .read(true)
        .create(true)
        .open("writeonly")
        .await
        .unwrap();

    // TODO:
    let _truncate = OpenOptions::new()
        .read(true)
        .create(true)
        .open("truncate")
        .await
        .unwrap();

    // TODO:
    let _append = OpenOptions::new()
        .read(true)
        .create(true)
        .open("append")
        .await
        .unwrap();
}

#[wasm_bindgen_test]
async fn test_metadata() {
    let meta_dir = "meta_dir";
    let meta_file = &format!("{meta_dir}/meta_file");

    assert_eq!(
        metadata("notfound").await.unwrap_err().kind(),
        io::ErrorKind::NotFound
    );

    create_dir(meta_dir).await.unwrap();

    assert!(metadata(meta_dir).await.unwrap().is_dir());

    write(meta_file, meta_file).await.unwrap();

    let f_metadata = metadata(meta_file).await.unwrap();

    assert!(f_metadata.is_file());

    assert!(f_metadata.len() == meta_file.len() as u64)
}
