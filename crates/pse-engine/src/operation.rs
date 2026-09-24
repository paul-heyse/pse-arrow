// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite operations. Typed bodies own algorithms; this module owns their
//! logical reconstruction, physical transport, effects and execution lifetime.

use crate::session::execution::NativeExecutionContext;
pub mod completion;
pub use completion::{CompletionFailure, CompletionStatus, FailureKind};
pub mod ownership;
pub mod ports;
use datafusion::{
    arrow::{array::RecordBatch, datatypes::SchemaRef},
    catalog::Session,
    common::{DFSchema, DFSchemaRef, DataFusionError, Result, tree_node::TreeNodeRecursion},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        Expr, Extension, LogicalPlan, UserDefinedLogicalNode, UserDefinedLogicalNodeCore,
        physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::{EquivalenceProperties, PhysicalExpr},
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, ExecutionPlanProperties, Partitioning,
        PlanProperties, SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        metrics::{BaselineMetrics, ExecutionPlanMetricsSet},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::TryStreamExt;
use pse_schema::model::provider::OperationEffect;
use std::{
    any::Any,
    collections::BTreeSet,
    fmt,
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Execution policies are distinct: a pure finite result is cancellable, while
/// a command owns its result until cooperative settlement has completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Complete bounded inputs, no replay or exact-count promise.
    Finite,
    /// One effectful attempt, with retained settlement after consumer abandonment.
    Command,
    /// One observation of mutable state; never eligible for round reuse.
    Observation,
    /// Cooperative foreign work retains its owner until its terminal result.
    Foreign,
}

/// Domain definition retained by its actual owner, never resolved by display name.
#[async_trait::async_trait]
pub trait Definition: Any + fmt::Debug + Send + Sync {
    /// Native plan display label.
    fn name(&self) -> &'static str;
    /// Established output fields.
    fn schema(&self) -> SchemaRef;
    /// Execution lifecycle.
    fn family(&self) -> Family {
        Family::Finite
    }
    /// Actual effects admitted under the caller's policies.
    fn effects(&self) -> BTreeSet<OperationEffect> {
        BTreeSet::from([OperationEffect::Read])
    }
    /// Expressions exposed to native rewriting.
    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }
    /// Semantic input demand; unknown algorithms retain their complete input evidence.
    fn dependencies(
        &self,
        inputs: &[LogicalPlan],
        _output: &crate::session::dependencies::SemanticDemand,
    ) -> Vec<crate::session::dependencies::SemanticDemand> {
        inputs
            .iter()
            .map(crate::session::dependencies::SemanticDemand::whole)
            .collect()
    }
    /// True for a retained definition (for example CREATE VIEW), rather than an executed port.
    fn definition_only(&self, _input: usize) -> bool {
        false
    }
    /// Prepare only; this must not run the operation or mutate external state.
    async fn prepare(
        self: Arc<Self>,
        expressions: &[Expr],
        logical_inputs: &[LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>>;
}

/// Typed execution body. The shell validates children before calling this method.
pub trait Body: fmt::Debug + Send + Sync {
    /// Carry established local field evidence through a finite-to-finite handoff.
    fn checked_output(
        &self,
        _inputs: Vec<Arc<dyn ExecutionPlan>>,
        _context: Arc<TaskContext>,
    ) -> Option<
        futures_util::future::BoxFuture<
            'static,
            Result<Vec<pse_relations::columnar::FieldCheckedBatch>>,
        >,
    > {
        None
    }
    /// Canonical children prepared once for sibling ports of a finite producer.
    fn prepared_inputs(&self) -> Option<Vec<Arc<dyn ExecutionPlan>>> {
        None
    }
    /// Rebind stateful completion after native children change.
    /// # Errors
    /// Changed input contracts or resource refusal.
    fn rebind(&self, _inputs: &[Arc<dyn ExecutionPlan>]) -> Result<Option<Arc<dyn Body>>> {
        Ok(None)
    }
    /// Start the actual native stream. Called on first poll, never during planning.
    /// # Errors
    /// Domain admission, native input, resource or execution failure.
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream>;
    /// Retained definition ports are visible but not consumed by this body.
    fn definition_only(&self, _input: usize) -> bool {
        false
    }
    /// Prepared mathematical expressions, if any, remain visible to native visitors.
    fn expressions(&self) -> Vec<&Arc<dyn PhysicalExpr>> {
        vec![]
    }
}

/// One identity-bound operation and its visible native ports and expressions.
#[derive(Clone)]
pub struct Operation {
    definition: Arc<dyn Definition>,
    inputs: Vec<LogicalPlan>,
    expressions: Vec<Expr>,
    schema: DFSchemaRef,
    attempt: Arc<AtomicBool>,
}
impl Operation {
    pub(crate) fn same_binding(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.definition, &other.definition)
            && Arc::ptr_eq(&self.attempt, &other.attempt)
            && ports::same_inputs(&self.inputs, &other.inputs)
            && ports::same_expressions(&self.expressions, &other.expressions)
    }
    /// Bind a typed operation to actual native inputs.
    /// # Errors
    /// Invalid native schema.
    pub fn plan(definition: Arc<dyn Definition>, inputs: Vec<LogicalPlan>) -> Result<LogicalPlan> {
        let schema = Arc::new(DFSchema::try_from(definition.schema().as_ref().clone())?);
        Ok(LogicalPlan::Extension(Extension {
            node: Arc::new(Self {
                expressions: definition.expressions(),
                definition,
                inputs,
                schema,
                attempt: Arc::new(AtomicBool::new(false)),
            }),
        }))
    }
    /// Actual typed definition for domain result decoding, not name-based dispatch.
    pub fn definition<T: Definition>(&self) -> Option<&T> {
        let definition: &dyn Any = self.definition.as_ref();
        definition.downcast_ref()
    }
    /// Validated semantic input demand; this does not authorize optimizer rewrites.
    /// # Errors
    /// Wrong child arity or an out-of-range field ordinal.
    pub fn dependencies(
        &self,
        output: &crate::session::dependencies::SemanticDemand,
    ) -> Result<Vec<crate::session::dependencies::SemanticDemand>> {
        let demands = self.definition.dependencies(&self.inputs, output);
        if demands.len() != self.inputs.len()
            || demands.iter().zip(&self.inputs).any(|(demand, input)| {
                demand
                    .columns
                    .iter()
                    .chain(&demand.keys)
                    .any(|i| *i >= input.schema().fields().len())
            })
        {
            return Err(invalid(
                "semantic dependency transfer changed child arity or field bounds",
            ));
        }
        Ok(demands)
    }
    /// Declared effect semantics, independent of optimizer output demand.
    pub fn effects(&self) -> BTreeSet<OperationEffect> {
        self.definition.effects()
    }
    /// Whether an input is retained as a definition rather than executed now.
    pub fn definition_only(&self, input: usize) -> bool {
        self.definition.definition_only(input)
    }
    /// Operation family, used by execution admission.
    pub fn family(&self) -> Family {
        self.definition.family()
    }
}
impl fmt::Debug for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
            || (Arc::ptr_eq(&self.definition, &other.definition)
                && Arc::ptr_eq(&self.attempt, &other.attempt)
                && self.inputs == other.inputs
                && self.expressions == other.expressions)
    }
}
impl Eq for Operation {}
impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.definition), state);
        std::ptr::hash(Arc::as_ptr(&self.attempt), state);
        self.inputs.hash(state);
        self.expressions.hash(state);
    }
}
impl PartialOrd for Operation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let left = Arc::as_ptr(&self.definition).cast::<()>();
        let right = Arc::as_ptr(&other.definition).cast::<()>();
        match left
            .cmp(&right)
            .then_with(|| Arc::as_ptr(&self.attempt).cmp(&Arc::as_ptr(&other.attempt)))
        {
            std::cmp::Ordering::Equal => match self.inputs.partial_cmp(&other.inputs)? {
                std::cmp::Ordering::Equal => self.expressions.partial_cmp(&other.expressions),
                order => Some(order),
            },
            order => Some(order),
        }
    }
}
impl UserDefinedLogicalNodeCore for Operation {
    fn name(&self) -> &'static str {
        self.definition.name()
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        self.inputs.iter().collect()
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn expressions(&self) -> Vec<Expr> {
        self.expressions.clone()
    }
    fn fmt_for_explain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: family={:?}, inputs={}, effects={:?}",
            self.definition.name(),
            self.family(),
            self.inputs.len(),
            self.effects()
        )
    }
    fn with_exprs_and_inputs(
        &self,
        expressions: Vec<Expr>,
        inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if expressions.len() != self.expressions.len()
            || inputs.len() != self.inputs.len()
            || inputs
                .iter()
                .zip(&self.inputs)
                .any(|(a, b)| !fields_match(b.schema().fields(), a.schema().fields()))
        {
            return Err(invalid(
                "operation rewrite changed its expression or port contract",
            ));
        }
        Ok(Self {
            inputs,
            expressions,
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct Planner;
#[async_trait::async_trait]
impl ExtensionPlanner for Planner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<Operation>() else {
            return Ok(None);
        };
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| invalid("operation requires the actual caller SessionState"))?;
        let services = NativeExecutionContext::from_session(session)?;
        services
            .admit_effects(&node.effects())
            .map_err(pse_columnar::external)?;
        validate(
            &node
                .inputs
                .iter()
                .map(|input| Arc::new(input.schema().as_arrow().clone()))
                .collect::<Vec<_>>(),
            inputs,
            |index| node.definition_only(index),
        )?;
        let body = node
            .definition
            .clone()
            .prepare(&node.expressions, &node.inputs, inputs, state)
            .await?;
        Ok(Some(Execution::plan(
            node.definition.name(),
            node.definition.schema(),
            body.prepared_inputs().unwrap_or_else(|| inputs.to_vec()),
            body,
            node.family(),
            if node.family() == Family::Command {
                node.attempt.clone()
            } else {
                Arc::default()
            },
            services,
        )?))
    }
}

