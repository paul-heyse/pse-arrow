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
        instance: pse_modeling::specialize::root_instance(root),
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
    let workspace = Workspace {
        workspace_id: pse_operations::mint_id(),
        name: "ephemeral".into(),
        root: base,
    };
    let refused = result
        .prepare_publication(
            &workspace,
            None,
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
        let workspace = runtime.register_workspace("durable", base).await.unwrap();
        let prepared = result.prepare_publication(
            &workspace,
            None,
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
    assert!(reused.starts.values().any(|s| *s
        == StartSource::Stored {
            solution: solution.as_id()
        }));
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

/// A strongly correlated 0-1 knapsack whose objective carries a constant:
/// `max Σ (wᵢ + 10)·xᵢ + 7` with `Σ wᵢ·xᵢ ≤ ½·Σ wᵢ`. It needs a branch-and-bound search.
fn knapsack() -> String {
    let weights: Vec<usize> = (0..16).map(|i| 30 + (i * 37) % 71).collect();
    let terms = |coefficient: &dyn Fn(usize) -> usize| {
        weights
            .iter()
            .enumerate()
            .map(|(i, w)| format!("{}*x{i}", coefficient(*w)))
            .collect::<Vec<_>>()
            .join(" + ")
    };
    let variables: String = (0..weights.len())
        .map(|i| format!("var x{i}: Indicator in binary; annotation start x{i}(0{{1}}); "))
        .collect();
    format!(
        "package p {{ def Root {{ {variables}\
         eq capacity: {} <= {}; \
         let total: Scalar = {} + 7; \
         annotation objective total(maximize); annotation report total(\"total\"); }} }}",
        terms(&|w| w),
        weights.iter().sum::<usize>() / 2,
        terms(&|w| w + 10),
    )
}

/// A discrete package on a durable runtime whose native jobs admit SCIP's memory limit,
/// optimized by `backend`.
async fn discrete_on(
    database: &TestDatabase,
    source: &str,
    backend: Backend,
) -> (Runtime, ModelingPackage, ModelingAnalysis) {
    let operations = Operations::connect(database.url(), "runtime-a", quick())
        .await
        .unwrap();
    let runtime = tests::runtime_with(16 << 20, 16 << 20, 2 << 30)
        .with_durability(Durability::Durable(operations));
    let physical = physical();
    let mut names = tests::discrete_names();
    names.insert(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    );
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
    solver.intent = pse_backend_native::solve::SolveIntent::Optimize;
    solver.selection = SolverSelection::Explicit(backend);
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: ModelingCaseBindings::default(),
        order: pse_kernels::DerivativeOrder::Second,
        compiler: compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    (runtime, package, analysis)
}

/// The incumbents a durable attempt stored, in order, with their captured solutions.
async fn stored_incumbents(
    database: &TestDatabase,
    attempt: pse_operations::attempts::AttemptId,
) -> Vec<(f64, bool)> {
    let session = database.session().await.unwrap();
    let rows = session
        .texts(&format!(
            "SELECT objective::text, (solution_id IS NOT NULL)::text FROM pse_ops.incumbents \
             WHERE attempt_id = '{attempt}'::uuid ORDER BY seq"
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|row| {
            let text = |i: usize| row[i].as_deref().unwrap();
            (text(0).parse().unwrap(), text(1) == "true")
        })
        .collect()
}

/// Solve the knapsack durably on `backend` and check what its incumbent stream stored:
/// objectives under the post-solve convention (the constant included, the last one the
/// result), and the captured solutions stored as seeds of the step, of the backend's
/// payload kind. Returns the stored incumbents.
async fn incumbents_stored_as_seeds(
    backend: Backend,
    kind: pse_model::generated::enums::StoredSeedKind,
) -> Vec<(f64, bool)> {
    let database = TestDatabase::create().await.unwrap();
    let (runtime, package, analysis) = discrete_on(&database, &knapsack(), backend).await;
    let Durability::Durable(operations) = runtime.durability() else {
        panic!()
    };
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let compatibility = prepared.solve.compatibility().cloned().unwrap();
    let preparation = prepared.solve.seed_preparation_identity().unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.assessments());
    let record = record(&result);
    let Ok(RunReport::Modeling(steps)) = result.report() else {
        panic!()
    };
    let crate::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!("{:?}", steps[0].outcome)
    };
    assert_eq!(native.backend, backend);
    let objective = native.candidate.as_ref().unwrap().objective.unwrap();
    let total = steps[0]
        .reports
        .iter()
        .find(|r| r.label == "total")
        .unwrap()
        .value;
    assert!((objective - total).abs() < 1e-6, "{objective} vs {total}");

    let incumbents = stored_incumbents(&database, record.attempt_id).await;
    assert!(!incumbents.is_empty());
    let (last, _) = *incumbents.last().unwrap();
    assert!(
        (last - objective).abs() < 1e-6,
        "{backend:?}: streamed {last} vs post-solve {objective}"
    );
    // The first incumbent's solution is captured at once and stored as a seed of the
    // step, keyed like its output seed.
    assert!(incumbents[0].1, "{incumbents:?}");
    let seed = operations
        .store()
        .solutions()
        .latest_in_attempt_chain(
            record.attempt_id,
            &compatibility.layout,
            &preparation,
            backend,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(seed.kind, kind);
    assert_eq!(seed.created_by, Some(record.attempt_id));
    assert_eq!(seed.profile_stamp, compatibility.profile);
    // One representation: incumbents are not also progress events.
    let progress = record.progress.as_ref().unwrap();
    assert!(!progress.iter().any(|e| e.phase.ends_with(".incumbent")));
    // The attempt's stored stream carries each incumbent with the progress context of
    // the event that reported it (Plan 22 O9).
    let mut stream = runtime
        .progress(
            record.attempt_id,
            false,
            64,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut streamed = Vec::new();
    while let Some(page) = stream.next_page().await.unwrap() {
        streamed.extend(page.into_iter().filter_map(|record| match record {
            StreamRecord::Incumbent(incumbent) => Some(incumbent),
            StreamRecord::Progress(_) => None,
        }));
    }
    assert_eq!(streamed.len(), incumbents.len());
    for incumbent in &streamed {
        assert_eq!(incumbent.step, 0);
        assert!(incumbent.phase.ends_with(".incumbent"), "{incumbent:?}");
        assert!(
            incumbent.nodes.is_some() && incumbent.seconds.is_some(),
            "{incumbent:?}"
        );
    }
    drop(runtime);
    database.remove().await.unwrap();
    incumbents
}

#[tokio::test]
async fn incumbent_stream_records_offset_objective() {
    let incumbents = incumbents_stored_as_seeds(
        Backend::Scip,
        pse_model::generated::enums::StoredSeedKind::Nlp,
    )
    .await;
    // The empty knapsack is worth the constant alone; nothing streams below it.
    assert!(
        incumbents
            .iter()
            .all(|(objective, _)| *objective >= 7.0 - 1e-9)
    );
}

#[tokio::test]
async fn highs_incumbents_stored_as_highs_seeds() {
    incumbents_stored_as_seeds(
        Backend::Highs,
        pse_model::generated::enums::StoredSeedKind::Highs,
    )
    .await;
}

/// A durable run publishes its incumbent stream as the store held it when the attempt
/// ended (Plan 22 I13): `runtime.incumbents` equals an independent reader's snapshot
/// field by field, captured points included by identity.
#[tokio::test]
async fn published_incumbents_equal_stream_snapshot() {
    use pse_relations::{columnar::RelationRow, generated::runtime::incumbents};
    let database = TestDatabase::create().await.unwrap();
    let (runtime, package, analysis) = discrete_on(&database, &knapsack(), Backend::Scip).await;
    let Durability::Durable(operations) = runtime.durability() else {
        panic!()
    };
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.assessments());
    let attempt = record(&result).attempt_id;
    let snapshot = operations
        .store()
        .streams()
        .incumbent_snapshot(attempt)
        .await
        .unwrap();
    assert!(!snapshot.is_empty());
    assert!(snapshot.iter().any(|i| i.solution_id.is_some()));
    let table = result.table("runtime.incumbents").unwrap();
    let published = incumbents::Row::rows(&table).unwrap();
    assert_eq!(published.len(), snapshot.len());
    for (row, stored) in published.iter().zip(&snapshot) {
        assert_eq!(row.run_id, result.run_id);
        assert_eq!(
            (
                row.seq,
                row.step,
                row.elapsed_seconds.to_bits(),
                row.phase.as_str(),
                row.objective.to_bits(),
                row.dual_bound.map(f64::to_bits),
                row.gap.map(f64::to_bits),
                row.nodes,
                row.seconds.map(f64::to_bits),
                row.solution_id,
            ),
            (
                stored.seq,
                i64::from(stored.step),
                stored.elapsed_seconds.to_bits(),
                stored.phase.as_str(),
                stored.objective.to_bits(),
                stored.dual_bound.map(f64::to_bits),
                stored.gap.map(f64::to_bits),
                stored.nodes,
                stored.seconds.map(f64::to_bits),
                stored.solution_id.map(|s| s.as_id()),
            )
        );
    }
    drop(runtime);
    database.remove().await.unwrap();
}
