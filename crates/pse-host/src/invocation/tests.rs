// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::envelope::{Completeness, RedactionPolicy, Settlement};
use crate::managed::result::{ManagedSummary, TerminalResult};
use std::{collections::BTreeMap, os::unix::fs::symlink};

fn store() -> (tempfile::TempDir, InvocationStore) {
    let temporary = tempfile::tempdir().expect("temporary store");
    let environment = BTreeMap::from([(
        "XDG_STATE_HOME".into(),
        temporary.path().join("state").into_os_string(),
    )]);
    let state = CheckoutState::from_paths(
        &temporary.path().join("checkout"),
        &temporary.path().join("common.git"),
        &environment,
    )
    .expect("checkout binding");
    (temporary, InvocationStore::new(state))
}
fn spec(session: Option<&str>) -> InvocationSpec<String> {
    InvocationSpec {
        invocation_id: CliInvocationId::new(),
        command: "run".into(),
        session: session.map(str::to_owned),
        parent: None,
        operation: "captured operation".into(),
        control: PreparedGeneration {
            generation: "qualified-generation".into(),
            protocol: 1,
            manifest_digest: "a".repeat(64),
        },
        environment_digest: "b".repeat(64),
        scientific: Vec::new(),
    }
}
fn response(directory: &InvocationDirectory) -> TerminalResult<ManagedSummary> {
    TerminalResult::finalizer_unknown(
        directory.id(),
        "run",
        "signal".into(),
        vec![HostError::Ownership(
            "original settlement unavailable".into(),
        )],
        &RedactionPolicy::default(),
    )
}

#[test]
fn observation_of_absent_state_is_passive() {
    let (_temporary, store) = store();
    let history = store.history::<String>().expect("empty history");
    assert!(history.records.is_empty());
    assert!(history.failures.is_empty());
    assert!(
        store
            .resolve("latest", Some("session"))
            .expect("empty selector")
            .is_none()
    );
    assert!(!store.state.root.exists());
}

#[test]
fn explicit_begin_establishes_private_namespace_without_adopting_child_ledgers_or_links() {
    use std::os::unix::fs::PermissionsExt as _;
    let (_temporary, store) = store();
    let parent = store.state.root.parent().unwrap();
    fs::create_dir_all(parent).unwrap();
    fs::set_permissions(parent, fs::Permissions::from_mode(0o775)).unwrap();
    store.history::<String>().unwrap();
    assert_eq!(
        fs::metadata(parent).unwrap().permissions().mode() & 0o777,
        0o775
    );
    store.begin(spec(None)).unwrap();
    assert_eq!(
        fs::metadata(parent).unwrap().permissions().mode() & 0o777,
        0o700
    );
    fs::set_permissions(&store.state.root, fs::Permissions::from_mode(0o775)).unwrap();
    assert!(store.begin(spec(None)).is_err());
    let destination = parent.with_extension("other");
    fs::create_dir(&destination).unwrap();
    let link = parent.with_extension("link");
    symlink(&destination, &link).unwrap();
    assert!(initialize_namespace(&link).is_err());
}

#[test]
fn original_owner_records_requests_and_launch_cutoff_keep_separate_custody() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).unwrap();
    let reopened = store.directory(directory.id()).unwrap();
    assert!(
        reopened
            .evidence::<String>(InvocationEvidence::Custody)
            .unwrap()
            .is_none()
    );
    reopened
        .publish_evidence(InvocationEvidence::Launch, &"original intent")
        .unwrap();
    assert!(
        reopened
            .publish_evidence(InvocationEvidence::Launch, &"replacement")
            .is_err()
    );
    assert_eq!(
        reopened
            .evidence::<String>(InvocationEvidence::Launch)
            .unwrap()
            .as_deref(),
        Some("original intent")
    );
    reopened.publish_request(&"private request").unwrap();
    assert_eq!(
        reopened.take_request::<String>().unwrap(),
        "private request"
    );
    assert!(!reopened.path().join("request.json").exists());
    assert!(reopened.take_request::<String>().is_err());
    assert!(
        store
            .result::<String, ManagedSummary>(directory.id())
            .unwrap()
            .is_none()
    );
}

