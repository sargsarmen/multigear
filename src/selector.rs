use std::collections::HashMap;

use crate::{
    MulterError, SelectedField, Selector, UnknownFieldPolicy,
};

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
    fields: HashMap<String, Option<usize>>,
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
                let Some(max_count) = self.fields.get(field_name).copied() else {
                    return self.handle_unknown_field(field_name);
                };
                self.record_with_limit(field_name, max_count)?;
                Ok(SelectorAction::Accept)
            }
            Selector::None => self.handle_unknown_field(field_name),
            Selector::Any => Ok(SelectorAction::Accept),
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

    fn record_with_limit(&mut self, field_name: &str, max_count: Option<usize>) -> Result<(), MulterError> {
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
}

fn build_fields_map(selector: &Selector) -> HashMap<String, Option<usize>> {
    match selector {
        Selector::Fields(fields) => {
            let mut map = HashMap::with_capacity(fields.len());
            for SelectedField { name, max_count } in fields {
                map.insert(name.clone(), *max_count);
            }
            map
        }
        _ => HashMap::new(),
    }
}
