// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable source ownership. Only the accounted loader can mint these wrappers.

use std::collections::BTreeMap;
use std::sync::Arc;

use pse_columnar::{AllocationLease, CancellationToken, MemoryPool};
use pse_schema::{
    Registry,
    model::{EnumSpec, RelationSpec},
};

use super::allocation::{Allocation, add, map_entry, mul};
use super::load::{DocumentBundle, contract};
use crate::authoring_driver::{DriverError, ParseBudget};

#[derive(Debug)]
struct BundleOwner {
    // Field order is deliberate: all trees/strings are destroyed before the lease.
    bundle: DocumentBundle,
    binding: Arc<RegistryBinding>,
    _lease: Arc<AllocationLease>,
}

/// Immutable validated loader result whose clones share one allocation and lease.
#[derive(Clone, Debug)]
pub struct OwnedDocumentBundle(Arc<BundleOwner>);

impl OwnedDocumentBundle {
    /// Borrow the original parsed bundle; mutation and detached ownership are private.
    pub fn bundle(&self) -> &DocumentBundle {
        &self.0.bundle
    }
}

#[derive(Debug)]
struct SetOwner {
    bundles: Vec<DocumentBundle>,
    parts: Vec<OwnedDocumentBundle>,
    _lease: Arc<AllocationLease>,
}

/// A contiguous immutable package inventory retaining every source allocation.
/// Cloning shares the inventory; it never deep-clones trees with an existing lease.
#[derive(Clone, Debug, Default)]
pub struct OwnedDocumentSet(Option<Arc<SetOwner>>);

