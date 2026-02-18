use crate::config::MulterConfig;

/// Builder for configuring a `Multer` instance.
#[derive(Debug, Clone, Default)]
pub struct MulterBuilder {
    config: MulterConfig,
}

impl MulterBuilder {
    /// Creates a builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current builder configuration snapshot.
    pub fn config(&self) -> &MulterConfig {
        &self.config
    }
}
