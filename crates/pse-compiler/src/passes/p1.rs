// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P1 retains source construction; application is only the external change boundary.
use crate::CompilerError;
use pse_authoring::{
    change_set::{AuthoredReader, ChangeSet, OwnedCandidateSnapshot},
    document::OwnedDocumentSet,
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::generated::authored;

/// Construct one immutable desired source universe through the native change plan.
/// # Errors
/// Source/binding/base failures, cancellation or native execution.
pub async fn stage(
    documents: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<OwnedCandidateSnapshot, CompilerError> {
    Ok(pse_authoring::p1::construct(documents, base, header, session, cancel).await?)
}
/// Apply an external current-format ordered change envelope.
/// # Errors
/// Exact before-image, source correspondence or native execution failure.
pub async fn apply(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<OwnedCandidateSnapshot, CompilerError> {
    Ok(pse_authoring::change_set::apply_owned(base, changes, session, cancel).await?)
}