#[test]
fn unconsumed_private_input_cleanup_retains_decode_category_without_secret_values() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).unwrap();
    assert!(!directory.discard_request().unwrap());
    directory
        .publish_request(&"private-credential-value")
        .unwrap();
    let error = directory.take_request::<u64>().unwrap_err().to_string();
    assert!(error.contains("Data at line"));
    assert!(!error.contains("private-credential-value"));
    assert!(directory.discard_request().unwrap());
    assert!(!directory.discard_request().unwrap());
    let foreign = directory.path().join("other-private-input");
    fs::write(&foreign, b"preserve original destination").unwrap();
    symlink(&foreign, directory.path().join("request.json")).unwrap();
    assert!(directory.discard_request().is_err());
    assert!(foreign.exists());
}

#[test]
fn original_metadata_and_terminal_response_round_trip() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(Some("session"))).expect("begin");
    let metadata: InvocationMetadata<String> = store.metadata(directory.id()).expect("metadata");
    assert_eq!(metadata.selection.operation, "captured operation");
    assert_eq!(metadata.selection.session.as_deref(), Some("session"));
    assert_eq!(metadata.checkout_id, store.state.checkout_id.as_str());
    assert!(
        store
            .result::<String, ManagedSummary>(directory.id())
            .expect("not completed")
            .is_none()
    );
    directory
        .publish_result(&response(&directory))
        .expect("publish");
    let result = store
        .result::<String, ManagedSummary>(directory.id())
        .expect("read")
        .expect("terminal");
    assert_eq!(result.child_exit, None);
    assert_eq!(result.payload.service_result.as_deref(), Some("signal"));
    assert_eq!(result.settlement, Settlement::Unknown);
    assert_eq!(result.exit_code(), 1);
}

#[test]
fn finality_and_identity_cannot_be_replaced_by_acknowledgements() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let wrong_identity = TerminalResult::finalizer_unknown(
        &CliInvocationId::new(),
        "run",
        "signal".into(),
        Vec::new(),
        &RedactionPolicy::default(),
    );
    assert!(directory.publish_result(&wrong_identity).is_err());
    let wrong_command = TerminalResult::finalizer_unknown(
        directory.id(),
        "test",
        "signal".into(),
        Vec::new(),
        &RedactionPolicy::default(),
    );
    assert!(directory.publish_result(&wrong_command).is_err());
    assert!(!directory.path().join("result.json").exists());
    let result = response(&directory);
    assert_eq!(
        result.response().observation.completeness,
        Completeness::Partial
    );
    directory
        .publish_result(&result)
        .expect("unknown settlement is retained honestly");
    assert_eq!(
        store
            .result::<String, ManagedSummary>(directory.id())
            .expect("read")
            .expect("record")
            .settlement,
        Settlement::Unknown
    );
}

#[test]
fn competing_terminal_writers_publish_exactly_one_complete_record() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let published = std::thread::scope(|scope| {
        let first = scope.spawn(|| directory.publish_result(&response(&directory)));
        let second = scope.spawn(|| directory.publish_result(&response(&directory)));
        [
            first.join().expect("writer"),
            second.join().expect("writer"),
        ]
    });
    assert_eq!(published.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(published.iter().filter(|result| result.is_err()).count(), 1);
    assert!(
        store
            .result::<String, ManagedSummary>(directory.id())
            .expect("complete JSON")
            .is_some()
    );
    assert_eq!(fs::read_dir(directory.path()).expect("files").count(), 2);
}

