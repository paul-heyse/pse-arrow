// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The single native preparation/execution boundary. Constructors stay private;
//! a completed computation can only result from draining its prepared execution.

use super::{PlanObservation, SnapshotSession, admission, observation::Recorder};
use crate::{CatalogError, PlanOrigin};
use datafusion::{
    arrow::array::RecordBatch,
    common::tree_node::{TreeNode, TreeNodeRecursion},
    logical_expr::LogicalPlan,
};
use pse_ids::{CancellationToken, owned_buffer::OwnedRecordBatch};
use std::sync::{Arc, Mutex};

/// A native plan bound to actual immutable providers, functions and configuration.
/// The original and analyzed plans remain available after optimizer elimination.
/// This establishes preparation, not the absence of residual data obligations.
#[derive(Clone)]
pub struct PreparedComputation(Arc<Preparation>);
struct Preparation {
    session: SnapshotSession,
    state: datafusion::execution::session_state::SessionState,
    original: LogicalPlan,
    analyzed: LogicalPlan,
    optimized: LogicalPlan,
    observation: PlanObservation,
    origin: PlanOrigin,
    volatile: bool,
    started: std::sync::atomic::AtomicBool,
    effects: std::collections::BTreeSet<pse_schema::model::provider::OperationEffect>,
    requirements: Option<PreparedComputation>,
    mutation: Option<super::mutation::PreparedMutation>,
}

impl std::fmt::Debug for PreparedComputation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedComputation")
            .field("schema", self.0.optimized.schema())
            .field("volatile", &self.0.volatile)
            .finish_non_exhaustive()
    }
}

/// Owned results of one successfully completed prepared computation.
/// This is not a publication token: stage obligations still have to be discharged.
#[derive(Debug)]
pub struct CompletedComputation {
    prepared: PreparedComputation,
    batches: Vec<OwnedRecordBatch>,
    physical: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    observation: PlanObservation,
    checked: Mutex<Option<pse_relations::columnar::FieldCheckedBatch>>,
    state: datafusion::execution::session_state::SessionState,
}

