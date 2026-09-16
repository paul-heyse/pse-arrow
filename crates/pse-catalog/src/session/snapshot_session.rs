// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Sealed session facade and bounded result export.

use super::{
    ExecutionSettings, ThreadBudget,
    candidate::CandidateTable,
    config,
    profile::{EngineProfile, EngineRules},
};
use crate::provider::{
    binding::{BindingKey, Bindings, TableBinding},
    table::RelationTable,
};
use crate::{CatalogError, PlanOrigin, Snapshot};
use datafusion::arrow::{
    ARROW_VERSION,
    array::{Array, RecordBatch, StringArray},
};
use datafusion::catalog::TableProvider;
use datafusion::common::{DataFusionError, TableReference};
use datafusion::datasource::provider_as_source;
use datafusion::execution::{
    context::SessionContext, runtime_env::RuntimeEnv, session_state::SessionStateBuilder,
};
use datafusion::logical_expr::{LogicalPlan, TableSource};
use pse_ids::{CancellationToken, ContentHash, FramedHasher, MemoryReserver, derive::context};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;
use std::sync::Arc;

/// An immutable operation environment retaining exact provider, function and policy owners.
#[derive(Clone)]
pub struct SnapshotSession {
    pub(super) trace: Arc<super::trace::ExecutionTrace>,
    pub(super) context: SessionContext,
    pub(super) registry: Arc<Registry>,
    pub(crate) bindings: Bindings,
    pub(super) policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
    pub(super) purpose: pse_schema::model::provider::OperationPurpose,
    pub(super) requirement_planner: Option<Arc<dyn super::policy::RequirementPlanner>>,
    pub(crate) reserver: Arc<dyn MemoryReserver>,
    pub(super) function_bindings: Arc<super::functions::Functions>,
    pub(super) profile: EngineProfile,
    rules: Arc<EngineRules>,
    settings: BTreeMap<String, Option<String>>,
    functions: ContentHash,
    function_names: BTreeMap<String, Vec<String>>,
    configuration_owner: Option<Arc<pse_ids::ReservationLease>>,
}
/// Build a session over exact admitted snapshots under the sealed `model` catalog.
/// # Errors
/// Missing/ambiguous members, invalid declarations, or unsupported explicit engine settings.
pub fn build_session(
    snapshots: Vec<Arc<Snapshot>>,
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
) -> Result<SnapshotSession, CatalogError> {
    super::SessionFactory::new(runtime, reserver, settings, budget, profile)?.open_session(
        snapshots,
        registry,
        &CancellationToken::default(),
    )
}

pub(super) fn bind_snapshots(
    snapshots: Vec<Arc<Snapshot>>,
    registry: Arc<Registry>,
    factory: &super::SessionFactory,
    cancel: &CancellationToken,
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
            )?);
            if tables
                .insert(
                    key,
                    TableBinding::new(
                        TableReference::full("model", key.namespace.as_str(), key.name),
                        table,
                        Some(key),
                        Some(relation.checked().clone()),
                    ),
                )
                .is_some()
            {
                return Err(invalid(
                    "two snapshot bindings supply the same relation; bind one explicit version",
                ));
            }
        }
    }
    build(tables, registry, factory, cancel)
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
    reason = "resource construction boundary accepts explicit caller cancellation"
)]
/// Construct a candidate session retaining the caller's cooperative cancellation token.
/// # Errors
/// Invalid schema/values, explicit engine configuration or bounded allocation failure.
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
    super::SessionFactory::new(runtime, reserver, settings, budget, profile)?
        .candidate(rows, registry, cancel)
}

