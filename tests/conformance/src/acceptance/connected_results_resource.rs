// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Termination};
use pse_runtime::{CancelSource, math::solves::Outcome, workflow::RunReport};
#[tokio::test]
async fn authored_connected_results_resource() {
    let owner = WorkflowRuntime::new().unwrap();
    let rt = runtime(&owner);
    let package = seed_package_on(&owner, rt.clone()).await;
    let case = pse_ids::SemanticId::parse_hex("68ba8dc2d6b05d9a9fe1b1a3625d8015").unwrap();
    let cancelled = CancelSource::new();
    cancelled.cancel();
    assert!(
        seed_prepare(&package, case, profile(Backend::Ipopt, false), &cancelled)
            .await
            .is_err()
    );
    let prepared = seed_prepare(
        &package,
        case,
        profile(Backend::Ipopt, false),
        &CancelSource::new(),
    )
    .await
    .unwrap();
    let a = prepared.start().unwrap();
    let b = prepared.start().unwrap();
    let (a, b) = tokio::join!(a.wait(), b.wait());
    let a = a.unwrap();
    let b = b.unwrap();
    authored_success(&a);
    authored_success(&b);
    assert_ne!(a.run_id, b.run_id);
    let run = a.canonical_run_key().unwrap();
    let attempt = a.canonical_attempt_key().unwrap();
    let mut reader = rt
        .results(
            &run,
            &attempt,
            "runtime.solve_variables",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let original = a.table("runtime.solve_variables").unwrap();
    let mut count = 0;
    while let Some(batch) = reader.next_batch().await.unwrap() {
        count += batch.num_rows();
    }
    assert_eq!(count, original.batch().num_rows());
    let cancelled = pse_columnar::CancellationToken::new();
    cancelled.cancel();
    assert!(
        rt.results(
            &run,
            &attempt,
            "runtime.solve_variables",
            0,
            u64::MAX,
            cancelled
        )
        .await
        .is_err()
    );
    assert!(
        rt.results(
            "run:absent",
            &attempt,
            "runtime.solve_variables",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new()
        )
        .await
        .is_err()
    );
    drop((reader, original));
    let retained = a.table("runtime.solve_variables").unwrap();
    let arrays = retained.batch().columns().to_vec();
    let reserved = owner.runtime.pool().reserved();
    drop(retained);
    drop(a);
    drop(b);
    drop(prepared);

    assert!(owner.runtime.pool().reserved() > 0);
    assert!(!arrays.is_empty());
    drop(arrays);
    assert!(owner.runtime.pool().reserved() <= reserved);
    // A limited or cancelled attempt remains an attempt, never an optimum certificate.
    let mut limited = profile(Backend::Ipopt, false);
    limited.controls.iterations = 1;
    let mut p = seed_prepare(&package, case, limited, &CancelSource::new())
        .await
        .unwrap();
    // Exercise a limited original native attempt. Auto conditional blocks may instead
    // stop before complete original coordinates exist, which has a different failure API.
    let original = p.solve.clone();
    let mut direct = original.numerical_strategy();
    direct.mechanisms[0].profile = Some(pse_model::strategy::ProfileRef {
        backend: Backend::Ipopt,
        key: original.strategy_profile().unwrap(),
    });
    p.solve = original
        .clone()
        .with_strategy(direct, vec![original.into()])
        .unwrap();
    let result = p.start().unwrap().wait().await.unwrap();
    let RunReport::Modeling(report) = result.report().unwrap() else {
        panic!()
    };
    let report = &report[0];
    assert!(
        matches!(&report.outcome,Outcome::Native(r) if r.termination.category != Termination::Success)
    );
    assert_eq!(
        result
            .table("runtime.solve_runs")
            .unwrap()
            .batch()
            .num_rows(),
        1
    );
    let p = seed_prepare(
        &package,
        case,
        profile(Backend::Ipopt, false),
        &CancelSource::new(),
    )
    .await
    .unwrap();
    let handle = p.start().unwrap();
    handle.cancel();
    let result = handle.wait().await.unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &result,
        &handle.wait().await.unwrap()
    ));
    assert!(result.table("runtime.solve_runs").is_ok());
    drop((result, handle, p, package, rt));
}
