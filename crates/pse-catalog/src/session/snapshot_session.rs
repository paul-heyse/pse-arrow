// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Sealed session facade and bounded result export.

use super::{
    ExecutionSettings, ThreadBudget,
    admission::{self, AdmissionRule},
    candidate::CandidateTable,
    config,
    profile::{EngineProfile, RuleCatalog},
};
use crate::provider::{catalog::SnapshotCatalog, list::SnapshotCatalogList, table::RelationTable};
use crate::{CatalogError, PlanOrigin, Snapshot};
use datafusion::arrow::{
    ARROW_VERSION,
    array::{Array, RecordBatch, StringArray},
};
use datafusion::catalog::{CatalogProvider, TableProvider};
use datafusion::common::{DataFusionError, TableReference};
use datafusion::datasource::provider_as_source;
use datafusion::execution::{
    context::{SQLOptions, SessionContext},
    runtime_env::RuntimeEnv,
    session_state::SessionStateBuilder,
};
use datafusion::logical_expr::{LogicalPlan, TableSource};
use pse_ids::{CancellationToken, ContentHash, FramedHasher, MemoryReserver, derive::context};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A read-only engine session pinned to an exact admitted or unpublished input inventory.
pub struct SnapshotSession {
    pub(super) context: SessionContext,
    pub(super) registry: Arc<Registry>,
    pub(super) sources: BTreeMap<RelationKey, (TableReference, Arc<dyn TableProvider>)>,
    pub(super) tables: Vec<Arc<dyn TableProvider>>,
    pub(super) reserver: Arc<dyn MemoryReserver>,
    pub(super) function_bindings: Arc<super::functions::Functions>,
    profile: EngineProfile,
    settings: BTreeMap<String, Option<String>>,
    functions: ContentHash,
    function_names: BTreeMap<String, Vec<String>>,
}
/// Build a session over exact admitted snapshots under the sealed `model` catalog.
/// # Errors
/// Missing/ambiguous members, invalid declarations, or unsupported explicit engine settings.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the public construction facade consumes explicit settings; the shared synchronous assembler borrows them during construction"
)]
pub fn build_session(
    snapshots: Vec<Arc<Snapshot>>,
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
) -> Result<SnapshotSession, CatalogError> {
    let mut tables = BTreeMap::new();
    for snapshot in snapshots {
        for relation in snapshot.relations().values() {
            let key = registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .ok_or_else(|| invalid("snapshot relation missing from registry"))?
                .key;
            let table: Arc<dyn TableProvider> = Arc::new(RelationTable::new(
                Arc::clone(&snapshot),
                relation.contract().canonical.relation_id,
                &registry,
                Arc::clone(&reserver),
            )?);
            if tables.insert(key, table).is_some() {
                return Err(invalid(
                    "two snapshot bindings supply the same relation; bind one explicit version",
                ));
            }
        }
    }
    build(
        tables,
        registry,
        runtime,
        reserver,
        &settings,
        budget,
        profile,
        &CancellationToken::default(),
    )
}
/// Build the constraint-free P2 input session. Duplicate keys remain visible to rules.
/// # Errors
/// Exact field/value admission fails; no snapshot or memo identity is assigned here.
pub fn build_candidate_session(
    rows: BTreeMap<RelationKey, RecordBatch>,
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
) -> Result<SnapshotSession, CatalogError> {
    build_candidate_session_with_cancel(
        rows,
        registry,
        runtime,
        reserver,
        settings,
        budget,
        profile,
        &CancellationToken::default(),
    )
}
#[expect(
    clippy::too_many_arguments,
    reason = "compatibility facade forwards exact resources and caller cancellation to shared builder"
)]
/// Construct a candidate session retaining the caller's cooperative cancellation token.
/// # Errors
/// Invalid schema/values, explicit engine configuration or bounded allocation failure.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the public construction facade consumes explicit settings; the shared synchronous assembler borrows them during construction"
)]
pub fn build_candidate_session_with_cancel(
    rows: BTreeMap<RelationKey, RecordBatch>,
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CatalogError> {
    let mut tables = BTreeMap::new();
    for (key, batch) in rows {
        cancel.checkpoint()?;
        let spec = registry
            .relation(&key.qualified_name())
            .filter(|spec| spec.key == key)
            .ok_or_else(|| invalid("candidate relation/version is undeclared"))?;
        admission::validate_batch(&registry, spec, &batch, reserver.as_ref(), cancel)?;
        let batch = pse_ids::owned_buffer::copy_batch(&batch, reserver.as_ref(), cancel)?;
        let table: Arc<dyn TableProvider> = Arc::new(CandidateTable { key, batch });
        tables.insert(key, table);
    }
    build(
        tables, registry, runtime, reserver, &settings, budget, profile, cancel,
    )
}
#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "sealing exact sources, resources and caller cancellation is one construction boundary"
)]
fn build(
    tables: BTreeMap<RelationKey, Arc<dyn TableProvider>>,
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: &ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CatalogError> {
    let config = config::build(settings, budget)?;
    let settings = config
        .options()
        .entries()
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect();
    let settings = config::semantic_settings(&settings)?;
    let mut state_builder = SessionStateBuilder::new().with_default_features();
    let function_bindings = super::functions::Functions::from_builder(&mut state_builder);
    let providers: Vec<_> = tables.values().cloned().collect();
    let first: Arc<dyn datafusion::optimizer::AnalyzerRule + Send + Sync> =
        Arc::new(AdmissionRule {
            name: "pse.admission.first",
            registry: Arc::clone(&registry),
            tables: providers.clone(),
            functions: Arc::clone(&function_bindings),
            reserver: Arc::clone(&reserver),
            cancel: cancel.clone(),
        });
    let mut analyzers = vec![first];
    for name in &profile.analyzer_rules {
        analyzers.push(RuleCatalog::analyzer(name)?);
    }
    analyzers.push(Arc::new(AdmissionRule {
        name: "pse.admission.final",
        registry: Arc::clone(&registry),
        tables: providers.clone(),
        functions: Arc::clone(&function_bindings),
        reserver: Arc::clone(&reserver),
        cancel: cancel.clone(),
    }));
    let optimizers = profile
        .optimizer_rules
        .iter()
        .map(|name| RuleCatalog::optimizer(name))
        .collect::<Result<Vec<_>, _>>()?;
    let physical = profile
        .physical_optimizer_rules
        .iter()
        .map(|name| RuleCatalog::physical(name))
        .collect::<Result<Vec<_>, _>>()?;
    let inventory = tables
        .iter()
        .map(|(key, table)| {
            (
                (key.namespace.as_str().to_owned(), key.name.to_owned()),
                Arc::clone(table),
            )
        })
        .collect();
    let catalog: Arc<dyn CatalogProvider> = Arc::new(SnapshotCatalog::from_tables(&inventory));
    let list = SnapshotCatalogList::new(BTreeMap::from([("model".to_owned(), catalog)]));
    let state = state_builder
        .with_config(config)
        .with_runtime_env(runtime)
        .with_catalog_list(Arc::new(list))
        .with_extension_type_registry(super::registry::build(&registry).map_err(engine)?)
        .with_analyzer_rules(analyzers)
        .with_optimizer_rules(optimizers)
        .with_physical_optimizer_rules(physical)
        .build();
    let mut hash = FramedHasher::new(context::SETTINGS);
    hash.str("pse.functions.builtin.v1");
    hash.str(datafusion::DATAFUSION_VERSION);
    let mut function_names = BTreeMap::new();
    for (kind, names) in [
        (
            "scalar",
            state.scalar_functions().keys().cloned().collect::<Vec<_>>(),
        ),
        (
            "aggregate",
            state.aggregate_functions().keys().cloned().collect(),
        ),
        ("window", state.window_functions().keys().cloned().collect()),
        (
            "higher_order",
            state.higher_order_functions().keys().cloned().collect(),
        ),
    ] {
        let mut names = names;
        names.sort();
        function_names.insert(kind.to_owned(), names.clone());
        hash.str(kind);
        for name in names {
            hash.str(&name);
        }
    }
    let functions = hash.finish_hash();
    let sources = tables
        .into_iter()
        .map(|(key, table)| {
            (
                key,
                (
                    TableReference::full("model", key.namespace.as_str(), key.name),
                    table,
                ),
            )
        })
        .collect();
    Ok(SnapshotSession {
        context: SessionContext::new_with_state(state),
        registry,
        sources,
        tables: providers,
        function_bindings,
        reserver,
        profile,
        settings,
        functions,
        function_names,
    })
}
/// Actual sealed engine semantics, suitable for direct dependency comparison.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSemantics {
    /// Pinned engine version defining built-in implementations.
    pub engine_version: String,
    /// Pinned Arrow type implementation version.
    pub arrow_version: String,
    /// Ordered rule implementation names and profile contract version.
    pub profile: EngineProfile,
    /// Actual selected semantic settings, including explicit absent values.
    pub settings: BTreeMap<String, Option<String>>,
    /// Complete built-in function names by implementation family.
    pub functions: BTreeMap<String, Vec<String>>,
}
impl SnapshotSession {
    fn execution_error(&self, error: DataFusionError, origin: PlanOrigin) -> CatalogError {
        let mut errors = crate::classify(error, origin);
        if matches!(
            self.context.runtime_env().memory_pool.memory_limit(),
            datafusion::execution::memory_pool::MemoryLimit::Finite(_)
        ) {
            for error in &mut errors {
                add_bound_pool_key(error);
            }
        }
        crate::failure::collapse_classified(errors)
    }
    /// Compare the complete caller binding with retained actual providers, including values.
    /// # Errors
    /// Missing/extra relation bindings or any actual schema/value disagreement.
    pub fn validate_bindings(
        &self,
        rows: &BTreeMap<RelationKey, RecordBatch>,
    ) -> Result<(), CatalogError> {
        if rows.keys().ne(self.sources.keys()) {
            return Err(invalid(
                "caller relation inventory differs from sealed session",
            ));
        }
        for (key, batch) in rows {
            let provider = &self.sources[key].1;
            let retained = if let Some(table) = provider.as_ref().downcast_ref::<CandidateTable>() {
                &table.batch
            } else if let Some(table) = provider.as_ref().downcast_ref::<RelationTable>() {
                table.relation().batch()
            } else {
                return Err(invalid("unknown retained provider"));
            };
            if retained != batch {
                return Err(invalid(
                    "caller rows differ from actual sealed provider rows",
                ));
            }
        }
        Ok(())
    }
    /// Shared accounted allocator for typed rule outputs.
    pub fn reserver(&self) -> &dyn MemoryReserver {
        self.reserver.as_ref()
    }
    /// A checked heap bound for cloning the actual borrowed engine semantic inputs.
    /// Producers reserve this before calling [`Self::semantic_inputs`].
    /// # Errors
    /// Addressable allocation-size overflow.
    pub fn semantic_inputs_extent(&self) -> Result<usize, CatalogError> {
        crate::store::stage_context::engine_inputs_extent(
            datafusion::DATAFUSION_VERSION.len(),
            ARROW_VERSION.len(),
            &self.profile,
            &self.settings,
            &self.function_names,
        )
    }
    /// Actual inputs behind the diagnostic profile identities; no hash admits reuse.
    pub fn semantic_inputs(&self) -> SessionSemantics {
        SessionSemantics {
            engine_version: datafusion::DATAFUSION_VERSION.to_owned(),
            arrow_version: ARROW_VERSION.to_owned(),
            profile: self.profile.clone(),
            settings: self.settings.clone(),
            functions: self.function_names.clone(),
        }
    }

