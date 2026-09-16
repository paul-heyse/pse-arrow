// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Retained candidate metadata, separate from temporary Arrow validation/decode work.

use super::{ChangeOp, ChangeSet, StagedMember};
use crate::{
    AuthoringError,
    work::{add, mul},
};
use arrow_array::{ArrayRef, RecordBatch};
use pse_ids::SemanticId;
use std::collections::BTreeMap;

/// A full pinned Rust B-tree node per entry bounds partially filled nodes as well
/// as their keys and values. Empty maps allocate no node.
pub(super) fn map<K, V>(count: usize) -> Result<usize, AuthoringError> {
    mul(
        count,
        add(
            mul(11, add(size_of::<K>(), size_of::<V>())?)?,
            mul(16, size_of::<usize>())?,
        )?,
    )
}

/// These arrays and their buffers are Arc-shared with independently retained
/// owners. A `RecordBatch` clone allocates only its vector of column Arc handles.
fn batch_handles(batch: &RecordBatch) -> Result<usize, AuthoringError> {
    mul(batch.num_columns(), size_of::<ArrayRef>())
}

/// Compute before cloning: Vec/String clones allocate their populated lengths,
/// while B-tree nodes use the conservative actual key/value type bound above.
pub(super) fn envelope(changes: &ChangeSet) -> Result<usize, AuthoringError> {
    let mut extent = add(changes.header.author.len(), changes.header.message.len())?;
    extent = add(extent, mul(changes.ops.len(), size_of::<ChangeOp>())?)?;
    for op in &changes.ops {
        extent = add(extent, op.row_key.staged_port.len())?;
        extent = add(
            extent,
            op.row.as_ref().map_or(0, |row| row.staged_port.len()),
        )?;
        extent = add(extent, op.precondition.as_ref().map_or(0, String::len))?;
    }
    Ok(extent)
}

fn changes(changes: &ChangeSet) -> Result<usize, AuthoringError> {
    let mut extent = add(
        envelope(changes)?,
        map::<String, StagedMember>(changes.staged.len())?,
    )?;
    for (port, member) in &changes.staged {
        extent = add(extent, add(port.len(), batch_handles(&member.batch)?)?)?;
    }
    Ok(extent)
}

/// Candidate wrapper/lease controls, exact inventory handles, and the complete
/// immutable `ChangeSet` handle inventory. Validation expansions and buffer payloads are not
/// retained here: the former are dropped and the latter own their own leases.
pub(super) fn candidate(
    relations: &BTreeMap<SemanticId, RecordBatch>,
    change_set: &ChangeSet,
) -> Result<usize, AuthoringError> {
    let mut extent = super::owned::candidate_wrapper_extent();
    extent = add(extent, map::<SemanticId, RecordBatch>(relations.len())?)?;
    for batch in relations.values() {
        extent = add(extent, batch_handles(batch)?)?;
    }
    add(extent, changes(change_set)?)
}
