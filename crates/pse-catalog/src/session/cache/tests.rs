// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Tiny physical-operator tests, including native spill. No compiler/Delta journey.
use super::*;
use datafusion::{
    arrow::{
        array::Int64Array,
        datatypes::{DataType, Field, Schema},
    },
    datasource::memory::MemorySourceConfig,
    execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool},
        runtime_env::RuntimeEnvBuilder,
    },
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct Counted {
    input: Arc<dyn ExecutionPlan>,
    calls: Arc<AtomicUsize>,
    gate: Option<Arc<tokio::sync::Notify>>,
}
impl DisplayAs for Counted {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Counted")
    }
}
impl ExecutionPlan for Counted {
    fn name(&self) -> &'static str {
        "Counted"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.input.properties()
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(Self {
            input: Arc::clone(&children[0]),
            calls: Arc::clone(&self.calls),
            gate: self.gate.clone(),
        }))
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(gate) = &self.gate {
            let gate = Arc::clone(gate);
            let input = Arc::clone(&self.input);
            let stream = futures_util::stream::once(async move {
                gate.notified().await;
                input.execute(partition, context)
            })
            .try_flatten();
            Ok(Box::pin(RecordBatchStreamAdapter::new(
                self.schema(),
                stream,
            )))
        } else {
            self.input.execute(partition, context)
        }
    }
}
fn source() -> (Arc<dyn ExecutionPlan>, Arc<AtomicUsize>) {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Int64,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(Int64Array::from(vec![3, 7]))],
    )
    .unwrap();
    let input = MemorySourceConfig::try_new_exec(&[vec![batch]], schema, None).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    (
        Arc::new(Counted {
            input,
            calls: Arc::clone(&calls),
            gate: None,
        }),
        calls,
    )
}

#[test]
fn shared_producer_optimization_compares_original_schemas_before_native_refinement() {
    let state = datafusion::prelude::SessionContext::new().state();
    let empty = |nullable| {
        LogicalPlan::EmptyRelation(datafusion::logical_expr::EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(
                datafusion::common::DFSchema::try_from(Schema::new(vec![Field::new(
                    "value",
                    DataType::Int64,
                    nullable,
                )]))
                .unwrap(),
            ),
        })
    };
    let producer = NativeCacheFactory.create(empty(true), &state).unwrap();
    let plan = LogicalPlan::Union(
        datafusion::logical_expr::Union::try_new(vec![
            Arc::new(producer.clone()),
            Arc::new(producer),
        ])
        .unwrap(),
    );
    let budget = pse_ids::FixedBudget::new(1 << 20);
    let cancel = pse_ids::CancellationToken::new();
    let mut calls = 0;
    let result = logical::Scope::new(budget.as_ref(), &cancel)
        .stage(plan, &mut |_| {
            calls += 1;
            Ok(empty(false))
        })
        .unwrap();
    assert_eq!(calls, 1);
    assert!(
        result
            .inputs()
            .iter()
            .all(|input| !input.schema().field(0).is_nullable())
    );
}

