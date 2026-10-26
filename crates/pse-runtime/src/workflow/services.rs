// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit effect capabilities supplied by the workflow composition root.
use super::{
    CanonicalDeployment, Durability, OuterAttestation, Runtime, RuntimeActivity, RuntimeCaller,
    WorkflowError,
};
use crate::math::MathService;
use datafusion::execution::memory_pool::MemoryPool;
use pse_engine::session::{BoundModel, EngineFactory};
use std::{future::Future, sync::Arc};

/// Logical caller admission without a runtime, executor, store or scientific input.
#[derive(Clone, Debug)]
pub struct ActivityAdmission {
    caller: Arc<RuntimeCaller>,
}
impl ActivityAdmission {
    pub(crate) fn admit(&self) -> Result<RuntimeActivity, WorkflowError> {
        self.caller.admit()
    }
    pub(crate) fn context(
        &self,
        model: &BoundModel,
    ) -> Arc<pse_relations::validate::ValidationContext> {
        if let Ok(activity) = super::RUNTIME_CONTINUATION.try_with(Clone::clone)
            && Arc::ptr_eq(&activity.0.caller, &self.caller)
        {
            return activity.0.validation.clone();
        }
        Arc::new(model.admission_context())
    }
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.caller, &other.caller)
    }
    /// Hold the admitted caller through its actual asynchronous work.
    pub async fn scope<T>(
        &self,
        work: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        self.admit()?.scope(work).await
    }
}

/// Compiler, allocation and validation capabilities; no canonical publication or execution policy.
#[derive(Clone, Debug)]
pub struct PreparationServices {
    pub(super) physical_cache: Arc<super::physical_cache::PhysicalCache>,
    pub(crate) spill: Arc<datafusion_execution::disk_manager::DiskManager>,
    pub(crate) math: Arc<MathService>,
    pub(crate) model: BoundModel,
    pub(crate) admission: ActivityAdmission,
}
impl PreparationServices {
    /// Existing mathematical preparation and accounted allocation owner.
    pub fn math(&self) -> &Arc<MathService> {
        &self.math
    }
    /// The schema authority supplied by the bound model.
    pub fn registry(&self) -> &Arc<pse_schema::Registry> {
        self.model.registry()
    }
    /// Existing compiler session factory; no additional executor is constructed.
    pub fn sessions(&self) -> &Arc<EngineFactory> {
        self.model.factory()
    }
    /// Existing immutable native binding used by compiler lineage admission.
    pub fn bound_model(&self) -> &BoundModel {
        &self.model
    }
    /// The shared allocation owner supplied to this preparation scope.
    pub fn pool(&self) -> Arc<dyn MemoryPool> {
        self.model.factory().pool().clone()
    }
    /// Complete existing validation authority.
    pub fn validation_context(&self) -> Arc<pse_relations::validate::ValidationContext> {
        self.model.validation_context()
    }
    /// Current operation's original admission context.
    pub fn admission_context(&self) -> Arc<pse_relations::validate::ValidationContext> {
        self.admission.context(&self.model)
    }
    pub(crate) fn admit_activity(&self) -> Result<RuntimeActivity, WorkflowError> {
        self.admission.admit()
    }
    /// Run explicit preparation under the original logical caller's lifetime.
    pub async fn scope_activity<T>(
        &self,
        work: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        self.admission.scope(work).await
    }
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.math, &other.math)
            && Arc::ptr_eq(self.sessions(), other.sessions())
            && Arc::ptr_eq(self.registry(), other.registry())
            && self.admission.same_owner(&other.admission)
    }
}

/// Original-source loading and commit capabilities, separate from immutable source values.
#[derive(Clone, Debug)]
pub struct SourceServices {
    pub(crate) read_operations: Option<Arc<super::Operations>>,
    pub(crate) preparation: PreparationServices,
    pub(crate) canonical: Option<CanonicalDeployment>,
    pub(crate) publication: Option<PublicationServices>,
}
impl SourceServices {
    pub(crate) fn operations(&self) -> Result<&super::Operations, WorkflowError> {
        self.read_operations
            .as_deref()
            .ok_or(WorkflowError::RecordingUnavailable)
    }
    pub(crate) fn admit_activity(&self) -> Result<RuntimeActivity, WorkflowError> {
        self.preparation.admit_activity()
    }
    /// Automatically selected source publication authority; local edits retain no publication effect.
    pub fn publication(&self) -> Option<&PublicationServices> {
        self.publication.as_ref()
    }
    /// Pure/compiler capabilities used while admitting original source input.
    pub fn preparation(&self) -> &PreparationServices {
        &self.preparation
    }
    /// Explicit canonical source boundary; ephemeral source operations can refuse its absence.
    pub fn canonical(&self) -> Result<&CanonicalDeployment, WorkflowError> {
        self.canonical
            .as_ref()
            .ok_or(WorkflowError::RecordingUnavailable)
    }
    /// Exact canonical original-source store.
    pub fn canonical_store(
        &self,
    ) -> Result<&pse_operations::canonical::CanonicalStore, WorkflowError> {
        Ok(self.canonical()?.store())
    }
}

