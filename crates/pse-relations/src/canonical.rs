// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The sole registry projection for canonical ordering and identity construction.

use crate::RelationError;
use pse_ids::{CanonicalContract, FieldPath, SchemaVersion};
use pse_schema::{
    Registry,
    model::{FieldContract, RelationSpec},
};
use std::{collections::BTreeMap, sync::Arc};

/// Project actual declared fields, keys and nested enum domains into the codec.
/// This establishes the transformation contract, not validity of any caller's rows.
///
/// # Errors
/// An unknown or different declaration, missing domain or unsupported canonical layout.
pub fn contract(
    registry: &Registry,
    spec: &RelationSpec,
) -> Result<CanonicalContract, RelationError> {
    if registry.relation_by_id(spec.id) != Some(spec) {
        return Err(RelationError::Contract {
            relation: spec.qualified_name(),
            reason: "canonical descriptor differs from the authoritative registry declaration"
                .into(),
        });
    }
    let schema = pse_schema::arrow::relation_schema(registry, spec)?;
    let mut domains = BTreeMap::new();
    for (index, column) in spec.columns.iter().enumerate() {
        domains_for(
            registry,
            &column.value_type(),
            &FieldPath::root().child(index),
            &mut domains,
        )?;
    }
    Ok(CanonicalContract::try_new(
        spec.id,
        SchemaVersion(spec.key.version),
        registry.fingerprint(),
        Arc::new(schema),
        &spec.primary_key,
        &domains,
    )?)
}

fn domains_for(
    registry: &Registry,
    ty: &FieldContract,
    path: &FieldPath,
    out: &mut BTreeMap<FieldPath, Arc<[String]>>,
) -> Result<(), RelationError> {
    if let Some(name) = ty.enum_name() {
        out.insert(path.clone(), enum_members(registry, name)?);
    }
    for (index, child) in ty.children().iter().enumerate() {
        domains_for(registry, child, &path.child(index), out)?;
    }
    Ok(())
}
fn enum_members(registry: &Registry, name: &str) -> Result<Arc<[String]>, RelationError> {
    let enumeration = registry
        .enum_spec(name)
        .ok_or_else(|| RelationError::Contract {
            relation: name.to_owned(),
            reason: "canonical enum domain is not declared".into(),
        })?;
    Ok(enumeration
        .members
        .iter()
        .map(|member| member.name.to_owned())
        .collect::<Vec<_>>()
        .into())
}
