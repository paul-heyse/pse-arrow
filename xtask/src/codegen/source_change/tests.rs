// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_codegen::codegen::provenance::protocol::{MAP_PATH, ScopeKind};

fn write(root: &Path, path: &str, bytes: &[u8]) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn fixture(root: &Path, fail_generation: bool) {
    use std::os::unix::fs::PermissionsExt as _;
    let git = |args: &[&str]| {
        let result = snapshot::git(root).args(args).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    git(&["init", "--quiet"]);
    git(&["config", "user.name", "Source Change Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    write(
        root,
        "rust-toolchain.toml",
        include_bytes!("../../../../rust-toolchain.toml"),
    );
    write(root, "Cargo.toml", b"[package]\nname = 'source-change-fixture'\nversion = '0.0.0'\nedition = '2024'\n[workspace]\n");
    write(root, "src/lib.rs", b"//! Fixture workspace.\n");
    write(root, ".gitignore", b"/target/\n");
    write(
        root,
        Case::OptionalProjectDescription.declaration(),
        b"\"canonical_projects\"\n\"canonical_project_branches\"\n",
    );
    write(
        root,
        Case::OptionalProjectDescription.consumer(),
        b"historical project consumer\n",
    );
    write(root, "fixture-generated/original.rs", b"original rust\n");
    write(
        root,
        "fixture-generated/original.surql",
        b"original surreal\n",
    );
    write(root, MAP_PATH, &serde_json::to_vec(&serde_json::json!({
        "schema_version": 1, "$comment": "fixture", "coverage_limits": [],
        "owners": [{"producer": "fixture", "outputs": [{"path": MAP_PATH, "kind": "file"},
            {"path": "fixture-generated", "kind": "tree"}], "regenerate": [], "inputs": [], "contributors": null}]
    })).unwrap());
    write(root, "justfile", b"codegen-all-check:\n    test -f fixture-generated/new.rs\n    test -f fixture-generated/original.surql\n");
    write(root, "scripts/bootstrap-host", b"#!/bin/sh\nexit 0\n");
    write(
        root,
        "scripts/pse-env",
        b"#!/bin/sh\nwhile [ \"$1\" != -- ]; do shift; done\nshift\nexec \"$@\"\n",
    );
    write(root, "scripts/pse", br#"#!/bin/sh
case "$1" in
    inspect) printf '%s\n' '{"observation":{"completeness":"complete"},"payload":{"selected_packages":[{"name":"pse-operations"}]}}' ;;
    test) test -f fixture-generated/new.rs ;;
    *) exit 2 ;;
esac
"#);
    write(
        root,
        "scripts/source-change-codegen",
        format!(
            r#"#!/bin/sh
set -eu
case "$1" in
    --stages) printf '%s\n' rust-contracts recorded-result surreal full-codegen ;;
    native-identity) echo 'original native identity' ;;
    rust-contracts)
        printf 'produced rust\n' > fixture-generated/new.rs
        if [ '{fail_generation}' = true ] && [ ! -f target/failed ]; then
            mkdir -p target
            touch target/failed
            exit 13
        fi ;;
    recorded-result) rm -f fixture-generated/original.rs ;;
    surreal) printf 'produced surreal\n' > fixture-generated/original.surql ;;
    full-codegen) test -f fixture-generated/new.rs ;;
    *) exit 2 ;;
esac
"#
        )
        .as_bytes(),
    );
    for path in ["scripts/bootstrap-host", "scripts/pse-env", "scripts/pse"] {
        fs::set_permissions(root.join(path), fs::Permissions::from_mode(0o755)).unwrap();
    }
    git(&["add", "."]);
    git(&[
        "-c",
        "commit.gpgsign=false",
        "commit",
        "--quiet",
        "-m",
        "fixture",
    ]);
    write(
        root,
        Case::OptionalProjectDescription.declaration(),
        "\"canonical_projects\"\n/* μ */ \"description\"\n\"canonical_project_branches\"\n"
            .as_bytes(),
    );
}

fn only_operation(state: &Path) -> (String, Operation) {
    let id = fs::read_dir(state)
        .unwrap()
        .map(|entry| entry.unwrap())
        .find(|entry| entry.file_type().unwrap().is_dir())
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();
    let operation = Operation::open(state, &id).unwrap();
    (id, operation)
}

