// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit live isolated-service controls. No existing database is connected or reset.
#![allow(
    clippy::print_stderr,
    reason = "explicit isolated lifecycle identities support independent native readback"
)]
use pse_host::extraction::TargetSetupRefusal;
use pse_operations::{
    canonical::{CanonicalError, CanonicalOptions, CanonicalStore},
    native_service::{
        NativeCanonicalService, NativeServiceError, NativeServiceSpec,
    },
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn fixture_spec() -> Result<NativeServiceSpec, NativeServiceError> {
    let helper = std::env::var_os("CARGO_BIN_EXE_pse-canonical-service").ok_or(
        NativeServiceError::Refused(
            "Cargo/nextest must supply the selected runtime service executable",
        ),
    )?;
    NativeServiceSpec::selected_fixture(Some(helper.into()))
}

fn current_fixture() -> Result<NativeCanonicalService, NativeServiceError> {
    let spec = fixture_spec().inspect_err(|_| {
        eprintln!("{}", TargetSetupRefusal::NativeServiceInputs.marker());
    })?;
    let service = NativeCanonicalService::new(spec).inspect_err(|_| {
        eprintln!("{}", TargetSetupRefusal::NativeServiceConstruction.marker());
    })?;
    eprintln!(
        "native fixture state={} original_service={}",
        service.state().display(),
        service.intent().unit()
    );
    Ok(service)
}
fn assert_schema_refusal(
    error: pse_operations::canonical::CanonicalError,
    expected: pse_operations::canonical::RefusalCode,
) -> Result<(), pse_operations::canonical::CanonicalError> {
    match error {
        pse_operations::canonical::CanonicalError::Refused { code, .. } => {
            assert_eq!(code, expected);
            Ok(())
        }
        pse_operations::canonical::CanonicalError::SourceEditRefused { source, .. } => {
            assert_schema_refusal(*source, expected)
        }
        error => Err(error),
    }
}
fn report_ingresses(service: &NativeCanonicalService) {
    for intent in service.ingress_intents() {
        eprintln!(
            "native fixture state={} original_ingress={}",
            service.state().display(),
            intent.unit()
        );
    }
}

async fn raw_native_rpc(
    address: std::net::SocketAddr,
) -> surrealdb_protocol::proto::rpc::v1::surreal_db_service_client::SurrealDbServiceClient<
    tonic::transport::Channel,
> {
    let channel = tonic::transport::Endpoint::from_shared(format!("http://{address}"))
        .unwrap()
        .connect_timeout(Duration::from_secs(5))
        .connect()
        .await
        .unwrap();
    surrealdb_protocol::proto::rpc::v1::surreal_db_service_client::SurrealDbServiceClient::new(
        channel,
    )
}

#[tokio::test]
async fn native_service_current_migration_joins_actual_ingress_and_reconstructs_ready() {
    let mut service = current_fixture().unwrap();
    let state = service.state().to_path_buf();
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                let old = service.ready()?.clone();
                assert_eq!(
                    service
                        .probe_ingress(Instant::now() + Duration::from_secs(30))
                        .await?,
                    old.fence().epoch()
                );
                let peer = service.connect().await?;
                // Remains fixture-owned and physically live, but is not an active writer.
                let idle = service.connect().await?;
                let options = CanonicalOptions::from_state(service.state())?;
                let maintenance = service.maintenance().await?;
                let outcome = async {
                    let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                    let pending = maintenance.begin(request).await?;
                    old.store().canonical_run("native-fixture-probe").await?;
                    assert_schema_refusal(
                        old.store()
                            .edit("closed-admission", None, "refused", &[])
                            .await
                            .unwrap_err(),
                        pse_operations::canonical::RefusalCode::SchemaMigrationClosed,
                    )?;
                    assert!(
                        CanonicalStore::connect_ready(&options).await.is_err(),
                        "new opens must refuse while native epoch admission is closed"
                    );
                    let closed = pending
                        .quiesce(
                            service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                            &[old.store().clone(), peer.store().clone()],
                        )
                        .await?;
                    let successor = closed.apply().await?;
                    service.retain_connection(successor.store().clone())?;
                    assert_ne!(successor.fence().epoch(), old.fence().epoch());
                    assert_eq!(successor.fence().issuer(), old.fence().issuer());
                    assert_schema_refusal(
                        idle.store()
                            .edit("stale-epoch", None, "refused", &[])
                            .await
                            .unwrap_err(),
                        pse_operations::canonical::RefusalCode::SchemaEpochChanged,
                    )?;
                    assert!(
                        old.store()
                            .canonical_run("native-fixture-probe")
                            .await
                            .is_err(),
                        "the physically drained original handle cannot revive"
                    );
                    successor
                        .store()
                        .canonical_run("native-fixture-probe")
                        .await?;
                    successor
                        .store()
                        .drain_owned_connection_until(pse_operations::transport::original_deadline(
                            Duration::from_secs(30),
                        ))
                        .await?;
                    Ok::<(), NativeServiceError>(())
                }
                .await;
                let cleanup = maintenance.disconnect().await;
                outcome?;
                cleanup?;
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_refuses_foreign_catalog_version_and_payload_before_maintenance() {
    use pse_operations::transport::client::{Client, RequestContext, Root};

    let mut service = current_fixture().unwrap();
    let deadline = Instant::now() + Duration::from_secs(90);
    service
        .run(deadline, |service| {
            Box::pin(async move {
                service.launch(deadline).await?;
                let options = CanonicalOptions::from_state(service.state())?;
                let maintenance = service.maintenance().await?;
                let address = options.endpoint.strip_prefix("grpc://").unwrap().parse().unwrap();
                let raw = Client::connect(address, pse_operations::canonical::MESSAGE_BYTES)
                    .await
                    .map_err(CanonicalError::from)?;
                let context = || RequestContext::control(deadline);
                raw.signin(Root {
                    username: options.username.clone(),
                    password: options.password.clone(),
                })
                .request_context(context())
                .await
                .map_err(CanonicalError::from)?;
                raw.use_ns(&options.namespace)
                    .use_db(&options.database)
                    .request_context(context())
                    .await
                    .map_err(CanonicalError::from)?;
                let original: surrealdb_types::Object = service
                    .fixture_query(
                        "SELECT * FROM ONLY canonical_interpretations:current;",
                        surrealdb_types::Object::new(),
                    )
                    .await?
                    .take::<Option<_>>(0)
                    .map_err(CanonicalError::from)?
                    .expect("original fixture marker");
                let version = original.get("schema_version").unwrap().clone();
                let payload = original.get("payload_incarnation").unwrap().clone();
                let other_version = pse_operations::canonical_codec::encode_uint(3)
                    .map_err(CanonicalError::from)?;
                let other_payload = surrealdb_types::Value::String(uuid::Uuid::new_v4().to_string());
                let update = "UPDATE ONLY canonical_interpretations:current SET schema_version=$version,payload_incarnation=$payload RETURN AFTER;";
                for (version, payload) in [
                    (other_version, payload.clone()),
                    (version.clone(), other_payload),
                ] {
                    let mut replacement = surrealdb_types::Object::new();
                    replacement.insert("version", version);
                    replacement.insert("payload", payload);
                    let before: Option<surrealdb_types::Object> = raw
                        .query(update)
                        .bind(replacement)
                        .request_context(context())
                        .await
                        .map_err(CanonicalError::from)?
                        .take(0)
                        .map_err(CanonicalError::from)?;
                    assert!(before.is_some());
                    assert!(matches!(
                        CanonicalStore::initialize_database(&options).await,
                        Err(CanonicalError::Configuration(_))
                    ));
                    assert!(matches!(
                        CanonicalStore::connect_ready(&options).await,
                        Err(CanonicalError::Configuration(_))
                    ));
                    assert!(matches!(
                        maintenance.request_current(uuid::Uuid::new_v4()).await,
                        Err(CanonicalError::Configuration(_))
                    ));
                    let mut after = raw
                        .query(
                            "SELECT * FROM ONLY canonical_interpretations:current; SELECT * FROM ONLY canonical_maintenance_operations:current;",
                        )
                        .request_context(context())
                        .await
                        .map_err(CanonicalError::from)?;
                    let marker: Option<surrealdb_types::Object> =
                        after.take(0).map_err(CanonicalError::from)?;
                    let pending: Option<surrealdb_types::Object> =
                        after.take(1).map_err(CanonicalError::from)?;
                    assert_eq!(marker, before);
                    assert!(pending.is_none());
                }
                let mut restored = surrealdb_types::Object::new();
                restored.insert("version", version);
                restored.insert("payload", payload);
                raw.query(update)
                    .bind(restored)
                    .request_context(context())
                    .await
                    .map_err(CanonicalError::from)?
                    .check()
                    .map_err(CanonicalError::from)?;
                raw.disconnect().await.map_err(CanonicalError::from)?;
                maintenance.disconnect().await?;
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
async fn native_service_process_stop_joins_without_database_transport() {
    let mut service = current_fixture().unwrap();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let stopped = service
                    .stop_process(Instant::now() + Duration::from_secs(30))
                    .await?;
                assert_eq!(stopped.main_pid, 0);
                assert_eq!(
                    service
                        .stop_process(Instant::now() + Duration::from_secs(30))
                        .await?
                        .invocation,
                    stopped.invocation
                );
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
#[ignore = "requires two separately managed invocations and one private overlap rendezvous"]
async fn native_service_independent_invocations_overlap_on_private_servers() {
    let exchange = PathBuf::from(std::env::var_os("PSE_TEST_PRIVATE_SERVICE_OVERLAP").unwrap());
    let participant = std::env::var("PSE_TEST_PRIVATE_SERVICE_PARTICIPANT").unwrap();
    let sibling = match participant.as_str() {
        "first" => "second",
        "second" => "first",
        _ => panic!("select one exact overlap participant"),
    };
    assert!(exchange.is_absolute() && exchange.is_dir());
    let mut service = current_fixture().unwrap();
    let state = service.state().to_owned();
    let work_end = Instant::now() + Duration::from_secs(120);
    service
        .run_until(work_end, work_end + Duration::from_secs(30), |service| {
            Box::pin(async move {
                service.launch(work_end).await?;
                let publish =
                    |phase: &str, path: &std::path::Path| -> Result<(), NativeServiceError> {
                        let temporary = exchange.join(format!("{participant}-{phase}.tmp"));
                        std::fs::write(
                            &temporary,
                            serde_json::to_vec(path).map_err(std::io::Error::other)?,
                        )?;
                        std::fs::rename(
                            temporary,
                            exchange.join(format!("{participant}-{phase}.json")),
                        )?;
                        Ok(())
                    };
                let observe = |phase: &'static str| {
                    let path = exchange.join(format!("{sibling}-{phase}.json"));
                    async move {
                        loop {
                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    return serde_json::from_slice::<PathBuf>(&bytes)
                                        .map_err(std::io::Error::other)
                                        .map_err(NativeServiceError::from);
                                }
                                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                                Err(error) => return Err(error.into()),
                            }
                            if Instant::now() >= work_end {
                                return Err(NativeServiceError::Refused(
                                    "independent private service did not reach its rendezvous",
                                ));
                            }
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                };
                publish("ready", service.state())?;
                let other = observe("ready").await?;
                assert_ne!(other, service.state());
                assert!(other.join("config.json").is_file());
                assert!(service.ready()?.store().endpoint().starts_with("grpc://"));
                publish("observed", service.state())?;
                assert_eq!(observe("observed").await?, other);
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_rejects_issued_session_on_another_accepted_connection() {
    use surrealdb_protocol::proto::rpc::v1::{AttachSessionRequest, RequestContext};

    let mut service = current_fixture().unwrap();
    let state = service.state().to_path_buf();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let endpoint = service.ready()?.store().endpoint().to_owned();
                let address = endpoint
                    .strip_prefix("grpc://")
                    .and_then(|address| address.parse().ok())
                    .ok_or(NativeServiceError::Refused(
                        "invalid native fixture endpoint",
                    ))?;
                // Independently connected channels open separate accepted sockets.
                let mut original = raw_native_rpc(address).await;
                let mut other = raw_native_rpc(address).await;
                let issued = original
                    .attach_session(AttachSessionRequest::default())
                    .await
                    .unwrap()
                    .into_inner();
                assert!(issued.created);
                let session = issued.session.unwrap();
                let reattach = AttachSessionRequest {
                    context: Some(RequestContext {
                        session: Some(session.clone()),
                        ..Default::default()
                    }),
                };
                let refusal = other.attach_session(reattach.clone()).await.unwrap_err();
                assert_eq!(refusal.code(), tonic::Code::NotFound);
                let retained = original
                    .attach_session(reattach)
                    .await
                    .unwrap()
                    .into_inner();
                assert_eq!(retained.session, Some(session));
                assert!(!retained.created);
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_listener_collision_refuses_foreign_listener_adoption() {
    let mut service = current_fixture().unwrap();
    let configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(service.state().join("config.json")).unwrap())
            .unwrap();
    let endpoint = configuration["endpoint"]
        .as_str()
        .unwrap()
        .strip_prefix("grpc://")
        .unwrap();
    let foreign = std::net::TcpListener::bind(endpoint).unwrap();
    let result = service
        .run(Instant::now() + Duration::from_secs(120), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(60))
                    .await
            })
        })
        .await;
    assert!(
        result.is_err(),
        "a pre-existing socket cannot establish original-owned database readiness"
    );
    assert!(service.ready().is_err());
    drop(foreign);
    service.dispose().unwrap();
}

#[tokio::test]
async fn native_service_body_panic_still_joins_original_owned_units() {
    use futures_util::FutureExt;
    let mut service = current_fixture().unwrap();
    let outcome = std::panic::AssertUnwindSafe(service.run(
        Instant::now() + Duration::from_secs(180),
        |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                let _maintenance = service.maintenance().await?;
                panic!("intentional fixture body failure after actual native launch");
                #[allow(
                    unreachable_code,
                    reason = "test verifies panic settlement before normal return"
                )]
                Ok::<(), NativeServiceError>(())
            })
        },
    ))
    .catch_unwind()
    .await;
    assert!(outcome.is_err());
    service
        .shutdown(Instant::now() + Duration::from_secs(30))
        .await
        .unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
#[ignore = "manual managed-retention control deliberately fails after physical settlement"]
async fn native_service_managed_release_after_assertion_failure_control() {
    let mut service = current_fixture().unwrap();
    let state = service.state().to_owned();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
    panic!("intentional managed assertion failure after original service and ingress drain");
}

#[tokio::test]
async fn native_service_closed_admission_preserves_original_attempt_settlement_only() {
    use pse_operations::canonical_execution::RunRequest;
    let mut service = current_fixture().unwrap();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let original = service.ready()?.clone();
                let store = original.store();
                let revision = store
                    .edit("schema-settlement-source", None, "original", &[])
                    .await?;
                store
                    .begin_run(&RunRequest {
                        key: "schema-settlement-run".into(),
                        purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                        admission_study: None,
                        admission_point: None,
                        revision,
                        sources: vec![],
                        request: vec![1],
                        source_selection: vec![2],
                        attestation: vec![3],
                    })
                    .await?;
                let fence = store
                    .claim_run(
                        "schema-settlement-run",
                        "schema-original-claim",
                        "original-worker",
                        Duration::from_secs(120),
                    )
                    .await?;
                let maintenance = service.maintenance().await?;
                let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                let _pending = maintenance.begin(request).await?;
                assert_schema_refusal(
                    store
                        .cancel_run(fence.run(), "fresh-cancel-command")
                        .await
                        .unwrap_err(),
                    pse_operations::canonical::RefusalCode::SchemaMigrationClosed,
                )?;
                assert_schema_refusal(
                    store
                        .claim_run(
                            fence.run(),
                            "new-worker-claim",
                            "replacement-worker",
                            Duration::from_secs(120),
                        )
                        .await
                        .unwrap_err(),
                    pse_operations::canonical::RefusalCode::SchemaMigrationClosed,
                )?;
                let settled = store
                    .cancel_attempt(&fence, "original-attempt-cancellation")
                    .await?;
                assert!(settled.cancelled);
                let repeated = store
                    .cancel_attempt(&fence, "original-attempt-cancellation")
                    .await?;
                assert_eq!(settled, repeated);
                maintenance.disconnect().await?;
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
}

#[path = "native_service/http_control.rs"]
mod http_control;

#[path = "native_service/surrealkit.rs"]
mod surrealkit;

#[path = "native_service/installed_schema.rs"]
mod installed_schema;
