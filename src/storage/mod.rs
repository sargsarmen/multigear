/// Async trait abstraction for file storage backends.
#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync + std::fmt::Debug {}

/// Placeholder storage implementation used during bootstrap.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopStorage;

#[async_trait::async_trait]
impl StorageEngine for NoopStorage {}
