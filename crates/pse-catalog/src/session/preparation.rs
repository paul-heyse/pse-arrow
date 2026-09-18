// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The single native preparation/execution boundary. Constructors stay private;
//! a completed computation can only result from draining its prepared execution.

use super::{PlanObservation, SnapshotSession, admission, observation::Recorder};
use crate::{CatalogError, PlanOrigin};
use datafusion::{
    arrow::array::RecordBatch, common::tree_node::TreeNodeRecursion, logical_expr::LogicalPlan,
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
    requirements: bool,
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
    /// One registry/budget-bound field admission scope for a complete artifact.
    /// Every graph still checks its sources and expressions; identical immutable
    /// Arrow child declarations are validated once across sibling outputs.
    /// # Errors
    /// Foreign sources, invalid native expressions or resource/cancellation refusal.
    pub fn derive_plan_fields_many(
        &self,
        plans: &[LogicalPlan],
        cancel: &CancellationToken,
    ) -> Result<Vec<LogicalPlan>, CatalogError> {
        admission::restore_semantic_fields_many(
            plans,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, PlanOrigin::RuleCompiler))
    }
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
            .prepare_scoped(plan, cancel, origin, None)
    }

    pub(crate) fn cache_planning_scope<'a>(
        &self,
        cancel: &'a CancellationToken,
    ) -> super::cache::logical::Scope<'a> {
        super::cache::logical::Scope::new(self.reserver.as_ref(), cancel)
    }

    pub(crate) fn prepare_group_member(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        scope: &mut super::cache::logical::Scope<'_>,
    ) -> Result<PreparedComputation, CatalogError> {
        // Only ArtifactPlan's immutable outputs, already derived and admitted
        // together against this exact captured session, use this private path.
        self.bind_targets(&plan)?.execution_scope()?.prepare_scoped(
            plan,
            cancel,
            PlanOrigin::RuleCompiler,
            Some(scope),
        )
    }

    fn output_declaration(&self, plan: &LogicalPlan) -> Option<&pse_schema::model::RelationSpec> {
        plan.schema()
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_ID)
            .and_then(|value| pse_ids::SemanticId::parse_hex(value).ok())
            .and_then(|id| self.registry.relation_by_id(id))
            .filter(|spec| {
                pse_schema::arrow::relation_schema(&self.registry, spec)
                    .is_ok_and(|schema| &schema == plan.schema().as_arrow())
            })
    }

    fn prepare_scoped(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
        cache_scope: Option<&mut super::cache::logical::Scope<'_>>,
    ) -> Result<PreparedComputation, CatalogError> {
        cancel.checkpoint()?;
        let requirements = !self.effective_policy()?.requirements.is_empty();
        if requirements && self.requirement_planner.is_none() {
            return Err(CatalogError::Admission {
                path: "provider.requirements".to_owned(),
                reason: "required invariants have no bound native implementation".to_owned(),
            });
        }
        // Native expressions already own their actual function implementations.
        // The SQL name registry is needed for resolution and diagnostic decoding,
        // not for restricting executable plans to registered function names.
        let output_relation = self.output_declaration(&plan);
        let plan = if cache_scope.is_some() {
            plan
        } else {
            admission::restore_semantic_fields(
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
            })?
        };
        let mut volatile = self
            .plans_require_fresh([&plan], cancel)
            .map_err(|error| self.execution_error(error, origin))?;
        let mut effects = self.admit_effects(&plan, volatile)?;
        volatile |= self.bindings.iter().any(|(_, binding)| {
            binding
                .effects
                .contains(&pse_schema::model::provider::OperationEffect::Observe)
        });
        let state =
            super::execution::NativeExecutionContext::bind(self, self.bound_state()?, cancel)?;
        let mut recorder = Recorder::new(self.reserver.as_ref());
        let mut optimize_producer =
            |input| self.optimize_cache_producer(input, &state, cancel, &mut recorder);
        let staged = if let Some(scope) = cache_scope {
            scope.stage(plan.clone(), &mut optimize_producer)
        } else {
            super::cache::logical::stage(
                plan.clone(),
                self.reserver.as_ref(),
                cancel,
                &mut optimize_producer,
            )
        }
        .map_err(|error| {
            stage_error(
                self.execution_error(error, origin),
                "native cache producer optimization",
            )
        })?;
        crate::cache_service::metrics::record(&state, |metrics| &metrics.analyses);
        let analyzed = state
            .analyzer()
            .execute_and_check(staged, state.config_options(), |_, rule| {
                recorder.rule(rule.name());
            })
            .map_err(|error| stage_error(self.execution_error(error, origin), "native analyzer"))?;
        cancel.checkpoint()?;
        // Native analysis can introduce function calls. Inspect them before
        // optimization can fold or eliminate their binding-time dependencies.
        let visible_analyzed = super::cache::logical::expand(analyzed.clone())
            .map_err(|error| self.execution_error(error, origin))?;
        let analyzed_varying = self
            .plans_require_fresh([&visible_analyzed], cancel)
            .map_err(|error| self.execution_error(error, origin))?;
        effects.extend(self.admit_effects(&visible_analyzed, analyzed_varying)?);
        volatile |= analyzed_varying;
        crate::cache_service::metrics::record(&state, |metrics| &metrics.optimizations);
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
        let optimized = self.finalize_native_fields(optimized, output_relation, cancel, origin)?;
        recorder.rule("pse.nested_field_materialization.v1");
        let observation = recorder.finish(&optimized, cancel)?;
        Ok(PreparedComputation(Arc::new(Preparation {
            session: self.clone(),
            state,
            original: plan,
            analyzed: visible_analyzed,
            optimized,
            observation,
            origin,
            volatile,
            started: std::sync::atomic::AtomicBool::new(false),
            effects,
            requirements,
        })))
    }
}

