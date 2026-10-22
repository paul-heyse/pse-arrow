// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::tooling::review_analysis::{
    AnalysisQuestion, AnalysisRequest, AnalysisStatement, QuestionAnalysis,
};
use pse_diagnostics::{StatementKind, observation::OperationId};
use std::sync::Arc;

fn analysis() -> AnalysisDocument {
    let mut request = AnalysisRequest::new(OperationId {
        process_instance: [1, 2],
        sequence: u64::MAX,
    });
    request.question = Some(AnalysisQuestion::Preparation);
    AnalysisDocument {
        version: Version,
        selections: vec![crate::tooling::review_analysis::AnalysisSelection {
            operation: request.operation,
            process_clock: request.operation.process_instance,
            interval_start: None,
            interval_end: None,
            scope: crate::tooling::review_analysis::AnalysisSelectionScope::Unknown,
            aligned_to_request: true,
        }],
        request,
        requirements: Vec::new(),
        questions: vec![QuestionAnalysis {
            question: AnalysisQuestion::Preparation,
            statements: vec![AnalysisStatement {
                kind: StatementKind::Limitation,
                code: "fixture.failed_source".into(),
                message: format!(
                    "Recorded pre-handle scientific failure; exact selection {}. <script>untrusted</script>",
                    u64::MAX
                ),
                evidence: vec![],
                next: vec![],
            }],
        }],
        inspected_source_items: 0,
        omitted_source_items: 0,
        omitted_statements: 0,
    }
}
fn populated(path: &Path, selection: PackSelection, cancel: CancellationToken) -> PackSession {
    let mut session = PackSession::create(path, selection, cancel).unwrap();
    session.add_frozen(&FrozenEvidenceSet::default()).unwrap();
    session.add_analysis(&analysis()).unwrap();
    session
}
fn raw(id: &str, required: bool, role: PackMemberRole) -> PackMemberSelection {
    PackMemberSelection {
        id: id.into(),
        role,
        required,
        format: PackFormat::Text,
        document_version: None,
        source: Some("/original/unreachable/descriptive-only".into()),
        references: Vec::new(),
        result: None,
        capture_scope: Some("stopped; sampled loss unknown".into()),
        diagnostic_table: None,
    }
}
fn source(dir: &Path, bytes: &[u8]) -> File {
    let path = dir.join("source");
    std::fs::write(&path, bytes).unwrap();
    File::open(path).unwrap()
}
fn patch_index(path: &Path, f: impl FnOnce(&mut serde_json::Value)) {
    let file = path.join(INDEX);
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    f(&mut value);
    std::fs::write(file, serde_json::to_vec(&value).unwrap()).unwrap();
}
#[test]
fn offline_pack_relocation_exact_integers_reports_and_unindexed_neighbors() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("pack");
    let mut selection = PackSelection::default();
    selection
        .members
        .push(raw("detail", true, PackMemberRole::Detail));
    let mut session = populated(&destination, selection, CancellationToken::new());
    session
        .copy_prefix(
            "detail",
            source(root.path(), b"acknowledged\nunacknowledged tail"),
            13,
        )
        .unwrap();
    let index = session.seal().unwrap();
    assert!(index.membership_complete);
    assert!(!index.self_contained);
    let relocated = root.path().join("relocated");
    std::fs::rename(&destination, &relocated).unwrap();
    std::fs::remove_file(root.path().join("source")).unwrap();
    std::fs::write(relocated.join("source"), b"tempting live sibling").unwrap();
    let reader = PackReader::open(&relocated, 1 << 20).unwrap();
    let mut copied = String::new();
    reader
        .open_member("detail")
        .unwrap()
        .read_to_string(&mut copied)
        .unwrap();
    assert_eq!(copied, "acknowledged\n");
    assert!(reader.open_member("source").is_err());
    assert_eq!(
        reader.read_analysis().unwrap().request.operation.sequence,
        u64::MAX
    );
    let mut html = String::new();
    reader
        .open_member("report_html")
        .unwrap()
        .read_to_string(&mut html)
        .unwrap();
    assert!(html.contains("18446744073709551615"));
    assert!(html.contains("&lt;script"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("https://"));
    assert!(
        PackSession::create(
            &relocated,
            PackSelection::default(),
            CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn offline_pack_cancellation_output_allowance_and_short_source_never_seal() {
    let root = tempfile::tempdir().unwrap();
    for mode in ["cancel", "output", "short"] {
        let destination = root.path().join(mode);
        let cancel = CancellationToken::new();
        let mut selection = PackSelection::default();
        selection
            .members
            .push(raw("raw", true, PackMemberRole::Detail));
        let mut session = populated(&destination, selection, cancel.clone());
        if mode == "cancel" {
            cancel.cancel();
        }
        if mode == "output" {
            session.index.selection.maximum_output_bytes = session.written + 2;
        }
        assert!(
            session
                .copy_prefix("raw", source(root.path(), b"abc"), 8)
                .is_err()
        );
        assert!(session.seal().is_err());
        let reader = PackReader::open(&destination, 1 << 20).unwrap();
        assert_eq!(reader.index().publication, PackPublication::Incomplete);
        assert!(reader.read_frozen().is_ok());
        if mode == "short" {
            let mut prefix = String::new();
            reader
                .open_member("raw")
                .unwrap()
                .read_to_string(&mut prefix)
                .unwrap();
            assert_eq!(prefix, "abc");
        }
    }
}
#[test]
fn offline_pack_unknown_versions_duplicate_selection_and_unsafe_paths_refuse() {
    let root = tempfile::tempdir().unwrap();
    let duplicate = PackSelection {
        members: vec![
            raw("same", false, PackMemberRole::Capture),
            raw("same", false, PackMemberRole::Capture),
        ],
        ..PackSelection::default()
    };
    assert!(
        PackSession::create(
            &root.path().join("duplicate"),
            duplicate,
            CancellationToken::new()
        )
        .is_err()
    );
    for mode in [
        "version",
        "nested",
        "traversal",
        "absolute",
        "duplicate",
        "format",
        "dot",
        "separator",
    ] {
        let path = root.path().join(mode);
        populated(&path, PackSelection::default(), CancellationToken::new())
            .seal()
            .unwrap();
        if mode == "nested" {
            let reader = PackReader::open(&path, 1 << 20).unwrap();
            let member = reader
                .index
                .members
                .iter()
                .find(|m| m.selection.id == "frozen")
                .unwrap();
            let file = path.join(&member.relative_path);
            let mut value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
            value["version"] = serde_json::json!(9);
            std::fs::write(file, serde_json::to_vec(&value).unwrap()).unwrap();
        } else {
            patch_index(&path, |value| match mode {
                "version" => value["version"] = serde_json::json!(900),
                "traversal" => {
                    value["members"][0]["relative_path"] =
                        serde_json::json!("members/../../outside")
                }
                "absolute" => {
                    value["members"][0]["relative_path"] = serde_json::json!("/tmp/outside")
                }
                "dot" => {
                    value["members"][0]["relative_path"] = serde_json::json!(
                        value["members"][0]["relative_path"]
                            .as_str()
                            .unwrap()
                            .replace("members/", "members/./")
                    )
                }
                "separator" => {
                    value["members"][0]["relative_path"] = serde_json::json!(
                        value["members"][0]["relative_path"]
                            .as_str()
                            .unwrap()
                            .replace("members/", "members//")
                    )
                }
                "duplicate" => value["members"][1]["selection"]["id"] = serde_json::json!("frozen"),
                "format" => {
                    value["members"][0]["selection"]["format"] =
                        serde_json::json!("arrow_ipc_stream")
                }
                _ => panic!("unexpected index mutation {mode}"),
            });
        }
        assert!(PackReader::open(&path, 1 << 20).is_err(), "{mode}");
    }
}
#[cfg(unix)]
#[test]
fn offline_pack_rejects_symlink_files_and_directories() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    for directory in [false, true] {
        let path = root
            .path()
            .join(if directory { "directory" } else { "file" });
        populated(&path, PackSelection::default(), CancellationToken::new())
            .seal()
            .unwrap();
        if directory {
            let original = path.join("members");
            let outside = root.path().join("outside-members");
            std::fs::rename(&original, &outside).unwrap();
            symlink(outside, original).unwrap();
        } else {
            let reader = PackReader::open(&path, 1 << 20).unwrap();
            let member = &reader.index.members[0];
            let file = path.join(&member.relative_path);
            let outside = root.path().join("outside-frozen");
            std::fs::rename(&file, &outside).unwrap();
            symlink(outside, file).unwrap();
        }
        assert!(PackReader::open(&path, 1 << 20).is_err());
    }
}
#[test]
fn offline_pack_missing_context_optional_capture_and_science_are_independent() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let selection = PackSelection {
        members: vec![
            raw("context", true, PackMemberRole::Context),
            raw("profiler", false, PackMemberRole::Capture),
        ],
        ..PackSelection::default()
    };
    let mut session = populated(&path, selection, CancellationToken::new());
    session
        .unavailable("context", "original inputs were not retained".into())
        .unwrap();
    session
        .unavailable(
            "profiler",
            "optional capture failed before producing data".into(),
        )
        .unwrap();
    let index = session.seal().unwrap();
    assert_eq!(index.publication, PackPublication::Sealed);
    assert!(!index.membership_complete);
    assert!(!index.self_contained);
    let reader = PackReader::open(&path, 1 << 20).unwrap();
    assert!(
        reader.read_analysis().unwrap().questions[0].statements[0]
            .message
            .contains("scientific failure")
    );
    assert!(reader.open_member("unrequested").is_err());
    assert!(matches!(
        reader
            .index
            .members
            .iter()
            .find(|m| m.selection.id == "profiler")
            .unwrap()
            .availability,
        PackAvailability::Unavailable { .. }
    ));
}
#[test]
fn offline_pack_selected_membership_exceeds_a_single_inventory_page() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let selection = PackSelection {
        members: (0..129)
            .map(|i| raw(&format!("child-{i}"), false, PackMemberRole::Context))
            .collect(),
        ..PackSelection::default()
    };
    let mut session = populated(&path, selection, CancellationToken::new());
    for i in 0..129 {
        session
            .unavailable(
                &format!("child-{i}"),
                "explicit original historical attempt, metadata unavailable".into(),
            )
            .unwrap();
    }
    let index = session.seal().unwrap();
    assert_eq!(index.members.len(), 133);
    let reader = PackReader::open(&path, 1 << 20).unwrap();
    assert_eq!(
        reader
            .index
            .members
            .iter()
            .filter(|m| m.selection.role == PackMemberRole::Context)
            .count(),
        129
    );
    assert!(
        reader
            .index
            .members
            .iter()
            .any(|m| m.selection.id == "child-128")
    );
}
#[test]
fn offline_pack_incomplete_supported_read_and_optional_copy_failure() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut selection = PackSelection::default();
    selection
        .members
        .push(raw("optional", false, PackMemberRole::Capture));
    let mut session = populated(&path, selection, CancellationToken::new());
    assert_eq!(
        PackReader::open(&path, 1 << 20).unwrap().index.publication,
        PackPublication::Incomplete
    );
    assert!(
        session
            .copy_file(&PackFileInput {
                member: "optional".into(),
                path: root.path().join("absent"),
                readable_prefix_bytes: 0
            })
            .is_err()
    );
    assert_eq!(session.seal().unwrap().publication, PackPublication::Sealed);
    assert!(PackReader::open(&path, 1 << 20).is_ok());
}
#[test]
fn offline_pack_source_references_resolve_to_original_selected_members() {
    use crate::tooling::review_analysis::AnalysisSourceReference;
    use pse_diagnostics::observation::Value;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut document = analysis();
    document.questions[0].statements[0]
        .evidence
        .push(AnalysisSourceReference {
            member: "frozen".into(),
            pointer: "/version".into(),
            event: None,
            value: Value::Count(2),
            partial: false,
        });
    let mut session =
        PackSession::create(&path, PackSelection::default(), CancellationToken::new()).unwrap();
    session.add_frozen(&FrozenEvidenceSet::default()).unwrap();
    session.add_analysis(&document).unwrap();
    session.seal().unwrap();
    let reader = PackReader::open(&path, 1 << 20).unwrap();
    let frozen = &reader
        .index
        .members
        .iter()
        .find(|m| m.selection.id == "frozen")
        .unwrap()
        .relative_path;
    let mut markdown = String::new();
    reader
        .open_member("report_markdown")
        .unwrap()
        .read_to_string(&mut markdown)
        .unwrap();
    let filename = Path::new(frozen).file_name().unwrap().to_str().unwrap();
    assert!(markdown.contains(&format!("{filename}#/version")));
    let mut html = String::new();
    reader
        .open_member("report_html")
        .unwrap()
        .read_to_string(&mut html)
        .unwrap();
    assert!(html.contains(&format!("href=\"{filename}#/version\"")));
    assert!(!html.contains("<pre>"));
    let analysis_member = reader
        .index
        .members
        .iter()
        .find(|m| m.selection.id == "analysis")
        .unwrap();
    let original_path = path.join(&analysis_member.relative_path);
    let original = std::fs::read(&original_path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["questions"][0]["statements"][0]["evidence"][0]["member"] = serde_json::json!("absent");
    let changed = serde_json::to_vec(&value).unwrap();
    std::fs::write(&original_path, &changed).unwrap();
    patch_index(&path, |index| {
        index["members"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|m| m["selection"]["id"] == "analysis")
            .unwrap()["readable_bytes"] = serde_json::json!(changed.len())
    });
    assert!(PackReader::open(&path, 1 << 20).is_err());
}
#[test]
fn offline_pack_metadata_and_report_bounds_refuse_without_seal() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut session =
        PackSession::create(&path, PackSelection::default(), CancellationToken::new()).unwrap();
    session.add_frozen(&FrozenEvidenceSet::default()).unwrap();
    session.index.selection.maximum_document_bytes = 64;
    assert!(session.add_analysis(&analysis()).is_err());
    assert!(session.seal().is_err());
    // Last atomic index remains an honest readable incomplete publication.
    assert_eq!(
        PackReader::open(&path, 1 << 20).unwrap().index.publication,
        PackPublication::Incomplete
    );
}
#[test]
fn offline_pack_pinned_source_replacement_and_unacknowledged_tail_do_not_change_selection() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut selection = PackSelection::default();
    selection
        .members
        .push(raw("detail", true, PackMemberRole::Detail));
    let mut session = populated(&path, selection, CancellationToken::new());
    let file = source(root.path(), b"record\n");
    let original = root.path().join("source");
    OpenOptions::new()
        .append(true)
        .open(&original)
        .unwrap()
        .write_all(b"unacknowledged tail")
        .unwrap();
    std::fs::rename(&original, root.path().join("old-inode")).unwrap();
    std::fs::write(&original, b"different active writer").unwrap();
    session.copy_prefix("detail", file, 7).unwrap();
    session.seal().unwrap();
    let reader = PackReader::open(&path, 1 << 20).unwrap();
    let mut copied = String::new();
    reader
        .open_member("detail")
        .unwrap()
        .read_to_string(&mut copied)
        .unwrap();
    assert_eq!(copied, "record\n");
    assert_eq!(std::fs::read(original).unwrap(), b"different active writer");
    assert_eq!(
        std::fs::read(root.path().join("old-inode")).unwrap(),
        b"record\nunacknowledged tail"
    );
}
#[test]
fn offline_pack_supplied_file_identity_preflight_precedes_copy_effects() {
    let root = tempfile::tempdir().unwrap();
    let file_path = root.path().join("input");
    std::fs::write(&file_path, b"retained").unwrap();
    for duplicate in [true, false] {
        let path = root
            .path()
            .join(if duplicate { "duplicate" } else { "orphan" });
        let mut selection = PackSelection::default();
        selection
            .members
            .push(raw("selected", false, PackMemberRole::Capture));
        let mut session = populated(&path, selection, CancellationToken::new());
        let inputs = vec![
            PackFileInput {
                member: "selected".into(),
                path: file_path.clone(),
                readable_prefix_bytes: 8,
            },
            PackFileInput {
                member: if duplicate { "selected" } else { "orphan" }.into(),
                path: file_path.clone(),
                readable_prefix_bytes: 8,
            },
        ];
        assert!(session.copy_files(&inputs).is_err());
        assert!(session.seal().is_err());
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        assert_eq!(
            reader
                .index
                .members
                .iter()
                .find(|m| m.selection.id == "selected")
                .unwrap()
                .availability,
            PackAvailability::Pending
        );
    }
}
#[test]
fn offline_pack_declared_json_versions_are_checked_before_availability_and_seal() {
    let root = tempfile::tempdir().unwrap();
    for (name, role, format, declared, bytes) in [
        (
            "json",
            PackMemberRole::Context,
            PackFormat::Json,
            1,
            b"{\"version\":2,\"meaning\":\"wrong original codec\"}".as_slice(),
        ),
        (
            "jsonl-version",
            PackMemberRole::Detail,
            PackFormat::JsonLines,
            2,
            b"{\"kind\":\"header\",\"value\":{\"version\":3}}\n".as_slice(),
        ),
        (
            "jsonl-tail",
            PackMemberRole::Detail,
            PackFormat::JsonLines,
            2,
            b"{\"kind\":\"header\",\"value\":{\"version\":2}}\n{\"partial\":".as_slice(),
        ),
    ] {
        let path = root.path().join(name);
        let mut member = raw("context", true, role);
        member.format = format;
        member.document_version = Some(declared);
        let mut selection = PackSelection::default();
        selection.members.push(member);
        let mut session = populated(&path, selection, CancellationToken::new());
        assert!(
            session
                .copy_prefix("context", source(root.path(), bytes), bytes.len() as u64)
                .is_err()
        );
        assert!(session.seal().is_err());
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        let context = reader
            .index
            .members
            .iter()
            .find(|m| m.selection.id == "context")
            .unwrap();
        assert!(matches!(
            context.availability,
            PackAvailability::Failed { .. }
        ));
        assert!(!context.finalized);
    }
}
#[test]
fn offline_pack_event_identity_and_field_ordinal_match_the_original_source() {
    use crate::tooling::review_analysis::{AnalysisEventIdentity, AnalysisSourceReference};
    use pse_diagnostics::observation::{CoherentRecord, ProducerId, RecordField, Value};
    let root = tempfile::tempdir().unwrap();
    for mode in ["matching", "wrong-sequence", "wrong-field"] {
        let mut document = analysis();
        let producer = ProducerId {
            operation: document.request.operation,
            sequence: 9,
        };
        let event = CoherentRecord {
            producer,
            generation: 4,
            sequence: u64::MAX,
            family_sequence: Some(8),
            observed_at: std::time::Duration::ZERO,
            family: "fixture.v1".into(),
            phase: "recorded".into(),
            coordinates: None,
            norm: None,
            values: vec![RecordField {
                name: "exact".into(),
                unit: "calls".into(),
                value: Value::Count(u64::MAX),
                availability: None,
            }],
        };
        document.questions[0].statements[0]
            .evidence
            .push(AnalysisSourceReference {
                member: "frozen".into(),
                pointer: "/records/0/values/0/value".into(),
                event: Some(AnalysisEventIdentity {
                    producer,
                    generation: 4,
                    sequence: if mode == "wrong-sequence" {
                        7
                    } else {
                        u64::MAX
                    },
                    family_sequence: Some(8),
                    field_ordinal: Some(if mode == "wrong-field" { 1 } else { 0 }),
                }),
                value: Value::Count(u64::MAX),
                partial: true,
            });
        let evidence = FrozenEvidenceSet {
            records: vec![event],
            ..Default::default()
        };
        let path = root.path().join(mode);
        let mut session =
            PackSession::create(&path, PackSelection::default(), CancellationToken::new()).unwrap();
        session.add_frozen(&evidence).unwrap();
        if mode == "matching" {
            session.add_analysis(&document).unwrap();
            session.seal().unwrap();
            assert!(PackReader::open(&path, 1 << 20).is_ok());
        } else {
            assert!(session.add_analysis(&document).is_err());
            assert!(session.seal().is_err());
            assert_eq!(
                PackReader::open(&path, 1 << 20).unwrap().index.publication,
                PackPublication::Incomplete
            );
        }
    }
}
#[test]
fn offline_pack_required_index_publication_failure_latches_incomplete() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut selection = PackSelection::default();
    selection
        .members
        .push(raw("optional", false, PackMemberRole::Capture));
    let mut session = populated(&path, selection, CancellationToken::new());
    // An independently created same-name temporary refuses this metadata write.
    let collision = path.join(format!(".index-{}", session.next_index));
    std::fs::write(collision, b"unindexed collision").unwrap();
    assert!(
        session
            .unavailable("optional", "not retained".into())
            .is_err()
    );
    assert!(session.seal().is_err());
    let reader = PackReader::open(&path, 1 << 20).unwrap();
    assert_eq!(reader.index.publication, PackPublication::Incomplete);
    assert!(matches!(
        reader
            .index
            .members
            .iter()
            .find(|m| m.selection.id == "optional")
            .unwrap()
            .availability,
        PackAvailability::Unavailable { .. }
    ));
    assert!(!reader.index.errors.is_empty());
}

