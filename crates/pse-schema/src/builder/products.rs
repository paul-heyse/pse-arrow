// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete declaration-derived products; hydration never imports runtime proofs.

use super::*;
use std::sync::Arc;

/// Read-only generation projection of an independently assembled registry.
#[doc(hidden)]
#[derive(Debug)]
pub struct ResolvedRegistryProducts {
    /// Registry package identity.
    pub package_id: SemanticId,
    /// Complete self-description identity.
    pub fingerprint: ContentHash,
    /// Exact resolved declarations.
    pub relations: Vec<RelationSpec>,
    /// Compiled declaration obligations.
    pub obligations: BTreeMap<RelationKey, crate::obligations::RelationObligations>,
    /// Declared dictionaries including generation metadata.
    pub enums: Vec<EnumSpec>,
    /// Resolved identity owners.
    pub identities: Vec<IdentitySpec>,
    /// Named native structures.
    pub structures: BTreeMap<String, FieldContract>,
    /// Complete logical type inventory.
    pub logical_types: Vec<FieldTypeRow>,
    /// Exact resolved logical storage in inventory order.
    pub logical_storage: Vec<arrow_schema::DataType>,
    /// Complete invariants including their declaration origin.
    pub invariants: Vec<InvariantSpec>,
    /// Exact registered migrations.
    pub migrations: Vec<MigrationSpec>,
    /// Resolved algorithm declarations.
    pub algorithms: Vec<AlgorithmSpec>,
    /// Authoring document declarations.
    pub documents: Vec<DocumentSpec>,
    /// Declared complete artifact roots.
    pub artifact_profiles: BTreeMap<String, BTreeSet<SemanticId>>,
    /// Complete resolved graph and execution schemas.
    pub contracts: crate::resolved_contract::ResolvedContractProducts,
    /// Canonical descriptions and declaration-bounded closure identities.
    pub semantics: crate::fingerprint::ResolvedSemanticProducts,
}

impl Registry {
    /// Export actual completed declarations for the standard-catalog generator.
    /// Runtime caches, owner identities and successful receiving proofs are excluded.
    /// # Errors
    /// Incomplete registry or immutable product construction failure.
    #[doc(hidden)]
    pub fn resolved_products(&self) -> Result<ResolvedRegistryProducts, SchemaError> {
        Ok(ResolvedRegistryProducts {
            package_id: self.package_id,
            fingerprint: self.fingerprint,
            relations: self.relations.clone(),
            obligations: self
                .obligations
                .iter()
                .map(|(k, v)| (*k, v.as_ref().clone()))
                .collect(),
            enums: self.enums.clone(),
            identities: self.identities.clone(),
            structures: self.structures.clone(),
            logical_types: self.logical_types.clone(),
            logical_storage: self.logical_storage.clone(),
            invariants: self.invariants.clone(),
            migrations: self.migrations.clone(),
            algorithms: self.algorithms.clone(),
            documents: self.documents.clone(),
            artifact_profiles: self.artifact_profiles.clone(),
            contracts: self.contract_arena()?.products(),
            semantics: crate::fingerprint::resolved_products(self)?,
        })
    }
}

impl ResolvedRegistryProducts {
    /// Consume the generated standard products into fresh runtime owners.
    pub(crate) fn hydrate(self, self_description: SchemaBatches) -> Result<Registry, SchemaError> {
        let relation_index = index_relations(&self.relations);
        let relation_by_id = self
            .relations
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id, i))
            .collect();
        let registry = Registry {
            implementations: crate::ImplementationCache::default(),
            package_id: self.package_id,
            fingerprint: self.fingerprint,
            relations: self.relations,
            obligations: self
                .obligations
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect(),
            relation_index,
            relation_by_id,
            enums: self.enums,
            identities: self.identities,
            structures: self.structures,
            logical_types: self.logical_types,
            logical_storage: self.logical_storage,
            invariants: self.invariants,
            migrations: self.migrations,
            algorithms: self.algorithms,
            documents: self.documents,
            artifact_profiles: self.artifact_profiles,
            self_description,
            contracts: Some(self.contracts.hydrate()),
        };
        crate::fingerprint::install_products(&registry, self.semantics)?;
        Ok(registry)
    }
}

