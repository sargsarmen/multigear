use thiserror::Error;

/// Error type used by `rust-multer`.
#[derive(Debug, Error)]
pub enum MulterError {
    /// Placeholder variant used during early bootstrap phases.
    #[error("not yet implemented")]
    NotYetImplemented,
}
