// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual solver calls through native plans, with independent analytic oracles.
#![cfg(feature = "ipopt")]
#![allow(
    clippy::unwrap_used,
    reason = "independent native solver test assertions"
)]
use datafusion::{
    arrow::array::{
        Array, BooleanArray, Float64Array, Int32Array, LargeListArray, RecordBatch, StringArray,
    },
    common::Column,
    execution::runtime_env::RuntimeEnv,
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_backend_native::{
    HessianPolicy, SolveOptions, Variable,
    native::{Constraint, NativeSolverPlanner, Solve},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
    planner::UnifiedPlanner,
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, Reservation};
use pse_schema::RegistryBuilder;
use std::{collections::BTreeMap, sync::Arc};

fn session(reserver: Arc<dyn MemoryReserver>) -> SnapshotSession {
    assert_eq!(
        std::env::var("IPOPT_DIR").unwrap(),
        "/opt/pse-solvers",
        "qualification must execute in the pinned solver container"
    );
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        reserver,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .with_query_planner(Arc::new(UnifiedPlanner::new(vec![Arc::new(
        NativeSolverPlanner::new(1.try_into().unwrap(), (8 << 20).try_into().unwrap()),
    )])))
    .candidate(
        BTreeMap::new(),
        Arc::new(RegistryBuilder::new().build().unwrap()),
        &CancellationToken::new(),
    )
    .unwrap()
}
fn options() -> SolveOptions {
    SolveOptions {
        hessian: HessianPolicy::LimitedMemory,
        max_iterations: 150,
        tolerance: 1e-9,
        max_wall_seconds: 10.0,
    }
}
fn variable(name: &str, lower: Option<f64>, upper: Option<f64>) -> Variable {
    Variable {
        column: Column::from_name(name),
        lower,
        upper,
        scale: 1.0,
    }
}
fn equal(expression: Expr) -> Constraint {
    Constraint {
        expression,
        lower: Some(0.0),
        upper: Some(0.0),
        scale: 1.0,
    }
}
fn input(values: &[(&str, f64)]) -> LogicalPlan {
    LogicalPlanBuilder::empty(true)
        .project(
            values
                .iter()
                .map(|(name, value)| lit(*value).alias(*name))
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .build()
        .unwrap()
}
async fn execute(
    session: &SnapshotSession,
    plan: LogicalPlan,
    cancel: &CancellationToken,
) -> RecordBatch {
    assert!(plan.display_pg_json().to_string().len() < 32768);
    let prepared = session.prepare_rule_plan(plan, cancel).unwrap();
    let done = prepared.execute(cancel).await.unwrap();
    let batches = done.into_batches();
    assert_eq!(batches.len(), 1);
    assert!(session.execution_observations().unwrap().iter().any(|o| {
        o.physical_plan()
            .is_some_and(|p| p.contains("PseSolveExec"))
    }));
    batches[0].clone()
}
fn status(batch: &RecordBatch) -> i32 {
    batch
        .column_by_name("ipopt_status")
        .unwrap()
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap()
        .value(0)
}
fn values(batch: &RecordBatch, column: &str) -> Vec<f64> {
    let list = batch
        .column_by_name(column)
        .unwrap()
        .as_any()
        .downcast_ref::<LargeListArray>()
        .unwrap()
        .value(0);
    let array = list.as_any().downcast_ref::<Float64Array>().unwrap();
    assert_eq!(array.null_count(), 0);
    array.values().to_vec()
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

#[tokio::test]
async fn nonlinear_feasibility_uses_exact_sparse_jacobian_and_returns_owned_values() {
    let budget = FixedBudget::new(128 << 20);
    let session = session(budget.clone());
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", Some(0.0), Some(3.0))],
        vec![equal(col("x") * col("x") - lit(2.0))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &CancellationToken::new()).await;
    assert_eq!(status(&batch), 0);
    close(values(&batch, "values")[0], 2.0_f64.sqrt());
    close(values(&batch, "constraints")[0], 0.0);
    drop(session);
    assert!(budget.reserved() > 0);
    close(values(&batch, "values")[0], 2.0_f64.sqrt());
    drop(batch);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn constrained_objective_and_multipliers_have_independent_optimum() {
    let session = session(FixedBudget::new(128 << 20));
    let objective = (col("x") - lit(1.0)) * (col("x") - lit(1.0))
        + (col("y") - lit(1.0)) * (col("y") - lit(1.0));
    let plan = Solve::plan(
        input(&[("x", 0.5), ("y", 2.5)]),
        vec![variable("x", None, None), variable("y", None, None)],
        vec![equal(col("x") + col("y") - lit(3.0))],
        objective,
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &CancellationToken::new()).await;
    assert_eq!(status(&batch), 0);
    for value in values(&batch, "values") {
        close(value, 1.5);
    }
    close(values(&batch, "constraint_duals")[0], -1.0);
}

#[tokio::test]
async fn nonlinear_mixer_heater_conserves_mass_and_integrated_enthalpy() {
    // cp(T) = 1 + .002 T; independent integration gives h(T)=T+.001 T^2.
    let enthalpy = |value: Expr| value.clone() + lit(0.001) * value.clone() * value;
    let session = session(FixedBudget::new(256 << 20));
    let plan = Solve::plan(
        input(&[("flow", 4.0), ("mix", 325.0), ("out", 340.0)]),
        vec![
            variable("flow", Some(0.1), None),
            variable("mix", Some(1.0), None),
            variable("out", Some(1.0), None),
        ],
        vec![
            equal(col("flow") - lit(5.0)),
            equal(
                col("flow") * enthalpy(col("mix"))
                    - lit(2.0 * (300.0 + 90.0) + 3.0 * (350.0 + 122.5)),
            ),
            equal(col("flow") * (enthalpy(col("out")) - enthalpy(col("mix"))) - lit(100.0)),
        ],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &CancellationToken::new()).await;
    assert_eq!(status(&batch), 0);
    let actual = values(&batch, "values");
    let mixed_enthalpy: f64 = (2.0 * 390.0 + 3.0 * 472.5) / 5.0;
    let inverse = |h: f64| ((1.0 + 0.004 * h).sqrt() - 1.0) / 0.002;
    close(actual[0], 5.0);
    close(actual[1], inverse(mixed_enthalpy));
    close(actual[2], inverse(mixed_enthalpy + 20.0));
    for residual in values(&batch, "constraints") {
        close(residual, 0.0);
    }
}

#[tokio::test]
async fn infeasible_and_invalid_evaluation_are_results_with_actual_status() {
    let session = session(FixedBudget::new(128 << 20));
    let plan = Solve::plan(
        input(&[("x", 0.5)]),
        vec![variable("x", Some(0.0), Some(1.0))],
        vec![equal(col("x") - lit(2.0))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &CancellationToken::new()).await;
    assert_eq!(status(&batch), 2);
    assert!(values(&batch, "constraints")[0].abs() > 0.9);
    let plan = Solve::plan(
        input(&[("x", -1.0)]),
        vec![variable("x", None, Some(0.0))],
        vec![equal(datafusion::functions::math::expr_fn::ln(col("x")))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &CancellationToken::new()).await;
    assert_eq!(status(&batch), -13);
    let code = batch
        .column_by_name("diagnostic_code")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(code.value(0), "solve::evaluation_error");
}

#[derive(Debug)]
struct CancelDuringEvaluation {
    budget: Arc<FixedBudget>,
    token: CancellationToken,
}
impl MemoryReserver for CancelDuringEvaluation {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        if owner == "numerics:native-expression-evaluation" {
            self.token.cancel();
        }
        self.budget.open(owner)
    }
}
#[tokio::test]
async fn callback_cancellation_settles_through_common_stream_with_last_values() {
    let cancel = CancellationToken::new();
    let session = session(Arc::new(CancelDuringEvaluation {
        budget: FixedBudget::new(128 << 20),
        token: cancel.clone(),
    }));
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", None, None)],
        vec![equal(col("x") * col("x") - lit(2.0))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &cancel).await;
    assert!(cancel.is_cancelled());
    assert_ne!(status(&batch), 0);
    assert!(
        batch
            .column_by_name("cancelled")
            .unwrap()
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap()
            .value(0)
    );
    assert_eq!(values(&batch, "values").len(), 1);
}

#[tokio::test]
async fn inspection_policy_refuses_solver_variation_before_callbacks() {
    let session = session(FixedBudget::new(128 << 20))
        .with_purpose(pse_schema::model::provider::OperationPurpose::Inspect);
    let cancel = CancellationToken::new();
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", None, None)],
        vec![equal(col("x"))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let error = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("nondeterministic"), "{error}");
}

#[tokio::test]
async fn panic_inside_an_actual_native_expression_is_contained_at_the_c_boundary() {
    use datafusion::{
        arrow::{
            array::Float64Array,
            datatypes::{DataType, Field, Schema},
        },
        common::TableReference,
        datasource::{MemTable, provider_as_source},
        logical_expr::{Volatility, create_udf},
    };
    let cancel = CancellationToken::new();
    let schema = Arc::new(Schema::new(vec![
        Field::new("x", DataType::Float64, false),
        Field::new("parameter", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Float64Array::from(vec![1.0])),
            Arc::new(Float64Array::from(vec![2.0])),
        ],
    )
    .unwrap();
    let provider = Arc::new(MemTable::try_new(schema, vec![vec![batch]]).unwrap());
    let reference = TableReference::full("model", "workspace", "panic_input");
    let session = session(FixedBudget::new(128 << 20))
        .with_provider(reference.clone(), provider.clone(), &cancel)
        .unwrap();
    let child = LogicalPlanBuilder::scan(reference, provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let function = create_udf(
        "panic_parameter",
        vec![DataType::Float64],
        DataType::Float64,
        Volatility::Immutable,
        Arc::new(|_| panic!("injected callback panic")),
    );
    let plan = Solve::plan(
        child,
        vec![variable("x", None, None)],
        vec![equal(col("x") + function.call(vec![col("parameter")]))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let batch = execute(&session, plan, &cancel).await;
    assert_eq!(status(&batch), -13);
    let diagnostic = batch
        .column_by_name("diagnostic")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert!(diagnostic.value(0).contains("callback panicked"));
}

#[derive(Debug)]
struct BlockingEvaluation {
    budget: Arc<FixedBudget>,
    entered: Arc<tokio::sync::Notify>,
    gate: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
}
impl MemoryReserver for BlockingEvaluation {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        if owner == "numerics:native-expression-evaluation" {
            self.entered.notify_one();
            let (mutex, condvar) = self.gate.as_ref();
            let mut released = mutex.lock().unwrap();
            while !*released {
                released = condvar.wait(released).unwrap();
            }
        }
        self.budget.open(owner)
    }
}
#[tokio::test]
async fn dropped_native_stream_retains_callback_owners_until_foreign_work_returns() {
    let cancel = CancellationToken::new();
    let budget = FixedBudget::new(128 << 20);
    let entered = Arc::new(tokio::sync::Notify::new());
    let gate = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let session = session(Arc::new(BlockingEvaluation {
        budget: budget.clone(),
        entered: entered.clone(),
        gate: gate.clone(),
    }));
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", None, None)],
        vec![equal(col("x") * col("x") - lit(2.0))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let mut stream = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute_stream(&cancel)
        .await
        .unwrap();
    tokio::select! {
        () = entered.notified() => {},
        outcome = stream.next_batch(&cancel) => panic!("solver returned before the callback gate: {outcome:?}"),
    }
    drop(stream);
    drop(session);
    assert!(
        !cancel.is_cancelled(),
        "dropping one stream must not cancel its parent"
    );
    assert!(
        budget.reserved() >= 8 << 20,
        "foreign workspace must stay owned while C is running"
    );
    *gate.0.lock().unwrap() = true;
    gate.1.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while budget.reserved() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn foreign_allocation_allowance_is_refused_by_the_actual_session_budget() {
    let session = session(FixedBudget::new(2 << 20));
    let cancel = CancellationToken::new();
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", None, None)],
        vec![equal(col("x"))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let error = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("solve:ipopt-workspace-and-foreign-allowance"),
        "{error}"
    );
}

#[tokio::test]
async fn pull_cancellation_signals_the_running_invocation_without_cancelling_its_parent() {
    let prepare_cancel = CancellationToken::new();
    let execute_cancel = CancellationToken::new();
    let pull_cancel = CancellationToken::new();
    let entered = Arc::new(tokio::sync::Notify::new());
    let gate = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let session = session(Arc::new(BlockingEvaluation {
        budget: FixedBudget::new(128 << 20),
        entered: entered.clone(),
        gate: gate.clone(),
    }));
    let plan = Solve::plan(
        input(&[("x", 1.0)]),
        vec![variable("x", None, None)],
        vec![equal(col("x") * col("x") - lit(2.0))],
        lit(0.0),
        options(),
    )
    .unwrap();
    let mut stream = session
        .prepare_rule_plan(plan, &prepare_cancel)
        .unwrap()
        .execute_stream(&execute_cancel)
        .await
        .unwrap();
    tokio::select! {
        () = entered.notified() => {},
        result = stream.next_batch(&pull_cancel) => panic!("unexpected early result: {result:?}"),
    }
    pull_cancel.cancel();
    let mut next = Box::pin(stream.next_batch(&pull_cancel));
    // Poll the cancellation branch while the callback remains inside its gate.
    assert!(futures_util::poll!(&mut next).is_pending());
    *gate.0.lock().unwrap() = true;
    gate.1.notify_one();
    let batch = next.await.unwrap().unwrap();
    assert!(
        batch
            .column_by_name("cancelled")
            .unwrap()
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap()
            .value(0)
    );
    assert!(stream.next_batch(&pull_cancel).await.unwrap().is_none());
    assert!(!prepare_cancel.is_cancelled());
    assert!(!execute_cancel.is_cancelled());
}
