//! Storage engine abstractions and built-in implementations.

use std::pin::Pin;

use bytes::Bytes;
use futures::Stream;

use crate::{MulterError, StorageError};

/// Disk-backed storage backend implementation.
pub mod disk;
/// In-memory storage backend implementation.
pub mod memory;
pub use disk::{DiskStorage, DiskStorageBuilder, FilenameStrategy};
pub use memory::MemoryStorage;

/// Boxed stream type used by storage backends.
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;

/// Metadata describing a file part before persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMeta {
    /// Multipart field name.
    pub field_name: String,
    /// Original filename from the multipart part, when present.
    pub file_name: Option<String>,
    /// Content type observed on the uploaded file part.
    pub content_type: String,
    /// Optional backend-specific size hint in bytes.
    pub size_hint: Option<u64>,
}

/// Metadata describing a stored file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    /// Backend-specific opaque key or location identifier.
    pub storage_key: String,
    /// Multipart field name.
    pub field_name: String,
    /// Original filename from the multipart part, when present.
    pub file_name: Option<String>,
    /// Content type observed on the uploaded file part.
    pub content_type: mime::Mime,
    /// Persisted file size in bytes.
    pub size: u64,
    /// Final filesystem path when stored on disk.
    pub path: Option<std::path::PathBuf>,
}

/// Async trait abstraction for file storage backends.
#[async_trait::async_trait(?Send)]
pub trait StorageEngine: Send + Sync + std::fmt::Debug + 'static {
    /// Backend-specific output type returned after a successful store.
    type Output: Send;
    /// Backend-specific error type surfaced on store failure.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Stores a file stream and returns backend output metadata.
    async fn store(
        &self,
        field_name: &str,
        file_name: Option<&str>,
        content_type: &str,
        stream: BoxStream<'_, Result<Bytes, MulterError>>,
    ) -> Result<Self::Output, Self::Error>;
}

/// Placeholder storage implementation used as the default backend.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopStorage;

#[async_trait::async_trait(?Send)]
impl StorageEngine for NoopStorage {
    type Output = StoredFile;
    type Error = StorageError;

    async fn store(
        &self,
        _field_name: &str,
        _file_name: Option<&str>,
        _content_type: &str,
        _stream: BoxStream<'_, Result<Bytes, MulterError>>,
    ) -> Result<Self::Output, Self::Error> {
        Err(StorageError::new(
            "no storage backend configured; choose a concrete storage engine",
        ))
    }
}