#[test]
fn latest_is_session_exact_with_checkout_fallback_only_when_identity_is_absent() {
    let (_temporary, store) = store();
    let mut first = spec(Some("a"));
    first.invocation_id =
        CliInvocationId::try_from("01800000-0000-7000-8000-000000000001".to_owned()).expect("id");
    let first = store.begin(first).expect("first");
    let mut second = spec(Some("b"));
    second.invocation_id =
        CliInvocationId::try_from("01800000-0000-7000-8000-000000000002".to_owned()).expect("id");
    let second = store.begin(second).expect("second");
    assert_eq!(
        store.resolve("latest", Some("a")).expect("session latest"),
        Some(first.id().clone())
    );
    assert_eq!(
        store
            .resolve("latest", Some("absent"))
            .expect("absent selected session"),
        None
    );
    assert_eq!(
        store.resolve("latest", None).unwrap(),
        Some(second.id().clone())
    );
    assert_eq!(
        store.resolve(second.id().as_str(), Some("a")).unwrap(),
        Some(second.id().clone()),
        "session correlation grants no exact-ID control authority"
    );
    assert_eq!(
        store.resolve(second.id().as_str(), None).expect("exact"),
        Some(second.id().clone())
    );
    assert!(store.resolve("01800000", None).is_err());
    assert!(store.resolve("../escape", None).is_err());
    assert!(store.resolve("", None).is_err());
}

#[test]
fn exact_and_unique_prefix_ignore_unrelated_corrupt_history() {
    let (_temporary, store) = store();
    let mut healthy = spec(Some("healthy-session"));
    healthy.invocation_id =
        CliInvocationId::try_from("01800000-0000-7000-8000-000000000001".to_owned()).expect("id");
    let healthy = store.begin(healthy).expect("healthy invocation");
    let mut corrupt = spec(Some("other-session"));
    corrupt.invocation_id =
        CliInvocationId::try_from("02800000-0000-7000-8000-000000000001".to_owned()).expect("id");
    let corrupt = store.begin(corrupt).expect("other invocation");
    fs::write(corrupt.path().join("meta.json"), b"invalid JSON").expect("corrupt fixture");

    assert_eq!(
        store
            .resolve(healthy.id().as_str(), None)
            .expect("exact ID"),
        Some(healthy.id().clone())
    );
    assert_eq!(
        store.resolve("01800000", None).expect("unique prefix"),
        Some(healthy.id().clone())
    );
    let history = store.history::<String>().expect("partial history");
    assert_eq!(history.records.len(), 1);
    assert_eq!(history.records[0].selection.invocation_id, *healthy.id());
    assert_eq!(history.failures.len(), 1);
    assert_eq!(
        history.failures[0].invocation_id.as_ref(),
        Some(corrupt.id())
    );
    assert_eq!(history.failures[0].path, corrupt.path());
    let latest_error = store.resolve("latest", None).unwrap_err();
    let latest_record =
        crate::envelope::ErrorRecord::from_typed(&latest_error, &RedactionPolicy::default());
    assert_eq!(latest_record.children.len(), 1);
    assert!(
        latest_record.children[0]
            .facts
            .observations
            .contains_key("invocation_id")
    );
    assert!(store.resolve("latest", Some("healthy-session")).is_err());
    assert!(store.resolve("latest", Some("absent-session")).is_err());
}

