// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2: primitive-fact validation (blueprint §14.1, §22.2).
//!
//! The commit contract: a committed revision is structurally and referentially valid. It
//! may still fail P3-P10; publication as a compiled problem is gated separately by the
//! closure report (§14.5).
//!
use crate::CompilerError;
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::RecordBatch;
use pse_rules::invariants::{InvariantReport, InvariantScope};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

/// Evaluate every applicable invariant against duplicate-preserving unpublished rows.
/// # Errors
/// Missing actual dependency, malformed rule plan, cancellation or resource refusal.
pub async fn validate(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<InvariantReport, CompilerError> {
    Ok(pse_rules::invariants::run_invariants(
        rows,
        session,
        registry,
        InvariantScope::Candidate,
        None,
        cancel,
    )
    .await?)
}