#[test]
fn plan47f_member_report_and_final_seal_failures_preserve_independent_facts() {
    use pse_diagnostics::observation::{ObservationHub, Outcome, Policy};
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Boundary {
        Member,
        Report,
        FinalSeal,
        Cancel,
    }
    let root = tempfile::tempdir().unwrap();
    for boundary in [
        Boundary::Member,
        Boundary::Report,
        Boundary::FinalSeal,
        Boundary::Cancel,
    ] {
        let hub = ObservationHub::new(Policy::default());
        let science = hub.start("independent scientific owner");
        let science_reader = science.reader();
        let path = root.path().join(format!("{boundary:?}"));
        let cancel = CancellationToken::new();
        let mut selection = PackSelection::default();
        selection
            .members
            .push(raw("selected", true, PackMemberRole::Detail));
        let mut session = populated(&path, selection, cancel.clone());
        let original = b"acknowledged original record\n";
        let copied = session.copy_prefix(
            "selected",
            source(root.path(), original),
            original.len() as u64 + u64::from(boundary == Boundary::Member),
        );
        match boundary {
            Boundary::Member => {
                copied.unwrap_err();
            }
            Boundary::Report => {
                copied.unwrap();
                // Fail the second report after Markdown has finalized.
                std::fs::create_dir(path.join(".staging/report_html")).unwrap();
            }
            Boundary::FinalSeal => {
                copied.unwrap();
                // Each report publishes once. Only the final sealed index collides.
                std::fs::write(
                    path.join(format!(".index-{}", session.next_index + 2)),
                    b"owned final index fault",
                )
                .unwrap();
            }
            Boundary::Cancel => {
                copied.unwrap();
                cancel.cancel();
            }
        }
        session.seal().unwrap_err();
        assert_eq!(
            science_reader
                .snapshot(std::time::Duration::from_secs(60))
                .outcome,
            None
        );
        science.finish(Outcome::Completed);
        assert_eq!(
            science_reader
                .snapshot(std::time::Duration::from_secs(60))
                .outcome,
            Some(Outcome::Completed)
        );
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        assert_eq!(reader.index().publication, PackPublication::Incomplete);
        assert_eq!(reader.read_frozen().unwrap().version, Version::<2>);
        assert_eq!(
            reader.read_analysis().unwrap().request.operation.sequence,
            u64::MAX
        );
        let mut retained = Vec::new();
        reader
            .open_member("selected")
            .unwrap()
            .read_to_end(&mut retained)
            .unwrap();
        assert_eq!(retained, original);
        let selected = reader
            .index()
            .members
            .iter()
            .find(|member| member.selection.id == "selected")
            .unwrap();
        assert_eq!(selected.finalized, boundary != Boundary::Member);
        if matches!(boundary, Boundary::Report | Boundary::FinalSeal) {
            assert!(reader.open_member("report_markdown").is_ok());
            assert_eq!(
                reader.open_member("report_html").is_ok(),
                boundary == Boundary::FinalSeal
            );
        }
    }
}

