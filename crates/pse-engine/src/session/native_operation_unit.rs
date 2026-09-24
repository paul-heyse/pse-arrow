// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fake native operations: no compiler, durable catalog, or solver journey.
use super::{EngineFactory, execution::NativeExecutionContext};
use crate::operation::{self, Body, Definition, Execution, Family, Operation};
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    common::Result,
    execution::{
        TaskContext,
        runtime_env::RuntimeEnv,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, UserDefinedLogicalNodeCore},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, collect},
};
use futures_util::{FutureExt, StreamExt};
use pse_columnar::CancellationToken;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn state() -> SessionState {
    let cancel = CancellationToken::new();
    let session = EngineFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
        "operation-unit",
        SessionStateBuilder::new_with_default_features(),
    )
    .candidate_checked(
        BTreeMap::new(),
        Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
        &cancel,
    )
    .unwrap()
    .with_purpose(pse_schema::model::provider::OperationPurpose::Mutate);
    let state =
        NativeExecutionContext::bind(&session, session.bound_state().unwrap(), &cancel).unwrap();
    NativeExecutionContext::execution_state(&state, &cancel).unwrap()
}
fn schema() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, false)]))
}
#[derive(Debug)]
struct Fake {
    calls: Arc<AtomicUsize>,
    rows: Vec<i64>,
    family: Family,
}
#[async_trait::async_trait]
impl Definition for Fake {
    fn name(&self) -> &'static str {
        "FakeNative"
    }
    fn schema(&self) -> SchemaRef {
        schema()
    }
    fn family(&self) -> Family {
        self.family
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        _: &[LogicalPlan],
        _: &[Arc<dyn ExecutionPlan>],
        _: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        Ok(self)
    }
}
impl Body for Fake {
    fn execute(
        &self,
        _: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let rows = self.rows.clone();
        Ok(operation::batch(schema(), async move {
            Ok(RecordBatch::try_new(
                schema(),
                vec![Arc::new(Int64Array::from(rows))],
            )?)
        }))
    }
}
fn fake(rows: &[i64], family: Family) -> (LogicalPlan, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    (
        Operation::plan(
            Arc::new(Fake {
                calls: calls.clone(),
                rows: rows.to_vec(),
                family,
            }),
            vec![],
        )
        .unwrap(),
        calls,
    )
}
async fn physical(plan: LogicalPlan, state: &SessionState) -> Arc<dyn ExecutionPlan> {
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20));
    let mut optimize = |input, _required| state.optimize(&input);
    let staged =
        super::cache::logical::stage(plan, &budget, &CancellationToken::new(), &mut optimize)
            .unwrap();
    let optimized = state.optimize(&staged).unwrap();
    let expanded = super::cache::logical::expand(
        optimized,
        &{
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20));
            pool
        },
        &CancellationToken::new(),
    )
    .unwrap();
    state.create_physical_plan(&expanded).await.unwrap()
}
#[tokio::test]
async fn requirement_survives_zero_limit_and_no_value_poll() {
    let state = state();
    let (values, value_calls) = fake(&[7], Family::Finite);
    let (required, check_calls) = fake(&[99], Family::Finite);
    let plan = super::contract::ExecutionContract::plan(
        values,
        Some(required),
        std::collections::BTreeSet::default(),
    );
    let plan = LogicalPlanBuilder::from(plan)
        .limit(0, Some(0))
        .unwrap()
        .build()
        .unwrap();
    let plan = physical(plan, &state).await;
    assert_eq!(check_calls.load(Ordering::SeqCst), 0);
    let result = collect(plan, state.task_ctx()).await;
    assert!(
        result.is_err(),
        "zero output demand cannot suppress a requirement"
    );
    assert_eq!(check_calls.load(Ordering::SeqCst), 1);
    assert_eq!(value_calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn cloned_command_executes_once_even_when_value_is_unused() {
    let state = state();
    let (command, calls) = fake(&[7], Family::Command);
    let plan = LogicalPlanBuilder::from(command.clone())
        .union(command)
        .unwrap()
        .limit(0, Some(0))
        .unwrap()
        .build()
        .unwrap();
    let plan = physical(plan, &state).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    collect(plan, state.task_ctx()).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn pure_completion_failure_and_abandonment_are_terminal() {
    let completion = Arc::new(operation::completion::Completion::<usize>::default());
    let calls = Arc::new(AtomicUsize::new(0));
    let mut first = Box::pin(completion.get(|| {
        calls.fetch_add(1, Ordering::SeqCst);
        futures_util::future::pending().boxed()
    }));
    assert!(futures_util::poll!(&mut first).is_pending());
    drop(first);
    let result = completion
        .get(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            async { Ok(7) }.boxed()
        })
        .await;
    assert!(result.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let success = Arc::new(operation::completion::Completion::default());
    let (a, b) = tokio::join!(
        success.get(|| async { Ok(9) }.boxed()),
        success.get(|| async { Ok(100) }.boxed())
    );
    assert_eq!(*a.unwrap(), 9);
    assert_eq!(*b.unwrap(), 9);
}
#[test]
fn rewrite_checks_arity_and_actual_identity() {
    let (plan, _) = fake(&[1], Family::Finite);
    let LogicalPlan::Extension(ext) = plan else {
        panic!("extension")
    };
    let node = ext.node.as_any().downcast_ref::<Operation>().unwrap();
    assert!(
        node.with_exprs_and_inputs(vec![datafusion::logical_expr::lit(1)], vec![])
            .is_err()
    );
    assert_eq!(node, &node.with_exprs_and_inputs(vec![], vec![]).unwrap());
    let (other, _) = fake(&[1], Family::Finite);
    assert_ne!(LogicalPlan::Extension(ext), other);
}
#[tokio::test]
async fn metrics_are_native_and_unpolled_work_does_not_start() {
    let state = state();
    let services = NativeExecutionContext::from_session(&state).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let plan = Execution::plan(
        "Fake",
        schema(),
        vec![],
        Arc::new(Fake {
            calls: calls.clone(),
            rows: vec![1, 2],
            family: Family::Finite,
        }),
        Family::Finite,
        Arc::default(),
        services,
    )
    .unwrap();
    drop(plan.execute(0, state.task_ctx()).unwrap());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    collect(plan.clone(), state.task_ctx()).await.unwrap();
    let metrics = plan.metrics().unwrap();
    assert_eq!(metrics.output_rows(), Some(2));
    assert_eq!(metrics.sum_by_name("starts").unwrap().as_usize(), 1);
    assert_eq!(metrics.sum_by_name("early_drops").unwrap().as_usize(), 1);
    assert!(metrics.elapsed_compute().is_some());
}
#[derive(Debug)]
struct Settling {
    services: Arc<NativeExecutionContext>,
    started: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
    completed: Arc<tokio::sync::Notify>,
}
impl Body for Settling {
    fn execute(
        &self,
        _: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let services = self.services.clone();
        let started = self.started.clone();
        let release = self.release.clone();
        let completed = self.completed.clone();
        Ok(operation::batch(schema(), async move {
            services.require_settlement();
            started.notify_one();
            release.notified().await;
            completed.notify_one();
            Ok(RecordBatch::new_empty(schema()))
        }))
    }
}
#[tokio::test]
async fn abandoned_command_settles_without_replay() {
    let state = state();
    let services = NativeExecutionContext::from_session(&state).unwrap();
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let completed = Arc::new(tokio::sync::Notify::new());
    let plan = Execution::plan(
        "Settling",
        schema(),
        vec![],
        Arc::new(Settling {
            services: services.clone(),
            started: started.clone(),
            release: release.clone(),
            completed: completed.clone(),
        }),
        Family::Command,
        Arc::default(),
        services,
    )
    .unwrap();
    let stream = plan.execute(0, state.task_ctx()).unwrap();
    let reader = tokio::spawn(async move {
        let _ = stream.collect::<Vec<_>>().await;
    });
    started.notified().await;
    reader.abort();
    let _ = reader.await;
    release.notify_one();
    tokio::time::timeout(std::time::Duration::from_secs(2), completed.notified())
        .await
        .unwrap();
    assert!(collect(plan, state.task_ctx()).await.is_err());
}

#[tokio::test]
async fn transparent_owner_survives_reconstruction_fetch_and_stream_drop() {
    let batch =
        RecordBatch::try_new(schema(), vec![Arc::new(Int64Array::from(vec![3, 7]))]).unwrap();
    let input = datafusion::datasource::memory::MemorySourceConfig::try_new_exec(
        &[vec![batch]],
        schema(),
        None,
    )
    .unwrap();
    let owner = Arc::new(());
    let witness = Arc::downgrade(&owner);
    let plan = operation::ownership::execution(input.clone(), owner.clone());
    drop(owner);
    let plan = plan
        .replace_children(
            vec![input],
            datafusion::physical_plan::execution_plan::ReplaceChildrenOptions::new(
                datafusion::physical_plan::execution_plan::ChildrenPropertiesMode::Recompute,
            ),
        )
        .unwrap();
    let fetched = plan.with_fetch(Some(1)).unwrap();
    drop(plan);
    let plan = fetched;
    let stats = datafusion::physical_plan::statistics::StatisticsContext::new()
        .compute(
            plan.as_ref(),
            &datafusion::physical_plan::StatisticsArgs::new(),
        )
        .unwrap();
    assert_eq!(
        stats.num_rows,
        datafusion::common::stats::Precision::Exact(1)
    );
    let mut stream = plan.execute(0, Arc::new(TaskContext::default())).unwrap();
    drop(plan);
    assert!(witness.upgrade().is_some());
    assert_eq!(stream.next().await.unwrap().unwrap().num_rows(), 1);
    assert!(witness.upgrade().is_some());
    drop(stream);
    assert!(witness.upgrade().is_none());
}

#[tokio::test]
async fn explain_does_not_execute_commands_and_distinct_commands_do_not_merge() {
    let state = state();
    let (command, first) = fake(&[1], Family::Command);
    let explained = LogicalPlanBuilder::from(command.clone())
        .explain(false, false)
        .unwrap()
        .build()
        .unwrap();
    collect(physical(explained, &state).await, state.task_ctx())
        .await
        .unwrap();
    assert_eq!(first.load(Ordering::SeqCst), 0);
    let (other, second) = fake(&[1], Family::Command);
    let plan = LogicalPlanBuilder::from(command)
        .union(other)
        .unwrap()
        .limit(0, Some(0))
        .unwrap()
        .build()
        .unwrap();
    collect(physical(plan, &state).await, state.task_ctx())
        .await
        .unwrap();
    assert_eq!(first.load(Ordering::SeqCst), 1);
    assert_eq!(second.load(Ordering::SeqCst), 1);
}

#[test]
fn protecting_pure_requirements_does_not_invent_volatility() {
    let (value, _) = fake(&[1], Family::Finite);
    let (requirement, _) = fake(&[], Family::Finite);
    let plan = super::contract::ExecutionContract::plan(
        value,
        Some(requirement),
        std::collections::BTreeSet::default(),
    );
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20));
    let protected = super::contract::protect(plan, &pool, &CancellationToken::new()).unwrap();
    assert!(!super::freshness::check([&protected], |_| Ok(()), &CancellationToken::new()).unwrap());
    // Restoring optimizer-hidden producers must preserve the distinction between
    // required-work transport and an opaque, uncontracted operation.
    let expanded =
        super::cache::logical::expand(protected, &pool, &CancellationToken::new()).unwrap();
    assert!(!super::freshness::check([&expanded], |_| Ok(()), &CancellationToken::new()).unwrap());
}
