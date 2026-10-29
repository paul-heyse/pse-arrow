// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit live isolated-service controls. No existing database is connected or reset.
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore},
    native_service::{NativeCanonicalService, NativeServiceSpec},
};
use std::{path::PathBuf, time::{Duration, Instant}};

fn fixture_with_schema(pre_epoch: bool) -> NativeCanonicalService {
    let environment = std::env::vars_os().collect();
    let server = pse_host::server::select(&environment).expect("explicit native immutable server preparation is required");
    let module_source_state = PathBuf::from(std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE").expect("explicit private original module receipt state is required"));
    let spec = NativeServiceSpec { server, helper: PathBuf::from(env!("CARGO_BIN_EXE_pse-canonical-service")), module_source_state, state_parent: std::env::temp_dir() };
    if pre_epoch { NativeCanonicalService::new_pre_epoch_fixture(spec).unwrap() } else { NativeCanonicalService::new(spec).unwrap() }
}

#[tokio::test]
async fn native_service_current_migration_joins_actual_ingress_and_reconstructs_ready() {
    let mut service = fixture_with_schema(false);
    let state = service.state().to_path_buf();
    service.run(Instant::now()+Duration::from_secs(240), |service| Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        service.start_ingress(Instant::now()+Duration::from_secs(30)).await?;
        let old = service.ready()?.clone();
        assert_eq!(service.probe_ingress(Instant::now()+Duration::from_secs(30)).await?, old.fence().epoch());
        let peer = service.connect().await?;
        let options = CanonicalOptions::from_state(service.state())?;
        let maintenance = service.maintenance().await?;
        let outcome = async {
            let request = maintenance.request_current(uuid::Uuid::new_v4()).await?;
            let pending = maintenance.begin(request).await?;
            assert!(CanonicalStore::connect_ready(&options).await.is_err(), "new opens must refuse while native epoch admission is closed");
            let closed = pending.quiesce(service.stop_ingress_and_join(Instant::now()+Duration::from_secs(30)), &[old.store().clone(),peer.store().clone()]).await?;
            let successor = closed.apply().await?;
            service.retain_connection(successor.store().clone())?;
            assert_ne!(successor.fence().epoch(),old.fence().epoch());
            assert_eq!(successor.fence().issuer(),old.fence().issuer());
            assert!(old.store().canonical_run("native-fixture-probe").await.is_err(), "the physically drained original handle cannot revive");
            successor.store().canonical_run("native-fixture-probe").await?;
            successor.store().drain_owned_connection_until(pse_operations::transport::original_deadline(Duration::from_secs(30))).await?;
            Ok::<(),pse_operations::native_service::NativeServiceError>(())
        }.await;
        let cleanup = maintenance.disconnect().await;
        outcome?; cleanup?;
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_service_process_stop_joins_without_database_transport() {
    let mut service = fixture_with_schema(false);
    service.run(Instant::now()+Duration::from_secs(180), |service| Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        let stopped = service.stop_process(Instant::now()+Duration::from_secs(30)).await?;
        assert_eq!(stopped.main_pid,0);
        assert_eq!(service.stop_process(Instant::now()+Duration::from_secs(30)).await?.invocation,stopped.invocation);
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
async fn native_service_adjacent_predecessor_migrates_only_after_original_native_join() {
    let mut service = fixture_with_schema(true);
    service.run(Instant::now()+Duration::from_secs(240), |service| Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        assert!(service.ready().is_err(), "the supported predecessor does not grant current readiness");
        let options=CanonicalOptions::from_state(service.state())?;
        assert!(CanonicalStore::connect_ready(&options).await.is_err());
        service.start_ingress(Instant::now()+Duration::from_secs(30)).await?;
        assert_eq!(service.probe_ingress(Instant::now()+Duration::from_secs(30)).await?,"maintenance");
        let maintenance=service.maintenance().await?;
        let outcome=async {
            let request=maintenance.request_current(uuid::Uuid::new_v4()).await?;
            let pending=maintenance.begin(request.clone()).await?;
            let quiescent=pending.quiesce(service.stop_ingress_and_join(Instant::now()+Duration::from_secs(30)), &[]).await?;
            let ready=quiescent.apply().await?;
            service.retain_connection(ready.store().clone())?;
            ready.store().canonical_run("native-fixture-probe").await?;
            let reconciled=maintenance.reconcile(&request).await?.expect("original immutable activation receipt must reconcile");
            assert_eq!(reconciled.fence(),ready.fence());
            service.retain_connection(reconciled.store().clone())?;
            let drains=futures_util::future::join_all([ready.store().drain_owned_connection_until(pse_operations::transport::original_deadline(Duration::from_secs(30))),reconciled.store().drain_owned_connection_until(pse_operations::transport::original_deadline(Duration::from_secs(30)))]).await;
            for drain in drains { drain?; }
            Ok::<(),pse_operations::native_service::NativeServiceError>(())
        }.await;
        let cleanup=maintenance.disconnect().await;
        outcome?; cleanup?;
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
}

#[tokio::test]
async fn native_service_listener_collision_refuses_foreign_listener_adoption() {
    let mut service=fixture_with_schema(false);
    let configuration:serde_json::Value=serde_json::from_slice(&std::fs::read(service.state().join("config.json")).unwrap()).unwrap();
    let endpoint=configuration["endpoint"].as_str().unwrap().strip_prefix("grpc://").unwrap();
    let foreign=std::net::TcpListener::bind(endpoint).unwrap();
    let result=service.run(Instant::now()+Duration::from_secs(120), |service|Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(60)).await
    })).await;
    assert!(result.is_err(),"a pre-existing socket cannot establish original-owned database readiness");
    assert!(service.ready().is_err());
    drop(foreign);
    service.dispose().unwrap();
}

#[tokio::test]
async fn native_service_body_panic_still_joins_original_owned_units() {
    use futures_util::FutureExt;
    let mut service=fixture_with_schema(false);
    let outcome=std::panic::AssertUnwindSafe(service.run(Instant::now()+Duration::from_secs(180), |service|Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        service.start_ingress(Instant::now()+Duration::from_secs(30)).await?;
        let _maintenance=service.maintenance().await?;
        panic!("intentional fixture body failure after actual native launch");
        #[allow(unreachable_code, reason="test verifies panic settlement before normal return")]
        Ok::<(),pse_operations::native_service::NativeServiceError>(())
    }))).catch_unwind().await;
    assert!(outcome.is_err());
    service.shutdown(Instant::now()+Duration::from_secs(30)).await.unwrap();
    service.dispose().unwrap();
}