#[test]
fn question_closure_distinguishes_missing_physical_context_and_optional_omissions() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    let root = tempfile::tempdir().unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("original recorded operation");
    let snapshot = operation
        .reader()
        .snapshot(std::time::Duration::from_secs(60));
    let mut evidence = FrozenEvidenceSet {
        operations: vec![snapshot],
        ..Default::default()
    };
    let id = operation.context().id();
    evidence.contexts.push(
        crate::workflow::ReviewContextDocument::pre_handle_failure(
            Default::default(),
            Some(id),
            "original pre-handle failure".into(),
            vec![],
        )
        .unwrap(),
    );
    for question in [
        AnalysisQuestion::Waiting,
        AnalysisQuestion::PhysicalAttribution,
        AnalysisQuestion::Comparability,
    ] {
        let path = root.path().join(format!("{question:?}"));
        let mut request = AnalysisRequest::new(id);
        request.question = Some(question);
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let document =
            crate::tooling::review_analysis::analyze(&evidence, &request, &pool, &CancellationToken::new())
                .unwrap();
        let mut selection = PackSelection::default();
        selection
            .members
            .push(raw("optional_profile", false, PackMemberRole::Capture));
        let mut session = PackSession::create(&path, selection, CancellationToken::new()).unwrap();
        session.add_frozen(&evidence).unwrap();
        session.add_analysis(&document).unwrap();
        session
            .unavailable(
                "optional_profile",
                "capability not selected for this original operation".into(),
            )
            .unwrap();
        let index = session.seal().unwrap();
        assert_eq!(index.self_contained, question == AnalysisQuestion::Waiting);
        assert!(!index.membership_complete);
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        assert_eq!(reader.index().self_contained, index.self_contained);
        if question != AnalysisQuestion::Waiting {
            assert!(
                index
                    .missing_context
                    .iter()
                    .any(|reason| reason.contains("requires original"))
            );
        }
    }
}

