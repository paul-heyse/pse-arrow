// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P1: authoring parse (blueprint §14.1).
//!
//! The pass wrapper around `pse_authoring::p1`.
//!
use crate::CompilerError;
use pse_authoring::{
    change_set::{AuthoredReader, CandidateSnapshot, ChangeSet},
    document::DocumentBundle,
};
use pse_relations::generated::authored;
use pse_schema::Registry;

/// Reparse, bind and stage a complete desired document universe, then apply exact preimages.
/// # Errors
/// Source syntax/binding, stale base, malformed operations or actual preimage disagreement.
pub fn stage(
    documents: &[DocumentBundle],
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    registry: &Registry,
) -> Result<CandidateSnapshot, CompilerError> {
    let changes = pse_authoring::p1::stage(documents, base, header, registry)?;
    apply(base, &changes, registry)
}
/// Apply a typed change-set, including fully bound rename proof, to an unpublished candidate.
/// # Errors
/// Exact base, typed preimage, source proof or complete operation inventory failures.
pub fn apply(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    registry: &Registry,
) -> Result<CandidateSnapshot, CompilerError> {
    Ok(pse_authoring::change_set::apply(base, changes, registry)?)
}
