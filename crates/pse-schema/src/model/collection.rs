// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Collection meaning on the native list field. Absence and element nullability
//! remain Arrow field properties; an empty visible collection has cardinality zero.

use arrow_schema::{DataType, Field};

use crate::SchemaError;

/// Canonical metadata for collection order, cardinality and uniqueness.
pub const KEY_COLLECTION: &str = "pse.semantic.collection";

/// Whether positions contribute to the meaning of a collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionOrder {
    /// Positions are meaningful and must be preserved by transformations.
    Sequence,
    /// Positions carry no domain meaning; no physical sorting is implied.
    Unordered,
}

/// Native list facets, independent of physical list offset width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionContract {
    /// Meaning of positions, not a claim that values are sorted.
    pub order: CollectionOrder,
    /// Minimum visible cardinality; zero permits an empty collection.
    pub minimum: i64,
    /// Maximum visible cardinality, if bounded.
    pub maximum: Option<i64>,
    /// No two visible members may compare equal, including repeated nulls.
    pub unique: bool,
}

impl CollectionContract {
    /// An ordered collection which admits empty values and repeated members.
    pub const SEQUENCE: Self = Self {
        order: CollectionOrder::Sequence,
        minimum: 0,
        maximum: None,
        unique: false,
    };

    /// An unordered collection of distinct members.
    pub const SET: Self = Self {
        order: CollectionOrder::Unordered,
        unique: true,
        ..Self::SEQUENCE
    };

    /// Canonical representation independent of `serde_json` feature unification.
    pub fn canonical(&self) -> String {
        let mut value = serde_json::json!({
            "order": self.order, "minimum": self.minimum,
            "maximum": self.maximum, "unique": self.unique,
        });
        value.sort_all_objects();
        value.to_string()
    }

    /// Attach the declaration; registry and field admission check its consistency.
    pub fn annotate(self, mut field: Field) -> Field {
        field
            .metadata_mut()
            .insert(KEY_COLLECTION.into(), self.canonical());
        field
    }

    /// Decode and admit a collection declaration on a native list field.
    /// # Errors
    /// Noncanonical metadata, invalid bounds, non-list storage or a conflicting width.
    pub fn from_field(field: &Field) -> Result<Option<Self>, SchemaError> {
        let Some(text) = field.metadata().get(KEY_COLLECTION) else {
            return Ok(None);
        };
        let invalid = |reason| crate::checks::invalid(field.name(), reason);
        let value: Self = serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
        let width = match field.data_type() {
            DataType::List(_)
            | DataType::LargeList(_)
            | DataType::ListView(_)
            | DataType::LargeListView(_) => None,
            DataType::FixedSizeList(_, width) => Some(i64::from(*width)),
            _ => {
                return Err(invalid(
                    "collection facets require native list storage".into(),
                ));
            }
        };
        if text != &value.canonical()
            || value.minimum < 0
            || value.maximum.is_some_and(|maximum| maximum < value.minimum)
            || width.is_some_and(|width| {
                width < value.minimum || value.maximum.is_some_and(|maximum| width > maximum)
            })
        {
            return Err(invalid(
                "invalid collection cardinality or noncanonical metadata".into(),
            ));
        }
        Ok(Some(value))
    }

    /// Check one typed value at a language boundary. Native execution lowers these
    /// same facets to `array_length` and `array_distinct` expressions.
    pub fn accepts<T: PartialEq>(&self, values: &[T]) -> bool {
        i64::try_from(values.len()).is_ok_and(|length| {
            length >= self.minimum && self.maximum.is_none_or(|maximum| length <= maximum)
        }) && (!self.unique
            || values
                .iter()
                .enumerate()
                .all(|(index, value)| !values[..index].contains(value)))
    }
}