#[test]
fn required_nonfrozen_json_pointer_resolves_actual_payload_on_seal_and_readback() {
    let root = tempfile::tempdir().unwrap();
    for pointer in ["/present", "/absent", "/facts/0/payload/absent"] {
        let path = root.path().join(if pointer == "/present" {
            "present"
        } else if pointer == "/absent" {
            "absent"
        } else {
            "payload"
        });
        let mut context = raw("context", true, PackMemberRole::Context);
        context.format = PackFormat::Json;
        context.document_version = Some(1);
        let mut query = raw("query", true, PackMemberRole::Query);
        query.references.push(PackSourceReference {
            member: "context".into(),
            pointer: pointer.into(),
        });
        let mut selection = PackSelection::default();
        selection.members.extend([context, query]);
        let mut session = populated(&path, selection, CancellationToken::new());
        let bytes =
            br#"{"version":1,"present":18446744073709551615,"facts":[{"payload":[123,125]}]}"#;
        session
            .copy_prefix("context", source(root.path(), bytes), bytes.len() as u64)
            .unwrap();
        session
            .copy_prefix("query", source(root.path(), b"query"), 5)
            .unwrap();
        if pointer != "/present" {
            assert!(session.seal().is_err());
            continue;
        }
        session.seal().unwrap();
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        let local = path.join(
            &reader
                .index
                .members
                .iter()
                .find(|member| member.selection.id == "context")
                .unwrap()
                .relative_path,
        );
        let changed =
            br#"{"version":1,"absent!":18446744073709551615,"facts":[{"payload":[123,125]}]}"#;
        std::fs::write(local, changed).unwrap();
        patch_index(&path, |index| {
            index["members"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|member| member["selection"]["id"] == "context")
                .unwrap()["readable_bytes"] = serde_json::json!(changed.len());
        });
        assert!(PackReader::open(&path, 1 << 20).is_err());
    }
}

