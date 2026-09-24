// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Versioned native declaration fingerprints. They identify content, never admission.
use crate::{
    Registry, SchemaError,
    model::{RelationKey, RelationSpec},
};
use pse_ids::{ContentHash, FramedHasher, derive::context};

/// Native field/value framing. There is no predecessor decoder.
pub const FRAME_VERSION: &str = "pse.schema.fingerprint.v2";

/// Digest the complete sorted native self-description.
/// # Errors
/// Native field serialization or value encoding fails.
pub fn registry(
    tables: &[(RelationKey, arrow_array::RecordBatch)],
) -> Result<ContentHash, SchemaError> {
    let mut h = FramedHasher::new(context::REGISTRY);
    h.str(FRAME_VERSION);
    h.u64(count(tables.len())?);
    for (key, table) in tables {
        h.str(&key.to_string());
        h.u64(count(table.num_rows())?);
        h.u64(count(table.num_columns())?);
        for (field, array) in table.schema().fields().iter().zip(table.columns()) {
            h.str(&pse_columnar::native_field::canonical_json(field.as_ref()).map_err(invalid)?);
            for row in 0..table.num_rows() {
                h.part(
                    pse_columnar::native_value::semantic_payload(array.as_ref(), field, row)
                        .map_err(invalid)?
                        .as_bytes(),
                );
            }
        }
    }
    Ok(h.finish_hash())
}

/// Digest the native declaration and the referenced logical types and enum domains.
/// # Errors
/// A declaration cannot be represented canonically.
pub fn relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, SchemaError> {
    let mut h = FramedHasher::new(context::REGISTRY);
    h.str(FRAME_VERSION);
    h.str("relation");
    h.str(&spec.key.to_string());
    h.str(&pse_columnar::native_field::canonical_json(&serde_json::json!({
        "authority": spec.authority.as_str(), "snapshot_class": spec.snapshot_class.as_str(),
        "primary_key": spec.primary_key, "granularity": spec.derivation_granularity.map(crate::model::DerivationGranularity::as_str),
        "stability": spec.stability.as_str(), "doc": spec.doc, "checks": spec.checks,
        "delta_properties": spec.delta_properties,
    })).map_err(invalid)?);
    h.u64(count(spec.columns.len())?);
    let mut types = std::collections::BTreeSet::new();
    let mut enums = std::collections::BTreeSet::new();
    for field in &spec.columns {
        h.str(&field.canonical_json()?);
        enums.extend(field.enum_domains());
        let mut reachable = Vec::new();
        field.value_type().walk(&mut reachable);
        for ty in reachable {
            types.insert(ty.type_name()?);
        }
    }
    h.u64(count(types.len())?);
    for name in types {
        let row = reg
            .logical_type(&name)
            .ok_or_else(|| invalid(format!("missing logical type {name}")))?;
        h.str(
            &pse_columnar::native_field::canonical_json(&(
                &row.name,
                &row.arrow_storage,
                &row.extension_name,
                &row.metadata_schema,
            ))
            .map_err(invalid)?,
        );
    }
    h.u64(count(enums.len())?);
    for name in enums {
        let domain = reg
            .enum_spec(&name)
            .ok_or_else(|| invalid(format!("missing enum {name}")))?;
        h.str(domain.name);
        h.str(&pse_columnar::native_field::canonical_json(&domain.idaes_source).map_err(invalid)?);
        h.u64(count(domain.members.len())?);
        for member in &domain.members {
            h.str(
                &pse_columnar::native_field::canonical_json(&(
                    member.name,
                    member.idaes_name,
                    member.deprecated,
                    member.doc,
                ))
                .map_err(invalid)?,
            );
        }
    }
    Ok(h.finish_hash())
}
fn count(value: usize) -> Result<u64, SchemaError> {
    u64::try_from(value).map_err(invalid)
}
fn invalid(error: impl std::fmt::Display) -> SchemaError {
    crate::checks::invalid("native fingerprint", error.to_string())
}
