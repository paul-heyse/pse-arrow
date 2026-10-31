// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Readable code witnesses while retaining original portable references and coverage.
use crate::{
    envelope::Envelope,
    investigation::{self as query, Report, SearchResult},
};
use std::fmt::Write as _;

pub(super) fn row(
    row: &SearchResult,
    path: Option<&query::NativePath>,
) -> Result<String, query::Error> {
    let mut text = String::new();
    let path = path
        .map(|path| format!("{:?}", path.to_path_buf()))
        .unwrap_or_else(|| row.source.clone());
    let line = row
        .range
        .as_ref()
        .map_or(String::new(), |range| format!(":{}", range.line_start));
    let _ = writeln!(
        text,
        "{path}{line}  {} [{}]",
        row.name.as_deref().unwrap_or(""),
        row.kind
    );
    if !row.context.is_empty() {
        text.push_str(&row.context);
        if !text.ends_with('\n') {
            text.push('\n');
        }
    }
    if let Some(bytes) = &row.bytes {
        let _ = writeln!(text, "original_bytes={}", serde_json::to_string(bytes)?);
    }
    if let Some(reference) = &row.reference {
        let _ = writeln!(text, "reference={}", serde_json::to_string(reference)?);
    }
    if let Some(reference) = &row.candidate_reference {
        let _ = writeln!(
            text,
            "candidate_reference={}",
            serde_json::to_string(reference)?
        );
    }
    if let Some(refinement) = &row.refinement {
        let _ = writeln!(text, "refinement={}", serde_json::to_string(refinement)?);
    }
    Ok(text)
}
pub(super) fn report(response: &Envelope<Report>) -> Result<String, query::Error> {
    let mut text = format!(
        "{}; invocation={}; outcome={:?}; subject={:?}; coverage={:?}\n",
        response.summary,
        response.invocation_id,
        response.outcome,
        response.subject_state,
        response.observation.completeness
    );
    if let Some(query) = &response.payload.query {
        let _ = writeln!(text, "query={}", serde_json::to_string(query)?);
    }
    for source in &response.payload.sources {
        let _ = writeln!(text, "source={}", serde_json::to_string(source)?);
    }
    for witness in &response.payload.results {
        let path = response
            .payload
            .sources
            .iter()
            .find(|source| source.id == witness.source)
            .map(|source| &source.path);
        text.push_str(&row(witness, path)?);
    }
    for diagnostic in &response.payload.diagnostics {
        let _ = writeln!(text, "diagnostic={}", serde_json::to_string(diagnostic)?);
    }
    for failure in &response.payload.native_failures {
        let _ = writeln!(text, "native_failure={}", serde_json::to_string(failure)?);
    }
    let _ = writeln!(
        text,
        "coverage={}",
        serde_json::to_string(&response.payload.summary)?
    );
    Ok(text)
}
