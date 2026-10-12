// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared engine resources without a dependency on the higher runtime/controller crate.
use super::{EngineProfile, EngineSession, ExecutionSettings, ThreadBudget};
use crate::EngineError;
use datafusion::{
    arrow::array::RecordBatch,
    execution::{
        runtime_env::RuntimeEnv,
        session_state::{SessionState, SessionStateBuilder},
    },
};

use pse_columnar::{CancellationToken, MemoryPool};
use pse_schema::{Registry, model::RelationKey};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex, Weak},
};

/// Reusable session construction settings with one shared deployment memory pool.
#[derive(Debug, Clone)]
pub struct EngineFactory {
    models: Arc<Mutex<HashMap<usize, Weak<super::assembly::ModelAssembly>>>>,
    pub(super) implementation_generation: pse_ids::SemanticId,
    pub(super) state: SessionState,
    pub(super) pool: Arc<dyn MemoryPool>,
    pub(super) profile: EngineProfile,
    pub(super) rules: Arc<super::EngineRules>,
    pub(super) policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
}
impl EngineFactory {
    /// Generation of native capabilities; selected provider policy settings remain separate inputs.
    pub fn implementation_generation(&self) -> pse_ids::SemanticId {
        self.implementation_generation
    }

    /// Bind the registry-dependent extension and validators once for a model owner.
    pub(super) fn model_assembly(
        &self,
        registry: &Arc<Registry>,
    ) -> Result<Arc<super::assembly::ModelAssembly>, EngineError> {
        let key = Arc::as_ptr(registry) as usize;
        let mut models = self
            .models
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(model) = models.get(&key).and_then(Weak::upgrade)
            && Arc::ptr_eq(&model.registry, registry)
        {
            crate::cache_service::metrics::record(&self.state, |m| &m.assembly_reuses);
            return Ok(model);
        }
        models.retain(|_, model| model.strong_count() != 0);
        let mut builder = SessionStateBuilder::new_from_existing(self.state.clone());
        let functions = builder.scalar_functions().get_or_insert_default();
        functions.retain(|function| function.name() != "pse_checked_value");
        functions.push(super::scalar::checked_value::function(
            registry.clone(),
            self.state.clone(),
        ));
        let state = builder
            .with_extension_type_registry(
                super::registry::build(registry).map_err(super::engine_session::engine)?,
            )
            .build();
        let state = crate::cache_service::inspection::bind(state, registry)
            .map_err(super::engine_session::engine)?;
        let state = super::assurance::instrument(state);
        let (functions, function_names) = super::engine_session::function_inventory(&state);
        let model = Arc::new(super::assembly::ModelAssembly {
            planning: Arc::new(()),
            validation: Default::default(),
            reflection: Default::default(),
            registry: registry.clone(),
            function_bindings: super::functions::Functions::from_state(&state),
            settings: super::config::semantic_settings(&super::config::inventory(&state))?,
            functions,
            function_names,
            generation: self.implementation_generation,
            context: datafusion::execution::context::SessionContext::new_with_state(state),
        });
        crate::cache_service::metrics::record(&self.state, |m| &m.assembly_builds);
        models.insert(key, Arc::downgrade(&model));
        Ok(model)
    }
    fn refresh_cache_identity(&mut self) {
        let config = self.state.config().clone().with_extension(Arc::new(
            super::assembly::AssemblyIdentity {
                generation: self.implementation_generation,
                policies: Arc::new(
                    self.policies
                        .iter()
                        .map(super::config::semantic_policy)
                        .collect(),
                ),
                settings: None,
            },
        ));
        *self.state.config_mut() = config;
        self.models = Arc::default();
    }
    /// Carry the deployment cache owner in native state; cloning a scope retains it.
    #[must_use]
    pub fn with_cache_service(
        mut self,
        caches: Arc<crate::cache_service::NativeCacheService>,
    ) -> Self {
        let mut config = self.state.config().clone();
        config
            .options_mut()
            .execution
            .parquet
            .max_predicate_cache_size = Some(caches.policy().predicate_cache_bytes);
        let config = config.with_extension(caches);
        *self.state.config_mut() = config;
        self.models = Arc::default();
        self
    }
    /// Select diagnostics without changing implementation identity or semantic settings.
    #[must_use]
    pub fn with_observation(mut self, policy: super::assurance::ObservationPolicy) -> Self {
        let config = self.state.config().clone().with_extension(Arc::new(policy));
        *self.state.config_mut() = config;
        self.models = Arc::default();
        self
    }

