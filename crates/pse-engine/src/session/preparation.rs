// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The single native preparation/execution boundary. Constructors stay private;
//! a completed computation can only result from draining its prepared execution.

use super::assurance::{ObservationPolicy, OperationObservation, TerminalStatus};
use super::{EngineSession, PlanObservation, admission, engine, observation::Recorder};
use crate::EngineError;
use datafusion::{
    arrow::array::RecordBatch, common::tree_node::TreeNodeRecursion, logical_expr::LogicalPlan,
};
use pse_columnar::PlanOrigin;

use pse_columnar::{CancellationToken, owned_buffer::OwnedRecordBatch};
use std::sync::{Arc, Mutex};
use tracing::Instrument;

mod terminal;
pub use terminal::{DiagnosticSample, SampleRetention, TerminalDemand};

/// A native plan bound to actual immutable providers, functions and configuration.
/// The original and analyzed plans remain available after optimizer elimination.
/// This establishes preparation, not the absence of residual data obligations.
#[derive(Clone)]
pub struct PreparedComputation(Arc<Preparation>);
struct Preparation {
    session: EngineSession,
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
    terminal: TerminalDemand,
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
    observation: PlanObservation,
    checked: Mutex<Option<pse_relations::columnar::FieldCheckedBatch>>,
    state: datafusion::execution::session_state::SessionState,
}