impl SnapshotSession {
    /// Derive native fields from exact retained sources without optimizing or executing.
    /// # Errors
    /// A foreign source or incompatible actual expression field.
    pub fn derive_plan_fields(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<LogicalPlan, CatalogError> {
        admission::restore_semantic_fields(
            plan,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, PlanOrigin::RuleCompiler))
    }
    /// Prepare a native generated-rule plan with platform error classification.
    /// # Errors
    /// Invalid semantic bindings, native analysis, optimization or cancellation.
    pub fn prepare_rule_plan(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, CatalogError> {
        self.prepare_with_origin(plan, cancel, PlanOrigin::RuleCompiler)
    }
    /// Bind and analyze a native logical plan without executing its data operators.
    /// # Errors
    /// A foreign binding, invalid field contract, native analysis or cancellation.
    pub fn prepare(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, CatalogError> {
        self.prepare_with_origin(plan, cancel, PlanOrigin::Analytics)
    }

    pub(super) fn prepare_with_origin(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<PreparedComputation, CatalogError> {
        self.bind_targets(&plan)?
            .execution_scope()?
            .prepare_scoped(plan, cancel, origin)
    }

    fn prepare_scoped(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<PreparedComputation, CatalogError> {
        cancel.checkpoint()?;
        let requirements = self.prepare_requirements(cancel)?;
        // Native expressions already own their actual function implementations.
        // The SQL name registry is needed for resolution and diagnostic decoding,
        // not for restricting executable plans to registered function names.
        let output_relation = plan
            .schema()
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_ID)
            .and_then(|value| pse_ids::SemanticId::parse_hex(value).ok())
            .and_then(|id| self.registry.relation_by_id(id))
            .filter(|spec| {
                pse_schema::arrow::relation_schema(&self.registry, spec)
                    .is_ok_and(|schema| &schema == plan.schema().as_arrow())
            });
        let plan = admission::restore_semantic_fields(
            plan,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| {
            stage_error(
                self.execution_error(error, origin),
                "initial native field derivation",
            )
        })?;
        let mut volatile =
            requires_fresh_execution(&plan).map_err(|error| self.execution_error(error, origin))?;
        let mut effects = self.admit_effects(&plan, volatile)?;
        volatile |= self.bindings.iter().any(|(_, binding)| {
            binding
                .effects
                .contains(&pse_schema::model::provider::OperationEffect::Observe)
        });
        let state =
            super::execution::NativeExecutionContext::bind(self, self.bound_state()?, cancel)?;
        let mut recorder = Recorder::new(self.reserver.as_ref());
        let analyzed = state
            .analyzer()
            .execute_and_check(plan.clone(), state.config_options(), |_, rule| {
                recorder.rule(rule.name());
            })
            .map_err(|error| stage_error(self.execution_error(error, origin), "native analyzer"))?;
        cancel.checkpoint()?;
        // Native analysis can introduce function calls. Inspect them before
        // optimization can fold or eliminate their binding-time dependencies.
        let analyzed_varying = requires_fresh_execution(&analyzed)
            .map_err(|error| self.execution_error(error, origin))?;
        effects.extend(self.admit_effects(&analyzed, analyzed_varying)?);
        volatile |= analyzed_varying;
        let optimized = state
            .optimizer()
            .optimize(analyzed.clone(), &state, |_, rule| {
                recorder.rule(rule.name());
            })
            .map_err(|error| {
                let detail = error.to_string();
                stage_error(
                    self.execution_error(error, origin),
                    &format!("native optimizer ({detail})"),
                )
            })?;
        let optimized = admission::restore_semantic_fields(
            optimized,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        let optimized = if let Some(spec) = output_relation {
            super::output::declare_relation_output(optimized, &self.registry, spec)
                .map_err(|error| self.execution_error(error, origin))?
        } else {
            optimized
        };
        let optimized = super::scalar::materialize_nested_fields(
            optimized,
            &self.scalar_function("pse_preserve_field")?,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        recorder.rule("pse.nested_field_materialization.v1");
        let mutation = super::mutation::PreparedMutation::bind(self, &optimized)?;
        let observation = recorder.finish(&optimized)?;
        Ok(PreparedComputation(Arc::new(Preparation {
            session: self.clone(),
            state,
            original: plan,
            analyzed,
            optimized,
            observation,
            origin,
            volatile,
            started: std::sync::atomic::AtomicBool::new(false),
            effects,
            requirements,
            mutation,
        })))
    }
}

impl PreparedComputation {
    pub(crate) fn bound_session(&self) -> SnapshotSession {
        self.0.session.clone()
    }
    pub(super) fn check_registry(
        &self,
        registry: &Arc<pse_schema::Registry>,
    ) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&self.0.session.registry, registry) {
            return Err(CatalogError::Admission {
                path: "computed input".to_owned(),
                reason: "different actual registry authority".to_owned(),
            });
        }
        Ok(())
    }
    /// Native intermediate roles retained before any optimizer can remove their scans.
    pub fn computation_roles(&self) -> impl Iterator<Item = &str> {
        self.0.session.computation_roles()
    }
    /// Original binding-time plan, including inputs later eliminated by optimization.
    pub fn original_plan(&self) -> &LogicalPlan {
        &self.0.original
    }
    /// Plan after the configured native semantic analyzer.
    pub fn analyzed_plan(&self) -> &LogicalPlan {
        &self.0.analyzed
    }
    /// The actual logical plan supplied to the physical planner.
    pub fn optimized_plan(&self) -> &LogicalPlan {
        &self.0.optimized
    }
    /// Actual optimizer observations; diagnostic only.
    pub fn observation(&self) -> &PlanObservation {
        &self.0.observation
    }
    /// Volatility observed before optimization. A false value alone does not admit reuse.
    pub fn contains_volatile_expression(&self) -> bool {
        self.0.volatile
    }
    /// Actual native query start time frozen when this computation was prepared.
    /// Stable functions and physical execution use this same retained state.
    pub fn query_start_time(&self) -> Option<String> {
        self.0
            .state
            .execution_props()
            .query_execution_start_time
            .map(|time| time.to_rfc3339())
    }
    /// Exact retained source inventory, including empty and negative-read scopes.
    pub fn input_keys(&self) -> impl Iterator<Item = pse_schema::model::RelationKey> + '_ {
        self.0.session.input_keys()
    }
    /// Named roles retained even when an optimizer removes their scans.
    pub fn input_roles(&self) -> impl Iterator<Item = (&str, pse_schema::model::RelationKey)> + '_ {
        self.0.session.input_roles()
    }
    /// Execute and explicitly materialize every owned result batch.
    /// # Errors
    /// Planning, policy, resource, cancellation or native execution failure.
    pub async fn execute(
        self,
        cancel: &CancellationToken,
    ) -> Result<CompletedComputation, CatalogError> {
        self.execute_stream(cancel).await?.collect(cancel).await
    }

    /// Begin native execution without draining or accumulating the result stream.
    /// The stream retains its complete preparation, providers and resource owners.
    /// # Errors
    /// Planning, unsupported actual implementation, cancellation or policy failure.
    pub async fn execute_stream(
        self,
        cancel: &CancellationToken,
    ) -> Result<OwnedComputationStream, CatalogError> {
        cancel.checkpoint()?;
        let session = &self.0.session;
        if let Some(requirements) = &self.0.requirements {
            let mut stream = Box::pin(requirements.clone().execute_stream(cancel)).await?;
            while let Some(batch) = stream.next_batch(cancel).await? {
                if batch.num_rows() != 0 {
                    return Err(CatalogError::Admission {
                        path: "operation.requirements".to_owned(),
                        reason: "scoped invariant requirements produced violations".to_owned(),
                    });
                }
            }
        }
        let execution_state =
            super::execution::NativeExecutionContext::execution_state(&self.0.state, cancel);
        let state = &execution_state;
        let command = matches!(
            self.0.original,
            LogicalPlan::Ddl(_) | LogicalPlan::Statement(_)
        );
        let effectful =
            command || matches!(self.0.original, LogicalPlan::Dml(_) | LogicalPlan::Copy(_));
        if effectful
            && self
                .0
                .started
                .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return Err(CatalogError::Admission {
                path: "operation.attempt".to_owned(),
                reason: "an effectful preparation can execute only once".to_owned(),
            });
        }
        let (physical, execution_state) = tokio::select! {
            biased;
            () = cancel.cancelled() => { cancel.checkpoint()?; unreachable_cancel()? },
            result = async {
                if let Some(mutation) = &self.0.mutation {
                    let physical = mutation.physical(session.clone(), cancel.clone(), state).await?;
                    Ok((physical, state.clone()))
                } else if let LogicalPlan::Extension(extension) = &self.0.optimized
                    && let Some(operation) = extension.node.as_any().downcast_ref::<super::operation::OperationNode>() {
                    let physical = datafusion::physical_planner::DefaultPhysicalPlanner::default()
                        .optimize_physical_plan(operation.physical(session.clone(), cancel.clone()), state, |_, _| {})?;
                    Ok((physical, state.clone()))
                } else if command {
                    // Eager native handlers are called only here, after preparation.
                    let context = datafusion::execution::context::SessionContext::new_with_state(state.clone());
                    let namespace = if let LogicalPlan::Ddl(command) = &self.0.optimized {
                        super::commands::create_namespace(&context, command)?
                    } else { None };
                    let frame = match namespace {
                        Some(frame) => frame,
                        None => context.execute_logical_plan(self.0.optimized.clone()).await?,
                    };
                    let physical = frame.create_physical_plan().await?;
                    Ok((physical, context.state()))
                } else {
                    state.query_planner().create_physical_plan(&self.0.optimized, state).await.map(|physical| (physical, state.clone()))
                }
            } => result,
        }.map_err(|error| session.execution_error(error, self.0.origin))?;
        unchanged_namespace(&self)?;
        let observation = self
            .0
            .observation
            .with_physical(physical.as_ref(), session.reserver.as_ref())?;
        session.trace.record(observation.clone(), self.0.volatile)?;
        let stream = datafusion::physical_plan::execute_stream(
            Arc::clone(&physical),
            execution_state.task_ctx(),
        )
        .map_err(|error| session.execution_error(error, self.0.origin))?;
        Ok(OwnedComputationStream {
            prepared: self,
            physical,
            observation,
            stream: Some(stream),
            finished: false,
            yielded: false,
            state: execution_state,
        })
    }
}

