// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deployment composition for the mandatory canonical scientific substrate.

use crate::math::portable::ReplayAdmission;
use pse_ids::ContentHash;
use pse_operations::canonical::CanonicalStore;

/// Available deployment attestation. It describes the executable containing the
/// library; it is deliberately distinct from relevant compilation producer keys.
#[derive(Clone, Copy, Debug)]
pub struct OuterAttestation {
    /// Complete source observation, absent for an installation without its authored checkout.
    pub source: Option<ContentHash>,
    /// Observed build/deployed artifact context recorded by the composition root.
    pub build: ContentHash,
}

/// One configured canonical store and independently qualified reuse producer.
#[derive(Clone, Debug)]
pub struct CanonicalDeployment {
    store: CanonicalStore,
    attestation: OuterAttestation,
    producer: Option<ReplayAdmission>,
}
impl CanonicalDeployment {
    /// Attach an already opened, interpretation-checked canonical database.
    /// An incomplete producer permits ordinary admission while refusing persisted reuse.
    pub fn new(
        store: CanonicalStore,
        attestation: OuterAttestation,
        producer: Option<ReplayAdmission>,
    ) -> Self {
        Self {
            store,
            attestation,
            producer,
        }
    }
    pub(super) fn independent_caller(&self) -> Result<Self, super::WorkflowError> {
        Ok(Self {
            store: self.store.independent_caller()?,
            ..self.clone()
        })
    }
    /// The sole canonical scientific storage boundary.
    pub fn store(&self) -> &CanonicalStore {
        &self.store
    }
    /// Eligible relevant mathematical producer, when deployment inputs were qualified.
    pub fn producer(&self) -> Option<&ReplayAdmission> {
        self.producer.as_ref()
    }
    /// Complete executable provenance used by result lineage and publications.
    pub const fn attestation(&self) -> OuterAttestation {
        self.attestation
    }
}
impl super::Runtime {
    /// Canonical scientific storage shared by authoring, preparation and retained queries.
    pub fn canonical_store(&self) -> Result<&CanonicalStore, super::WorkflowError> {
        Ok(self.canonical()?.store())
    }
}

/// Actual composition-root evidence supplied by a controlled worker or imported module.
pub struct DeploymentHost {
    /// Loaded artifact selected and reverified by the host.
    pub artifact: std::path::PathBuf,
    /// Optional operator-selected strict producer receipt.
    pub receipt: Option<std::path::PathBuf>,
    /// Expected actual host role.
    pub target: crate::math::portable::ExpectedProducerTarget,
    /// Concrete synchronous code address in that host.
    pub anchor: usize,
    /// Complete bounded configuration under the host's controlled lifetime.
    pub configuration: crate::math::portable::RuntimeConfigurationObserver,
}
impl std::fmt::Debug for DeploymentHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeploymentHost").field("artifact", &self.artifact)
            .field("receipt", &self.receipt).field("target", &self.target)
            .field("anchor", &self.anchor).finish_non_exhaustive()
    }
}
impl CanonicalDeployment {
    /// Observe deployment provenance, reverify the host, then connect/open the store.
    /// The escaped observation stays owned through store publication or refusal.
    /// # Safety
    /// The actual host must satisfy both `MathService::observe_deployment_runtime`
    /// and `MathService::observe_local_runtime` provenance/controlled-root contracts.
    /// `verify_loaded` must verify this actual artifact immediately on both calls.
    /// # Errors
    /// Original host verification, observation, cancellation or store refusal.
    #[allow(unsafe_code, reason = "ADR-0164 composition roots retain their per-host provenance assertion")]
    pub async unsafe fn open_observed(
        math: &std::sync::Arc<crate::math::MathService>,
        host: DeploymentHost,
        state: &std::path::Path,
        database: Option<&str>,
        stop: &crate::CancelSource,
        verify_loaded: impl Fn() -> Result<(), super::WorkflowError>,
    ) -> Result<Self, super::WorkflowError> {
        use crate::math::portable::{PortableError, ReplayAdmission};
        let typed = |error: PortableError| super::WorkflowError::Typed(
            pse_model::diagnostic::DiagnosticCause::new(pse_model::diagnostic::project_typed(
                &error, pse_diagnostics::DiagnosticStage::ModelingAdmission,
            )),
        );
        let checkpoint = || stop.checkpoint().map_err(|_| super::WorkflowError::Math(crate::math::MathRuntimeError::Cancelled));
        let deadline = std::time::Instant::now()
            .checked_add(ReplayAdmission::STARTUP_OBSERVATION_LIMIT)
            .ok_or_else(|| super::contract("deployment startup deadline extent"))?;
        checkpoint()?;
        verify_loaded()?;
        // SAFETY: the caller is the actual controlled composition root and asserts
        // the receipt provenance contract above; no generic embedding is admitted.
        let startup = unsafe { math.observe_deployment_runtime(host.artifact, host.receipt.clone(), host.target, deadline, stop) }
            .await.map_err(typed)?;
        let producer = if host.receipt.is_some() {
            startup.producer.clone().map(Into::into)
        } else {
            // SAFETY: the same host supplies its own code anchor and complete
            // bounded configuration under the controlled reconstruction contract.
            match unsafe { math.observe_local_runtime(host.target, host.anchor, host.configuration, deadline, stop) }.await {
                Ok(producer) => Some(producer),
                Err(PortableError::Qualification(_)) => None,
                Err(error) => return Err(typed(error)),
            }
        };
        verify_loaded()?;
        checkpoint()?;
        let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(state, database)?;
        let store = CanonicalStore::connect(&options).await?;
        checkpoint()?;
        store.open().await?;
        checkpoint()?;
        Ok(Self::new(store, OuterAttestation { source: startup.outer.source, build: startup.outer.build }, producer))
    }
}