/// Shared native physical shell, also usable by provider DML hooks.
pub struct Execution {
    name: &'static str,
    body: Arc<dyn Body>,
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    ports: Vec<SchemaRef>,
    properties: Arc<PlanProperties>,
    family: Family,
    attempt: Arc<AtomicBool>,
    services: Arc<NativeExecutionContext>,
    metrics: ExecutionPlanMetricsSet,
}
impl Execution {
    pub(crate) fn checked_output(
        &self,
        context: Arc<TaskContext>,
    ) -> Option<
        futures_util::future::BoxFuture<
            'static,
            Result<Vec<pse_relations::columnar::FieldCheckedBatch>>,
        >,
    > {
        self.body.checked_output(self.inputs.clone(), context)
    }
    /// Construct bounded execution with actual input contracts and shared attempt ownership.
    /// # Errors
    /// An unbounded input was supplied to a finite operation.
    pub fn plan(
        name: &'static str,
        schema: SchemaRef,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        body: Arc<dyn Body>,
        family: Family,
        attempt: Arc<AtomicBool>,
        services: Arc<NativeExecutionContext>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let ports = inputs
            .iter()
            .map(|input| input.schema())
            .collect::<Vec<_>>();
        validate(&ports, &inputs, |index| body.definition_only(index))?;
        Ok(Arc::new(Self {
            name,
            body,
            inputs,
            ports,
            family,
            attempt,
            services,
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(schema),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
            metrics: ExecutionPlanMetricsSet::new(),
        }))
    }
}
impl fmt::Debug for Execution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as(DisplayFormatType::Default, f)
    }
}
impl DisplayAs for Execution {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}Exec: family={:?}, inputs={}",
            self.name,
            self.family,
            self.inputs.len()
        )
    }
}
impl ExecutionPlan for Execution {
    fn name(&self) -> &'static str {
        self.name
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        self.inputs.iter().collect()
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.replace_children(
            children,
            datafusion::physical_plan::execution_plan::ReplaceChildrenOptions::new(
                datafusion::physical_plan::execution_plan::ChildrenPropertiesMode::Recompute,
            ),
        )
    }
    fn replace_children(
        self: Arc<Self>,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: datafusion::physical_plan::execution_plan::ReplaceChildrenOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        validate(&self.ports, &inputs, |index| {
            self.body.definition_only(index)
        })?;
        if self
            .inputs
            .iter()
            .zip(&inputs)
            .all(|(a, b)| Arc::ptr_eq(a, b))
        {
            return Ok(self);
        }
        let body = self
            .body
            .rebind(&inputs)?
            .unwrap_or_else(|| self.body.clone());
        Self::plan(
            self.name,
            self.schema(),
            inputs,
            body,
            self.family,
            self.attempt.clone(),
            self.services.clone(),
        )
    }
    fn apply_expressions(
        &self,
        visitor: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        for expr in self.body.expressions() {
            if visitor(expr)? == TreeNodeRecursion::Stop {
                return Ok(TreeNodeRecursion::Stop);
            }
        }
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
            return Err(invalid("finite operation has one output partition"));
        }
        let body = self.body.clone();
        let inputs = self.inputs.clone();
        let family = self.family;
        let attempt = self.attempt.clone();
        let services = self.services.clone();
        let starts = datafusion::physical_plan::metrics::MetricBuilder::new(&self.metrics)
            .counter("starts", partition);
        let stream = futures_util::stream::once(async move {
            services
                .cancellation()
                .checkpoint()
                .map_err(pse_columnar::external)?;
            if family != Family::Finite && attempt.swap(true, Ordering::AcqRel) {
                return Err(invalid("operation attempt cannot be replayed"));
            }
            starts.add(1);
            let stream = body.execute(inputs, context)?;
            if matches!(family, Family::Command | Family::Foreign) {
                settle(stream, services).await
            } else {
                Ok(stream)
            }
        })
        .try_flatten();
        Ok(observe(
            self.schema(),
            Box::pin(stream),
            &self.metrics,
            partition,
        ))
    }
}

