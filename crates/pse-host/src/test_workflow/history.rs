// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected developer history through the original foreground and inventory owners.

use super::{
    WorkflowError,
    execute::{Action, Outcome},
    source::failed_cases,
};
use crate::{
    HostError,
    envelope::{CliInvocationId, Completeness, Envelope, Settlement},
    invocation::{InvocationMetadata, InvocationStore},
    managed::result::ManagedSummary,
    selection::{Filters, IgnoredPolicy, RerunCase},
};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

const MAX_FACTS: usize = 128;
const MAX_BYTES: usize = 64 * 1024;

/// Bounded observations from a settled native test workflow. No working payload is included.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct NativeTestHistory {
    /// Original workflow, independent of science identities.
    pub invocation: CliInvocationId,
    /// Original checkout identity.
    pub checkout_id: String,
    /// Original repository identity.
    pub repository_id: String,
    /// Original creation clock.
    pub created_unix_ms: u64,
    /// Original prepared control manifest, without an inferred Git revision.
    pub control_manifest_sha256: String,
    /// Original typed test result.
    pub outcome: Outcome,
    /// Original report interpretation.
    pub report_version: u32,
    /// Original selected test count.
    pub selected_tests: usize,
    /// Exact original report commitment.
    pub report_sha256: String,
    /// Exact original inventory commitment.
    pub inventory_sha256: String,
    /// Exact original foreground settlement commitment.
    pub settlement_sha256: String,
    /// Count of original omissions, absent where the interpretation has no dependency account.
    pub omitted_evidence: Option<usize>,
    /// Original top-level diagnostic codes in occurrence order.
    pub failure_codes: Vec<String>,
    /// Exact original failure selection for a later fresh inventory lookup.
    pub failed_cases: Vec<RerunCase>,
    /// Original effective test profile.
    pub profile: String,
    /// Original explicit target triple, if selected.
    pub platform: Option<String>,
    /// Original native test filters.
    pub filters: Filters,
    /// Original ignored-test policy.
    pub ignored: IgnoredPolicy,
}

fn invalid(reason: &str) -> HostError {
    HostError::Ownership(format!("native developer history: {reason}"))
}

/// Acquire compact facts from one exact original foreground result.
/// # Errors
/// Unsettled, borrowed, incomplete, foreign, changed or oversized observations refuse.
pub fn native_test_history(
    store: &InvocationStore,
    id: &CliInvocationId,
) -> Result<NativeTestHistory, HostError> {
    let metadata: InvocationMetadata<serde::de::IgnoredAny> = store.metadata(id)?;
    let directory = store.directory(id)?;
    let settlement_bytes = directory.paired_settlement()?;
    let settled: Envelope<ManagedSummary> =
        serde_json::from_slice(&settlement_bytes).map_err(std::io::Error::other)?;
    if &settled.invocation_id != id
        || settled.command != Action::Test.command()
        || settled.settlement != Settlement::Complete
        || settled.observation.completeness != Completeness::Complete
        || settled.payload.native_lifetime_pending != Some(false)
        || settled.payload.borrowed_reservation
    {
        return Err(invalid("the original foreground test has no complete owned settlement"));
    }
    let evidence = crate::test_inventory::captured_at_limit(
        store,
        id.as_str(),
        None,
        Some(crate::developer_evidence::MAX_INPUT_BYTES),
    )?;
    let input = crate::managed::workflow::retained_input(
        store,
        id,
        Action::Test,
        &evidence.observation,
    )?;
    let selected_tests = evidence
        .observation
        .selected_tests
        .ok_or_else(|| invalid("the original selected population is unavailable"))?;
    let failed_cases = failed_cases(&evidence)
        .map_err(|error: WorkflowError| HostError::Workflow(Box::new(error)))?;
    let failure_codes = evidence
        .observation
        .failures
        .iter()
        .filter_map(|failure| failure.code.clone())
        .take(MAX_FACTS + 1)
        .collect::<Vec<_>>();
    if failed_cases.len() > MAX_FACTS || failure_codes.len() > MAX_FACTS {
        return Err(invalid("selected test history exceeds its fact allowance"));
    }
    let history = NativeTestHistory {
        invocation: id.clone(),
        checkout_id: metadata.checkout_id,
        repository_id: metadata.repository_id,
        created_unix_ms: metadata.created_unix_ms,
        control_manifest_sha256: metadata.selection.control.manifest_digest,
        outcome: evidence.observation.outcome,
        report_version: evidence.observation.version,
        selected_tests,
        report_sha256: evidence.terminal.report.sha256,
        inventory_sha256: evidence.inventory_sha256,
        settlement_sha256: Sha256::digest(&settlement_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        omitted_evidence: evidence.observation.evidence.map(|files| {
            files.iter().filter(|file| file.omission.is_some()).count()
        }),
        failure_codes,
        failed_cases,
        profile: evidence.inventory.runner_profile,
        platform: input.request.platform,
        filters: evidence.inventory.selection.filters().clone(),
        ignored: evidence.inventory.selection.ignored(),
    };
    if serde_json::to_vec(&history).map_err(std::io::Error::other)?.len() > MAX_BYTES {
        return Err(invalid("selected test history exceeds its byte allowance"));
    }
    Ok(history)
}