/// A native stream with explicit batch ownership and no implicit full-result collection.
/// Dropping it cancels unconsumed work and releases its retained resources.
pub struct OwnedComputationStream {
    prepared: PreparedComputation,
    physical: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    observation: PlanObservation,
    stream: Option<datafusion::physical_plan::SendableRecordBatchStream>,
    finished: bool,
    yielded: bool,
    state: datafusion::execution::session_state::SessionState,
}
impl std::fmt::Debug for OwnedComputationStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedComputationStream")
            .field("finished", &self.finished)
            .finish_non_exhaustive()
    }
}
impl OwnedComputationStream {
    /// Whether the actual physical source promises a finite result.
    pub fn is_bounded(&self) -> bool {
        matches!(
            self.physical.properties().boundedness,
            datafusion::physical_plan::execution_plan::Boundedness::Bounded
        )
    }
    /// Split native result batches without copying buffers or draining ahead.
    /// # Errors
    /// Export already started, or the stream failed before this selection.
    pub fn with_batch_size(mut self, size: std::num::NonZeroUsize) -> Result<Self, CatalogError> {
        if self.yielded || self.finished {
            return Err(CatalogError::Admission {
                path: "operation.stream".into(),
                reason: "select batch size before consuming the stream".into(),
            });
        }
        let input = self.stream.take().ok_or_else(|| CatalogError::Admission {
            path: "operation.stream".into(),
            reason: "stream already failed".into(),
        })?;
        let metrics = datafusion::physical_plan::metrics::ExecutionPlanMetricsSet::new();
        self.stream = Some(Box::pin(
            datafusion::physical_plan::stream::BatchSplitStream::new(
                input,
                size.get(),
                datafusion::physical_plan::metrics::SplitMetrics::new(&metrics, 0),
            ),
        ));
        Ok(self)
    }
    /// Pull one batch; no later batch is drained on the caller's behalf.
    /// # Errors
    /// Cancellation, allocation refusal, generation change or native stream error.
    pub async fn next_batch(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Option<OwnedRecordBatch>, CatalogError> {
        let result = self.next_owned(cancel).await;
        if result.is_err() {
            self.stream = None;
        }
        result.map_err(|error| match self.prepared.operation() {
            Some(operation) => operation.execution_failure(error),
            None => error,
        })
    }
    async fn next_owned(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Option<OwnedRecordBatch>, CatalogError> {
        let completed_cancel = CancellationToken::new();
        let cancel = if self
            .prepared
            .operation()
            .is_some_and(super::operation::NativeOperation::irreversible_completion)
        {
            &completed_cancel
        } else {
            cancel
        };
        unchanged_namespace(&self.prepared)?;
        if self.finished {
            if !self.must_settle() {
                cancel.checkpoint()?;
            }
            return Ok(None);
        }
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| CatalogError::Admission {
                path: "operation.stream".to_owned(),
                reason: "stream already failed".to_owned(),
            })?;
        let next = if self
            .prepared
            .operation()
            .is_some_and(super::operation::NativeOperation::handles_cancellation)
        {
            std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await
        } else {
            tokio::select! {
                biased;
                next = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)) => next,
                () = cancel.cancelled() => {
                    if let Some(services) = self.state.config().get_extension::<super::execution::NativeExecutionContext>()
                        && services.must_settle() {
                        // Pull cancellation may differ from execution cancellation.
                        // Signal this invocation before waiting for its terminal data.
                        services.cancellation().cancel();
                        std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await
                    } else { cancel.checkpoint()?; unreachable_cancel()? }
                },
            }
        };
        unchanged_namespace(&self.prepared)?;
        let Some(batch) = next else {
            if !self.must_settle() {
                cancel.checkpoint()?;
            }
            self.finished = true;
            self.stream = None;
            return Ok(None);
        };
        let session = &self.prepared.0.session;
        let batch =
            batch.map_err(|error| session.execution_error(error, self.prepared.0.origin))?;
        let cancel = if self
            .prepared
            .operation()
            .is_some_and(super::operation::NativeOperation::irreversible_completion)
            || self.must_settle()
        {
            &completed_cancel
        } else {
            cancel
        };
        self.yielded = true;
        Ok(Some(OwnedRecordBatch::export(
            batch,
            session.reserver.as_ref(),
            cancel,
        )?))
    }
    /// Successful exhaustion, distinct from drop, cancellation and error.
    pub const fn is_complete(&self) -> bool {
        self.finished
    }
    fn must_settle(&self) -> bool {
        self.state
            .config()
            .get_extension::<super::execution::NativeExecutionContext>()
            .is_some_and(|services| services.must_settle())
    }
    /// Materialize the entire stream, before any batch has been pulled separately.
    /// # Errors
    /// Any failure encountered while draining the native stream.
    pub async fn collect(
        mut self,
        cancel: &CancellationToken,
    ) -> Result<CompletedComputation, CatalogError> {
        if self.yielded {
            return Err(CatalogError::Admission {
                path: "operation.materialization".to_owned(),
                reason:
                    "cannot establish a complete materialization after exporting stream batches"
                        .to_owned(),
            });
        }
        let mut batches = Vec::new();
        while let Some(batch) = self.next_batch(cancel).await? {
            batches.push(batch);
        }
        Ok(CompletedComputation {
            prepared: self.prepared,
            batches,
            physical: self.physical,
            observation: self.observation,
            checked: Mutex::new(None),
            state: self.state,
        })
    }
}

