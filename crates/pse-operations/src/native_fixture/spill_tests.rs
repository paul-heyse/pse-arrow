// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native SurrealQL sort spill and original scratch lifetime controls.
use crate::native_service::{NativeCanonicalService, NativeQueryMemory, NativeServiceError};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use surrealdb_types::{Object, SurrealValue, Value};

fn has_runs(root: &Path) -> std::io::Result<bool> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if has_runs(&entry.path())? {
                return Ok(true);
            }
        } else if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("run-"))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn fixture() -> NativeCanonicalService {
    let mut spec = super::owned_test_service_spec();
    // A small query-accounting selection forces this disposable operator to spill.
    // It does not restrict the host or the process's physical allocation.
    spec.query_memory = Some(
        NativeQueryMemory::bytes(
            Some(2 * 1024 * 1024),
            Some(256 * 1024),
            Some(128 * 1024 * 1024),
        )
        .unwrap(),
    );
    NativeCanonicalService::new(spec).unwrap()
}
async fn populate(service: &NativeCanonicalService) -> Result<(), NativeServiceError> {
    service
        .fixture_query("DEFINE TABLE spill_probe SCHEMALESS;", Object::new())
        .await?;
    for batch in 0..50 {
        let rows = (batch * 100..(batch + 1) * 100)
            .rev()
            .map(|generation| {
                let mut row = Object::new();
                row.insert("generation", (generation as i64).into_value());
                row.insert(
                    "payload",
                    format!("{generation:08}{}", "x".repeat(8192)).into_value(),
                );
                Value::Object(row)
            })
            .collect::<Vec<_>>();
        let mut bindings = Object::new();
        bindings.insert("rows", rows.into_value());
        service
            .fixture_query("INSERT INTO spill_probe $rows;", bindings)
            .await?;
    }
    Ok(())
}
#[tokio::test]
async fn native_sort_spills_exact_results_then_cancellation_and_shutdown_release_owned_scratch() {
    let mut service = fixture();
    let state = service.state().to_path_buf();
    let work_end = Instant::now() + Duration::from_secs(240);
    let final_end = work_end + Duration::from_secs(60);
    service.run_until(work_end,final_end,|service|Box::pin(async move {
        service.launch(work_end).await?;
        populate(service).await?;
        let scratch=service.state().join("tmp");
        let mut saw_spill=false;
        let mut result={
            // The outer projection keeps the diagnostic response within the ordinary
            // transport envelope. Its bounded subquery array remains nonspillable.
            let query=service.fixture_query("SELECT VALUE generation FROM (SELECT generation,payload FROM spill_probe ORDER BY payload);",Object::new());
            tokio::pin!(query);
            loop {
                tokio::select! {
                    result=&mut query=>break result?,
                    ()=tokio::time::sleep(Duration::from_millis(1))=> {saw_spill|=has_runs(&scratch)?;}
                }
            }
        };
        if !saw_spill {return Err(NativeServiceError::Refused("native sort did not exercise physical spill"));}
        let rows:Vec<i64>=result.take(0).map_err(crate::canonical::CanonicalError::from)?;
        if rows.len()!=5000 || rows.iter().copied().ne(0..5000) {
            return Err(NativeServiceError::Refused("native spilled sort differs from independent integer order"));
        }
        drop(rows);
        if has_runs(&scratch)? {return Err(NativeServiceError::Refused("completed native sort retained a run"));}
        // Drop the actual RPC only after it has produced a physical spill run.
        // The connection keeps cancellation/drain authority; this sample grants none.
        {
            let query=service.fixture_query("SELECT VALUE generation FROM (SELECT generation,payload FROM spill_probe ORDER BY payload DESC);",Object::new());
            tokio::pin!(query);
            loop {
                tokio::select! {
                    result=&mut query=> {result?;return Err(NativeServiceError::Refused("native cancellation did not intersect physical spill"));}
                    ()=tokio::time::sleep(Duration::from_millis(1))=> {if has_runs(&scratch)? {break;}}
                }
            }
        }
        // Wait for the original engine's cleanup, never infer it from a zero gauge.
        while has_runs(&scratch)? {
            if Instant::now()>=work_end {return Err(NativeServiceError::Refused("cancelled native spill retained scratch"));}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        service.fixture_query("RETURN 17;",Object::new()).await?;
        // The selected store keeps an actual session directory. Removing this
        // disposable session forces the next native run allocation to fail;
        // the engine must return failure rather than quietly disable spilling.
        let mut sessions=0;
        for entry in std::fs::read_dir(&scratch)? {
            let entry=entry?;
            if entry.file_type()?.is_dir() && entry.file_name().to_str().is_some_and(|name|name.starts_with("surreal-spill-")) {
                std::fs::remove_dir_all(entry.path())?;
                sessions+=1;
            }
        }
        if sessions!=1 {return Err(NativeServiceError::Refused("native spill store did not retain one owned session"));}
        match service.fixture_query("SELECT VALUE generation FROM (SELECT generation,payload FROM spill_probe ORDER BY payload);",Object::new()).await {
            Err(NativeServiceError::Canonical(crate::canonical::CanonicalError::Driver(error))) if error.is_internal()=>{},
            Err(error)=>return Err(error),
            Ok(_)=>return Err(NativeServiceError::Refused("failed native spill allocation became a successful query")),
        }
        service.fixture_query("RETURN 17;",Object::new()).await?;
        if has_runs(&scratch)? {return Err(NativeServiceError::Refused("failed native spill retained a run"));}
        Ok(())
    })).await.unwrap();
    // Service shutdown and final connection settlement precede state disposal.
    service.dispose().unwrap();
    assert!(!state.exists());
}

#[tokio::test]
async fn native_explicit_invalid_scratch_refuses_supplier_readiness() {
    use pse_host::{HostManager, LaunchIntent, ResourceControls, ServiceSpec};
    let work_end = Instant::now() + Duration::from_secs(60);
    let final_end = work_end + Duration::from_secs(60);
    let directory = tempfile::tempdir().unwrap();
    let invalid = directory.path().join("not-a-directory");
    std::fs::write(&invalid, b"owned refusal control").unwrap();
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let bind = listener.local_addr().unwrap();
    drop(listener);
    let server = super::owned_test_service_spec().server;
    let manager = HostManager::connect_user(None).await.unwrap();
    let enclosing = manager.current_unit().await.unwrap();
    let intent = LaunchIntent::fresh().unwrap();
    let effect = pse_host::fixtures::target_settlement::TargetEffect::begin().unwrap();
    let spec = ServiceSpec {
        intent: intent.clone(),
        executable: server.executable(),
        arguments: vec![
            "start".into(),
            "--no-banner".into(),
            "--no-defaults".into(),
            "--bind".into(),
            bind.to_string(),
            "--temporary-directory".into(),
            invalid.to_string_lossy().into_owned(),
            format!(
                "mem://{}?aol=sync&sync=every&versioned=false&snapshot=60s",
                directory.path().join("database").display()
            ),
        ],
        environment: vec!["PATH=/usr/bin:/bin".into(), "LANG=C.UTF-8".into()],
    };
    let start = manager
        .start_service_bound(
            &spec,
            &manager.borrow(enclosing.unit),
            &ResourceControls::default(),
            work_end,
        )
        .await;
    let mut unit = match start {
        Ok(unit) => unit,
        Err(error) => {
            if let Some(mut original) = manager
                .reconcile_if_present(&intent, final_end)
                .await
                .unwrap()
            {
                original.stop_and_drain(final_end).await.unwrap();
            }
            if let Some(effect) = effect {
                effect.settle().unwrap();
            }
            panic!("scratch refusal launch itself failed: {error}");
        }
    };
    let observed = unit.wait(Some(work_end)).await;
    let settled = unit.stop_and_drain(final_end).await;
    if settled.is_ok() {
        if let Some(effect) = effect {
            effect.settle().unwrap();
        }
    }
    settled.unwrap();
    let observed = observed
        .unwrap()
        .expect("explicit invalid scratch must terminate startup");
    assert!(
        observed
            .completed_status()
            .is_some_and(|status| status != 0)
    );
    assert!(invalid.is_file());
}
