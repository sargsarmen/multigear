#![allow(missing_docs)]

use bytes::Bytes;
use futures::{StreamExt, channel::mpsc};
use rust_multer::{Limits, MemoryStorage, Multer, MulterConfig, Selector};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = MulterConfig {
        selector: Selector::single("upload"),
        limits: Limits {
            max_file_size: Some(16 * 1024 * 1024),
            ..Limits::default()
        },
        ..MulterConfig::default()
    };
    let multer = Multer::with_config(MemoryStorage::new(), config).expect("valid config");

    let (tx, rx) = mpsc::unbounded();
    tx.unbounded_send(Ok(Bytes::from_static(
        b"--BOUND\r\nContent-Disposition: form-data; name=\"upload\"; filename=\"big.bin\"\r\n\r\n",
    )))
    .expect("send prelude");
    tx.unbounded_send(Ok(Bytes::from(vec![b'x'; 1024 * 64])))
        .expect("send chunk");
    tx.unbounded_send(Ok(Bytes::from_static(b"\r\n--BOUND--\r\n")))
        .expect("send trailer");
    drop(tx);

    let mut multipart = multer
        .multipart_from_boundary("BOUND", rx)
        .expect("multipart should initialize");

    while let Some(item) = multipart.next().await {
        let part = item.expect("part should parse");
        if part.file_name().is_some() {
            let stored = multer.store(part).await.expect("store should succeed");
            println!("stored {} bytes", stored.size);
        }
    }
}
