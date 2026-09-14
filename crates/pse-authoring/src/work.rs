// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked temporary-work extents; reservations never establish semantic validity.
use crate::{
    AuthoringError,
    document::{DocumentBundle, Rows},
};
use pse_schema::model::Cell;

pub(crate) fn add(a: usize, b: usize) -> Result<usize, AuthoringError> {
    a.checked_add(b)
        .ok_or_else(|| crate::change_set::contract("workspace extent overflow"))
}
pub(crate) fn mul(a: usize, b: usize) -> Result<usize, AuthoringError> {
    a.checked_mul(b)
        .ok_or_else(|| crate::change_set::contract("workspace extent overflow"))
}
pub(crate) fn cell(value: &Cell) -> Result<usize, AuthoringError> {
    let payload = match value {
        Cell::Text(text) => text.len(),
        Cell::List(values) | Cell::Struct(values) => {
            values.iter().try_fold(0, |n, value| add(n, cell(value)?))?
        }
        _ => 0,
    };
    add(size_of::<Cell>(), payload)
}
pub(crate) fn rows(rows: &Rows) -> Result<usize, AuthoringError> {
    rows.values().flatten().try_fold(4096, |n, row| {
        row.iter()
            .try_fold(add(n, 256)?, |n, value| add(n, cell(value)?))
    })
}
pub(crate) fn sources(bundles: &[DocumentBundle]) -> Result<usize, AuthoringError> {
    bundles.iter().try_fold(4096, |n, bundle| {
        let source = bundle
            .documents
            .iter()
            .try_fold(0, |n, document| add(n, document.text.len()))?;
        // AST nodes, decoded DTO fields, path indexes and staging pre/post/key copies
        // coexist; each is bounded by source bytes and actual recursive Cell inventory.
        add(n, add(mul(source, 128)?, mul(rows(&bundle.rows)?, 16)?)?)
    })
}
