// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry-to-catalog contract projection, including nested enum domains.

use crate::{CatalogError, EncodingPolicy, RelationContract};
use pse_schema::Registry;
use pse_schema::model::{ExtensionUse, RelationSpec};

impl RelationContract {
    /// Constructs the canonical and catalog projections from one exact registry
    /// declaration. Only the declared primary key is advertised automatically;
    /// additional uniqueness requires its own validated invariant evidence.
    ///
    /// # Errors
    /// Unknown or modified declarations, missing enum domains and invalid layouts.
    pub fn from_spec(
        reg: &Registry,
        spec: &RelationSpec,
        encodings: EncodingPolicy,
    ) -> Result<Self, CatalogError> {
        let canonical = pse_relations::canonical::contract(reg, spec)?;
        let enums = spec
            .columns
            .iter()
            .filter(|column| matches!(column.extension(), Some(ExtensionUse::Enum(_))))
            .map(pse_schema::model::FieldContract::name)
            .collect::<Vec<_>>();
        Self::try_new(
            canonical,
            spec.key.namespace.as_str(),
            spec.key.name,
            &enums,
            &[],
            encodings,
        )
    }

    /// Checks every contract component against its registry projection. Equal registry
    /// fingerprints alone do not establish that a caller preserved the declaration.
    ///
    /// # Errors
    /// A forged identity, version, field, enum domain, key or constraint declaration.
    pub fn validate_against_registry(
        &self,
        reg: &Registry,
        spec: &RelationSpec,
    ) -> Result<(), CatalogError> {
        let expected = Self::from_spec(reg, spec, self.encodings)?;
        let left = &self.canonical;
        let right = &expected.canonical;
        if self.namespace != expected.namespace
            || self.name != expected.name
            || self.enum_columns != expected.enum_columns
            || self.unique_sets != expected.unique_sets
            || left.relation_id != right.relation_id
            || left.schema_version != right.schema_version
            || left.registry_fingerprint != right.registry_fingerprint
            || left.schema != right.schema
            || left.primary_key != right.primary_key
            || left.layouts != right.layouts
        {
            return Err(admission(
                &spec.key.to_string(),
                "offered canonical/catalog projection differs from the actual registry declaration",
            ));
        }
        Ok(())
    }
}
fn admission(path: &str, reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}