/// Canonical effect capability; publication does not grant scientific permission.
#[derive(Clone, Debug)]
pub struct PublicationServices {
    pub(crate) canonical: CanonicalDeployment,
    pub(crate) pool: Arc<dyn MemoryPool>,
    pub(crate) registry: Arc<pse_schema::Registry>,
    pub(crate) admission: ActivityAdmission,
}
impl PublicationServices {
    /// Exact selected publication boundary.
    pub fn canonical(&self) -> &CanonicalDeployment {
        &self.canonical
    }
    /// Existing canonical connection owner.
    pub fn store(&self) -> &pse_operations::canonical::CanonicalStore {
        self.canonical.store()
    }
    /// Shared reservation owner for publication copies.
    pub fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.pool
    }
    /// Exact source/result schema authority.
    pub fn registry(&self) -> &Arc<pse_schema::Registry> {
        &self.registry
    }
    pub(crate) fn admit_activity(&self) -> Result<RuntimeActivity, WorkflowError> {
        self.admission.admit()
    }
}

/// Run-supervisor capabilities, supplied at execution rather than retained by scientific values.
#[derive(Clone, Debug)]
pub struct ExecutionServices {
    pub(crate) read_operations: Option<Arc<super::Operations>>,
    pub(crate) preparation: PreparationServices,
    pub(crate) durability: Durability,
    pub(crate) publication: Option<PublicationServices>,
    pub(crate) observations: Arc<crate::observation::Diagnostics>,
    pub(crate) attestation: Option<OuterAttestation>,
}
impl ExecutionServices {
    /// Explicit compiler/allocation and caller admission services.
    pub fn preparation(&self) -> &PreparationServices {
        &self.preparation
    }
    /// Actual selected recording policy, independent of scientific permission.
    pub fn durability(&self) -> &Durability {
        &self.durability
    }
    /// Canonical publication capability when configured by the composition root.
    pub fn publication(&self) -> Option<&PublicationServices> {
        self.publication.as_ref()
    }
    /// Original passive diagnostic observation owner.
    pub fn observations(&self) -> &Arc<crate::observation::Diagnostics> {
        &self.observations
    }
    /// Actual observed outer provenance, separate from mathematical producer admission.
    pub fn outer_attestation(&self) -> Option<OuterAttestation> {
        self.attestation
    }
    /// Original-source effects explicitly requested during an execution.
    pub fn source_services(&self) -> SourceServices {
        SourceServices {
            read_operations: self.read_operations.clone(),
            preparation: self.preparation.clone(),
            canonical: self
                .publication
                .as_ref()
                .map(|publication| publication.canonical.clone()),
            publication: matches!(&self.durability, Durability::Durable(_))
                .then(|| self.publication.clone())
                .flatten(),
        }
    }
}
impl Runtime {
    /// Supply existing compiler/allocation services without retaining the composition root.
    pub fn preparation_services(&self) -> PreparationServices {
        PreparationServices {
            physical_cache: self.physical_cache.clone(),
            spill: self.shared.runtime_env().disk_manager.clone(),
            math: self.shared.math().clone(),
            model: self.model.clone(),
            admission: ActivityAdmission {
                caller: self.caller.clone(),
            },
        }
    }
    /// Supply original-source effects independently of scientific values.
    pub fn source_services(&self) -> SourceServices {
        SourceServices {
            read_operations: self
                .recording_operations
                .as_ref()
                .map(|owner| Arc::new(owner.as_ref().clone())),
            preparation: self.preparation_services(),
            canonical: self.canonical.clone(),
            publication: matches!(&self.durability, Durability::Durable(_))
                .then(|| self.publication_services().ok())
                .flatten(),
        }
    }
    /// Supply the explicitly configured publication boundary.
    pub fn publication_services(&self) -> Result<PublicationServices, WorkflowError> {
        Ok(PublicationServices {
            canonical: self.canonical()?.clone(),
            pool: self.shared.pool(),
            registry: self.registry().clone(),
            admission: ActivityAdmission {
                caller: self.caller.clone(),
            },
        })
    }
    /// Supply one run supervisor's capabilities under the selected durability policy.
    pub fn execution_services(&self) -> ExecutionServices {
        ExecutionServices {
            read_operations: self
                .recording_operations
                .as_ref()
                .map(|owner| Arc::new(owner.as_ref().clone())),
            preparation: self.preparation_services(),
            durability: self.durability.clone(),
            publication: self
                .canonical
                .as_ref()
                .map(|canonical| PublicationServices {
                    canonical: canonical.clone(),
                    pool: self.shared.pool(),
                    registry: self.registry().clone(),
                    admission: ActivityAdmission {
                        caller: self.caller.clone(),
                    },
                }),
            observations: self.observations().clone(),
            attestation: self.outer_attestation(),
        }
    }
}
