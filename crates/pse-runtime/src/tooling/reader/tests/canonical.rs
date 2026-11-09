// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use std::num::NonZeroUsize;

#[test]
fn project_reader_refuses_invalid_target_before_preparation() {
    use super::super::{ProjectRead, ProjectReadSelection, ReaderError, Request, encode};
    use pse_host::investigation::NativePath;
    use std::{io, path::PathBuf};

    let parent = tempfile::tempdir().unwrap();
    let absent = parent.path().join("no-configured-service");
    let cancel = pse_columnar::CancellationToken::new();
    for (path, timeout, invalid_selection) in [
        (PathBuf::from("relative-state"), None, true),
        (absent.clone(), Some(25_001), true),
        (absent.clone(), None, false),
    ] {
        let bytes = super::request(
            "exact-project-read",
            Request::Project {
                canonical: ProjectReadSelection {
                    state: NativePath::from(path),
                    database: None,
                    expected_issuer: None,
                    timeout_ms: timeout,
                },
                query: ProjectRead::Show {
                    project: "p".into(),
                },
            },
        );
        let refusal = encode(&bytes, &cancel).unwrap();
        let value: serde_json::Value = serde_json::from_slice(refusal.bytes()).unwrap();
        assert_eq!(value["correlation"], "exact-project-read");
        assert_eq!(value["payload"]["kind"], "refusal");
        assert_eq!(refusal.exit_code(), 2);
        assert!(matches!(
            refusal.delivery_failed(ReaderError::Io(io::Error::other("fixture delivery failed"))),
            ReaderError::Delivery { response, .. }
                if matches!(*response, ReaderError::Invalid(_)) && invalid_selection
                    || matches!(*response, ReaderError::Canonical(_)) && !invalid_selection
        ));
    }
    assert!(!absent.exists());
}

#[test]
fn project_study_workbench_reads_the_selected_store_and_refuses_foreign_cursor() {
    use super::super::{ProjectRead, ProjectReadSelection, Request, encode};
    use crate::workflow::ResearchReadRequest;
    use pse_host::investigation::NativePath;
    use pse_operations::{
        canonical_control::StudyWorkbenchRequest, native_fixture::canonical_fixture_store,
    };

    let store = canonical_fixture_store().unwrap();
    let issuing_store = store.issuing_store().unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    let query = |expected_issuer: &str, query| {
        let bytes = super::request(
            "study-workbench-read",
            Request::Project {
                canonical: ProjectReadSelection {
                    state: NativePath::from(store.deployment_state().to_owned()),
                    database: Some(store.database().into()),
                    expected_issuer: Some(expected_issuer.into()),
                    timeout_ms: None,
                },
                query,
            },
        );
        let result = encode(&bytes, &cancel).unwrap();
        let value: serde_json::Value = serde_json::from_slice(result.bytes()).unwrap();
        assert_eq!(value["correlation"], "study-workbench-read");
        (result.exit_code(), value["payload"].clone())
    };
    let browse = |cursor| ProjectRead::StudyWorkbench {
        request: Box::new(StudyWorkbenchRequest {
            filter: Default::default(),
            maximum: 1,
            cursor,
        }),
    };
    let (exit, payload) = query(issuing_store, browse(None));
    assert_eq!(exit, 0, "{payload}");
    assert_eq!(payload["kind"], "project");
    assert_eq!(payload["target"]["issuing_store"], issuing_store);
    assert_eq!(payload["target"]["database"], store.database());
    assert_eq!(payload["target"]["namespace"], store.namespace());
    assert_eq!(payload["result"]["kind"], "study_workbench");
    assert_eq!(payload["result"]["page"]["headers"], serde_json::json!([]));
    assert!(payload["result"]["page"]["next"].is_null());

    let foreign = serde_json::from_value(serde_json::json!({
        "issuing_store": "different-store",
        "filter": {},
        "after": "a",
    }))
    .unwrap();
    let (exit, payload) = query(issuing_store, browse(Some(foreign)));
    assert_eq!(exit, 2, "{payload}");
    assert_eq!(payload["kind"], "refusal");
    assert!(payload.to_string().contains("workbench cursor identity"));

    let (exit, payload) = query(issuing_store, ProjectRead::Target);
    assert_eq!(exit, 0, "{payload}");
    assert_eq!(payload["target"]["issuing_store"], issuing_store);
    assert_eq!(payload["result"]["kind"], "target");

    let (exit, payload) = query("different-store", ProjectRead::Target);
    assert_eq!(exit, 2, "{payload}");
    assert_eq!(payload["kind"], "refusal");
    assert!(payload.to_string().contains("expected project issuer"));

    let (exit, payload) = query(
        issuing_store,
        ProjectRead::ResearchRead {
            request: ResearchReadRequest::Dataset {
                key: "absent".into(),
            },
        },
    );
    assert_eq!(exit, 0, "{payload}");
    assert_eq!(payload["result"]["kind"], "research_read");
    assert_eq!(payload["result"]["result"]["kind"], "dataset");
    assert!(payload["result"]["result"]["row"].is_null());
    assert!(payload["result"]["result"]["members"].is_null());

    let (exit, payload) = query(
        issuing_store,
        ProjectRead::ResearchRead {
            request: ResearchReadRequest::Datasets {
                project: "p".into(),
                after: String::new(),
                limit: 129,
            },
        },
    );
    assert_eq!(exit, 2, "{payload}");
    assert!(payload.to_string().contains("bounded protocol batch"));
}

#[test]
fn native_retained_read_model_uses_explicit_pool_and_disabled_spill() {
    let model = super::super::canonical::model(NonZeroUsize::new(256 << 20).unwrap()).unwrap();
    assert!(
        matches!(model.factory().pool().memory_limit(),datafusion::execution::memory_pool::MemoryLimit::Finite(limit) if limit==256<<20)
    );
    assert!(
        !model
            .native_state()
            .runtime_env()
            .disk_manager
            .tmp_files_enabled()
    );
}