#[test]
fn binary_field_coordinates_are_explicit_missing_interpretation() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut query = raw("query", true, PackMemberRole::Query);
    query.references.push(PackSourceReference {
        member: "raw_profile".into(),
        pointer: "/absent".into(),
    });
    let mut selection = PackSelection::default();
    selection
        .members
        .extend([query, raw("raw_profile", true, PackMemberRole::Capture)]);
    let mut session = populated(&path, selection, CancellationToken::new());
    session
        .copy_prefix("query", source(root.path(), b"query"), 5)
        .unwrap();
    session
        .copy_prefix("raw_profile", source(root.path(), b"raw"), 3)
        .unwrap();
    let index = session.seal().unwrap();
    assert!(!index.self_contained);
    assert!(
        index
            .missing_context
            .iter()
            .any(|reason| reason.contains("whole member"))
    );
    assert!(PackReader::open(&path, 1 << 20).is_ok());
}

#[test]
fn diagnostic_registry_ipc_has_independent_schema_and_exact_html_cells() {
    use crate::tooling::review_query::{ReviewTableKind, ReviewTableRequest};
    use pse_diagnostics::observation::{CoherentRecord, ProducerId, RecordField, Value};
    let root = tempfile::tempdir().unwrap();
    let document = analysis();
    let record = CoherentRecord {
        producer: ProducerId {
            operation: document.request.operation,
            sequence: 1,
        },
        generation: 0,
        sequence: u64::MAX,
        family_sequence: Some(1),
        observed_at: std::time::Duration::ZERO,
        family: "fixture.v1".into(),
        phase: "original".into(),
        coordinates: None,
        norm: None,
        values: vec![RecordField {
            name: "exact UInt64".into(),
            unit: "count".into(),
            value: Value::Count(u64::MAX),
            availability: None,
        }],
    };
    let evidence = FrozenEvidenceSet {
        detail_pages: vec![crate::observation_detail::DetailRecordPage {
            version: Version,
            records: vec![record],
            producers: vec![],
            activations: vec![],
            next_offset: None,
            partial: false,
        }],
        ..Default::default()
    };
    let request = ReviewTableRequest {
        version: Version,
        kind: ReviewTableKind::Detail,
        index: 0,
        memory_bytes: 1 << 20,
    };
    let relation = crate::tooling::review_query::table_relation(&request);
    let registry = pse_schema::registry().unwrap();
    let schema =
        pse_schema::arrow::relation_schema_ref(registry, registry.relation(relation).unwrap())
            .unwrap();
    let version = schema.metadata()[pse_schema::arrow::KEY_CONTRACT_VERSION]
        .parse::<u32>()
        .unwrap();
    for invalid_schema in [false, true] {
        let path = root
            .path()
            .join(if invalid_schema { "invalid" } else { "valid" });
        let mut member = raw("diagnostic", true, PackMemberRole::Detail);
        member.format = PackFormat::ArrowIpcStream;
        member.diagnostic_table = Some(PackDiagnosticTable {
            request: request.clone(),
            relation: relation.into(),
            schema_version: if invalid_schema { version + 1 } else { version },
            source: PackSourceReference {
                member: "frozen".into(),
                pointer: crate::tooling::review_query::table_source(&request),
            },
        });
        let mut selection = PackSelection::default();
        selection.members.push(member);
        let mut session = PackSession::create(&path, selection, CancellationToken::new()).unwrap();
        session.add_frozen(&evidence).unwrap();
        session.add_analysis(&document).unwrap();
        if invalid_schema {
            assert!(
                session
                    .export_table("diagnostic", &evidence, &request)
                    .is_err()
            );
            assert!(session.seal().is_err());
            continue;
        }
        session
            .export_table("diagnostic", &evidence, &request)
            .unwrap();
        session.seal().unwrap();
        let reader = PackReader::open(&path, 1 << 20).unwrap();
        let mut html = String::new();
        reader
            .open_member("report_html")
            .unwrap()
            .read_to_string(&mut html)
            .unwrap();
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>18446744073709551615</td>"));
        assert!(html.contains("<a href=\""));
        assert!(!html.contains("<pre>"));
        assert!(html.contains("diagnostic.arrow#"));
    }
}

