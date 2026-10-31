// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Result detail selection preserves original addresses and native coverage.
use super::{QueryPresentation, SearchResult};

/// Detail omitted by an explicit field selection; identity is never selectable away.
pub fn omitted(presentation: &QueryPresentation) -> Vec<String> {
    if presentation.fields.is_empty() {
        return Vec::new();
    }
    ["name", "context", "captures", "bytes"]
        .into_iter()
        .filter(|field| !presentation.fields.iter().any(|selected| selected == field))
        .map(str::to_owned)
        .collect()
}

/// Project one row and return its requested grouping key, before any serialization.
pub fn project(row: &mut SearchResult, presentation: &QueryPresentation) -> Option<String> {
    let group = match presentation.group_by.as_deref() {
        Some("source") => Some(row.source.clone()),
        Some("kind") => Some(row.kind.clone()),
        Some("name") => Some(row.name.clone().unwrap_or_else(|| "<unavailable>".into())),
        _ => None,
    };
    for field in omitted(presentation) {
        match field.as_str() {
            "name" => row.name = None,
            "context" => row.context.clear(),
            "captures" => row.captures.clear(),
            "bytes" => row.bytes = None,
            _ => {}
        }
    }
    group
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn selection_keeps_identity_and_grouping_uses_original_name() {
        let mut row = SearchResult {
            source: "s1".into(),
            kind: "function".into(),
            name: Some("model::prepare".into()),
            range: None,
            context: "body".into(),
            reference: None,
            captures: BTreeMap::new(),
            bytes: None,
            entity: None,
            candidate_reference: None,
        };
        let presentation = QueryPresentation {
            fields: vec!["context".into()],
            group_by: Some("name".into()),
            ..Default::default()
        };
        assert_eq!(
            project(&mut row, &presentation).as_deref(),
            Some("model::prepare")
        );
        assert_eq!(row.source, "s1");
        assert_eq!(row.kind, "function");
        assert_eq!(row.context, "body");
        assert_eq!(row.name, None);
    }
}
