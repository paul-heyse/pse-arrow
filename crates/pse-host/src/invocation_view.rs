// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive views of original invocation evidence, without admission or stop authority.
use crate::{
    HostError,
    artifacts::Content,
    envelope::{
        CliInvocationId, Completeness, Envelope, ErrorRecord, RedactionPolicy, SubjectState,
    },
    extraction::OutputStream,
    generation::PreparedGeneration,
    invocation::InvocationStore,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    io::{Read, Seek, SeekFrom},
    num::NonZeroU64,
};

/// One metadata-only row. Operation payloads and historical results are not acquired.
#[derive(Debug, Serialize, JsonSchema)]
pub struct HistoryRow {
    /// Original CLI identity, independent of this observation.
    pub invocation_id: CliInvocationId,
    /// Original creation timestamp; ordering is descending timestamp then identity.
    pub created_unix_ms: u64,
    /// Original owning verb.
    pub command: String,
    /// Explicit session association.
    pub session: Option<String>,
    /// Explicit parent association.
    pub parent: Option<CliInvocationId>,
    /// Recorded control identity, not permission to execute a recorded path.
    pub control: PreparedGeneration,
}
/// Bounded metadata presentation; collection failures remain mandatory envelope facts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct HistoryPage {
    /// Requested session, or all checkout sessions.
    pub session: Option<String>,
    /// Metadata rows in this selected window.
    pub records: Vec<HistoryRow>,
    /// Successfully inspected rows matching this scope, excluding unreadable sources.
    pub known_matches: usize,
    /// Live collection membership is not an atomic snapshot or workload census.
    pub atomic_snapshot: bool,
}
/// Inspect newest metadata without loading stored operation or terminal payloads.
pub fn history(
    store: &InvocationStore,
    session: Option<&str>,
    limit: usize,
    policy: &RedactionPolicy,
) -> Result<Envelope<HistoryPage>, HostError> {
    let history = store.history::<serde::de::IgnoredAny>()?;
    let mut records: Vec<_> = history
        .records
        .into_iter()
        .filter(|record| {
            session.is_none_or(|session| record.selection.session.as_deref() == Some(session))
        })
        .map(|record| HistoryRow {
            invocation_id: record.selection.invocation_id,
            created_unix_ms: record.created_unix_ms,
            command: record.selection.command,
            session: record.selection.session,
            parent: record.selection.parent,
            control: record.selection.control,
        })
        .collect();
    records.reverse();
    let known_matches = records.len();
    records.truncate(limit);
    let mut response = Envelope::response(
        "history",
        format!(
            "{} of {known_matches} known invocation metadata rows",
            records.len()
        ),
        HistoryPage {
            session: session.map(str::to_owned),
            records,
            known_matches,
            atomic_snapshot: false,
        },
    );
    response.observation.scope = "checkout_invocation_metadata_scan".into();
    response.observation.limitations.push(
        "metadata only; terminal results and current workload state were not inspected".into(),
    );
    if known_matches > limit {
        response.view.truncated = true;
        response
            .observation
            .limitations
            .push("newest metadata window selected; increase --limit for older records".into());
    }
    for failure in history.failures {
        response
            .failures
            .push(ErrorRecord::from_typed(&failure, policy));
        response.observation.completeness = Completeness::Partial;
    }
    Ok(response)
}

/// The original full response is nested under the observing invocation's identity.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ResultView {
    /// Authenticated selected original identity.
    pub original_invocation: Option<CliInvocationId>,
    /// Original immutable response, including opaque owner payload and all original failures.
    #[schemars(with = "Option<Envelope<serde_json::Value>>")]
    pub response: Option<Envelope<Box<serde_json::value::RawValue>>>,
}
/// Absence of result evidence is unknown completion, never an inferred running or success state.
pub fn result(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<Envelope<ResultView>, HostError> {
    let selected = store.resolve(selector, session)?;
    let retained = selected
        .as_ref()
        .map(|id| store.result::<serde::de::IgnoredAny, Box<serde_json::value::RawValue>>(id))
        .transpose()?
        .flatten();
    let mut response = Envelope::response(
        "result",
        if selected.is_none() {
            "no matching invocation"
        } else if retained.is_none() {
            "no terminal result observed; completion is unknown"
        } else {
            "original terminal evidence observed"
        },
        ResultView {
            original_invocation: selected,
            response: retained,
        },
    );
    response.subject_state = if response.payload.original_invocation.is_none() {
        SubjectState::NoMatch
    } else if response.payload.response.is_some() {
        SubjectState::Available
    } else {
        SubjectState::Unknown
    };
    response.observation.scope = "original_invocation_terminal_evidence".into();
    Ok(response)
}

/// One live raw-stream extent with exact original byte coordinates.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LogView {
    /// Original owner identity.
    pub original_invocation: CliInvocationId,
    /// Exact stream; there is no inferred cross-stream producer ordering.
    pub stream: OutputStream,
    /// Original mutable file size at this observation.
    pub observed_bytes: u64,
    /// Actual half-open selected range.
    pub byte_range: std::ops::Range<u64>,
    /// Exact bytes, encoding non-UTF-8 without loss.
    pub content: Content,
    /// Explicit continuation if the selected extent did not cover the observed remainder.
    pub next_offset: Option<u64>,
    /// A live ledger stream is not an immutable object or snapshot.
    pub atomic_snapshot: bool,
}
/// Read a protected local raw stream directly. Generic object-store paths cannot establish
/// the local ledger's no-follow/ownership contract.
pub fn log(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    stream: OutputStream,
    offset: u64,
    length: NonZeroU64,
    tail: bool,
) -> Result<Envelope<LogView>, HostError> {
    let id = store
        .resolve(selector, session)?
        .ok_or_else(|| HostError::Ownership("no matching invocation for log read".into()))?;
    let directory = store.directory(&id)?;
    let mut file = directory.stream(stream)?;
    let observed_bytes = file.metadata()?.len();
    let start = if tail {
        observed_bytes.saturating_sub(length.get())
    } else {
        offset
    };
    if start > observed_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "log offset exceeds observed bytes",
        )
        .into());
    }
    let end = start.saturating_add(length.get()).min(observed_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.take(end - start).read_to_end(&mut bytes)?;
    let actual_end = start + bytes.len() as u64;
    let content = match String::from_utf8(bytes) {
        Ok(text) => Content::Utf8(text),
        Err(error) => Content::Base64(STANDARD.encode(error.into_bytes())),
    };
    let mut response = Envelope::response(
        "log",
        format!("original {stream:?} bytes {start}..{actual_end} of {observed_bytes} observed"),
        LogView {
            original_invocation: id,
            stream,
            observed_bytes,
            byte_range: start..actual_end,
            content,
            next_offset: (actual_end < observed_bytes).then_some(actual_end),
            atomic_snapshot: false,
        },
    );
    response.observation.scope = "original_raw_stream_extent".into();
    if actual_end != end {
        response.observation.completeness = Completeness::Partial;
        response
            .observation
            .limitations
            .push("raw stream shortened during observation".into());
    }
    Ok(response)
}

#[cfg(test)]
mod tests;
