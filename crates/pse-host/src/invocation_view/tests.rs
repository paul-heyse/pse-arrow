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
    // Simulate an existing external owner's original record. Read compatibility does not
    // grant generic envelopes the new typed publication capability.
    let path = directory.path().join("result.json");
    fs::write(&path, serde_json::to_vec(&terminal).unwrap()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
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
    let compact = inspect(
        &store,
        "latest",
        Some("a"),
        Some(ResultDetail::Provenance),
        &RedactionPolicy::default(),
    )
    .unwrap();
    assert_eq!(
        compact.payload.original_invocation.as_ref(),
        Some(directory.id())
    );
    let compact = serde_json::to_value(compact).unwrap();
    assert_eq!(compact["payload"]["response"]["child_exit"], 3);
    assert_eq!(compact["payload"]["detail"]["kind"], "provenance");
    assert_eq!(
        compact["payload"]["detail"]["environment_digest"],
        "b".repeat(64)
    );
    assert!(!compact.to_string().contains("18446744073709551616"));
}

#[test]
fn unavailable_detail_and_observed_empty_detail_are_distinct() {
    let (_temporary, store) = fixture();
    let directory = store.begin(spec("a")).unwrap();
    let pending = inspect(
        &store,
        directory.id().as_str(),
        None,
        Some(ResultDetail::Failures),
        &RedactionPolicy::default(),
    )
    .unwrap();
    assert!(matches!(
        pending.payload.detail,
        Some(InspectionDetail::Unavailable {
            detail: ResultDetail::Failures
        })
    ));
    let mut terminal = Envelope::response("run", "original outcome", ());
    terminal.invocation_id = directory.id().clone();
    terminal.subject_state = SubjectState::Succeeded;
    let result = directory.path().join("result.json");
    fs::write(&result, serde_json::to_vec(&terminal).unwrap()).unwrap();
    fs::set_permissions(result, fs::Permissions::from_mode(0o600)).unwrap();
    let settled = inspect(
        &store,
        directory.id().as_str(),
        None,
        Some(ResultDetail::Failures),
        &RedactionPolicy::default(),
    )
    .unwrap();
    let Some(InspectionDetail::Failures { failures }) = settled.payload.detail else {
        panic!("expected observed failures");
    };
    assert!(failures.records.is_empty());
    assert_eq!(failures.omitted, 0);
}

#[test]
fn result_export_publishes_only_the_bounded_original_view_to_a_fresh_path() {
    let (temporary, store) = fixture();
    let original = store.begin(spec("a")).unwrap();
    let result_path = original.path().join("result.json");
    let mut terminal = Envelope::response("run", "original private-token", "opaque owner data");
    terminal.invocation_id = original.id().clone();
    terminal.outcome = CommandOutcome::Failure;
    terminal.subject_state = SubjectState::Failed;
    terminal.child_exit = Some(3);
    terminal.settlement = Settlement::Complete;
    let original_bytes = serde_json::to_vec(&terminal).unwrap();
    fs::write(&result_path, &original_bytes).unwrap();
    fs::set_permissions(&result_path, fs::Permissions::from_mode(0o600)).unwrap();
    let foreign = store.begin(spec("b")).unwrap();
    let receipt_id = CliInvocationId::new();
    let policy = RedactionPolicy::new([("TOKEN".into(), "private-token".into())]);
    let perform = |selector: &str, destination: &Path| {
        export(
            &store,
            selector,
            Some("a"),
            Some(ResultDetail::Provenance),
            destination,
            &policy,
            &receipt_id,
        )
    };
    let output = temporary.path().join("inspection.json");
    let receipt = perform("latest", &output).unwrap();
    assert_eq!(receipt.subject_state, SubjectState::Available);
    assert_eq!(receipt.invocation_id, receipt_id);
    assert_eq!(
        receipt.payload.original_invocation.as_ref(),
        Some(original.id())
    );
    let published = receipt.payload.published.unwrap();
    let bytes = fs::read(&output).unwrap();
    assert_eq!(published.bytes, bytes.len());
    assert_eq!(published.sha256, crate::managed::workflow::digest(&bytes));
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document["invocation_id"], receipt_id.as_str());
    assert_eq!(document["payload"]["response"]["outcome"], "failure");
    assert_eq!(document["payload"]["response"]["child_exit"], 3);
    assert_eq!(
        document["payload"]["response"]["summary"],
        "original <redacted:TOKEN>"
    );
    assert_eq!(
        document["payload"]["detail"]["environment_digest"],
        "b".repeat(64)
    );
    assert!(
        !String::from_utf8(bytes.clone())
            .unwrap()
            .contains("opaque owner data")
    );
    assert!(perform("latest", &output).is_err());
    assert_eq!(fs::read(&output).unwrap(), bytes);
    assert_eq!(fs::read(&result_path).unwrap(), original_bytes);

    let internal = store.state_root().join("inspection.json");
    assert!(perform("latest", &internal).is_err());
    assert!(!internal.exists());
    let missing = temporary.path().join("unavailable.json");
    let pending = perform(foreign.id().as_str(), &missing).unwrap();
    assert_eq!(pending.subject_state, SubjectState::Unknown);
    assert!(pending.payload.published.is_none());
    assert!(!missing.exists());
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
        LogWindow {
            phase_file: None,
            offset: 1,
            length: NonZeroU64::new(3).unwrap(),
            tail: false,
        },
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
        LogWindow {
            phase_file: None,
            offset: 0,
            length: NonZeroU64::new(3).unwrap(),
            tail: true,
        },
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
            LogWindow {
                phase_file: None,
                offset: 0,
                length: NonZeroU64::new(256 * 1024 + 1).unwrap(),
                tail: false
            },
        )
        .is_err()
    );
    assert!(
        log(
            &store,
            directory.id().as_str(),
            None,
            OutputStream::Stdout,
            LogWindow {
                phase_file: None,
                offset: 9,
                length: NonZeroU64::MIN,
                tail: false
            },
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
