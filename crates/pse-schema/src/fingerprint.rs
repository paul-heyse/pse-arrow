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

fn count(value: usize) -> Result<u64, SchemaError> {
    u64::try_from(value).map_err(invalid)
}
fn invalid(error: impl std::fmt::Display) -> SchemaError {
    crate::checks::invalid("native fingerprint", error.to_string())
}

/// Version of the complete semantic contract. Prose and physical layouts are separate.
pub const SEMANTIC_VERSION: u32 = 2;

/// Canonical supported SQL syntax, with no claim of algebraic equivalence.
/// # Errors
/// Invalid syntax, trailing expressions, or multiple statements.
pub fn canonical_sql(sql: &str, expression: bool) -> Result<String, SchemaError> {
    use sqlparser::{dialect::GenericDialect, parser::Parser, tokenizer::Token};
    if expression {
        let mut parser = Parser::new(&GenericDialect)
            .try_with_sql(sql)
            .map_err(invalid)?;
        let expr = parser.parse_expr().map_err(invalid)?;
        parser.expect_token(&Token::EOF).map_err(invalid)?;
        Ok(expr.to_string())
    } else {
        let statements = Parser::parse_sql(&GenericDialect, sql).map_err(invalid)?;
        if statements.len() != 1 {
            return Err(invalid("contract SQL requires one statement"));
        }
        Ok(statements[0].to_string())
    }
}

/// Logical field declaration, preserving unknown metadata and ordered nested paths.
/// # Errors
/// Invalid native metadata.
pub fn semantic_field(field: &arrow_schema::Field) -> Result<arrow_schema::Field, SchemaError> {
    use arrow_schema::DataType;
    use pse_columnar::native_field::{MetadataPurpose, map, project};
    let logical = project(field, MetadataPurpose::ExecutionIdentity).map_err(invalid)?;
    map(&logical, &mut |field| {
        let declared = crate::model::FieldContract::from_field(field.clone());
        let kind = if declared.extension().is_some() {
            DataType::Null
        } else {
            let mut kind = field.data_type();
            while let DataType::Dictionary(_, value) = kind {
                kind = value;
            }
            match kind {
                DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => DataType::Utf8,
                DataType::Binary | DataType::LargeBinary | DataType::BinaryView => DataType::Binary,
                DataType::List(child)
                | DataType::LargeList(child)
                | DataType::ListView(child)
                | DataType::LargeListView(child) => DataType::List(child.clone()),
                kind => kind.clone(),
            }
        };
        let mut metadata = field.metadata().clone();
        if declared.extension().is_none() {
            // This reserved annotation is the registry's physical type alias.
            // Native field shape and its semantic normalization above own meaning.
            metadata.remove(crate::arrow::KEY_LOGICAL_TYPE);
        }
        arrow_schema::Field::new(field.name(), kind, field.is_nullable()).with_metadata(metadata)
    })
    .map_err(invalid)
}

