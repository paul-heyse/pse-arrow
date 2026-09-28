// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durability classes against isolated PostgreSQL 18 databases (Plan 22 O3–O6). Each
//! durable test creates its own database with the generated schema on the server
//! `DATABASE_URL` names.
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
    assert_eq!(listed.run_id, run_id.into());
    assert_eq!(listed.state, AttemptState::Completed);
    assert_eq!(listed.worker.as_deref(), Some("runtime-a"));
    assert!(listed.finished_at.is_some());
    assert_eq!(
        pse_operations::attempts::TerminationCode::of(listed).unwrap(),
        Some(pse_operations::attempts::TerminationCode::Native(
            pse_operations::attempts::NativeTermination::Success
        ))
    );
    let history = operations
        .store()
        .attempts()
        .history(attempt_id)
        .await
        .unwrap();
    let steps: Vec<(Option<AttemptState>, AttemptState)> =
        history.iter().map(|t| (t.from_state, t.to_state)).collect();
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

/// Rosenbrock from the classic start: Ipopt reports one progress event per iteration.
const ROSENBROCK: &str = "package p { def Root { var x: Scalar; var y: Scalar; let f: Scalar = (1-x)*(1-x) + 100*(y-x*x)*(y-x*x); annotation objective f(minimize); annotation start x(-1.2); annotation start y(1); } }";

fn optimize(analysis: &mut ModelingAnalysis, history: usize) {
    analysis.solver.intent = pse_backend_native::solve::SolveIntent::Optimize;
    analysis.solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    analysis.solver.controls.history = history;
}

#[tokio::test]
async fn progress_stream_complete_under_volume() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable_runtime(&database, "runtime-a").await;
    let (package, mut analysis) = package_on(&runtime, ROSENBROCK);
    // A tiny in-memory cap: the durable stream does not share it.
    optimize(&mut analysis, 4);
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let steps = vec![prepared; 24];
    let handle = runtime.start_modeling(steps, true, &cancel).await.unwrap();
    let result = handle.wait().await.unwrap();
    assert!(result.usable());
    let (retained, dropped) = handle.progress();
    let observed = retained.len() as u64 + dropped;
    assert!(
        observed > 256 && dropped > 0,
        "{} retained, {dropped} dropped",
        retained.len()
    );
    let stream = record(&result).progress.as_ref().unwrap();
    // Every event reached the store, numbered without gaps, and every step is present.
    assert_eq!(stream.len() as u64, observed);
    assert!(stream.iter().enumerate().all(|(i, e)| e.seq == i as i64));
    let steps: std::collections::BTreeSet<i32> = stream.iter().map(|e| e.step).collect();
    assert_eq!(steps, (0..24).collect());
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn published_metrics_equal_stream_snapshot() {
    use pse_operations::streams::ProgressValue as V;
    use pse_relations::generated::runtime::solve_metrics;
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable_runtime(&database, "runtime-a").await;
    let (package, mut analysis) = package_on(&runtime, ROSENBROCK);
    optimize(&mut analysis, 2);
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    assert!(result.usable());
    let attempt = record(&result).attempt_id;
    // The snapshot, read back from the store by an independent reader.
    let Durability::Durable(operations) = runtime.durability() else {
        panic!()
    };
    let snapshot = operations
        .store()
        .streams()
        .snapshot(attempt)
        .await
        .unwrap();
    assert!(snapshot.len() > 2, "{} events", snapshot.len());
    let table = result.table("runtime.solve_metrics").unwrap();
    let rows = solve_metrics::RuntimeSolveMetricsView::from_checked(&table)
        .unwrap()
        .rows()
        .unwrap();
    let mut published: Vec<_> = rows
        .iter()
        .filter(|r| r.namespace.starts_with("event."))
        .map(|r| {
            (
                r.namespace.clone(),
                r.name.clone(),
                r.kind,
                r.real.map(f64::to_bits),
                r.integer,
                r.boolean,
                r.text.clone(),
                r.unavailable,
            )
        })
        .collect();
    let mut expected = Vec::new();
    for event in &snapshot {
        let namespace = format!("event.{}.{}", event.seq, event.phase);
        let mut values = vec![("elapsed_seconds".to_owned(), V::real(event.elapsed_seconds))];
        values.extend(event.values.iter().map(|(k, v)| (k.clone(), v.clone())));
        for (name, value) in values {
            let (real, integer, boolean, text, unavailable) = match &value {
                V::Real(v) => (Some(v.to_bits()), None, None, None, None),
                V::Integer(v) => (None, Some(*v), None, None, None),
                V::Boolean(v) => (None, None, Some(*v), None, None),
                V::Text(v) => (None, None, None, Some(v.clone()), None),
                V::Unavailable(r) => (None, None, None, None, Some(*r)),
            };
            expected.push((
                namespace.clone(),
                name,
                value.kind(),
                real,
                integer,
                boolean,
                text,
                unavailable,
            ));
        }
    }
    published.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    expected.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    assert_eq!(published, expected);
    let dropped = rows
        .iter()
        .find(|r| r.namespace == "progress" && r.name == "dropped_events")
        .unwrap();
    assert_eq!(dropped.integer, Some(0));
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn incompatible_seed_refused() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable_runtime(&database, "runtime-a").await;
    let Durability::Durable(operations) = runtime.durability() else {
        panic!()
    };
    let cancel = crate::CancelSource::new();
    let (package, mut analysis) = package_on(&runtime, ROSENBROCK);
    optimize(&mut analysis, 8);
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let [(0, solution)] = record(&result).solutions[..] else {
        panic!("{:?}", record(&result).solutions)
    };
    // The same preparation reuses its stored seed; its start sources name the solution.
    let reused = prepared
        .clone()
        .with_stored_start(operations, StoredStart::Latest)
        .await
        .unwrap();
    assert!(
        reused
            .starts
            .values()
            .any(|s| *s == StartSource::Stored { solution: solution.as_id() })
    );
    // Different coordinates: the explicit stored seed is refused, and none is found.
    let (other, mut different) = package_on(&runtime, LINEAR);
    optimize(&mut different, 8);
    different.solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
    let elsewhere = other.prepare_analysis(&different, &cancel).await.unwrap();
    let refused = elsewhere
        .clone()
        .with_stored_start(operations, StoredStart::Solution(solution))
        .await
        .unwrap_err();
    // The one compatibility rule, `WarmStart::validate`, refuses it with its typed contract.
    assert!(
        matches!(
            &refused,
            WorkflowError::Math(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Contract(reason)
            )) if reason.contains("incompatible warm-start layout/backend")
        ),
        "{refused:?}"
    );
    assert!(
        elsewhere
            .with_stored_start(operations, StoredStart::Latest)
            .await
            .is_err()
    );
    drop(runtime);
    database.remove().await.unwrap();
}
