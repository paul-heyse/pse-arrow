// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Protocol-owner projection. Only this producer interprets original LSP payloads.
use super::{
    CandidateKind, FactDirection, FactLocation, FactRelation, LazyCandidateFact, ObservedFacts,
    PredicateFact, ProtocolRange, RelationshipFact, Result, Section, SymbolFact, TestCandidateFact,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    #[serde(alias = "targetUri")]
    uri: String,
    #[serde(alias = "targetSelectionRange", alias = "targetRange")]
    range: ProtocolRange,
}
pub(super) fn location(value: &Value) -> Option<FactLocation> {
    // LocationLink has both targetRange and targetSelectionRange; choose the
    // selected declaration interval explicitly rather than duplicate aliases.
    let parsed = if let (Some(uri), Some(range)) = (
        value.get("targetUri").and_then(Value::as_str),
        value
            .get("targetSelectionRange")
            .or_else(|| value.get("targetRange")),
    ) {
        Location {
            uri: uri.into(),
            range: serde_json::from_value(range.clone()).ok()?,
        }
    } else {
        serde_json::from_value(value.clone()).ok()?
    };
    Some(FactLocation::Protocol {
        uri: parsed.uri,
        range: parsed.range,
    })
}
fn call_location(value: &Value) -> Option<FactLocation> {
    // CallHierarchyItem.range covers the entire body; the original selected
    // identifier interval is the declaration/edge anchor used by consumers.
    let uri = value.get("uri")?.as_str()?;
    let range = value.get("selectionRange").or_else(|| value.get("range"))?;
    Some(FactLocation::Protocol {
        uri: uri.into(),
        range: serde_json::from_value(range.clone()).ok()?,
    })
}
/// A portable kind category is presentation only; original native kind is retained in evidence.
pub(super) fn symbol_kind(value: &Value) -> Option<String> {
    if let Some(kind) = value.as_str() {
        return Some(match kind {
            "TypeAlias" => "type_alias".into(),
            "Const" => "constant".into(),
            other => other.to_ascii_lowercase(),
        });
    }
    let kind = match value.as_u64()? {
        1 => "file",
        2 => "module",
        3 => "namespace",
        4 => "package",
        5 => "class",
        6 => "method",
        7 => "property",
        8 => "field",
        9 => "constructor",
        10 => "enum",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        15 => "string",
        16 => "number",
        17 => "boolean",
        18 => "array",
        19 => "object",
        20 => "key",
        21 => "null",
        22 => "enum_member",
        23 => "struct",
        24 => "event",
        25 => "operator",
        26 => "type_parameter",
        _ => return None,
    };
    Some(kind.into())
}
pub(super) fn symbols(value: &Value, document_uri: Option<&str>) -> Vec<SymbolFact> {
    let mut result = Vec::new();
    for row in records(value) {
        if let Some(name) = row.get("name").and_then(Value::as_str) {
            let location = row.get("location").and_then(location).or_else(|| {
                Some(FactLocation::Protocol {
                    uri: document_uri?.into(),
                    range: serde_json::from_value(
                        row.get("selectionRange")
                            .or_else(|| row.get("range"))?
                            .clone(),
                    )
                    .ok()?,
                })
            });
            result.push(SymbolFact {
                name: name.into(),
                kind: symbol_kind(&row["kind"]),
                container: row
                    .get("containerName")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                location,
                source_sha256: row
                    .get("source_sha256")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
        if let Some(children) = row.get("children") {
            result.extend(symbols(children, document_uri));
        }
    }
    result
}
fn records(value: &Value) -> Vec<&Value> {
    match value {
        Value::Null => Vec::new(),
        Value::Array(rows) => rows.iter().collect(),
        Value::Object(_) => value
            .get("items")
            .and_then(Value::as_array)
            .map_or_else(|| vec![value], |items| items.iter().collect()),
        _ => Vec::new(),
    }
}
fn relation_rows(
    value: &Value,
    relation: FactRelation,
    direction: FactDirection,
    result: &mut ObservedFacts,
) {
    for row in records(value) {
        if let Some(location) = location(row.get("location").unwrap_or(row)) {
            result.relationships.push(RelationshipFact {
                relation,
                direction,
                location,
                name: row.get("name").and_then(Value::as_str).map(str::to_owned),
                read: None,
                write: None,
                import: None,
                test: None,
            });
        }
    }
}
pub(super) fn project(
    sections: &[Section],
    document_uri: &str,
) -> Result<(ObservedFacts, BTreeMap<FactRelation, usize>)> {
    let mut result = ObservedFacts::default();
    let mut omitted = BTreeMap::new();
    for section in sections {
        let value = &section.evidence;
        let relation = match section.category.as_str() {
            "definition" => Some(FactRelation::Definition),
            "type_definition" => Some(FactRelation::TypeDefinition),
            "implementations" => Some(FactRelation::Implementations),
            "references_by_file" => Some(FactRelation::References),
            "incoming_calls_one_hop" | "outgoing_calls_one_hop" => Some(FactRelation::Calls),
            _ => None,
        };
        if let Some(relation) = relation {
            *omitted.entry(relation).or_default() += section.omitted;
        }
        match section.category.as_str() {
            "definition" | "type_definition" | "implementations" => relation_rows(
                value,
                relation.unwrap_or(FactRelation::Definition),
                FactDirection::Outgoing,
                &mut result,
            ),
            "references_by_file" => {
                for file in value["files"].as_array().into_iter().flatten() {
                    relation_rows(
                        &file["references"],
                        FactRelation::References,
                        FactDirection::Incoming,
                        &mut result,
                    );
                }
            }
            "incoming_calls_one_hop" | "outgoing_calls_one_hop" => {
                let incoming = section.category == "incoming_calls_one_hop";
                for row in records(value) {
                    let item = &row[if incoming { "from" } else { "to" }];
                    if let Some(location) = call_location(item) {
                        result.relationships.push(RelationshipFact {
                            relation: FactRelation::Calls,
                            direction: if incoming {
                                FactDirection::Incoming
                            } else {
                                FactDirection::Outgoing
                            },
                            location,
                            name: item.get("name").and_then(Value::as_str).map(str::to_owned),
                            read: None,
                            write: None,
                            import: None,
                            test: None,
                        });
                    }
                }
            }
            "workspace_symbols" | "document_symbols" => {
                result.symbols.extend(symbols(value, Some(document_uri)))
            }
            "predicate_evaluation" => {
                let holds = value
                    .as_bool()
                    .or_else(|| value.get("holds").and_then(Value::as_bool));
                result.predicate = Some(PredicateFact {
                    outcome: match value.get("status").and_then(Value::as_str) {
                        Some("holds" | "Holds") => super::PredicateOutcome::Holds,
                        Some("notProven" | "not_proven" | "NotProven") => {
                            super::PredicateOutcome::NotProven
                        }
                        Some("invalid" | "Invalid") => super::PredicateOutcome::Invalid,
                        Some(_) => super::PredicateOutcome::Unsupported,
                        _ if holds == Some(true) => super::PredicateOutcome::Holds,
                        _ if holds == Some(false) => super::PredicateOutcome::NotProven,
                        _ => super::PredicateOutcome::Unsupported,
                    },
                    cause: value
                        .get("message")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                });
            }
            "completion_candidates" | "assist_candidates" | "selected_inlay_annotations" => {
                let kind = match section.category.as_str() {
                    "completion_candidates" => CandidateKind::Completion,
                    "assist_candidates" => CandidateKind::Assist,
                    _ => CandidateKind::Hint,
                };
                let rows = records(value);
                for handle in &section.handles {
                    let Some(row) = rows.get(handle.index) else {
                        continue;
                    };
                    result.candidates.push(LazyCandidateFact {
                        kind,
                        index: handle.index,
                        fingerprint: handle.fingerprint.clone(),
                        label: row
                            .get("label")
                            .or_else(|| row.get("title"))
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        location: row.get("location").and_then(location),
                    });
                }
            }
            "related_test_candidates" => {
                for row in records(value) {
                    let runnable = row.get("runnable").unwrap_or(row);
                    let strings = |key: &str| {
                        runnable["args"][key]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    };
                    result.tests.push(TestCandidateFact {
                        test_id: None,
                        cfg: None,
                        relatedness: Some("native related-tests provider".into()),
                        eligible: None,
                        label: runnable
                            .get("label")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        location: runnable.get("location").and_then(location),
                        runnable_kind: runnable
                            .get("kind")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        cargo_args: strings("cargoArgs"),
                        executable_args: strings("executableArgs"),
                        original: row.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok((result, omitted))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_retains_unknown_and_original_lazy_identity() {
        let sections = vec![
            Section {
                category: "predicate_evaluation".into(),
                evidence: serde_json::json!({"status":"not_proven"}),
                omitted: 0,
                handles: vec![],
            },
            Section {
                category: "completion_candidates".into(),
                evidence: serde_json::json!([{"label":"same"}]),
                omitted: 3,
                handles: vec![super::super::CandidateHandle {
                    index: 0,
                    fingerprint: "original".into(),
                }],
            },
        ];
        let (facts, _) = project(&sections, "file:///source.rs").unwrap();
        assert_eq!(
            facts.predicate.unwrap().outcome,
            super::super::PredicateOutcome::NotProven
        );
        assert_eq!(facts.candidates[0].fingerprint, "original");
    }
    #[test]
    fn native_symbol_categories_keep_existing_filter_vocabulary() {
        assert_eq!(
            symbol_kind(&Value::String("TypeAlias".into())).as_deref(),
            Some("type_alias")
        );
        assert_eq!(
            symbol_kind(&Value::String("Const".into())).as_deref(),
            Some("constant")
        );
        assert_eq!(
            symbol_kind(&Value::String("Function".into())).as_deref(),
            Some("function")
        );
    }
    #[test]
    fn selected_definition_range_and_call_direction_are_typed() {
        let sections = vec![Section {
            category: "definition".into(),
            evidence: serde_json::json!([{"targetUri":"file:///a.rs","targetRange":{"start":{"line":0,"character":0},"end":{"line":5,"character":0}},"targetSelectionRange":{"start":{"line":1,"character":3},"end":{"line":1,"character":7}}}]),
            omitted: 2,
            handles: vec![],
        }];
        let (facts, omitted) = project(&sections, "file:///source.rs").unwrap();
        let FactLocation::Protocol { range, .. } = &facts.relationships[0].location else {
            panic!("protocol range required")
        };
        assert_eq!(range.start.character, 3);
        assert_eq!(omitted[&FactRelation::Definition], 2);
    }
    #[test]
    fn call_edges_keep_selected_identifier_instead_of_function_body() {
        let item = serde_json::json!({"name":"actual","uri":"file:///target.rs","range":{"start":{"line":0,"character":0},"end":{"line":20,"character":1}},"selectionRange":{"start":{"line":1,"character":3},"end":{"line":1,"character":9}}});
        let sections = vec![
            Section {
                category: "incoming_calls_one_hop".into(),
                evidence: serde_json::json!([{"from":item}]),
                omitted: 0,
                handles: vec![],
            },
            Section {
                category: "outgoing_calls_one_hop".into(),
                evidence: serde_json::json!([{"to":item}]),
                omitted: 0,
                handles: vec![],
            },
        ];
        let (facts, _) = project(&sections, "file:///source.rs").unwrap();
        assert_eq!(facts.relationships.len(), 2);
        assert_eq!(facts.relationships[0].direction, FactDirection::Incoming);
        assert_eq!(facts.relationships[1].direction, FactDirection::Outgoing);
        for fact in facts.relationships {
            assert_eq!(fact.name.as_deref(), Some("actual"));
            let FactLocation::Protocol { uri, range } = fact.location else {
                panic!("native protocol interval required")
            };
            assert_eq!(uri, "file:///target.rs");
            assert_eq!(range.start.line, 1);
            assert_eq!(range.start.character, 3);
            assert_eq!(range.end.line, 1);
            assert_eq!(range.end.character, 9);
        }
    }
}
