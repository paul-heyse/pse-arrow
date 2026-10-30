// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn source(path: &str, text: &str) -> WorkingSource {
    WorkingSource::new(path.into(), text.into()).unwrap()
}
fn syntax() -> ProducerContext {
    ProducerContext::Syntax {
        backend: "independent syntax example".into(),
        configuration: "Rust 2024; written tokens only".into(),
    }
}
fn proposal(
    sources: Vec<WorkingSource>,
    candidates: Vec<EditCandidate>,
    selected: Vec<usize>,
) -> Result<ChangeProposal, ProposalError> {
    ChangeProposal::new(syntax(), sources, candidates, selected, vec![], vec![])
}

#[test]
fn exact_original_digest_and_native_path_are_bound_without_acquisition() {
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
    let path = OsString::from_vec(b"/source/opaque-\xff.rs".to_vec());
    let original = WorkingSource::new(PathBuf::from(&path), "abc".into()).unwrap();
    assert_eq!(
        original.identity().path().as_os_str().as_bytes(),
        path.as_bytes()
    );
    assert_eq!(
        original.identity().source_sha256(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let encoded = serde_json::to_value(&original).unwrap();
    assert_eq!(encoded["identity"]["path"]["Unix"][15], 255);
    assert!(matches!(
        WorkingSource::new("relative.rs".into(), "abc".into()),
        Err(ProposalError::RelativePath(_))
    ));
}

#[test]
fn independent_syntax_and_semantic_inputs_share_exact_inspection() {
    let syntax_source = source(
        "/source/syntax.rs",
        "fn café() { old(); /* untouched */ }\n",
    );
    let syntax_edit = EditCandidate::new(syntax_source.identity().clone(), 13..16, "new".into());
    let syntax_proposal = proposal(vec![syntax_source], vec![syntax_edit], vec![0]).unwrap();
    let semantic_source = source("/source/semantic.rs", "fn α() { first(β); }\n");
    let semantic_edit =
        EditCandidate::new(semantic_source.identity().clone(), 10..15, "second".into());
    let semantic_proposal = ChangeProposal::new(
        ProducerContext::Semantic {
            backend: "independent resolved example".into(),
            configuration: "crate=demo,target=lib,features=[]".into(),
            synchronized_generation: "generation-7".into(),
        },
        vec![semantic_source],
        vec![semantic_edit],
        vec![0],
        vec!["macro-generated regions omitted".into()],
        vec![UnresolvedPlaceholder {
            candidate: 0,
            description: "review effect ordering".into(),
        }],
    )
    .unwrap();
    let syntax_view = syntax_proposal.inspect(0).unwrap();
    assert_eq!(
        (syntax_view.original, syntax_view.replacement),
        ("old", "new")
    );
    let semantic_view = semantic_proposal.inspect(0).unwrap();
    assert_eq!(
        (semantic_view.original, semantic_view.replacement),
        ("first", "second")
    );
    assert_eq!(
        syntax_proposal.sources()[0].text(),
        "fn café() { old(); /* untouched */ }\n"
    );
    assert_eq!(
        semantic_proposal.sources()[0].text(),
        "fn α() { first(β); }\n"
    );
    assert_eq!(
        semantic_proposal.unresolved_placeholders()[0].description,
        "review effect ordering"
    );
    assert_eq!(
        semantic_proposal.limitations(),
        ["macro-generated regions omitted"]
    );
    assert!(matches!(
        syntax_proposal.producer(),
        ProducerContext::Syntax { .. }
    ));
    assert!(matches!(
        semantic_proposal.producer(),
        ProducerContext::Semantic { .. }
    ));
    assert!(semantic_proposal.inspect(1).is_none());
}

#[test]
fn stale_digest_wrong_path_and_missing_evidence_refuse() {
    let original = source("/source/file.rs", "alpha");
    let stale = source("/source/file.rs", "bravo");
    let wrong = source("/source/other.rs", "alpha");
    let stale_edit = EditCandidate::new(stale.identity().clone(), 0..5, "new".into());
    assert!(matches!(
        proposal(vec![original.clone()], vec![stale_edit], vec![0]),
        Err(ProposalError::StaleSource(_))
    ));
    let wrong_edit = EditCandidate::new(wrong.identity().clone(), 0..5, "new".into());
    assert!(matches!(
        proposal(vec![original.clone()], vec![wrong_edit], vec![0]),
        Err(ProposalError::MissingSource(_))
    ));
    let edit = EditCandidate::new(original.identity().clone(), 0..5, "new".into());
    let checked = proposal(vec![original.clone()], vec![edit], vec![0]).unwrap();
    assert!(matches!(
        checked.validate_sources(&[stale]),
        Err(ProposalError::StaleSource(_))
    ));
    assert!(matches!(
        checked.validate_sources(&[wrong.clone()]),
        Err(ProposalError::MissingSource(_))
    ));
    checked
        .validate_sources(&[original.clone(), wrong])
        .unwrap();
    assert!(matches!(
        checked.validate_sources(&[original.clone(), original]),
        Err(ProposalError::DuplicateSource(_))
    ));
}

#[test]
fn utf8_boundaries_reversed_and_outside_ranges_refuse_even_unselected() {
    let original = source("/source/file.rs", "aé🦀z");
    for range in [2..3, 3..4, 8..9, Range { start: 5, end: 3 }] {
        let edit = EditCandidate::new(original.identity().clone(), range, "x".into());
        assert!(matches!(
            proposal(vec![original.clone()], vec![edit], vec![]),
            Err(ProposalError::InvalidRange(0))
        ));
    }
    let edit = EditCandidate::new(original.identity().clone(), 1..7, "λ".into());
    let checked = proposal(vec![original], vec![edit], vec![0]).unwrap();
    assert_eq!(checked.inspect(0).unwrap().original, "é🦀");
}

#[test]
fn all_matches_keep_conflicts_but_selected_overlaps_refuse() {
    let original = source("/source/file.rs", "outer(inner())");
    let candidates = vec![
        EditCandidate::new(original.identity().clone(), 0..14, "whole()".into()),
        EditCandidate::new(original.identity().clone(), 6..13, "part()".into()),
    ];
    let checked = proposal(vec![original.clone()], candidates.clone(), vec![1]).unwrap();
    assert_eq!(checked.candidates().len(), 2);
    assert_eq!(checked.selected(), [1]);
    assert_eq!(
        checked.conflicts(),
        [CandidateConflict {
            first: 0,
            second: 1
        }]
    );
    assert_eq!(checked.inspect(0).unwrap().original, "inner()");
    assert!(matches!(
        proposal(vec![original], candidates, vec![0, 1]),
        Err(ProposalError::SelectedOverlap {
            first: 0,
            second: 1
        })
    ));
}

#[test]
fn adjacent_edits_and_different_files_are_independent() {
    let left = source("/source/left.rs", "abcd");
    let right = source("/source/right.rs", "wxyz");
    let edits = vec![
        EditCandidate::new(left.identity().clone(), 0..2, "AB".into()),
        EditCandidate::new(left.identity().clone(), 2..4, "CD".into()),
        EditCandidate::new(right.identity().clone(), 0..4, "WXYZ".into()),
    ];
    let checked = proposal(vec![left, right], edits, vec![2, 1, 0]).unwrap();
    assert!(checked.conflicts().is_empty());
    assert_eq!(checked.inspect(0).unwrap().original, "wxyz");
    assert_eq!(checked.inspect(2).unwrap().original, "ab");
}

#[test]
fn insertion_conflicts_are_explicit_and_eof_insert_is_valid() {
    let original = source("/source/file.rs", "abcd");
    for (first, second) in [(1..1, 1..1), (1..1, 0..2), (1..1, 1..3)] {
        let edits = vec![
            EditCandidate::new(original.identity().clone(), first, "a".into()),
            EditCandidate::new(original.identity().clone(), second, "b".into()),
        ];
        assert!(matches!(
            proposal(vec![original.clone()], edits, vec![0, 1]),
            Err(ProposalError::SelectedOverlap { .. })
        ));
    }
    let edits = vec![
        EditCandidate::new(original.identity().clone(), 0..4, "new".into()),
        EditCandidate::new(original.identity().clone(), 4..4, "!".into()),
    ];
    let checked = proposal(vec![original], edits, vec![0, 1]).unwrap();
    assert_eq!(checked.inspect(1).unwrap().original, "");
    assert_eq!(checked.inspect(1).unwrap().replacement, "!");
}

#[test]
fn checked_set_rejects_invalid_selection_source_and_placeholder_combinations() {
    let original = source("/source/file.rs", "a");
    let candidate = EditCandidate::new(original.identity().clone(), 0..1, "b".into());
    assert!(matches!(
        proposal(vec![original.clone()], vec![candidate.clone()], vec![1]),
        Err(ProposalError::UnknownCandidate(1))
    ));
    assert!(matches!(
        proposal(vec![original.clone()], vec![candidate.clone()], vec![0, 0]),
        Err(ProposalError::DuplicateSelection(0))
    ));
    assert!(matches!(
        proposal(
            vec![original.clone(), original.clone()],
            vec![candidate.clone()],
            vec![0]
        ),
        Err(ProposalError::DuplicateSource(_))
    ));
    assert!(matches!(
        proposal(vec![original.clone()], vec![], vec![]),
        Err(ProposalError::UnusedSource(_))
    ));
    assert!(matches!(
        ChangeProposal::new(
            syntax(),
            vec![original],
            vec![candidate],
            vec![0],
            vec![],
            vec![UnresolvedPlaceholder {
                candidate: 1,
                description: "unbound".into()
            }]
        ),
        Err(ProposalError::UnknownCandidate(1))
    ));
}

#[test]
fn semantic_authority_requires_synchronized_context() {
    let producer = ProducerContext::Semantic {
        backend: "semantic example".into(),
        configuration: "crate=demo".into(),
        synchronized_generation: "  ".into(),
    };
    assert!(matches!(
        ChangeProposal::new(producer, vec![], vec![], vec![], vec![], vec![]),
        Err(ProposalError::MissingProducerContext)
    ));
    let producer = ProducerContext::Syntax {
        backend: "syntax example".into(),
        configuration: "".into(),
    };
    assert!(matches!(
        ChangeProposal::new(producer, vec![], vec![], vec![], vec![], vec![]),
        Err(ProposalError::MissingProducerContext)
    ));
}