#[test]
fn unicode_declaration_span_keeps_independently_known_byte_coordinates() {
    let case = Case::RecordedResultFieldRename;
    let content = "// 水\npub struct PackResultSelection {\n  /* μ */ pub selected_field: Option<String>,\n}\npub struct PackResultPredicate {}";
    let span = locate(case, Path::new(case.declaration()), content.into()).unwrap();
    assert_eq!(span.byte_offset, 51);
    assert_eq!(span.byte_length, 18);
    assert_eq!(span.line, 2);
    assert_eq!(span.column, 11);
    assert_eq!(
        &content.as_bytes()[span.byte_offset..][..span.byte_length],
        b"pub selected_field"
    );
}

#[test]
fn diagnostic_capture_drains_without_retaining_unbounded_child_output() {
    let mut output = vec![b'a'; 2 * MAX_LOG_STREAM];
    output.extend(b"original failure");
    let retained = drain(output.as_slice(), MAX_LOG_STREAM).unwrap();
    assert_eq!(retained.total, output.len() as u64);
    assert_eq!(retained.bytes, output[output.len() - MAX_LOG_STREAM..]);
}

#[test]
fn generation_failure_resumes_same_snapshot_and_publishes_cross_generated_delta() {
    let checkout = tempfile::tempdir().unwrap();
    let owner = tempfile::tempdir().unwrap();
    let state = owner.path().join("source-change");
    let root = checkout.path();
    fixture(root, true);
    let case = Case::OptionalProjectDescription;
    assert_eq!(preview(root, &state, case, Vec::new(), None, None).unwrap(), 1);
    assert!(!root.join("fixture-generated/new.rs").exists());
    let (id, operation) = only_operation(&state);
    let mut record = operation.read(root, &id).unwrap();
    assert_eq!(record.status.state, State::GenerationFailed);
    let failure = record.status.failure.as_ref().unwrap();
    assert_eq!(failure.stage, "rust-contracts");
    assert_eq!(failure.original_exit, Some(13));
    assert_eq!(failure.original_log.as_deref(), Some(FAILURE_LOG));
    let log = fs::read(operation.directory.join(FAILURE_LOG)).unwrap();
    assert_eq!(
        failure.original_log_sha256.as_deref(),
        Some(hex(&Sha256::digest(log)).as_str())
    );
    assert!(operation.snapshot().join("target/failed").is_file());
    assert!(
        preview(root, &state, case, Vec::new(), None, None)
            .unwrap_err()
            .to_string()
            .contains(&id)
    );
    assert_eq!(generate(&operation, &mut record).unwrap(), 0);
    assert_eq!(record.status.state, State::Ready);
    assert!(!operation.snapshot().exists());
    assert_eq!(snapshot_claim(&state).unwrap(), None);
    assert!(!root.join("fixture-generated/new.rs").exists());
    assert_eq!(record.status.generated.len(), 3);
    assert_eq!(apply(&operation, &mut record, false).unwrap(), 0);
    assert_eq!(record.status.state, State::Applied);
    assert!(!operation.custody().exists());
    assert_eq!(
        fs::read(root.join("fixture-generated/new.rs")).unwrap(),
        b"produced rust\n"
    );
    assert_eq!(
        fs::read(root.join("fixture-generated/original.surql")).unwrap(),
        b"produced surreal\n"
    );
    assert!(!root.join("fixture-generated/original.rs").exists());
    assert!(
        fs::read_to_string(root.join(case.declaration()))
            .unwrap()
            .contains('μ')
    );
}

