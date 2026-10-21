// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Versioned native declaration fingerprints. They identify content, never admission.
use crate::{
    Registry, SchemaError,
    model::{RelationKey, RelationSpec},
};
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Declaration-derived indexes belong to the immutable registry, not a caller.
struct InvariantIndex(BTreeMap<String, Arc<Vec<usize>>>);

pub(crate) fn invariant_indices_for(
    reg: &Registry,
    spec: &RelationSpec,
) -> Result<Arc<Vec<usize>>, SchemaError> {
    let index = invariant_index(reg)?;
    Ok(index
        .0
        .get(&spec.key.qualified_name())
        .cloned()
        .unwrap_or_default())
}

fn invariant_index(reg: &Registry) -> Result<Arc<InvariantIndex>, SchemaError> {
    reg.derived_implementation(|| {
        let mut by_relation: BTreeMap<String, Vec<_>> = reg
            .relations()
            .iter()
            .map(|spec| (spec.key.qualified_name(), Vec::new()))
            .collect();
        for (index, invariant) in reg.invariants().iter().enumerate() {
            by_relation
                .entry(invariant.relation.clone())
                .or_default()
                .push(index);
        }
        Ok(InvariantIndex(
            by_relation
                .into_iter()
                .map(|(name, values)| (name, Arc::new(values)))
                .collect(),
        ))
    })
}

struct SemanticDescriptions(BTreeMap<SemanticId, serde_json::Value>);
struct SemanticProducts {
    slots: BTreeMap<BTreeSet<SemanticId>, u64>,
    values: crate::ImplementationCache,
}
impl SemanticProducts {
    fn new(reg: &Registry) -> Result<Self, SchemaError> {
        let roots: BTreeSet<_> = reg
            .relations()
            .iter()
            .map(|spec| BTreeSet::from([spec.id]))
            .chain(reg.artifact_profile_roots().cloned())
            .chain([
                BTreeSet::new(),
                reg.relations().iter().map(|spec| spec.id).collect(),
            ])
            .collect();
        let slots = roots
            .into_iter()
            .enumerate()
            .map(|(index, roots)| Ok((roots, u64::try_from(index).map_err(invalid)?)))
            .collect::<Result<_, SchemaError>>()?;
        Ok(Self {
            slots,
            values: crate::ImplementationCache::default(),
        })
    }
}
struct SemanticProduct {
    closure: BTreeSet<SemanticId>,
    identity: ContentHash,
}

/// Canonical immutable products exported solely for standard-catalog generation.
#[doc(hidden)]
#[derive(Debug)]
pub struct ResolvedSemanticProducts {
    /// Complete local descriptions in semantic-ID order.
    pub descriptions: BTreeMap<SemanticId, serde_json::Value>,
    /// Declaration-bounded roots, complete support and original framed identity.
    pub closures: Vec<(BTreeSet<SemanticId>, BTreeSet<SemanticId>, ContentHash)>,
    /// Declaration invariant positions by exact qualified relation name.
    pub invariants: BTreeMap<String, Vec<usize>>,
}
pub(crate) fn resolved_products(reg: &Registry) -> Result<ResolvedSemanticProducts, SchemaError> {
    let products = reg.derived_implementation(|| SemanticProducts::new(reg))?;
    let closures = products
        .slots
        .keys()
        .map(|roots| {
            let value = semantic_product_shared(reg, roots)?;
            Ok((roots.clone(), value.closure.clone(), value.identity))
        })
        .collect::<Result<_, SchemaError>>()?;
    let index = invariant_index(reg)?;
    Ok(ResolvedSemanticProducts {
        descriptions: descriptions(reg)?.0.clone(),
        closures,
        invariants: index
            .0
            .iter()
            .map(|(k, v)| (k.clone(), v.as_ref().clone()))
            .collect(),
    })
}
pub(crate) fn install_products(
    reg: &Registry,
    image: ResolvedSemanticProducts,
) -> Result<(), SchemaError> {
    reg.derived_implementation(|| Ok(SemanticDescriptions(image.descriptions)))?;
    reg.derived_implementation(|| {
        Ok(InvariantIndex(
            image
                .invariants
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect(),
        ))
    })?;
    let mut slots = BTreeMap::new();
    let values = crate::ImplementationCache::default();
    for (index, (roots, closure, identity)) in image.closures.into_iter().enumerate() {
        let slot = u64::try_from(index).map_err(invalid)?;
        slots.insert(roots, slot);
        values.get_at(slot, || Ok(SemanticProduct { closure, identity }))?;
    }
    reg.derived_implementation(|| Ok(SemanticProducts { slots, values }))?;
    Ok(())
}

