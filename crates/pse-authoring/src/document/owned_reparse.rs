// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Accounted admission of caller DTO source inventories.
use super::{DocumentBundle, OwnedDocumentSet};
use crate::AuthoringError;
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::Registry;

/// Retain immutable private loader results without reparsing original bytes.
/// # Errors
/// Source/schema/identity failures, cancellation or an unavailable shared reservation.
pub fn load_bundles_owned(
    bundles: &[DocumentBundle],
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentSet, AuthoringError> {
    let mut work = reserver.open("authoring:bundle-inventory");
    work.try_grow(crate::work::mul(bundles.len(), 256)?)?;
    let mut parts = Vec::with_capacity(bundles.len());
    for bundle in bundles {
        cancel.checkpoint()?;
        parts.push(super::owned::retain_bundle(
            bundle, registry, reserver, cancel,
        )?);
    }
    OwnedDocumentSet::try_from_bundles(parts, reserver, cancel)
}

/// Checked temporary parse/binding/staging extent from actual retained sources and cells.
/// This is allocation accounting only; callers must separately validate the inventory.
/// # Errors
/// Arithmetic overflow in the recursive allocation forecast.
pub fn workspace_extent(bundles: &OwnedDocumentSet) -> Result<usize, AuthoringError> {
    crate::work::sources(bundles.bundles())
}
