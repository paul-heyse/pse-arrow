// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit live isolated-service controls. No existing database is connected or reset.
#![allow(
    clippy::print_stderr,
    reason = "explicit isolated lifecycle identities support independent native readback"
)]
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    native_service::{
        NativeCanonicalService, NativeModuleSource, NativeServiceError, NativeServiceSpec,
    },
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn fixture_spec() -> Result<NativeServiceSpec, NativeServiceError> {
    let environment = std::env::vars_os().collect();
    let server = pse_host::server::select(&environment)?;
    let module_source = if let Some(record) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
    {
        NativeModuleSource::Prepared(pse_host::study_module::select(
            PathBuf::from(record).as_path(),
            pse_host::study_module::Variant::Production,
        )?)
    } else {
        NativeModuleSource::RecordedState(PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE").ok_or(
                NativeServiceError::Refused(
                    "explicit prepared receipt or historical module state required",
                ),
            )?,
        ))
    };
    Ok(NativeServiceSpec {
        server,
        helper: PathBuf::from(
            std::env::var_os("CARGO_BIN_EXE_pse-canonical-service").ok_or(
                NativeServiceError::Refused(
                    "Cargo/nextest must supply the selected runtime service executable",
                ),
            )?,
        ),
        module_source,
        state_parent: std::env::temp_dir(),
    })
}

