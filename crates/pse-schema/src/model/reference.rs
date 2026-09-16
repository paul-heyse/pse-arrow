// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Correlated references attached to a native value. Paths stay within one visible
//! occurrence; collections are traversed before applying this mapping.

use arrow_schema::{DataType, Field};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{SchemaError, model::FieldContract};

/// Portable logical reference metadata, retained through declared Delta layouts.
pub const KEY_REFERENCE: &str = "pse.semantic.reference";

/// Presence of the mapped key inside a visible containing value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceNullPolicy {
    /// All mapped values must be present.
    Required,
    /// All absent means no reference; a partially present key is invalid.
    AllOrNone,
}

/// One local scalar path and its target column. An empty source path denotes the
/// containing scalar itself; a composite mapping uses named struct-child paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceColumn {
    /// Exact native field names, without parsing dots or SQL identifiers.
    pub source: Vec<String>,
    /// Exact column name in the selected target relation.
    pub target: String,
}

/// One correlated key bound to a relation in the same selected publication catalog.
/// Target-key uniqueness is established by a native query over that exact selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceContract {
    /// Qualified semantic relation name.
    pub relation: String,
    /// Ordered mappings; no independent unnesting of individual key components.
    pub columns: Vec<ReferenceColumn>,
    /// Explicit presence semantics for the correlated key.
    pub null_policy: ReferenceNullPolicy,
}

impl ReferenceContract {
    /// Attach the authoritative declaration. Registry admission validates it before
    /// any native schema or fingerprint is exposed.
    ///
    /// # Errors
    /// Returns an error for an invalid mapping or unsupported source key type.
    pub fn annotate(&self, mut field: Field) -> Result<Field, SchemaError> {
        field
            .metadata_mut()
            .insert(KEY_REFERENCE.into(), self.canonical()?);
        Self::from_field(&field)?;
        Ok(field)
    }

    /// Stable struct serialization, independent of `serde_json` map feature unification.
    ///
    /// # Errors
    /// Returns an error if the declaration cannot be serialized.
    pub fn canonical(&self) -> Result<String, SchemaError> {
        serde_json::to_string(self)
            .map_err(|error| crate::checks::invalid(KEY_REFERENCE, error.to_string()))
    }

    /// Read and validate an explicit mapping. Target resolution occurs at registry
    /// admission and selected-provider binding, never against an ambient catalog.
    ///
    /// # Errors
    /// Rejects malformed metadata, repeated declarations and invalid local mappings.
    pub fn from_field(field: &Field) -> Result<Option<Self>, SchemaError> {
        let Some(text) = field.metadata().get(KEY_REFERENCE) else {
            return Ok(None);
        };
        let invalid = |reason| crate::checks::invalid(field.name(), reason);
        let contract: Self =
            serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
        if contract.canonical()? != *text
            || contract.relation.is_empty()
            || contract.columns.is_empty()
        {
            return Err(invalid(
                "reference requires canonical metadata, a target and a nonempty mapping".into(),
            ));
        }
        if FieldContract::from_field(field.clone()).fk().is_some() {
            return Err(invalid(
                "a field must declare its reference only once".into(),
            ));
        }
        let mut sources = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for mapping in &contract.columns {
            if mapping.target.is_empty()
                || !sources.insert(&mapping.source)
                || !targets.insert(&mapping.target)
            {
                return Err(invalid(
                    "reference mappings require distinct source paths and target columns".into(),
                ));
            }
            let source = source_field(field, &mapping.source)?;
            if !FieldContract::from_field(source.clone()).admits_exact_key() {
                return Err(invalid(
                    "reference components require exact key types".into(),
                ));
            }
        }
        Ok(Some(contract))
    }

    /// Normalize the scalar convenience declaration and correlated declaration into
    /// one native reference contract. Neither form has a separate execution path.
    ///
    /// # Errors
    /// Rejects an invalid explicit reference declaration.
    pub fn for_contract(field: &FieldContract) -> Result<Option<Self>, SchemaError> {
        if let Some(contract) = Self::from_field(field.field())? {
            return Ok(Some(contract));
        }
        Ok(field.fk().map(|fk| Self {
            relation: fk.relation.into(),
            columns: vec![ReferenceColumn {
                source: vec![],
                target: fk.column.into(),
            }],
            null_policy: if field.nullable() {
                ReferenceNullPolicy::AllOrNone
            } else {
                ReferenceNullPolicy::Required
            },
        }))
    }
}

/// Resolve exact struct-child names within one value, preserving field metadata.
///
/// # Errors
/// Rejects missing children or a path that crosses a collection or scalar value.
pub fn source_field<'a>(mut field: &'a Field, path: &[String]) -> Result<&'a Field, SchemaError> {
    for name in path {
        let DataType::Struct(children) = field.data_type() else {
            return Err(crate::checks::invalid(
                field.name(),
                "reference paths may traverse only structs within one occurrence",
            ));
        };
        field = children
            .iter()
            .find(|child| child.name() == name)
            .ok_or_else(|| {
                crate::checks::invalid(field.name(), format!("unknown reference component {name}"))
            })?;
    }
    Ok(field)
}
