// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Termination};
use pse_runtime::{CancelSource, math::solves::Outcome, workflow::RunReport};
#[tokio::test]
async fn authored_publication_resource() {
    let owner = WorkflowRuntime::new().unwrap();
    // Only durable runs publish (ADR-0112 Outcome 16): the runs below are attempts in an
    // isolated operational store, and their publications are in its catalog.
    let database = pse_operations::testing::TestDatabase::create()
        .await
        .unwrap();
    let rt = durable(&owner, database.url()).await;
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
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    let workspace = rt.register_workspace("plan14", base).await.unwrap();
    let publication = a
        .prepare_publication(&workspace, None, None, &owner.cancel)
        .unwrap();
    let published = publication.commit(&owner.cancel).await.unwrap();
    let reopened = rt
        .open(published.publication_id, &owner.cancel)
        .await
        .unwrap();
    assert_eq!(reopened.publication_id(), published.publication_id);
    assert_eq!(
        rt.head(workspace.workspace_id).await.unwrap(),
        Some(published.publication_id)
    );
    let name = datafusion::common::ResolvedTableReference {
        catalog: "artifact".into(),
        schema: "authored".into(),
        table: "modeling_declarations".into(),
    };
    assert!(reopened.publication().member(&name).is_ok());
    // A publication the catalog does not hold is refused.
    assert!(
        rt.open(pse_operations::mint_id(), &owner.cancel)
            .await
            .is_err()
    );
    let cancelled = pse_columnar::CancellationToken::new();
    cancelled.cancel();
    let interrupted = b
        .prepare_publication(
            &workspace,
            Some(published.publication_id),
            None,
            &owner.cancel,
        )
        .unwrap();
    assert!(interrupted.commit(&cancelled).await.is_err());
    drop(reopened);
    assert!(
        rt.open(published.publication_id, &owner.cancel)
            .await
            .is_ok()
    );
    assert_eq!(
        rt.head(workspace.workspace_id).await.unwrap(),
        Some(published.publication_id)
    );
    // Cancel only after an actual native object write; the catalog commit never happens.
    let faults = pse_testkit::fault_store::FaultStore::new(std::sync::Arc::new(
        object_store::memory::InMemory::new(),
    ));
    let fault_base = url::Url::parse("memory://plan14-interrupted/").unwrap();
    owner
        .runtime
        .runtime_env()
        .object_store_registry
        .register_store(&fault_base, faults.clone());
    let fault_workspace = rt
        .register_workspace("plan14-interrupted", fault_base)
        .await
        .unwrap();
    let interrupted = pse_columnar::CancellationToken::new();
    faults.arm(pse_testkit::fault_store::FaultPlan {
        operation: "put",
        prefix: String::new(),
        call: 1,
        fault: pse_testkit::fault_store::Fault::Cancel(interrupted.clone()),
    });
    let command = a
        .prepare_publication(&fault_workspace, None, None, &owner.cancel)
        .unwrap();
    assert!(command.commit(&interrupted).await.is_err());
    assert_eq!(faults.fired(), 1);
    assert_eq!(rt.head(fault_workspace.workspace_id).await.unwrap(), None);
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
    database.remove().await.unwrap();
}