/// Native field/value framing. There is no predecessor decoder.
pub const FRAME_VERSION: &str = "pse.schema.fingerprint.v2";

/// Digest the complete sorted native self-description.
/// # Errors
/// Native field serialization or value encoding fails.
pub fn registry(
    tables: &[(RelationKey, arrow_array::RecordBatch)],
) -> Result<ContentHash, SchemaError> {
    let mut h = FramedHasher::new(Frame::RegistryV1);
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
    // Callers can describe modified independent declarations too. Only the actual
    // declaration belongs to the immutable registry memo.
    if reg
        .relation_by_id(spec.id)
        .is_some_and(|actual| std::ptr::eq(actual, spec))
    {
        return Ok(descriptions(reg)?.0[&spec.id].clone());
    }
    uncached_semantic_description(reg, spec)
}

fn descriptions(reg: &Registry) -> Result<Arc<SemanticDescriptions>, SchemaError> {
    reg.derived_implementation(|| {
        Ok(SemanticDescriptions(
            reg.relations()
                .iter()
                .map(|spec| Ok((spec.id, uncached_semantic_description(reg, spec)?)))
                .collect::<Result<_, SchemaError>>()?,
        ))
    })
}

fn uncached_semantic_description(
    reg: &Registry,
    spec: &RelationSpec,
) -> Result<serde_json::Value, SchemaError> {
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
    let invariants = invariant_indices_for(reg, spec)?.iter().map(|index| {
        let i = &reg.invariants()[*index];
        let inputs: BTreeSet<_> = i.inputs.iter().collect();
        Ok((i.name.clone(), serde_json::json!({"kind":i.kind.as_str(),"query":canonical_sql(&i.query, false)?,"severity":i.severity.as_str(),"inputs":inputs,"keys":i.key_columns})))
    }).collect::<Result<BTreeMap<_, _>, SchemaError>>()?;
    let mut description = serde_json::json!({"relation":spec.key.to_string(),"id":spec.id,"authority":spec.authority.as_str(),"snapshot":spec.snapshot_class.as_str(),"stability":spec.stability.as_str(),"granularity":spec.derivation_granularity.map(crate::model::DerivationGranularity::as_str),"primary_key":spec.primary_key,"fields":fields,"enums":enums,"extensions":extensions,"checks":checks,"invariants":invariants});
    // Key constraints enter the description only when declared, so a relation without
    // them keeps the identity it had before they could be declared.
    if !spec.unique_keys.is_empty() {
        description["unique_keys"] = spec
            .unique_keys
            .iter()
            .map(|key| (key.name.to_owned(), serde_json::json!(key.columns)))
            .collect::<serde_json::Map<_, _>>()
            .into();
    }
    if !spec.foreign_keys.is_empty() {
        description["foreign_keys"] = spec
            .foreign_keys
            .iter()
            .map(|reference| {
                let target = crate::product::referenced_relation(reg, spec, reference.target)
                    .ok_or_else(|| invalid(format!("missing relation {}", reference.target)))?;
                Ok((
                    reference.name.to_owned(),
                    serde_json::json!({"columns":reference.columns,"target":target.id,"target_columns":reference.target_columns}),
                ))
            })
            .collect::<Result<serde_json::Map<_, _>, SchemaError>>()?
            .into();
    }
    Ok(description)
}