/// Native metrics for a lazy stream, including early drop and per-poll compute time.
pub(crate) fn observe(
    schema: SchemaRef,
    mut stream: std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<RecordBatch>> + Send>>,
    metrics: &ExecutionPlanMetricsSet,
    partition: usize,
) -> SendableRecordBatchStream {
    let baseline = BaselineMetrics::new(metrics, partition);
    let errors = datafusion::physical_plan::metrics::MetricBuilder::new(metrics)
        .counter("errors", partition);
    let mut guard = StreamCompletion {
        terminal: false,
        cancelled: datafusion::physical_plan::metrics::MetricBuilder::new(metrics)
            .counter("early_drops", partition),
    };
    let stream = futures_util::stream::poll_fn(move |cx| {
        let _timer = baseline.elapsed_compute().timer();
        let poll = stream.as_mut().poll_next(cx);
        if let std::task::Poll::Ready(result) = &poll {
            if result.as_ref().is_some_and(Result::is_err) {
                errors.add(1);
            }
            if result.is_none() || result.as_ref().is_some_and(Result::is_err) {
                guard.finish();
            }
        }
        baseline.record_poll(poll)
    });
    Box::pin(RecordBatchStreamAdapter::new(schema, stream))
}
struct StreamCompletion {
    terminal: bool,
    cancelled: datafusion::physical_plan::metrics::Count,
}
impl StreamCompletion {
    fn finish(&mut self) {
        self.terminal = true;
    }
}
impl Drop for StreamCompletion {
    fn drop(&mut self) {
        if !self.terminal {
            self.cancelled.add(1);
        }
    }
}