impl OwnedDocumentSet {
    /// Whether both handles retain the same immutable source inventory allocation.
    /// Distinct owners require exact source/declaration comparison by the caller.
    pub fn same_owner(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    /// Borrow all complete, original loader results in supplied package order.
    pub fn bundles(&self) -> &[DocumentBundle] {
        self.0
            .as_ref()
            .map_or(&[], |owner| owner.bundles.as_slice())
    }

    /// Check the actual declarations retained by the loader before a trusted
    /// consumer skips reparsing. Full values, including resolved enum members,
    /// must agree; digest equality never establishes this agreement.
    ///
    /// # Errors
    /// Refuses any changed relation, enum or used document declaration.
    pub fn validate_registry(&self, registry: &Registry) -> Result<(), DriverError> {
        let Some(owner) = &self.0 else {
            return Ok(());
        };
        for part in &owner.parts {
            part.0.binding.validate(registry)?;
        }
        for bundle in &owner.bundles {
            for document in &bundle.documents {
                if super::load::select(registry, &document.path)? != &document.declaration {
                    return Err(contract(
                        None,
                        "owned document declaration differs from the consumer registry",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Reuse local admission only with its actual immutable native predicate owner.
    /// Changed owners re-admit values; registry equality alone is insufficient.
    /// # Errors
    /// Changed declarations, actual native predicate refusal, cancellation or resources.
    pub fn validate_context(
        &self,
        registry: &Registry,
        validation: &pse_relations::validate::ValidationContext,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<(), DriverError> {
        self.validate_registry(registry)?;
        for bundle in self.bundles() {
            for document in &bundle.documents {
                cancel.checkpoint()?;
                if document.modeling_validation_required(registry, validation)? {
                    let mut projection = Allocation::new(pool, cancel);
                    projection.grow(super::allocation::modeling_batch_extent(document)?)?;
                    document.modeling_batch(registry, validation, cancel)?;
                }
            }
            for (id, batch) in &bundle.batches {
                let spec = registry
                    .relation_by_id(*id)
                    .ok_or_else(|| contract(None, "retained source relation missing"))?;
                batch.validate_context(registry, spec, validation, cancel)?;
            }
        }
        Ok(())
    }

    /// Retain package owners without copying source bytes or parsed trees.
    ///
    /// # Errors
    /// Cancellation or insufficient shared memory for the new inventory.
    pub fn try_from_bundles(
        parts: Vec<OwnedDocumentBundle>,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<Self, DriverError> {
        cancel.checkpoint()?;
        let mut allocation = Allocation::new(pool, cancel);
        allocation.grow(add(
            size_of::<SetOwner>() + size_of::<AllocationLease>() + 4 * size_of::<usize>(),
            mul(
                parts.len(),
                size_of::<DocumentBundle>() + size_of::<OwnedDocumentBundle>(),
            )?,
        )?)?;
        let bundles = parts.iter().map(|part| part.bundle().clone()).collect();
        Ok(Self(Some(Arc::new(SetOwner {
            bundles,
            parts,
            _lease: allocation.finish(),
        }))))
    }
}

impl OwnedDocumentSet {
    /// Apply exact source edits, parsing only changed documents and reusing all
    /// unchanged parser owners. Hydration is recomputed against the complete package.
    /// # Errors
    /// Stale/duplicate edit, source contract failure, cancellation or reservation refusal.
    pub fn edit(
        &self,
        edits: &[super::DocumentEdit],
        registry: &Registry,
        budget: ParseBudget,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
        validation: &pse_relations::validate::ValidationContext,
    ) -> Result<Self, DriverError> {
        self.validate_registry(registry)?;
        let Some(owner) = &self.0 else {
            return if edits.is_empty() {
                Ok(Self::default())
            } else {
                Err(contract(None, "source edit has no original document"))
            };
        };
        let work = pse_columnar::MemoryConsumer::new("authoring:source-edits").register(pool);
        work.try_grow(crate::authoring_driver::work::sources(self.bundles())?)?;
        for edit in edits {
            work.try_grow(add(edit.before.len(), edit.after.len())?)?;
        }
        let mut seen = std::collections::BTreeSet::new();
        for edit in edits {
            if !seen.insert(edit.document_id)
                || !owner.bundles.iter().any(|bundle| {
                    bundle.documents.iter().any(|document| {
                        document.id == edit.document_id && document.path == edit.path
                    })
                })
            {
                return Err(contract(
                    None,
                    "source edit is duplicate or outside the document inventory",
                ));
            }
        }
        let mut parts = Vec::with_capacity(owner.parts.len());
        for part in &owner.parts {
            cancel.checkpoint()?;
            let own = edits
                .iter()
                .filter(|edit| {
                    part.bundle()
                        .documents
                        .iter()
                        .any(|document| document.id == edit.document_id)
                })
                .cloned()
                .collect::<Vec<_>>();
            if own.is_empty() {
                parts.push(part.clone());
                continue;
            }
            let mut allocation = Allocation::new(pool, cancel);
            allocation.grow(256)?;
            allocation.grow(crate::authoring_driver::work::sources(
                std::slice::from_ref(part.bundle()),
            )?)?;
            let mut texts = part
                .bundle()
                .documents
                .iter()
                .map(|document| (document.path.clone(), document.bytes().to_vec()))
                .collect();
            super::apply_edits(&mut texts, &own)?;
            let mut bundle = super::load::load_reusing(
                texts,
                Some(part.bundle()),
                registry,
                budget,
                Some(&mut allocation),
                validation,
            )?;
            bundle.retain_columns(pool, cancel)?;
            let (lease, _) = finalize_bundle(&mut bundle, allocation, 0)?;
            parts.push(OwnedDocumentBundle(Arc::new(BundleOwner {
                bundle,
                binding: Arc::clone(&part.0.binding),
                _lease: lease,
            })));
        }
        Self::try_from_bundles(parts, pool, cancel)
    }
}

/// Transfer admitted capacity into source-local lifetimes without a release/reacquire gap.
fn finalize_bundle(
    bundle: &mut DocumentBundle,
    mut allocation: Allocation<'_>,
    registry_bytes: usize,
) -> Result<(Arc<AllocationLease>, Option<Arc<AllocationLease>>), DriverError> {
    let metadata = add(
        super::allocation::bundle_retained(bundle)?,
        size_of::<BundleOwner>() + 256,
    )?;
    let registry_bytes = if registry_bytes == 0 {
        0
    } else {
        add(registry_bytes, 256)?
    };
    allocation.retain(0, add(metadata, registry_bytes)?)?;
    let lease = allocation.finish();
    if registry_bytes == 0 {
        Ok((bundle.attach_lease(lease)?, None))
    } else {
        let mut owners = lease
            .partition(&[registry_bytes, metadata])
            .map_err(|_| contract(None, "registry admission lease must be unique"))?
            .into_iter();
        let binding = owners
            .next()
            .ok_or_else(|| contract(None, "registry owner absent"))?;
        let source = owners
            .next()
            .ok_or_else(|| contract(None, "source owner absent"))?;
        Ok((bundle.attach_lease(source)?, Some(binding)))
    }
}

pub(super) fn retain_bundle(
    bundle: &DocumentBundle,
    registry: &Registry,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<OwnedDocumentBundle, DriverError> {
    let mut allocation = Allocation::new(pool, cancel);
    allocation.grow(add(
        super::allocation::bundle_retained(bundle)?,
        add(
            super::allocation::registry_extent(registry)?,
            size_of::<BundleOwner>() + 512,
        )?,
    )?)?;
    for document in &bundle.documents {
        if super::load::select(registry, &document.path)? != &document.declaration {
            return Err(contract(
                None,
                "parsed source declaration differs from the registry",
            ));
        }
        cancel.checkpoint()?;
        if document.modeling_validation_required(registry, validation)? {
            let mut projection = Allocation::new(pool, cancel);
            projection.grow(super::allocation::modeling_batch_extent(document)?)?;
            document.modeling_batch(registry, validation, cancel)?;
        }
    }
    for (relation, batch) in &bundle.batches {
        let spec = registry
            .relation_by_id(*relation)
            .ok_or_else(|| contract(None, "retained source relation missing"))?;
        batch.validate_context(registry, spec, validation, cancel)?;
    }
    // DocumentBundle has a private constructor and immutable shared data. This
    // retains the actual parsed value, not an arbitrary caller-created DTO.
    let mut bundle = bundle.clone();
    bundle.retain_columns(pool, cancel)?;
    let mut binding = RegistryBinding {
        relations: registry.relations().to_vec(),
        enums: registry.enums().to_vec(),
        _lease: None,
    };
    let (lease, binding_lease) = finalize_bundle(
        &mut bundle,
        allocation,
        super::allocation::registry_extent(registry)?,
    )?;
    binding._lease = binding_lease;
    Ok(OwnedDocumentBundle(Arc::new(BundleOwner {
        bundle,
        binding: Arc::new(binding),
        _lease: lease,
    })))
}

/// Load borrowed package document bytes after reserving copies and all expansion phases:
/// text documents are parsed, data documents decoded (ADR-0125). Original bytes, parser
/// values, decoded rows, paths, generated rows and their leases live together. Each actual
/// source is checked and reserved independently, even if a custom iterator clone yields a
/// different inventory.
///
/// # Errors
/// Refuses invalid paths, duplicates, text that is not UTF-8, parser/decoder/schema/identity
/// failures, cancellation or a shared resource limit before the associated construction.
pub fn load_package_sources_owned<'a>(
    sources: impl Iterator<Item = (&'a str, &'a [u8])> + Clone,
    registry: &Registry,
    budget: ParseBudget,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<OwnedDocumentBundle, DriverError> {
    let mut allocation = Allocation::new(pool, cancel);
    allocation
        .grow(size_of::<BundleOwner>() + size_of::<AllocationLease>() + 4 * size_of::<usize>())?;
    let mut texts = BTreeMap::new();
    let mut total = 0;
    for (path, bytes) in sources {
        cancel.checkpoint()?;
        total = add(total, bytes.len())?;
        if u64::try_from(total).unwrap_or(u64::MAX) > budget.max_bytes {
            return Err(DriverError::Authoring(
                pse_authoring::AuthoringError::Budget {
                    limit: "package bytes",
                    allowed: budget.max_bytes,
                    needed: u64::try_from(total).unwrap_or(u64::MAX),
                },
            ));
        }
        allocation.grow(add(
            map_entry::<String, Vec<u8>>(),
            add(path.len(), bytes.len())?,
        )?)?;
        super::load::select(registry, path)?;
        if texts.insert(path.to_owned(), bytes.to_vec()).is_some() {
            return Err(contract(None, "duplicate package-relative source path"));
        }
    }
    allocation.grow(add(super::allocation::registry_extent(registry)?, 256)?)?;
    let mut binding = RegistryBinding {
        relations: registry.relations().to_vec(),
        enums: registry.enums().to_vec(),
        _lease: None,
    };
    let mut bundle =
        super::load::load_inventory(texts, registry, budget, Some(&mut allocation), validation)?;
    bundle.retain_columns(pool, cancel)?;
    cancel.checkpoint()?;
    let (lease, binding_lease) = finalize_bundle(
        &mut bundle,
        allocation,
        super::allocation::registry_extent(registry)?,
    )?;
    binding._lease = binding_lease;
    Ok(OwnedDocumentBundle(Arc::new(BundleOwner {
        bundle,
        binding: Arc::new(binding),
        _lease: lease,
    })))
}

/// Load caller-owned package documents by path while retaining independently reserved
/// immutable copies.
///
/// # Errors
/// The same checked failures as [`load_package_sources_owned`].
pub fn load_package_documents_owned(
    sources: &BTreeMap<String, Vec<u8>>,
    registry: &Registry,
    budget: ParseBudget,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<OwnedDocumentBundle, DriverError> {
    load_package_sources_owned(
        sources
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice())),
        registry,
        budget,
        pool,
        cancel,
        validation,
    )
}

#[derive(Debug)]
struct RegistryBinding {
    relations: Vec<RelationSpec>,
    enums: Vec<EnumSpec>,
    _lease: Option<Arc<AllocationLease>>,
}
impl RegistryBinding {
    fn validate(&self, registry: &Registry) -> Result<(), DriverError> {
        if self.relations != registry.relations() || self.enums != registry.enums() {
            return Err(contract(
                None,
                "owned source relation or enum declarations differ from the consumer registry",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_validation(registry: &Registry) -> Arc<pse_relations::validate::ValidationContext> {
        // This source-only fixture deliberately captures its fixed native session state.
        Arc::new(pse_relations::validate::ValidationContext::new(
            registry,
            pse_engine::validation::NativeValidation(
                datafusion::prelude::SessionContext::new().state(),
            ),
        ))
    }

    #[test]
    fn immutable_modeling_rows_reuse_native_owner_and_recheck_new_context()
    -> Result<(), DriverError> {
        let registry = pse_schema::registry().map_err(pse_relations::RelationError::from)?;
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let cancel = CancellationToken::new();
        let validation = fixture_validation(registry);
        let sources = BTreeMap::from([
            (
                "package.toml".to_owned(),
                include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml")
                    .to_vec(),
            ),
            (
                "models/one.pse".to_owned(),
                b"package library {def Root {var x:Scalar;eq residual:x==2;}}".to_vec(),
            ),
        ]);
        let part = load_package_documents_owned(
            &sources,
            registry,
            ParseBudget::default(),
            &pool,
            &cancel,
            &validation,
        )?;
        let owned = OwnedDocumentSet::try_from_bundles(vec![part], &pool, &cancel)?;
        let spec = registry
            .relation_by_id(pse_relations::generated::authored::modeling_declarations::RELATION_ID)
            .unwrap();
        let prepared = validation.relation(registry, spec)?;
        let count = prepared.evaluation_count();
        assert!(
            count > 0,
            "generated construction actually evaluated native field obligations"
        );
        let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        owned.validate_context(registry, &validation, &tiny, &cancel)?;
        assert_eq!(prepared.evaluation_count(), count);
        assert_eq!(
            tiny.reserved(),
            0,
            "same-owner reuse creates no temporary modeling columns"
        );
        let different = fixture_validation(registry);
        let different_prepared = different.relation(registry, spec)?;
        assert!(
            owned
                .validate_context(registry, &different, &tiny, &cancel)
                .is_err()
        );
        assert_eq!(
            different_prepared.evaluation_count(),
            0,
            "finite projection reservation precedes new-owner evaluation"
        );
        owned.validate_context(registry, &different, &pool, &cancel)?;
        let new_count = different_prepared.evaluation_count();
        assert!(new_count > 0);
        owned.validate_context(registry, &different, &tiny, &cancel)?;
        assert_eq!(different_prepared.evaluation_count(), new_count);
        cancel.cancel();
        assert!(
            owned
                .validate_context(registry, &different, &tiny, &cancel)
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn registry_binding_compares_fields_even_when_identity_and_fingerprint_match() {
        let registry = pse_schema::registry().unwrap();
        let mut binding = RegistryBinding {
            relations: registry.relations().to_vec(),
            enums: registry.enums().to_vec(),
            _lease: None,
        };
        binding.validate(registry).unwrap();
        let old_id = binding.relations[0].id;
        let old_hash = binding.relations[0].fingerprint;
        binding.relations[0].columns[0] = binding.relations[0].columns[0]
            .clone()
            .with_doc("different actual declaration");
        assert_eq!(binding.relations[0].id, old_id);
        assert_eq!(binding.relations[0].fingerprint, old_hash);
        assert!(binding.validate(registry).is_err());
    }
    #[test]
    fn shared_typed_rows_retain_their_source_allocation() -> Result<(), DriverError> {
        let registry = pse_schema::registry().map_err(pse_relations::RelationError::from)?;
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let cancel = CancellationToken::new();
        let sources = BTreeMap::from([
            (
                "package.toml".to_owned(),
                include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml")
                    .to_vec(),
            ),
            ("models/one.pse".to_owned(), b"package library {}".to_vec()),
        ]);
        let owned = load_package_documents_owned(
            &sources,
            registry,
            ParseBudget::default(),
            &pool,
            &cancel,
            &fixture_validation(registry),
        )?;
        let rows = owned
            .bundle()
            .documents
            .iter()
            .find_map(|document| document.modeling_rows())
            .unwrap()
            .clone();
        drop(owned);
        assert_eq!(rows.len(), 1);
        assert!(pool.reserved() > 0);
        drop(rows);
        assert_eq!(pool.reserved(), 0);
        Ok(())
    }
    #[test]
    fn editing_one_document_reuses_other_parser_owners() -> Result<(), DriverError> {
        let registry = pse_schema::registry().map_err(pse_relations::RelationError::from)?;
        let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20));
        let cancel = CancellationToken::new();
        let texts = BTreeMap::from([
            ("package.toml".to_owned(), include_bytes!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml").to_vec()),
            ("materials/quantity-kinds.yaml".to_owned(), include_bytes!("../../../../../tests/fixtures/packages/minimal_explicit/materials/quantity-kinds.yaml").to_vec()),
            ("models/library.pse".to_owned(), b"@id(\"00000000000000000000000000000031\") package library { @id(\"00000000000000000000000000000032\") enum Choice { @id(\"00000000000000000000000000000033\") one, @id(\"00000000000000000000000000000034\") two } }".to_vec()),
        ]);
        let part = load_package_documents_owned(
            &texts,
            registry,
            ParseBudget::default(),
            &budget,
            &cancel,
            &fixture_validation(registry),
        )?;
        let original = OwnedDocumentSet::try_from_bundles(vec![part], &budget, &cancel)?;
        let source = original.bundles()[0]
            .documents
            .iter()
            .find(|document| document.path == "materials/quantity-kinds.yaml")
            .ok_or_else(|| contract(None, "fixture source absent"))?;
        let edited = original.edit(
            &[super::super::DocumentEdit {
                document_id: source.id,
                path: source.path.clone(),
                before: source.text().unwrap_or_default().to_owned(),
                after: source
                    .text()
                    .unwrap_or_default()
                    .replace("name: probe", "name: changed"),
            }],
            registry,
            ParseBudget::default(),
            &budget,
            &cancel,
            &fixture_validation(registry),
        )?;
        assert!(!original.same_owner(&edited));
        assert!(original.same_owner(&original.clone()));
        for prior in &original.bundles()[0].documents {
            let next = edited.bundles()[0]
                .documents
                .iter()
                .find(|document| document.id == prior.id)
                .ok_or_else(|| contract(None, "edited fixture document absent"))?;
            assert_eq!(
                Arc::ptr_eq(&prior.syntax, &next.syntax),
                prior.path != "materials/quantity-kinds.yaml"
            );
            if let Some(rows) = prior.modeling_rows() {
                assert!(Arc::ptr_eq(rows, next.modeling_rows().unwrap()));
                assert!(!prior.batches.contains_key(
                    &pse_relations::generated::authored::modeling_declarations::RELATION_ID
                ));
                assert!(!next.batches.contains_key(
                    &pse_relations::generated::authored::modeling_declarations::RELATION_ID
                ));
            }
        }
        let rows = pse_relations::generated::reference::quantity_kinds::View::from_checked(
            &edited.bundles()[0].batches
                [&pse_relations::generated::reference::quantity_kinds::RELATION_ID],
        )?
        .rows()?;
        assert_eq!(rows[0].name, "changed");
        assert!(
            !edited.bundles()[0].batches.contains_key(
                &pse_relations::generated::authored::modeling_declarations::RELATION_ID
            )
        );
        let retained = budget.reserved();
        edited.validate_context(registry, &fixture_validation(registry), &budget, &cancel)?;
        assert_eq!(
            budget.reserved(),
            retained,
            "native projection is temporary"
        );
        let refused: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        assert!(
            edited
                .validate_context(registry, &fixture_validation(registry), &refused, &cancel)
                .is_err()
        );
        assert_eq!(refused.reserved(), 0);
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(
            edited
                .validate_context(registry, &fixture_validation(registry), &budget, &cancelled)
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn successive_edits_release_ancestors_but_shared_payloads_keep_their_own_charge()
    -> Result<(), DriverError> {
        use datafusion::arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::{DataType, Field, Schema},
        };
        let registry = pse_schema::registry().map_err(pse_relations::RelationError::from)?;
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20));
        let cancel = CancellationToken::new();
        let validation = fixture_validation(registry);
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![11, 22, 33]))],
        )
        .unwrap();
        let mut parquet = Vec::new();
        let mut writer = parquet::arrow::ArrowWriter::try_new(&mut parquet, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        let model = format!(
            "// {}\npackage library {{}}",
            "unchanged source ".repeat(4096)
        );
        let sources = BTreeMap::from([
            ("package.toml".to_owned(), include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml").to_vec()),
            ("materials/quantity-kinds.yaml".to_owned(), include_bytes!("../../../../../tests/fixtures/packages/minimal_named/materials/quantity-kinds.yaml").to_vec()),
            ("models/library.pse".to_owned(), model.as_bytes().to_vec()),
            ("data/bank.parquet".to_owned(), parquet.clone()),
        ]);
        let part = load_package_documents_owned(
            &sources,
            registry,
            ParseBudget::default(),
            &pool,
            &cancel,
            &validation,
        )?;
        let mut current = OwnedDocumentSet::try_from_bundles(vec![part], &pool, &cancel)?;
        let doc = current.bundles()[0]
            .documents
            .iter()
            .find(|d| d.path == "models/library.pse")
            .unwrap();
        let rows = doc.modeling_rows().unwrap().clone();
        let text = match &doc.content {
            super::super::Content::Modeling { text, .. } => Arc::clone(text),
            _ => panic!("expected modeling text in the library source fixture"),
        };
        let syntax = Arc::clone(&doc.syntax);
        let data = current.bundles()[0]
            .documents
            .iter()
            .find(|d| d.path == "data/bank.parquet")
            .unwrap();
        let decoded = data.data().unwrap().clone();
        let binary = match &data.content {
            super::super::Content::Data { bytes, .. } => bytes.clone(),
            _ => panic!("expected Parquet bytes in the data source fixture"),
        };
        assert!(decoded.is_owned());
        let mut plateau = None;
        for n in 0..12 {
            let old_owner = Arc::downgrade(&current.0.as_ref().unwrap().parts[0].0);
            let changed = current.bundles()[0]
                .documents
                .iter()
                .find(|d| d.path == "materials/quantity-kinds.yaml")
                .unwrap();
            let before = changed.text().unwrap().to_owned();
            let old_name = if n == 0 {
                "name: probe".to_owned()
            } else {
                format!("name: edited{:02}", n - 1)
            };
            let after = before.replace(&old_name, &format!("name: edited{n:02}"));
            assert_ne!(before, after);
            current = current.edit(
                &[super::super::DocumentEdit {
                    document_id: changed.id,
                    path: changed.path.clone(),
                    before,
                    after,
                }],
                registry,
                ParseBudget::default(),
                &pool,
                &cancel,
                &validation,
            )?;
            assert!(
                old_owner.upgrade().is_none(),
                "an edited bundle retained an ancestor"
            );
            let shared = current.bundles()[0]
                .documents
                .iter()
                .find(|d| d.path == "models/library.pse")
                .unwrap();
            assert!(Arc::ptr_eq(&rows, shared.modeling_rows().unwrap()));
            assert!(Arc::ptr_eq(&syntax, &shared.syntax));
            let shared_data = current.bundles()[0]
                .documents
                .iter()
                .find(|d| d.path == "data/bank.parquet")
                .unwrap();
            assert!(Arc::ptr_eq(&decoded, shared_data.data().unwrap()));
            if let Some(retained) = plateau {
                assert_eq!(pool.reserved(), retained);
            } else {
                plateau = Some(pool.reserved());
            }
        }
        let complete = pool.reserved();
        drop(current);
        assert!(
            pool.reserved() > 0 && pool.reserved() < complete,
            "escaped payloads must retain only their own charge"
        );
        assert_eq!(text.as_str(), model.as_str());
        assert_eq!(rows.len(), 1);
        assert_eq!(&binary[..], parquet.as_slice());
        assert_eq!(decoded.rows.rows(), 3);
        drop(rows);
        drop(text);
        drop(syntax);
        assert!(
            pool.reserved() > 0,
            "decoded data and exact bytes still have owners"
        );
        drop(decoded);
        assert!(
            pool.reserved() > 0,
            "an escaped byte buffer retains its owner"
        );
        drop(binary);
        assert_eq!(pool.reserved(), 0);
        Ok(())
    }
}