#[test]
fn authored_edit_after_preview_refuses_with_zero_output_writes() {
    let checkout = tempfile::tempdir().unwrap();
    let owner = tempfile::tempdir().unwrap();
    let state = owner.path().join("source-change");
    let root = checkout.path();
    fixture(root, false);
    let case = Case::OptionalProjectDescription;
    assert_eq!(preview(root, &state, case, Vec::new(), None, None).unwrap(), 0);
    let (id, operation) = only_operation(&state);
    let mut record = operation.read(root, &id).unwrap();
    write(root, case.consumer(), b"intervening authored edit\n");
    assert_eq!(apply(&operation, &mut record, false).unwrap(), 1);
    assert_eq!(record.status.state, State::Ready);
    assert_eq!(
        record.status.failure.unwrap().code,
        "source_change.stale_preview"
    );
    assert!(!root.join("fixture-generated/new.rs").exists());
    assert_eq!(
        fs::read(root.join("fixture-generated/original.rs")).unwrap(),
        b"original rust\n"
    );
    assert_eq!(
        fs::read(root.join("fixture-generated/original.surql")).unwrap(),
        b"original surreal\n"
    );
    assert_eq!(
        fs::read(root.join(case.consumer())).unwrap(),
        b"intervening authored edit\n"
    );
}

#[test]
fn reviewed_postimages_are_required_before_generation_and_bound_into_the_receipt() {
    let checkout = tempfile::tempdir().unwrap();
    let owner = tempfile::tempdir().unwrap();
    let checkout_id = "a".repeat(64);
    let state = owner.path().join(&checkout_id).join("source-change");
    let root = checkout.path();
    fixture(root, false);
    let case = Case::OptionalProjectDescription;
    let relative = Path::new("src/lib.rs");
    let expected_bytes = b"//! Reviewed change.\n";
    let expected_sha256 = hex(&Sha256::digest(expected_bytes));
    let projection = || ProposalHandoff {
        version: handoff::VERSION, checkout_id: checkout_id.clone(), checkout_revision: snapshot::revision(root).unwrap(),
        files: vec![ProposalFile { path: relative.into(), before_sha256: snapshot::head_file_identity(root, relative).unwrap().unwrap(), after_sha256: expected_sha256.clone() }],
    };
    let stale = preview(root, &state, case, Vec::new(), None, Some(projection())).unwrap_err();
    assert!(stale.to_string().contains("reviewed proposal postimage"));
    assert!(fs::read_dir(&state).unwrap().next().is_none());
    assert!(!root.join("fixture-generated/new.rs").exists());

    fs::write(root.join(relative), expected_bytes).unwrap();
    let mut foreign = projection();
    foreign.checkout_revision = "b".repeat(40);
    assert!(preview(root, &state, case, Vec::new(), None, Some(foreign)).unwrap_err().to_string().contains("checkout or revision"));
    assert!(fs::read_dir(&state).unwrap().next().is_none());
    let projected = serde_json::to_vec(&projection()).unwrap();
    let proposal = read_proposal(projected.as_slice()).unwrap();
    assert_eq!(preview(root, &state, case, Vec::new(), None, Some(proposal)).unwrap(), 0);
    let (id, operation) = only_operation(&state);
    let record = operation.read(root, &id).unwrap();
    assert_eq!(record.status.proposal.unwrap().files[0].after_sha256, expected_sha256);
    assert!(record.status.authored.iter().any(|entry| entry.path == relative && entry.working_sha256 == expected_sha256));
    assert!(!root.join("fixture-generated/new.rs").exists());
}

#[test]
fn publication_recovery_replays_preimages_and_refuses_a_third_state() {
    let root = tempfile::tempdir().unwrap();
    let candidate = tempfile::tempdir().unwrap();
    let custody = tempfile::tempdir().unwrap();
    write(root.path(), "output/first", b"old first");
    write(root.path(), "output/last", b"old last");
    write(candidate.path(), "output/first", b"new first");
    write(candidate.path(), "output/last", b"new last");
    let delta = delta::Delta::stage(
        root.path(),
        candidate.path(),
        custody.path(),
        vec![Scope {
            path: "output".into(),
            kind: ScopeKind::Tree,
        }],
    )
    .unwrap();
    write(root.path(), "output/first", b"new first");
    write(root.path(), "output/last", b"conflicting last");
    assert!(delta.preflight(root.path(), custody.path(), true).is_err());
    assert_eq!(
        fs::read(root.path().join("output/last")).unwrap(),
        b"conflicting last"
    );
    write(root.path(), "output/last", b"old last");
    assert!(delta.preflight(root.path(), custody.path(), false).is_err());
    delta.preflight(root.path(), custody.path(), true).unwrap();
    delta.publish(root.path(), custody.path()).unwrap();
    assert!(delta.all_postimages(root.path()).unwrap());
}
