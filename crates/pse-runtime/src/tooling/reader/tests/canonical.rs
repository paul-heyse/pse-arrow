// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::num::NonZeroUsize;

fn selection(state: PathBuf) -> CanonicalReadSelection {
    CanonicalReadSelection {
        state,
        database: None,
        memory_bytes: NonZeroUsize::new(256 << 20).unwrap(),
        timeout_ms: 5000,
    }
}
#[test]
fn canonical_request_preflight_precedes_any_configured_connection_or_state_creation() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("never-created");
    let body = request(
        "invalid-history",
        Request::Study {
            canonical: selection(state.clone()),
            request: crate::workflow::StudyObservationRequest {
                study: "original-study".into(),
                maximum_members: 1,
                maximum_events: 0,
            },
            cursor: None,
        },
    );
    let encoded = encode(&body).unwrap();
    assert_eq!(encoded.exit_code(), 2);
    let response: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(response["correlation"], "invalid-history");
    assert!(
        response["payload"]["failure"]["message"]
            .as_str()
            .unwrap()
            .contains("allowance outside bounds")
    );
    assert!(!state.exists());
    let body = request(
        "missing-existing-owner",
        Request::Attempt {
            canonical: selection(state.clone()),
            request: crate::workflow::AttemptObservationRequest {
                run: "original-run".into(),
                attempt: "original-attempt".into(),
                maximum_events: 1,
                cursor: None,
            },
        },
    );
    let encoded = encode(&body).unwrap();
    assert_eq!(encoded.exit_code(), 2);
    let response: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(response["payload"]["kind"], "refusal");
    assert!(!state.exists());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let mut invalid = selection(state);
    invalid.timeout_ms = 25_001;
    assert!(invalid.validate().is_err());
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
#[test]
fn product_discovery_matches_the_native_request_enum_schema_without_an_operation_copy() {
    let discovery = dispatch(&request("capabilities", Request::Discover));
    let Payload::Discovery { operations, .. } = discovery.payload else {
        panic!("native discovery")
    };
    let schema = schema(Contract::Request);
    let request = &schema.as_value()["$defs"]["Request"]["oneOf"];
    let mut schema_operations = request
        .as_array()
        .unwrap()
        .iter()
        .map(|variant| {
            variant["properties"]["operation"]["const"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    let mut operations = operations
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    schema_operations.sort();
    operations.sort();
    assert_eq!(operations, schema_operations);
    assert!(operations.iter().any(|operation| operation == "study"));
    assert!(operations.iter().any(|operation| operation == "attempt"));
}