    /// Install an explicitly composed native query planner before opening sessions.
    /// All actual functions, rules, settings and resources are retained.
    #[must_use]
    pub fn with_query_planner(
        mut self,
        planner: Arc<dyn datafusion::execution::context::QueryPlanner + Send + Sync>,
    ) -> Self {
        self.implementation_generation = new_generation();
        self.state = SessionStateBuilder::new_from_existing(self.state)
            .with_query_planner(planner)
            .build();
        self.refresh_cache_identity();
        self
    }
    /// Bind the already constructed shared runtime and explicit execution profile.
    /// # Errors
    /// Invalid thread/configuration settings or an unknown explicit rule implementation.
    pub fn new(
        runtime: Arc<RuntimeEnv>,
        pool: Arc<dyn MemoryPool>,
        settings: ExecutionSettings,
        budget: ThreadBudget,
        profile: EngineProfile,
    ) -> Result<Self, EngineError> {
        let rules = super::EngineRules::from_profile(&profile)?;
        let builder = SessionStateBuilder::new()
            .with_default_features()
            .with_config(super::config::build(settings, budget)?)
            .with_query_planner(Arc::new(super::planner::UnifiedPlanner::default()))
            .with_analyzer_rules(rules.analyzers)
            .with_optimizer_rules(rules.optimizers)
            .with_physical_optimizer_rules(rules.physical);
        let version = profile.version;
        Ok(Self::from_builder(runtime, pool, &version, builder))
    }
    /// Freeze a native engine assembly before binding any input or preparing a plan.
    /// Callers may install native scalar, aggregate, window and higher-order functions,
    /// analyzers, expression rewrites, logical/physical optimizers and planners using
    /// `SessionStateBuilder`. The builder's entire native configuration, including
    /// opaque extensions, remains authoritative alongside the actual implementations.
    /// The known unsafe leaf-expression pushdown in DataFusion 55.1 is disabled
    /// during assembly; all other caller options and optimizer implementations remain.
    #[must_use]
    pub fn from_builder(
        runtime: Arc<RuntimeEnv>,
        pool: Arc<dyn MemoryPool>,
        version: &str,
        mut builder: SessionStateBuilder,
    ) -> Self {
        // Assembly selects one pool for both native operators and application
        // consumers, while retaining the caller's I/O, spill and cache owners.
        let runtime = if Arc::ptr_eq(&runtime.memory_pool, &pool) {
            runtime
        } else {
            Arc::new(RuntimeEnv {
                memory_pool: pool.clone(),
                disk_manager: runtime.disk_manager.clone(),
                cache_manager: runtime.cache_manager.clone(),
                object_store_registry: runtime.object_store_registry.clone(),
            })
        };
        let options = builder.config().get_or_insert_default().options_mut();
        // DataFusion 55.1's leaf-projection recovery compares only field names.
        // It can discard same-named conversions (including provider decoding) while
        // retaining raw storage fields. Keep native optimization enabled, but
        // select its supported opt-out for this unsafe rewrite pair until the
        // upstream recovery preserves expressions and complete field contracts.
        // Preserve source-span field obligations at the native provider boundary.
        options.optimizer.enable_leaf_expression_pushdown = false;
        // A plain native builder retains the generic planning path from `new`.
        // Explicit caller planners remain the actual implementations;
        // custom assemblies compose domain planners through UnifiedPlanner::new.
        builder
            .query_planner()
            .get_or_insert_with(|| Arc::new(super::planner::UnifiedPlanner::default()));
        builder
            .cache_factory()
            .get_or_insert_with(|| Arc::new(super::cache::NativeCacheFactory));
        let state = super::scalar::register(builder, Arc::clone(&pool))
            .with_runtime_env(runtime)
            .build();
        let mut physical = state.physical_optimizers().to_vec();
        // Identify the actual owned rule, never a caller-controlled display name.
        physical.retain(|rule| {
            let implementation: &dyn std::any::Any = rule.as_ref();
            !implementation.is::<super::physical_fields::SemanticFields>()
        });
        for rule in &mut physical {
            let implementation: &dyn std::any::Any = rule.as_ref();
            if implementation
                .is::<datafusion::physical_optimizer::aggregate_statistics::AggregateStatistics>()
            {
                *rule = Arc::new(super::physical_fields::AggregateFields {
                    native: rule.clone(),
                });
            }
        }
        physical.insert(0, Arc::new(super::physical_fields::SemanticFields));
        let state = SessionStateBuilder::new_from_existing(state)
            .with_physical_optimizer_rules(physical)
            .build();
        let rules = Arc::new(super::EngineRules {
            analyzers: state.analyzer().rules.clone(),
            optimizers: state.optimizer().rules.clone(),
            physical: state.physical_optimizers().to_vec(),
        });
        let mut factory = Self {
            models: Arc::default(),
            implementation_generation: new_generation(),
            state,
            pool,
            profile: rules.profile(version),
            rules,
            policies: Arc::default(),
        };
        factory.refresh_cache_identity();
        factory
    }
    /// Construct a constraint-free session over the complete actual candidate rows.
    /// # Errors
    /// Schema/value admission, cancellation, reservation or explicit engine configuration.
    pub fn candidate(
        &self,
        rows: BTreeMap<RelationKey, RecordBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<EngineSession, EngineError> {
        cancel.checkpoint()?;
        super::engine_session::bind_candidates(rows, registry, self, cancel)
    }
    /// Bind generated or admitted fields without another raw-ingress value scan.
    /// # Errors
    /// Declaration mismatch, cancellation or resource failure.
    pub fn candidate_checked(
        &self,
        rows: BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<EngineSession, EngineError> {
        self.candidate(BTreeMap::new(), registry, cancel)?
            .with_checked_workspace(rows, cancel)
    }
    /// The same allocator used by every session and compiler output.
    pub fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.pool
    }
    /// Borrow the retained native assembly without constructing a new runtime.
    pub fn native_state(&self) -> &SessionState {
        &self.state
    }

    /// Derive a query-scoped factory with a selected actual DataFusion partition
    /// demand while retaining this factory's runtime, allocator, cache service,
    /// deployment CPU admission, implementations and other extensions.
    ///
    /// The resulting session inventory records the selected partition count. This
    /// does not create or replace a CPU owner; query admission still consumes the
    /// shared deployment semaphore.
    #[must_use]
    pub fn with_target_partitions(mut self, target_partitions: std::num::NonZeroUsize) -> Self {
        let mut config = self.state.config().clone();
        config.options_mut().execution.target_partitions = target_partitions.get();
        *self.state.config_mut() = config;
        self.refresh_cache_identity();
        self
    }

    /// Retain this configured factory's actual immutable registry-bound native owner.
    /// Validation is initialized before the binding can reach any consumer.
    /// # Errors
    /// Registry-dependent native assembly fails.
    pub fn bind_model(
        self: &Arc<Self>,
        registry: Arc<Registry>,
    ) -> Result<BoundModel, EngineError> {
        let native = self.model_assembly(&registry)?;
        native.validation_context();
        Ok(BoundModel(Arc::new(BoundModelInner {
            factory: self.clone(),
            native,
        })))
    }
    /// Select the canonical provider policies inherited by every catalog operation.
    /// # Errors
    /// Duplicate identities or conflicting root/invocation policy declarations.
    pub fn with_policies(
        mut self,
        policies: Vec<pse_schema::model::provider::ProviderPolicy>,
    ) -> Result<Self, EngineError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let mut identities = std::collections::BTreeSet::new();
        for policy in &policies {
            if !identities.insert(policy.id) {
                return Err(EngineError::Admission {
                    path: "provider.policy".into(),
                    reason: "duplicate policy identity in factory assembly".into(),
                });
            }
        }
        super::policy::EffectivePolicy::compose(
            OperationPurpose::Query,
            policies.iter().filter(|policy| {
                matches!(
                    policy.scope,
                    ProviderScope::Root | ProviderScope::Invocation
                )
            }),
        )?;
        self.policies = Arc::new(policies);
        self.refresh_cache_identity();
        Ok(self)
    }
}

