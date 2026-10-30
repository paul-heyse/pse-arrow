// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn query(pattern: &str) -> Query {
    Query {
        pattern: pattern.into(),
        strictness: Strictness::Smart,
        inside: vec![],
        has: vec![],
        excluded: vec![],
        captures: vec![],
        edition: RustEdition::E2024,
        max_records: None,
    }
}
fn source(text: &str) -> SourceText {
    SourceText::new("/source/example.rs".into(), text.into()).unwrap()
}

#[test]
fn native_grammar_abi_named_and_sequence_captures_retain_exact_unicode() {
    let original = source("fn α() { target(first, /* retained */ β); }");
    let result = analyze(&original, &query("target($A, $$$REST)"), None, usize::MAX).unwrap();
    assert_eq!(RustGrammar.get_ts_language().abi_version(), 15);
    assert_eq!(result.coverage.total_matches, 1);
    let found = &result.matches[0];
    assert_eq!(found.text, "target(first, /* retained */ β)");
    let first = found
        .captures
        .iter()
        .find(|capture| capture.name == "A")
        .unwrap();
    assert_eq!(first.nodes[0].text, "first");
    assert!(!first.sequence);
    let rest = found
        .captures
        .iter()
        .find(|capture| capture.name == "REST")
        .unwrap();
    assert!(rest.sequence);
    assert!(rest.nodes.iter().any(|node| node.text == "β"));
    for capture in &found.captures {
        for node in &capture.nodes {
            assert_eq!(node.locator.select(&original).unwrap(), node.text);
        }
    }
}

#[test]
fn nested_matches_remain_candidates_and_capture_preserving_preview_shares_proposal() {
    let original = source("fn f() { outer(inner(β)); }");
    let result = analyze(
        &original,
        &query("$CALL($ARG)"),
        Some("wrapped($ARG)"),
        usize::MAX,
    )
    .unwrap();
    assert_eq!(result.matches.len(), 2);
    assert_eq!(result.matches[0].text, "outer(inner(β))");
    assert_eq!(result.matches[1].text, "inner(β)");
    let proposal = ChangeProposal::new(
        ProducerContext::Syntax {
            backend: "ast-grep-core/0.51.0".into(),
            configuration: "smart;syntax only".into(),
        },
        vec![original.clone()],
        result.edits,
        vec![1],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(proposal.candidates().len(), 2);
    assert_eq!(proposal.conflicts().len(), 1);
    assert_eq!(proposal.inspect(0).unwrap().replacement, "wrapped(β)");
    assert_eq!(original.text(), "fn f() { outer(inner(β)); }");
}

#[test]
fn contexts_capture_selection_and_recovery_do_not_claim_resolution() {
    let original = source("fn first() { target(a); }\nfn second() { target(b); }\nfn malformed(\n");
    let mut selected = query("target($A)");
    selected.inside.push("fn first() { $$$BODY }".into());
    selected.captures.push("A".into());
    let result = analyze(&original, &selected, None, usize::MAX).unwrap();
    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].captures[0].nodes[0].text, "a");
    assert!(!result.coverage.recovery_regions.is_empty());
    assert!(!result.coverage.rust_parse_issues.is_empty());
    let mut selected = query("fn $NAME() { $$$BODY }");
    selected.has.push("target(b)".into());
    assert_eq!(
        analyze(&original, &selected, None, usize::MAX)
            .unwrap()
            .matches
            .len(),
        1
    );
    let mut selected = query("target($A)");
    selected.excluded.push("target(a)".into());
    assert_eq!(
        analyze(&original, &selected, None, usize::MAX)
            .unwrap()
            .matches[0]
            .text,
        "target(b)"
    );
}

#[test]
fn undefined_captures_templates_and_native_invalid_patterns_refuse() {
    let original = source("fn f() { call(a); }");
    assert!(
        matches!(analyze(&original,&query("call($A)"),Some("other($MISSING)"),1),Err(QueryError::UndefinedCapture(name)) if name=="MISSING")
    );
    let mut selected = query("call($A)");
    selected.captures.push("MISSING".into());
    assert!(matches!(
        analyze(&original, &selected, None, 1),
        Err(QueryError::UndefinedCapture(_))
    ));
    assert!(matches!(
        analyze(&original, &query(""), None, 1),
        Err(QueryError::Pattern(_))
    ));
    assert!(matches!(
        analyze(&original, &query("$$$ONLY"), None, 1),
        Err(QueryError::Pattern(_))
    ));
}

#[tokio::test]
async fn query_budget_and_preview_selection_use_shared_cpu_completion_without_writes() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("example.rs");
    let text = "fn f() { first(a); second(b); }";
    std::fs::write(&file, text).unwrap();
    let mut selected = query("$CALL($ARG)");
    selected.max_records = Some(1);
    let mut task = start_query(
        vec![SourceRequest::Working { path: file.clone() }],
        selected,
        CancellationToken::new(),
    );
    let response = task.settle().await.unwrap().unwrap();
    assert_eq!(response.payload.matches.len(), 1);
    assert_eq!(response.payload.total_matches, 2);
    assert_eq!(response.payload.omitted_matches, 1);
    assert_eq!(response.exit_code(), 0);
    assert!(response.payload.cpu_work_completed);
    let mut task = start_preview(
        vec![SourceRequest::Working { path: file.clone() }],
        query("first($ARG)"),
        "replacement($ARG)".into(),
        vec![0],
        CancellationToken::new(),
    );
    let response = task.settle().await.unwrap().unwrap();
    assert_eq!(
        response.payload.proposal.inspect(0).unwrap().original,
        "first(a)"
    );
    assert_eq!(
        response.payload.proposal.inspect(0).unwrap().replacement,
        "replacement(a)"
    );
    assert_eq!(std::fs::read_to_string(file).unwrap(), text);
}
