// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use std::{collections::BTreeMap, sync::Arc};

mod derived;
pub use derived::InputSelection;

impl super::EngineSession {
    /// Retain a subsystem's derived indexes in the actual native context.
    /// Extensions must not retain sessions or assemblies: use weak source owners
    /// and owned immutable values to avoid a context ownership cycle.
    pub fn retained_extension<T: Default + Send + Sync + 'static>(&self) -> Arc<T> {
        let state = self.native.context.state_ref();
        if let Some(value) = state.read().config().get_extension::<T>() {
            return value;
        }
        let mut state = state.write();
        if let Some(value) = state.config().get_extension::<T>() {
            return value;
        }
        let value = Arc::new(T::default());
        state.config_mut().set_extension(value.clone());
        value
    }
}

/// The registry-bound native capabilities retained across model selections.
/// Query states clone this context; changing a capability creates a new owner.
#[derive(Clone)]
pub(super) struct ModelAssembly {
    pub planning: Arc<()>,
    pub registry: Arc<pse_schema::Registry>,
    pub context: datafusion::execution::context::SessionContext,
    pub function_bindings: Arc<super::functions::Functions>,
    pub settings: BTreeMap<String, Option<String>>,
    pub functions: pse_ids::ContentHash,
    pub function_names: BTreeMap<String, Vec<String>>,
    pub generation: pse_ids::SemanticId,
}

impl std::fmt::Debug for ModelAssembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelAssembly")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// A derived selection retains its exact inputs. Clones start with the retained
/// answer but diverge independently when a model revision changes a binding.
#[derive(Debug, Default)]
pub(super) struct SelectionCache(std::sync::Mutex<Option<Arc<Selection>>>);

impl Clone for SelectionCache {
    fn clone(&self) -> Self {
        Self(std::sync::Mutex::new(
            self.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        ))
    }
}

#[derive(Debug)]
pub(super) struct Selection {
    native: Arc<ModelAssembly>,
    bindings: crate::provider::binding::Bindings,
    policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
    purpose: pse_schema::model::provider::OperationPurpose,
    pub effective: Arc<super::policy::EffectivePolicy>,
    pub config: datafusion::execution::context::SessionConfig,
    pub catalogs: Arc<crate::provider::list::SnapshotCatalogList>,
}

impl Selection {
    pub(super) fn matches_owners(&self, session: &super::EngineSession) -> bool {
        Arc::ptr_eq(&self.native, &session.native)
            && self.bindings.same_selection(&session.bindings)
            && Arc::ptr_eq(&self.policies, &session.policies)
            && self.purpose == session.purpose
    }
}

impl super::EngineSession {
    pub(super) fn selection(&self) -> Result<Arc<Selection>, crate::EngineError> {
        let mut cached = self
            .selection
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(selected) = cached.as_ref()
            && selected.matches_owners(self)
        {
            return Ok(selected.clone());
        }
        let effective = self.compose_policy()?;
        // This is selection construction, not an execution boundary. Cloning
        // under the native read lock does not start a query or cross an await.
        let state = self.native.context.state_ref().read().clone();
        let (state, _settings) =
            super::config::EffectiveSettings::resolve(state, &effective.settings)?;
        let config = state
            .config()
            .clone()
            .with_extension(Arc::new(AssemblyIdentity {
                generation: self.implementation_generation(),
                policies: Arc::new(
                    self.policies
                        .iter()
                        .map(super::config::semantic_policy)
                        .collect(),
                ),
                settings: Some(Arc::new(super::config::semantic_settings(
                    &super::config::inventory(&state),
                )?)),
            }));
        let defaults = &config.options().catalog;
        let catalogs = self
            .bindings
            .catalogs(&defaults.default_catalog, &defaults.default_schema)
            .map_err(super::engine_session::engine)?;
        catalogs.freeze();
        let selected = Arc::new(Selection {
            native: self.native.clone(),
            bindings: self.bindings.clone(),
            policies: self.policies.clone(),
            purpose: self.purpose,
            effective: Arc::new(effective),
            config,
            catalogs: Arc::new(catalogs),
        });
        *cached = Some(selected.clone());
        Ok(selected)
    }
}

#[derive(Debug, Clone)]
/// Native typed extension distinguishing real implementation assemblies.
pub struct AssemblyIdentity {
    /// Generation of the actual function, rule and planner implementations.
    pub generation: pse_ids::SemanticId,
    /// The retained declared provider policy hierarchy.
    pub policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
    /// Semantic settings retained after selected policy resolution.
    pub settings: Option<Arc<BTreeMap<String, Option<String>>>>,
}

#[cfg(test)]
mod integrated_performance_unit;