pub(super) fn bind_candidates(
    rows: BTreeMap<RelationKey, RecordBatch>,
    registry: Arc<Registry>,
    factory: &super::SessionFactory,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CatalogError> {
    let mut tables = BTreeMap::new();
    for (key, batch) in rows {
        cancel.checkpoint()?;
        let spec = registry
            .relation_by_key(key)
            .ok_or_else(|| invalid("candidate relation/version is undeclared"))?;
        let input = pse_relations::columnar::FieldCheckedBatch::admit_external(
            &registry,
            spec,
            &batch,
            factory.reserver.as_ref(),
            cancel,
        )?;
        let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
            input: input.clone(),
        });
        tables.insert(
            key,
            TableBinding::new(
                TableReference::full("model", key.namespace.as_str(), key.name),
                table,
                Some(key),
                Some(input),
            ),
        );
    }
    build(tables, registry, factory, cancel)
}
fn build(
    tables: BTreeMap<RelationKey, TableBinding>,
    registry: Arc<Registry>,
    factory: &super::SessionFactory,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CatalogError> {
    cancel.checkpoint()?;
    let state_builder = SessionStateBuilder::new_from_existing(factory.state.clone());
    let function_bindings = Arc::clone(&factory.function_bindings);
    let rules = Arc::clone(&factory.rules);
    let profile = factory.profile.clone();
    let reserver = Arc::clone(&factory.reserver);
    let settings = config::semantic_settings(
        &factory
            .state
            .config_options()
            .entries()
            .into_iter()
            .chain(factory.state.runtime_env().config_entries())
            .map(|entry| (entry.key, entry.value))
            .collect(),
    )?;
    let mut bindings = Bindings::default();
    let defaults = &factory.state.config_options().catalog;
    bindings.namespace(&defaults.default_catalog, None);
    bindings.namespace(&defaults.default_catalog, Some(&defaults.default_schema));
    for (key, binding) in tables {
        bindings
            .insert(BindingKey::Relation(key), binding)
            .map_err(engine)?;
    }
    let state = state_builder
        .with_extension_type_registry(super::registry::build(&registry).map_err(engine)?)
        .build();
    let (functions, function_names) = function_inventory(&state);
    Ok(SnapshotSession {
        configuration_owner: None,
        trace: Arc::new(super::trace::ExecutionTrace::new(reserver.as_ref())),
        context: SessionContext::new_with_state(state),
        registry,
        bindings,
        policies: Arc::clone(&factory.policies),
        purpose: pse_schema::model::provider::OperationPurpose::Query,
        requirement_planner: factory.requirement_planner.clone(),
        function_bindings,
        reserver,
        profile,
        rules,
        settings,
        functions,
        function_names,
    })
}
fn function_inventory(
    state: &datafusion::execution::session_state::SessionState,
) -> (ContentHash, BTreeMap<String, Vec<String>>) {
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
    (hash.finish_hash(), function_names)
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
    /// Complete registered function names by family; the actual implementation owners
    /// remain necessary when binding a session.
    pub functions: BTreeMap<String, Vec<String>>,
    /// Actual scoped declarations captured before optimization, including defaults.
    pub policies: Vec<pse_schema::model::provider::ProviderPolicy>,
    /// Declared effect purpose, compared as part of producer correspondence.
    pub purpose: pse_schema::model::provider::OperationPurpose,
}
impl SessionSemantics {
    pub(crate) fn heap_extent(&self) -> Result<usize, CatalogError> {
        crate::store::invocation::engine_inputs_extent(
            self.engine_version.capacity(),
            self.arrow_version.capacity(),
            &self.profile,
            &self.settings,
            &self.functions,
        )?
        .checked_add(super::policy::policies_extent(&self.policies)?)
        .ok_or_else(|| invalid("policy allocation extent overflows"))
    }
}
impl SnapshotSession {
    /// Restore captured settings against this session's actual implementation assembly.
    /// Existing functions and native rules remain bound by their actual owners.
    ///
    /// # Errors
    /// Missing/incompatible implementations, unsupported settings, changed runtime
    /// configuration or allocation refusal. Explicit absent settings are restored too.
    pub fn restore_engine(mut self, inputs: &SessionSemantics) -> Result<Self, CatalogError> {
        use datafusion::common::config::ConfigField;
        if self
            .policies
            .iter()
            .any(|required| !inputs.policies.contains(required))
        {
            return Err(invalid(
                "stored operation does not satisfy the current factory policy declarations",
            ));
        }
        if inputs.engine_version != datafusion::DATAFUSION_VERSION
            || inputs.arrow_version != ARROW_VERSION
            || inputs.profile != self.profile
            || inputs.functions != self.function_names
        {
            return Err(invalid(
                "stored engine requires a different implementation assembly",
            ));
        }
        if inputs.settings.keys().ne(self.settings.keys()) {
            return Err(invalid(
                "stored engine has a different setting declaration inventory",
            ));
        }
        let mut reservation = self.reserver.open("session:restored-configuration");
        reservation.try_grow(
            self.semantic_inputs_extent()?
                .checked_add(inputs.heap_extent()?)
                .and_then(|bytes| bytes.checked_mul(4))
                .ok_or_else(|| invalid("restored configuration allocation overflows"))?,
        )?;
        let state = self.context.state();
        let mut config = state.config().clone();
        for (key, value) in &inputs.settings {
            if self.settings.get(key) == Some(value) {
                continue;
            }
            match value {
                Some(value) => config.options_mut().set(key, value),
                None => config.options_mut().reset(key),
            }
            .map_err(|error| CatalogError::ConfigInvalid {
                key: key.clone(),
                reason: format!("cannot restore captured setting: {error}"),
            })?;
        }
        let restored: BTreeMap<_, _> = config
            .options()
            .entries()
            .into_iter()
            .chain(state.runtime_env().config_entries())
            .map(|entry| (entry.key, entry.value))
            .collect();
        if restored != inputs.settings {
            return Err(invalid(
                "restored settings differ from the captured engine context",
            ));
        }
        self.context = SessionContext::new_with_state(
            SessionStateBuilder::new_from_existing(state)
                .with_config(config)
                .build(),
        );
        self.settings = restored;
        self.policies = Arc::new(inputs.policies.clone());
        self.purpose = inputs.purpose;
        self.effective_policy()?;
        self.configuration_owner = Some(pse_ids::ReservationLease::new(reservation));
        Ok(self)
    }
    /// Borrow the actual immutable provider retained for a declared source.
    pub fn table_provider(&self, key: &RelationKey) -> Option<&Arc<dyn TableProvider>> {
        self.bindings
            .relation(*key)
            .map(|binding| &binding.provider)
    }
    /// Retain the exact checked Arrow source already bound to this relation name.
    /// This inspects private provider ownership, without executing a query or rescanning
    /// any values. Computed intermediates use their completed-computation interface.
    /// # Errors
    /// Missing source or a provider that does not own a declared checked relation.
    pub fn checked_input(
        &self,
        key: &RelationKey,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, CatalogError> {
        self.bindings
            .relation(*key)
            .and_then(|binding| binding.checked.clone())
            .ok_or_else(|| invalid("binding has no checked declared source owner"))
    }

    /// Resolve the actual scalar implementation retained by this immutable engine.
    /// # Errors
    /// The name has no registered scalar function in this session.
    pub fn scalar_function(
        &self,
        name: &str,
    ) -> Result<Arc<datafusion::logical_expr::ScalarUDF>, CatalogError> {
        self.context
            .state()
            .scalar_functions()
            .get(name)
            .cloned()
            .ok_or_else(|| invalid(&format!("scalar function {name} is not registered")))
    }
    /// Fork a new immutable function environment before preparing dependent plans.
    /// Existing names cannot be replaced; the original session remains frozen.
    /// # Errors
    /// A duplicate name would change an already-bound implementation.
    pub fn with_scalar_functions(
        &self,
        functions: Vec<Arc<datafusion::logical_expr::ScalarUDF>>,
    ) -> Result<Self, CatalogError> {
        let state = self.context.state();
        let mut bound = state.scalar_functions().clone();
        for function in functions {
            if let Some(existing) = bound.get(function.name()) {
                if Arc::ptr_eq(existing.inner(), function.inner()) {
                    continue;
                }
                return Err(invalid(
                    "function name is already bound to a different implementation",
                ));
            }
            bound.insert(function.name().to_owned(), function);
        }
        let state = SessionStateBuilder::new_from_existing(state)
            .with_scalar_functions(bound.into_values().collect())
            .build();
        let mut result = self.clone();
        result.function_bindings = super::functions::Functions::from_state(&state);
        (result.functions, result.function_names) = function_inventory(&state);
        result.context = SessionContext::new_with_state(state);
        Ok(result)
    }

    pub(super) fn execution_error(
        &self,
        error: DataFusionError,
        origin: PlanOrigin,
    ) -> CatalogError {
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
        if rows.keys().copied().ne(self.input_keys()) {
            return Err(invalid(
                "caller relation inventory differs from the bound inventory",
            ));
        }
        for (key, batch) in rows {
            self.validate_input_values(key, batch)?;
        }
        Ok(())
    }
    /// Compare values with the actual retained checked owner, without type dispatch.
    /// # Errors
    /// The owner is absent, unchecked or different from the supplied values.
    pub fn validate_input_values(
        &self,
        key: &RelationKey,
        batch: &RecordBatch,
    ) -> Result<(), CatalogError> {
        let owner = self.checked_input(key)?;
        if owner.batch() != batch {
            return Err(invalid("located input differs from its actual bound owner"));
        }
        Ok(())
    }

    /// Verify the actual checked source was bound as a private workspace input.
    /// # Errors
    /// Missing workspace binding, unchecked owner or different values.
    pub fn validate_workspace_input_values(
        &self,
        key: &RelationKey,
        batch: &RecordBatch,
    ) -> Result<(), CatalogError> {
        let binding = self
            .bindings
            .relation(*key)
            .ok_or_else(|| invalid("workspace input absent"))?;
        if binding.reference.catalog() != Some("workspace") || binding.checked.is_none() {
            return Err(invalid("input is not a checked private workspace binding"));
        }
        self.validate_input_values(key, batch)
    }

    /// Create an isolated rule workspace over this session's sealed engine state.
    /// The added providers share the runtime and allocator. Existing providers cannot
    /// be replaced; callers construct each round from the original pinned session.
    ///
    /// # Errors
    /// An undeclared relation, attempted shadowing, invalid values or resource failure.
    pub fn with_workspace(
        &self,
        rows: BTreeMap<RelationKey, RecordBatch>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for (key, batch) in rows {
            cancel.checkpoint()?;
            let spec = self
                .registry
                .relation_by_key(key)
                .ok_or_else(|| invalid("workspace relation/version is undeclared"))?;
            if spec.authority != pse_schema::model::Authority::Derived {
                return Err(invalid("workspace relation must be derived"));
            }
            let input = pse_relations::columnar::FieldCheckedBatch::admit_external(
                &self.registry,
                spec,
                &batch,
                self.reserver.as_ref(),
                cancel,
            )?;
            let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
                input: input.clone(),
            });
            result
                .bindings
                .insert(
                    BindingKey::Relation(key),
                    TableBinding::new(
                        TableReference::full("workspace", key.namespace.as_str(), key.name),
                        table,
                        Some(key),
                        Some(input),
                    ),
                )
                .map_err(engine)?;
        }
        Ok(result)
    }

    /// Fork an explicit subset of existing input providers while retaining the
    /// same runtime, functions, configuration and immutable provider objects.
    /// A caller can then add declared recomputed outputs as private worktables.
    /// # Errors
    /// A selected relation was absent or cancellation was requested.
    pub fn select_inputs(
        &self,
        selected: &std::collections::BTreeSet<RelationKey>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        for key in selected {
            cancel.checkpoint()?;
            if self.bindings.relation(*key).is_none() {
                return Err(invalid("selected input is outside the bound inventory"));
            }
        }
        let mut result = self.clone();
        result.bindings.retain(|key| match key {
            BindingKey::Relation(key) => selected.contains(key),
            _ => true,
        });
        Ok(result)
    }

    /// Exact declared source keys retained by this generation.
    pub fn input_keys(&self) -> impl Iterator<Item = RelationKey> + '_ {
        self.bindings.iter().filter_map(|(key, _)| match key {
            BindingKey::Relation(key) => Some(*key),
            _ => None,
        })
    }

    /// Check that a derived workspace still uses the exact retained engine and allocator.
    /// # Errors
    /// A different runtime, allocator, registry, function inventory or semantic setting.
    pub fn validate_execution_environment(&self, other: &Self) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&self.context.runtime_env(), &other.context.runtime_env())
            || !Arc::ptr_eq(&self.reserver, &other.reserver)
            || !Arc::ptr_eq(&self.registry, &other.registry)
            || !Arc::ptr_eq(&self.function_bindings, &other.function_bindings)
            || !Arc::ptr_eq(&self.rules, &other.rules)
            || match (&self.requirement_planner, &other.requirement_planner) {
                (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
                (None, None) => false,
                _ => true,
            }
            || self.policies != other.policies
            || self.purpose != other.purpose
            || self.profile != other.profile
            || self.settings != other.settings
        {
            return Err(invalid(
                "constructed workspace changed the retained execution environment",
            ));
        }
        Ok(())
    }

    /// Materialize native lookup only when preparing an operation. Adding a role
    /// changes the immutable binding index without rebuilding engine configuration.
    pub(super) fn bound_state(
        &self,
    ) -> Result<datafusion::execution::session_state::SessionState, CatalogError> {
        let state = self.context.state();
        let mut config = state.config().clone();
        for (key, value) in self.effective_policy()?.settings {
            config.options_mut().set(&key, &value).map_err(|error| {
                CatalogError::ConfigInvalid {
                    key,
                    reason: error.to_string(),
                }
            })?;
        }
        let defaults = &config.options().catalog;
        let catalogs = self
            .bindings
            .catalogs(&defaults.default_catalog, &defaults.default_schema);
        Ok(SessionStateBuilder::new_from_existing(state)
            .with_config(config)
            .with_catalog_list(Arc::new(catalogs))
            .build())
    }

    /// Shared accounted allocator for typed rule outputs.
    pub(super) fn capture_configuration(
        &mut self,
        state: &datafusion::execution::session_state::SessionState,
    ) {
        self.context = SessionContext::new_with_state(state.clone());
        self.settings = state
            .config_options()
            .entries()
            .into_iter()
            .chain(state.runtime_env().config_entries())
            .map(|entry| (entry.key, entry.value))
            .collect();
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
        crate::store::invocation::engine_inputs_extent(
            datafusion::DATAFUSION_VERSION.len(),
            ARROW_VERSION.len(),
            &self.profile,
            &self.settings,
            &self.function_names,
        )?
        .checked_add(super::policy::policies_extent(&self.policies)?)
        .ok_or_else(|| invalid("policy allocation extent overflows"))
    }
    /// Actual inputs behind the diagnostic profile identities; no hash admits reuse.
    pub fn semantic_inputs(&self) -> SessionSemantics {
        SessionSemantics {
            engine_version: datafusion::DATAFUSION_VERSION.to_owned(),
            arrow_version: ARROW_VERSION.to_owned(),
            profile: self.profile.clone(),
            settings: self.settings.clone(),
            functions: self.function_names.clone(),
            policies: self.policies.as_ref().clone(),
            purpose: self.purpose,
        }
    }

    /// Compare complete engine semantic values without allocating another snapshot.
    pub fn matches_semantic_inputs(&self, inputs: &SessionSemantics) -> bool {
        inputs.engine_version == datafusion::DATAFUSION_VERSION
            && inputs.arrow_version == ARROW_VERSION
            && inputs.profile == self.profile
            && inputs.settings == self.settings
            && inputs.functions == self.function_names
            && inputs.policies == *self.policies
            && inputs.purpose == self.purpose
    }

    /// Exact registry used at all semantic boundaries.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }
    /// Resolve the actual source object under a declared relation key.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_source(&self, key: &RelationKey) -> Result<Arc<dyn TableSource>, CatalogError> {
        self.bindings
            .relation(*key)
            .map(|binding| provider_as_source(Arc::clone(&binding.provider)))
            .ok_or_else(|| invalid("undeclared session read"))
    }
    /// The SQL/plan reference bound to this exact relation version.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_reference(&self, key: &RelationKey) -> Result<TableReference, CatalogError> {
        self.bindings
            .relation(*key)
            .map(|binding| binding.reference.clone())
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
    /// Plan and execute native SQL through the same purpose-aware preparation as rule plans.
    /// # Errors
    /// Syntax, unsupported writes, semantic admission, cancellation or resource exhaustion.
    pub async fn sql(
        &self,
        sql: &str,
        cancel: &CancellationToken,
    ) -> Result<Vec<RecordBatch>, CatalogError> {
        cancel.checkpoint()?;
        Ok(self
            .prepare_sql(sql, cancel)
            .await?
            .execute(cancel)
            .await?
            .into_observed_batches()
            .0)
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
        let completed = self
            .prepare_with_origin(plan, cancel, origin)?
            .execute(cancel)
            .await?;
        Ok(completed.into_observed_batches())
    }
    /// Read the actual native settings view through ordinary provider admission.
    /// # Errors
    /// Engine failure, malformed settings output, or disagreement with configured semantics.
    pub async fn read_back_settings(
        &self,
    ) -> Result<BTreeMap<String, Option<String>>, CatalogError> {
        let batches = self
            .sql(
                "SELECT name, value FROM information_schema.df_settings",
                &CancellationToken::default(),
            )
            .await?;
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
        let state = self.execution_scope()?.bound_state()?;
        let expected: BTreeMap<_, _> = state
            .config_options()
            .entries()
            .into_iter()
            .chain(state.runtime_env().config_entries())
            .map(|entry| (entry.key, entry.value))
            .collect();
        if selected != expected {
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
            .field("bindings", &self.bindings)
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}
