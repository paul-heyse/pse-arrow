// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked temporary-work extents; reservations never establish semantic validity.
use crate::authoring_driver::{DriverError, document::DocumentBundle};

pub(crate) fn add(a: usize, b: usize) -> Result<usize, DriverError> {
    a.checked_add(b).ok_or_else(|| {
        crate::authoring_driver::native_relations::contract("workspace extent overflow")
    })
}
pub(crate) fn mul(a: usize, b: usize) -> Result<usize, DriverError> {
    a.checked_mul(b).ok_or_else(|| {
        crate::authoring_driver::native_relations::contract("workspace extent overflow")
    })
}
pub(crate) fn sources(bundles: &[DocumentBundle]) -> Result<usize, DriverError> {
    bundles.iter().try_fold(4096, |n, bundle| {
        let source = bundle
            .documents
            .iter()
            .try_fold(0, |n, document| add(n, document.text.len()))?;
        // AST nodes, decoded DTO fields, path indexes and staging pre/post/key copies
        // coexist; each is bounded by source bytes and actual visible Arrow inventory.
        let columns = bundle.batches.values().try_fold(0, |bytes, batch| {
            add(bytes, pse_columnar::algorithm_decode_extent(batch.batch())?)
        })?;
        add(n, add(mul(source, 128)?, columns)?)
    })
}
