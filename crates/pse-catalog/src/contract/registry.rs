// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry-to-catalog contract projection, including nested enum domains.

use crate::{CatalogError, EncodingPolicy, RelationContract};
use pse_ids::{CanonicalContract, FieldPath, SchemaVersion};
use pse_schema::Registry;
use pse_schema::model::{ExtensionUse, LogicalType, RelationSpec};
use std::collections::BTreeMap;
use std::sync::Arc;

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
        if reg.relation_by_id(spec.id) != Some(spec) {
            return Err(admission(
                &spec.key.to_string(),
                "descriptor differs from its registry declaration",
            ));
        }
        let schema = pse_schema::arrow::relation_schema(reg, spec)
            .map_err(|error| admission(&spec.key.to_string(), &error.to_string()))?;
        pse_relations::validate::validate_schema(reg, spec, &schema).map_err(|errors| {
            admission(
                &spec.key.to_string(),
                &errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;
        let mut domains = BTreeMap::new();
        let mut enums = Vec::new();
        for (index, column) in spec.columns.iter().enumerate() {
            domains_for(
                reg,
                &column.logical_type,
                &FieldPath::root().child(index),
                &mut domains,
            )?;
            if matches!(column.logical_type, LogicalType::Ext(ExtensionUse::Enum(_))) {
                enums.push(column.name);
            }
        }
        let canonical = CanonicalContract::try_new(
            spec.id,
            SchemaVersion(spec.key.version),
            reg.fingerprint(),
            Arc::new(schema),
            &spec.primary_key,
            &domains,
        )?;
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
fn domains_for(
    reg: &Registry,
    ty: &LogicalType,
    path: &FieldPath,
    out: &mut BTreeMap<FieldPath, Arc<[String]>>,
) -> Result<(), CatalogError> {
    match ty {
        LogicalType::Ext(ExtensionUse::Enum(name)) => {
            out.insert(path.clone(), enum_members(reg, name)?);
        }
        LogicalType::Ext(ExtensionUse::Bound) => {
            out.insert(path.child(0), enum_members(reg, "BoundKind")?);
        }
        LogicalType::List(child) | LogicalType::FixedList(child, _) => {
            domains_for(reg, child, &path.child(0), out)?;
        }
        LogicalType::Struct(children) => {
            for (index, (_, child, _)) in children.iter().enumerate() {
                domains_for(reg, child, &path.child(index), out)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn enum_members(reg: &Registry, name: &str) -> Result<Arc<[String]>, CatalogError> {
    let enumeration = reg
        .enum_spec(name)
        .ok_or_else(|| admission(name, "enum domain is not declared in the bound registry"))?;
    Ok(enumeration
        .members
        .iter()
        .map(|member| member.name.to_owned())
        .collect::<Vec<_>>()
        .into())
}