#[test]
fn history_keeps_all_healthy_rows_and_structured_independent_source_failures() {
    let (_temporary, store) = store();
    let first = store.begin(spec(None)).unwrap();
    let second = store.begin(spec(None)).unwrap();
    let corrupt = store.begin(spec(None)).unwrap();
    fs::write(corrupt.path().join("meta.json"), b"{\"version\":").unwrap();
    let invalid_name = store.state.invocations().join("invalid-identity");
    fs::create_dir(&invalid_name).unwrap();
    let history = store.history::<serde::de::IgnoredAny>().unwrap();
    assert_eq!(history.records.len(), 2);
    let ids = history
        .records
        .iter()
        .map(|record| &record.selection.invocation_id)
        .collect::<Vec<_>>();
    assert!(ids.contains(&first.id()));
    assert!(ids.contains(&second.id()));
    assert_eq!(history.failures.len(), 2);
    let error = history
        .failures
        .iter()
        .find(|failure| failure.invocation_id.as_ref() == Some(corrupt.id()))
        .unwrap();
    let record = crate::envelope::ErrorRecord::from_typed(error, &RedactionPolicy::default());
    assert_eq!(record.code.as_deref(), Some("host.ownership"));
    assert!(
        record
            .facts
            .observations
            .contains_key("evidence_path_platform_bytes")
    );
    assert!(record.facts.observations.contains_key("invocation_id"));
    assert!(!record.children.is_empty());
    assert!(
        history
            .failures
            .iter()
            .any(|failure| failure.path == invalid_name && failure.invocation_id.is_none())
    );
    let latest_error = store.resolve("latest", None).unwrap_err();
    let latest_record =
        crate::envelope::ErrorRecord::from_typed(&latest_error, &RedactionPolicy::default());
    assert_eq!(latest_record.children.len(), 2);
}

#[test]
fn prefix_ambiguity_is_established_before_candidate_metadata_is_decoded() {
    let (_temporary, store) = store();
    for suffix in ["1", "2"] {
        let mut selection = spec(None);
        selection.invocation_id =
            CliInvocationId::try_from(format!("01800000-0000-7000-8000-00000000000{suffix}"))
                .expect("id");
        let directory = store.begin(selection).expect("candidate");
        if suffix == "2" {
            fs::write(directory.path().join("meta.json"), b"invalid JSON")
                .expect("corrupt candidate");
        }
    }
    let error = store
        .resolve("01800000", None)
        .expect_err("ambiguous prefix");
    assert!(error.to_string().contains("invocation prefix is ambiguous"));
}

#[test]
fn missing_id_is_an_empty_passive_observation_even_with_corrupt_history() {
    let (_temporary, store) = store();
    let missing = "01800000-0000-7000-8000-000000000001";
    assert!(
        store
            .resolve(missing, None)
            .expect("absent store")
            .is_none()
    );
    assert!(!store.state.root.exists());

    let mut selection = spec(None);
    selection.invocation_id =
        CliInvocationId::try_from("02800000-0000-7000-8000-000000000001".to_owned()).expect("id");
    let directory = store.begin(selection).expect("unrelated invocation");
    fs::write(directory.path().join("meta.json"), b"invalid JSON").expect("corrupt fixture");
    assert!(store.resolve(missing, None).expect("missing ID").is_none());
    assert!(
        store
            .resolve("01800000", None)
            .expect("missing prefix")
            .is_none()
    );
    assert!(!store.state.invocations().join(missing).exists());
}

#[test]
fn malformed_or_missing_chosen_metadata_remains_an_error() {
    let (_temporary, store) = store();
    let mut selection = spec(None);
    selection.invocation_id =
        CliInvocationId::try_from("01800000-0000-7000-8000-000000000001".to_owned()).expect("id");
    let directory = store.begin(selection).expect("chosen invocation");
    let metadata = directory.path().join("meta.json");
    fs::write(&metadata, b"invalid JSON").expect("corrupt fixture");
    for selector in [directory.id().as_str(), "01800000"] {
        let error = store
            .resolve(selector, None)
            .expect_err("malformed selection");
        assert!(error.to_string().contains("invalid selection metadata"));
    }
    fs::remove_file(metadata).expect("missing metadata fixture");
    for selector in [directory.id().as_str(), "01800000"] {
        assert!(store.resolve(selector, None).is_err());
    }
}

#[test]
fn wrong_checkout_or_version_is_reported_instead_of_empty_history() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let path = directory.path().join("meta.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("bytes")).expect("JSON");
    value["checkout_id"] = "another-checkout".into();
    fs::write(&path, serde_json::to_vec(&value).expect("encode")).expect("tamper fixture");
    let history = store.history::<String>().expect("partial history");
    assert!(history.records.is_empty());
    assert_eq!(history.failures.len(), 1);
    assert!(store.resolve(directory.id().as_str(), None).is_err());
    value["checkout_id"] = store.state.checkout_id.as_str().into();
    value["version"] = 9000.into();
    fs::write(&path, serde_json::to_vec(&value).expect("encode")).expect("tamper fixture");
    assert!(store.resolve("latest", None).is_err());
}