    /// Exact registry used at all semantic boundaries.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }
    /// Resolve the actual source object under a declared relation key.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_source(&self, key: &RelationKey) -> Result<Arc<dyn TableSource>, CatalogError> {
        self.sources
            .get(key)
            .map(|(_, table)| provider_as_source(Arc::clone(table)))
            .ok_or_else(|| invalid("undeclared session read"))
    }
    /// The SQL/plan reference bound to this exact relation version.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_reference(&self, key: &RelationKey) -> Result<TableReference, CatalogError> {
        self.sources
            .get(key)
            .map(|(reference, _)| reference.clone())
            .ok_or_else(|| invalid("undeclared session read"))
    }
    /// Semantic setting identity from the immutable configured state.
    pub fn settings_hash(&self) -> ContentHash {
        config::settings_hash(&self.settings)
    }
    /// Complete engine profile and semantic-setting identity.
    pub fn profile_hash(&self) -> ContentHash {
        self.profile.hash(self.settings_hash())
    }
    /// Identity of the pinned built-in function registry; no runtime registration is exposed.
    pub const fn function_registry_hash(&self) -> ContentHash {
        self.functions
    }
    /// Plan and execute read-only SQL through the same admission route as rule plans.
    /// # Errors
    /// Syntax, unsupported writes, semantic admission, cancellation or resource exhaustion.
    pub async fn sql(
        &self,
        sql: &str,
        cancel: &CancellationToken,
    ) -> Result<Vec<RecordBatch>, CatalogError> {
        cancel.checkpoint()?;
        let options = SQLOptions::new()
            .with_allow_ddl(false)
            .with_allow_dml(false)
            .with_allow_statements(false);
        let frame = self
            .context
            .sql_with_options(sql, options)
            .await
            .map_err(engine)?;
        self.execute_plan(frame.logical_plan().clone(), cancel)
            .await
    }
    /// Execute a caller-supplied plan only after checking every source and nested field.
    /// # Errors
    /// Invalid provider/context/metadata, engine execution, cancellation, or budget failure.
    pub async fn execute_plan(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<Vec<RecordBatch>, CatalogError> {
        self.execute_with_origin(plan, cancel, PlanOrigin::Analytics)
            .await
            .map(|(batches, _)| batches)
    }
    /// Execute a generated rule plan with platform-invariant error classification.
    /// # Errors
    /// Invalid rule plan, semantic admission, engine execution, cancellation or budget failure.
    pub async fn execute_rule_plan(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<Vec<RecordBatch>, CatalogError> {
        self.execute_with_origin(plan, cancel, PlanOrigin::RuleCompiler)
            .await
            .map(|(batches, _)| batches)
    }
    /// Execute a rule and retain actual optimizer callbacks and its executed plan.
    /// # Errors
    /// Admission, engine, cancellation, result budget or diagnostic budget failures.
    pub async fn execute_rule_observed(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
    ) -> Result<(Vec<RecordBatch>, super::PlanObservation), CatalogError> {
        self.execute_with_origin(plan, cancel, PlanOrigin::RuleCompiler)
            .await
    }
    async fn execute_with_origin(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<(Vec<RecordBatch>, super::PlanObservation), CatalogError> {
        cancel.checkpoint()?;
        self.function_bindings
            .admit_plan(&plan)
            .map_err(|error| self.execution_error(error, origin))?;
        admission::admit_plan(
            &plan,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        let state = self.context.state();
        let mut recorder = super::observation::Recorder::new(self.reserver.as_ref());
        let analyzed = state
            .analyzer()
            .execute_and_check(plan, state.config_options(), |_, rule| {
                recorder.rule(rule.name());
            })
            .map_err(|error| self.execution_error(error, origin))?;
        let optimized = state
            .optimizer()
            .optimize(analyzed, &state, |_, rule| recorder.rule(rule.name()))
            .map_err(|error| self.execution_error(error, origin))?;
        let optimized = admission::restore_semantic_fields(
            optimized,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        self.function_bindings
            .admit_plan(&optimized)
            .map_err(|error| self.execution_error(error, origin))?;
        admission::admit_plan(
            &optimized,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(|error| self.execution_error(error, origin))?;
        let observed = recorder.finish(&optimized)?;
        cancel.checkpoint()?;
        // Call the physical planner directly: the observed optimization is the one used.
        let physical = state
            .query_planner()
            .create_physical_plan(&optimized, &state)
            .await
            .map_err(|error| self.execution_error(error, origin))?;
        let mut stream =
            datafusion::physical_plan::execute_stream(physical, self.context.task_ctx())
                .map_err(|error| self.execution_error(error, origin))?;
        let mut batches = Vec::new();
        loop {
            cancel.checkpoint()?;
            let next = std::future::poll_fn(|context| stream.as_mut().poll_next(context)).await;
            let Some(batch) = next else {
                break;
            };
            let batch = batch.map_err(|error| self.execution_error(error, origin))?;
            for field in batch.schema().fields() {
                admission::admit_field(&self.registry, field)
                    .map_err(|error| self.execution_error(error, origin))?;
            }
            batches.push(pse_ids::owned_buffer::export_query_batch(
                batch,
                self.reserver.as_ref(),
                cancel,
            )?);
        }
        Ok((batches, observed))
    }
    /// Read the actual `information_schema.df_settings` view from a clone of the sealed state.
    ///
    /// Only this fixed trusted introspection query bypasses source admission. Its context
    /// is private; callers cannot execute arbitrary plans through that exception.
    /// # Errors
    /// Engine failure, malformed settings output, or disagreement with configured semantics.
    pub async fn read_back_settings(
        &self,
    ) -> Result<BTreeMap<String, Option<String>>, CatalogError> {
        let analyzers = self
            .profile
            .analyzer_rules
            .iter()
            .map(|name| RuleCatalog::analyzer(name))
            .collect::<Result<Vec<_>, _>>()?;
        let state = SessionStateBuilder::new_from_existing(self.context.state())
            .with_analyzer_rules(analyzers)
            .build();
        let context = SessionContext::new_with_state(state);
        let batches = context
            .sql("SELECT name, value FROM information_schema.df_settings")
            .await
            .map_err(engine)?
            .collect()
            .await
            .map_err(engine)?;
        let mut values = BTreeMap::new();
        for batch in batches {
            let names = batch
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| invalid("settings name storage"))?;
            let entries = batch
                .column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| invalid("settings value storage"))?;
            for row in 0..batch.num_rows() {
                values.insert(
                    names.value(row).to_owned(),
                    (!entries.is_null(row)).then(|| entries.value(row).to_owned()),
                );
            }
        }
        let selected = config::semantic_settings(&values)?;
        if selected != self.settings {
            return Err(invalid(
                "information_schema settings differ from the sealed semantic profile",
            ));
        }
        Ok(values)
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::ConfigInvalid {
        key: "session.admission".to_owned(),
        reason: reason.to_owned(),
    }
}
pub(crate) fn engine(error: DataFusionError) -> CatalogError {
    engine_origin(error, PlanOrigin::Analytics)
}
fn engine_origin(error: DataFusionError, origin: PlanOrigin) -> CatalogError {
    crate::failure::collapse_classified(crate::classify(error, origin))
}

fn add_bound_pool_key(error: &mut CatalogError) {
    match error {
        CatalogError::ResourceLimit { config_keys, .. } => {
            let key = "datafusion.runtime.memory_limit";
            if !config_keys.iter().any(|actual| actual == key) {
                config_keys.push(key.to_owned());
            }
        }
        CatalogError::Multiple { errors } => {
            for error in errors {
                add_bound_pool_key(error);
            }
        }
        _ => {}
    }
}

impl std::fmt::Debug for SnapshotSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotSession")
            .field("sources", &self.sources.keys())
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}
