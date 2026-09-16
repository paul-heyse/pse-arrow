// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Inclusive integer domains on native signed fields. Storage and meaning are distinct:
//! Int64 is portable to Delta; the declared bounds preserve the domain's actual width.

use arrow_schema::{DataType, Field};

use crate::SchemaError;

/// Canonical inclusive `[minimum,maximum]` domain on an Int64 field.
pub const KEY_INTEGER_RANGE: &str = "pse.semantic.integer_range";

/// A closed interval, projected to language validators and native predicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegerRange {
    /// Smallest admitted value.
    pub minimum: i64,
    /// Largest admitted value.
    pub maximum: i64,
}

impl IntegerRange {
    /// Canonical ordinals and counts, bounded by signed storage.
    pub const NONNEGATIVE: Self = Self {
        minimum: 0,
        maximum: i64::MAX,
    };
    /// Source byte offsets retain the parser's checked 32-bit width.
    pub const SOURCE_OFFSET: Self = Self {
        minimum: 0,
        maximum: u32::MAX as i64,
    };

    /// Declare a nonnegative value with an explicit maximum.
    pub const fn nonnegative(maximum: i64) -> Self {
        Self {
            minimum: 0,
            maximum,
        }
    }

    /// Whether a signed value belongs to this domain.
    pub const fn contains(self, value: i64) -> bool {
        self.minimum <= value && value <= self.maximum
    }

    /// Canonical metadata spelling; no second editable physical type is stored.
    pub fn canonical(self) -> String {
        format!("[{},{}]", self.minimum, self.maximum)
    }

    /// Annotate a field. Registry admission checks its storage and interval.
    pub fn annotate(self, mut field: Field) -> Field {
        field
            .metadata_mut()
            .insert(KEY_INTEGER_RANGE.into(), self.canonical());
        field
    }

    /// Required Int64 storage for this domain.
    pub fn field(self, name: &str) -> Field {
        self.annotate(Field::new(name, DataType::Int64, false))
    }

    /// Read and validate the domain, including extension-owned obligations.
    ///
    /// # Errors
    /// Malformed/noncanonical metadata, incompatible storage or an invalid interval.
    pub fn from_field(field: &Field) -> Result<Option<Self>, SchemaError> {
        let ordinal = field
            .metadata()
            .get(crate::arrow::KEY_EXTENSION_NAME)
            .is_some_and(|name| name == "pse.ordinal_ref");
        let Some(text) = field.metadata().get(KEY_INTEGER_RANGE) else {
            return if ordinal {
                Err(crate::checks::invalid(
                    field.name(),
                    "ordinal reference requires its integer domain",
                ))
            } else {
                Ok(None)
            };
        };
        let [minimum, maximum]: [i64; 2] = serde_json::from_str(text)
            .map_err(|error| crate::checks::invalid(field.name(), error.to_string()))?;
        let range = Self { minimum, maximum };
        if field.data_type() != &DataType::Int64
            || minimum > maximum
            || range.canonical() != *text
            || (ordinal && range != Self::NONNEGATIVE)
        {
            return Err(crate::checks::invalid(
                field.name(),
                "integer domain requires canonical Int64 storage and ordered bounds",
            ));
        }
        Ok(Some(range))
    }
}
