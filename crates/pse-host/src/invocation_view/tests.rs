// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{invocation::InvocationSpec, paths::CheckoutState};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

fn fixture() -> (tempfile::TempDir, InvocationStore) {
    let temporary = tempfile::tempdir().unwrap();
    let environment = BTreeMap::from([(
        "XDG_STATE_HOME".into(),
        temporary.path().join("state").into_os_string(),
    )]);
    let state = CheckoutState::from_paths(
        &temporary.path().join("checkout"),
        &temporary.path().join("common.git"),
        &environment,
    )
    .unwrap();
    (temporary, InvocationStore::new(state))
}
fn spec(session: &str) -> InvocationSpec<String> {
    InvocationSpec {
        invocation_id: CliInvocationId::new(),
        command: "run".into(),
        session: Some(session.into()),
        parent: None,
        operation: "original operation".into(),
        control: PreparedGeneration {
            generation: "generation.fixture".into(),
            protocol: 1,
            manifest_digest: "a".repeat(64),
        },
        environment_digest: "b".repeat(64),
        scientific: Vec::new(),
    }
}
#[test]
fn absent_queries_do_not_create_state_and_pending_results_do_not_claim_running() {
    let (_temporary, store) = fixture();
    let empty = history(&store, None, 5, &RedactionPolicy::default()).unwrap();
    assert!(empty.payload.records.is_empty());
    assert_eq!(
        result(&store, "latest", None).unwrap().subject_state,
        SubjectState::NoMatch
    );
    let directory = store.begin(spec("a")).unwrap();
    let pending = result(&store, directory.id().as_str(), None).unwrap();
    assert_eq!(pending.subject_state, SubjectState::Unknown);
    assert!(pending.payload.response.is_none());
    assert!(!directory.path().join("result.json").exists());
}
#[test]
fn history_retains_partial_failures_and_never_decodes_results() {
    let (_temporary, store) = fixture();
    let first = store.begin(spec("first")).unwrap();
    let broken = store.begin(spec("broken")).unwrap();
    fs::write(
        first.path().join("result.json"),
        b"not JSON; must not be read by metadata history",
    )
    .unwrap();
    fs::write(broken.path().join("meta.json"), b"not JSON").unwrap();
    let response = history(&store, Some("first"), 1, &RedactionPolicy::default()).unwrap();
    assert_eq!(response.payload.records.len(), 1);
    assert_eq!(response.payload.records[0].invocation_id, *first.id());
    assert_eq!(response.observation.completeness, Completeness::Partial);
    assert_eq!(response.failures.len(), 1);
    assert_eq!(store.directory(first.id()).unwrap().id(), first.id());
}
#[test]
fn result_retains_original_identity_failures_and_raw_number_fidelity() {
    let (_temporary, store) = fixture();
    let directory = store.begin(spec("a")).unwrap();
    let original = serde_json::value::RawValue::from_string(
        "{\"large\":18446744073709551616,\"precise\":1.234567890123456789}".into(),
    )
    .unwrap();
    let mut terminal = Envelope::response("run", "original outcome", original);
    terminal.invocation_id = directory.id().clone();
    terminal.subject_state = SubjectState::Failed;
    terminal.child_exit = Some(3);
    directory.publish_result(&terminal).unwrap();
    let observed = result(&store, directory.id().as_str(), None).unwrap();
    assert_ne!(observed.invocation_id, *directory.id());
    let original = observed.payload.response.as_ref().unwrap();
    assert_eq!(original.invocation_id, *directory.id());
    assert_eq!(original.child_exit, Some(3));
    assert!(original.payload.get().contains("1.234567890123456789"));
    assert!(
        serde_json::to_string(&observed)
            .unwrap()
            .contains("18446744073709551616")
    );
}
#[test]
fn raw_stream_windows_preserve_non_utf8_and_tail_offsets_without_cross_pipe_order() {
    let (_temporary, store) = fixture();
    let directory = store.begin(spec("a")).unwrap();
    let path = directory.path().join("stdout.raw");
    fs::write(&path, b"a\xffbc\ndef").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let part = log(
        &store,
        directory.id().as_str(),
        None,
        OutputStream::Stdout,
        1,
        NonZeroU64::new(3).unwrap(),
        false,
    )
    .unwrap();
    assert_eq!(part.payload.byte_range, 1..4);
    assert_eq!(part.payload.next_offset, Some(4));
    assert!(
        matches!(part.payload.content, Content::Base64(ref value) if STANDARD.decode(value).unwrap() == b"\xffbc")
    );
    let tail = log(
        &store,
        directory.id().as_str(),
        None,
        OutputStream::Stdout,
        0,
        NonZeroU64::new(3).unwrap(),
        true,
    )
    .unwrap();
    assert_eq!(tail.payload.byte_range, 5..8);
    assert!(matches!(tail.payload.content, Content::Utf8(ref value) if value == "def"));
    assert!(!tail.payload.atomic_snapshot);
    assert!(
        log(
            &store,
            directory.id().as_str(),
            None,
            OutputStream::Stdout,
            9,
            NonZeroU64::MIN,
            false
        )
        .is_err()
    );
}
#[test]
fn raw_stream_symlinks_and_writable_foreign_evidence_refuse() {
    let (_temporary, store) = fixture();
    let directory = store.begin(spec("a")).unwrap();
    let destination = directory.path().join("other");
    fs::write(&destination, b"wrong source").unwrap();
    let path = directory.path().join("stderr.raw");
    symlink(&destination, &path).unwrap();
    assert!(directory.stream(OutputStream::Stderr).is_err());
    fs::remove_file(&path).unwrap();
    fs::write(&path, b"bad mode").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
    assert!(directory.stream(OutputStream::Stderr).is_err());
}