#[test]
fn symlinked_evidence_and_invocation_directories_are_refused() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let metadata = directory.path().join("meta.json");
    let copied = directory.path().join("copied.json");
    fs::rename(&metadata, &copied).expect("move fixture");
    symlink("copied.json", &metadata).expect("symlink fixture");
    assert!(store.metadata::<String>(directory.id()).is_err());
    assert!(store.resolve(directory.id().as_str(), None).is_err());
    fs::remove_file(&metadata).expect("remove fixture symlink");
    fs::rename(&copied, &metadata).expect("restore fixture");
    let original = directory.path().with_extension("other");
    fs::rename(directory.path(), &original).expect("move directory");
    symlink(&original, directory.path()).expect("directory symlink fixture");
    assert!(store.metadata::<String>(directory.id()).is_err());
    assert!(store.resolve(directory.id().as_str(), None).is_err());
}

#[test]
fn opaque_operation_resolution_does_not_normalize_numbers_or_duplicate_keys() {
    let (_temporary, store) = store();
    let selection = spec(None);
    let raw = serde_json::value::RawValue::from_string(
        "{\"x\":12345678901234567890123456789012345678901234567890,\"x\":7}".into(),
    )
    .expect("raw JSON");
    let selection = InvocationSpec {
        invocation_id: selection.invocation_id,
        command: selection.command,
        session: selection.session,
        parent: selection.parent,
        operation: raw,
        control: selection.control,
        environment_digest: selection.environment_digest,
        scientific: selection.scientific,
    };
    let directory = store.begin(selection).expect("begin");
    assert_eq!(
        store
            .resolve("latest", None)
            .expect("resolve raw selection"),
        Some(directory.id().clone())
    );
    let metadata: InvocationMetadata<Box<serde_json::value::RawValue>> =
        store.metadata(directory.id()).expect("raw metadata");
    assert!(
        metadata
            .selection
            .operation
            .get()
            .contains("12345678901234567890123456789012345678901234567890")
    );
    assert_eq!(
        metadata.selection.operation.get().matches("\"x\"").count(),
        2
    );
}

#[test]
fn private_unpublished_creation_is_not_an_invocation_and_duplicate_ids_refuse() {
    let (_temporary, store) = store();
    let selection = spec(None);
    let id = selection.invocation_id.clone();
    store.begin(selection).expect("begin");
    fs::create_dir(store.state.invocations().join(".prepare-unpublished"))
        .expect("private preparation fixture");
    assert_eq!(
        store
            .history::<String>()
            .expect("committed history")
            .records
            .len(),
        1
    );
    let mut duplicate = spec(None);
    duplicate.invocation_id = id;
    assert!(store.begin(duplicate).is_err());
    assert_eq!(
        store
            .history::<String>()
            .expect("unchanged history")
            .records
            .len(),
        1
    );
}

#[test]
fn serialization_failure_never_publishes_partial_owner_evidence() {
    struct BrokenPayload;
    impl Serialize for BrokenPayload {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("payload source failed"))
        }
    }
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    assert!(
        directory
            .publish_evidence(InvocationEvidence::Settlement, &BrokenPayload)
            .is_err()
    );
    assert!(!directory.path().join("settlement.json").exists());
    assert!(
        store
            .result::<String, ManagedSummary>(directory.id())
            .expect("no terminal evidence")
            .is_none()
    );
    assert_eq!(
        fs::read_dir(directory.path())
            .expect("temporary cleaned")
            .count(),
        1
    );
    directory
        .publish_result(&response(&directory))
        .expect("original owner can publish complete result");
}
