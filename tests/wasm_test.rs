#![cfg(all(target_family = "wasm", target_os = "unknown"))]

#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test_configure;

#[cfg(test)]
wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg(test)]
use wasm_bindgen_test::*;

#[cfg(test)]
use tokio_fs_ext::fs::*;

#[wasm_bindgen_test]
async fn test_dir() {
    let _ = remove_dir_all("1").await;

    assert!(!try_exists("1").await.unwrap());

    create_dir("1").await.unwrap();

    assert!(try_exists("1").await.unwrap());

    let _ = remove_dir_all("1/2").await;

    assert!(!try_exists("1/2").await.unwrap());

    create_dir_all("1/2").await.unwrap();

    assert!(try_exists("1/2").await.unwrap());
}

#[wasm_bindgen_test]
async fn test_file() {
    let path = "hello_0";
    let data = "world";

    let _ = remove_file(path).await;

    assert!(!try_exists(path).await.unwrap());

    write(path, data.as_bytes()).await.unwrap();

    assert!(try_exists(path).await.unwrap());

    assert_eq!(read(path).await.unwrap(), data.as_bytes());

    let copy_path = &format!("{path}_copy");
    copy(path, copy_path).await.unwrap();
    assert_eq!(read(copy_path).await.unwrap(), data.as_bytes());

    remove_file(path).await.unwrap();

    assert!(!try_exists(path).await.unwrap());
}

#[wasm_bindgen_test]
async fn test_open_options() {
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
        std::io::ErrorKind::NotFound
    );

    let should_create = "should_create";
    let _ = remove_file(should_create).await;
    assert!(
        OpenOptions::new()
            .create(true)
            .open(should_create)
            .await
            .is_ok()
    );

    let _readonly = OpenOptions::new()
        .read(true)
        .create(true)
        .open("reaonly")
        .await
        .unwrap();

    let _writeonly = OpenOptions::new()
        .read(true)
        .create(true)
        .open("writeonly")
        .await
        .unwrap();

    let _truncate = OpenOptions::new()
        .read(true)
        .create(true)
        .open("truncate")
        .await
        .unwrap();

    let _append = OpenOptions::new()
        .read(true)
        .create(true)
        .open("append")
        .await
        .unwrap();
}
