// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Connection-only CLI observation of an owned v4 canonical fixture.

use super::{NativeServiceError, fixture_with_schema};
use pse_operations::{generated::surreal::SCHEMA_DIGEST, native_service::NativeCanonicalService};
use serde_json::Value;
use std::{
    fs,
    net::{SocketAddr, TcpStream},
    path::Path,
    time::{Duration, Instant},
};
use surrealdb_types::{Object, Value as SqlValue};

async fn installed_schema(
    state: &Path,
    reader: &str,
    issuer: &str,
    timeout_ms: &str,
) -> (bool, Value) {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let result = tokio::process::Command::new(checkout.join("scripts/pse"))
        .current_dir(checkout)
        .args(["--format", "json", "project", "installed-schema", "--state"])
        .arg(state)
        .args([
            "--reader",
            reader,
            "--issuing-store",
            issuer,
            "--timeout-ms",
            timeout_ms,
        ])
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();
    let bytes = if result.stdout.is_empty() {
        &result.stderr
    } else {
        &result.stdout
    };
    let envelope =
        serde_json::from_slice(bytes).expect("installed-schema returns one JSON envelope");
    (result.status.success(), envelope)
}

async fn observed_state(service: &NativeCanonicalService) -> (SqlValue, SqlValue) {
    let mut result = service
        .fixture_query(
            "SELECT * FROM ONLY canonical_interpretations:current; INFO FOR DB;",
            Object::default(),
        )
        .await
        .unwrap();
    (result.take(0).unwrap(), result.take(1).unwrap())
}

fn local_state(state: &Path) -> (Vec<u8>, Vec<std::ffi::OsString>) {
    let config = fs::read(state.join("config.json")).unwrap();
    let mut entries: Vec<_> = fs::read_dir(state)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    entries.sort_unstable();
    (config, entries)
}

#[tokio::test]
#[ignore = "requires the explicitly prepared installed-schema product reader selection"]
async fn installed_schema_cli_observes_only_the_original_v4_issuer()
-> Result<(), NativeServiceError> {
    let reader = std::env::var("PSE_TEST_INSTALLED_SCHEMA_READER")
        .expect("pass the original `pse observe prepare --installed-schema` reader ID");
    let mut service = fixture_with_schema(false)?;
    let state = service.state().to_path_buf();
    let (issuer, address) = service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            let reader = reader.clone();
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let store = service.ready()?.store();
                let issuer = store.issuing_store()?.to_owned();
                let address: SocketAddr = store
                    .endpoint()
                    .strip_prefix("grpc://")
                    .unwrap()
                    .parse()
                    .unwrap();
                let namespace = store.namespace().to_owned();
                let database = store.database().to_owned();
                let original = observed_state(service).await;
                let local = local_state(service.state());
                let SqlValue::Object(marker) = &original.0 else {
                    panic!("original marker absent")
                };
                assert_eq!(marker.get("admission_open"), Some(&SqlValue::Bool(true)));
                let SqlValue::Object(catalog) = &original.1 else {
                    panic!("original catalog absent")
                };
                let Some(SqlValue::Object(tables)) = catalog.get("tables") else {
                    panic!("installed tables absent")
                };
                assert!(tables.get("__rollout").is_none());

                let (success, receipt) =
                    installed_schema(service.state(), &reader, &issuer, "20000").await;
                assert!(success, "{receipt}");
                assert_eq!(
                    receipt["observation"]["scope"],
                    "canonical_installed_schema"
                );
                let observation = &receipt["payload"]["product"]["observation"];
                assert_eq!(observation["target"]["issuing_store"], issuer);
                assert_eq!(observation["target"]["namespace"], namespace);
                assert_eq!(observation["target"]["database"], database);
                assert_eq!(observation["marker"]["issuing_store"], issuer);
                assert_eq!(observation["marker"]["schema_digest"], SCHEMA_DIGEST);
                let comparison = &observation["comparison"];
                assert!(
                    comparison["installed"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|item| {
                            item["kind"] == "table" && item["name"] == "canonical_interpretations"
                        })
                );
                for key in ["missing", "changed", "unexpected"] {
                    assert_eq!(comparison[key], serde_json::json!([]), "{key}");
                }

                let foreign = uuid::Uuid::new_v4().to_string();
                let (success, refusal) =
                    installed_schema(service.state(), &reader, &foreign, "20000").await;
                assert!(!success, "{refusal}");
                assert!(
                    refusal.to_string().contains(
                        "installed catalog issuing store differs from the selected issuer"
                    )
                );
                assert_eq!(observed_state(service).await, original);
                assert!(local_state(service.state()) == local);
                Ok((issuer, address))
            })
        })
        .await?;
    let stopped = local_state(&state);
    let unavailable = || TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err();
    assert!(unavailable());
    let (success, refusal) = installed_schema(&state, &reader, &issuer, "1000").await;
    assert!(!success, "{refusal}");
    assert_eq!(
        refusal["observation"]["scope"],
        "canonical_installed_schema"
    );
    assert!(unavailable());
    assert!(local_state(&state) == stopped);
    service.dispose()?;
    assert!(!state.exists());
    Ok(())
}
