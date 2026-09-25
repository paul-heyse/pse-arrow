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

/// Logical contract identity, independent of prose and qualified native encodings.
/// This is additive: durable opening continues to enforce `relation` until the
/// explicit P12 compatibility migration. Unknown metadata remains significant.
/// # Errors
/// Invalid declarations or missing enum domains.
pub fn semantic_relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, SchemaError> {
    use arrow_schema::DataType;
    use pse_columnar::native_field::{MetadataPurpose, canonical_json, map, project};
    let mut h = FramedHasher::new("pse.schema.semantic-relation.v1");
    h.str(&spec.key.to_string())
        .str(spec.authority.as_str())
        .str(spec.snapshot_class.as_str())
        .str(spec.stability.as_str());
    h.str(
        &canonical_json(&(
            &spec.primary_key,
            spec.derivation_granularity
                .map(crate::model::DerivationGranularity::as_str),
            &spec.checks,
        ))
        .map_err(invalid)?,
    );
    h.u64(count(spec.columns.len())?);
    let mut enums = std::collections::BTreeSet::new();
    let mut logical_types = std::collections::BTreeSet::new();
    for field in &spec.columns {
        enums.extend(field.enum_domains());
        let mut reachable = Vec::new();
        field.value_type().walk(&mut reachable);
        for ty in reachable {
            if ty.extension().is_some() {
                logical_types.insert(ty.type_name()?);
            }
        }
        let logical =
            project(field.field(), MetadataPurpose::ExecutionIdentity).map_err(invalid)?;
        let logical = map(&logical, &mut |field| {
            let declared = crate::model::FieldContract::from_field(field.clone());
            let kind = if declared.extension().is_some() {
                // The named logical extension and all its parameters are metadata.
                // Its physical child layout belongs to the encoding fingerprint.
                DataType::Null
            } else {
                let mut kind = field.data_type();
                while let DataType::Dictionary(_, value) = kind {
                    kind = value;
                }
                match kind {
                    DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => DataType::Utf8,
                    DataType::Binary | DataType::LargeBinary | DataType::BinaryView => {
                        DataType::Binary
                    }
                    DataType::List(child)
                    | DataType::LargeList(child)
                    | DataType::ListView(child)
                    | DataType::LargeListView(child) => DataType::List(child.clone()),
                    kind => kind.clone(),
                }
            };
            // Constructing a new field removes dictionary key/order encoding details.
            arrow_schema::Field::new(field.name(), kind, field.is_nullable())
                .with_metadata(field.metadata().clone())
        })
        .map_err(invalid)?;
        h.str(&canonical_json(&logical).map_err(invalid)?);
    }
    h.u64(count(logical_types.len())?);
    for name in logical_types {
        let logical = reg
            .logical_type(&name)
            .ok_or_else(|| invalid(format!("missing logical type {name}")))?;
        h.str(
            &canonical_json(&(
                &logical.name,
                &logical.extension_name,
                &logical.metadata_schema,
            ))
            .map_err(invalid)?,
        );
    }
    h.u64(count(enums.len())?);
    for name in enums {
        let domain = reg
            .enum_spec(&name)
            .ok_or_else(|| invalid(format!("missing enum {name}")))?;
        h.str(domain.name).u64(count(domain.members.len())?);
        // Enum order is presentation, not meaning. Names and deprecation are semantic.
        let mut members: Vec<_> = domain.members.iter().collect();
        members.sort_by_key(|member| member.name);
        for member in members {
            h.str(member.name).bool(member.deprecated);
        }
    }
    let mut invariants: Vec<_> = reg
        .invariants()
        .iter()
        .filter(|i| i.relation == spec.key.qualified_name())
        .collect();
    invariants.sort_by_key(|i| &i.name);
    h.u64(count(invariants.len())?);
    for invariant in invariants {
        let mut inputs = invariant.inputs.clone();
        inputs.sort();
        h.str(&invariant.name)
            .str(invariant.kind.as_str())
            .str(&invariant.query)
            .str(invariant.severity.as_str());
        h.str(&canonical_json(&(&inputs, &invariant.key_columns)).map_err(invalid)?);
    }
    Ok(h.finish_hash())
}

/// Semantic identity of the entire referenced contract, including empty support
/// relations and invariant inputs. Physical encodings retain their separate digest.
/// # Errors
/// An unknown root, reference or logical declaration.
pub fn semantic_product(
    reg: &Registry,
    roots: &std::collections::BTreeSet<pse_ids::SemanticId>,
) -> Result<ContentHash, SchemaError> {
    let closure = crate::product::support_closure(reg, roots)?;
    let mut h = FramedHasher::new("pse.schema.semantic-product.v1");
    h.u64(count(roots.len())?);
    for root in roots {
        h.id(root);
    }
    h.u64(count(closure.len())?);
    for id in closure {
        let relation = reg
            .relation_by_id(id)
            .ok_or_else(|| invalid("missing support relation"))?;
        h.id(&id).hash(&semantic_relation(reg, relation)?);
    }
    Ok(h.finish_hash())
}

#[cfg(test)]
mod semantic_tests {
    use super::*;
    use crate::model::FieldContract;
    use arrow_schema::DataType;

    #[test]
    fn semantic_contract_separates_documentation_and_native_encoding() {
        let registry = crate::registry().unwrap();
        let mut relation = registry.relations()[0].clone();
        relation.columns = vec![
            FieldContract::native(DataType::Utf8)
                .with_name("label")
                .with_doc("first"),
        ];
        let original = semantic_relation(registry, &relation).unwrap();
        relation.doc = "different relation prose";
        relation.columns[0] = relation.columns[0].clone().with_doc("second");
        assert_eq!(original, semantic_relation(registry, &relation).unwrap());
        relation.columns[0] = FieldContract::native(DataType::LargeUtf8).with_name("label");
        assert_eq!(original, semantic_relation(registry, &relation).unwrap());
        relation.columns[0] = relation.columns[0].clone().optional();
        assert_ne!(original, semantic_relation(registry, &relation).unwrap());
    }
}
