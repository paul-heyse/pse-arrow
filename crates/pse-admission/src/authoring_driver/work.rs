// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked temporary-work extents; reservations never establish semantic validity.
use crate::authoring_driver::{DriverError, document::DocumentBundle};

pub(crate) fn add(a: usize, b: usize) -> Result<usize, DriverError> {
    a.checked_add(b)
        .ok_or_else(|| crate::authoring_driver::contract("workspace extent overflow"))
}
pub(crate) fn mul(a: usize, b: usize) -> Result<usize, DriverError> {
    a.checked_mul(b)
        .ok_or_else(|| crate::authoring_driver::contract("workspace extent overflow"))
}
pub(crate) fn sources(bundles: &[DocumentBundle]) -> Result<usize, DriverError> {
    bundles.iter().try_fold(4096, |n, bundle| {
        let source = bundle
            .documents
            .iter()
            .filter_map(|document| document.text())
            .try_fold(0, |n, text| add(n, text.len()))?;
        // A data document is decoded once, never parsed into nodes (ADR-0125): its bytes
        // and decoded rows are copied, not expanded.
        let data = bundle.documents.iter().try_fold(0, |n, document| {
            document.data().map_or(Ok(n), |data| {
                add(n, add(document.bytes().len(), data.retained_bytes())?)
            })
        })?;
        // AST nodes, decoded DTO fields, path indexes and staging pre/post/key copies
        // coexist; each is bounded by source bytes and actual visible Arrow inventory.
        let columns = bundle.batches.values().try_fold(0, |bytes, batch| {
            add(bytes, pse_columnar::algorithm_decode_extent(batch.batch())?)
        })?;
        add(n, add(add(mul(source, 128)?, mul(data, 2)?)?, columns)?)
    })
}
