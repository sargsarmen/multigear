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
    pub fn fields<F>(mut self, fields: impl IntoIterator<Item = F>) -> Self
    where
        F: Into<crate::config::SelectedField>,
    {
        self.config.selector = Selector::fields(fields.into_iter().map(Into::into));
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

    /// Alias for [`MulterBuilder::unknown_field_policy`].
    pub fn on_unknown_field(self, policy: UnknownFieldPolicy) -> Self {
        self.unknown_field_policy(policy)
    }

    /// Sets global multipart limits.
    pub fn limits(mut self, limits: Limits) -> Self {
        self.config.limits = limits;
        self
    }

    /// Sets the maximum accepted file size in bytes.
    pub fn max_file_size(mut self, max_file_size: u64) -> Self {
        self.config.limits.max_file_size = Some(max_file_size);
        self
    }

    /// Sets the maximum accepted number of files.
    pub fn max_files(mut self, max_files: usize) -> Self {
        self.config.limits.max_files = Some(max_files);
        self
    }

    /// Sets the maximum accepted text field size in bytes.
    pub fn max_field_size(mut self, max_field_size: u64) -> Self {
        self.config.limits.max_field_size = Some(max_field_size);
        self
    }

    /// Sets the maximum accepted number of text fields.
    pub fn max_fields(mut self, max_fields: usize) -> Self {
        self.config.limits.max_fields = Some(max_fields);
        self
    }

    /// Sets the maximum accepted multipart request size in bytes.
    pub fn max_body_size(mut self, max_body_size: u64) -> Self {
        self.config.limits.max_body_size = Some(max_body_size);
        self
    }

    /// Sets the global list of allowed MIME patterns.
    pub fn allowed_mime_types<I, M>(mut self, allowed_mime_types: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: Into<String>,
    {
        self.config.limits.allowed_mime_types =
            allowed_mime_types.into_iter().map(Into::into).collect();
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