pub(super) fn new_generation() -> pse_ids::SemanticId {
    pse_ids::SemanticId::from_bytes(*uuid::Uuid::now_v7().as_bytes())
}

impl EngineFactory {
    /// Carry a native extension without replacing any installed implementation.
    #[must_use]
    pub fn with_extension<T: std::fmt::Debug + Send + Sync + 'static>(
        mut self,
        extension: Arc<T>,
    ) -> Self {
        let config = self.state.config().clone().with_extension(extension);
        *self.state.config_mut() = config;
        self.models = Arc::default();
        self.implementation_generation = new_generation();
        self.refresh_cache_identity();
        self
    }
}

/// An immutable binding retaining the actual configured native implementations and registry.
/// Native validation snapshots never retain this handle or its assembly.
#[derive(Clone, Debug)]
pub struct BoundModel(Arc<BoundModelInner>);

#[derive(Debug)]
struct BoundModelInner {
    factory: Arc<EngineFactory>,
    native: Arc<super::assembly::ModelAssembly>,
}

impl BoundModel {
    /// Actual configured factory from which this native owner was assembled.
    pub fn factory(&self) -> &Arc<EngineFactory> {
        &self.0.factory
    }
    /// Registry retained by the native owner.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.0.native.registry
    }
    /// Stable native admission owner, initialized before this binding was returned.
    pub fn validation_context(&self) -> Arc<pse_relations::validate::ValidationContext> {
        self.0.native.validation_context()
    }
    /// Admit an immutable registry reflection relation once for this actual native owner.
    /// Returned clones can be streamed independently; a changed binding admits its own rows.
    /// # Errors
    /// Unknown relation, cancellation or native CHECK refusal.
    pub fn registry_relation(
        &self,
        name: &str,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, EngineError> {
        cancel.checkpoint()?;
        let (key, batch) = self
            .registry()
            .schema_batches()
            .iter()
            .find(|(key, _)| key.qualified_name() == name)
            .ok_or_else(|| EngineError::Admission {
                path: "registry.reflection".into(),
                reason: "unknown declared registry reflection relation".into(),
            })?;
        let mut reflection = self
            .0
            .native
            .reflection
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(checked) = reflection.get(key) {
            return Ok(checked.clone());
        }
        let spec = self
            .registry()
            .relation_by_key(*key)
            .ok_or_else(|| EngineError::Admission {
                path: "registry.reflection".into(),
                reason: "registry reflection declaration absent".into(),
            })?;
        let checked = pse_relations::columnar::FieldCheckedBatch::admit(
            self.registry(),
            spec,
            batch.clone(),
            &self.validation_context(),
            cancel,
        )?;
        reflection.insert(*key, checked.clone());
        Ok(checked)
    }
    /// Construct candidate providers using this exact retained factory and registry pair.
    /// # Errors
    /// Schema/value admission, cancellation or allocation refusal.
    pub fn candidate(
        &self,
        rows: BTreeMap<RelationKey, RecordBatch>,
        cancel: &CancellationToken,
    ) -> Result<EngineSession, EngineError> {
        self.factory()
            .candidate(rows, self.registry().clone(), cancel)
    }
    /// Bind admitted rows using this exact native owner.
    /// # Errors
    /// Declaration mismatch, cancellation or resource failure.
    pub fn candidate_checked(
        &self,
        rows: BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        cancel: &CancellationToken,
    ) -> Result<EngineSession, EngineError> {
        self.factory()
            .candidate_checked(rows, self.registry().clone(), cancel)
    }
}
