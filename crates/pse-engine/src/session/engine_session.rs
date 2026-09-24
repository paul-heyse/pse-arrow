// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Sealed session facade and bounded result export.

use super::{
    config,
    materialized::ImmutableTable,
    profile::{EngineProfile, EngineRules},
};
use crate::EngineError;
use crate::provider::binding::{BindingKey, Bindings, TableBinding};
use datafusion::arrow::{
    ARROW_VERSION,
    array::{Array, RecordBatch, StringArray},
};
use datafusion::catalog::TableProvider;
use datafusion::common::{DataFusionError, TableReference};
use datafusion::datasource::provider_as_source;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::{LogicalPlan, TableSource};
use pse_columnar::PlanOrigin;
use pse_columnar::{CancellationToken, MemoryPool};
use pse_ids::{ContentHash, FramedHasher, derive::context};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;
use std::sync::Arc;

/// An immutable operation environment retaining exact provider, function and policy owners.
#[derive(Clone)]
pub struct EngineSession {
    pub(super) invocation: Option<Arc<super::cache::CacheStore>>,
    pub(super) query_admission: std::sync::Weak<tokio::sync::OwnedSemaphorePermit>,
    pub(crate) owners: Vec<Arc<dyn crate::provider::witness::ExecutionOwner>>,
    pub(super) requires_fresh: Arc<std::sync::atomic::AtomicBool>,
    pub(super) trace: Arc<super::trace::ExecutionTrace>,
    pub(super) native: Arc<super::assembly::ModelAssembly>,
    pub(super) selection: super::assembly::SelectionCache,
    pub(super) registry: Arc<Registry>,
    pub(crate) bindings: Bindings,
    pub(super) policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
    pub(super) purpose: pse_schema::model::provider::OperationPurpose,
    pub(super) requirement_planner: Option<Arc<dyn super::policy::RequirementPlanner>>,
    pub(crate) pool: Arc<dyn MemoryPool>,
    pub(super) execution_runtime: Option<Arc<datafusion::execution::runtime_env::RuntimeEnv>>,
    pub(super) profile: EngineProfile,
    pub(super) rules: Arc<EngineRules>,
    configuration_owner: Option<Arc<pse_columnar::AllocationLease>>,
}
pub(super) fn bind_candidates(
    rows: BTreeMap<RelationKey, RecordBatch>,
    registry: Arc<Registry>,
    factory: &super::EngineFactory,
    cancel: &CancellationToken,
) -> Result<EngineSession, EngineError> {
    let native = factory.model_assembly(&registry)?;
    let validation = pse_relations::validate::ValidationContext::new(
        &registry,
        crate::validation::NativeValidation(native.context.state()),
    );
    let mut tables = BTreeMap::new();
    for (key, batch) in rows {
        cancel.checkpoint()?;
        let spec = registry
            .relation_by_key(key)
            .ok_or_else(|| invalid("candidate relation/version is undeclared"))?;
        let input = pse_relations::columnar::FieldCheckedBatch::admit_external_in(
            &registry,
            spec,
            &batch,
            &validation,
            &factory.pool,
            cancel,
        )?;
        let table: Arc<dyn TableProvider> = Arc::new(ImmutableTable::candidate(input.clone()));
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
    factory: &super::EngineFactory,
    cancel: &CancellationToken,
) -> Result<EngineSession, EngineError> {
    cancel.checkpoint()?;
    let native = factory.model_assembly(&registry)?;
    let rules = Arc::clone(&factory.rules);
    let profile = factory.profile.clone();
    let pool = Arc::clone(&factory.pool);
    let mut bindings = Bindings::default();
    let defaults = &factory.state.config_options().catalog;
    bindings
        .namespace(&defaults.default_catalog, None)
        .map_err(engine)?;
    bindings
        .namespace(&defaults.default_catalog, Some(&defaults.default_schema))
        .map_err(engine)?;
    for (key, binding) in tables {
        bindings
            .insert(BindingKey::Relation(key), binding)
            .map_err(engine)?;
    }
    Ok(EngineSession {
        invocation: None,
        query_admission: std::sync::Weak::new(),
        native,
        selection: super::assembly::SelectionCache::default(),
        owners: Vec::new(),
        configuration_owner: None,
        requires_fresh: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        trace: Arc::new(super::trace::ExecutionTrace::new(&pool)),
        registry,
        bindings,
        policies: Arc::clone(&factory.policies),
        purpose: pse_schema::model::provider::OperationPurpose::Query,
        requirement_planner: factory.requirement_planner.clone(),
        pool,
        execution_runtime: None,
        profile,
        rules,
    })
}
pub(super) fn function_inventory(
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
    pub(crate) fn heap_extent(&self) -> Result<usize, EngineError> {
        super::semantic_extent::engine_inputs_extent(
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
impl EngineSession {
    /// Share model capabilities while dropping the parent's selected providers.
    pub(super) fn empty_selection(&self) -> Result<Self, EngineError> {
        let mut result = self.clone();
        result.bindings = Bindings::default();
        result.selection = super::assembly::SelectionCache::default();
        let config = self.native.context.copied_config();
        let defaults = &config.options().catalog;
        result
            .bindings
            .namespace(&defaults.default_catalog, Some(&defaults.default_schema))
            .map_err(engine)?;
        Ok(result)
    }

    /// Restore captured settings against this session's actual implementation assembly.
    /// Existing functions and native rules remain bound by their actual owners.
    ///
    /// # Errors
    /// Missing/incompatible implementations, unsupported settings, changed runtime
    /// configuration or allocation refusal. Explicit absent settings are restored too.
    pub fn restore_engine(mut self, inputs: &SessionSemantics) -> Result<Self, EngineError> {
        use datafusion::common::config::ConfigField;
        if self.policies.iter().any(|required| {
            !inputs
                .policies
                .iter()
                .any(|policy| config::policy_semantics_equal(required, policy))
        }) {
            return Err(invalid(
                "stored operation does not satisfy the current factory policy declarations",
            ));
        }
        if inputs.engine_version != datafusion::DATAFUSION_VERSION
            || inputs.arrow_version != ARROW_VERSION
            || inputs.profile != self.profile
            || inputs.functions != self.native.function_names
        {
            return Err(invalid(
                "stored engine requires a different implementation assembly",
            ));
        }
        if inputs.settings.keys().ne(self.native.settings.keys()) {
            return Err(invalid(
                "stored engine has a different setting declaration inventory",
            ));
        }
        let reservation = pse_columnar::MemoryConsumer::new("session:restored-configuration")
            .register(&self.pool);
        reservation.try_grow(
            self.semantic_inputs_extent()?
                .checked_add(inputs.heap_extent()?)
                .and_then(|bytes| bytes.checked_mul(4))
                .ok_or_else(|| invalid("restored configuration allocation overflows"))?,
        )?;
        let mut state = self.native.context.state();
        let mut config = state.config().clone();
        for (key, value) in &inputs.settings {
            if self.native.settings.get(key) == Some(value) {
                continue;
            }
            match value {
                Some(value) => config.options_mut().set(key, value),
                None => config.options_mut().reset(key),
            }
            .map_err(|error| EngineError::ConfigInvalid {
                key: key.clone(),
                reason: format!("cannot restore captured setting: {error}"),
            })?;
        }
        let restored = config::semantic_settings(
            &config
                .options()
                .entries()
                .into_iter()
                .map(|entry| (entry.key, entry.value))
                .collect(),
        )?;
        if restored != inputs.settings {
            return Err(invalid(
                "restored settings differ from the captured engine context",
            ));
        }
        *state.config_mut() = config;
        let native = Arc::make_mut(&mut self.native);
        native.context = SessionContext::new_with_state(state);
        native.settings = restored;
        self.policies = Arc::new(
            inputs
                .policies
                .iter()
                .map(|policy| {
                    self.policies
                        .iter()
                        .find(|current| config::policy_semantics_equal(current, policy))
                        .cloned()
                        .unwrap_or_else(|| config::semantic_policy(policy))
                })
                .collect(),
        );
        self.purpose = inputs.purpose;
        self.effective_policy()?;
        self.configuration_owner = Some(pse_columnar::AllocationLease::new(reservation));
        Ok(self)
    }
    /// Borrow the actual immutable provider retained for a declared source.
    pub fn table_provider(&self, key: &RelationKey) -> Option<Arc<dyn TableProvider>> {
        self.bindings
            .relation(*key)
            .map(|binding| Arc::clone(&binding.provider))
    }
    /// Retain the exact checked Arrow source already bound to this relation name.
    /// This inspects private provider ownership, without executing a query or rescanning
    /// any values. Computed intermediates use their completed-computation interface.
    /// # Errors
    /// Missing source or a provider that does not own a declared checked relation.
    pub fn checked_input(
        &self,
        key: &RelationKey,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, EngineError> {
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
    ) -> Result<Arc<datafusion::logical_expr::ScalarUDF>, EngineError> {
        self.native
            .context
            .state_ref()
            .read()
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
    ) -> Result<Self, EngineError> {
        use datafusion::logical_expr::registry::FunctionRegistry;
        let mut state = self.native.context.state();
        let mut bound = state.scalar_functions().clone();
        let mut changed = false;
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
            changed = true;
        }
        if !changed {
            return Ok(self.clone());
        }
        for function in bound.into_values() {
            state.register_udf(function).map_err(engine)?;
        }
        let mut result = self.clone();
        let native = Arc::make_mut(&mut result.native);
        native.generation = super::factory::new_generation();
        native.function_bindings = super::functions::Functions::from_state(&state);
        (native.functions, native.function_names) = function_inventory(&state);
        native.context = SessionContext::new_with_state(state);
        Ok(result)
    }

    pub(super) fn execution_error(
        &self,
        error: DataFusionError,
        origin: PlanOrigin,
    ) -> EngineError {
        let mut error = pse_columnar::classify(error, origin);
        if matches!(
            self.pool.memory_limit(),
            datafusion::execution::memory_pool::MemoryLimit::Finite(_)
        ) {
            error = error.with_resource_key("datafusion.runtime.memory_limit");
        }
        error.into()
    }
    /// Compare the complete caller binding with retained actual providers, including values.
    /// # Errors
    /// Missing/extra relation bindings or any actual schema/value disagreement.
    pub fn validate_bindings(
        &self,
        rows: &BTreeMap<RelationKey, RecordBatch>,
    ) -> Result<(), EngineError> {
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
    ) -> Result<(), EngineError> {
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
    ) -> Result<(), EngineError> {
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
    ) -> Result<Self, EngineError> {
        let validation = pse_relations::validate::ValidationContext::new(
            &self.registry,
            crate::validation::NativeValidation(self.native.context.state()),
        );
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
            let input = pse_relations::columnar::FieldCheckedBatch::admit_external_in(
                &self.registry,
                spec,
                &batch,
                &validation,
                &self.pool,
                cancel,
            )?;
            let table: Arc<dyn TableProvider> = Arc::new(ImmutableTable::candidate(input.clone()));
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
    ) -> Result<Self, EngineError> {
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
    pub fn validate_execution_environment(&self, other: &Self) -> Result<(), EngineError> {
        if !Arc::ptr_eq(
            &self.native.context.runtime_env(),
            &other.native.context.runtime_env(),
        ) || !Arc::ptr_eq(&self.pool, &other.pool)
            || !Arc::ptr_eq(&self.registry, &other.registry)
            || !Arc::ptr_eq(
                &self.native.function_bindings,
                &other.native.function_bindings,
            )
            || !Arc::ptr_eq(&self.rules, &other.rules)
            || match (&self.requirement_planner, &other.requirement_planner) {
                (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
                (None, None) => false,
                _ => true,
            }
            || self.policies != other.policies
            || self.purpose != other.purpose
            || self.profile != other.profile
            || self.native.settings != other.native.settings
        {
            return Err(invalid(
                "constructed workspace changed the retained execution environment",
            ));
        }
        Ok(())
    }

    /// Materialize native lookup only when preparing an operation. Adding a role
    /// changes the immutable binding index without rebuilding engine configuration.
    /// # Errors
    /// Policy settings, namespace admission or native inspection binding failure.
    pub fn bound_state(
        &self,
    ) -> Result<datafusion::execution::session_state::SessionState, EngineError> {
        let selected = self.selection()?;
        let mut state = self.native.context.state();
        *state.config_mut() = selected.config.clone();
        // Native namespace commands get a private mutable tree. Read selections
        // share the retained immutable lookup view and exact table owners.
        let catalogs: Arc<dyn datafusion::catalog::CatalogProviderList> = if selected
            .effective
            .effects
            .contains(&pse_schema::model::provider::OperationEffect::Namespace)
        {
            Arc::new(selected.catalogs.as_ref().clone())
        } else {
            selected.catalogs.clone()
        };
        state.register_catalog_list(catalogs);
        Ok(state)
    }

    /// Shared accounted allocator for typed rule outputs.
    pub(super) fn capture_configuration(
        &mut self,
        state: &datafusion::execution::session_state::SessionState,
    ) -> Result<(), EngineError> {
        let settings = config::semantic_settings(&config::inventory(state))?;
        // Resource-bound execution states have no SQL definition inventory at
        // this pin. Apply their native command changes to the retained state;
        // PREPARE/DEALLOCATE use the model runtime and carry their own inventory.
        let mut retained = self.native.context.state();
        // Opaque caller extensions belong to the model. Attempt extensions and
        // the selected catalog are never captured back into that owner.
        let mut config = retained.config().clone();
        *config.options_mut() = state.config_options().as_ref().clone();
        let catalogs = retained.catalog_list().clone();
        let functions_changed =
            super::functions::apply_registry_changes(&mut retained, state).map_err(engine)?;
        let mut captured = if Arc::ptr_eq(retained.runtime_env(), state.runtime_env()) {
            state.clone()
        } else {
            retained
        };
        *captured.config_mut() = config;
        captured.register_catalog_list(catalogs);
        let native = Arc::make_mut(&mut self.native);
        if functions_changed {
            native.function_bindings = super::functions::Functions::from_state(&captured);
            (native.functions, native.function_names) = function_inventory(&captured);
            native.generation = super::factory::new_generation();
        }
        native.context = SessionContext::new_with_state(captured);
        native.settings = settings;
        Ok(())
    }
    /// Shared accounted allocator for typed rule outputs.
    pub fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.pool
    }
    /// Owned native function/planner/rule assembly, independent of a write attempt.
    /// Unknown cold assemblies receive a fresh generation and cannot claim equality.
    pub fn implementation_generation(&self) -> pse_ids::SemanticId {
        self.native.generation
    }
    /// A checked heap bound for cloning the actual borrowed engine semantic inputs.
    /// Producers reserve this before calling [`Self::semantic_inputs`].
    /// # Errors
    /// Addressable allocation-size overflow.
    pub fn semantic_inputs_extent(&self) -> Result<usize, EngineError> {
        super::semantic_extent::engine_inputs_extent(
            datafusion::DATAFUSION_VERSION.len(),
            ARROW_VERSION.len(),
            &self.profile,
            &self.native.settings,
            &self.native.function_names,
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
            settings: self.native.settings.clone(),
            functions: self.native.function_names.clone(),
            policies: self.policies.iter().map(config::semantic_policy).collect(),
            purpose: self.purpose,
        }
    }

    /// Compare complete engine semantic values without allocating another snapshot.
    pub fn matches_semantic_inputs(&self, inputs: &SessionSemantics) -> bool {
        inputs.engine_version == datafusion::DATAFUSION_VERSION
            && inputs.arrow_version == ARROW_VERSION
            && inputs.profile == self.profile
            && inputs.settings == self.native.settings
            && inputs.functions == self.native.function_names
            && inputs.policies.len() == self.policies.len()
            && inputs
                .policies
                .iter()
                .zip(self.policies.iter())
                .all(|(a, b)| config::policy_semantics_equal(a, b))
            && inputs.purpose == self.purpose
    }

    /// Exact registry used at all semantic boundaries.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }
    /// Resolve the actual source object under a declared relation key.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_source(&self, key: &RelationKey) -> Result<Arc<dyn TableSource>, EngineError> {
        self.bindings
            .relation(*key)
            .map(|binding| provider_as_source(Arc::clone(&binding.provider)))
            .ok_or_else(|| invalid("undeclared session read"))
    }
    /// The SQL/plan reference bound to this exact relation version.
    /// # Errors
    /// The requested relation/version is outside the pinned inventory.
    pub fn table_reference(&self, key: &RelationKey) -> Result<TableReference, EngineError> {
        self.bindings
            .relation(*key)
            .map(|binding| binding.reference.clone())
            .ok_or_else(|| invalid("undeclared session read"))
    }
    /// Semantic setting identity from the immutable configured state.
    pub fn settings_hash(&self) -> ContentHash {
        config::settings_hash(&self.native.settings)
    }
    /// Complete engine profile and semantic-setting identity.
    pub fn profile_hash(&self) -> ContentHash {
        self.profile.hash(self.settings_hash())
    }
    /// Identity of the pinned built-in function registry; no runtime registration is exposed.
    pub fn function_registry_hash(&self) -> ContentHash {
        self.native.functions
    }
    /// Plan and execute native SQL through the same purpose-aware preparation as rule plans.
    /// # Errors
    /// Syntax, unsupported writes, semantic admission, cancellation or resource exhaustion.
    pub async fn sql(
        &self,
        sql: &str,
        cancel: &CancellationToken,
    ) -> Result<Vec<RecordBatch>, EngineError> {
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
    ) -> Result<Vec<RecordBatch>, EngineError> {
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
    ) -> Result<Vec<RecordBatch>, EngineError> {
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
    ) -> Result<(Vec<RecordBatch>, super::PlanObservation), EngineError> {
        self.execute_with_origin(plan, cancel, PlanOrigin::RuleCompiler)
            .await
    }
    async fn execute_with_origin(
        &self,
        plan: LogicalPlan,
        cancel: &CancellationToken,
        origin: PlanOrigin,
    ) -> Result<(Vec<RecordBatch>, super::PlanObservation), EngineError> {
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
    ) -> Result<BTreeMap<String, Option<String>>, EngineError> {
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
        let state = self.execution_scope()?.bound_state()?;
        if values != config::inventory(&state) {
            return Err(invalid(
                "information_schema settings differ from the full native configuration",
            ));
        }
        Ok(values)
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::ConfigInvalid {
        key: "session.admission".to_owned(),
        reason: reason.to_owned(),
    }
}
/// Preserve classified native causes in the common engine error boundary.
pub fn engine(error: DataFusionError) -> EngineError {
    engine_origin(error, PlanOrigin::Analytics)
}
fn engine_origin(error: DataFusionError, origin: PlanOrigin) -> EngineError {
    pse_columnar::classify(error, origin).into()
}

impl std::fmt::Debug for EngineSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineSession")
            .field("bindings", &self.bindings)
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}

impl EngineSession {
    /// Bind an actual provider under its admitted source role.
    /// Read the exact immutable native binding inventory.
    pub fn bindings(&self) -> &Bindings {
        &self.bindings
    }
    /// Include a declared target in effective policy admission.
    pub fn bind_target(&mut self, target: pse_schema::model::provider::ProviderScope) {
        self.bindings.target(target);
    }
    /// Bind an actual source after checking declared fields; rejects role conflicts.
    /// # Errors
    /// Declared fields differ or the requested role conflicts with an existing source.
    pub fn bind_source(
        &mut self,
        key: BindingKey,
        binding: TableBinding,
    ) -> datafusion::common::Result<()> {
        if let Some(relation) = binding.relation {
            let spec = self
                .registry
                .relation_by_key(relation)
                .ok_or_else(|| DataFusionError::Plan("source declaration absent".into()))?;
            let schema = pse_schema::arrow::relation_schema(&self.registry, spec)
                .map_err(pse_columnar::external)?;
            if binding.provider.schema().fields() != schema.fields() {
                return Err(DataFusionError::Plan(
                    "source fields differ from declaration".into(),
                ));
            }
        }
        self.bindings.insert(key, binding)
    }
    /// Retain real catalog/resource owners through every derived execution.
    pub fn retain_owner(&mut self, owner: Arc<dyn crate::provider::witness::ExecutionOwner>) {
        self.owners.push(owner);
    }
    /// Actual retained resource guards for specialized source streams.
    pub fn owners(&self) -> &[Arc<dyn crate::provider::witness::ExecutionOwner>] {
        &self.owners
    }
}
