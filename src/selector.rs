use std::collections::HashMap;

use crate::{MulterError, SelectedField, SelectedFieldKind, Selector, UnknownFieldPolicy};

/// Runtime decision for a candidate incoming file part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorAction {
    /// Accept and continue yielding this part.
    Accept,
    /// Ignore and skip this part.
    Ignore,
}

/// Stateful runtime selector engine.
#[derive(Debug, Clone)]
pub struct SelectorEngine {
    selector: Selector,
    unknown_field_policy: UnknownFieldPolicy,
    counts: HashMap<String, usize>,
    fields: HashMap<String, FieldRules>,
}

impl SelectorEngine {
    /// Creates a selector engine with runtime counters.
    pub fn new(selector: Selector, unknown_field_policy: UnknownFieldPolicy) -> Self {
        let fields = build_fields_map(&selector);
        Self {
            selector,
            unknown_field_policy,
            counts: HashMap::new(),
            fields,
        }
    }

    /// Applies selector rules for a file field and returns the action.
    pub fn evaluate_file_field(&mut self, field_name: &str) -> Result<SelectorAction, MulterError> {
        match &self.selector {
            Selector::Single { name } => {
                if field_name != name {
                    return self.handle_unknown_field(field_name);
                }
                self.record_with_limit(field_name, Some(1))?;
                Ok(SelectorAction::Accept)
            }
            Selector::Array { name, max_count } => {
                if field_name != name {
                    return self.handle_unknown_field(field_name);
                }
                self.record_with_limit(field_name, *max_count)?;
                Ok(SelectorAction::Accept)
            }
            Selector::Fields(_) => {
                let Some(rules) = self.fields.get(field_name).cloned() else {
                    return self.handle_unknown_field(field_name);
                };
                if rules.kind != SelectedFieldKind::File {
                    return self.handle_unknown_field(field_name);
                }
                self.record_with_limit(field_name, rules.max_count)?;
                Ok(SelectorAction::Accept)
            }
            Selector::None => self.handle_unknown_field(field_name),
            Selector::Any => Ok(SelectorAction::Accept),
        }
    }

    /// Applies selector rules for a text field and returns the action.
    pub fn evaluate_text_field(&self, field_name: &str) -> Result<SelectorAction, MulterError> {
        match &self.selector {
            Selector::Fields(_) => {
                let Some(rules) = self.fields.get(field_name) else {
                    return self.handle_unknown_field(field_name);
                };
                if rules.kind != SelectedFieldKind::Text {
                    return self.handle_unknown_field(field_name);
                }
                Ok(SelectorAction::Accept)
            }
            Selector::Single { .. } | Selector::Array { .. } | Selector::None | Selector::Any => {
                Ok(SelectorAction::Accept)
            }
        }
    }

    fn handle_unknown_field(&self, field_name: &str) -> Result<SelectorAction, MulterError> {
        match self.unknown_field_policy {
            UnknownFieldPolicy::Reject => Err(MulterError::UnexpectedField {
                field: field_name.to_owned(),
            }),
            UnknownFieldPolicy::Ignore => Ok(SelectorAction::Ignore),
        }
    }

    fn record_with_limit(
        &mut self,
        field_name: &str,
        max_count: Option<usize>,
    ) -> Result<(), MulterError> {
        let next = self.counts.get(field_name).copied().unwrap_or(0) + 1;
        if let Some(max_count) = max_count {
            if next > max_count {
                return Err(MulterError::FieldCountLimitExceeded {
                    field: field_name.to_owned(),
                    max_count,
                });
            }
        }
        self.counts.insert(field_name.to_owned(), next);
        Ok(())
    }

    /// Returns MIME patterns configured for a selected field, if present.
    pub fn field_allowed_mime_types(&self, field_name: &str) -> Option<&[String]> {
        self.fields
            .get(field_name)
            .map(|rules| rules.allowed_mime_types.as_slice())
    }

    /// Returns the configured text size limit for a selected field, if present.
    pub fn field_text_max_size(&self, field_name: &str) -> Option<u64> {
        self.fields.get(field_name).and_then(|rules| {
            if rules.kind == SelectedFieldKind::Text {
                rules.max_size
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone)]
struct FieldRules {
    kind: SelectedFieldKind,
    max_count: Option<usize>,
    max_size: Option<u64>,
    allowed_mime_types: Vec<String>,
}

fn build_fields_map(selector: &Selector) -> HashMap<String, FieldRules> {
    match selector {
        Selector::Fields(fields) => {
            let mut map = HashMap::with_capacity(fields.len());
            for SelectedField {
                name,
                kind,
                max_count,
                max_size,
                allowed_mime_types,
            } in fields
            {
                map.insert(
                    name.clone(),
                    FieldRules {
                        kind: *kind,
                        max_count: *max_count,
                        max_size: *max_size,
                        allowed_mime_types: allowed_mime_types.clone(),
                    },
                );
            }
            map
        }
        _ => HashMap::new(),
    }
}