impl EngineSession {
    /// One registry/budget-bound field admission scope for a complete artifact.
    /// Every graph still checks its sources and expressions; identical immutable
    /// Arrow child declarations are validated once across sibling outputs.
    /// # Errors
    /// Foreign sources, invalid native expressions or resource/cancellation refusal.
    pub fn derive_plan_fields_many(
        &self,
        plans: &[LogicalPlan],
        cancel: &CancellationToken,
    ) -> Result<Vec<LogicalPlan>, EngineError> {
        admission::restore_semantic_fields_many(
            &super::cache::rebind_many(plans, self, cancel).map_err(engine)?,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
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
    ) -> Result<LogicalPlan, EngineError> {
        admission::restore_semantic_fields(
            super::cache::rebind(plan, self, cancel).map_err(engine)?,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
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
    ) -> Result<PreparedComputation, EngineError> {
        self.prepare_with_origin(plan, cancel, PlanOrigin::RuleCompiler)
    }
    /// Bind and analyze a native logical plan without executing its data operators.
    /// # Errors
    /// A foreign binding, invalid field contract, native analysis or cancellation.
    pub fn prepare(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        self.prepare_with_origin(plan, cancel, PlanOrigin::Analytics)
    }

    /// Select the terminal before native analysis and optimization.
    /// # Errors
    /// Ordinary binding, policy, preparation or cancellation failure.
    pub fn prepare_terminal(
        &self,
        plan: LogicalPlan,
        terminal: TerminalDemand,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        self.prepare_terminal_with_origin(plan, terminal, cancel, PlanOrigin::Analytics)
    }
    /// Prepare compiler-generated terminal demand with native rule attribution.
    /// # Errors
    /// Ordinary generated-plan preparation failure.
    pub fn prepare_rule_terminal(
        &self,
        plan: LogicalPlan,
        terminal: TerminalDemand,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        self.prepare_terminal_with_origin(plan, terminal, cancel, PlanOrigin::RuleCompiler)
    }
    fn prepare_terminal_with_origin(
        &self,
        plan: LogicalPlan,
        terminal: TerminalDemand,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<PreparedComputation, EngineError> {
        self.bind_targets(&plan)?
            .execution_scope()?
            .prepare_scoped(plan, cancel, origin, None, terminal)
    }

    pub(super) fn prepare_with_origin(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<PreparedComputation, EngineError> {
        self.bind_targets(&plan)?.execution_scope()?.prepare_scoped(
            plan,
            cancel,
            origin,
            None,
            TerminalDemand::Stream,
        )
    }

    /// Open one accounted producer-planning scope for a group of immutable outputs.
    pub fn cache_planning_scope<'a>(
        &'a self,
        cancel: &'a CancellationToken,
    ) -> super::cache::logical::Scope<'a> {
        super::cache::logical::Scope::new(&self.pool, cancel)
    }

    /// Prepare one output using the shared producer-planning scope and exact bindings.
    /// # Errors
    /// Source admission, policy, cancellation or native planning failure.
    fn prepare_group_member(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        scope: &mut super::cache::logical::Scope<'_>,
    ) -> Result<PreparedComputation, EngineError> {
        // Only ArtifactPlan's immutable outputs, already derived and admitted
        // together against this exact captured session, use this private path.
        scope.bind_owner(self).map_err(engine)?;
        self.bind_targets(&plan)?.execution_scope()?.prepare_scoped(
            plan,
            cancel,
            PlanOrigin::RuleCompiler,
            Some(scope),
            TerminalDemand::Retain,
        )
    }

    /// Prepare related value and requirement roots together without executing them.
    /// Native producer ownership and local field admission are shared across roots;
    /// each root still receives its own policy, effects and query-time admission.
    /// # Errors
    /// Source admission, policy, cancellation or native planning failure.
    pub fn prepare_many(
        &self,
        plans: &[LogicalPlan],
        cancel: &CancellationToken,
    ) -> Result<Vec<PreparedComputation>, EngineError> {
        // Field derivation reconstructs native schemas. Retain explicit root
        // declarations before that pass, then check/redeclare their output fields.
        let declarations = plans
            .iter()
            .map(|plan| self.output_declaration(plan))
            .collect::<Vec<_>>();
        let plans = self.derive_plan_fields_many(plans, cancel)?;
        let mut scope = self.cache_planning_scope(cancel);
        plans
            .into_iter()
            .zip(declarations)
            .map(|(plan, declaration)| {
                let plan = if let Some(declaration) = declaration {
                    super::output::declare_relation_output(plan, &self.registry, declaration)
                        .map_err(|error| self.execution_error(error, PlanOrigin::RuleCompiler))?
                } else {
                    plan
                };
                self.prepare_group_member(plan, cancel, &mut scope)
            })
            .collect()
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

    fn bound_requirements(&self) -> Result<bool, EngineError> {
        let requirements = !self.effective_policy()?.requirements.is_empty();
        if requirements && self.requirement_planner.is_none() {
            return Err(EngineError::Admission {
                path: "provider.requirements".to_owned(),
                reason: "required invariants have no bound native implementation".to_owned(),
            });
        }
        Ok(requirements)
    }

    fn prepare_scoped(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
        cache_scope: Option<&mut super::cache::logical::Scope<'_>>,
        terminal: TerminalDemand,
    ) -> Result<PreparedComputation, EngineError> {
        cancel.checkpoint()?;
        let requirements = self.bound_requirements()?;
        // Native expressions already own their actual function implementations.
        // The SQL name registry is needed for resolution and diagnostic decoding,
        // not for restricting executable plans to registered function names.
        let output_relation = (terminal != TerminalDemand::Count)
            .then(|| self.output_declaration(&plan))
            .flatten();
        let plan = if cache_scope.is_some() {
            plan
        } else {
            admission::restore_semantic_fields(
                super::cache::rebind(plan, self, cancel).map_err(engine)?,
                &self.registry,
                &self.bindings.providers(),
                &self.pool,
                cancel,
            )
            .map_err(|error| {
                stage_error(
                    self.execution_error(error, origin),
                    "initial native field derivation",
                )
            })?
        };
        let (plan, mut volatile, mut effects) = self.admit_and_isolate(plan, cancel, origin)?;
        let pure = !requirements
            && !volatile
            && effects
                .iter()
                .all(|effect| *effect == pse_schema::model::provider::OperationEffect::Read);
        let original = plan.clone();
        let plan = terminal::specialize(plan, terminal, pure).map_err(engine)?;
        volatile |= self.bindings.iter().any(|(_, binding)| {
            binding
                .effects
                .contains(&pse_schema::model::provider::OperationEffect::Observe)
        });
        let state =
            super::execution::NativeExecutionContext::bind(self, self.bound_state()?, cancel)?;
        let mut recorder = Recorder::new(&self.pool, ObservationPolicy::from_state(&state));
        let mut optimize_producer = |input, required| {
            self.optimize_cache_producer(input, required, &state, cancel, &mut recorder)
        };
        let staged = if let Some(scope) = cache_scope {
            scope.stage(plan.clone(), &mut optimize_producer)
        } else {
            super::cache::logical::stage(plan.clone(), &self.pool, cancel, &mut optimize_producer)
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
        let visible_analyzed = super::cache::logical::expand(analyzed.clone(), &self.pool, cancel)
            .map_err(|error| self.execution_error(error, origin))?;
        let analyzed_freshness = self
            .plan_freshness([&visible_analyzed], cancel)
            .map_err(|error| self.execution_error(error, origin))?;
        effects.extend(self.admit_effects(&visible_analyzed, analyzed_freshness.nondeterministic)?);
        volatile |= analyzed_freshness.required;
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
            original,
            analyzed: visible_analyzed,
            optimized,
            observation,
            origin,
            volatile,
            started: std::sync::atomic::AtomicBool::new(false),
            effects,
            requirements,
            terminal,
        })))
    }
}

