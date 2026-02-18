use crate::config::{SelectedField, SelectedFieldKind};

/// Multipart field model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    /// File upload field metadata.
    File(FileField),
    /// Text field metadata.
    Text(TextField),
}

impl Field {
    /// Creates a selector field descriptor.
    ///
    /// This is a convenience alias for [`SelectedField::new`].
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: impl Into<String>) -> SelectedField {
        SelectedField::new(name)
    }

    /// Creates a file field model for the provided name.
    pub fn file(name: impl Into<String>) -> Self {
        Self::File(FileField::new(name))
    }

    /// Creates a text field model for the provided name.
    pub fn text(name: impl Into<String>) -> Self {
        Self::Text(TextField::new(name))
    }

    /// Sets the maximum number of file parts accepted for this field.
    pub fn max_count(mut self, max_count: usize) -> Self {
        if let Self::File(field) = &mut self {
            field.max_count = Some(max_count);
        }
        self
    }

    /// Sets MIME patterns accepted for this file field.
    pub fn allowed_mime_types<I, M>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: Into<String>,
    {
        if let Self::File(field) = &mut self {
            field.allowed_mime_types = patterns.into_iter().map(Into::into).collect();
        }
        self
    }

    /// Sets the maximum accepted text length in bytes for this text field.
    pub fn max_size(mut self, max_size: u64) -> Self {
        if let Self::Text(field) = &mut self {
            field.max_size = Some(max_size);
        }
        self
    }

    /// Returns the logical field name.
    pub fn name(&self) -> &str {
        match self {
            Self::File(field) => &field.name,
            Self::Text(field) => &field.name,
        }
    }

    /// Returns the field kind.
    pub fn kind(&self) -> FieldKind {
        match self {
            Self::File(_) => FieldKind::File,
            Self::Text(_) => FieldKind::Text,
        }
    }
}

/// Discriminates between file and text fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Binary file payload.
    File,
    /// Plain text payload.
    Text,
}

/// File field metadata and constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileField {
    /// Logical field name.
    pub name: String,
    /// Maximum number of file parts accepted for this field.
    pub max_count: Option<usize>,
    /// Allowed MIME patterns for this field.
    pub allowed_mime_types: Vec<String>,
}

impl FileField {
    /// Creates a file field with no explicit per-field count limit.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_count: None,
            allowed_mime_types: Vec::new(),
        }
    }

    /// Sets the maximum number of file parts for this field.
    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count = Some(max_count);
        self
    }

    /// Alias for [`FileField::with_max_count`].
    pub fn max_count(self, max_count: usize) -> Self {
        self.with_max_count(max_count)
    }

    /// Sets MIME patterns accepted for this file field.
    pub fn with_allowed_mime_types<I, M>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: Into<String>,
    {
        self.allowed_mime_types = patterns.into_iter().map(Into::into).collect();
        self
    }

    /// Alias for [`FileField::with_allowed_mime_types`].
    pub fn allowed_mime_types<I, M>(self, patterns: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: Into<String>,
    {
        self.with_allowed_mime_types(patterns)
    }
}

/// Text field metadata and constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextField {
    /// Logical field name.
    pub name: String,
    /// Maximum accepted text size in bytes.
    pub max_size: Option<u64>,
}

impl TextField {
    /// Creates a text field with no explicit per-field size limit.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            max_size: None,
        }
    }

    /// Sets the maximum text size in bytes for this field.
    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Alias for [`TextField::with_max_size`].
    pub fn max_size(self, max_size: u64) -> Self {
        self.with_max_size(max_size)
    }

    /// Backward-compatible alias for [`TextField::with_max_size`].
    pub fn with_max_length(self, max_length: usize) -> Self {
        self.with_max_size(max_length as u64)
    }
}

impl From<Field> for SelectedField {
    fn from(value: Field) -> Self {
        match value {
            Field::File(field) => field.into(),
            Field::Text(field) => field.into(),
        }
    }
}

impl From<FileField> for SelectedField {
    fn from(value: FileField) -> Self {
        SelectedField {
            name: value.name,
            kind: SelectedFieldKind::File,
            max_count: value.max_count,
            max_size: None,
            allowed_mime_types: value.allowed_mime_types,
        }
    }
}

impl From<TextField> for SelectedField {
    fn from(value: TextField) -> Self {
        SelectedField {
            name: value.name,
            kind: SelectedFieldKind::Text,
            max_count: None,
            max_size: value.max_size,
            allowed_mime_types: Vec::new(),
        }
    }
}
