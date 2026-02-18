#![allow(missing_docs)]

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use futures::stream;
use rust_multer::{Multer, MulterError, Part, StorageEngine, StorageError, StoredFile};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
struct HashMapStorage {
    files: Arc<RwLock<HashMap<String, Bytes>>>,
}

#[async_trait::async_trait]
impl StorageEngine for HashMapStorage {
    async fn store(&self, mut part: Part) -> Result<StoredFile, StorageError> {
        let key = format!("{}-{}", part.field_name(), self.files.read().await.len());
        let content = part
            .bytes()
            .await
            .map_err(|err| StorageError::new(err.to_string()))?;
        let size = content.len() as u64;
        let field_name = part.field_name().to_owned();
        let file_name = part.file_name().map(ToOwned::to_owned);
        let content_type = part.content_type().clone();

        self.files.write().await.insert(key.clone(), content);
        Ok(StoredFile {
            storage_key: key,
            field_name,
            file_name,
            content_type,
            size,
            path: None,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let storage = HashMapStorage::default();
    let multer = Multer::new(storage.clone());

    let body = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"upload\"; filename=\"a.txt\"\r\n",
        "Content-Type: text/plain\r\n",
        "\r\n",
        "hello world\r\n",
        "--BOUND--\r\n"
    );
    let output = multer
        .parse_and_store(
            "BOUND",
            stream::iter([Ok::<Bytes, MulterError>(Bytes::from_static(body.as_bytes()))]),
        )
        .await
        .expect("pipeline should succeed");

    println!("stored files: {}", output.stored_files.len());
}