// The fixture may select retained immutable bytes directly without synthesizing
// a deployment state. Native artifact admission still validates the exact record.
fn predecessor_module() -> Result<NativeModuleSource, NativeServiceError> {
    if let Some(record) = std::env::var_os(pse_host::fixtures::PREDECESSOR_MODULE_RECEIPT_ENV) {
        Ok(NativeModuleSource::Prepared(
            pse_host::study_module::select(
                PathBuf::from(record).as_path(),
                pse_host::study_module::Variant::Production,
            )?,
        ))
    } else {
        Ok(NativeModuleSource::RecordedState(PathBuf::from(
            std::env::var_os(pse_host::fixtures::PREDECESSOR_MODULE_STATE_ENV).ok_or(
                NativeServiceError::Refused(
                    "explicit predecessor receipt or recorded module state required",
                ),
            )?,
        )))
    }
}
fn fixture_with_schema(pre_epoch: bool) -> Result<NativeCanonicalService, NativeServiceError> {
    let spec = fixture_spec()?;
    let service = if pre_epoch {
        NativeCanonicalService::new_pre_epoch_fixture(spec)?
    } else {
        NativeCanonicalService::new(spec)?
    };
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
    let mut service = fixture_with_schema(false).unwrap();
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
async fn native_service_process_stop_joins_without_database_transport() {
    let mut service = fixture_with_schema(false).unwrap();
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
async fn native_service_rejects_issued_session_on_another_accepted_connection() {
    use surrealdb_protocol::proto::rpc::v1::{AttachSessionRequest, RequestContext};

    let mut service = fixture_with_schema(false).unwrap();
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
async fn native_service_adjacent_predecessor_migrates_only_after_original_native_join() {
    let mut service = fixture_with_schema(true).unwrap();
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                assert!(
                    service.ready().is_err(),
                    "the supported predecessor does not grant current readiness"
                );
                let options = CanonicalOptions::from_state(service.state())?;
                assert!(CanonicalStore::connect_ready(&options).await.is_err());
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                assert_eq!(
                    service
                        .probe_ingress(Instant::now() + Duration::from_secs(30))
                        .await?,
                    "maintenance"
                );
                let maintenance = service.maintenance().await?;
                let outcome = async {
                    let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                    let pending = maintenance.begin(request.clone()).await.inspect_err(|error| eprintln!("pre-epoch migration begin failed: {error:?}"))?;
                    let quiescent = pending
                        .quiesce(
                            service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                            &[],
                        )
                        .await.inspect_err(|error| eprintln!("pre-epoch migration quiesce failed: {error:?}"))?;
                    let ready = match quiescent.apply().await {
                        Ok(ready) => ready,
                        Err(error) => {
                            let receipt = maintenance.reconcile_receipt(&request).await;
                            eprintln!("pre-epoch migration apply failed: {error:?}; original receipt readback: {receipt:?}");
                            return Err(error.into());
                        }
                    };
                    service.retain_connection(ready.store().clone())?;
                    ready.store().canonical_run("native-fixture-probe").await?;
                    let reconciled = maintenance
                        .reconcile(&request)
                        .await?
                        .expect("original immutable activation receipt must reconcile");
                    assert_eq!(reconciled.fence(), ready.fence());
                    service.retain_connection(reconciled.store().clone())?;
                    let drains = futures_util::future::join_all([
                        ready.store().drain_owned_connection_until(
                            pse_operations::transport::original_deadline(Duration::from_secs(30)),
                        ),
                        reconciled.store().drain_owned_connection_until(
                            pse_operations::transport::original_deadline(Duration::from_secs(30)),
                        ),
                    ])
                    .await;
                    for drain in drains {
                        drain?;
                    }
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
}

#[tokio::test]
async fn native_service_listener_collision_refuses_foreign_listener_adoption() {
    let mut service = fixture_with_schema(false).unwrap();
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
    let mut service = fixture_with_schema(false).unwrap();
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
async fn native_service_retained_nonempty_v2_reopens_then_migrates_with_original_identity() {
    let mut service = NativeCanonicalService::new_v2_fixture(fixture_spec().unwrap()).unwrap();
    let state = service.state().to_path_buf();
    let original_unit = service.intent().unit().to_owned();
    eprintln!(
        "native retained fixture state={} original_service={original_unit}",
        state.display()
    );
    let config = std::fs::read(state.join("config.json")).unwrap();
    let credentials = std::fs::read(state.join("credentials.json")).unwrap();
    let module = std::fs::read(state.join("original-module-receipt.json")).unwrap();
    let original = service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                assert!(service.ready().is_err());
                service
                    .start_ingress(Instant::now() + Duration::from_secs(30))
                    .await?;
                report_ingresses(service);
                assert_eq!(
                    service
                        .probe_ingress(Instant::now() + Duration::from_secs(30))
                        .await?,
                    "maintenance"
                );
                let maintenance = service.maintenance().await?;
                let original = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                let encoded = serde_json::to_value(&original).unwrap();
                assert_eq!(encoded["predecessor"]["schema_version"], 2);
                assert_eq!(
                    encoded["predecessor"]["schema_digest"],
                    pse_schema::catalog::substrate::SCHEMA_V2_DIGEST
                );
                maintenance.disconnect().await?;
                service
                    .stop_ingress_and_join(Instant::now() + Duration::from_secs(30))
                    .await?;
                Ok(original)
            })
        })
        .await
        .unwrap();
    assert!(
        std::fs::read_dir(state.join("database"))
            .unwrap()
            .next()
            .is_some()
    );
    assert!(
        service
            .launch(Instant::now() + Duration::from_secs(1))
            .await
            .is_err()
    );
    let stopped = service.try_into_stopped().unwrap();
    assert_eq!(stopped.state(), state);
    let mut service = stopped.reopen().unwrap();
    assert_eq!(service.state(), state);
    assert_ne!(service.intent().unit(), original_unit);
    eprintln!(
        "native retained fixture state={} reopened_service={}",
        state.display(),
        service.intent().unit()
    );
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                assert!(service.ready().is_err());
                let maintenance = service.maintenance().await?;
                let inspected = maintenance.inspect().await?;
                let recorded = serde_json::to_value(&original).unwrap();
                assert_eq!(
                    serde_json::to_value(&inspected.predecessor).unwrap(),
                    recorded["predecessor"]
                );
                let pending = maintenance.begin(original.clone()).await?;
                let closed = pending
                    .quiesce(
                        service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                        &[],
                    )
                    .await?;
                let ready = closed.apply().await?;
                service.retain_connection(ready.store().clone())?;
                assert_eq!(
                    ready.fence().version(),
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                assert_eq!(
                    ready.fence().issuer(),
                    recorded["predecessor"]["issuing_store"].as_str().unwrap()
                );
                let after =
                    serde_json::to_value(&maintenance.inspect().await?.predecessor).unwrap();
                assert_eq!(
                    after["payload_incarnation"],
                    recorded["predecessor"]["payload_incarnation"]
                );
                assert_eq!(
                    after["study_module_sha256"],
                    recorded["predecessor"]["study_module_sha256"]
                );
                let receipt = maintenance.reconcile_receipt(&original).await?.unwrap();
                assert_eq!(receipt.request(), &original);
                ready.store().canonical_run("native-fixture-probe").await?;
                ready
                    .store()
                    .drain_owned_connection_until(pse_operations::transport::original_deadline(
                        Duration::from_secs(30),
                    ))
                    .await?;
                maintenance.disconnect().await?;
                Ok(())
            })
        })
        .await
        .unwrap();
    assert_eq!(std::fs::read(state.join("config.json")).unwrap(), config);
    assert_eq!(
        std::fs::read(state.join("credentials.json")).unwrap(),
        credentials
    );
    assert_eq!(
        std::fs::read(state.join("original-module-receipt.json")).unwrap(),
        module
    );
    assert_eq!(std::fs::read_dir(state.join("runs")).unwrap().count(), 2);
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_original_v2_target_and_old_module_resume_after_restart() {
    use pse_operations::canonical::migration::SchemaMigrationRequest;
    let mut spec = fixture_spec().unwrap();
    spec.module_source = predecessor_module().unwrap();
    let mut service = NativeCanonicalService::new_pre_epoch_fixture(spec).unwrap();
    let state = service.state().to_owned();
    let original = service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let maintenance = service.maintenance().await?;
                // Decode the exact historical wire shape; current constructors correctly
                // issue the current target. No effect has yet been dispatched under this original intent.
                let mut wire =
                    serde_json::to_value(maintenance.request_current(uuid::Uuid::new_v4()).await?)
                        .unwrap();
                wire["target_version"] =
                    serde_json::json!(pse_schema::catalog::substrate::SCHEMA_V2_VERSION);
                wire["target_digest"] =
                    serde_json::json!(pse_schema::catalog::substrate::SCHEMA_V2_DIGEST);
                let original: SchemaMigrationRequest = serde_json::from_value(wire).unwrap();
                maintenance
                    .begin(original.clone())
                    .await?
                    .quiesce(async { Ok(()) }, &[])
                    .await?;
                assert!(maintenance.reconcile_receipt(&original).await?.is_none());
                Ok(original)
            })
        })
        .await
        .unwrap();
    let original_wire = serde_json::to_value(&original).unwrap();
    let mut service = service.try_into_stopped().unwrap().reopen().unwrap();
    service
        .run(Instant::now() + Duration::from_secs(180), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let maintenance = service.maintenance().await?;
                let receipt = maintenance
                    .begin(original.clone())
                    .await?
                    .quiesce(async { Ok(()) }, &[])
                    .await?
                    .apply_receipt()
                    .await?;
                assert_eq!(
                    serde_json::to_value(receipt.request()).unwrap(),
                    original_wire
                );
                let replay = maintenance
                    .reconcile_receipt(&original)
                    .await?
                    .expect("original v2 receipt survives restart");
                assert_eq!(
                    serde_json::to_value(replay.request()).unwrap(),
                    original_wire
                );
                let original_receipt = replay;
                assert!(
                    maintenance.reconcile(&original).await.is_err(),
                    "a v2 receipt cannot grant current readiness"
                );
                let options = CanonicalOptions::from_state(service.state())?;
                assert!(CanonicalStore::connect_ready(&options).await.is_err());
                let current = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                let current_wire = serde_json::to_value(&current).unwrap();
                assert_eq!(
                    current_wire["target_version"],
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                assert_ne!(
                    current_wire["target_digest"],
                    original_wire["target_digest"]
                );
                assert_ne!(current_wire["operation"], original_wire["operation"]);
                assert_ne!(
                    current_wire["successor_epoch"],
                    original_wire["successor_epoch"]
                );
                // The current authored capture transaction requires the same pure
                // module API as this genuine retained artifact. Artifact age alone
                // cannot refuse a separately issued current-target activation.
                let activated = maintenance
                    .begin(current.clone())
                    .await?
                    .quiesce(async { Ok(()) }, &[])
                    .await?
                    .apply_receipt()
                    .await?;
                assert_eq!(
                    serde_json::to_value(activated.request()).unwrap(),
                    current_wire
                );
                for field in [
                    "issuing_store",
                    "payload_incarnation",
                    "study_module_sha256",
                ] {
                    assert_eq!(
                        current_wire["predecessor"][field],
                        original_wire["predecessor"][field]
                    );
                }
                let ready = maintenance
                    .reconcile(&current)
                    .await?
                    .expect("only the separately issued current activation grants readiness");
                assert_eq!(
                    ready.fence().version(),
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                assert_eq!(
                    ready.fence().epoch(),
                    current_wire["successor_epoch"].as_str().unwrap()
                );
                assert_eq!(
                    ready.fence().issuer(),
                    original_wire["predecessor"]["issuing_store"]
                        .as_str()
                        .unwrap()
                );
                ready.store().canonical_run("native-fixture-probe").await?;
                ready
                    .store()
                    .drain_owned_connection_until(pse_operations::transport::original_deadline(
                        Duration::from_secs(30),
                    ))
                    .await?;
                let after =
                    serde_json::to_value(&maintenance.inspect().await?.predecessor).unwrap();
                for field in [
                    "issuing_store",
                    "payload_incarnation",
                    "study_module_sha256",
                ] {
                    assert_eq!(after[field], original_wire["predecessor"][field]);
                }
                assert_eq!(after["schema_digest"], current_wire["target_digest"]);
                assert_eq!(
                    maintenance.reconcile_receipt(&original).await?.unwrap(),
                    original_receipt
                );
                assert!(maintenance.reconcile(&original).await.is_err());
                Ok(())
            })
        })
        .await
        .unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_retained_v2_old_module_upgrades_to_explicit_prepared_target()
