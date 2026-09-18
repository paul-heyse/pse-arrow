// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared engine resources without a dependency on the higher runtime/controller crate.
use super::{EngineProfile, ExecutionSettings, SnapshotSession, ThreadBudget};
use crate::CatalogError;
use datafusion::{
    arrow::array::RecordBatch,
    execution::{
        runtime_env::RuntimeEnv,
        session_state::{SessionState, SessionStateBuilder},
    },
};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// Reusable session construction settings with one shared deployment memory pool.
#[derive(Debug)]
pub struct SessionFactory {
    pub(super) implementation_generation: pse_ids::SemanticId,
    pub(super) state: SessionState,
    pub(super) reserver: Arc<dyn MemoryReserver>,
    pub(super) profile: EngineProfile,
    pub(super) rules: Arc<super::EngineRules>,
    pub(super) requirement_planner: Option<Arc<dyn super::policy::RequirementPlanner>>,
    pub(super) policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
}
impl SessionFactory {
    fn refresh_cache_identity(&mut self) {
        let config = self.state.config().clone().with_extension(Arc::new(
            crate::cache_service::resident::CacheIdentity {
                generation: self.implementation_generation,
                policies: self.policies.clone(),
            },
        ));
        self.state = SessionStateBuilder::new_from_existing(self.state.clone())
            .with_config(config)
            .build();
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
        self.state = SessionStateBuilder::new_from_existing(self.state)
            .with_config(config)
            .build();
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
        reserver: Arc<dyn MemoryReserver>,
        settings: ExecutionSettings,
        budget: ThreadBudget,
        profile: EngineProfile,
    ) -> Result<Self, CatalogError> {
        let rules = super::EngineRules::from_profile(&profile)?;
        let builder = SessionStateBuilder::new()
            .with_default_features()
            .with_config(super::config::build(settings, budget)?)
            .with_query_planner(Arc::new(super::planner::UnifiedPlanner::default()))
            .with_analyzer_rules(rules.analyzers)
            .with_optimizer_rules(rules.optimizers)
            .with_physical_optimizer_rules(rules.physical);
        let version = profile.version;
        Ok(Self::from_builder(runtime, reserver, &version, builder))
    }
    /// Freeze a native engine assembly before binding any input or preparing a plan.
    /// Callers may install native scalar, aggregate, window and higher-order functions,
    /// analyzers, expression rewrites, logical/physical optimizers and planners using
    /// `SessionStateBuilder`. The builder's entire native configuration, including
    /// opaque extensions, remains authoritative alongside the actual implementations.
    #[must_use]
    pub fn from_builder(
        runtime: Arc<RuntimeEnv>,
        reserver: Arc<dyn MemoryReserver>,
        version: &str,
        mut builder: SessionStateBuilder,
    ) -> Self {
        builder
            .config()
            .get_or_insert_default()
            .options_mut()
            .extensions
            .insert(super::config::PseOptions);
        // A plain native builder must retain the same Delta/domain planning path
        // as `new`. Explicit caller planners remain the actual implementations;
        // custom assemblies compose domain planners through UnifiedPlanner::new.
        builder
            .query_planner()
            .get_or_insert_with(|| Arc::new(super::planner::UnifiedPlanner::default()));
        builder
            .cache_factory()
            .get_or_insert_with(|| Arc::new(super::cache::NativeCacheFactory));
        let state = super::scalar::register(builder, Arc::clone(&reserver))
            .with_runtime_env(runtime)
            .build();
        let mut physical = state.physical_optimizers().to_vec();
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
            implementation_generation: new_generation(),
            state,
            reserver,
            profile: rules.profile(version),
            rules,
            requirement_planner: None,
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
    ) -> Result<SnapshotSession, CatalogError> {
        cancel.checkpoint()?;
        super::snapshot_session::bind_candidates(rows, registry, self, cancel)
    }
    /// Bind generated or admitted fields without another raw-ingress value scan.
    /// # Errors
    /// Declaration mismatch, cancellation or resource failure.
    pub fn candidate_checked(
        &self,
        rows: BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        self.candidate(BTreeMap::new(), registry, cancel)?
            .with_checked_workspace(rows, cancel)
    }
    /// The same allocator used by every session and compiler output.
    pub fn reserver(&self) -> &Arc<dyn MemoryReserver> {
        &self.reserver
    }
    /// Bind an exact target Delta publication into the common native policy and
    /// execution boundary. Member data remains lazy; normal SQL and supplied plans
    /// use this factory's actual functions, rules, runtime and selected policies.
    /// # Errors
    /// The selected control/member versions or declared contracts cannot be opened.
    pub async fn open_publication(
        &self,
        root: crate::delta::publication::PublicationRoot,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        cancel.checkpoint()?;
        Ok(
            crate::delta::publication::Publication::open(root, registry, self, cancel)
                .await?
                .into_session(),
        )
    }
    pub(crate) fn native_state(&self) -> &SessionState {
        &self.state
    }

    /// Install the actual invariant lowering implementation before opening sessions.
    #[must_use]
    pub fn with_requirement_planner(
        mut self,
        planner: Arc<dyn super::policy::RequirementPlanner>,
    ) -> Self {
        self.implementation_generation = new_generation();
        self.requirement_planner = Some(planner);
        self.refresh_cache_identity();
        self
    }
    /// Select the canonical provider policies inherited by every catalog operation.
    /// # Errors
    /// Duplicate identities or conflicting root/invocation policy declarations.
    pub fn with_policies(
        mut self,
        policies: Vec<pse_schema::model::provider::ProviderPolicy>,
    ) -> Result<Self, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let mut identities = std::collections::BTreeSet::new();
        for policy in &policies {
            if !identities.insert(policy.id) {
                return Err(CatalogError::Admission {
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
