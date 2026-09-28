// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durability classes against isolated PostgreSQL 18 databases (Plan 22 O3–O6). Each
//! durable test creates its own migrated database on the server `DATABASE_URL` names.
use super::tests::{compiler_profile, physical, profile, runtime};
use super::*;
use pse_backend_native::solve::{Backend, ReusePolicy, SolverSelection};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_ids::SemanticId;
use pse_operations::{
    attempts::{AttemptFilter, AttemptKind, NewAttempt, TransitionNote},
    lifecycle::AttemptState,
    testing::TestDatabase,
};
use std::{collections::BTreeMap, time::Duration};

pub(super) const LINEAR: &str = "package p { def Root { param t: Scalar = 1; var x: Scalar; eq e: x == 2+t; annotation start x(2+t); annotation check x(x > 0); } }";

/// A package over `runtime` and its root analysis, solved by KINSOL.
pub(super) fn package_on(runtime: &Runtime, source: &str) -> (ModelingPackage, ModelingAnalysis) {
    let physical = physical();
    let names = BTreeMap::from([(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    )]);
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical, names).unwrap();
    let mut solver = profile();
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    solver.controls.reuse = ReusePolicy::AllowRebuild;
    let analysis = ModelingAnalysis {
        root,
        instance: root,
        bindings: Default::default(),
        limits: Default::default(),
        case: ModelingCaseBindings::default(),
        order: pse_kernels::DerivativeOrder::Second,
        compiler: compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    (package, analysis)
}

/// A lease short enough for tests to observe expiry and heartbeats.
pub(super) fn quick() -> LeasePolicy {
    LeasePolicy {
        lease: Duration::from_secs(5),
        heartbeat: Duration::from_millis(100),
        flush: Duration::from_millis(20),
        ..LeasePolicy::default()
    }
}

pub(super) async fn durable_runtime(database: &TestDatabase, worker: &str) -> Runtime {
    let operations = Operations::connect(database.url(), worker, quick())
        .await
        .unwrap();
    runtime().with_durability(Durability::Durable(operations))
}

pub(super) fn record(result: &RunResult) -> &DurableRecord {
    match result.durability() {
        RunDurability::Durable(record) => record,
        RunDurability::Ephemeral => panic!("run {} is ephemeral", result.run_id),
    }
}

fn base() -> (tempfile::TempDir, url::Url) {
    let directory = tempfile::tempdir().unwrap();
    let base = url::Url::from_directory_path(directory.path()).unwrap();
    (directory, base)
}

#[tokio::test]
async fn ephemeral_cannot_publish() {
    let runtime = runtime();
    assert!(matches!(runtime.durability(), Durability::Ephemeral));
    let (package, analysis) = package_on(&runtime, LINEAR);
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let handle = prepared.start().unwrap();
    assert_eq!(handle.attempt_id(), None);
    let result = handle.wait().await.unwrap();
    assert!(result.usable());
    assert!(matches!(result.durability(), RunDurability::Ephemeral));
    let (_directory, base) = base();
    let refused = result
        .prepare_publication(
            base,
            pse_authoring::ids::uuid_v7(),
            None,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap_err();
    assert!(
        matches!(refused, WorkflowError::EphemeralPublication { run_id } if run_id == result.run_id),
        "{refused:?}"
    );
    assert_eq!(
        refused.boundary_diagnostic().rule,
        "workflow.ephemeral_publication"
    );
}

#[tokio::test]
async fn durable_run_listed_after_restart() {
    let database = TestDatabase::create().await.unwrap();
    let (run_id, attempt_id) = {
        let runtime = durable_runtime(&database, "runtime-a").await;
        let (package, analysis) = package_on(&runtime, LINEAR);
        let cancel = crate::CancelSource::new();
        let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let handle = prepared.start().unwrap();
        let attempt_id = handle.attempt_id().unwrap();
        let result = handle.wait().await.unwrap();
        assert!(result.usable());
        assert_eq!(handle.run_id(), result.run_id);
        let record = record(&result);
        assert_eq!(record.attempt_id, attempt_id);
        let stored = record.attempt.as_ref().unwrap();
        assert_eq!(stored.state, AttemptState::Completed);
        assert_eq!(stored.kind, AttemptKind::Modeling);
        // A durable, recorded run passes the durability check of publication. (This
        // fixture's physical context lacks reference relations, so the product admission
        // after that check refuses; no write happens either way.)
        let (_directory, base) = base();
        let prepared = result.prepare_publication(
            base,
            pse_authoring::ids::uuid_v7(),
            None,
            &pse_columnar::CancellationToken::new(),
        );
        assert!(
            !matches!(
                prepared,
                Err(WorkflowError::EphemeralPublication { .. } | WorkflowError::Shared(_))
            ),
            "{prepared:?}"
        );
        (result.run_id, attempt_id)
    };
    // A new process: nothing is shared with the first but the store.
    let operations = Operations::connect(database.url(), "runtime-b", quick())
        .await
        .unwrap();
    let runs = operations.runs(&AttemptFilter::newest(10)).await.unwrap();
    let listed = runs
        .iter()
        .find(|a| a.attempt_id == attempt_id)
        .expect("the durable run is listed after a restart");
    assert_eq!(listed.run_id, run_id);
    assert_eq!(listed.state, AttemptState::Completed);
    assert_eq!(listed.worker.as_deref(), Some("runtime-a"));
    assert!(listed.finished_at.is_some());
    let termination = listed.termination.as_ref().unwrap();
    assert_eq!(termination.code, "success");
    let history = operations
        .store()
        .attempts()
        .history(attempt_id)
        .await
        .unwrap();
    let steps: Vec<(Option<AttemptState>, AttemptState)> =
        history.iter().map(|t| (t.from, t.to)).collect();
    assert_eq!(
        steps,
        [
            (None, AttemptState::Planned),
            (Some(AttemptState::Planned), AttemptState::Queued),
            (Some(AttemptState::Queued), AttemptState::Running),
            (Some(AttemptState::Running), AttemptState::Completed),
        ]
    );
    drop(operations);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn startup_sweep_marks_expired_leases_stale() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store();
    // A process that crashed while running an attempt: its lease expires unrenewed.
    let attempt = NewAttempt {
        attempt_id: pse_operations::mint_id(),
        run_id: pse_operations::mint_id(),
        kind: AttemptKind::Modeling,
        request_identity: pse_ids::ContentHash::from_bytes([1; 32]),
        preparation_identity: None,
        parent_attempt: None,
    };
    store.attempts().create(&attempt, None).await.unwrap();
    store
        .attempts()
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("crashed"),
        )
        .await
        .unwrap();
    store
        .attempts()
        .start(attempt.attempt_id, "crashed", Duration::from_millis(1))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    let operations = Operations::connect(database.url(), "restarted", quick())
        .await
        .unwrap();
    let stale = operations
        .runs(&AttemptFilter {
            run: Some(attempt.run_id),
            states: vec![AttemptState::Stale],
            limit: 10,
        })
        .await
        .unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].attempt_id, attempt.attempt_id);
    // A second start-up finds nothing more to sweep.
    assert!(operations.recover().await.unwrap().stale.is_empty());
    drop(operations);
    database.remove().await.unwrap();
}