-> Result<(), NativeServiceError> {
    retained_old_module_upgrade(false).await
}

#[tokio::test]
async fn native_service_retained_v3_old_module_upgrades_to_explicit_prepared_target()
-> Result<(), NativeServiceError> {
    retained_old_module_upgrade(true).await
}

async fn retained_old_module_upgrade(schema_v3: bool) -> Result<(), NativeServiceError> {
    let mut spec = fixture_spec()?;
    spec.module_source = predecessor_module()?;
    let target = if let Some(record) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT") {
        NativeModuleSource::Prepared(pse_host::study_module::select(
            &PathBuf::from(record),
            pse_host::study_module::Variant::Production,
        )?)
    } else {
        NativeModuleSource::RecordedState(PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_TARGET_MODULE_STATE").ok_or(
                NativeServiceError::Refused("explicit new module source state required"),
            )?,
        ))
    };
    let mut service = if schema_v3 {
        NativeCanonicalService::new_v3_upgrade_fixture(spec, target)?
    } else {
        NativeCanonicalService::new_v2_upgrade_fixture(spec, target)?
    };
    let state = service.state().to_path_buf();
    let configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("config.json"))?)
            .map_err(std::io::Error::other)?;
    let old_hash = configuration["study_module"]["sha256"]
        .as_str()
        .ok_or(NativeServiceError::Refused(
            "original module hash absent from fixture configuration",
        ))?
        .to_owned();
    let target_hash = configuration["schema_migration_target"]["sha256"]
        .as_str()
        .ok_or(NativeServiceError::Refused(
            "migration target module hash absent from fixture configuration",
        ))?
        .to_owned();
    assert_ne!(old_hash, target_hash);
    let credentials = std::fs::read(state.join("credentials.json"))?;
    let original_receipt = std::fs::read(state.join("original-module-receipt.json"))?;
    eprintln!(
        "native module-upgrade fixture state={} original_service={} old_module={old_hash} target_module={target_hash}",
        state.display(),
        service.intent().unit()
    );
    let request = service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let maintenance = service.maintenance().await?;
                let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
                maintenance.disconnect().await?;
                Ok(request)
            })
        })
        .await?;
    let recorded = serde_json::to_vec(&request).map_err(std::io::Error::other)?;
    let old = serde_json::to_value(&request).map_err(std::io::Error::other)?;
    assert_eq!(old["predecessor"]["study_module_sha256"], old_hash);
    assert_eq!(old["target_module"]["receipt"]["sha256"], target_hash);
    let stopped = service.try_into_stopped().map_err(std::io::Error::other)?;
    let mut service = stopped.reopen().map_err(std::io::Error::other)?;
    eprintln!(
        "native module-upgrade fixture state={} reopened_service={}",
        state.display(),
        service.intent().unit()
    );
    service
        .run(Instant::now() + Duration::from_secs(240), |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let maintenance = service.maintenance().await?;
                assert_eq!(
                    serde_json::to_value(maintenance.inspect().await?.predecessor)
                        .map_err(std::io::Error::other)?,
                    old["predecessor"]
                );
                let pending = maintenance.begin(request.clone()).await?;
                let closed = pending
                    .quiesce(
                        service.stop_ingress_and_join(Instant::now() + Duration::from_secs(30)),
                        &[],
                    )
                    .await?;
                let receipt = closed.apply_receipt().await?;
                assert_eq!(
                    serde_json::to_vec(receipt.request()).map_err(std::io::Error::other)?,
                    recorded
                );
                // Publication is explicitly separate from the committed database activation.
                assert!(
                    CanonicalStore::connect_ready(&CanonicalOptions::from_state(service.state())?)
                        .await
                        .is_err()
                );
                service.promote_activated_module(&receipt)?;
                // Recover selection from the original durable receipt after discarding local success.
                let recovered = maintenance.reconcile_receipt(&request).await?.ok_or(
                    NativeServiceError::Refused(
                        "original module activation receipt absent during publication recovery",
                    ),
                )?;
                service.promote_activated_module(&recovered)?;
                let ready = service.connect().await?;
                assert_eq!(
                    ready.fence().version(),
                    pse_schema::catalog::substrate::SCHEMA_VERSION
                );
                let marker = serde_json::to_value(maintenance.inspect().await?.predecessor)
                    .map_err(std::io::Error::other)?;
                assert_eq!(marker["study_module_sha256"], target_hash);
                for field in ["issuing_store", "payload_incarnation", "interpretation"] {
                    assert_eq!(marker[field], old["predecessor"][field]);
                }
                assert_eq!(
                    marker["study_module_sha256"],
                    receipt
                        .target_module()
                        .ok_or(NativeServiceError::Refused(
                            "original activation receipt has no prepared target module"
                        ),)?
                        .sha256()
                );
                let reconciled = maintenance.reconcile_receipt(&request).await?.ok_or(
                    NativeServiceError::Refused(
                        "original module activation receipt absent during final reconciliation",
                    ),
                )?;
                assert_eq!(reconciled.request(), &request);
                ready.store().canonical_run("native-fixture-probe").await?;
                ready
                    .store()
                    .drain_owned_connection_until(pse_operations::transport::original_deadline(
                        Duration::from_secs(30),
                    ))
                    .await?;
                maintenance.disconnect().await?;
                Ok(())
            })
        })
        .await?;
    assert_eq!(std::fs::read(state.join("credentials.json"))?, credentials);
    assert_eq!(
        std::fs::read(state.join("original-module-receipt.json"))?,
        original_receipt
    );
    service.dispose()?;
    assert!(!state.exists());
    Ok(())
}

#[tokio::test]
async fn native_service_closed_admission_preserves_original_attempt_settlement_only() {
    use pse_operations::canonical_execution::RunRequest;
    let mut service = fixture_with_schema(false).unwrap();
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
