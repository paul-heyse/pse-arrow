// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked finite outputs on the existing native operation/completion seam.
//! The invocation owns execution; each escaped chunk owns only its allocations.
use super::{Body, Definition, FailureKind, Operation, completion::Completion};
use crate::session::execution::NativeExecutionContext;
use datafusion::{
    arrow::datatypes::SchemaRef,
    common::{DataFusionError, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, stream::RecordBatchStreamAdapter},
};
use futures_util::{FutureExt, TryStreamExt};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::provider::OperationEffect;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

/// Every port contains locally checked chunks, including an explicit empty vector.
pub type Outputs = BTreeMap<String, Vec<FieldCheckedBatch>>;
/// Which outputs must settle together before any selected value becomes readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionMode {
    /// All declared outputs form one valid finite result.
    Atomic,
    /// The body can compute just the explicitly selected output set.
    Independent,
}
/// Actual finite algorithm owner; names alone never identify a producer.
#[async_trait::async_trait]
pub trait Producer: std::fmt::Debug + Send + Sync + 'static {
    /// Registry declarations for the complete output inventory.
    fn declarations(&self) -> BTreeMap<String, pse_ids::SemanticId>;
    /// Whether outputs have indivisible validity or explicit independent demand.
    fn completion_mode(&self) -> CompletionMode;
    /// Effects admitted by the enclosing native operation.
    fn effects(&self) -> BTreeSet<OperationEffect>;
    /// Prepare the selected logical inputs without executing any rows.
    async fn prepare(
        self: Arc<Self>,
        inputs: &[LogicalPlan],
        state: &SessionState,
    ) -> Result<Arc<dyn ProducerBody>>;
}
/// Prepared finite work. DataFusion schedules its visible input children.
pub trait ProducerBody: std::fmt::Debug + Send + Sync {
    /// Replace physical inputs without preserving source-specific receipts.
    /// # Errors
    /// A producer cannot rebind its prepared domain state safely.
    fn rebind(&self) -> Result<Arc<dyn ProducerBody>>;
    /// Compute the complete demanded inventory. No partial result is published.
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        demand: BTreeSet<String>,
        context: Arc<TaskContext>,
    ) -> futures_util::future::BoxFuture<'static, Result<Outputs>>;
}