impl PreparedComputation {
    fn operation(&self) -> Option<&dyn super::operation::NativeOperation> {
        if let LogicalPlan::Extension(extension) = &self.0.original {
            extension
                .node
                .as_any()
                .downcast_ref::<super::operation::OperationNode>()
                .map(|node| node.operation.as_ref())
        } else {
            None
        }
    }
}

fn unchanged_namespace(prepared: &PreparedComputation) -> Result<(), CatalogError> {
    if prepared
        .0
        .effects
        .contains(&pse_schema::model::provider::OperationEffect::Namespace)
    {
        return Ok(());
    }
    if prepared
        .0
        .state
        .catalog_list()
        .downcast_ref::<crate::provider::list::SnapshotCatalogList>()
        .is_some_and(|catalogs| catalogs.generation() != 0)
    {
        return Err(CatalogError::Admission {
            path: "operation.namespace_effect".to_owned(),
            reason: "read execution changed its captured catalog generation".to_owned(),
        });
    }
    Ok(())
}

impl CompletedComputation {
    /// Bind the completed command's private native namespace as a new session.
    /// Earlier sessions and preparations retain their original namespace owners.
    /// # Errors
    /// A new provider cannot satisfy source admission or resolution is cancelled.
    pub async fn resulting_session(
        &self,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        cancel.checkpoint()?;
        if let Some(mutation) = &self.prepared.0.mutation {
            return mutation.resulting_session(&self.prepared.0.session);
        }
        let created_memory_table = match &self.prepared.0.original {
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(
                command,
            )) => {
                let defaults = &self.state.config_options().catalog;
                let name = command
                    .name
                    .clone()
                    .resolve(&defaults.default_catalog, &defaults.default_schema);
                Some(datafusion::common::TableReference::full(
                    name.catalog,
                    name.schema,
                    name.table,
                ))
            }
            _ => None,
        };
        self.prepared
            .0
            .session
            .capture_namespace(&self.state, created_memory_table.as_ref(), cancel)
            .await
    }
    /// Actual optimized physical execution retained with this completed result.
    pub fn physical_plan(&self) -> &Arc<dyn datafusion::physical_plan::ExecutionPlan> {
        &self.physical
    }
    /// Logical and physical observations from this particular execution.
    pub fn observation(&self) -> &PlanObservation {
        &self.observation
    }
    /// Materialize one explicitly declared relation. Its schema must already be
    /// present in the prepared native plan; this operation never relabels results.
    /// Local field predicates not established by the native transfer execute at
    /// this boundary. Relational keys/FKs remain separate obligations.
    /// # Errors
    /// A different declaration/schema, a local value predicate, allocation or cancellation.
    pub fn into_checked_relation(
        self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, CatalogError> {
        self.checked_relation(registry, spec, cancel)
    }
    /// Borrow this completed computation while retaining its exact output buffers.
    /// Successful materialization retains its checked fields and allocation claim.
    /// Subsequent access checks the declaration and shares that exact result; it
    /// neither repeats local value admission nor concatenates its buffers again.
    /// # Errors
    /// Declaration mismatch, unresolved value predicate, allocation or cancellation.
    pub fn checked_relation(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, CatalogError> {
        cancel.checkpoint()?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(registry, spec)
                .map_err(pse_relations::RelationError::from)?,
        );
        if self.prepared.optimized_plan().schema().as_arrow() != schema.as_ref() {
            return Err(CatalogError::Admission {
                path: spec.key.qualified_name(),
                reason: "native output plan is not bound to this exact declared relation"
                    .to_owned(),
            });
        }
        let mut checked = self.checked.lock().map_err(|_| CatalogError::Internal {
            message: "completed relation admission lock was poisoned".to_owned(),
        })?;
        cancel.checkpoint()?;
        if let Some(batch) = checked.as_ref() {
            batch.check_declaration(registry, spec)?;
            return Ok(batch.clone());
        }
        let pieces = self
            .batches
            .iter()
            .map(|batch| {
                cancel.checkpoint()?;
                Ok(pse_relations::columnar::FieldCheckedBatch::admit_owned(
                    registry,
                    spec,
                    batch.clone(),
                )?)
            })
            .collect::<Result<Vec<_>, CatalogError>>()?;
        let batch = pse_relations::columnar::FieldCheckedBatch::concat_reserved(
            registry,
            spec,
            &pieces,
            self.prepared.0.session.reserver.as_ref(),
            cancel,
        )?;
        *checked = Some(batch.clone());
        Ok(batch)
    }
    /// Preparation and source owners that produced these exact results.
    pub const fn prepared(&self) -> &PreparedComputation {
        &self.prepared
    }
    /// Immutable owned result batches.
    pub fn batches(&self) -> &[OwnedRecordBatch] {
        &self.batches
    }
    /// Consume the results at an explicit materialization boundary.
    pub fn into_batches(self) -> Vec<RecordBatch> {
        self.batches
            .into_iter()
            .map(OwnedRecordBatch::into_batch)
            .collect()
    }
    pub(super) fn into_observed_batches(self) -> (Vec<RecordBatch>, PlanObservation) {
        (
            self.batches
                .into_iter()
                .map(OwnedRecordBatch::into_batch)
                .collect(),
            self.observation,
        )
    }
}

