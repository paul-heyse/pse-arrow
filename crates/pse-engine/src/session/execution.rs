// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed native session extensions for specialized domain execution.
//!
//! An extension planner receives DataFusion's actual `SessionState`. This extension
//! supplies the matching domain declaration, cancellation and allocation owners;
//! nested algorithm queries never construct a default engine or retain an ancestor
//! query session. It contains no model facts, providers or producing plans.
use super::EngineSession;
use crate::EngineError;
use datafusion::{
    catalog::Session,
    common::{DataFusionError, Result},
    execution::session_state::SessionState,
};

use pse_columnar::{CancellationToken, MemoryPool};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::provider::ProviderScope};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Actual PSE execution services bound into native SessionConfig for this operation.
#[derive(Debug)]
pub struct NativeExecutionContext {
    template: EngineSession,
    pub(super) caches: Arc<super::cache::CacheStore>,
    completed_state: std::sync::Mutex<Option<std::sync::Weak<SessionState>>>,
    settlement: std::sync::atomic::AtomicBool,
    effective: Arc<super::policy::EffectivePolicy>,
    cancel: CancellationToken,
    scopes: BTreeSet<ProviderScope>,
    inherited: bool,
    parent_admission: std::sync::Weak<tokio::sync::OwnedSemaphorePermit>,
    query_admission: std::sync::Mutex<std::sync::Weak<tokio::sync::OwnedSemaphorePermit>>,
}
impl NativeExecutionContext {
    pub(crate) fn port_store(&self) -> &crate::operation::ports::Store {
        &self.caches.3
    }
    /// Invalidate invocation-local round-dependent completions after every stream
    /// of the previous round has settled. Physical plan ownership remains native.
    pub fn advance_round_epoch(&self) {
        self.caches.advance_epoch();
    }
    pub(crate) fn bind(
        session: &EngineSession,
        state: SessionState,
        cancel: &CancellationToken,
    ) -> Result<SessionState, EngineError> {
        let services = Arc::new(Self {
            template: session.empty_selection()?,
            caches: session.invocation.clone().unwrap_or_default(),
            inherited: session.invocation.is_some(),
            parent_admission: session.query_admission.clone(),
            query_admission: std::sync::Mutex::default(),
            completed_state: std::sync::Mutex::new(None),
            settlement: std::sync::atomic::AtomicBool::new(false),
            effective: session.selection()?.effective.clone(),
            cancel: cancel.clone(),
            scopes: session
                .bindings
                .targets()
                .cloned()
                .chain(
                    session
                        .bindings
                        .scopes()
                        .map(|(catalog, schema)| match schema {
                            Some(schema) => ProviderScope::Schema(catalog, schema),
                            None => ProviderScope::Catalog(catalog),
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
        for (_, binding) in session.bindings.iter() {
            if let Some(ownership) = &binding.ownership {
                services
                    .ownership()
                    .import_scope(ownership)
                    .map_err(EngineError::from)?;
            }
            if let Some(owned) = binding.checked.as_ref().and_then(FieldCheckedBatch::owned) {
                services
                    .ownership()
                    .import(owned)
                    .map_err(EngineError::from)?;
            }
        }
        let config = state
            .config()
            .clone()
            .with_extension(Arc::new(services.ownership().clone()))
            .with_extension(Arc::new(services.attempt_scope()))
            .with_extension(services);
        // SQL is resolved against the retained context before this boundary.
        // Only execution receives a distinct runtime for a scoped resource pool.
        let mut state = if let Some(runtime) = &session.execution_runtime {
            datafusion::execution::session_state::SessionStateBuilder::new_from_existing(state)
                .with_runtime_env(runtime.clone())
                .build()
        } else {
            state
        };
        *state.config_mut() = config;
        Ok(state)
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
    /// Nested native queries use their live outer query's admission.
    pub(crate) fn inherited_query_admission(
        &self,
    ) -> Option<Arc<tokio::sync::OwnedSemaphorePermit>> {
        self.parent_admission.upgrade()
    }
    /// Retain the enclosing query admission through an actual blocking foreign job.
    pub fn running_query_admission(&self) -> Option<Arc<tokio::sync::OwnedSemaphorePermit>> {
        self.query_admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade()
            .or_else(|| self.parent_admission.upgrade())
    }
    pub(super) fn register_query_admission(&self, permit: &Arc<tokio::sync::OwnedSemaphorePermit>) {
        *self
            .query_admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(permit);
    }

    /// Nested validation shares the running command's admission and invocation owners.
    pub(super) fn inherit_invocation(&self, session: &mut EngineSession) {
        session.invocation = Some(self.caches.clone());
        session.query_admission = self
            .running_query_admission()
            .as_ref()
            .map(Arc::downgrade)
            .unwrap_or_default();
    }

    /// Opaque actual invocation owner, shared by sibling and nested operations.
    pub fn attempt_scope(&self) -> AttemptScope {
        self.caches.4.scope.clone()
    }

    /// Retain a typed invocation resource. Values must not retain a session, plan,
    /// or this context. Construction is synchronous; callers account value storage.
    /// # Errors
    /// Resource construction or the invocation resource lock fails.
    pub fn invocation_resource<T: std::fmt::Debug + Send + Sync + 'static>(
        &self,
        create: impl FnOnce() -> Result<T>,
    ) -> Result<Arc<T>> {
        self.caches.4.get(self.pool(), create)
    }

    /// Look up attempt-owned state without creating or charging an absent resource.
    /// # Errors
    /// Poisoned resource storage or inconsistent runtime type.
    pub fn find_invocation_resource<T: std::fmt::Debug + Send + Sync + 'static>(
        &self,
    ) -> Result<Option<Arc<T>>> {
        self.caches.4.find::<T>()
    }
    /// Exact declaration owner admitted by common preparation.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.template.registry
    }
    /// Shared operation allocation budget, including the selected policy ceiling.
    pub fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.template.pool
    }
    /// Explicit allocation provenance shared by this invocation's producers.
    pub fn ownership(&self) -> &pse_columnar::owned_buffer::AllocationScope {
        &self.caches.2
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
    ) -> Result<(), EngineError> {
        self.effective.admit(effects)
    }

    /// Each execution of a prepared plan gets its own settlement state. Concurrent
    /// or repeated executions must not inherit another stream's cancellation mode.
    pub(crate) fn execution_state(
        state: &SessionState,
        cancel: &CancellationToken,
    ) -> Result<SessionState> {
        Self::execution_state_with_caches(
            state,
            cancel,
            state
                .config()
                .get_extension::<Self>()
                .filter(|source| source.inherited)
                .map_or_else(
                    || Arc::new(super::cache::CacheStore::default()),
                    |source| source.caches.clone(),
                ),
        )
    }
    pub(super) fn execution_state_with_caches(
        state: &SessionState,
        cancel: &CancellationToken,
        caches: Arc<super::cache::CacheStore>,
    ) -> Result<SessionState> {
        let Some(source) = state.config().get_extension::<Self>() else {
            return Ok(state.clone());
        };
        caches
            .2
            .import_scope(source.ownership())
            .map_err(pse_columnar::external)?;
        let services = Arc::new(Self {
            caches,
            inherited: source.inherited,
            parent_admission: source.parent_admission.clone(),
            query_admission: std::sync::Mutex::default(),
            template: source.template.clone(),
            completed_state: std::sync::Mutex::new(None),
            settlement: std::sync::atomic::AtomicBool::new(false),
            effective: source.effective.clone(),
            cancel: cancel.child_token(),
            scopes: source.scopes.clone(),
        });
        let mut state = state.clone();
        *state.config_mut() = state
            .config()
            .clone()
            .with_extension(Arc::new(services.ownership().clone()))
            .with_extension(Arc::new(services.attempt_scope()))
            .with_extension(services);
        state.mark_start_execution();
        Ok(state)
    }

    /// SQL definition statements operate on a clone of the retained model context.
    /// They execute no data operator and must not lose earlier registrations when
    /// an attempt has a distinct allocation/cache runtime.
    pub(super) fn definition_state(&self, attempt: &SessionState) -> SessionState {
        let mut state = self.template.native.context.state();
        *state.config_mut() = attempt.config().clone();
        state.register_catalog_list(attempt.catalog_list().clone());
        state
    }

    /// Command state is owned by its physical result, never by this configuration
    /// extension. A weak link avoids state -> extension -> state reference cycles.
    pub(super) fn record_command_state(&self, state: &Arc<SessionState>) -> Result<()> {
        *self.completed_state.lock().map_err(|_| {
            DataFusionError::Internal("native command state lock poisoned".into())
        })? = Some(Arc::downgrade(state));
        Ok(())
    }
    pub(super) fn command_state(&self, original: &SessionState) -> Result<SessionState> {
        Ok(self
            .completed_state
            .lock()
            .map_err(|_| DataFusionError::Internal("native command state lock poisoned".into()))?
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .map_or_else(|| original.clone(), |state| state.as_ref().clone()))
    }

    /// Called immediately before starting cooperative foreign work that cannot
    /// be aborted by dropping a future. The native stream must then await that work
    /// and yield its terminal outcome (or error), including on cancellation.
    /// No result or producer is retained here; the physical stream owns settlement.
    pub fn require_settlement(&self) {
        self.settlement
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub(crate) fn must_settle(&self) -> bool {
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
    ) -> Result<EngineSession, EngineError> {
        if state
            .config()
            .get_extension::<Self>()
            .is_none_or(|actual| !Arc::ptr_eq(&actual, self))
        {
            return Err(EngineError::Admission {
                path: "native.execution".into(),
                reason: "execution state has different actual domain services".into(),
            });
        }
        self.cancel.checkpoint()?;
        let mut session = self
            .template
            .with_checked_role_inputs(rows, &self.cancel)?
            .with_unique_relation_aliases(&self.cancel)?;
        // A store-bound runtime is an actual distinct resource/cache scope.
        // Ordinary nested work keeps the retained model context unchanged.
        let model_runtime = session.native.context.runtime_env();
        let runtime = session.execution_runtime.as_ref().unwrap_or(&model_runtime);
        if !Arc::ptr_eq(runtime, state.runtime_env()) {
            session.execution_runtime = Some(state.runtime_env().clone());
        }
        for scope in &self.scopes {
            session.bindings.target(scope.clone());
        }
        self.inherit_invocation(&mut session);
        Ok(session)
    }
}

#[cfg(test)]
mod tests;

/// An owned identity, never an unowned address or durable attempt label.
#[derive(Clone, Debug)]
pub struct AttemptScope(Arc<()>);
impl Default for AttemptScope {
    fn default() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for AttemptScope {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for AttemptScope {}
impl std::hash::Hash for AttemptScope {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&Arc::as_ptr(&self.0), state);
    }
}

type InvocationValue = (
    Arc<dyn std::any::Any + Send + Sync>,
    pse_columnar::MemoryReservation,
);

#[derive(Default)]
pub(super) struct InvocationResources {
    scope: AttemptScope,
    values: std::sync::Mutex<std::collections::HashMap<std::any::TypeId, InvocationValue>>,
}
impl std::fmt::Debug for InvocationResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvocationResources")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}
impl InvocationResources {
    fn find<T: Send + Sync + 'static>(&self) -> Result<Option<Arc<T>>> {
        self.values
            .lock()
            .map_err(|_| DataFusionError::Internal("invocation resources poisoned".into()))?
            .get(&std::any::TypeId::of::<T>())
            .map(|(value, _)| {
                value.clone().downcast::<T>().map_err(|_| {
                    DataFusionError::Internal("invocation resource type mismatch".into())
                })
            })
            .transpose()
    }
    fn get<T: std::fmt::Debug + Send + Sync + 'static>(
        &self,
        pool: &Arc<dyn MemoryPool>,
        create: impl FnOnce() -> Result<T>,
    ) -> Result<Arc<T>> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| DataFusionError::Internal("invocation resources poisoned".into()))?;
        if let Some((value, _)) = values.get(&std::any::TypeId::of::<T>()) {
            return value.clone().downcast::<T>().map_err(|_| {
                DataFusionError::Internal("invocation resource type mismatch".into())
            });
        }
        let owner = pse_columnar::MemoryConsumer::new("native:invocation-resource").register(pool);
        owner.try_grow(size_of::<T>().saturating_add(512))?;
        let value = Arc::new(create()?);
        values.insert(std::any::TypeId::of::<T>(), (value.clone(), owner));
        Ok(value)
    }
}
