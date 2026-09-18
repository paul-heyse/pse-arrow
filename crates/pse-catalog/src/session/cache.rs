// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Invocation-local native caching. Binding never executes input. The first reader
//! materializes under DataFusion's memory pool, spilling through its `SpillManager`;
//! sibling readers share completion and failures within this execution only.
use datafusion::{
    arrow::array::RecordBatch,
    catalog::Session,
    common::{
        DFSchemaRef, DataFusionError, Result,
        tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    },
    execution::{
        TaskContext,
        memory_pool::{MemoryConsumer, MemoryReservation},
        session_state::{CacheFactory, SessionState},
    },
    logical_expr::{
        Expr, Extension, LogicalPlan, UserDefinedLogicalNode, UserDefinedLogicalNodeCore,
        physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream, SpillManager, execute_stream,
        execution_plan::{Boundedness, EmissionType},
        metrics::{ExecutionPlanMetricsSet, SpillMetrics},
        stream::RecordBatchStreamAdapter,
        streaming::{PartitionStream, StreamingTableExec},
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::{FutureExt, TryStreamExt};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

mod admission;
pub(super) mod logical;
pub(super) mod physical;
pub(super) use admission::admitted;

impl super::SnapshotSession {
    /// Insert the actual caller's native cache plan without executing any input.
    /// The default cache shares one bounded computation within an invocation and
    /// spills using the same runtime as its input. It confers no durable reuse.
    /// # Errors
    /// Invalid fields/sources or the caller's factory refuses this plan.
    pub fn cache_plan(
        &self,
        input: LogicalPlan,
        cancel: &pse_ids::CancellationToken,
    ) -> std::result::Result<LogicalPlan, crate::CatalogError> {
        let input = self.derive_plan_fields(input, cancel)?;
        let state =
            super::execution::NativeExecutionContext::bind(self, self.bound_state()?, cancel)?;
        let factory = state
            .cache_factory()
            .ok_or_else(|| super::engine(invalid("native cache factory absent")))?;
        let plan = factory.create(input, &state).map_err(super::engine)?;
        // Admit the actual result: a caller factory can return another computation.
        let plan = self.derive_plan_fields(plan, cancel)?;
        admission::seal(plan, self, cancel).map_err(super::engine)
    }
}

/// Default deferred cache factory. Explicit caller factories are never replaced.
#[derive(Debug)]
pub struct NativeCacheFactory;
impl CacheFactory for NativeCacheFactory {
    fn create(&self, input: LogicalPlan, state: &SessionState) -> Result<LogicalPlan> {
        let services = state
            .config()
            .get_extension::<super::execution::NativeExecutionContext>();
        let fresh = if let Some(services) = services {
            let mut reservation = services.reserver().open("session:cache-freshness");
            super::freshness::check(
                [&input],
                |bytes| {
                    reservation.try_grow(bytes).map_err(|error| {
                        DataFusionError::External(Box::new(crate::CatalogError::from(error)))
                    })
                },
                services.cancellation(),
            )?
        } else {
            // Standalone native CacheFactory callers have their actual native
            // pool, even when no platform execution context has been attached.
            let reservation = MemoryConsumer::new("session:cache-freshness")
                .register(&state.runtime_env().memory_pool);
            super::freshness::check(
                [&input],
                |bytes| reservation.try_grow(bytes),
                &pse_ids::CancellationToken::new(),
            )?
        };
        if fresh {
            return Err(invalid(
                "native cache requires an explicitly pure, repeatable input",
            ));
        }
        Ok(super::contract::ExecutionContract::plan(
            LogicalPlan::Extension(Extension {
                node: Arc::new(Cache {
                    input,
                    identity: Arc::new(()),
                    prepared: None,
                    optimizer_leaf: false,
                    admission: None,
                    binding: None,
                }),
            }),
            None,
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        ))
    }
}

#[derive(Clone)]
struct Cache {
    input: LogicalPlan,
    identity: Arc<()>,
    // Physical-planning-only substitution for a completed cell in this invocation.
    prepared: Option<Arc<dyn ExecutionPlan>>,
    // A private, temporary boundary around an independently optimized producer.
    // Expanded again before admission, physical planning and public inspection.
    optimizer_leaf: bool,
    // Disposable structural admission of this exact, unchanged closed input.
    admission: Option<Arc<admission::Admission>>,
    // Temporary SQL binding boundary, restored to this exact producer afterward.
    binding: Option<Arc<dyn UserDefinedLogicalNode>>,
}

pub(super) fn supports_round_reset(node: &dyn UserDefinedLogicalNode) -> bool {
    node.as_any().is::<Cache>()
}
pub(super) fn supports_physical_reset(plan: &dyn ExecutionPlan) -> bool {
    plan.is::<CacheExec>()
}
pub(super) fn reset_dependency(plan: &dyn ExecutionPlan) -> Option<&Arc<dyn ExecutionPlan>> {
    plan.downcast_ref::<CacheExec>()
        .filter(|cache| cache.planned_boundary)
        .map(|cache| &cache.input)
}

#[cfg(test)]
mod freshness_tests {
    use super::*;

    fn fresh<'a>(plans: impl IntoIterator<Item = &'a LogicalPlan>) -> Result<bool> {
        let budget = pse_ids::FixedBudget::new(1 << 20);
        let mut reservation = pse_ids::MemoryReserver::open(budget.as_ref(), "test:freshness");
        super::super::freshness::check(
            plans,
            |bytes| {
                reservation
                    .try_grow(bytes)
                    .map_err(|error| DataFusionError::External(Box::new(error)))
            },
            &pse_ids::CancellationToken::new(),
        )
    }

    #[test]
    fn unknown_extensions_need_fresh_execution_and_owned_read_contract_is_explicit() {
        let input = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
            .build()
            .unwrap();
        let unknown = LogicalPlan::Extension(Extension {
            node: Arc::new(Cache {
                input,
                identity: Arc::new(()),
                prepared: None,
                optimizer_leaf: false,
                admission: None,
                binding: None,
            }),
        });
        assert!(fresh([&unknown]).unwrap());
        let contracted = crate::session::contract::ExecutionContract::plan(
            unknown.clone(),
            None,
            [pse_schema::model::provider::OperationEffect::Read]
                .into_iter()
                .collect(),
        );
        assert!(!fresh([&contracted]).unwrap());
        assert!(
            fresh([&contracted, &unknown]).unwrap(),
            "a sibling contract must not qualify an uncontracted root"
        );

        let observed = crate::session::contract::ExecutionContract::plan(
            unknown,
            None,
            [pse_schema::model::provider::OperationEffect::Observe]
                .into_iter()
                .collect(),
        );
        assert!(fresh([&observed]).unwrap());
    }
}
impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
            && self.optimizer_leaf == other.optimizer_leaf
            && (self.optimizer_leaf || self.input == other.input)
    }
}
impl Eq for Cache {}
impl PartialOrd for Cache {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.identity).cmp(&Arc::as_ptr(&other.identity)) {
            std::cmp::Ordering::Equal => match self.optimizer_leaf.cmp(&other.optimizer_leaf) {
                std::cmp::Ordering::Equal if self.optimizer_leaf => Some(std::cmp::Ordering::Equal),
                std::cmp::Ordering::Equal => self.input.partial_cmp(&other.input),
                order => Some(order),
            },
            order => Some(order),
        }
    }
}
impl Hash for Cache {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.identity).hash(state);
        self.optimizer_leaf.hash(state);
        if !self.optimizer_leaf {
            self.input.hash(state);
        }
    }
}
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl UserDefinedLogicalNodeCore for Cache {
    fn name(&self) -> &'static str {
        "NativeCache"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        if self.prepared.is_some() || self.optimizer_leaf {
            vec![]
        } else {
            vec![&self.input]
        }
    }
    fn schema(&self) -> &DFSchemaRef {
        self.input.schema()
    }
    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeCache: invocation-local, spillable, deferred")
    }
    fn with_exprs_and_inputs(
        &self,
        exprs: Vec<Expr>,
        mut inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if (self.prepared.is_some() || self.optimizer_leaf) && exprs.is_empty() && inputs.is_empty()
        {
            return Ok(self.clone());
        }
        if !exprs.is_empty() || inputs.len() != 1 {
            return Err(invalid("cache needs one input"));
        }
        let input = inputs.pop().ok_or_else(|| invalid("cache input absent"))?;
        Ok(Self {
            input,
            // A rewritten computation cannot share completion with its predecessor.
            // Native planning-only leaves above preserve the exact original owner.
            identity: Arc::new(()),
            prepared: None,
            optimizer_leaf: false,
            admission: None,
            binding: None,
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.input
            .schema()
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect()
    }
}

type CachedFuture = futures_util::future::Shared<
    futures_util::future::BoxFuture<
        'static,
        std::result::Result<Arc<Cached>, Arc<DataFusionError>>,
    >,
>;
type Cell = Arc<Completion>;
#[derive(Default)]
struct Completion(
    Mutex<Option<(usize, CachedFuture)>>,
    Arc<std::sync::atomic::AtomicUsize>,
);
impl std::fmt::Debug for Completion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CacheCompletion")
    }
}
type CacheEntry = (
    Arc<()>,
    std::sync::Weak<Completion>,
    Option<std::sync::Weak<dyn ExecutionPlan>>,
);
#[derive(Debug, Default)]
pub(super) struct CacheStore(
    Mutex<HashMap<usize, CacheEntry>>,
    Arc<std::sync::atomic::AtomicUsize>,
);
impl CacheStore {
    pub(super) fn advance_epoch(&self) {
        self.1.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    fn cell(&self, identity: &Arc<()>) -> Result<Cell> {
        let mut cells = self
            .0
            .lock()
            .map_err(|_| invalid("cache owner lock poisoned"))?;
        let (_, weak, _) = cells
            .entry(Arc::as_ptr(identity) as usize)
            .or_insert_with(|| (Arc::clone(identity), std::sync::Weak::new(), None));
        if let Some(cell) = weak.upgrade() {
            return Ok(cell);
        }
        let cell = Arc::new(Completion(Mutex::new(None), Arc::clone(&self.1)));
        *weak = Arc::downgrade(&cell);
        Ok(cell)
    }
}

/// Fold only completed, successful cells from the actual invocation into their
/// native leaf readers. Logical admission and its complete source
/// graph run before this planning-only substitution; fresh operations have no cells.
pub(super) fn reuse_completed(plan: &LogicalPlan, session: &dyn Session) -> Result<LogicalPlan> {
    let Some(services) = session
        .as_any()
        .downcast_ref::<SessionState>()
        .and_then(|state| {
            state
                .config()
                .get_extension::<super::execution::NativeExecutionContext>()
        })
    else {
        return Ok(plan.clone());
    };
    let mut reservation = services
        .reserver()
        .open("session:cache-physical-substitution");
    substitute_completed(
        plan.clone(),
        &services,
        &mut HashMap::new(),
        reservation.as_mut(),
    )
}

type Substitutions = HashMap<usize, (Arc<dyn UserDefinedLogicalNode>, LogicalPlan)>;
fn substitute_completed(
    plan: LogicalPlan,
    services: &super::execution::NativeExecutionContext,
    substitutions: &mut Substitutions,
    reservation: &mut dyn pse_ids::Reservation,
) -> Result<LogicalPlan> {
    Ok(plan
        .transform_down(|node| {
            services.cancellation().checkpoint().map_err(|error| {
                DataFusionError::External(Box::new(crate::CatalogError::from(error)))
            })?;
            let LogicalPlan::Extension(extension) = &node else {
                return Ok(Transformed::no(node));
            };
            let Some(cache) = extension.node.as_any().downcast_ref::<Cache>() else {
                return Ok(Transformed::no(node));
            };
            let key = Arc::as_ptr(&extension.node).cast::<()>() as usize;
            if let Some((_, substituted)) = substitutions.get(&key) {
                return Ok(Transformed::new(
                    substituted.clone(),
                    true,
                    TreeNodeRecursion::Jump,
                ));
            }
            reservation
                .try_grow(size_of::<LogicalPlan>() + 128)
                .map_err(|error| {
                    DataFusionError::External(Box::new(crate::CatalogError::from(error)))
                })?;
            let prepared = completed_reader(cache, services)?;
            let input = if prepared.is_some() {
                cache.input.clone()
            } else {
                substitute_completed(cache.input.clone(), services, substitutions, reservation)?
            };
            let substituted = LogicalPlan::Extension(Extension {
                node: Arc::new(Cache {
                    input,
                    prepared,
                    admission: None,
                    ..cache.clone()
                }),
            });
            substitutions.insert(key, (Arc::clone(&extension.node), substituted.clone()));
            Ok(Transformed::new(substituted, true, TreeNodeRecursion::Jump))
        })?
        .data)
}

fn completed_reader(
    cache: &Cache,
    services: &super::execution::NativeExecutionContext,
) -> Result<Option<Arc<dyn ExecutionPlan>>> {
    let owners = services
        .caches
        .0
        .lock()
        .map_err(|_| invalid("cache owner lock poisoned"))?;
    let Some((_, weak, Some(physical))) = owners.get(&(Arc::as_ptr(&cache.identity) as usize))
    else {
        return Ok(None);
    };
    let (Some(cell), Some(physical)) = (weak.upgrade(), physical.upgrade()) else {
        return Ok(None);
    };
    let completed = cell
        .0
        .lock()
        .map_err(|_| invalid("cache completion lock poisoned"))?;
    let Some((epoch, future)) = completed.as_ref() else {
        return Ok(None);
    };
    let Some(Ok(value)) = future.peek() else {
        return Ok(None);
    };
    if *epoch != cell.1.load(std::sync::atomic::Ordering::Acquire)
        || physical.schema().as_ref() != cache.input.schema().as_arrow()
    {
        return Ok(None);
    }
    Ok(Some(Arc::new(StreamingTableExec::try_new(
        physical.schema(),
        vec![Arc::new(CompletedPartition {
            value: Arc::clone(value),
            epoch: *epoch,
            current: Arc::clone(&cell.1),
        })],
        None,
        [],
        false,
        None,
    )?)))
}

/// A completed producer is a source for sibling consumers. Its native reader
/// retains the memory/spill owner without exposing the already executed producer
/// tree to physical optimization again.
#[derive(Debug)]
struct CompletedPartition {
    value: Arc<Cached>,
    epoch: usize,
    current: Arc<std::sync::atomic::AtomicUsize>,
}
impl PartitionStream for CompletedPartition {
    fn schema(&self) -> &datafusion::arrow::datatypes::SchemaRef {
        self.value.manager.schema()
    }
    fn execute(&self, _: Arc<TaskContext>) -> SendableRecordBatchStream {
        let stream = if self.epoch == self.current.load(std::sync::atomic::Ordering::Acquire) {
            self.value.stream()
        } else {
            Err(invalid(
                "completed cache reader belongs to an earlier execution epoch",
            ))
        };
        stream.unwrap_or_else(|error| {
            Box::pin(RecordBatchStreamAdapter::new(
                Arc::clone(self.schema()),
                futures_util::stream::once(async { Err(error) }),
            ))
        })
    }
}

/// Observe the actual optimized owners. Weak references cannot retain a runtime
/// cycle or keep a producer alive after its completed output owners are released.
pub(super) fn record_physical(plan: &Arc<dyn ExecutionPlan>, session: &dyn Session) -> Result<()> {
    let Some(services) = session
        .as_any()
        .downcast_ref::<SessionState>()
        .and_then(|state| {
            state
                .config()
                .get_extension::<super::execution::NativeExecutionContext>()
        })
    else {
        return Ok(());
    };
    let mut visited = std::collections::HashSet::new();
    plan.apply(|node| {
        // The retained root pins every reachable physical owner for this walk.
        if !visited.insert(Arc::as_ptr(node).cast::<()>() as usize) {
            return Ok(TreeNodeRecursion::Jump);
        }
        if let Some(cache) = node.downcast_ref::<CacheExec>() {
            let mut owners = services
                .caches
                .0
                .lock()
                .map_err(|_| invalid("cache owner lock poisoned"))?;
            for (_, completion, physical) in owners.values_mut() {
                if completion.as_ptr() == Arc::as_ptr(&cache.completion)
                    && physical
                        .as_ref()
                        .is_none_or(|owner| owner.upgrade().is_none())
                {
                    *physical = Some(Arc::downgrade(node));
                }
            }
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct CachePlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for CachePlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<Cache>() else {
            return Ok(None);
        };
        if let Some(prepared) = &node.prepared {
            if !inputs.is_empty() {
                return Err(invalid("completed native cache has no replanned inputs"));
            }
            return Ok(Some(Arc::clone(prepared)));
        }
        let [input] = inputs else {
            return Err(invalid("cache requires one physical input"));
        };
        if !matches!(input.properties().boundedness, Boundedness::Bounded) {
            return Err(invalid("materialized cache needs a bounded input"));
        }
        let services = super::execution::NativeExecutionContext::from_session(session)?;
        Ok(Some(Arc::new(CacheExec::new(
            Arc::clone(input),
            services.caches.cell(&node.identity)?,
        ))))
    }
}

pub(crate) struct Cached {
    batches: Vec<RecordBatch>,
    reservation: MemoryReservation,
    spill: Option<Arc<dyn datafusion::execution::spill_file::SpillFile>>,
    manager: SpillManager,
    live: Option<(
        Arc<std::sync::atomic::AtomicUsize>,
        Arc<std::sync::atomic::AtomicUsize>,
    )>,
    readers: std::sync::atomic::AtomicUsize,
}
impl std::fmt::Debug for Cached {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cached")
            .field("batches", &self.batches.len())
            .field("reserved", &self.reservation.size())
            .field("spilled", &self.spill.is_some())
            .finish_non_exhaustive()
    }
}
impl Cached {
    async fn collect(
        input: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        metrics: &ExecutionPlanMetricsSet,
    ) -> Result<Self> {
        Self::collect_bounded(input, context, metrics, usize::MAX).await
    }
    pub(crate) async fn collect_bounded(
        input: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        metrics: &ExecutionPlanMetricsSet,
        limit: usize,
    ) -> Result<Self> {
        let runtime = context.runtime_env();
        let reservation = MemoryConsumer::new("NativeCache")
            .with_can_spill(true)
            .register(&runtime.memory_pool);
        let manager = SpillManager::new(runtime, SpillMetrics::new(metrics, 0), input.schema());
        let mut source = execute_stream(input, context)?;
        let mut batches = Vec::new();
        let mut spill = None;
        let mut allocations = pse_ids::owned_buffer::RetainedBuffers::default();
        while let Some(batch) = source.try_next().await? {
            let bytes = if spill.is_none() {
                allocations
                    .additional(&batch)
                    .map_err(|error| DataFusionError::External(Box::new(error)))?
                    .saturating_add(size_of::<RecordBatch>() + 256)
            } else {
                0
            };
            if spill.is_none()
                && reservation
                    .size()
                    .checked_add(bytes)
                    .is_some_and(|total| total <= limit)
                && reservation.try_grow(bytes).is_ok()
            {
                batches.push(batch);
            } else {
                if spill.is_none() {
                    let mut writer = manager.create_in_progress_file("native invocation cache")?;
                    for retained in &batches {
                        writer.append_batch(retained)?;
                    }
                    batches.clear();
                    allocations = pse_ids::owned_buffer::RetainedBuffers::default();
                    reservation.free();
                    spill = Some(writer);
                }
                if let Some(writer) = spill.as_mut() {
                    writer.append_batch(&batch)?;
                }
            }
        }
        let spill = match spill {
            Some(mut writer) => writer.finish()?,
            None => None,
        };
        Ok(Self {
            batches,
            reservation,
            spill,
            manager,
            live: None,
            readers: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    pub(crate) fn account(
        mut self,
        live: Arc<std::sync::atomic::AtomicUsize>,
        pinned: Arc<std::sync::atomic::AtomicUsize>,
    ) -> Self {
        live.fetch_add(self.reservation.size(), std::sync::atomic::Ordering::AcqRel);
        self.live = Some((live, pinned));
        self
    }
    pub(crate) fn reserve_identity(&self, bytes: usize) -> bool {
        self.reservation.try_grow(bytes).is_ok()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.reservation.size()
    }
    pub(crate) fn resident(&self) -> bool {
        self.spill.is_none()
    }
    pub(crate) fn stream(self: &Arc<Self>) -> Result<SendableRecordBatchStream> {
        // A stream retains the cache's reservation even if its physical parent is dropped.
        let owner = Arc::new(CachePin::new(Arc::clone(self)));
        let stream: SendableRecordBatchStream = match &self.spill {
            Some(file) => self
                .manager
                .read_spill_as_stream_unbuffered(Arc::clone(file), None)?,
            None => Box::pin(RecordBatchStreamAdapter::new(
                self.manager.schema().clone(),
                futures_util::stream::iter(self.batches.clone().into_iter().map(Ok)),
            )),
        };
        let schema = stream.schema();
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            schema,
            stream.and_then(move |batch| {
                let owner = owner.clone();
                async move {
                    pse_ids::owned_buffer::retain_owner(batch, owner)
                        .map_err(|error| DataFusionError::External(Box::new(error)))
                }
            }),
        )))
    }
}

impl Drop for Cached {
    fn drop(&mut self) {
        if let Some((live, _)) = &self.live {
            live.fetch_sub(self.reservation.size(), std::sync::atomic::Ordering::AcqRel);
        }
    }
}
struct CachePin(Arc<Cached>);
impl CachePin {
    fn new(value: Arc<Cached>) -> Self {
        if value
            .readers
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            == 0
            && let Some((_, pinned)) = &value.live
        {
            pinned.fetch_add(
                value.reservation.size(),
                std::sync::atomic::Ordering::AcqRel,
            );
        }
        Self(value)
    }
}
impl Drop for CachePin {
    fn drop(&mut self) {
        if self
            .0
            .readers
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel)
            == 1
            && let Some((_, pinned)) = &self.0.live
        {
            pinned.fetch_sub(
                self.0.reservation.size(),
                std::sync::atomic::Ordering::AcqRel,
            );
        }
    }
}

struct CacheExec {
    input: Arc<dyn ExecutionPlan>,
    completion: Cell,
    properties: Arc<PlanProperties>,
    metrics: ExecutionPlanMetricsSet,
    planned_boundary: bool,
}
impl std::fmt::Debug for CacheExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_as(DisplayFormatType::Default, f)
    }
}
impl CacheExec {
    fn new(input: Arc<dyn ExecutionPlan>, completion: Cell) -> Self {
        Self {
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(input.schema()),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
            input,
            completion,
            metrics: ExecutionPlanMetricsSet::new(),
            planned_boundary: false,
        }
    }
}
impl DisplayAs for CacheExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NativeCacheExec: shared completion, native pool/spill, independently_planned={}",
            self.planned_boundary
        )
    }
}
impl ExecutionPlan for CacheExec {
    fn name(&self) -> &'static str {
        "NativeCacheExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        if self.planned_boundary {
            vec![]
        } else {
            vec![&self.input]
        }
    }
    fn with_new_children(
        self: Arc<Self>,
        mut children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if self.planned_boundary {
            if !children.is_empty() {
                return Err(invalid(
                    "independently planned cache has no optimizer inputs",
                ));
            }
            return Ok(self);
        }
        if children.len() != 1 {
            return Err(invalid("cache requires one child"));
        }
        let input = children
            .pop()
            .ok_or_else(|| invalid("cache child absent"))?;
        Ok(Arc::new(Self::new(input, Arc::clone(&self.completion))))
    }
    fn reset_state(self: Arc<Self>) -> Result<Arc<dyn ExecutionPlan>> {
        // An independently planned producer is a separate native execution graph,
        // not an optimizer child. Reset it explicitly through DataFusion's API.
        let input = if self.planned_boundary {
            datafusion::physical_plan::execution_plan::reset_plan_states(Arc::clone(&self.input))?
        } else {
            Arc::clone(&self.input)
        };
        let mut reset = Self::new(input, Arc::clone(&self.completion));
        reset.planned_boundary = self.planned_boundary;
        Ok(Arc::new(reset))
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn metrics(&self) -> Option<datafusion::physical_plan::metrics::MetricsSet> {
        Some(self.metrics.clone_inner())
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(invalid("cache has one output partition"));
        }
        let input = Arc::clone(&self.input);
        let metrics = self.metrics.clone();
        let cached = {
            let mut completion = self
                .completion
                .0
                .lock()
                .map_err(|_| invalid("cache completion lock poisoned"))?;
            let epoch = self.completion.1.load(std::sync::atomic::Ordering::Acquire);
            if completion
                .as_ref()
                .is_some_and(|(recorded, _)| *recorded != epoch)
            {
                *completion = None;
            }
            completion
                .get_or_insert_with(|| {
                    let future = async move {
                        Cached::collect(input, context, &metrics)
                            .await
                            .map(Arc::new)
                            .map_err(Arc::new)
                    }
                    .boxed()
                    .shared();
                    (epoch, future)
                })
                .1
                .clone()
        };
        let stream = futures_util::stream::once(async move {
            match cached.await {
                Ok(cached) => cached.stream(),
                Err(error) => Err(DataFusionError::Shared(error)),
            }
        })
        .try_flatten();
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}

#[cfg(test)]
mod tests;
