// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Result detail selection preserves original addresses and native coverage.
use super::{QueryPresentation, SearchResult};

/// Query producers own record budgets; shared provenance and coverage cannot be
/// removed by the generic envelope payload selector. Schema and output use this owner.
pub fn envelope_view(requested: &crate::envelope::Presentation) -> crate::envelope::Presentation {
    let mut fields = requested.fields.clone();
    if let Some(fields) = &mut fields {
        for mandatory in [
            "query",
            "sources",
            "summary",
            "diagnostics",
            "native_failures",
        ] {
            if !fields.iter().any(|field| field == mandatory) {
                fields.push(mandatory.into());
            }
        }
    }
    crate::envelope::Presentation {
        fields,
        budget: None,
    }
}

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
            source_relationship: None,
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
            refinement: None,
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