impl EngineSession {
    fn finalize_native_fields(
        &self,
        plan: LogicalPlan,
        output: Option<&pse_schema::model::RelationSpec>,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<LogicalPlan, EngineError> {
        let plan = admission::restore_semantic_fields(
            plan,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
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
        super::cache::logical::expand(plan, &self.pool, cancel)
            .map_err(|error| self.execution_error(error, origin))
    }

    fn admit_and_isolate(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<
        (
            LogicalPlan,
            bool,
            std::collections::BTreeSet<pse_schema::model::provider::OperationEffect>,
        ),
        EngineError,
    > {
        let freshness = self
            .plan_freshness([&plan], cancel)
            .map_err(|error| self.execution_error(error, origin))?;
        let volatile = freshness.required;
        let effects = self.admit_effects(&plan, freshness.nondeterministic)?;
        let plan = if effects.iter().any(|effect| {
            matches!(
                effect,
                pse_schema::model::provider::OperationEffect::Write
                    | pse_schema::model::provider::OperationEffect::Namespace
                    | pse_schema::model::provider::OperationEffect::Publish
            )
        }) || matches!(plan, LogicalPlan::Explain(_))
        {
            super::mutation::isolate(&plan, self, cancel).map_err(engine)?
        } else {
            plan
        };
        Ok((plan, volatile, effects))
    }
    fn optimize_cache_producer(
        &self,
        input: LogicalPlan,
        required: bool,
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
        let visible = super::cache::logical::expand(analyzed.clone(), &self.pool, cancel)?;
        if !required && self.plans_require_fresh([&visible], cancel)? {
            return Err(datafusion::common::DataFusionError::Plan(
                "native analysis introduced a varying or uncontracted cache producer".into(),
            ));
        }
        self.admit_effects(&visible, false)
            .map_err(pse_columnar::external)?;
        crate::cache_service::metrics::record(state, |metrics| &metrics.optimizations);
        let optimized = state
            .optimizer()
            .optimize(analyzed, state, |_, rule| recorder.rule(rule.name()))?;
        let optimized = admission::restore_semantic_fields(
            optimized,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
            cancel,
        )?;
        super::scalar::materialize_nested_fields(
            optimized,
            &self
                .scalar_function("pse_preserve_field")
                .map_err(pse_columnar::external)?,
        )
    }
}

impl PreparedComputation {
    pub(crate) fn bound_session(&self) -> EngineSession {
        self.0.session.clone()
    }
    pub(super) fn check_registry(
        &self,
        registry: &Arc<pse_schema::Registry>,
    ) -> Result<(), EngineError> {
        if !Arc::ptr_eq(&self.0.session.registry, registry) {
            return Err(EngineError::Admission {
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
    ) -> Result<CompletedComputation, EngineError> {
        self.execute_stream(cancel).await?.collect(cancel).await
    }

    /// Begin native execution without draining or accumulating the result stream.
    /// The stream retains its complete preparation, providers and resource owners.
    /// # Errors
    /// Planning, unsupported actual implementation, cancellation or policy failure.
    pub async fn execute_stream(
        self,
        cancel: &CancellationToken,
    ) -> Result<OwnedComputationStream, EngineError> {
        self.execute_stream_in_scope(cancel, None).await
    }

    /// Execute a group with one retained native producer cache and owned results.
    /// # Errors
    /// Any group member fails preparation, execution or ownership admission.
    pub async fn execute_group(
        preparations: impl IntoIterator<Item = Result<Self, EngineError>>,
        cancel: &CancellationToken,
    ) -> Result<Vec<CompletedComputation>, EngineError> {
        use futures_util::StreamExt;
        let mut preparations = preparations.into_iter();
        let Some(first) = preparations.next() else {
            return Ok(Vec::new());
        };
        let first = first?;
        let caches = first.0.session.invocation.clone().unwrap_or_default();
        let limit = first
            .0
            .state
            .config()
            .get_extension::<crate::cache_service::NativeCacheService>()
            .map_or(1, |service| service.policy().concurrent_outputs.get());
        let slots = pse_columnar::MemoryConsumer::new("native:output-completions")
            .register(&first.0.session.pool);
        let failed = std::sync::atomic::AtomicBool::new(false);
        let preparations = std::iter::once(Ok(first)).chain(preparations);
        let mut completed = futures_util::stream::iter(preparations.enumerate())
            .map(|(ordinal, prepared)| {
                let caches = caches.clone();
                let failed = &failed;
                let slots = &slots;
                async move {
                    if failed.load(std::sync::atomic::Ordering::Acquire) {
                        return (ordinal, None);
                    }
                    let result = async {
                        slots.try_grow(256).map_err(engine)?;
                        prepared?
                            .execute_stream_in_scope(cancel, Some(caches))
                            .await?
                            .collect(cancel)
                            .await
                    }
                    .await;
                    if result.is_err() {
                        failed.store(true, std::sync::atomic::Ordering::Release);
                    }
                    (ordinal, Some(result))
                }
            })
            .buffer_unordered(limit)
            .filter_map(|(ordinal, result)| async move { result.map(|result| (ordinal, result)) })
            .collect::<Vec<_>>()
            .await;
        completed.sort_unstable_by_key(|(ordinal, _)| *ordinal);
        settled_results(completed.into_iter().map(|(_, result)| result))
    }

    async fn execute_stream_in_scope(
        mut self,
        cancel: &CancellationToken,
        caches: Option<Arc<super::cache::CacheStore>>,
    ) -> Result<OwnedComputationStream, EngineError> {
        let mut operation =
            OperationObservation::start(ObservationPolicy::from_state(&self.0.state), "query");
        // Stable/ambient calls may already have become literals. Fresh task
        // properties cannot refresh those literals; replay the retained original.
        if self.0.volatile
            && self.0.effects.iter().all(|effect| {
                matches!(
                    effect,
                    pse_schema::model::provider::OperationEffect::Read
                        | pse_schema::model::provider::OperationEffect::Observe
                        | pse_schema::model::provider::OperationEffect::Nondeterministic
                )
            })
            && self
                .0
                .started
                .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            self = self
                .0
                .session
                .prepare_terminal_with_origin(
                    self.0.original.clone(),
                    self.0.terminal,
                    cancel,
                    self.0.origin,
                )
                .inspect_err(|_| {
                    operation.finish(if cancel.is_cancelled() {
                        TerminalStatus::Cancelled
                    } else {
                        TerminalStatus::Failed
                    });
                })?;
        }
        let query_permit = query_admission(&self.0.state, cancel)
            .await
            .inspect_err(|_| {
                operation.finish(if cancel.is_cancelled() {
                    TerminalStatus::Cancelled
                } else {
                    TerminalStatus::Failed
                });
            })?;
        let result = self
            .plan_execution(cancel, false, caches)
            .instrument(operation.span())
            .await;
        let (physical, execution_state, observation) = match result {
            Ok(value) => value,
            Err(error) => {
                operation.finish(if cancel.is_cancelled() {
                    TerminalStatus::Cancelled
                } else {
                    TerminalStatus::Failed
                });
                return Err(error);
            }
        };
        register_query_admission(&execution_state, query_permit.as_ref());
        let session = &self.0.session;
        let result = operation.span().in_scope(|| {
            datafusion::physical_plan::execute_stream(
                Arc::clone(&physical),
                execution_state.task_ctx(),
            )
        });
        let stream = match result {
            Ok(stream) => stream,
            Err(error) => {
                operation.finish(TerminalStatus::Failed);
                observation.complete(physical.as_ref(), TerminalStatus::Failed, &session.pool);
                return Err(session.execution_error(error, self.0.origin));
            }
        };
        Ok(OwnedComputationStream {
            capture: super::observation::CompletionGuard::new(
                &physical,
                &observation,
                &session.pool,
            ),
            query_permit,
            operation,
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
    ) -> Result<ReusableComputation, EngineError> {
        self.prepare_reusable_with_caches(cancel, None).await
    }
    async fn prepare_reusable_with_caches(
        self,
        cancel: &CancellationToken,
        caches: Option<Arc<super::cache::CacheStore>>,
    ) -> Result<ReusableComputation, EngineError> {
        if self.0.volatile
            || self
                .0
                .effects
                .iter()
                .any(|effect| *effect != pse_schema::model::provider::OperationEffect::Read)
        {
            return Err(engine(datafusion::common::DataFusionError::Plan(
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
            .map_err(engine)?;
        let (physical, state, observation) = self.plan_execution(cancel, true, caches).await?;
        super::round::validate_reset_plan(
            &physical,
            state
                .config()
                .get_extension::<super::round::RoundResetSupport>()
                .as_deref(),
            &super::round::reset_partitions(&self.0.session),
        )
        .map_err(engine)?;
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
        EngineError,
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
        let native = self.0.optimized.clone();
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
        }
        .map_err(engine)?;
        if reusable {
            let mut config = execution_state.config().clone();
            let options = config.options_mut();
            options.optimizer.enable_dynamic_filter_pushdown = false;
            options.optimizer.enable_join_dynamic_filter_pushdown = false;
            options.optimizer.enable_topk_dynamic_filter_pushdown = false;
            options.optimizer.enable_aggregate_dynamic_filter_pushdown = false;
            *execution_state.config_mut() = config;
        }
        let state = &execution_state;
        if effectful
            && self
                .0
                .started
                .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return Err(EngineError::Admission {
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
        let mut recorder = Recorder::new(
            &session.pool,
            ObservationPolicy::from_state(&execution_state),
        );
        for rule in self.0.observation.rules_fired() {
            recorder.rule(rule);
        }
        let observation = recorder
            .finish(&execution_plan, cancel)?
            .with_physical(physical.as_ref(), &session.pool)?;
        session
            .requires_fresh
            .fetch_or(self.0.volatile, std::sync::atomic::Ordering::AcqRel);
        session.trace.record(observation.clone())?;
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
    ) -> Result<CompletedComputation, EngineError> {
        let mut operation =
            OperationObservation::start(ObservationPolicy::from_state(&self.state), "round");
        let result = self
            .execute_round(cancel, true)
            .instrument(operation.span())
            .await;
        operation.finish(match &result {
            Ok(_) => TerminalStatus::Completed,
            Err(_) if cancel.is_cancelled() => TerminalStatus::Cancelled,
            Err(_) => TerminalStatus::Failed,
        });
        result
    }

    async fn execute_round(
        &mut self,
        cancel: &CancellationToken,
        advance: bool,
    ) -> Result<CompletedComputation, EngineError> {
        if !self.ready {
            return Err(engine(datafusion::common::DataFusionError::Execution(
                "reusable execution has an unfinished or failed epoch".into(),
            )));
        }
        self.ready = false;
        crate::cache_service::metrics::record(&self.state, |metrics| &metrics.reusable_executions);
        cancel.checkpoint()?;
        unchanged_namespace(&self.prepared)?;
        if advance {
            super::execution::NativeExecutionContext::from_session(&self.state)
                .map_err(engine)?
                .advance_round_epoch();
        }
        let query_permit = query_admission(&self.state, cancel).await?;
        register_query_admission(&self.state, query_permit.as_ref());
        let mut operation =
            OperationObservation::start(ObservationPolicy::from_state(&self.state), "round_stream");
        let observation = self
            .observation
            .with_physical(self.physical.as_ref(), &self.prepared.0.session.pool)?;
        self.prepared.0.session.trace.record(observation.clone())?;
        let stream = operation
            .span()
            .in_scope(|| {
                datafusion::physical_plan::execute_stream(
                    Arc::clone(&self.physical),
                    self.state.task_ctx(),
                )
            })
            .map_err(|error| {
                operation.finish(TerminalStatus::Failed);
                observation.complete(
                    self.physical.as_ref(),
                    TerminalStatus::Failed,
                    &self.prepared.0.session.pool,
                );
                engine(error)
            })?;
        let result = OwnedComputationStream {
            capture: super::observation::CompletionGuard::new(
                &self.physical,
                &observation,
                &self.prepared.0.session.pool,
            ),
            query_permit,
            operation,
            prepared: self.prepared.clone(),
            physical: Arc::clone(&self.physical),
            observation,
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
        .map_err(engine)?;
        self.ready = true;
        Ok(result)
    }
}

/// Related pure finite roots prepared against one selection and executed in one
/// epoch. All streams settle before any completed result is returned to the caller.
#[derive(Debug)]
pub struct ReusableGroup {
    members: Vec<ReusableComputation>,
    caches: Arc<super::cache::CacheStore>,
    ready: bool,
}
impl ReusableGroup {
    /// Prepare related roots with shared native producer completions.
    /// # Errors
    /// Incompatible selections, empty roots or any ordinary reusable-plan refusal.
    pub async fn prepare(
        roots: Vec<PreparedComputation>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        let first = roots.first().ok_or_else(|| {
            engine(datafusion::common::DataFusionError::Plan(
                "reusable group is empty".into(),
            ))
        })?;
        let selection = first.0.session.selection()?;
        if roots
            .iter()
            .any(|root| !selection.matches_owners(&root.0.session))
        {
            return Err(engine(datafusion::common::DataFusionError::Plan(
                "reusable roots have different selections".into(),
            )));
        }
        let caches = Arc::new(super::cache::CacheStore::default());
        let mut members = Vec::with_capacity(roots.len());
        for root in roots {
            members.push(
                root.prepare_reusable_with_caches(cancel, Some(caches.clone()))
                    .await?,
            );
        }
        Ok(Self {
            members,
            caches,
            ready: true,
        })
    }
    /// Execute all members atomically with respect to returned completion evidence.
    /// # Errors
    /// A failed/unfinished epoch, cancellation, reset or native execution failure.
    pub async fn execute(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Vec<CompletedComputation>, EngineError> {
        use futures_util::{FutureExt, StreamExt};
        if !self.ready {
            return Err(engine(datafusion::common::DataFusionError::Execution(
                "reusable group has a failed epoch".into(),
            )));
        }
        self.ready = false;
        self.caches.advance_epoch();
        let limit = self
            .members
            .first()
            .and_then(|member| {
                member
                    .state
                    .config()
                    .get_extension::<crate::cache_service::NativeCacheService>()
            })
            .map_or(1, |service| service.policy().concurrent_outputs.get());
        let failed = std::sync::atomic::AtomicBool::new(false);
        let members = std::mem::take(&mut self.members);
        let slots = members.first().map(|member| {
            pse_columnar::MemoryConsumer::new("native:round-output-completions")
                .register(&member.prepared.0.session.pool)
        });
        if let Some(slots) = &slots {
            slots
                .try_grow(
                    members
                        .len()
                        .saturating_mul(size_of::<ReusableComputation>() + 512),
                )
                .map_err(engine)?;
        }
        let mut completed = futures_util::stream::iter(members.into_iter().enumerate())
            .map(|(ordinal, mut member)| {
                let failed = &failed;
                async move {
                    if failed.load(std::sync::atomic::Ordering::Acquire) {
                        return (ordinal, member, None);
                    }
                    let result = member.execute_round(cancel, false).await;
                    if result.is_err() {
                        failed.store(true, std::sync::atomic::Ordering::Release);
                    }
                    (ordinal, member, Some(result))
                }.boxed()
            })
            // Refill any completed slot; restore declaration order after settlement.
            .buffer_unordered(limit)
            .collect::<Vec<_>>()
            .await;
        completed.sort_unstable_by_key(|(ordinal, _, _)| *ordinal);
        let values = settled_results(completed.into_iter().filter_map(|(_, member, result)| {
            self.members.push(member);
            result
        }))?;
        self.ready = true;
        Ok(values)
    }
}

// Called only after every started output has settled, in declaration order.
fn settled_results(
    results: impl IntoIterator<Item = Result<CompletedComputation, EngineError>>,
) -> Result<Vec<CompletedComputation>, EngineError> {
    let mut values = Vec::new();
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(value) => values.push(value),
            Err(error) => errors.push(pse_columnar::external(error)),
        }
    }
    if errors.is_empty() {
        Ok(values)
    } else {
        Err(engine(datafusion::common::DataFusionError::Collection(
            errors,
        )))
    }
}

/// A native stream with explicit batch ownership and no implicit full-result collection.
/// Dropping it cancels unconsumed work and releases its retained resources.
pub struct OwnedComputationStream {
    query_permit: Option<Arc<tokio::sync::OwnedSemaphorePermit>>,
    operation: OperationObservation,
    prepared: PreparedComputation,
    physical: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    observation: PlanObservation,
    stream: Option<datafusion::physical_plan::SendableRecordBatchStream>,
    // Fields drop in declaration order: flush native stream metrics before the
    // abandoned snapshot releases its final physical-plan owner.
    capture: super::observation::CompletionGuard,
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
    pub fn with_batch_size(mut self, size: std::num::NonZeroUsize) -> Result<Self, EngineError> {
        if self.yielded || self.finished {
            return Err(EngineError::Admission {
                path: "operation.stream".into(),
                reason: "select batch size before consuming the stream".into(),
            });
        }
        let input = self.stream.take().ok_or_else(|| EngineError::Admission {
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
    ) -> Result<Option<OwnedRecordBatch>, EngineError> {
        let span = self.operation.span();
        let result = self.next_owned(cancel).instrument(span).await;
        if result.is_err() {
            let terminal = if cancel.is_cancelled() {
                TerminalStatus::Cancelled
            } else {
                TerminalStatus::Failed
            };
            self.operation.finish(terminal);
            self.stream = None;
            self.query_permit.take();
            self.capture.finish(terminal);
        } else if matches!(result, Ok(None)) {
            self.operation.finish(TerminalStatus::Completed);
            self.capture.finish(TerminalStatus::Completed);
        }
        result
    }
    async fn next_owned(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<Option<OwnedRecordBatch>, EngineError> {
        let completed_cancel = CancellationToken::new();
        unchanged_namespace(&self.prepared)?;
        if self.finished {
            if !self.must_settle() {
                cancel.checkpoint()?;
            }
            return Ok(None);
        }
        let stream = self.stream.as_mut().ok_or_else(|| EngineError::Admission {
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
            self.query_permit.take();
            self.stream = None;
            return Ok(None);
        };
        let session = &self.prepared.0.session;
        let batch = batch.map_err(|error| {
            session.execution_error(
                crate::operation::completion::stream_error(error, self.yielded),
                self.prepared.0.origin,
            )
        })?;
        let cancel = if self.must_settle() {
            &completed_cancel
        } else {
            cancel
        };
        self.yielded = true;
        let services =
            super::execution::NativeExecutionContext::from_session(&self.state).map_err(engine)?;
        Ok(Some(services.ownership().export(
            batch,
            &session.pool,
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
    ) -> Result<CompletedComputation, EngineError> {
        if self.yielded {
            return Err(EngineError::Admission {
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
            observation: self.observation,
            checked: Mutex::new(None),
            state: self.state,
        })
    }
}

fn unchanged_namespace(prepared: &PreparedComputation) -> Result<(), EngineError> {
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
        return Err(EngineError::Admission {
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
    ) -> Result<EngineSession, EngineError> {
        cancel.checkpoint()?;
        let state = super::execution::NativeExecutionContext::from_session(&self.state)
            .and_then(|services| services.command_state(&self.state))
            .map_err(engine)?;
        let created_memory_table = super::commands::deferred::created_memory_table(
            &self.prepared.0.original,
        )
        .map(|command| {
            let defaults = &state.config_options().catalog;
            let name = command
                .clone()
                .resolve(&defaults.default_catalog, &defaults.default_schema);
            datafusion::common::TableReference::full(name.catalog, name.schema, name.table)
        });
        self.prepared
            .0
            .session
            .capture_namespace(&state, created_memory_table.as_ref(), cancel)
            .await
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
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, EngineError> {
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
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, EngineError> {
        cancel.checkpoint()?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(registry, spec)
                .map_err(pse_relations::RelationError::from)?,
        );
        if self.prepared.optimized_plan().schema().as_arrow() != schema.as_ref() {
            return Err(EngineError::Admission {
                path: spec.key.qualified_name(),
                reason: "native output plan is not bound to this exact declared relation"
                    .to_owned(),
            });
        }
        let mut checked = self.checked.lock().map_err(|_| EngineError::Internal {
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
            .collect::<Result<Vec<_>, EngineError>>()?;
        let batch = pse_relations::columnar::FieldCheckedBatch::concat_reserved(
            registry,
            spec,
            &pieces,
            &self.prepared.0.session.pool,
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

fn unreachable_cancel<T>() -> Result<T, EngineError> {
    Err(EngineError::ConfigInvalid {
        key: "runtime.cancellation".to_owned(),
        reason: "cancellation notification did not retain its state".to_owned(),
    })
}
fn stage_error(error: EngineError, stage: &str) -> EngineError {
    match error {
        EngineError::Internal { message } => EngineError::Internal {
            message: format!("{stage}: {message}"),
        },
        EngineError::UserModel { message } => EngineError::UserModel {
            message: format!("{stage}: {message}"),
        },
        other => other,
    }
}

async fn query_admission(
    state: &datafusion::execution::session_state::SessionState,
    cancel: &CancellationToken,
) -> Result<Option<Arc<tokio::sync::OwnedSemaphorePermit>>, EngineError> {
    if let Some(permit) = state
        .config()
        .get_extension::<super::execution::NativeExecutionContext>()
        .and_then(|services| services.inherited_query_admission())
    {
        return Ok(Some(permit));
    }
    if let Some(cpu) = state
        .config()
        .get_extension::<crate::resources::CpuAdmission>()
    {
        // Refuse saturation instead of accumulating an unbounded async wait queue.
        cancel.checkpoint()?;
        let permit = cpu
            .permits
            .clone()
            .try_acquire_many_owned(
                u32::try_from(state.config().target_partitions())
                    .unwrap_or(u32::MAX)
                    .min(cpu.workers.get()),
            )
            .map_err(|_| EngineError::ResourceLimit {
                consumer: "deployment CPU".into(),
                config_keys: vec![],
                detail: "deployment worker admission is saturated".into(),
            })?;
        return Ok(Some(Arc::new(permit)));
    }
    match state
        .config()
        .get_extension::<crate::cache_service::NativeCacheService>()
    {
        Some(service) => service
            .admit_query(cancel)
            .await
            .map(|permit| Some(Arc::new(permit)))
            .map_err(engine),
        None => Ok(None),
    }
}
fn register_query_admission(
    state: &datafusion::execution::session_state::SessionState,
    permit: Option<&Arc<tokio::sync::OwnedSemaphorePermit>>,
) {
    if let Some(permit) = permit
        && let Some(services) = state
            .config()
            .get_extension::<super::execution::NativeExecutionContext>()
    {
        services.register_query_admission(permit);
    }
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    use datafusion::execution::{config::SessionConfig, session_state::SessionStateBuilder};
    #[tokio::test]
    async fn roots_admit_partition_capacity_and_nested_work_borrows_the_owner() {
        let cpu = Arc::new(tokio::sync::Semaphore::new(2));
        let factory = super::super::EngineFactory::from_builder(
            Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            "cpu-unit",
            SessionStateBuilder::new_with_default_features().with_config(
                SessionConfig::new()
                    .with_target_partitions(8)
                    .with_extension(Arc::new(crate::resources::CpuAdmission {
                        permits: cpu.clone(),
                        workers: std::num::NonZeroU32::new(2).unwrap(),
                    })),
            ),
        );
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                std::collections::BTreeMap::new(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let state = super::super::execution::NativeExecutionContext::bind(
            &session,
            session.bound_state().unwrap(),
            &cancel,
        )
        .unwrap();
        let permit = query_admission(&state, &cancel).await.unwrap().unwrap();
        register_query_admission(&state, Some(&permit));
        assert_eq!(cpu.available_permits(), 0);
        assert!(matches!(
            query_admission(&state, &cancel).await,
            Err(EngineError::ResourceLimit { .. })
        ));
        let services =
            super::super::execution::NativeExecutionContext::from_session(&state).unwrap();
        let child = services
            .candidate_roles(&state, std::collections::BTreeMap::new())
            .unwrap();
        let nested = super::super::execution::NativeExecutionContext::bind(
            &child,
            child.bound_state().unwrap(),
            &cancel,
        )
        .unwrap();
        let borrowed = query_admission(&nested, &cancel).await.unwrap().unwrap();
        assert!(Arc::ptr_eq(&permit, &borrowed));
        drop(permit);
        assert_eq!(cpu.available_permits(), 0);
        drop(borrowed);
        assert_eq!(cpu.available_permits(), 2);
    }
    #[tokio::test]
    async fn root_admission_uses_effective_session_partitions_and_releases_on_drop() {
        let permits = Arc::new(tokio::sync::Semaphore::new(2));
        let state = SessionStateBuilder::new_with_default_features()
            .with_config(
                SessionConfig::new()
                    .with_target_partitions(1)
                    .with_extension(Arc::new(crate::resources::CpuAdmission {
                        permits: permits.clone(),
                        workers: std::num::NonZeroU32::new(2).unwrap(),
                    })),
            )
            .build();
        let cancel = CancellationToken::new();
        let owner = query_admission(&state, &cancel).await.unwrap().unwrap();
        assert_eq!(permits.available_permits(), 1);
        cancel.cancel();
        assert!(query_admission(&state, &cancel).await.is_err());
        assert_eq!(permits.available_permits(), 1);
        drop(owner);
        assert_eq!(permits.available_permits(), 2);
    }
}