/// Adapt one deferred native batch without a bespoke stream implementation.
pub fn batch(
    schema: SchemaRef,
    future: impl Future<Output = Result<RecordBatch>> + Send + 'static,
) -> SendableRecordBatchStream {
    Box::pin(RecordBatchStreamAdapter::new(
        schema,
        futures_util::stream::once(future),
    ))
}

async fn settle(
    mut stream: SendableRecordBatchStream,
    services: Arc<NativeExecutionContext>,
) -> Result<SendableRecordBatchStream> {
    use tracing::Instrument;
    use tracing::instrument::WithSubscriber;
    let schema = stream.schema();
    let task_services = services.clone();
    let work = async move {
        let mut batches = Vec::new();
        loop {
            let next = tokio::select! {
                biased;
                result = stream.try_next() => result?,
                () = task_services.cancellation().cancelled() => {
                    if task_services.must_settle() { stream.try_next().await? }
                    else { return Err(invalid("command cancelled before settlement")); }
                }
            };
            match next {
                Some(batch) => batches.push(batch),
                None => return Ok(batches),
            }
        }
    };
    let task = tokio::spawn(
        work.instrument(tracing::Span::current())
            .with_subscriber(tracing::dispatcher::get_default(Clone::clone)),
    );
    let mut guard = CommandReader(Some(services));
    let batches = task
        .await
        .map_err(|error| DataFusionError::Execution(format!("command task: {error}")))??;
    // Successful completion does not signal cancellation to sibling operations.
    guard.0.take();
    Ok(Box::pin(RecordBatchStreamAdapter::new(
        schema,
        futures_util::stream::iter(batches.into_iter().map(Ok)),
    )))
}
struct CommandReader(Option<Arc<NativeExecutionContext>>);
impl Drop for CommandReader {
    fn drop(&mut self) {
        if let Some(services) = &self.0 {
            services.cancellation().cancel();
        }
    }
}

pub(crate) fn fields_match(
    expected: &datafusion::arrow::datatypes::Fields,
    actual: &datafusion::arrow::datatypes::Fields,
) -> bool {
    expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            (!actual.is_nullable() || expected.is_nullable())
                && actual
                    .as_ref()
                    .clone()
                    .with_nullable(expected.is_nullable())
                    == **expected
        })
}
fn validate(
    ports: &[SchemaRef],
    inputs: &[Arc<dyn ExecutionPlan>],
    definition_only: impl Fn(usize) -> bool,
) -> Result<()> {
    if ports.len() != inputs.len()
        || ports
            .iter()
            .zip(inputs)
            .enumerate()
            .any(|(index, (schema, input))| {
                !fields_match(schema.fields(), input.schema().fields())
                    || (!definition_only(index) && input.boundedness().is_unbounded())
            })
    {
        return Err(invalid(
            "operation requires its complete bounded input fields",
        ));
    }
    Ok(())
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