#[test]
fn plan47f_all_table_kinds_compare_only_the_selected_finalized_source() {
    use crate::tooling::capture::{CaptureDocument, CaptureKind, CaptureStatus};
use crate::observation_detail::{DetailRecordPage};
    use crate::invocation::{InvocationEndpoints, InvocationEvidenceDocument, InvocationOutcome};
    use crate::tooling::review_query::{
        ReviewQuery, ReviewQueryDocument, ReviewQueryRequest, ReviewQueryStatus, ReviewTableKind,
        ReviewTableRequest,
    };
    use pse_diagnostics::observation::{CoherentRecord, ProducerId};
    use std::time::Duration;
    fn evidence() -> FrozenEvidenceSet {
        let operation = analysis().request.operation;
        let run = pse_model::generated::identities::RunId::from_bytes([73; 16]);
        let mut set = FrozenEvidenceSet::default();
        set.detail_pages.push(DetailRecordPage {
            version: Version,
            records: vec![CoherentRecord {
                producer: ProducerId {
                    operation,
                    sequence: 1,
                },
                generation: 0,
                sequence: 1,
                family_sequence: Some(1),
                observed_at: Duration::ZERO,
                family: "original.opaque.v1".into(),
                phase: "original".into(),
                coordinates: None,
                norm: None,
                values: vec![],
            }],
            producers: vec![],
            activations: vec![],
            next_offset: None,
            partial: false,
        });
        set.invocations.push(InvocationEvidenceDocument {
            version: 2,
            run_id: run,
            attempt_id: None,
            operation: Some(operation),
            family: "original".into(),
            clock_scope: "original recorded invocation".into(),
            timing_unavailable: None,
            canonical_run: None,
            canonical_attempt: None,
            outcome: InvocationOutcome::Failed,
            endpoints: InvocationEndpoints::default(),
            intervals: vec![],
            omitted_intervals: 0,
            products: vec![],
            omitted_product_steps: 0,
            preparations: None,
            attempts: Arc::new(crate::workflow::AttemptSummary::empty_for_test(run)),
        });
        // Recorded metadata only. No profile was captured and no sampling/CPU claim follows.
        set.captures.push(CaptureDocument {
            version: Version,
            kind: CaptureKind::Timeline,
            operation: Some(operation),
            related_operations: vec![],
            launch_process: None,
            status: CaptureStatus::Stopped,
            requested_duration: Duration::ZERO,
            observed_duration: Duration::ZERO,
            targets: vec![],
            outputs: vec![],
            loss: Default::default(),
            symbols: Default::default(),
            open_intervals: 0,
            errors: vec![],
        });
        set.engine_observations
            .push(pse_engine::session::CopiedExecutionObservations {
                policy: Some(pse_engine::session::CopiedObservationPolicy::Contract),
                operation: Some(operation),
                dropped: 0,
                observations: vec![pse_engine::session::CopiedPlanObservation {
                    operation: Some(operation),
                    policy: pse_engine::session::CopiedObservationPolicy::Contract,
                    logical_plan: "original copied plan".into(),
                    physical_plan: None,
                    rules: vec![],
                    diagnostic: None,
                    execution: None,
                }],
            });
        set.query_results.push(ReviewQueryDocument {
            version: Version,
            request: ReviewQueryRequest {
                version: Version,
                input_member: "original-unavailable-capture".into(),
                question: ReviewQuery::Coverage,
                clock_scope: "original capture clock".into(),
                interval: None,
                timeout_seconds: 1.0,
                output_bytes: 1024,
            },
            query_version: 1,
            sql: "original query".into(),
            tool_version: None,
            status: ReviewQueryStatus::Unavailable,
            exit_code: None,
            columns: vec![],
            covered_interval: None,
            output: "original.csv".into(),
            output_prefix_bytes: 0,
            error_output: "original.stderr".into(),
            error_prefix_bytes: 0,
            rows: None,
            limitations: vec!["recorded unavailable query; no tool execution".into()],
        });
        set.contexts.push(
            crate::workflow::ReviewContextDocument::pre_handle_failure(
                Default::default(),
                Some(operation),
                "unselected opaque attachment".into(),
                vec![crate::workflow::ReviewContextFact {
                    path: "/original_inputs".into(),
                    codec: "physical-input-bits.v1".into(),
                    payload: serde_json::to_vec(&"x".repeat(2 << 20)).unwrap(),
                }],
            )
            .unwrap(),
        );
        set
    }
    let directory = tempfile::tempdir().unwrap();
    for kind in [
        ReviewTableKind::Detail,
        ReviewTableKind::Invocation,
        ReviewTableKind::Capture,
        ReviewTableKind::Engine,
        ReviewTableKind::Query,
    ] {
        for mode in ["unrelated", "changed", "missing", "tiny"] {
            let path = directory.path().join(format!("{kind:?}-{mode}"));
            let mut supplied = evidence();
            let request = ReviewTableRequest {
                version: Version,
                kind,
                index: usize::from(mode == "missing"),
                memory_bytes: if mode == "tiny" { 1 } else { 1 << 20 },
            };
            // A 4 KiB comparison limit admits each selected source while the unrelated input
            // attachment alone exceeds 2 MiB. This challenges selection before serialization.
            assert_eq!(
                same_table_source(&supplied, &supplied, &request, 4096).unwrap(),
                mode != "missing"
            );
            let mut selected = raw("diagnostic", true, PackMemberRole::Detail);
            selected.format = PackFormat::ArrowIpcStream;
            selected.diagnostic_table = Some(PackDiagnosticTable::new(request.clone()).unwrap());
            let mut selection = PackSelection::default();
            selection.members.push(selected);
            let mut session =
                PackSession::create(&path, selection, CancellationToken::new()).unwrap();
            session.add_frozen(&supplied).unwrap();
            session.add_analysis(&analysis()).unwrap();
            // Caller-side unrelated data may differ; only the requested original source is
            // compared, and the exported rows must still come from the finalized frozen copy.
            supplied.contexts[0].facts[0].payload =
                br#"{"different":"unselected caller input"}"#.to_vec();
            if mode == "changed" {
                match kind {
                    ReviewTableKind::Detail => supplied.detail_pages[0].partial = true,
                    ReviewTableKind::Invocation => {
                        supplied.invocations[0].family = "changed".into()
                    }
                    ReviewTableKind::Capture => supplied.captures[0].errors.push("changed".into()),
                    ReviewTableKind::Engine => supplied.engine_observations[0].dropped = 1,
                    ReviewTableKind::Query => supplied.query_results[0].sql = "changed".into(),
                }
            }
            let exported = session.export_table("diagnostic", &supplied, &request);
            if mode == "unrelated" {
                exported.unwrap();
                session.seal().unwrap();
                let reader = PackReader::open(&path, MAXIMUM_PACK_DOCUMENT_BYTES).unwrap();
                let mut stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
                    reader.open_member("diagnostic").unwrap(),
                    None,
                )
                .unwrap();
                let batch = stream.next().unwrap().unwrap();
                assert_eq!(batch.num_rows(), 1);
                assert!(stream.next().is_none());
                // Finalized content, including unrelated original context, was not replaced.
                let frozen = reader.read_frozen().unwrap();
                assert!(frozen.contexts[0].facts[0].payload.len() > 2 << 20);
                assert!(same_table_source(&frozen, &supplied, &request, 4096).unwrap());
                assert!(same_table_source(&frozen, &supplied, &request, 1).is_err());
            } else {
                let error = exported.unwrap_err().to_string();
                if mode != "tiny" {
                    assert!(error.contains("source differs from finalized frozen member"));
                }
                session.seal().unwrap_err();
                if mode == "missing" {
                    // The selected descriptor itself has a dangling frozen coordinate.
                    // A whole-pack reader must refuse it even though publication never
                    // sealed; independent finalized file prefixes remain usable on their
                    // explicit supported document route.
                    let error = PackReader::open(&path, MAXIMUM_PACK_DOCUMENT_BYTES)
                        .unwrap_err()
                        .to_string();
                    assert!(error.contains(&format!(
                        "source pointer frozen#{} is absent",
                        crate::tooling::review_query::table_source(&request)
                    )));
                    let index: PackIndex = decode(
                        File::open(path.join(INDEX)).unwrap(),
                        MAXIMUM_PACK_DOCUMENT_BYTES,
                    )
                    .unwrap();
                    assert_eq!(index.publication, PackPublication::Incomplete);
                    let diagnostic = index
                        .members
                        .iter()
                        .find(|member| member.selection.id == "diagnostic")
                        .unwrap();
                    assert!(matches!(
                        diagnostic.availability,
                        PackAvailability::Failed { .. }
                    ));
                    assert_eq!(diagnostic.readable_bytes, 0);
                    let frozen = index
                        .members
                        .iter()
                        .find(|member| member.selection.id == "frozen")
                        .unwrap();
                    assert_eq!(frozen.availability, PackAvailability::Available);
                    assert!(frozen.finalized);
                    let source = File::open(path.join(&frozen.relative_path)).unwrap();
                    assert_eq!(source.metadata().unwrap().len(), frozen.readable_bytes);
                    let document: FrozenEvidenceSet = decode(
                        source.take(frozen.readable_bytes),
                        MAXIMUM_PACK_DOCUMENT_BYTES,
                    )
                    .unwrap();
                    document.validate().unwrap();
                    assert!(document.contexts[0].facts[0].payload.len() > 2 << 20);
                    continue;
                }
                let reader = PackReader::open(&path, MAXIMUM_PACK_DOCUMENT_BYTES).unwrap();
                assert_eq!(reader.index().publication, PackPublication::Incomplete);
                assert!(reader.open_member("diagnostic").is_err());
                assert!(reader.read_frozen().is_ok());
            }
        }
    }
}

