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
    let path = row
        .reference
        .as_ref()
        .and_then(|reference| match &reference.locator {
            query::Locator::Semantic { path, .. } => Some(path),
            _ => None,
        })
        .or(path)
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
        let _ = writeln!(text, "address={}", reference.encode()?);
        let _ = writeln!(text, "reference={}", serde_json::to_string(reference)?);
    }
    if let Some(reference) = &row.candidate_reference {
        let _ = writeln!(text, "candidate_address={}", reference.encode()?);
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
    for evidence in &response.payload.summary.relationships {
        text.push_str(&relationships(evidence)?);
    }
    let mut coverage = response.payload.summary.clone();
    // The graph's identities and coverage were rendered once above, with reusable actions.
    coverage.relationships.clear();
    let _ = writeln!(text, "coverage={}", serde_json::to_string(&coverage)?);
    Ok(text)
}

fn relationships(evidence: &query::RelationshipEvidence) -> Result<String, query::Error> {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "relationships: seeds={:?}; relations={:?}; direction={:?}; seed_match={:?}; depth={}; complete={}; stop={:?}; examined_edges={}; frontier={:?}",
        evidence.seeds,
        evidence.relations,
        evidence.direction,
        evidence.seed_match,
        evidence.depth,
        evidence.graph.complete,
        evidence.graph.stop,
        evidence.graph.examined_edges,
        evidence.graph.frontier
    );
    let _ = writeln!(
        text,
        "traversal={}",
        serde_json::to_string(&evidence.traversal)?
    );
    for node in &evidence.nodes {
        let selected = evidence.graph.selected.contains(&node.id);
        let distances = evidence
            .graph
            .reached
            .iter()
            .find(|reached| reached.node == node.id)
            .map(|reached| &reached.distances);
        let _ = writeln!(
            text,
            "node {} [{}] {}; seed_distances={distances:?}",
            node.id,
            if selected { "answer" } else { "support" },
            node.name.as_deref().unwrap_or("<unnamed>")
        );
        let _ = writeln!(text, "  entity={}", serde_json::to_string(&node.entity)?);
        if let query::Locator::Semantic {
            path, line, column, ..
        } = &node.reference.locator
        {
            let _ = writeln!(text, "  location={:?}:{line}:{column}", path.to_path_buf());
        }
        let _ = writeln!(text, "  inspect --result {}", node.reference.encode()?);
    }
    for edge in &evidence.graph.edges {
        let _ = writeln!(
            text,
            "edge {} -> {} [{:?}]; witness={}",
            edge.source, edge.target, edge.relation, edge.witness
        );
    }
    for witness in &evidence.witnesses {
        let _ = writeln!(
            text,
            "witness {} role={}; declared_type={:?}; instantiated_type={:?}",
            witness.id,
            serde_json::to_string(&witness.role)?,
            witness.declared_type,
            witness.instantiated_type
        );
        if let Some(reference) = &witness.reference {
            let _ = writeln!(text, "  inspect --result {}", reference.encode()?);
        }
        for limitation in &witness.limitations {
            let _ = writeln!(text, "  limitation={limitation}");
        }
    }
    for limitation in &evidence.limitations {
        let _ = writeln!(text, "limitation={limitation}");
    }
    Ok(text)
}