#[tokio::test]
async fn abandoned_reader_does_not_restart_partial_input() {
    let (input, _) = source();
    let calls = Arc::new(AtomicUsize::new(0));
    let gate = Arc::new(tokio::sync::Notify::new());
    let input = Arc::new(Counted {
        input,
        calls: Arc::clone(&calls),
        gate: Some(Arc::clone(&gate)),
    });
    let cache = CacheExec::new(input, Arc::new(Completion::default()));
    let context = Arc::new(TaskContext::default());
    let mut first = cache.execute(0, Arc::clone(&context)).unwrap();
    assert!(futures_util::poll!(first.try_next()).is_pending());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(first);
    gate.notify_one();
    let rows = cache
        .execute(0, context)
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(rows[0].num_rows(), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn completed_physical_reuse_requires_live_owners_success_and_the_current_epoch() {
    use datafusion::logical_expr::{LogicalPlanBuilder, lit};
    use pse_ids::CancellationToken;
    let budget = pse_ids::FixedBudget::new(32 << 20);
    let factory = super::super::SessionFactory::new(
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        budget.clone(),
        super::super::ExecutionSettings::default(),
        super::super::ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        super::super::native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    assert_rewritten_producer_keeps_its_own_values(&session, &cancel).await;
    let plan = session
        .cache_plan(
            LogicalPlanBuilder::empty(true)
                .project([lit(3i64).alias("value")])
                .unwrap()
                .build()
                .unwrap(),
            &cancel,
        )
        .unwrap();
    let prepared = session.prepare(plan, &cancel).unwrap();
    let plan = prepared.optimized_plan();
    let state = super::super::execution::NativeExecutionContext::bind(
        &session,
        session.bound_state().unwrap(),
        &cancel,
    )
    .unwrap();
    assert_uncompleted_substitution_shares_producers(plan.clone(), &state);
    assert_shared_producer_planning_is_lazy(plan.clone(), &state).await;
    let physical = state
        .query_planner()
        .create_physical_plan(plan, &state)
        .await
        .unwrap();
    assert_eq!(completed_nodes(&reuse_completed(plan, &state).unwrap()), 0);
    let first = datafusion::physical_plan::collect(physical.clone(), state.task_ctx())
        .await
        .unwrap();
    let replay = {
        let ready = reuse_completed(plan, &state).unwrap();
        assert_eq!(completed_nodes(&ready), 1);
        let replay = state
            .query_planner()
            .create_physical_plan(&ready, &state)
            .await
            .unwrap();
        assert_eq!(
            datafusion::physical_plan::collect(replay.clone(), state.task_ctx())
                .await
                .unwrap(),
            first
        );
        replay
    };
    let fresh = super::super::execution::NativeExecutionContext::execution_state(&state, &cancel);
    assert_eq!(completed_nodes(&reuse_completed(plan, &fresh).unwrap()), 0);
    let services = super::super::execution::NativeExecutionContext::from_session(&state).unwrap();
    services.advance_round_epoch();
    assert_eq!(completed_nodes(&reuse_completed(plan, &state).unwrap()), 0);
    assert!(
        datafusion::physical_plan::collect(replay, state.task_ctx())
            .await
            .unwrap_err()
            .to_string()
            .contains("earlier execution epoch")
    );
    drop(physical);
    assert!(
        services
            .caches
            .0
            .lock()
            .unwrap()
            .values()
            .all(|(_, cell, physical)| cell.upgrade().is_none()
                && physical
                    .as_ref()
                    .is_none_or(|plan| plan.upgrade().is_none()))
    );
}

fn assert_uncompleted_substitution_shares_producers(mut plan: LogicalPlan, state: &SessionState) {
    for _ in 0..20 {
        let shared = Arc::new(plan);
        let input = LogicalPlan::Union(datafusion::logical_expr::Union {
            schema: Arc::clone(shared.schema()),
            inputs: vec![Arc::clone(&shared), shared],
        });
        plan = NativeCacheFactory.create(input, state).unwrap();
    }
    let services = super::super::execution::NativeExecutionContext::from_session(state).unwrap();
    let mut memory = services.reserver().open("test:physical-substitution");
    let mut substitutions = HashMap::new();
    let result =
        substitute_completed(plan, &services, &mut substitutions, memory.as_mut()).unwrap();
    assert_eq!(
        substitutions.len(),
        21,
        "substitution scales with producers, not expanded paths"
    );
    assert!(
        services.caches.0.lock().unwrap().is_empty(),
        "planning did not execute any input"
    );
    let mut owners = HashMap::new();
    result
        .apply_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            {
                assert!(cache.prepared.is_none());
                let key = Arc::as_ptr(&cache.identity) as usize;
                if let Some(owner) = owners.get(&key) {
                    assert!(Arc::ptr_eq(owner, &extension.node));
                    return Ok(TreeNodeRecursion::Jump);
                }
                owners.insert(key, Arc::clone(&extension.node));
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .unwrap();
    assert_eq!(owners.len(), 21);
}

async fn assert_rewritten_producer_keeps_its_own_values(
    session: &super::super::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) {
    use datafusion::logical_expr::{LogicalPlanBuilder, lit};
    let value = |number| {
        LogicalPlanBuilder::empty(true)
            .project([lit(number).alias("value")])
            .unwrap()
            .build()
            .unwrap()
    };
    let original = session.cache_plan(value(3i64), cancel).unwrap();
    let changed = original
        .clone()
        .transform_down_with_subqueries(|node| {
            if let LogicalPlan::Extension(extension) = &node
                && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            {
                return Ok(Transformed::new(
                    LogicalPlan::Extension(Extension {
                        node: Arc::new(UserDefinedLogicalNodeCore::with_exprs_and_inputs(
                            cache,
                            vec![],
                            vec![value(7i64)],
                        )?),
                    }),
                    true,
                    TreeNodeRecursion::Jump,
                ));
            }
            Ok(Transformed::no(node))
        })
        .unwrap()
        .data;
    let plan = LogicalPlan::Union(
        datafusion::logical_expr::Union::try_new(vec![Arc::new(original), Arc::new(changed)])
            .unwrap(),
    );
    let output = session
        .prepare(plan, cancel)
        .unwrap()
        .execute(cancel)
        .await
        .unwrap();
    let mut values = output
        .batches()
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect::<Vec<_>>();
    values.sort_unstable();
    assert_eq!(values, [3, 7]);
}

async fn assert_shared_producer_planning_is_lazy(mut plan: LogicalPlan, state: &SessionState) {
    for _ in 0..10 {
        let shared = Arc::new(plan);
        let input = LogicalPlan::Union(datafusion::logical_expr::Union {
            schema: Arc::clone(shared.schema()),
            inputs: vec![Arc::clone(&shared), shared],
        });
        plan = NativeCacheFactory.create(input, state).unwrap();
    }
    let physical = state
        .query_planner()
        .create_physical_plan(&plan, state)
        .await
        .unwrap();
    let services = super::super::execution::NativeExecutionContext::from_session(state).unwrap();
    let cells = services.caches.0.lock().unwrap();
    assert_eq!(
        cells.len(),
        11,
        "native planning retains one physical producer per cache"
    );
    for (_, cell, _) in cells.values() {
        assert!(
            cell.upgrade().unwrap().0.lock().unwrap().is_none(),
            "planning must not execute any producer"
        );
    }
    assert_eq!(physical.schema().as_ref(), plan.schema().as_arrow());
}

fn completed_nodes(plan: &LogicalPlan) -> usize {
    let mut count = 0;
    plan.apply_with_subqueries(|node| {
        if let LogicalPlan::Extension(extension) = node
            && let Some(cache) = extension.node.as_any().downcast_ref::<Cache>()
            && cache.prepared.is_some()
        {
            assert!(UserDefinedLogicalNodeCore::inputs(cache).is_empty());
            assert!(cache.prepared.as_ref().unwrap().children().is_empty());
            count += 1;
        }
        Ok(TreeNodeRecursion::Continue)
    })
    .unwrap();
    count
}

#[test]
fn caller_cache_factory_is_retained() {
    #[derive(Debug)]
    struct Caller;
    impl CacheFactory for Caller {
        fn create(&self, plan: LogicalPlan, _: &SessionState) -> Result<LogicalPlan> {
            Ok(plan)
        }
    }
    let caller: Arc<dyn CacheFactory> = Arc::new(Caller);
    let factory = super::super::SessionFactory::from_builder(
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        pse_ids::FixedBudget::new(1 << 20),
        "cache-unit",
        datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .with_cache_factory(Some(Arc::clone(&caller))),
    );
    assert!(Arc::ptr_eq(
        factory.native_state().cache_factory().unwrap(),
        &caller
    ));
}

#[tokio::test]
async fn lazy_siblings_share_input_and_release_reservations() {
    let (input, calls) = source();
    let store = CacheStore::default();
    let identity = Arc::new(());
    let first = CacheExec::new(Arc::clone(&input), store.cell(&identity).unwrap());
    let second = CacheExec::new(input, store.cell(&identity).unwrap());
    let context = Arc::new(TaskContext::default());
    let pool = Arc::clone(&context.runtime_env().memory_pool);
    let left = first.execute(0, Arc::clone(&context)).unwrap();
    let right = second.execute(0, context).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let (left, right) = tokio::join!(left.try_collect::<Vec<_>>(), right.try_collect::<Vec<_>>());
    assert_eq!(left.unwrap(), right.unwrap());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(pool.reserved() > 0);
    drop(first);
    drop(second);
    assert_eq!(
        pool.reserved(),
        0,
        "the invocation index must not own completed buffers"
    );
}

#[tokio::test]
async fn native_spill_preserves_values_and_is_not_a_durable_memo() {
    let (input, calls) = source();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(Arc::new(GreedyMemoryPool::new(1)))
        .build_arc()
        .unwrap();
    let context = Arc::new(TaskContext::default().with_runtime(Arc::clone(&runtime)));
    let identity = Arc::new(());
    for _ in 0..2 {
        let invocation = CacheStore::default();
        let cached = CacheExec::new(Arc::clone(&input), invocation.cell(&identity).unwrap());
        let left = cached
            .execute(0, Arc::clone(&context))
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        let right = cached
            .execute(0, Arc::clone(&context))
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert_eq!(left, right);
        assert_eq!(left[0].num_rows(), 2);
        let future = cached
            .completion
            .0
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        assert!(future.1.await.unwrap().spill.is_some());
        assert_eq!(runtime.memory_pool.reserved(), 0);
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "a new invocation must execute again"
    );
}

#[tokio::test]
async fn a_new_epoch_discards_completed_input() {
    let (input, calls) = source();
    let store = CacheStore::default();
    let identity = Arc::new(());
    let cache = CacheExec::new(input, store.cell(&identity).unwrap());
    let context = Arc::new(TaskContext::default());
    for _ in 0..2 {
        cache
            .execute(0, context.clone())
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    store.advance_epoch();
    cache
        .execute(0, context)
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn exported_cached_arrays_keep_the_unique_reservation_after_eviction_and_stream_drop() {
    let (input, _) = source();
    let pool = Arc::new(GreedyMemoryPool::new(1 << 20));
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let context = Arc::new(TaskContext::default().with_runtime(runtime));
    let live = Arc::new(AtomicUsize::new(0));
    let pinned = Arc::new(AtomicUsize::new(0));
    let cached = Arc::new(
        Cached::collect(input, context, &ExecutionPlanMetricsSet::new())
            .await
            .unwrap()
            .account(live.clone(), pinned.clone()),
    );
    let bytes = cached.retained_bytes();
    let batches = cached
        .stream()
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    drop(cached);
    assert_eq!(live.load(Ordering::Acquire), bytes);
    assert_eq!(pinned.load(Ordering::Acquire), bytes);
    let array = batches[0].column(0).clone();
    drop(batches);
    assert!(pool.reserved() > 0);
    drop(array);
    assert_eq!(pool.reserved(), 0);
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(pinned.load(Ordering::Acquire), 0);
}
