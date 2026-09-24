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

#[test]
fn equality_and_hash_preserve_shared_producer_owners_without_expanding_paths() {
    use datafusion::logical_expr::{LogicalPlanBuilder, Union, lit};
    let mut plan = LogicalPlanBuilder::empty(true)
        .project([lit(1_i64).alias("n")])
        .unwrap()
        .build()
        .unwrap();
    for _ in 0..28 {
        let cache = Cache {
            input: plan,
            identity: Arc::new(()),
            required: false,
            prepared: None,
            optimizer_leaf: false,
            admission: None,
            repeatable_input: false,
            binding: None,
            retention: None,
        };
        let shared = Arc::new(LogicalPlan::Extension(Extension {
            node: Arc::new(cache),
        }));
        plan = LogicalPlan::Union(Union {
            schema: shared.schema().clone(),
            inputs: vec![shared.clone(), shared],
        });
    }
    let clone = plan.clone();
    assert_eq!(plan, clone);
    assert!(crate::operation::ports::same_input(&plan, &clone));
    let hash = |value: &LogicalPlan| {
        let mut state = std::hash::DefaultHasher::new();
        value.hash(&mut state);
        state.finish()
    };
    assert_eq!(hash(&plan), hash(&clone));
    let LogicalPlan::Extension(extension) = plan.inputs()[0] else {
        panic!("shared producer absent");
    };
    let producer = extension.node.as_any().downcast_ref::<Cache>().unwrap();
    let unchanged = UserDefinedLogicalNodeCore::with_exprs_and_inputs(
        producer,
        vec![],
        vec![producer.input.clone()],
    )
    .unwrap();
    assert!(Arc::ptr_eq(&producer.identity, &unchanged.identity));
    assert_eq!(producer, &unchanged);
    assert_eq!(
        producer.partial_cmp(&unchanged),
        Some(std::cmp::Ordering::Equal)
    );
    let replacement = LogicalPlanBuilder::empty(true)
        .project([lit(2_i64).alias("n")])
        .unwrap()
        .build()
        .unwrap();
    let changed =
        UserDefinedLogicalNodeCore::with_exprs_and_inputs(producer, vec![], vec![replacement])
            .unwrap();
    assert!(!Arc::ptr_eq(&producer.identity, &changed.identity));
    assert_ne!(producer, &changed);
    let other_owner = Cache {
        identity: Arc::new(()),
        ..producer.clone()
    };
    assert_ne!(
        producer, &other_owner,
        "equal-looking work is not the same producer"
    );
    let incompatible = Cache {
        input: LogicalPlanBuilder::empty(true)
            .project([lit("changed storage").alias("n")])
            .unwrap()
            .build()
            .unwrap(),
        ..producer.clone()
    };
    assert_ne!(producer, &incompatible);
    assert_eq!(producer.partial_cmp(&incompatible), None);
}

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
fn diagnostic_physical_graph_includes_independently_planned_producers() {
    let (input, calls) = source();
    let mut cache = CacheExec::new(input, Arc::new(Completion::default()));
    cache.planned_boundary = true;
    assert!(
        cache.children().is_empty(),
        "optimizer boundary remains closed"
    );
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64 << 10));
    let logical = datafusion::logical_expr::LogicalPlanBuilder::empty(true)
        .build()
        .unwrap();
    let observation = super::super::observation::Recorder::new(
        &pool,
        super::super::assurance::ObservationPolicy::Diagnostic,
    )
    .finish(&logical, &pse_columnar::CancellationToken::new())
    .unwrap()
    .with_physical(&cache, &pool)
    .unwrap();
    let physical = observation.physical_plan().unwrap();
    assert!(physical.contains("NativeCacheExec"));
    assert!(physical.contains("Counted"));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "observation never executes a producer"
    );
    drop(observation);
    assert_eq!(pool.reserved(), 0);
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
    let budget: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 20));
    let cancel = pse_columnar::CancellationToken::new();
    let mut calls = 0;
    let result = logical::Scope::new(&budget, &cancel)
        .stage(plan, &mut |_, _| {
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
async fn abandoned_reader_is_terminal_and_does_not_restart_partial_input() {
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
    let result = cache
        .execute(0, context)
        .unwrap()
        .try_collect::<Vec<_>>()
        .await;
    assert!(
        result.is_err(),
        "last-reader abandonment terminates this invocation"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn completed_physical_reuse_requires_live_owners_success_and_the_current_epoch() {
    use datafusion::logical_expr::{LogicalPlanBuilder, lit};
    use pse_columnar::CancellationToken;
    let budget: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(32 << 20));
    let factory = super::super::EngineFactory::new(
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
    let fresh =
        super::super::execution::NativeExecutionContext::execution_state(&state, &cancel).unwrap();
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
    let mut memory = MemoryConsumer::new("test:physical-substitution").register(services.pool());
    let mut substitutions = HashMap::new();
    let result = substitute_completed(plan, &services, &mut substitutions, &mut memory).unwrap();
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
    session: &super::super::EngineSession,
    cancel: &pse_columnar::CancellationToken,
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
    struct Caller(Arc<AtomicUsize>);
    impl CacheFactory for Caller {
        fn create(&self, plan: LogicalPlan, _: &SessionState) -> Result<LogicalPlan> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(plan)
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let caller: Arc<dyn CacheFactory> = Arc::new(Caller(calls.clone()));
    let factory = super::super::EngineFactory::from_builder(
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        Arc::new(GreedyMemoryPool::new(1 << 20)),
        "cache-unit",
        datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .with_cache_factory(Some(Arc::clone(&caller))),
    );
    assert!(Arc::ptr_eq(
        factory.native_state().cache_factory().unwrap(),
        &caller
    ));
    let cancel = pse_columnar::CancellationToken::new();
    let session = factory
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let inputs = [true, false, true].map(|rows| {
        datafusion::logical_expr::LogicalPlanBuilder::empty(rows)
            .build()
            .unwrap()
    });
    assert_eq!(session.cache_plans(&inputs, &cancel).unwrap(), inputs);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    cancel.cancel();
    assert!(session.cache_plans(&inputs, &cancel).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 3);
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
async fn native_spill_preserves_values_and_charges_live_decoded_buffers() {
    let (input, calls) = source();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(Arc::new(GreedyMemoryPool::new(1 << 20)))
        .build_arc()
        .unwrap();
    let context = Arc::new(TaskContext::default().with_runtime(runtime.clone()));
    for _ in 0..2 {
        let cached = Arc::new(
            Cached::collect_bounded(
                input.clone(),
                context.clone(),
                &ExecutionPlanMetricsSet::new(),
                1,
            )
            .await
            .unwrap(),
        );
        assert!(!cached.resident());
        assert_eq!(runtime.memory_pool.reserved(), 0);
        let left = cached
            .stream()
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        let right = cached
            .stream()
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert_eq!(left, right);
        assert_eq!(left[0].num_rows(), 2);
        assert!(runtime.memory_pool.reserved() > 0);
        drop(left);
        drop(right);
        drop(cached);
        assert_eq!(runtime.memory_pool.reserved(), 0);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn spill_reader_cannot_bypass_the_native_allocation_budget() {
    let (input, _) = source();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(Arc::new(GreedyMemoryPool::new(1)))
        .build_arc()
        .unwrap();
    let context = Arc::new(TaskContext::default().with_runtime(runtime.clone()));
    let cached = Arc::new(
        Cached::collect_bounded(input, context, &ExecutionPlanMetricsSet::new(), 1)
            .await
            .unwrap(),
    );
    assert!(!cached.resident());
    assert!(
        cached
            .stream()
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .is_err()
    );
    assert_eq!(runtime.memory_pool.reserved(), 0);
}

#[tokio::test]
async fn resident_cache_and_export_share_allocation_capacity() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 20));
    let array: datafusion::arrow::array::ArrayRef = Arc::new(Int64Array::from(vec![3, 7]));
    let batch = RecordBatch::try_from_iter([("value", array)]).unwrap();
    let scope = pse_columnar::owned_buffer::AllocationScope::default();
    let owned = scope.retain_native(batch, &pool).unwrap();
    let batch = owned.batch().clone();
    let original = pool.reserved();
    let input =
        MemorySourceConfig::try_new_exec(&[vec![batch.clone()]], batch.schema(), None).unwrap();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let cached = Arc::new(
        Cached::collect(
            input,
            Arc::new(
                TaskContext::default()
                    .with_session_config(
                        datafusion::execution::context::SessionConfig::new()
                            .with_extension(Arc::new(scope)),
                    )
                    .with_runtime(runtime),
            ),
            &ExecutionPlanMetricsSet::new(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(pool.reserved(), original + cached.reservation.size());
    let exported = cached
        .stream()
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(pool.reserved(), original + cached.reservation.size());
    drop(cached);
    drop(batch);
    drop(owned);
    assert!(pool.reserved() > 0);
    drop(exported);
    assert_eq!(pool.reserved(), 0);
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
    assert!(cached.retained_bytes() > 0);
    let batches = cached
        .stream()
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    drop(cached);
    // Cache metadata and active-reader pins are gone; escaped buffers retain
    // only their actual allocation charges, independently of the whole cache.
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(pinned.load(Ordering::Acquire), 0);
    let array = batches[0].column(0).clone();
    drop(batches);
    assert!(pool.reserved() > 0);
    drop(array);
    assert_eq!(pool.reserved(), 0);
    assert_eq!(live.load(Ordering::Acquire), 0);
    assert_eq!(pinned.load(Ordering::Acquire), 0);
}

#[tokio::test]
async fn reusable_group_refills_slots_before_slow_first_output_and_restores_order() {
    use crate::session::{
        EngineFactory, ReusableGroup, physical_input::PhysicalInput, round::RoundResetSupport,
    };
    use datafusion::execution::session_state::SessionStateBuilder;
    let context = datafusion::prelude::SessionContext::new();
    let config = context
        .copied_config()
        .with_extension(Arc::new(RoundResetSupport(
            <dyn ExecutionPlan>::is::<Counted>,
        )));
    let mut policy = crate::cache_service::CacheBudget::disabled(1 << 20);
    policy.concurrent_queries = 2.try_into().unwrap();
    policy.concurrent_outputs = 2.try_into().unwrap();
    let service =
        crate::cache_service::NativeCacheService::new(policy, &context.runtime_env().memory_pool)
            .unwrap();
    let factory = EngineFactory::from_builder(
        context.runtime_env(),
        context.runtime_env().memory_pool.clone(),
        "unit",
        SessionStateBuilder::from(context.state())
            .with_config(config)
            .with_query_planner(Arc::new(crate::session::planner::UnifiedPlanner::default())),
    )
    .with_cache_service(service);
    let cancel = pse_columnar::CancellationToken::new();
    let mut session = factory
        .candidate(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let gate = Arc::new(tokio::sync::Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let mut roots = Vec::new();
    for ordinal in 0..3_i64 {
        let name = format!("source{ordinal}");
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![ordinal]))],
        )
        .unwrap();
        let input = MemorySourceConfig::try_new_exec(&[vec![batch]], schema, None).unwrap();
        let provider = Arc::new(PhysicalInput::native(Arc::new(Counted {
            input,
            calls: calls.clone(),
            gate: (ordinal == 0).then(|| gate.clone()),
        })));
        session
            .bindings
            .insert(
                crate::provider::binding::BindingKey::Input(name.clone()),
                crate::provider::binding::TableBinding::new(
                    datafusion::common::TableReference::full("unit", "source", name.clone()),
                    provider,
                    None,
                    None,
                ),
            )
            .unwrap();
        roots.push(session.scan_role(&name).unwrap());
    }
    let mut group = ReusableGroup::prepare(session.prepare_many(&roots, &cancel).unwrap(), &cancel)
        .await
        .unwrap();
    let (results, ()) = tokio::join!(group.execute(&cancel), async {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while calls.load(Ordering::SeqCst) != 3 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("third output must start while first is still blocked");
        gate.notify_one();
    });
    let values: Vec<_> = results
        .unwrap()
        .iter()
        .map(|output| {
            output.batches()[0]
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(0)
        })
        .collect();
    assert_eq!(values, vec![0, 1, 2]);
}

#[tokio::test]
async fn related_reusable_roots_execute_one_actual_producer_per_group_epoch() {
    use crate::session::{
        EngineFactory, ReusableGroup, physical_input::PhysicalInput, round::RoundResetSupport,
    };
    use datafusion::execution::session_state::SessionStateBuilder;
    let (input, calls) = source();
    let context = datafusion::prelude::SessionContext::new();
    let config = context
        .copied_config()
        .with_extension(Arc::new(RoundResetSupport(
            <dyn ExecutionPlan>::is::<Counted>,
        )));
    let mut limits = crate::cache_service::CacheBudget::disabled(1 << 20);
    limits.concurrent_queries = 2.try_into().unwrap();
    limits.concurrent_outputs = 2.try_into().unwrap();
    let service =
        crate::cache_service::NativeCacheService::new(limits, &context.runtime_env().memory_pool)
            .unwrap();
    let factory = EngineFactory::from_builder(
        context.runtime_env(),
        context.runtime_env().memory_pool.clone(),
        "unit",
        SessionStateBuilder::from(context.state())
            .with_config(config)
            .with_query_planner(Arc::new(crate::session::planner::UnifiedPlanner::default())),
    )
    .with_cache_service(service);
    let cancel = pse_columnar::CancellationToken::new();
    let mut session = factory
        .candidate(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let provider: Arc<dyn datafusion::catalog::TableProvider> =
        Arc::new(PhysicalInput::native(input));
    session
        .bindings
        .insert(
            crate::provider::binding::BindingKey::Input("counted".into()),
            crate::provider::binding::TableBinding::new(
                datafusion::common::TableReference::full("unit", "source", "counted"),
                provider.clone(),
                None,
                None,
            ),
        )
        .unwrap();
    let input = session.scan_role("counted").unwrap();
    let shared = session.cache_plan(input, &cancel).unwrap();
    let mut group = ReusableGroup::prepare(
        vec![
            session.prepare(shared.clone(), &cancel).unwrap(),
            session.prepare(shared, &cancel).unwrap(),
        ],
        &cancel,
    )
    .await
    .unwrap();
    for expected in 1..=2 {
        let completed = group.execute(&cancel).await.unwrap();
        assert_eq!(completed.len(), 2);
        for output in completed {
            assert_eq!(
                output
                    .batches()
                    .iter()
                    .map(|batch| batch.num_rows())
                    .sum::<usize>(),
                2
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), expected);
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one actual runtime exercises completed reuse and saturated sibling/nested permit ownership"
)]
async fn model_retention_is_completed_only_and_preserves_live_nested_admission() {
    use crate::cache_service::{CacheBudget, NativeCacheService};
    use datafusion::logical_expr::{LogicalPlanBuilder, lit};
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(32 << 20));
    let runtime = Arc::new(
        RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .build()
            .unwrap(),
    );
    let mut policy = CacheBudget::disabled(1 << 20);
    policy.model_result_bytes = 1 << 20;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let factory = super::super::EngineFactory::new(
        runtime,
        pool.clone(),
        super::super::ExecutionSettings::default(),
        super::super::ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        super::super::native_engine_profile(),
    )
    .unwrap()
    .with_cache_service(service.clone());
    let cancel = pse_columnar::CancellationToken::new();
    let session = factory
        .candidate(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let plan = session
        .cache_plan(
            LogicalPlanBuilder::empty(true)
                .project([lit(9_i64).alias("value")])
                .unwrap()
                .build()
                .unwrap(),
            &cancel,
        )
        .unwrap();
    for _ in 0..2 {
        session
            .prepare(plan.clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
    }
    let report = service
        .report()
        .into_iter()
        .find(|r| r.name == "pse.cache.model_results")
        .unwrap();
    assert_eq!(report.entries, 1);
    assert!(
        report.hits >= 1,
        "the second attempt must reuse the completed output"
    );
    // A source the producer does not consume cannot invalidate its completion.
    let unrelated = session
        .with_provider(
            datafusion::common::TableReference::bare("unrelated"),
            Arc::new(
                datafusion::datasource::MemTable::try_new(Arc::new(Schema::empty()), vec![vec![]])
                    .unwrap(),
            ),
            &cancel,
        )
        .unwrap();
    unrelated
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert!(
        service
            .report()
            .into_iter()
            .find(|r| r.name == "pse.cache.model_results")
            .unwrap()
            .hits
            > report.hits
    );
    // Saturate the only outer-query permit. A nested candidate inherits it.
    let permit = Arc::new(service.admit_query(&cancel).await.unwrap());
    let state = super::super::execution::NativeExecutionContext::bind(
        &session,
        session.bound_state().unwrap(),
        &cancel,
    )
    .unwrap();
    let services = super::super::execution::NativeExecutionContext::from_session(&state).unwrap();
    services.register_query_admission(&permit);
    // Sibling executions share producer cells, never their admission slot.
    let sibling_state =
        super::super::execution::NativeExecutionContext::execution_state_with_caches(
            &state,
            &cancel,
            services.caches.clone(),
        )
        .unwrap();
    let sibling =
        super::super::execution::NativeExecutionContext::from_session(&sibling_state).unwrap();
    let sibling_permit = Arc::new(
        Arc::new(tokio::sync::Semaphore::new(1))
            .acquire_owned()
            .await
            .unwrap(),
    );
    sibling.register_query_admission(&sibling_permit);

    let nested = services
        .candidate_roles(&state, std::collections::BTreeMap::new())
        .unwrap();
    assert!(Arc::ptr_eq(
        &nested.query_admission.upgrade().unwrap(),
        &permit
    ));
    let child = nested
        .prepare(LogicalPlanBuilder::empty(true).build().unwrap(), &cancel)
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), child.execute(&cancel))
        .await
        .unwrap()
        .unwrap();
    let mut waiting = Box::pin(service.admit_query(&cancel));
    assert!(futures_util::poll!(waiting.as_mut()).is_pending());
    drop(permit);
    let _permit = waiting.await.unwrap();
    service.invalidate();
    assert_eq!(
        service
            .report()
            .into_iter()
            .find(|r| r.name == "pse.cache.model_results")
            .unwrap()
            .entries,
        0
    );
}

#[tokio::test]
async fn completed_reader_publishes_only_exhausted_exact_row_statistics() {
    use datafusion::physical_plan::statistics::{StatisticsArgs, StatisticsContext};
    let (input, _) = source();
    let value = Arc::new(
        Cached::collect(
            input,
            Arc::new(TaskContext::default()),
            &ExecutionPlanMetricsSet::new(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(value.row_count(), 2);
    let reader = value.reader_plan().unwrap();
    let stats = StatisticsContext::new()
        .compute(reader.as_ref(), &StatisticsArgs::new())
        .unwrap();
    assert_eq!(
        stats.num_rows,
        datafusion::common::stats::Precision::Exact(2)
    );
}