/// Bind composable relation outputs. This function does not execute the producer.
/// # Errors
/// An unknown output, incompatible registry declaration or invalid native schema.
pub fn plans(
    producer: Arc<dyn Producer>,
    inputs: Vec<LogicalPlan>,
    selected: BTreeSet<String>,
    registry: &pse_schema::Registry,
) -> Result<BTreeMap<String, LogicalPlan>> {
    let declarations = producer.declarations();
    if selected.is_empty() || selected.iter().any(|name| !declarations.contains_key(name)) {
        return Err(invalid("finite output demand is empty or undeclared"));
    }
    let demand = match producer.completion_mode() {
        CompletionMode::Atomic => declarations.keys().cloned().collect(),
        CompletionMode::Independent => selected.clone(),
    };
    let contracts = declarations
        .into_iter()
        .map(|(name, relation)| {
            let spec = registry
                .relation_by_id(relation)
                .ok_or_else(|| invalid("finite output relation is undeclared"))?;
            let contract = registry.contract(spec).map_err(pse_columnar::external)?;
            Ok((name, contract))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let contracts = Arc::new(contracts);
    selected
        .into_iter()
        .map(move |name| {
            let schema = contracts[&name].schema().clone();
            let definition = Arc::new(Port {
                producer: producer.clone(),
                name: name.clone(),
                schema,
                demand: demand.clone(),
                contracts: contracts.clone(),
            });
            Ok((name, Operation::plan(definition, inputs.clone())?))
        })
        .collect()
}

type Contracts = BTreeMap<String, pse_schema::resolved_contract::RelationContractHandle>;
#[derive(Debug)]
struct Port {
    producer: Arc<dyn Producer>,
    name: String,
    schema: SchemaRef,
    demand: BTreeSet<String>,
    contracts: Arc<Contracts>,
}
#[async_trait::async_trait]
impl Definition for Port {
    fn name(&self) -> &'static str {
        "CheckedFinitePort"
    }
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    fn effects(&self) -> BTreeSet<OperationEffect> {
        self.producer.effects()
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        logical: &[LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        let services = NativeExecutionContext::from_session(state)?;
        if let Some(bound) = services
            .port_store()
            .find(&self.producer, logical, &self.demand)?
        {
            return Ok(Arc::new(Reader { port: self, bound }));
        }
        let preparing = services
            .port_store()
            .preparation(&self.producer, logical, &self.demand)?;
        let producer = self.producer.clone();
        let logical_inputs = logical.to_vec();
        let selected_state = state.clone();
        let body = preparing
            .completion
            .get(|| {
                async move {
                    Ok(PreparedBody(
                        producer.prepare(&logical_inputs, &selected_state).await?,
                    ))
                }
                .boxed()
            })
            .await?
            .0
            .clone();
        let bound = services.port_store().insert(Arc::new(Bound {
            producer: self.producer.clone(),
            logical: logical.to_vec(),
            demand: self.demand.clone(),
            inputs: inputs.to_vec(),
            body,
            services: services.clone(),
            epoch: services.port_store().epoch.clone(),
            completion: Mutex::new(None),
        }))?;
        Ok(Arc::new(Reader { port: self, bound }))
    }
}

/// Local to the existing invocation/epoch cache owner; no independent scheduler.
#[derive(Debug, Default)]
pub(crate) struct Store {
    entries: Mutex<Vec<Weak<Bound>>>,
    preparing: Mutex<Vec<Weak<Preparing>>>,
    epoch: Arc<AtomicUsize>,
}
#[derive(Debug)]
struct PreparedBody(Arc<dyn ProducerBody>);
#[derive(Debug)]
struct Preparing {
    producer: Arc<dyn Producer>,
    logical: Vec<LogicalPlan>,
    demand: BTreeSet<String>,
    completion: Arc<Completion<PreparedBody>>,
}
impl Store {
    fn preparation(
        &self,
        producer: &Arc<dyn Producer>,
        logical: &[LogicalPlan],
        demand: &BTreeSet<String>,
    ) -> Result<Arc<Preparing>> {
        let mut entries = self
            .preparing
            .lock()
            .map_err(|_| invalid("finite preparation lock poisoned"))?;
        entries.retain(|entry| entry.strong_count() != 0);
        if let Some(entry) = entries.iter().filter_map(Weak::upgrade).find(|entry| {
            Arc::ptr_eq(&entry.producer, producer)
                && same_inputs(&entry.logical, logical)
                && entry.demand == *demand
        }) {
            return Ok(entry);
        }
        let entry = Arc::new(Preparing {
            producer: producer.clone(),
            logical: logical.to_vec(),
            demand: demand.clone(),
            completion: Arc::default(),
        });
        entries.push(Arc::downgrade(&entry));
        Ok(entry)
    }
    pub(crate) fn advance_epoch(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
    }
    fn find(
        &self,
        producer: &Arc<dyn Producer>,
        logical: &[LogicalPlan],
        demand: &BTreeSet<String>,
    ) -> Result<Option<Arc<Bound>>> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| invalid("finite producer lock poisoned"))?;
        entries.retain(|entry| entry.strong_count() != 0);
        Ok(entries.iter().filter_map(Weak::upgrade).find(|bound| {
            Arc::ptr_eq(&bound.producer, producer)
                && same_inputs(&bound.logical, logical)
                && bound.demand == *demand
        }))
    }
    fn insert(&self, bound: Arc<Bound>) -> Result<Arc<Bound>> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| invalid("finite producer lock poisoned"))?;
        if let Some(existing) = entries.iter().filter_map(Weak::upgrade).find(|existing| {
            Arc::ptr_eq(&existing.producer, &bound.producer)
                && same_inputs(&existing.logical, &bound.logical)
                && existing.demand == bound.demand
        }) {
            return Ok(existing);
        }
        entries.push(Arc::downgrade(&bound));
        Ok(bound)
    }
}
type EpochCompletion = Option<(usize, Arc<Completion<Outputs>>)>;
#[derive(Debug)]
struct Bound {
    producer: Arc<dyn Producer>,
    logical: Vec<LogicalPlan>,
    demand: BTreeSet<String>,
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    body: Arc<dyn ProducerBody>,
    services: Arc<NativeExecutionContext>,
    epoch: Arc<AtomicUsize>,
    completion: Mutex<EpochCompletion>,
}
impl Bound {
    fn cell(&self) -> Result<Arc<Completion<Outputs>>> {
        let epoch = self.epoch.load(Ordering::Acquire);
        let mut slot = self
            .completion
            .lock()
            .map_err(|_| invalid("finite completion lock poisoned"))?;
        if let Some((previous, completion)) = slot.as_ref()
            && *previous == epoch
        {
            return Ok(completion.clone());
        }
        let completion = Arc::new(Completion::default());
        *slot = Some((epoch, completion.clone()));
        Ok(completion)
    }
}
#[derive(Debug)]
struct Reader {
    port: Arc<Port>,
    bound: Arc<Bound>,
}
impl Body for Reader {
    fn prepared_inputs(&self) -> Option<Vec<Arc<dyn ExecutionPlan>>> {
        Some(self.bound.inputs.clone())
    }
    fn rebind(&self, inputs: &[Arc<dyn ExecutionPlan>]) -> Result<Option<Arc<dyn Body>>> {
        if self.bound.inputs.len() == inputs.len()
            && self
                .bound
                .inputs
                .iter()
                .zip(inputs)
                .all(|(a, b)| same_physical_input(a, b))
        {
            return Ok(Some(Arc::new(Self {
                port: self.port.clone(),
                bound: self.bound.clone(),
            })));
        }
        let bound = Arc::new(Bound {
            producer: self.bound.producer.clone(),
            logical: self.bound.logical.clone(),
            demand: self.bound.demand.clone(),
            inputs: inputs.to_vec(),
            body: self.bound.body.rebind()?,
            services: self.bound.services.clone(),
            epoch: self.bound.epoch.clone(),
            completion: Mutex::new(None),
        });
        Ok(Some(Arc::new(Self {
            port: self.port.clone(),
            bound,
        })))
    }
    fn checked_output(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        context: Arc<TaskContext>,
    ) -> Option<futures_util::future::BoxFuture<'static, Result<Vec<FieldCheckedBatch>>>> {
        Some(self.chunks(inputs, context))
    }
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let future = self.chunks(inputs, context);
        let stream = futures_util::stream::once(async move {
            Ok::<_, DataFusionError>(futures_util::stream::iter(
                future
                    .await?
                    .into_iter()
                    .map(|chunk| Ok(chunk.into_batch())),
            ))
        })
        .try_flatten();
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.port.schema.clone(),
            stream,
        )))
    }
}
impl Reader {
    fn chunks(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        context: Arc<TaskContext>,
    ) -> futures_util::future::BoxFuture<'static, Result<Vec<FieldCheckedBatch>>> {
        let bound = self.bound.clone();
        let cell = bound.cell();
        let port = self.port.clone();
        let name = port.name.clone();
        async move {
            let cell = cell?;
            tracing::debug!(target: "pse::producer", port = %name, "producer output requested");
            let result = cell
                .get(|| {
                    async move {
                        tracing::debug!(target: "pse::producer", demanded_outputs = bound.demand.len(), "producer started");
                        bound
                            .services
                            .cancellation()
                            .checkpoint()
                            .map_err(pse_columnar::external)?;
                        let mut outputs = bound
                            .body
                            .execute(inputs, bound.demand.clone(), context)
                            .await?;
                        if outputs.keys().cloned().collect::<BTreeSet<_>>() != bound.demand {
                            return Err(FailureKind::Invalid
                                .error(invalid("finite output inventory differs from demand")));
                        }
                        // Admit all outputs before any port is readable. A field contract
                        // remains distinct from global keys and publication obligations.
                        for (name, chunks) in &mut outputs {
                            let contract = &port.contracts[name];
                            let registry = bound.services.registry();
                            registry
                                .admit_contract(contract)
                                .map_err(pse_columnar::external)?;
                            let spec = registry
                                .relation_by_id(contract.relation_id())
                                .ok_or_else(|| invalid("finite contract absent"))?;
                            for chunk in chunks {
                                chunk.check_declaration(registry, spec).map_err(|error| {
                                    FailureKind::Invalid.error(pse_columnar::external(error))
                                })?;
                                *chunk = chunk
                                    .retained(bound.services.pool(), bound.services.cancellation())
                                    .map_err(pse_columnar::external)?;
                                if let Some(owned) = chunk.owned() {
                                    bound
                                        .services
                                        .ownership()
                                        .import(owned)
                                        .map_err(pse_columnar::external)?;
                                }
                            }
                        }
                        bound
                            .services
                            .cancellation()
                            .checkpoint()
                            .map_err(pse_columnar::external)?;
                        tracing::debug!(target: "pse::producer", completed_outputs = outputs.len(), batches = outputs.values().map(Vec::len).sum::<usize>(), rows = outputs.values().flatten().map(|chunk| chunk.batch().num_rows()).sum::<usize>(), "producer completed");
                        Ok(outputs)
                    }
                    .boxed()
                })
                .await?;
            let chunks = result
                .get(&name)
                .ok_or_else(|| invalid("finite selected output absent"))?;
            Ok(chunks.clone())
        }
        .boxed()
    }
}
pub(crate) fn same_input(left: &LogicalPlan, right: &LogicalPlan) -> bool {
    if std::ptr::eq(left, right)
        || matches!((left, right), (LogicalPlan::Extension(a), LogicalPlan::Extension(b))
            if Arc::ptr_eq(&a.node, &b.node))
    {
        return true;
    }
    if crate::session::cache::same_producer(left, right) {
        return true;
    }
    if let (LogicalPlan::Extension(a), LogicalPlan::Extension(b)) = (left, right)
        && let (Some(a), Some(b)) = (
            a.node
                .as_any()
                .downcast_ref::<crate::session::contract::ExecutionContract>(),
            b.node
                .as_any()
                .downcast_ref::<crate::session::contract::ExecutionContract>(),
        )
    {
        return a.same_binding(b);
    }
    if let (LogicalPlan::Extension(a), LogicalPlan::Extension(b)) = (left, right) {
        // Unknown extension equality need not include its actual implementation
        // or dependencies. Only the same retained object establishes that fact.
        return match (
            a.node.as_any().downcast_ref::<Operation>(),
            b.node.as_any().downcast_ref::<Operation>(),
        ) {
            (Some(a), Some(b)) => a.same_binding(b),
            _ => false,
        };
    }
    if std::mem::discriminant(left) != std::mem::discriminant(right) {
        return false;
    }
    if let (LogicalPlan::TableScan(a), LogicalPlan::TableScan(b)) = (left, right)
        && !Arc::ptr_eq(&a.source, &b.source)
        && !matches!(
            (datafusion::datasource::source_as_provider(&a.source), datafusion::datasource::source_as_provider(&b.source)),
            (Ok(a), Ok(b)) if Arc::ptr_eq(&a, &b)
        )
    {
        return false;
    }
    if let (LogicalPlan::Dml(a), LogicalPlan::Dml(b)) = (left, right)
        && !Arc::ptr_eq(&a.target, &b.target)
    {
        return false;
    }
    let a = left.inputs();
    let b = right.inputs();
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| same_input(a, b))
        && same_expressions(&left.expressions(), &right.expressions())
        && left == right
}
pub(super) fn same_inputs(left: &[LogicalPlan], right: &[LogicalPlan]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| same_input(a, b))
}
pub(super) fn same_expressions(left: &[Expr], right: &[Expr]) -> bool {
    use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion};
    use datafusion::logical_expr::WindowFunctionDefinition;
    // Native expression equality can compare functions by name/signature. Reuse
    // additionally requires the actual immutable implementations. A reconstructed
    // subquery without the same owner conservatively acquires a fresh producer.
    let owners = |values: &[Expr]| {
        let mut owners = Vec::new();
        for expression in values {
            expression
                .apply(|node| {
                    let owner = match node {
                        Expr::ScalarFunction(call) => {
                            Some(Arc::as_ptr(call.func.inner()).cast::<()>() as usize)
                        }
                        Expr::AggregateFunction(call) => {
                            Some(Arc::as_ptr(call.func.inner()).cast::<()>() as usize)
                        }
                        Expr::HigherOrderFunction(call) => {
                            Some(Arc::as_ptr(call.func.inner()).cast::<()>() as usize)
                        }
                        Expr::WindowFunction(call) => Some(match &call.fun {
                            WindowFunctionDefinition::AggregateUDF(function) => {
                                Arc::as_ptr(function.inner()).cast::<()>() as usize
                            }
                            WindowFunctionDefinition::WindowUDF(function) => {
                                Arc::as_ptr(function.inner()).cast::<()>() as usize
                            }
                        }),
                        Expr::ScalarSubquery(query) => Some(Arc::as_ptr(&query.subquery) as usize),
                        Expr::Exists(query) => Some(Arc::as_ptr(&query.subquery.subquery) as usize),
                        Expr::InSubquery(query) => {
                            Some(Arc::as_ptr(&query.subquery.subquery) as usize)
                        }
                        Expr::SetComparison(query) => {
                            Some(Arc::as_ptr(&query.subquery.subquery) as usize)
                        }
                        _ => None,
                    };
                    if let Some(owner) = owner {
                        owners.push(owner);
                    }
                    Ok(TreeNodeRecursion::Continue)
                })
                .ok()?;
        }
        Some(owners)
    };
    left == right && owners(left).zip(owners(right)).is_some_and(|(a, b)| a == b)
}
/// Whether both children retain the actual same producer. Native cooperative
/// scheduling shells are transparent; qualified cache epochs retain their own
/// exact owner check. Arbitrary equal-looking rewrites never establish identity.
pub fn same_physical_input(a: &Arc<dyn ExecutionPlan>, b: &Arc<dyn ExecutionPlan>) -> bool {
    let a = cooperative_input(a);
    let b = cooperative_input(b);
    Arc::ptr_eq(a, b) || crate::session::cache::same_execution(a, b)
}
fn cooperative_input(mut plan: &Arc<dyn ExecutionPlan>) -> &Arc<dyn ExecutionPlan> {
    while let Some(cooperative) =
        plan.downcast_ref::<datafusion::physical_plan::coop::CooperativeExec>()
    {
        plan = cooperative.input();
    }
    plan
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}

