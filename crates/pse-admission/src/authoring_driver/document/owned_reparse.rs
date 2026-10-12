// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Accounted admission of caller DTO source inventories.
use super::{DocumentBundle, OwnedDocumentSet};
use crate::authoring_driver::DriverError;

use pse_columnar::{CancellationToken, MemoryPool};
use pse_schema::Registry;

/// Retain immutable private loader results without reparsing original bytes.
/// # Errors
/// Source/schema/identity failures, cancellation or an unavailable shared reservation.
pub fn load_bundles_owned(
    bundles: &[DocumentBundle],
    registry: &Registry,
    pool: &std::sync::Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<OwnedDocumentSet, DriverError> {
    let work = pse_columnar::MemoryConsumer::new("authoring:bundle-inventory").register(pool);
    work.try_grow(crate::authoring_driver::work::mul(bundles.len(), 256)?)?;
    let mut parts = Vec::with_capacity(bundles.len());
    for bundle in bundles {
        cancel.checkpoint()?;
        parts.push(super::owned::retain_bundle(
            bundle, registry, pool, cancel, validation,
        )?);
    }
    OwnedDocumentSet::try_from_bundles(parts, pool, cancel)
}

/// Checked temporary parse/binding/staging extent from actual retained sources and cells.
/// This is allocation accounting only; callers must separately validate the inventory.
/// # Errors
/// Arithmetic overflow in the recursive allocation forecast.
pub fn workspace_extent(bundles: &OwnedDocumentSet) -> Result<usize, DriverError> {
    crate::authoring_driver::work::sources(bundles.bundles())
}

/// Exact encoded data carried by parsed reload. This is an edit intent, never
/// an admitted decoded table. Its immutable bytes retain their allocation owner.
#[derive(Debug)]
pub struct EncodedReloadDocument {
    pub(super) document: pse_ids::SemanticId,
    pub(super) package: pse_ids::SemanticId,
    pub(super) path: String,
    pub(super) content_hash: pse_ids::ContentHash,
    pub(super) bytes: bytes::Bytes,
    pub(super) lease: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
}
impl EncodedReloadDocument {
    /// Stable document identity supplied by the exact package/path loader.
    pub fn id(&self) -> pse_ids::SemanticId {
        self.document
    }
    /// Exact encoded bytes, still requiring data admission when newly consumed.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Content integrity identity of those exact bytes.
    pub fn content_hash(&self) -> pse_ids::ContentHash {
        self.content_hash
    }
    /// The package owning this source path.
    pub fn package(&self) -> pse_ids::SemanticId {
        self.package
    }
    /// Package-relative declared source path.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Opaque loader-issued text grammar/attribution and encoded-data edit intent.
/// It deliberately cannot be consumed as an admitted OwnedDocumentSet.
#[derive(Debug)]
struct ReloadOwner {
    bundles: Vec<DocumentBundle>,
    encoded: Vec<EncodedReloadDocument>,
    retained: usize,
    _leases: Vec<std::sync::Arc<pse_columnar::AllocationLease>>,
}
/// Immutable opaque reload owner. Clones retain one accounted source allocation.
#[derive(Clone, Debug)]
pub struct ParsedReload(std::sync::Arc<ReloadOwner>);
impl ParsedReload {
    /// Borrow immutable parsed text/context products in submitted package order.
    pub fn bundles(&self) -> &[DocumentBundle] {
        &self.0.bundles
    }
    /// Borrow exact encoded data intents without decoding their rows.
    pub fn encoded_data(&self) -> &[EncodedReloadDocument] {
        &self.0.encoded
    }
    /// Owned source extent for the evictable retained ingress accelerator.
    pub fn retained_bytes(&self) -> usize {
        self.0.retained
    }

    /// Parse changed texts through the same loader, reusing exact previous grammar
    /// owners. Add/remove is determined by the supplied full document inventory.
    /// Data remains encoded until the parent-qualified Runtime edit consumes it.
    #[expect(
        clippy::too_many_arguments,
        reason = "one pure source reload with its exact parser/native/resource owners"
    )]
    pub fn load(
        sources: &[std::collections::BTreeMap<String, Vec<u8>>],
        previous: &[DocumentBundle],
        registry: &Registry,
        budget: crate::authoring_driver::ParseBudget,
        pool: &std::sync::Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
        validation: &pse_relations::validate::ValidationContext,
    ) -> Result<Self, DriverError> {
        use super::allocation::{Allocation, add, mul};
        let mut bundles = Vec::new();
        let mut encoded = Vec::new();
        let mut leases = Vec::new();
        let mut packages = std::collections::BTreeSet::new();
        let mut retained = 0usize;
        let mut handles = Allocation::new(pool, cancel);
        handles.grow(mul(sources.len(), size_of::<DocumentBundle>() + 1024)?)?;
        for source in sources {
            cancel.checkpoint()?;
            let mut allocation = Allocation::new(pool, cancel);
            let source_bytes = source
                .values()
                .try_fold(0usize, |sum, bytes| add(sum, bytes.len()))?;
            allocation.grow(add(source_bytes, mul(source.len(), 1024)?)?)?;
            let header = source
                .get("package.toml")
                .ok_or_else(|| super::load::contract(None, "missing package.toml"))?;
            let text = std::str::from_utf8(header)
                .map_err(|_| super::load::contract(None, "package.toml must be UTF-8"))?;
            // Prefer the exact header owner; changed headers are identified by the
            // same existing header parser before selecting any previous documents.
            allocation.grow(super::allocation::parser_extent(text, &budget, false)?)?;
            let checksum = super::package_checksum(source);
            let (package, _, _) = super::load::header(text, checksum, &budget)?;
            if !packages.insert(package.package_id) {
                return Err(super::load::contract(
                    None,
                    "reload repeats a package identity",
                ));
            }
            let prior = previous
                .iter()
                .find(|bundle| bundle.package.package_id == package.package_id);
            let start = encoded.len();
            let mut bundle = super::load::load_reusing_mode(
                source.clone(),
                prior,
                registry,
                budget,
                Some(&mut allocation),
                validation,
                Some(&mut encoded),
            )?;
            bundle.retain_columns(pool, cancel)?;
            let bundle_bytes = add(super::allocation::bundle_retained(&bundle)?, 1024)?;
            let data_bytes = encoded[start..].iter().try_fold(0usize, |sum, data| {
                add(
                    sum,
                    add(data.bytes.len(), add(data.path.capacity(), 1024)?)?,
                )
            })?;
            allocation.retain(0, add(bundle_bytes, data_bytes)?)?;
            let lease = allocation.finish();
            let mut sizes = vec![bundle_bytes];
            sizes.extend(
                encoded[start..]
                    .iter()
                    .map(|data| add(data.bytes.len(), add(data.path.capacity(), 1024)?))
                    .collect::<Result<Vec<_>, DriverError>>()?,
            );
            let mut owners = lease
                .partition(&sizes)
                .map_err(|_| super::load::contract(None, "reload source ownership partition"))?
                .into_iter();
            let bundle_owner = owners
                .next()
                .ok_or_else(|| super::load::contract(None, "reload bundle owner absent"))?;
            leases.push(bundle.attach_lease(bundle_owner)?);
            for (data, lease) in encoded[start..].iter_mut().zip(owners) {
                data.lease = Some(lease);
            }
            retained = add(retained, add(bundle_bytes, data_bytes)?)?;
            bundles.push(bundle);
        }
        retained = add(retained, handles.size())?;
        leases.push(handles.finish());
        Ok(Self(std::sync::Arc::new(ReloadOwner {
            bundles,
            encoded,
            retained,
            _leases: leases,
        })))
    }
}