/// Complete local semantic declaration, usable as a recorded independent witness.
/// # Errors
/// Invalid SQL, field declaration, or missing enum/extension definition.
pub fn semantic_description(
    reg: &Registry,
    spec: &RelationSpec,
) -> Result<serde_json::Value, SchemaError> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut enums = BTreeSet::new();
    let mut extensions = BTreeSet::new();
    let fields = spec
        .columns
        .iter()
        .map(|field| {
            enums.extend(field.enum_domains());
            let mut reachable = vec![];
            field.value_type().walk(&mut reachable);
            for ty in reachable {
                if let Some(extension) = ty.extension() {
                    extensions.insert(extension.extension_name().to_owned());
                }
            }
            // Resolved metadata includes the actual enum and quantity identities.
            semantic_field(&crate::arrow::field_for(reg, field)?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let enums = enums
        .into_iter()
        .map(|name| {
            let domain = reg
                .enum_spec(&name)
                .ok_or_else(|| invalid(format!("missing enum {name}")))?;
            let members: BTreeMap<_, _> = domain
                .members
                .iter()
                .map(|m| (m.name, m.deprecated))
                .collect();
            Ok((name, serde_json::json!({"id":domain.id,"members":members})))
        })
        .collect::<Result<BTreeMap<_, _>, SchemaError>>()?;
    let extensions = extensions.into_iter().map(|name| {
        let spec = crate::model::EXTENSION_TYPES.iter().find(|e| e.name == name).ok_or_else(|| invalid(format!("missing extension {name}")))?;
        Ok((name, serde_json::json!({"metadata_kind":spec.metadata.to_string(), "version":spec.metadata_version})))
    }).collect::<Result<BTreeMap<_, _>, SchemaError>>()?;
    let checks = spec
        .checks
        .iter()
        .map(|(name, sql)| Ok((name, canonical_sql(sql, true)?)))
        .collect::<Result<BTreeMap<_, _>, SchemaError>>()?;
    let invariants = reg.invariants().iter().filter(|i| i.relation == spec.key.qualified_name()).map(|i| {
        let inputs: BTreeSet<_> = i.inputs.iter().collect();
        Ok((i.name.clone(), serde_json::json!({"kind":i.kind.as_str(),"query":canonical_sql(&i.query, false)?,"severity":i.severity.as_str(),"inputs":inputs,"keys":i.key_columns})))
    }).collect::<Result<BTreeMap<_, _>, SchemaError>>()?;
    Ok(
        serde_json::json!({"relation":spec.key.to_string(),"id":spec.id,"authority":spec.authority.as_str(),"snapshot":spec.snapshot_class.as_str(),"stability":spec.stability.as_str(),"granularity":spec.derivation_granularity.map(crate::model::DerivationGranularity::as_str),"primary_key":spec.primary_key,"fields":fields,"enums":enums,"extensions":extensions,"checks":checks,"policies":spec.delta_properties,"invariants":invariants}),
    )
}

/// Complete local semantic identity; dependencies are closed by `semantic_product`.
/// # Errors
/// Invalid semantic description.
pub fn semantic_relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, SchemaError> {
    digest(
        "pse.schema.semantic-relation.v2",
        &semantic_description(reg, spec)?,
    )
}

/// Persistable support graph. Ordered roots and relation identities retain empty members.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticContract {
    /// Explicit interpretation version, independent of Arrow/Delta encoding.
    pub version: u32,
    /// Requested root identities; the relation map is their complete support closure.
    pub roots: std::collections::BTreeSet<pse_ids::SemanticId>,
    /// Complete semantic descriptions, keyed by relation identity.
    pub relations: std::collections::BTreeMap<pse_ids::SemanticId, serde_json::Value>,
}
impl SemanticContract {
    /// Build from authoritative declarations, never a receiving consumer's expectations.
    /// # Errors
    /// An unknown root or malformed semantic declaration.
    pub fn new(
        reg: &Registry,
        roots: &std::collections::BTreeSet<pse_ids::SemanticId>,
    ) -> Result<Self, SchemaError> {
        let relations = crate::product::support_closure(reg, roots)?
            .into_iter()
            .map(|id| {
                let spec = reg
                    .relation_by_id(id)
                    .ok_or_else(|| invalid("missing support relation"))?;
                Ok((id, semantic_description(reg, spec)?))
            })
            .collect::<Result<_, SchemaError>>()?;
        Ok(Self {
            version: SEMANTIC_VERSION,
            roots: roots.clone(),
            relations,
        })
    }
    /// Canonical framed digest; descriptions are the witness, not a validity certificate.
    /// # Errors
    /// Canonical encoding fails.
    pub fn identity(&self) -> Result<ContentHash, SchemaError> {
        digest("pse.schema.semantic-product.v2", self)
    }
}

/// Semantic identity of the complete selected contract and invariant inputs.
/// # Errors
/// Invalid closure or declaration.
pub fn semantic_product(
    reg: &Registry,
    roots: &std::collections::BTreeSet<pse_ids::SemanticId>,
) -> Result<ContentHash, SchemaError> {
    SemanticContract::new(reg, roots)?.identity()
}

/// Profile requirements and requested support closure, independent of implementation provenance.
/// # Errors
/// Unknown profile, relation, or invalid declaration.
pub fn semantic_profile(
    reg: &Registry,
    profile: &str,
    roots: &std::collections::BTreeSet<pse_ids::SemanticId>,
) -> Result<ContentHash, SchemaError> {
    let required = reg
        .artifact_profile(profile)
        .ok_or_else(|| invalid("unknown artifact profile"))?;
    let mut selected = roots.clone();
    selected.extend(required);
    let mut h = FramedHasher::new("pse.schema.semantic-profile.v2");
    h.str(profile)
        .hash(&semantic_product(reg, &selected)?)
        .u64(count(required.len())?);
    for id in required {
        h.id(id);
    }
    Ok(h.finish_hash())
}

/// Exact native field layout requirements, with prose removed and no SQL/proto bytes.
/// # Errors
/// Invalid fields or metadata.
pub fn encoding_relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, SchemaError> {
    let fields = spec
        .columns
        .iter()
        .map(|f| crate::arrow::field_for(reg, f))
        .collect::<Result<Vec<_>, _>>()?;
    encoding_fields(fields.iter())
}
/// Exact observed native layout. This does not select semantic meaning.
/// # Errors
/// Invalid field metadata.
pub fn encoding_fields<'a>(
    fields: impl IntoIterator<Item = &'a arrow_schema::Field>,
) -> Result<ContentHash, SchemaError> {
    use pse_columnar::native_field::{MetadataPurpose, project};
    let fields = fields
        .into_iter()
        .map(|f| project(f, MetadataPurpose::ExecutionIdentity).map_err(invalid))
        .collect::<Result<Vec<_>, SchemaError>>()?;
    digest("pse.schema.execution-encoding.v1", &fields)
}
fn digest(domain: &'static str, value: &impl serde::Serialize) -> Result<ContentHash, SchemaError> {
    let mut h = FramedHasher::new(domain);
    h.str(&pse_columnar::native_field::canonical_json(value).map_err(invalid)?);
    Ok(h.finish_hash())
}

