use crate::{
    config::{MulterConfig, Selector, UnknownFieldPolicy},
    error::ConfigError,
    limits::Limits,
    storage::NoopStorage,
    Multer,
};

/// Builder for configuring a `Multer` instance.
#[derive(Debug, Clone)]
pub struct MulterBuilder<S = NoopStorage> {
    config: MulterConfig,
    storage: S,
}

impl Default for MulterBuilder<NoopStorage> {
    fn default() -> Self {
        Self {
            config: MulterConfig::default(),
            storage: NoopStorage,
        }
    }
}

impl MulterBuilder<NoopStorage> {
    /// Creates a builder with default configuration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> MulterBuilder<S> {
    /// Returns the current builder configuration snapshot.
    pub fn config(&self) -> &MulterConfig {
        &self.config
    }

    /// Replaces the storage backend used by the built `Multer`.
    pub fn storage<T>(self, storage: T) -> MulterBuilder<T> {
        MulterBuilder {
            config: self.config,
            storage,
        }
    }

    /// Replaces the full builder configuration.
    pub fn with_config(mut self, config: MulterConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the active file field selector strategy.
    pub fn selector(mut self, selector: Selector) -> Self {
        self.config.selector = selector;
        self
    }

    /// Selects exactly one file for a field.
    pub fn single(mut self, name: impl Into<String>) -> Self {
        self.config.selector = Selector::single(name);
        self
    }

    /// Selects multiple files for a field.
    pub fn array(mut self, name: impl Into<String>, max_count: usize) -> Self {
        self.config.selector = Selector::array(name, max_count);
        self
    }

    /// Selects multiple named fields.
    pub fn fields(
        mut self,
        fields: impl IntoIterator<Item = crate::config::SelectedField>,
    ) -> Self {
        self.config.selector = Selector::fields(fields);
        self
    }

    /// Rejects all file fields.
    pub fn none(mut self) -> Self {
        self.config.selector = Selector::none();
        self
    }

    /// Accepts any file field.
    pub fn any(mut self) -> Self {
        self.config.selector = Selector::any();
        self
    }

    /// Sets how unknown fields should be handled.
    pub fn unknown_field_policy(mut self, policy: UnknownFieldPolicy) -> Self {
        self.config.unknown_field_policy = policy;
        self
    }

    /// Sets global multipart limits.
    pub fn limits(mut self, limits: Limits) -> Self {
        self.config.limits = limits;
        self
    }

    /// Validates builder configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.config.validate()
    }

    /// Finalizes and returns validated configuration.
    pub fn build_config(self) -> Result<MulterConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Builds a fully configured `Multer` instance.
    pub fn build(self) -> Result<Multer<S>, ConfigError> {
        Multer::with_config(self.storage, self.config)
    }
}
