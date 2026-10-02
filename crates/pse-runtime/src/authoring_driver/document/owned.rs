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
    _parent: Option<OwnedDocumentBundle>,
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

    /// Re-admit retained native values in the caller's exact current validation context.
    /// Registry/declaration equality alone cannot establish native predicate agreement.
    /// # Errors
    /// Changed declarations, actual native predicate refusal, cancellation or resources.
    pub fn validate_context(
        &self,
        registry: &Registry,
        validation: &pse_relations::validate::ValidationContext,
        cancel: &CancellationToken,
    ) -> Result<(), DriverError> {
        self.validate_registry(registry)?;
        for bundle in self.bundles() {
            for (id, batch) in &bundle.batches {
                let spec = registry
                    .relation_by_id(*id)
                    .ok_or_else(|| contract(None, "retained source relation missing"))?;
                pse_relations::columnar::FieldCheckedBatch::admit(
                    registry,
                    spec,
                    batch.batch().clone(),
                    validation,
                    cancel,
                )?;
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
            let lease = allocation.finish();
            bundle.attach_lease(Arc::clone(&lease));
            parts.push(OwnedDocumentBundle(Arc::new(BundleOwner {
                bundle,
                binding: Arc::clone(&part.0.binding),
                _parent: Some(part.clone()),
                _lease: lease,
            })));
        }
        Self::try_from_bundles(parts, pool, cancel)
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
        super::allocation::registry_extent(registry)?,
    )?)?;
    for document in &bundle.documents {
        if super::load::select(registry, &document.path)? != &document.declaration {
            return Err(contract(
                None,
                "parsed source declaration differs from the registry",
            ));
        }
    }
    for (relation, batch) in &bundle.batches {
        let spec = registry
            .relation_by_id(*relation)
            .ok_or_else(|| contract(None, "retained source relation missing"))?;
        pse_relations::columnar::FieldCheckedBatch::admit(
            registry,
            spec,
            batch.batch().clone(),
            validation,
            cancel,
        )?;
    }
    // DocumentBundle has a private constructor and immutable shared data. This
    // retains the actual parsed value, not an arbitrary caller-created DTO.
    let mut bundle = bundle.clone();
    bundle.retain_columns(pool, cancel)?;
    let lease = allocation.finish();
    bundle.attach_lease(Arc::clone(&lease));
    Ok(OwnedDocumentBundle(Arc::new(BundleOwner {
        bundle,
        binding: Arc::new(RegistryBinding {
            relations: registry.relations().to_vec(),
            enums: registry.enums().to_vec(),
        }),
        _parent: None,
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
    allocation.grow(super::allocation::registry_extent(registry)?)?;
    let binding = Arc::new(RegistryBinding {
        relations: registry.relations().to_vec(),
        enums: registry.enums().to_vec(),
    });
    let mut bundle =
        super::load::load_inventory(texts, registry, budget, Some(&mut allocation), validation)?;
    bundle.retain_columns(pool, cancel)?;
    cancel.checkpoint()?;
    allocation.retain(
        0,
        add(
            super::allocation::bundle_retained(&bundle)?,
            add(
                super::allocation::registry_extent(registry)?,
                size_of::<BundleOwner>() + size_of::<AllocationLease>() + 4 * size_of::<usize>(),
            )?,
        )?,
    )?;
    let lease = allocation.finish();
    bundle.attach_lease(Arc::clone(&lease));
    Ok(OwnedDocumentBundle(Arc::new(BundleOwner {
        bundle,
        binding,
        _parent: None,
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
    fn registry_binding_compares_fields_even_when_identity_and_fingerprint_match() {
        let registry = pse_schema::registry().unwrap();
        let mut binding = RegistryBinding {
            relations: registry.relations().to_vec(),
            enums: registry.enums().to_vec(),
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
        }
        let rows = pse_relations::generated::reference::quantity_kinds::View::from_checked(
            &edited.bundles()[0].batches
                [&pse_relations::generated::reference::quantity_kinds::RELATION_ID],
        )?
        .rows()?;
        assert_eq!(rows[0].name, "changed");
        Ok(())
    }
}