#[cfg(test)]
mod semantic_tests {
    use super::*;
    use crate::model::FieldContract;
    use arrow_schema::DataType;

    #[test]
    fn sql_canonicalization_preserves_meaning_and_refuses_trailing_input() {
        assert_eq!(
            canonical_sql("x > 1 AND y IS NOT NULL", true).unwrap(),
            canonical_sql("x>1  and y is not null", true).unwrap()
        );
        assert_eq!(
            canonical_sql("SELECT x FROM t WHERE x > 1", false).unwrap(),
            canonical_sql("select x from t where x>1", false).unwrap()
        );
        assert_ne!(
            canonical_sql("x + y", true).unwrap(),
            canonical_sql("y + x", true).unwrap()
        );
        assert!(canonical_sql("x > 1; DELETE FROM t", true).is_err());
        assert!(canonical_sql("SELECT x FROM t; SELECT y FROM t", false).is_err());
    }
    #[test]
    fn closure_carries_empty_support_and_invariant_inputs() {
        let registry = crate::registry().unwrap();
        let root = registry.relation("authored.entities").unwrap().id;
        let contract = SemanticContract::new(registry, &[root].into()).unwrap();
        let required = crate::product::support_closure(registry, &[root].into()).unwrap();
        assert_eq!(
            contract
                .relations
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            required
        );
        let description = &contract.relations[&root];
        assert!(!description["invariants"].as_object().unwrap().is_empty());
        assert!(!description["primary_key"].as_array().unwrap().is_empty());
        assert_ne!(
            semantic_profile(registry, "inspection", &[root].into()).unwrap(),
            semantic_product(registry, &[root].into()).unwrap()
        );
    }

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
