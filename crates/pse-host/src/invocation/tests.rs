// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::envelope::{CommandOutcome, Completeness, Settlement};
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
fn response(directory: &InvocationDirectory) -> Envelope<String> {
    let mut result = Envelope::response("run", "child exited 3", "original detail".to_owned());
    result.invocation_id = directory.id().clone();
    result.outcome = CommandOutcome::Failure;
    result.subject_state = SubjectState::Failed;
    result.child_exit = Some(3);
    result.settlement = Settlement::Complete;
    result
}

#[test]
fn observation_of_absent_state_is_passive() {
    let (_temporary, store) = store();
    assert!(store.history::<String>().expect("empty history").is_empty());
    assert!(
        store
            .resolve("latest", Some("session"))
            .expect("empty selector")
            .is_none()
    );
    assert!(!store.state.root.exists());
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
            .result::<String, String>(directory.id())
            .expect("not completed")
            .is_none()
    );
    directory
        .publish_result(&response(&directory))
        .expect("publish");
    let result = store
        .result::<String, String>(directory.id())
        .expect("read")
        .expect("terminal");
    assert_eq!(result.child_exit, Some(3));
    assert_eq!(result.payload, "original detail");
    assert_eq!(result.settlement, Settlement::Complete);
    assert_eq!(result.exit_code(), 1);
}

#[test]
fn finality_and_identity_cannot_be_replaced_by_acknowledgements() {
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let mut result = response(&directory);
    result.subject_state = SubjectState::Running;
    assert!(directory.publish_result(&result).is_err());
    result.subject_state = SubjectState::Failed;
    result.invocation_id = CliInvocationId::new();
    assert!(directory.publish_result(&result).is_err());
    result.invocation_id = directory.id().clone();
    result.view.truncated = true;
    assert!(directory.publish_result(&result).is_err());
    assert!(!directory.path().join("result.json").exists());
    result.view.truncated = false;
    result.settlement = Settlement::Unknown;
    result.observation.completeness = Completeness::Partial;
    directory
        .publish_result(&result)
        .expect("unknown settlement is retained honestly");
    assert_eq!(
        store
            .result::<String, String>(directory.id())
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
            .result::<String, String>(directory.id())
            .expect("complete JSON")
            .is_some()
    );
    assert_eq!(fs::read_dir(directory.path()).expect("files").count(), 2);
}

#[test]
fn latest_prefers_session_and_prefixes_refuse_ambiguity() {
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
            .expect("checkout fallback"),
        Some(second.id().clone())
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
    assert!(store.history::<String>().is_err());
    assert!(store.resolve("latest", None).is_err());
    assert!(store.resolve("latest", Some("healthy-session")).is_err());
    assert!(store.resolve("latest", Some("absent-session")).is_err());
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
    assert!(store.history::<String>().is_err());
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
        store.history::<String>().expect("committed history").len(),
        1
    );
    let mut duplicate = spec(None);
    duplicate.invocation_id = id;
    assert!(store.begin(duplicate).is_err());
    assert_eq!(
        store.history::<String>().expect("unchanged history").len(),
        1
    );
}

#[test]
fn serialization_failure_never_publishes_a_partial_response() {
    struct BrokenPayload;
    impl Serialize for BrokenPayload {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("payload source failed"))
        }
    }
    let (_temporary, store) = store();
    let directory = store.begin(spec(None)).expect("begin");
    let result = response(&directory).with_payload(BrokenPayload);
    assert!(directory.publish_result(&result).is_err());
    assert!(
        store
            .result::<String, String>(directory.id())
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