/// Decode exactly one generated native self-description table, with full array checks.
pub(crate) fn batch(bytes: &[u8]) -> Result<arrow_array::RecordBatch, SchemaError> {
    let mut reader = arrow_ipc::reader::StreamReader::try_new(std::io::Cursor::new(bytes), None)
        .map_err(invalid)?;
    let batch = reader
        .next()
        .transpose()
        .map_err(invalid)?
        .ok_or_else(|| invalid("missing standard registry batch"))?;
    if reader.next().is_some() {
        return Err(invalid("extra standard registry batch"));
    }
    for array in batch.columns() {
        array.to_data().validate_full().map_err(invalid)?;
    }
    arrow_array::RecordBatch::try_new(batch.schema(), batch.columns().to_vec()).map_err(invalid)
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, SchemaError> {
    serde_json::from_str(text).map_err(invalid)
}
fn invalid(error: impl std::fmt::Display) -> SchemaError {
    crate::checks::invalid("standard registry products", error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn equivalent(source: &Registry, restored: &Registry) {
        let a = source.resolved_products().unwrap();
        let b = restored.resolved_products().unwrap();
        assert_eq!(a.package_id, b.package_id);
        assert_eq!(a.fingerprint, b.fingerprint);
        assert_eq!(a.relations, b.relations);
        assert_eq!(a.obligations, b.obligations);
        assert_eq!(a.enums, b.enums);
        assert_eq!(a.identities, b.identities);
        assert_eq!(a.structures, b.structures);
        assert_eq!(a.logical_types, b.logical_types);
        // Arrow's ordinary equality omits dictionary ordering at nested fields.
        assert_eq!(
            pse_columnar::native_field::canonical_json(&a.logical_storage).unwrap(),
            pse_columnar::native_field::canonical_json(&b.logical_storage).unwrap()
        );
        assert_eq!(a.invariants, b.invariants);
        assert_eq!(a.migrations, b.migrations);
        assert_eq!(a.algorithms, b.algorithms);
        assert_eq!(a.documents, b.documents);
        assert_eq!(a.artifact_profiles, b.artifact_profiles);
        assert_eq!(a.contracts.relations, b.contracts.relations);
        assert_eq!(a.contracts.enums, b.contracts.enums);
        assert_eq!(a.contracts.extensions, b.contracts.extensions);
        assert_eq!(
            pse_columnar::native_field::canonical_json(&a.contracts.schemas).unwrap(),
            pse_columnar::native_field::canonical_json(&b.contracts.schemas).unwrap()
        );
        assert_eq!(a.contracts.expectations, b.contracts.expectations);
        assert_eq!(a.semantics.descriptions, b.semantics.descriptions);
        assert_eq!(a.semantics.closures, b.semantics.closures);
        assert_eq!(a.semantics.invariants, b.semantics.invariants);
        assert_eq!(
            crate::fingerprint::registry(source.schema_batches()).unwrap(),
            crate::fingerprint::registry(restored.schema_batches()).unwrap()
        );
        assert!(!Arc::ptr_eq(
            &source.implementation_owner(),
            &restored.implementation_owner()
        ));
        assert!(!Arc::ptr_eq(
            source.contract_arena().unwrap(),
            restored.contract_arena().unwrap()
        ));
        let expected = Box::leak(
            a.contracts
                .expectations
                .values()
                .copied()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        for spec in source.relations() {
            let handle = restored
                .admit_contract(&source.contract(spec).unwrap())
                .unwrap();
            handle.require_generated(expected).unwrap();
            assert_eq!(
                crate::arrow::relation_schema(source, spec).unwrap(),
                crate::arrow::relation_schema(restored, restored.relation_by_id(spec.id).unwrap())
                    .unwrap()
            );
        }
    }

    #[test]
    fn completed_registry_products_hydrate_every_meaning_with_fresh_runtime_owners() {
        let source = crate::catalog::assemble().unwrap();
        let restored = source
            .resolved_products()
            .unwrap()
            .hydrate(source.schema_batches().to_vec())
            .unwrap();
        equivalent(&source, &restored);
    }

    #[test]
    fn empty_custom_registry_products_preserve_declared_scalars_and_fresh_owners() {
        let source = RegistryBuilder::new().build().unwrap();
        assert!(source.relations().is_empty());
        assert!(source.logical_type("f64").is_some());
        let restored = source
            .resolved_products()
            .unwrap()
            .hydrate(source.schema_batches().to_vec())
            .unwrap();
        equivalent(&source, &restored);
    }

    #[test]
    fn generated_standard_hydration_matches_independent_source_and_new_runtime_owners() {
        let source = crate::catalog::assemble().unwrap();
        let first = crate::generated::standard_registry::hydrate().unwrap();
        let second = crate::generated::standard_registry::hydrate().unwrap();
        equivalent(&source, &first);
        equivalent(&first, &second);
        assert!(Arc::ptr_eq(
            &crate::shared_registry().unwrap(),
            &crate::shared_registry().unwrap()
        ));
    }
    #[test]
    fn standard_registry_complete_construction_and_hydration_probe() {
        let mut samples = Vec::new();
        for _ in 0..5 {
            let start = std::time::Instant::now();
            let source = crate::catalog::assemble().unwrap();
            let source_us = start.elapsed().as_micros();
            let start = std::time::Instant::now();
            let hydrated = crate::generated::standard_registry::hydrate().unwrap();
            let hydrate_us = start.elapsed().as_micros();
            assert_eq!(source.fingerprint(), hydrated.fingerprint());
            assert_eq!(source.relations().len(), hydrated.relations().len());
            let source_owner = Arc::downgrade(&source.implementation_owner());
            let hydrated_owner = Arc::downgrade(&hydrated.implementation_owner());
            let start = std::time::Instant::now();
            drop(source);
            let source_drop_us = start.elapsed().as_micros();
            let start = std::time::Instant::now();
            drop(hydrated);
            let hydrate_drop_us = start.elapsed().as_micros();
            assert!(source_owner.upgrade().is_none() && hydrated_owner.upgrade().is_none());
            samples.push((source_us, hydrate_us, source_drop_us, hydrate_drop_us));
        }
        eprintln!(
            "B04 (complete source-construction, complete hydration, source drop, hydrated drop)_us current dev test binary, five fresh runtime owners, explicit native receiving checks and complete owner release: {samples:?}"
        );
    }

    #[test]
    fn native_product_receiving_rejects_corrupt_missing_and_extra_batches() {
        let source = crate::catalog::assemble().unwrap();
        let native = &source.schema_batches()[0].1;
        let encode = |copies| {
            let mut bytes = Vec::new();
            {
                let mut writer =
                    arrow_ipc::writer::StreamWriter::try_new(&mut bytes, native.schema().as_ref())
                        .unwrap();
                for _ in 0..copies {
                    writer.write(native).unwrap();
                }
                writer.finish().unwrap();
            }
            bytes
        };
        let bytes = encode(1);
        assert_eq!(
            crate::fingerprint::registry(&[(source.schema_batches()[0].0, batch(&bytes).unwrap())])
                .unwrap(),
            crate::fingerprint::registry(&[source.schema_batches()[0].clone()]).unwrap()
        );
        assert!(batch(&encode(0)).is_err());
        assert!(batch(&encode(2)).is_err());
        assert!(batch(&bytes[..bytes.len() / 2]).is_err());
        assert!(batch(b"not arrow").is_err());
    }
}