impl SnapshotSession {
    fn finalize_native_fields(
        &self,
        plan: LogicalPlan,
        output: Option<&pse_schema::model::RelationSpec>,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<LogicalPlan, CatalogError> {
        let plan = admission::restore_semantic_fields(
            plan,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        let plan = if let Some(spec) = output {
            super::output::declare_relation_output(plan, &self.registry, spec)
                .map_err(|error| self.execution_error(error, origin))?
        } else {
            plan
        };
        let plan = super::scalar::materialize_nested_fields(
            plan,
            &self.scalar_function("pse_preserve_field")?,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        super::cache::logical::expand(plan).map_err(|error| self.execution_error(error, origin))
    }

    fn optimize_cache_producer(
        &self,
        input: LogicalPlan,
        state: &datafusion::execution::session_state::SessionState,
        cancel: &CancellationToken,
        recorder: &mut Recorder,
    ) -> datafusion::common::Result<LogicalPlan> {
        crate::cache_service::metrics::record(state, |metrics| &metrics.analyses);
        let analyzed =
            state
                .analyzer()
                .execute_and_check(input, state.config_options(), |_, rule| {
                    recorder.rule(rule.name());
                })?;
        let visible = super::cache::logical::expand(analyzed.clone())?;
        if self.plans_require_fresh([&visible], cancel)? {
            return Err(datafusion::common::DataFusionError::Plan(
                "native analysis introduced a varying or uncontracted cache producer".into(),
            ));
        }
        self.admit_effects(&visible, false)
            .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))?;
        crate::cache_service::metrics::record(state, |metrics| &metrics.optimizations);
        let optimized = state
            .optimizer()
            .optimize(analyzed, state, |_, rule| recorder.rule(rule.name()))?;
        let optimized = admission::restore_semantic_fields(
            optimized,
            &self.registry,
            &self.bindings.providers(),
            self.reserver.as_ref(),
            cancel,
        )?;
        super::scalar::materialize_nested_fields(
            optimized,
            &self
                .scalar_function("pse_preserve_field")
                .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))?,
        )
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
        self.execute_stream_in_scope(cancel, None).await
    }

    pub(crate) async fn execute_group(
        preparations: impl IntoIterator<Item = Result<Self, CatalogError>>,
        cancel: &CancellationToken,
    ) -> Result<Vec<CompletedComputation>, CatalogError> {
        let caches = Arc::new(super::cache::CacheStore::default());
        let mut completed = Vec::new();
        for prepared in preparations {
            completed.push(
                prepared?
                    .execute_stream_in_scope(cancel, Some(Arc::clone(&caches)))
                    .await?
                    .collect(cancel)
                    .await?,
            );
        }
        Ok(completed)
    }

    async fn execute_stream_in_scope(
        self,
        cancel: &CancellationToken,
        caches: Option<Arc<super::cache::CacheStore>>,
    ) -> Result<OwnedComputationStream, CatalogError> {
        let (physical, execution_state, observation) =
            self.plan_execution(cancel, false, caches).await?;
        let session = &self.0.session;
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
    /// Plan a pure finite computation once for repeated, fully drained rounds.
    /// Native dynamic filtering and recursive operators are ineligible for reset.
    /// # Errors
    /// Effects, volatility, unsupported reset semantics or physical planning failure.
    pub async fn prepare_reusable(
        self,
        cancel: &CancellationToken,
    ) -> Result<ReusableComputation, CatalogError> {
        if self.0.volatile
            || self
                .0
                .effects
                .iter()
                .any(|effect| *effect != pse_schema::model::provider::OperationEffect::Read)
        {
            return Err(super::engine(datafusion::common::DataFusionError::Plan(
                "reusable round plans require immutable read semantics".into(),
            )));
        }
        self.0
            .optimized
            .apply_with_subqueries(|node| {
                if matches!(node, LogicalPlan::RecursiveQuery(_)) {
                    return Err(datafusion::common::DataFusionError::Plan(
                        "native recursive queries cannot be reset for outer rounds".into(),
                    ));
                }
                if let LogicalPlan::Extension(extension) = node
                    && !super::cache::supports_round_reset(extension.node.as_ref())
                    && !extension
                        .node
                        .as_any()
                        .is::<super::contract::ExecutionContract>()
                {
                    return Err(datafusion::common::DataFusionError::Plan(format!(
                        "extension {} has no reusable round reset contract",
                        extension.node.name()
                    )));
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .map_err(super::engine)?;
        let (physical, state, observation) = self.plan_execution(cancel, true, None).await?;
        super::round::validate_reset_plan(&physical).map_err(super::engine)?;
        Ok(ReusableComputation {
            prepared: self,
            physical,
            state,
            observation,
            ready: true,
        })
    }

    async fn plan_execution(
        &self,
        cancel: &CancellationToken,
        reusable: bool,
        caches: Option<Arc<super::cache::CacheStore>>,
    ) -> Result<
        (
            Arc<dyn datafusion::physical_plan::ExecutionPlan>,
            datafusion::execution::session_state::SessionState,
            PlanObservation,
        ),
        CatalogError,
    > {
        cancel.checkpoint()?;
        let session = &self.0.session;
        let requirements = if self.0.requirements {
            session.prepare_requirements(cancel).await?
        } else {
            None
        };
        let effectful = self.0.effects.iter().any(|effect| {
            matches!(
                effect,
                pse_schema::model::provider::OperationEffect::Write
                    | pse_schema::model::provider::OperationEffect::Namespace
                    | pse_schema::model::provider::OperationEffect::Publish
            )
        });
        let native = if effectful || matches!(self.0.optimized, LogicalPlan::Explain(_)) {
            super::mutation::isolate(&self.0.optimized, session, cancel).map_err(super::engine)?
        } else {
            self.0.optimized.clone()
        };
        let contract = |plan| {
            super::contract::ExecutionContract::plan(
                plan,
                requirements
                    .as_ref()
                    .map(|plan| plan.optimized_plan().clone()),
                self.0.effects.clone(),
            )
        };
        // DataFusion requires EXPLAIN at the root. The inspected operation
        // retains its contract, without executing either child while explaining.
        let execution_plan = match native {
            LogicalPlan::Explain(mut explain) => {
                explain.plan = Arc::new(contract(Arc::unwrap_or_clone(explain.plan)));
                LogicalPlan::Explain(explain)
            }
            plan => contract(plan),
        };
        let mut execution_state = match caches {
            Some(caches) => super::execution::NativeExecutionContext::execution_state_with_caches(
                &self.0.state,
                cancel,
                caches,
            ),
            None => {
                super::execution::NativeExecutionContext::execution_state(&self.0.state, cancel)
            }
        };
        if reusable {
            let mut config = execution_state.config().clone();
            let options = config.options_mut();
            options.optimizer.enable_dynamic_filter_pushdown = false;
            options.optimizer.enable_join_dynamic_filter_pushdown = false;
            options.optimizer.enable_topk_dynamic_filter_pushdown = false;
            options.optimizer.enable_aggregate_dynamic_filter_pushdown = false;
            execution_state =
                datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
                    execution_state,
                )
                .with_config(config)
                .build();
        }
        let state = &execution_state;
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
        crate::cache_service::metrics::record(state, |metrics| &metrics.physical_plans);
        let (physical, execution_state) = tokio::select! {
            biased;
            () = cancel.cancelled() => { cancel.checkpoint()?; unreachable_cancel()? },
            result = async {
                state.query_planner().create_physical_plan(&execution_plan, state).await.map(|physical| (physical, state.clone()))
            } => result,
        }.map_err(|error| session.execution_error(error, self.0.origin))?;
        unchanged_namespace(self)?;
        let mut recorder = Recorder::new(session.reserver.as_ref());
        for rule in self.0.observation.rules_fired() {
            recorder.rule(rule);
        }
        let observation = recorder
            .finish(&execution_plan, cancel)?
            .with_physical(physical.as_ref(), session.reserver.as_ref())?;
        session.trace.record(observation.clone(), self.0.volatile)?;
        Ok((physical, execution_state, observation))
    }
}

/// One physical strategy reused only after complete successful exhaustion.
/// A failed or cancelled invocation poisons this owner; it cannot replay partial work.
#[derive(Debug)]
pub struct ReusableComputation {
    prepared: PreparedComputation,
    physical: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    state: datafusion::execution::session_state::SessionState,
    observation: PlanObservation,
    ready: bool,
}
impl ReusableComputation {
    /// Execute one complete epoch using native operator resets; never replans.
    /// # Errors
    /// An earlier failed execution, cancellation, reset or native execution failure.
    pub async fn execute(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<CompletedComputation, CatalogError> {
        if !self.ready {
            return Err(super::engine(
                datafusion::common::DataFusionError::Execution(
                    "reusable execution has an unfinished or failed epoch".into(),
                ),
            ));
        }
        self.ready = false;
        crate::cache_service::metrics::record(&self.state, |metrics| &metrics.reusable_executions);
        cancel.checkpoint()?;
        unchanged_namespace(&self.prepared)?;
        super::execution::NativeExecutionContext::from_session(&self.state)
            .map_err(super::engine)?
            .advance_round_epoch();
        let stream = datafusion::physical_plan::execute_stream(
            Arc::clone(&self.physical),
            self.state.task_ctx(),
        )
        .map_err(super::engine)?;
        let result = OwnedComputationStream {
            prepared: self.prepared.clone(),
            physical: Arc::clone(&self.physical),
            observation: self.observation.clone(),
            stream: Some(stream),
            finished: false,
            yielded: false,
            state: self.state.clone(),
        }
        .collect(cancel)
        .await?;
        // A short-circuited native join may retain an unfinished input future in
        // its plan state even after the root output is exhausted. Release that
        // state before the caller advances RoundInputs, not at the next execute.
        self.physical = datafusion::physical_plan::execution_plan::reset_plan_states(Arc::clone(
            &self.physical,
        ))
        .map_err(super::engine)?;
        self.ready = true;
        Ok(result)
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
        result
    }
    async fn next_owned(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Option<OwnedRecordBatch>, CatalogError> {
        let completed_cancel = CancellationToken::new();
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
        let next = tokio::select! {
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
        let cancel = if self.must_settle() {
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
        let state = super::execution::NativeExecutionContext::from_session(&self.state)
            .and_then(|services| services.command_state(&self.state))
            .map_err(super::engine)?;
        let created_memory_table = match &self.prepared.0.original {
            LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateMemoryTable(
                command,
            )) => {
                let defaults = &state.config_options().catalog;
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
            .capture_namespace(&state, created_memory_table.as_ref(), cancel)
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
