// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed native session extensions for specialized domain execution.
//!
//! An extension planner receives DataFusion's actual `SessionState`. This extension
//! supplies the matching domain declaration, cancellation and allocation owners;
//! nested algorithm queries never construct a default engine or retain an ancestor
//! query session. It contains no model facts, providers or producing plans.
use super::{EngineProfile, EngineRules, SessionFactory, SnapshotSession};
use crate::CatalogError;
use datafusion::{
    catalog::Session,
    common::{DataFusionError, Result},
    execution::session_state::{SessionState, SessionStateBuilder},
};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::provider::{OperationPurpose, ProviderPolicy, ProviderScope},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Actual PSE execution services bound into native SessionConfig for this operation.
#[derive(Debug)]
pub struct NativeExecutionContext {
    settlement: std::sync::atomic::AtomicBool,
    effective: super::policy::EffectivePolicy,
    registry: Arc<Registry>,
    reserver: Arc<dyn MemoryReserver>,
    cancel: CancellationToken,
    profile: EngineProfile,
    policies: Arc<Vec<ProviderPolicy>>,
    purpose: OperationPurpose,
    requirements: Option<Arc<dyn super::policy::RequirementPlanner>>,
    scopes: BTreeSet<ProviderScope>,
    trace: Arc<super::trace::ExecutionTrace>,
}
impl NativeExecutionContext {
    pub(super) fn bind(
        session: &SnapshotSession,
        state: SessionState,
        cancel: &CancellationToken,
    ) -> Result<SessionState, CatalogError> {
        let services = Arc::new(Self {
            settlement: std::sync::atomic::AtomicBool::new(false),
            effective: session.effective_policy()?,
            registry: Arc::clone(&session.registry),
            reserver: Arc::clone(&session.reserver),
            cancel: cancel.clone(),
            profile: session.profile.clone(),
            policies: Arc::clone(&session.policies),
            purpose: session.purpose,
            requirements: session.requirement_planner.clone(),
            trace: Arc::clone(&session.trace),
            scopes: session
                .bindings
                .targets()
                .cloned()
                .chain(
                    session
                        .bindings
                        .scopes()
                        .map(|(catalog, schema)| match schema {
                            Some(schema) => {
                                ProviderScope::Schema(catalog.to_owned(), schema.to_owned())
                            }
                            None => ProviderScope::Catalog(catalog.to_owned()),
                        }),
                )
                .chain(session.bindings.iter().map(|(_, binding)| {
                    ProviderScope::Table(
                        binding.reference.catalog().unwrap_or_default().to_owned(),
                        binding.reference.schema().unwrap_or_default().to_owned(),
                        binding.reference.table().to_owned(),
                    )
                }))
                .collect(),
        });
        let config = state.config().clone().with_extension(services);
        Ok(SessionStateBuilder::new_from_existing(state)
            .with_config(config)
            .build())
    }

    /// Obtain the actual declaration and resource owners at physical planning.
    /// # Errors
    /// The caller supplied no actual `SessionState` or bypassed common preparation.
    pub fn from_session(session: &dyn Session) -> Result<Arc<Self>> {
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("domain execution requires the actual `SessionState`".into())
            })?;
        state.config().get_extension::<Self>().ok_or_else(|| {
            DataFusionError::Plan(
                "domain execution services are not bound in this SessionState".into(),
            )
        })
    }
    /// Exact declaration owner admitted by common preparation.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }
    /// Shared operation allocation budget, including the selected policy ceiling.
    pub fn reserver(&self) -> &Arc<dyn MemoryReserver> {
        &self.reserver
    }
    /// Cooperative cancellation bound when the native operation was prepared.
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Admit a specialized physical operator against the same actual policy as
    /// native scans and commands. Unknown domain effects cannot bypass scope ceilings.
    /// # Errors
    /// An effect is outside the invocation purpose or an applicable scope policy.
    pub fn admit_effects(
        &self,
        effects: &BTreeSet<pse_schema::model::provider::OperationEffect>,
    ) -> Result<(), CatalogError> {
        self.effective.admit(effects)
    }

    /// Each execution of a prepared plan gets its own settlement state. Concurrent
    /// or repeated executions must not inherit another stream's cancellation mode.
    pub(super) fn execution_state(
        state: &SessionState,
        cancel: &CancellationToken,
    ) -> SessionState {
        let Some(source) = state.config().get_extension::<Self>() else {
            return state.clone();
        };
        let services = Arc::new(Self {
            settlement: std::sync::atomic::AtomicBool::new(false),
            effective: source.effective.clone(),
            registry: Arc::clone(&source.registry),
            reserver: Arc::clone(&source.reserver),
            cancel: cancel.child_token(),
            profile: source.profile.clone(),
            policies: Arc::clone(&source.policies),
            purpose: source.purpose,
            requirements: source.requirements.clone(),
            scopes: source.scopes.clone(),
            trace: Arc::clone(&source.trace),
        });
        SessionStateBuilder::new_from_existing(state.clone())
            .with_config(state.config().clone().with_extension(services))
            .build()
    }

    /// Called immediately before starting cooperative foreign work that cannot
    /// be aborted by dropping a future. The native stream must then await that work
    /// and yield its terminal outcome (or error), including on cancellation.
    /// No result or producer is retained here; the physical stream owns settlement.
    pub fn require_settlement(&self) {
        self.settlement
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub(super) fn must_settle(&self) -> bool {
        self.settlement.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Bind actual executed child facts as algorithm inputs, preserving the native
    /// caller's complete configuration, functions, planners, resources and policies.
    /// This creates only a private name scope; it installs no default features.
    /// # Errors
    /// A foreign native service binding, invalid child facts, cancellation or resources.
    pub fn candidate_roles(
        self: &Arc<Self>,
        state: &SessionState,
        rows: BTreeMap<String, FieldCheckedBatch>,
    ) -> Result<SnapshotSession, CatalogError> {
        if state
            .config()
            .get_extension::<Self>()
            .is_none_or(|actual| !Arc::ptr_eq(&actual, self))
        {
            return Err(CatalogError::Admission {
                path: "native.execution".into(),
                reason: "execution state has different actual domain services".into(),
            });
        }
        let factory = SessionFactory {
            state: state.clone(),
            reserver: Arc::clone(&self.reserver),
            profile: self.profile.clone(),
            rules: Arc::new(EngineRules {
                analyzers: state.analyzer().rules.clone(),
                optimizers: state.optimizer().rules.clone(),
                physical: state.physical_optimizers().to_vec(),
            }),
            function_bindings: super::functions::Functions::from_state(state),
            requirement_planner: self.requirements.clone(),
            policies: Arc::clone(&self.policies),
        };
        let mut session = factory
            .candidate_checked_ports(rows, Arc::clone(&self.registry), &self.cancel)?
            .with_purpose(self.purpose);
        for scope in &self.scopes {
            session.bindings.target(scope.clone());
        }
        session.trace = Arc::clone(&self.trace);
        Ok(session)
    }
}

#[cfg(test)]
mod tests;