/// Complete local semantic identity; dependencies are closed by `semantic_product`.
/// # Errors
/// Invalid semantic description.
pub fn semantic_relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, SchemaError> {
    digest(
        Frame::SchemaSemanticRelationV2,
        &semantic_description(reg, spec)?,
    )
}

/// Persistable support graph. Ordered roots and relation identities retain empty members.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticContract {
    /// Explicit interpretation version, independent of Arrow encoding.
    pub version: u32,
    /// Requested root identities; the relation map is their complete support closure.
    pub roots: BTreeSet<SemanticId>,
    /// Complete semantic descriptions, keyed by relation identity.
    pub relations: BTreeMap<SemanticId, serde_json::Value>,
}
impl SemanticContract {
    /// Build from authoritative declarations, never a receiving consumer's expectations.
    /// # Errors
    /// An unknown root or malformed semantic declaration.
    pub fn new(reg: &Registry, roots: &BTreeSet<SemanticId>) -> Result<Self, SchemaError> {
        let product = semantic_product_shared(reg, roots)?;
        contract_for_closure(reg, roots, &product.closure)
    }
    /// Canonical framed digest; descriptions are the witness, not a validity certificate.
    /// # Errors
    /// Canonical encoding fails.
    pub fn identity(&self) -> Result<ContentHash, SchemaError> {
        digest(Frame::SchemaSemanticProductV2, self)
    }
}

/// Semantic identity of the complete selected contract and invariant inputs.
/// # Errors
/// Invalid closure or declaration.
pub fn semantic_product(
    reg: &Registry,
    roots: &BTreeSet<SemanticId>,
) -> Result<ContentHash, SchemaError> {
    Ok(semantic_product_shared(reg, roots)?.identity)
}

fn semantic_product_shared(
    reg: &Registry,
    roots: &BTreeSet<SemanticId>,
) -> Result<Arc<SemanticProduct>, SchemaError> {
    let products = reg.derived_implementation(|| SemanticProducts::new(reg))?;
    if let Some(slot) = products.slots.get(roots) {
        return products.values.get_at(*slot, || {
            let closure = crate::product::support_closure(reg, roots)?;
            let identity = contract_for_closure(reg, roots, &closure)?.identity()?;
            Ok(SemanticProduct { closure, identity })
        });
    }
    // Arbitrary caller subsets must not accumulate under a process-lived registry.
    // Compose declaration-bounded singleton closures, retaining the original full
    // graph preimage rather than hashing the singleton digests.
    let mut closure = BTreeSet::new();
    for id in roots {
        if reg.relation_by_id(*id).is_none() {
            // Preserve the typed unknown-root failure; never allocate a cache slot.
            return Err(SchemaError::UnknownReference {
                context: "product support".into(),
                reference: id.to_string(),
            });
        }
        closure.extend(
            semantic_product_shared(reg, &BTreeSet::from([*id]))?
                .closure
                .iter()
                .copied(),
        );
    }
    let identity = contract_for_closure(reg, roots, &closure)?.identity()?;
    Ok(Arc::new(SemanticProduct { closure, identity }))
}

fn contract_for_closure(
    reg: &Registry,
    roots: &BTreeSet<SemanticId>,
    closure: &BTreeSet<SemanticId>,
) -> Result<SemanticContract, SchemaError> {
    let descriptions = descriptions(reg)?;
    let relations = closure
        .iter()
        .map(|id| {
            descriptions
                .0
                .get(id)
                .cloned()
                .map(|description| (*id, description))
                .ok_or_else(|| invalid("missing support relation"))
        })
        .collect::<Result<_, SchemaError>>()?;
    // Witnesses are independently owned mutable interchange values. Retain only
    // their shared descriptions and compact closure/identity, not a full cloned
    // witness for every root (which would amplify catalog retention).
    Ok(SemanticContract {
        version: SEMANTIC_VERSION,
        roots: roots.clone(),
        relations,
    })
}