fn requires_fresh_execution(plan: &LogicalPlan) -> datafusion::common::Result<bool> {
    let mut fresh = false;
    plan.apply_with_subqueries(|node| {
        for expression in node.expressions() {
            expression.apply(|expression| {
                use datafusion::logical_expr::{Expr, Volatility};
                fresh |= match expression {
                    Expr::ScalarFunction(value) => {
                        value.func.signature().volatility != Volatility::Immutable
                    }
                    Expr::AggregateFunction(value) => {
                        value.func.signature().volatility != Volatility::Immutable
                    }
                    Expr::WindowFunction(value) => {
                        value.fun.signature().volatility != Volatility::Immutable
                    }
                    Expr::HigherOrderFunction(value) => {
                        value.func.signature().volatility != Volatility::Immutable
                    }
                    Expr::ScalarVariable(..) => true,
                    _ => false,
                };
                Ok(TreeNodeRecursion::Continue)
            })?;
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(fresh)
}

fn unreachable_cancel<T>() -> Result<T, CatalogError> {
    Err(CatalogError::ConfigInvalid {
        key: "runtime.cancellation".to_owned(),
        reason: "cancellation notification did not retain its state".to_owned(),
    })
}
fn stage_error(error: CatalogError, stage: &str) -> CatalogError {
    match error {
        CatalogError::Internal { message } => CatalogError::Internal {
            message: format!("{stage}: {message}"),
        },
        CatalogError::UserModel { message } => CatalogError::UserModel {
            message: format!("{stage}: {message}"),
        },
        other => other,
    }
}
