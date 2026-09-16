// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One policy composition for every native plan and operation purpose.

use super::SnapshotSession;
use crate::CatalogError;
use datafusion::logical_expr::LogicalPlan;
use pse_ids::SemanticId;
use pse_schema::model::provider::{
    OperationEffect, OperationPurpose, ProviderPolicy, ProviderScope,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Actual implementation that lowers required invariant IDs to a native violation plan.
/// A zero-row successful execution discharges the selected requirements for its exact
/// captured sources. It does not validate unrelated bundle/publication obligations.
pub trait RequirementPlanner: std::fmt::Debug + Send + Sync {
    /// Lower all selected requirements. Missing declarations/dependencies must fail.
    /// # Errors
    /// Unsupported requirement, invalid dependency or native planning failure.
    fn plan(
        &self,
        session: &SnapshotSession,
        requirements: &BTreeSet<SemanticId>,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<LogicalPlan, CatalogError>;
}

/// Immutable effective policy. Values and their origins are derived together.
#[derive(Clone, Debug)]
pub struct EffectivePolicy {
    /// Actual invariant requirements, conjunctive across applicable scopes.
    pub requirements: BTreeSet<SemanticId>,
    /// Purpose and scoped effect ceilings intersect; implementation support is separate.
    pub effects: BTreeSet<OperationEffect>,
    /// Resolved native configuration values.
    pub settings: BTreeMap<String, String>,
    /// Declaration owners that selected each resolved setting.
    pub setting_origins: BTreeMap<String, BTreeSet<SemanticId>>,
    /// Effective accounted allocation ceiling, always bounded by ancestor ceilings.
    pub max_bytes: Option<usize>,
}

impl EffectivePolicy {
    /// Compose already applicable declarations; no provider-specific interpretation.
    /// # Errors
    /// Incompatible requirements/defaults, conflicting declaration identities or zero limit.
    pub fn compose<'a>(
        purpose: OperationPurpose,
        policies: impl IntoIterator<Item = &'a ProviderPolicy>,
    ) -> Result<Self, CatalogError> {
        let mut result = Self {
            requirements: BTreeSet::new(),
            effects: purpose.effects(),
            settings: BTreeMap::new(),
            setting_origins: BTreeMap::new(),
            max_bytes: None,
        };
        let policies: Vec<_> = policies.into_iter().collect();
        let mut declarations = BTreeMap::new();
        let mut defaults = BTreeMap::<String, (u8, BTreeMap<String, BTreeSet<SemanticId>>)>::new();
        let mut required = BTreeMap::<String, (String, BTreeSet<SemanticId>)>::new();
        for policy in policies {
            if declarations
                .insert(policy.id, policy)
                .is_some_and(|previous| previous != policy)
            {
                return Err(invalid("one policy identity has conflicting declarations"));
            }
            result.requirements.extend(&policy.requirements);
            result
                .effects
                .retain(|effect| policy.effects.contains(effect));
            if let Some(limit) = policy.max_bytes {
                if limit == 0 {
                    return Err(invalid("an accounted budget must be positive"));
                }
                result.max_bytes = Some(result.max_bytes.map_or(limit, |parent| parent.min(limit)));
            }
            for (key, value) in &policy.required_settings {
                let entry = required
                    .entry(key.clone())
                    .or_insert_with(|| (value.clone(), BTreeSet::new()));
                if entry.0 != *value {
                    return Err(invalid(&format!("incompatible required setting {key}")));
                }
                entry.1.insert(policy.id);
            }
            for (key, value) in &policy.defaults {
                let rank = policy.scope.precedence();
                let entry = defaults
                    .entry(key.clone())
                    .or_insert_with(|| (rank, BTreeMap::new()));
                if rank > entry.0 {
                    *entry = (rank, BTreeMap::new());
                }
                if rank == entry.0 {
                    entry.1.entry(value.clone()).or_default().insert(policy.id);
                }
            }
        }
        for (key, (rank, choices)) in defaults {
            if let Some((value, _)) = required.get(&key) {
                if rank == ProviderScope::Invocation.precedence()
                    && choices.keys().any(|choice| choice != value)
                {
                    return Err(invalid(&format!(
                        "invocation conflicts with required setting {key}"
                    )));
                }
                continue;
            }
            if choices.len() != 1 {
                return Err(invalid(&format!(
                    "conflicting scoped defaults for {key}; select an invocation value"
                )));
            }
            if let Some((value, origins)) = choices.into_iter().next() {
                result.settings.insert(key.clone(), value);
                result.setting_origins.insert(key, origins);
            }
        }
        for (key, (value, origins)) in required {
            result.settings.insert(key.clone(), value);
            result.setting_origins.insert(key, origins);
        }
        Ok(result)
    }

    pub(super) fn admit(&self, effects: &BTreeSet<OperationEffect>) -> Result<(), CatalogError> {
        if let Some(effect) = effects.difference(&self.effects).next() {
            return Err(invalid(&format!(
                "operation effect {} is outside the effective purpose/policy",
                effect.as_str()
            )));
        }
        Ok(())
    }
}

impl SnapshotSession {
    pub(super) async fn check_requirements(
        &self,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<(), CatalogError> {
        if let Some(requirements) = self.prepare_requirements(cancel)? {
            let mut stream = Box::pin(requirements.execute_stream(cancel)).await?;
            while let Some(batch) = stream.next_batch(cancel).await? {
                if batch.num_rows() != 0 {
                    return Err(invalid("result violates scoped invariant requirements"));
                }
            }
        }
        Ok(())
    }
    /// Bind a canonical policy in a new generation; existing preparations remain unchanged.
    /// # Errors
    /// Repeated identity or an incompatible effective policy.
    pub fn with_policy(&self, policy: ProviderPolicy) -> Result<Self, CatalogError> {
        self.with_policies([policy])
    }

    /// Bind related scope declarations atomically, including their invocation selection.
    /// # Errors
    /// Repeated identity or an incompatible combined policy.
    pub fn with_policies(
        &self,
        policies: impl IntoIterator<Item = ProviderPolicy>,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for policy in policies {
            if result
                .policies
                .iter()
                .any(|existing| existing.id == policy.id)
            {
                return Err(invalid(
                    "policy identity is already bound; construct a new generation",
                ));
            }
            Arc::make_mut(&mut result.policies).push(policy);
        }
        result.effective_policy()?;
        Ok(result)
    }

    /// Select an explicit purpose. Existing scope ceilings still apply.
    #[must_use]
    pub fn with_purpose(&self, purpose: OperationPurpose) -> Self {
        let mut result = self.clone();
        result.purpose = purpose;
        result
    }

    /// Explain the exact combined policy over the currently captured binding scopes.
    /// # Errors
    /// Incompatible defaults or requirements across participating scopes.
    pub fn effective_policy(&self) -> Result<EffectivePolicy, CatalogError> {
        EffectivePolicy::compose(
            self.purpose,
            self.policies.iter().filter(|policy| {
                matches!(
                    policy.scope,
                    ProviderScope::Root | ProviderScope::Invocation
                ) || self.bindings.scopes().any(|(catalog, schema)| {
                    policy.scope.covers(catalog, schema.unwrap_or_default(), "")
                }) || self.bindings.targets().any(|target| match target {
                    ProviderScope::Catalog(c) => policy.scope.covers(c, "", ""),
                    ProviderScope::Schema(c, s) => policy.scope.covers(c, s, ""),
                    ProviderScope::Table(c, s, t) => policy.scope.covers(c, s, t),
                    ProviderScope::Root | ProviderScope::Invocation => false,
                }) || self.bindings.iter().any(|(_, binding)| {
                    policy.scope.covers(
                        binding.reference.catalog().unwrap_or_default(),
                        binding.reference.schema().unwrap_or_default(),
                        binding.reference.table(),
                    )
                })
            }),
        )
    }

    pub(super) fn admit_effects(
        &self,
        plan: &LogicalPlan,
        varying: bool,
    ) -> Result<BTreeSet<OperationEffect>, CatalogError> {
        let policy = self.effective_policy()?;
        let mut effects = BTreeSet::from([OperationEffect::Read]);
        for (_, binding) in self.bindings.iter() {
            effects.extend(&binding.effects);
        }
        if varying {
            effects.insert(OperationEffect::Nondeterministic);
        }
        plan.apply_with_subqueries(|node| {
            if matches!(node, LogicalPlan::Explain(_)) {
                return Ok(datafusion::common::tree_node::TreeNodeRecursion::Jump);
            }
            match node {
                LogicalPlan::Extension(extension) => {
                    if extension
                        .node
                        .as_any()
                        .is::<crate::delta::write::DeltaWrite>()
                    {
                        effects.insert(OperationEffect::Write);
                    }
                    if extension
                        .node
                        .as_any()
                        .is::<crate::delta::publish::DeltaPublish>()
                    {
                        effects.insert(OperationEffect::Write);
                        effects.insert(OperationEffect::Publish);
                    }
                    if let Some(operation) = extension
                        .node
                        .as_any()
                        .downcast_ref::<super::operation::OperationNode>()
                    {
                        effects.extend(&operation.effects);
                    }
                }
                LogicalPlan::Dml(_) => {
                    effects.insert(OperationEffect::Write);
                }
                LogicalPlan::Ddl(_) | LogicalPlan::Statement(_) => {
                    effects.insert(OperationEffect::Namespace);
                }
                LogicalPlan::Copy(_) => {
                    effects.insert(OperationEffect::Publish);
                }
                _ => {}
            }
            Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
        })
        .map_err(super::snapshot_session::engine)?;
        policy.admit(&effects)?;
        Ok(effects)
    }

    pub(super) fn prepare_requirements(
        &self,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<Option<super::PreparedComputation>, CatalogError> {
        let requirements = self.effective_policy()?.requirements;
        if requirements.is_empty() {
            return Ok(None);
        }
        let planner = self
            .requirement_planner
            .as_ref()
            .ok_or_else(|| invalid("required invariants have no bound native implementation"))?;
        let mut diagnostic = self.clone();
        // Only the internal discharge preparation omits its own recursive obligations.
        for policy in Arc::make_mut(&mut diagnostic.policies) {
            policy.requirements.clear();
        }
        diagnostic.purpose = OperationPurpose::Inspect;
        let plan = planner.plan(&diagnostic, &requirements, cancel)?;
        Ok(Some(diagnostic.prepare_rule_plan(plan, cancel)?))
    }
}

fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.policy".to_owned(),
        reason: reason.to_owned(),
    }
}

pub(super) fn policies_extent(policies: &[ProviderPolicy]) -> Result<usize, CatalogError> {
    policies.iter().try_fold(0_usize, |total, policy| {
        let scope = match &policy.scope {
            ProviderScope::Root | ProviderScope::Invocation => 0,
            ProviderScope::Catalog(c) => c.capacity(),
            ProviderScope::Schema(c, s) => c.capacity().saturating_add(s.capacity()),
            ProviderScope::Table(c, s, t) => c
                .capacity()
                .saturating_add(s.capacity())
                .saturating_add(t.capacity()),
        };
        let nodes = policy
            .requirements
            .len()
            .checked_add(policy.effects.len())
            .and_then(|count| count.checked_mul(128))
            .ok_or_else(|| invalid("policy allocation extent overflows"))?;
        let base = total
            .checked_add(512)
            .and_then(|n| n.checked_add(scope))
            .and_then(|n| n.checked_add(nodes))
            .ok_or_else(|| invalid("policy allocation extent overflows"))?;
        policy
            .defaults
            .iter()
            .chain(&policy.required_settings)
            .try_fold(base, |n, (k, v)| {
                n.checked_add(128)
                    .and_then(|n| n.checked_add(k.capacity()))
                    .and_then(|n| n.checked_add(v.capacity()))
                    .ok_or_else(|| invalid("policy allocation extent overflows"))
            })
    })
}