#[test]
fn stored_requirement_sources_resolve_actual_supplied_members() {
    use crate::tooling::review_analysis::{
        AnalysisSourceReference,
        requirements::{ExplanatoryRequirement, RequiredFact, RequirementStatus},
    };
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pack");
    let mut selection = PackSelection::default();
    let mut context = raw("context", true, PackMemberRole::Context);
    context.format = PackFormat::Json;
    selection.members.push(context);
    let mut session = PackSession::create(&path, selection, CancellationToken::new()).unwrap();
    session.add_frozen(&FrozenEvidenceSet::default()).unwrap();
    let mut document = analysis();
    document.requirements.push(ExplanatoryRequirement {
        question: document.request.question.unwrap(),
        subject: document.request.operation,
        fact: RequiredFact::OperationEvidence,
        status: RequirementStatus::Available,
        sources: vec![AnalysisSourceReference {
            member: "context".into(),
            pointer: "/absent".into(),
            event: None,
            value: pse_diagnostics::observation::Value::Count(1),
            partial: false,
        }],
        missing_reason: None,
    });
    session.add_analysis(&document).unwrap();
    session
        .copy_prefix("context", source(root.path(), b"{}"), 2)
        .unwrap();
    assert!(session.seal().unwrap_err().to_string().contains("absent"));
}
