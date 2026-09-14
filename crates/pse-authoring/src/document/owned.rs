// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable source ownership. Only the accounted loader can mint these wrappers.

use std::collections::BTreeMap;
use std::sync::Arc;

use pse_ids::{CancellationToken, MemoryReserver, ReservationLease};
use pse_schema::{
    Registry,
    model::{EnumSpec, RelationSpec},
};

use super::allocation::{Allocation, add, map_entry, mul};
use super::load::{DocumentBundle, contract};
use crate::{AuthoringError, ParseBudget};

#[derive(Debug)]
struct BundleOwner {
    // Field order is deliberate: all trees/strings are destroyed before the lease.
    bundle: DocumentBundle,
    binding: RegistryBinding,
    lease: Arc<ReservationLease>,
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

#[derive(Debug, Default)]
struct SetOwner {
    bundles: Vec<DocumentBundle>,
    bindings: Vec<RegistryBinding>,
    _leases: Vec<Arc<ReservationLease>>,
}

/// A contiguous immutable package inventory retaining every source allocation.
/// Cloning shares the inventory; it never deep-clones trees with an existing lease.
#[derive(Clone, Debug, Default)]
pub struct OwnedDocumentSet(Option<Arc<SetOwner>>);

impl OwnedDocumentSet {
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
    pub fn validate_registry(&self, registry: &Registry) -> Result<(), AuthoringError> {
        let Some(owner) = &self.0 else {
            return Ok(());
        };
        for binding in &owner.bindings {
            binding.validate(registry)?;
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

    /// Consume unique package owners without copying any source or parsed tree.
    ///
    /// # Errors
    /// Refuses shared input owners, cancellation or insufficient shared memory for
    /// the new contiguous inventory. All consumed reservations release on failure.
    pub fn try_from_bundles(
        parts: Vec<OwnedDocumentBundle>,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<Self, AuthoringError> {
        cancel.checkpoint()?;
        if parts.iter().any(|part| Arc::strong_count(&part.0) != 1) {
            return Err(contract(
                None,
                "assembling a document set requires unique package owners",
            ));
        }
        let mut allocation = Allocation::new(reserver, cancel);
        allocation.grow(add(
            size_of::<SetOwner>() + size_of::<ReservationLease>() + 4 * size_of::<usize>(),
            add(
                mul(
                    parts.len(),
                    size_of::<DocumentBundle>() + size_of::<RegistryBinding>(),
                )?,
                mul(add(parts.len(), 1)?, size_of::<Arc<ReservationLease>>())?,
            )?,
        )?)?;
        let mut bundles = Vec::with_capacity(parts.len());
        let mut bindings = Vec::with_capacity(parts.len());
        let mut leases = Vec::with_capacity(add(parts.len(), 1)?);
        for part in parts {
            cancel.checkpoint()?;
            let owner = Arc::try_unwrap(part.0)
                .map_err(|_| contract(None, "package ownership became shared"))?;
            bundles.push(owner.bundle);
            bindings.push(owner.binding);
            leases.push(owner.lease);
        }
        leases.push(allocation.finish());
        Ok(Self(Some(Arc::new(SetOwner {
            bundles,
            bindings,
            _leases: leases,
        }))))
    }
}

/// Parse borrowed UTF-8 source bytes after reserving copies and all expansion phases.
/// Original source bytes, parser values, paths, generated rows and their leases live
/// together. Each actual source is checked and reserved independently, even if
/// a custom iterator clone yields a different inventory.
///
/// # Errors
/// Refuses invalid paths, duplicates, UTF-8, parser/schema/identity failures,
/// cancellation or a shared resource limit before the associated construction.
pub fn load_package_sources_owned<'a>(
    sources: impl Iterator<Item = (&'a str, &'a [u8])> + Clone,
    registry: &Registry,
    budget: ParseBudget,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentBundle, AuthoringError> {
    let mut allocation = Allocation::new(reserver, cancel);
    allocation
        .grow(size_of::<BundleOwner>() + size_of::<ReservationLease>() + 4 * size_of::<usize>())?;
    let mut texts = BTreeMap::new();
    let mut total = 0;
    for (path, bytes) in sources {
        cancel.checkpoint()?;
        total = add(total, bytes.len())?;
        if u64::try_from(total).unwrap_or(u64::MAX) > budget.max_bytes {
            return Err(AuthoringError::Budget {
                limit: "package bytes",
                allowed: budget.max_bytes,
                needed: u64::try_from(total).unwrap_or(u64::MAX),
            });
        }
        allocation.grow(add(
            map_entry::<String, String>(),
            add(path.len(), bytes.len())?,
        )?)?;
        super::load::select(registry, path)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| contract(None, "document source must be UTF-8"))?;
        if texts.insert(path.to_owned(), text.to_owned()).is_some() {
            return Err(contract(None, "duplicate package-relative source path"));
        }
    }
    allocation.grow(super::allocation::registry_extent(registry)?)?;
    let binding = RegistryBinding {
        relations: registry.relations().to_vec(),
        enums: registry.enums().to_vec(),
    };
    let bundle = super::load::load_inventory(texts, registry, budget, Some(&mut allocation))?;
    cancel.checkpoint()?;
    allocation.retain(
        0,
        add(
            super::allocation::bundle_retained(&bundle)?,
            add(
                super::allocation::registry_extent(registry)?,
                size_of::<BundleOwner>() + size_of::<ReservationLease>() + 4 * size_of::<usize>(),
            )?,
        )?,
    )?;
    Ok(OwnedDocumentBundle(Arc::new(BundleOwner {
        bundle,
        binding,
        lease: allocation.finish(),
    })))
}

/// Load caller-owned text while retaining independently reserved immutable copies.
///
/// # Errors
/// The same checked failures as [`load_package_sources_owned`].
pub fn load_package_texts_owned(
    texts: &BTreeMap<String, String>,
    registry: &Registry,
    budget: ParseBudget,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentBundle, AuthoringError> {
    load_package_sources_owned(
        texts
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_bytes())),
        registry,
        budget,
        reserver,
        cancel,
    )
}

#[derive(Debug)]
struct RegistryBinding {
    relations: Vec<RelationSpec>,
    enums: Vec<EnumSpec>,
}
impl RegistryBinding {
    fn validate(&self, registry: &Registry) -> Result<(), AuthoringError> {
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
        binding.relations[0].columns[0].doc = "different actual declaration";
        assert_eq!(binding.relations[0].id, old_id);
        assert_eq!(binding.relations[0].fingerprint, old_hash);
        assert!(binding.validate(registry).is_err());
    }
}