/// Exact transparent native shells can carry checked chunks without readmission.
/// Any relational rewrite falls back to ordinary field admission at the new boundary.
pub(crate) fn capture(
    plan: &Arc<dyn ExecutionPlan>,
    context: Arc<TaskContext>,
) -> Option<futures_util::future::BoxFuture<'static, Result<Vec<FieldCheckedBatch>>>> {
    let plan = cooperative_input(plan);
    if let Some(operation) = plan.downcast_ref::<super::Execution>() {
        return operation.checked_output(context);
    }
    if let Some(contract) = plan.downcast_ref::<crate::session::contract::ContractExec>() {
        return contract.checked_output(context);
    }
    None
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use crate::session::{EngineFactory, EngineSession};
    use datafusion::{
        execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
        logical_expr::LogicalPlanBuilder,
        physical_plan::collect,
    };
    use pse_columnar::CancellationToken;
    use pse_relations::{
        columnar::RelationRow,
        generated::reference::{elements, math_context},
    };

    #[test]
    fn logical_reconstruction_requires_actual_provider_and_function_owners() {
        use datafusion::{
            arrow::datatypes::{DataType, Schema},
            common::ScalarValue,
            datasource::{MemTable, provider_as_source},
            logical_expr::{ColumnarValue, Volatility, create_udf},
        };
        let provider = || -> Arc<dyn datafusion::catalog::TableProvider> {
            Arc::new(MemTable::try_new(Arc::new(Schema::empty()), vec![vec![]]).unwrap())
        };
        let scan = |source| {
            LogicalPlanBuilder::scan("same_name", provider_as_source(source), None)
                .unwrap()
                .build()
                .unwrap()
        };
        let owner = provider();
        let left = scan(owner.clone());
        let same = scan(owner);
        let replaced = scan(provider());
        assert_eq!(
            left, replaced,
            "native scan equality deliberately omits the provider"
        );
        assert!(same_input(&left, &same));
        assert!(!same_input(&left, &replaced));
        let function = |value| {
            Arc::new(create_udf(
                "same_function",
                vec![],
                DataType::Int64,
                Volatility::Immutable,
                Arc::new(move |_| Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(value))))),
            ))
        };
        let first = function(1);
        let project = |function: Arc<datafusion::logical_expr::ScalarUDF>| {
            LogicalPlanBuilder::from(left.clone())
                .project([function.call(vec![]).alias("n")])
                .unwrap()
                .build()
                .unwrap()
        };
        let original = project(first.clone());
        assert!(same_input(&original, &project(first)));
        assert!(!same_input(&original, &project(function(2))));
    }

    #[derive(Clone, Debug)]
    struct Fake {
        registry: Arc<pse_schema::Registry>,
        calls: Arc<AtomicUsize>,
        preparations: Arc<AtomicUsize>,
        mode: CompletionMode,
        invalid: bool,
    }
    #[async_trait::async_trait]
    impl Producer for Fake {
        fn declarations(&self) -> BTreeMap<String, pse_ids::SemanticId> {
            BTreeMap::from([
                ("values".into(), elements::spec(&self.registry).unwrap().id),
                (
                    "empty".into(),
                    math_context::spec(&self.registry).unwrap().id,
                ),
            ])
        }
        fn completion_mode(&self) -> CompletionMode {
            self.mode
        }
        fn effects(&self) -> BTreeSet<OperationEffect> {
            BTreeSet::from([OperationEffect::Read])
        }
        async fn prepare(
            self: Arc<Self>,
            _: &[LogicalPlan],
            _: &SessionState,
        ) -> Result<Arc<dyn ProducerBody>> {
            self.preparations.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            Ok(self)
        }
    }
    impl ProducerBody for Fake {
        fn rebind(&self) -> Result<Arc<dyn ProducerBody>> {
            Ok(Arc::new(self.clone()))
        }
        fn execute(
            &self,
            inputs: Vec<Arc<dyn ExecutionPlan>>,
            demand: BTreeSet<String>,
            context: Arc<TaskContext>,
        ) -> futures_util::future::BoxFuture<'static, Result<Outputs>> {
            let registry = self.registry.clone();
            let calls = self.calls.clone();
            let invalid_output = self.invalid;
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                let mut rows = 0;
                for input in inputs {
                    rows += collect(input, context.clone())
                        .await?
                        .iter()
                        .map(datafusion::arrow::array::RecordBatch::num_rows)
                        .sum::<usize>();
                }
                let mut builder = elements::Row::builder(&registry, 1).unwrap();
                elements::Row::push(
                    &mut builder,
                    elements::Row {
                        element_id: pse_ids::SemanticId::from_bytes([1; 16]),
                        symbol: "C".into(),
                        name: "carbon".into(),
                        atomic_mass: rows as f64,
                    },
                )
                .unwrap();
                let values = elements::Row::finish(builder).unwrap();
                let empty = FieldCheckedBatch::concat(
                    &registry,
                    math_context::spec(&registry).unwrap(),
                    &[],
                )
                .unwrap();
                let mut result = BTreeMap::new();
                if demand.contains("values") {
                    result.insert("values".into(), vec![values.clone()]);
                }
                if demand.contains("empty") {
                    result.insert(
                        "empty".into(),
                        vec![if invalid_output { values } else { empty }],
                    );
                }
                Ok(result)
            }
            .boxed()
        }
    }
    fn setup(mode: CompletionMode, invalid: bool) -> (EngineSession, Arc<Fake>, CancellationToken) {
        let cancel = CancellationToken::new();
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let session = EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20)),
            "ports-unit",
            SessionStateBuilder::new_with_default_features(),
        )
        .candidate_checked(BTreeMap::new(), registry.clone(), &cancel)
        .unwrap();
        (
            session,
            Arc::new(Fake {
                registry,
                calls: Arc::new(AtomicUsize::new(0)),
                preparations: Arc::new(AtomicUsize::new(0)),
                mode,
                invalid,
            }),
            cancel,
        )
    }
    fn input(row: bool) -> LogicalPlan {
        LogicalPlanBuilder::empty(row).build().unwrap()
    }
    fn select(fake: Arc<Fake>, row: bool, names: &[&str]) -> BTreeMap<String, LogicalPlan> {
        let registry = fake.registry.clone();
        plans(
            fake,
            vec![input(row)],
            names.iter().map(|name| (*name).into()).collect(),
            &registry,
        )
        .unwrap()
    }
    fn execution_state(session: &EngineSession, cancel: &CancellationToken) -> SessionState {
        let state =
            NativeExecutionContext::bind(session, session.bound_state().unwrap(), cancel).unwrap();
        NativeExecutionContext::execution_state(&state, cancel).unwrap()
    }
    #[tokio::test]
    async fn sibling_ports_share_one_body_and_arrays_survive_the_bundle() {
        let (session, fake, cancel) = setup(CompletionMode::Atomic, false);
        let plans = select(fake.clone(), true, &["values", "empty"]);
        let state = execution_state(&session, &cancel);
        let (a, b) = futures_util::try_join!(
            state.create_physical_plan(&plans["values"]),
            state.create_physical_plan(&plans["empty"]),
        )
        .unwrap();
        assert_eq!(fake.preparations.load(Ordering::SeqCst), 1);
        assert_eq!(
            fake.calls.load(Ordering::SeqCst),
            0,
            "planning has no effects"
        );
        let (a_result, b_result) =
            futures_util::try_join!(collect(a, state.task_ctx()), collect(b, state.task_ctx()))
                .unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert_eq!(a_result[0].num_rows(), 1);
        assert_eq!(
            b_result
                .iter()
                .map(datafusion::arrow::array::RecordBatch::num_rows)
                .sum::<usize>(),
            0
        );
        let array = a_result[0].column(3).clone();
        drop((a_result, b_result, state));
        assert_eq!(array.len(), 1);
        assert!(session.pool().reserved() > 0);
        drop(array);
        assert_eq!(session.pool().reserved(), 0);
    }
    #[tokio::test]
    async fn cooperative_shell_preserves_checked_chunks_without_another_admission() {
        let (session, fake, cancel) = setup(CompletionMode::Atomic, false);
        let plans = select(fake.clone(), true, &["values"]);
        let state = execution_state(&session, &cancel);
        let plan = state.create_physical_plan(&plans["values"]).await.unwrap();
        let plan: Arc<dyn ExecutionPlan> =
            Arc::new(datafusion::physical_plan::coop::CooperativeExec::new(plan));
        let first = capture(&plan, state.task_ctx()).unwrap().await.unwrap();
        let bytes = session.pool().reserved();
        let second = capture(&plan, state.task_ctx()).unwrap().await.unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert_eq!(session.pool().reserved(), bytes);
        assert!(Arc::ptr_eq(
            first[0].batch().column(3),
            second[0].batch().column(3)
        ));
    }
    #[tokio::test]
    async fn changed_inputs_demand_and_epoch_never_reuse_a_completion() {
        let (session, fake, cancel) = setup(CompletionMode::Independent, false);
        let state = execution_state(&session, &cancel);
        let a = select(fake.clone(), false, &["values"]);
        let a = state.create_physical_plan(&a["values"]).await.unwrap();
        collect(a.clone(), state.task_ctx()).await.unwrap();
        let b = select(fake.clone(), true, &["values"]);
        let b = state.create_physical_plan(&b["values"]).await.unwrap();
        let replacement = a
            .clone()
            .replace_children(
                b.children().into_iter().cloned().collect(),
                datafusion::physical_plan::execution_plan::ReplaceChildrenOptions::new(
                    datafusion::physical_plan::execution_plan::ChildrenPropertiesMode::Recompute,
                ),
            )
            .unwrap();
        collect(replacement, state.task_ctx()).await.unwrap();
        collect(b, state.task_ctx()).await.unwrap();
        let c = select(fake.clone(), false, &["values", "empty"]);
        let c = state.create_physical_plan(&c["values"]).await.unwrap();
        collect(c, state.task_ctx()).await.unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 4);
        NativeExecutionContext::from_session(&state)
            .unwrap()
            .advance_round_epoch();
        collect(a, state.task_ctx()).await.unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 5);
    }
    #[tokio::test]
    async fn invalid_atomic_output_and_cancellation_publish_no_port() {
        let (session, fake, cancel) = setup(CompletionMode::Atomic, true);
        let plans = select(fake.clone(), true, &["values", "empty"]);
        let state = execution_state(&session, &cancel);
        let a = state.create_physical_plan(&plans["values"]).await.unwrap();
        let b = state.create_physical_plan(&plans["empty"]).await.unwrap();
        assert!(collect(a, state.task_ctx()).await.is_err());
        assert!(collect(b, state.task_ctx()).await.is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        let (session, fake, cancel) = setup(CompletionMode::Atomic, false);
        let plans = select(fake.clone(), true, &["values"]);
        let state = execution_state(&session, &cancel);
        let a = state.create_physical_plan(&plans["values"]).await.unwrap();
        cancel.cancel();
        assert!(collect(a, state.task_ctx()).await.is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
    }
}