/// Profile requirements and requested support closure, independent of implementation provenance.
/// # Errors
/// Unknown profile, relation, or invalid declaration.
pub fn semantic_profile(
    reg: &Registry,
    profile: &str,
    roots: &BTreeSet<SemanticId>,
) -> Result<ContentHash, SchemaError> {
    let required = reg
        .artifact_profile(profile)
        .ok_or_else(|| invalid("unknown artifact profile"))?;
    let mut selected = roots.clone();
    selected.extend(required);
    let mut h = FramedHasher::new(Frame::SchemaSemanticProfileV2);
    h.str(profile)
        .hash(&semantic_product(reg, &selected)?)
        .u64(count(required.len())?);
    for id in required {
        h.id(id);
    }
    Ok(h.finish_hash())
}

/// The original profile identity from its verified portable required-root inventory.
/// # Errors
/// Requested and required roots do not match the recorded contract roots.
pub fn semantic_profile_recorded(
    recorded: &crate::compatibility::VerifiedRecordedContract,
    profile: &str,
    requested: &BTreeSet<SemanticId>,
    required: &BTreeSet<SemanticId>,
) -> Result<ContentHash, SchemaError> {
    let mut roots = requested.clone();
    roots.extend(required);
    let selected = recorded.select_roots(&roots).map_err(invalid)?;
    let mut h = FramedHasher::new(Frame::SchemaSemanticProfileV2);
    h.str(profile)
        .hash(&selected.identity())
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
    digest(Frame::SchemaExecutionEncodingV1, &fields)
}
fn digest(frame: Frame, value: &impl serde::Serialize) -> Result<ContentHash, SchemaError> {
    let mut h = FramedHasher::new(frame);
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
            contract.relations.keys().copied().collect::<BTreeSet<_>>(),
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

#[cfg(test)]
mod immutable_reuse_unit {
    use super::*;

    #[test]
    fn immutable_reuse_preserves_frozen_catalog_and_reachable_contract_bytes() {
        let registry = crate::catalog::assemble().unwrap();
        // Frozen before B01 from the generated catalog at ac2438a6c. These
        // literals are independent of the runtime memo and its derivation.
        assert_eq!(
            registry.fingerprint().to_hex(),
            "f26f4add2bbd9aebe27ec9bc85532c40b4e0264110866a9edbcfc98ef55d5bb6"
        );
        for (name, bytes) in [
            (
                "reference.dimensions",
                [
                    88, 37, 157, 236, 23, 164, 183, 205, 80, 25, 108, 153, 255, 139, 89, 221, 14,
                    144, 23, 234, 38, 89, 239, 124, 66, 100, 50, 113, 134, 133, 87, 191,
                ],
            ),
            (
                "authored.entities",
                [
                    237, 211, 69, 195, 80, 184, 210, 110, 142, 221, 236, 3, 146, 84, 15, 47, 121,
                    182, 27, 230, 154, 188, 253, 133, 37, 220, 56, 243, 98, 37, 48, 105,
                ],
            ),
        ] {
            let spec = registry.relation(name).unwrap();
            assert_eq!(
                semantic_product(&registry, &BTreeSet::from([spec.id])).unwrap(),
                ContentHash::from_bytes(bytes)
            );
        }
    }

    #[test]
    fn shared_products_keep_the_original_complete_description_preimage() {
        let registry = crate::catalog::assemble().unwrap();
        for spec in registry.relations() {
            let roots = BTreeSet::from([spec.id]);
            let relations = crate::product::support_closure(&registry, &roots)
                .unwrap()
                .into_iter()
                .map(|id| {
                    (
                        id,
                        uncached_semantic_description(
                            &registry,
                            registry.relation_by_id(id).unwrap(),
                        )
                        .unwrap(),
                    )
                })
                .collect();
            let original = SemanticContract {
                version: SEMANTIC_VERSION,
                roots: roots.clone(),
                relations,
            };
            let first = semantic_product_shared(&registry, &roots).unwrap();
            let second = semantic_product_shared(&registry, &roots).unwrap();
            assert!(Arc::ptr_eq(&first, &second));
            assert_eq!(first.identity, original.identity().unwrap());
            assert_eq!(SemanticContract::new(&registry, &roots).unwrap(), original);
            assert_eq!(spec.fingerprint, first.identity);
            assert_eq!(
                semantic_description(&registry, spec).unwrap(),
                uncached_semantic_description(&registry, spec).unwrap()
            );
        }
        // Independent literal wire fixture: no member hashes or implicit roots
        // may replace the existing canonical graph framing.
        let empty = BTreeSet::new();
        let mut expected = FramedHasher::new(Frame::SchemaSemanticProductV2);
        expected.str(r#"{"relations":{},"roots":[],"version":2}"#);
        assert_eq!(
            semantic_product(&registry, &empty).unwrap(),
            expected.finish_hash()
        );
    }

    #[test]
    fn product_cache_completes_once_concurrently_and_keeps_failure_local_to_its_roots() {
        let registry = crate::catalog::assemble().unwrap();
        let roots = registry.artifact_profile("source").unwrap();
        let products = std::thread::scope(|scope| {
            let first = scope.spawn(|| semantic_product_shared(&registry, roots).unwrap());
            let second = scope.spawn(|| semantic_product_shared(&registry, roots).unwrap());
            (first.join().unwrap(), second.join().unwrap())
        });
        assert!(Arc::ptr_eq(&products.0, &products.1));
        let missing = BTreeSet::from([SemanticId::NIL]);
        let first = semantic_product_shared(&registry, &missing)
            .err()
            .unwrap()
            .to_string();
        assert_eq!(
            semantic_product_shared(&registry, &missing)
                .err()
                .unwrap()
                .to_string(),
            first
        );
        assert_eq!(
            semantic_product(&registry, roots).unwrap(),
            products.0.identity
        );
    }

    #[test]
    fn caller_root_combinations_do_not_grow_the_declaration_bounded_memo() {
        let registry = crate::catalog::assemble().unwrap();
        let products = registry
            .derived_implementation(|| SemanticProducts::new(&registry))
            .unwrap();
        let declared = products.slots.len();
        assert!(
            declared <= registry.relations().len() + registry.artifact_profile_roots().count() + 2
        );
        let ids: Vec<_> = registry
            .relations()
            .iter()
            .take(6)
            .map(|spec| spec.id)
            .collect();
        let mut uncached = 0;
        for mask in 0..(1 << ids.len()) {
            let roots = ids
                .iter()
                .enumerate()
                .filter_map(|(index, id)| ((mask & (1 << index)) != 0).then_some(*id))
                .collect();
            let product = semantic_product_shared(&registry, &roots).unwrap();
            if !products.slots.contains_key(&roots) {
                uncached += 1;
                assert!(!Arc::ptr_eq(
                    &product,
                    &semantic_product_shared(&registry, &roots).unwrap()
                ));
            }
            let closure = crate::product::support_closure(&registry, &roots).unwrap();
            assert_eq!(product.closure, closure);
            assert_eq!(
                product.identity,
                contract_for_closure(&registry, &roots, &closure)
                    .unwrap()
                    .identity()
                    .unwrap()
            );
        }
        assert!(uncached > 0);
        assert_eq!(products.slots.len(), declared);
        for roots in registry.artifact_profile_roots() {
            assert!(Arc::ptr_eq(
                &semantic_product_shared(&registry, roots).unwrap(),
                &semantic_product_shared(&registry, roots).unwrap()
            ));
        }
    }

    #[test]
    fn registry_preparation_and_frozen_witness_reconstruction_probe() {
        use std::time::Instant;
        #[derive(serde::Serialize, serde::Deserialize)]
        struct FrozenProjection {
            semantics: SemanticContract,
            schemas: BTreeMap<SemanticId, arrow_schema::Schema>,
        }
        fn median(mut samples: Vec<u128>) -> u128 {
            samples.sort_unstable();
            samples[samples.len() / 2]
        }
        // This is a bounded dev-profile decision probe, not an end-to-end SLA
        // or a claim that this projection can replace the live Registry.
        let mut construction = Vec::new();
        let mut last = None;
        for _ in 0..5 {
            let start = Instant::now();
            let registry = crate::catalog::assemble().unwrap();
            construction.push(start.elapsed().as_micros());
            last = Some(registry);
        }
        let registry = last.unwrap();
        let catalog_roots = registry.relations().iter().map(|spec| spec.id).collect();
        for (label, roots) in [
            (
                "dimensions",
                BTreeSet::from([registry.relation("reference.dimensions").unwrap().id]),
            ),
            ("catalog", catalog_roots),
        ] {
            let semantics = SemanticContract::new(&registry, &roots).unwrap();
            let projection = FrozenProjection {
                schemas: semantics
                    .relations
                    .keys()
                    .map(|id| {
                        (
                            *id,
                            registry
                                .contract(registry.relation_by_id(*id).unwrap())
                                .unwrap()
                                .schema()
                                .as_ref()
                                .clone(),
                        )
                    })
                    .collect(),
                semantics,
            };
            let start = Instant::now();
            let image = pse_columnar::native_field::canonical_json(&projection).unwrap();
            let serialize_us = start.elapsed().as_micros();
            let mut reconstruction = Vec::new();
            let mut decode_and_check = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                let restored: FrozenProjection = serde_json::from_str(&image).unwrap();
                reconstruction.push(start.elapsed().as_micros());
                let identity = restored.semantics.identity().unwrap();
                assert_eq!(identity, semantic_product(&registry, &roots).unwrap());
                assert_eq!(restored.schemas.len(), projection.schemas.len());
                for (id, schema) in &restored.schemas {
                    let expected = registry
                        .contract(registry.relation_by_id(*id).unwrap())
                        .unwrap();
                    // Compare exact serialization, including nested metadata and
                    // dictionary ordering that Arrow's PartialEq can omit.
                    assert_eq!(
                        pse_columnar::native_field::canonical_json(schema).unwrap(),
                        pse_columnar::native_field::canonical_json(expected.schema().as_ref())
                            .unwrap()
                    );
                }
                decode_and_check.push(start.elapsed().as_micros());
            }
            eprintln!(
                "registry-image-probe {{\"slice\":\"{}\",\"conditions\":\"dev, force-validate, five independent catalogs and five serde reconstructions\",\"relations\":{},\"enums\":{},\"invariants\":{},\"image_bytes\":{},\"registry_median_us\":{},\"serialize_us\":{},\"reconstruct_median_us\":{},\"reconstruct_and_check_median_us\":{}}}",
                label,
                projection.schemas.len(),
                registry.enums().len(),
                registry.invariants().len(),
                image.len(),
                median(construction.clone()),
                serialize_us,
                median(reconstruction),
                median(decode_and_check)
            );
        }
    }

    #[test]
    fn derived_description_cycles_are_typed_failures_and_complete_for_later_callers() {
        let registry = crate::RegistryBuilder::new().build().unwrap();
        let result = registry.derived_implementation::<SemanticDescriptions>(|| {
            descriptions(&registry).map(|_| SemanticDescriptions(BTreeMap::new()))
        });
        assert!(matches!(
            result,
            Err(SchemaError::ImplementationCycle { .. })
        ));
        assert!(matches!(
            descriptions(&registry),
            Err(SchemaError::ImplementationCycle { .. })
        ));
    }
}
