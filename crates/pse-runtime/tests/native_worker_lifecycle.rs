// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production scientific bootstrap and original physical ownership over a native service.
#![cfg(all(feature = "native-study-tests", feature = "product-operators"))]
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "private native lifecycle assertions"
)]

#[path = "../../../tests/fixtures/native-worker-environment.rs"]
mod native_worker_environment;

use futures_util::FutureExt;
use pse_operations::native_service::{
    NativeCanonicalService, NativeModuleSource, NativeServiceSpec,
};
use pse_runtime::workflow::{
    NativeAllocation, NativeExecutionAllocation, NativeStudyWorker, NativeStudyWorkerSpec,
    WorkerSettings,
};
use std::{
    panic::AssertUnwindSafe,
    path::PathBuf,
    time::{Duration, Instant},
};

fn canonical_service() -> NativeCanonicalService {
    let environment = std::env::vars_os().collect();
    let module_source = if let Some(receipt) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
    {
        NativeModuleSource::Prepared(
            pse_host::study_module::select(
                &PathBuf::from(receipt),
                pse_host::study_module::Variant::Production,
            )
            .unwrap(),
        )
    } else {
        NativeModuleSource::RecordedState(PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE")
                .expect("explicit prepared canonical module state"),
        ))
    };
    NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment).unwrap(),
        helper: PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
                .expect("explicit already-built control ingress helper"),
        ),
        module_source,
        state_parent: std::env::temp_dir(),
    })
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn native_scientific_worker_ready_then_drains_without_canonical_connectivity() {
    let mut service = canonical_service();
    let mut worker: Option<NativeStudyWorker> = None;
    let body = AssertUnwindSafe(async {
        service
            .launch(Instant::now() + Duration::from_secs(90))
            .await
            .unwrap();
        let allocation = NativeAllocation::materialize(
            8 << 30,
            512 << 20,
            1,
            6 << 30,
            Some(NativeExecutionAllocation {
                pool_memory_bytes: 4 << 30,
                worker_bytes: 1 << 30,
                cpu_threads: 16,
                case_lanes: 16,
                math_jobs: 32,
                compiler_cores: 1,
                admission_wait_ms: 30_000,
                process_headroom_bytes: 2 << 30,
                observer_memory_bytes: 512 << 20,
            }),
        )
        .unwrap();
        let mut environment = native_worker_environment::selected();
        environment.insert("INVOCATION_ID".into(), "inherited-parent-invocation".into());
        worker = Some(
            NativeStudyWorker::new(NativeStudyWorkerSpec {
                ready: service.ready().unwrap(),
                executable: PathBuf::from(env!("CARGO_BIN_EXE_pse-worker")),
                environment,
                allocation,
                settings: WorkerSettings::default(),
            })
            .unwrap(),
        );
        let worker = worker.as_mut().unwrap();
        assert!(
            worker.observer().is_err(),
            "manager launch alone has no application readiness"
        );
        worker
            .launch(Instant::now() + Duration::from_secs(90))
            .await
            .unwrap();
        let _borrow = worker.observer().unwrap();
        assert!(
            worker.terminal().unwrap().is_none(),
            "readiness is not a native join or science receipt"
        );
        // Original shutdown must work after the canonical owner has stopped.
        service
            .shutdown(Instant::now() + Duration::from_secs(45))
            .await
            .unwrap();
        worker
            .shutdown(Instant::now() + Duration::from_secs(45))
            .await
            .unwrap();
        let joined = worker
            .terminal()
            .unwrap()
            .expect("actual original application join receipt");
        assert!(
            joined.native_joined,
            "the original scientific application joined its native lanes"
        );
    })
    .catch_unwind()
    .await;
    // Attempt both independent teardowns even after a failed assertion.
    let worker_drain = if let Some(worker) = &mut worker {
        worker
            .shutdown(Instant::now() + Duration::from_secs(45))
            .await
            .map(|_| ())
    } else {
        Ok(())
    };
    let service_drain = service
        .shutdown(Instant::now() + Duration::from_secs(45))
        .await;
    let worker_dispose = if body.is_ok() && worker_drain.is_ok() {
        worker
            .as_mut()
            .map(NativeStudyWorker::dispose)
            .transpose()
            .map(|_| ())
    } else {
        Ok(())
    };
    let service_dispose = if body.is_ok() && service_drain.is_ok() {
        service.dispose()
    } else {
        Ok(())
    };
    if let Err(panic) = body {
        eprintln!(
            "native lifecycle failed; worker drain={worker_drain:?}; service drain={service_drain:?}; canonical state={}",
            service.state().display()
        );
        if let Some(worker) = &worker {
            eprintln!(
                "original scientific worker diagnostics={}",
                worker.directory().display()
            );
        }
        // Failure details remain in the original private diagnostic directories.
        std::panic::resume_unwind(panic);
    }
    worker_drain.unwrap();
    service_drain.unwrap();
    worker_dispose.unwrap();
    service_dispose.unwrap();
}
