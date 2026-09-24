// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A selected struct arm, declared once on the native field tree.

use std::collections::{BTreeMap, BTreeSet};

use arrow_schema::{DataType, Field};

use crate::SchemaError;

/// Canonical discriminator and tag-to-arm binding; payload types live in Arrow fields.
pub const KEY_TAGGED_ALTERNATIVE: &str = "pse.semantic.tagged_alternative";

/// A required tag selects a payload arm or an explicitly payload-free alternative.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaggedAlternative {
    /// Required string field containing the selected tag.
    pub discriminator: String,
    /// Each tag selects an optional struct field, or no payload. Tags may share an arm.
    pub arms: BTreeMap<String, Option<String>>,
}

impl TaggedAlternative {
    /// Declare selection independently of the payloads' native field types.
    pub fn new(discriminator: &str, arms: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            discriminator: discriminator.into(),
            arms: arms
                .into_iter()
                .map(|(tag, arm)| (tag, Some(arm)))
                .collect(),
        }
    }

    /// Add a tag whose valid value has no payload arms present.
    #[must_use]
    pub fn with_unit(mut self, tag: impl Into<String>) -> Self {
        self.arms.insert(tag.into(), None);
        self
    }

    /// The distinct payload fields, independent of how many tags select each one.
    pub fn payloads(&self) -> BTreeSet<&str> {
        self.arms.values().filter_map(Option::as_deref).collect()
    }

    /// Deterministic metadata; no second payload schema is serialized.
    pub fn canonical(&self) -> String {
        let mut value = serde_json::json!({"discriminator": self.discriminator, "arms": self.arms});
        value.sort_all_objects();
        value.to_string()
    }

    /// Attach the declaration. Normal field admission verifies its shape.
    pub fn annotate(&self, mut field: Field) -> Field {
        field
            .metadata_mut()
            .insert(KEY_TAGGED_ALTERNATIVE.into(), self.canonical());
        field
    }

    /// Decode and verify the contract without depending on runtime registry state.
    /// # Errors
    /// Noncanonical metadata, missing/extra fields, invalid tag storage or arm shape.
    pub fn from_field(field: &Field) -> Result<Option<Self>, SchemaError> {
        let Some(text) = field.metadata().get(KEY_TAGGED_ALTERNATIVE) else {
            return Ok(None);
        };
        let invalid = |reason| crate::checks::invalid(field.name(), reason);
        let value: Self = serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
        let DataType::Struct(fields) = field.data_type() else {
            return Err(invalid("tagged alternative requires struct storage".into()));
        };
        if value.canonical() != *text || value.arms.is_empty() {
            return Err(invalid(
                "tagged alternative requires canonical nonempty arms".into(),
            ));
        }
        let tag = fields
            .iter()
            .find(|child| child.name() == &value.discriminator);
        if !tag.is_some_and(|tag| !tag.is_nullable() && tag.data_type() == &DataType::Utf8) {
            return Err(invalid(
                "discriminator must be a required Utf8 field".into(),
            ));
        }
        let payloads = value.payloads();
        if value.arms.keys().any(String::is_empty)
            || payloads.contains(value.discriminator.as_str())
            || payloads.iter().any(|name| !fields.iter().any(|field| {
                field.name() == name && field.is_nullable()
                    && matches!(field.data_type(), DataType::Struct(children) if !children.is_empty())
            }))
        {
            return Err(invalid("payload tags must select optional nonempty struct arms".into()));
        }
        if payloads.len() + 1 != fields.len() {
            return Err(invalid(
                "tagged alternative has undeclared payload fields".into(),
            ));
        }
        Ok(Some(value))
    }
}
