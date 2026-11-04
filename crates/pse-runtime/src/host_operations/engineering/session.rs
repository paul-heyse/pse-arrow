// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original operation custody, independent of RPC waiter and connection lifetimes.

use crate::host_operations::project::handler::{admit_project_documents, admit_project_physical};
use crate::{
    CancelSource, SharedRuntime,
    workflow::{
        ModelingPackage, OperationSource, PreparationSettings, PreparedOperation,
        ProjectDraftDocuments, RunHandle, Runtime, RuntimeOperationRequest,
    },
};
use futures_util::FutureExt as _;
use pse_ids::{ContentHash, EncodingHasher};
use pse_model::boundary_projection::ValidateProjection as _;
use pse_model::engineering::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::Notify;
mod contract;
mod edit;
mod inspection;
mod source_read;
mod study;
pub use inspection::EngineeringModelingOutput;
pub use source_read::EngineeringPhysicalOutput;
pub use study::EngineeringStudyOutput;
#[cfg(all(test, feature = "canonical-tests", feature = "solver-kinsol"))]
mod canonical_study_tests;

type OwnerError = Box<dyn std::error::Error + Send + Sync>;

/// Boundary/lifetime failures never grant scientific permission or imply native settlement.
#[derive(Debug, thiserror::Error)]
pub enum EngineeringServiceError {
    /// Missing original identity, stale parent or changed request under one original identity.
    #[error("engineering request refused: {0}")]
    Request(String),
    /// The prepared peer has another compiled boundary interpretation.
    #[error("engineering interpretation refused: {0}")]
    Interpretation(String),
    /// A finite retained-operation or response allowance refused work before acknowledgement.
    #[error("engineering resource allowance: {0}")]
    Resource(String),
    /// The original native owner failure is available without replacing its type.
    #[error("engineering native operation: {0}")]
    Owner(#[source] OwnerError),
    /// Repeated waits retain the original failure allocation and its typed source.
    #[error("{0}")]
    Original(#[source] Arc<EngineeringServiceError>),
    /// The service could not establish its own original custody or join.
    #[error("engineering service: {0}")]
    Infrastructure(String),
    /// Independent shutdown failures remain individually available.
    #[error("engineering shutdown failures: {0:?}")]
    Shutdown(Vec<EngineeringServiceError>),
}

fn original<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a dyn pse_diagnostics::TypedDiagnostic> {
    if let Some(error) = error.downcast_ref::<EngineeringServiceError>() {
        return Some(error);
    }
    if let Some(error) =
        error.downcast_ref::<crate::host_operations::project::handler::ProjectOperationError>()
    {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<crate::workflow::WorkflowError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<Arc<crate::workflow::WorkflowError>>() {
        return Some(error.as_ref());
    }
    if let Some(error) = error.downcast_ref::<pse_engine::EngineError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<crate::RuntimeError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_operations::canonical::CanonicalError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<crate::math::MathRuntimeError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_model::boundary_projection::ValidationError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_model::boundary_projection::AdapterError>() {
        return Some(error);
    }
    if let Some(error) =
        error.downcast_ref::<crate::preparation_projection::PreparationDiscoveryProjectionError>()
    {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<crate::projection_support::BoundaryProjectionError>()
    {
        return Some(error);
    }
    None
}
pse_diagnostics::impl_diagnostic! {
    EngineeringServiceError,
    code(this) { match this { Self::Request(_) => Some(pse_diagnostics::DiagnosticCode::WorkflowInput), Self::Interpretation(_) => Some(pse_diagnostics::DiagnosticCode::SchemaVersionMismatch), Self::Resource(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit), Self::Owner(source) if original(source.as_ref()).is_some() => None, Self::Original(_) => None, _ => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) } },
    forward(this) { match this { Self::Owner(source) => original(source.as_ref()), Self::Original(source) => Some(source.as_ref()), _ => None } },
    help(_this) { None },
    related(this) { match this { Self::Shutdown(failures) => Some(Box::new(failures.iter().map(|failure| -> &dyn pse_diagnostics::TypedDiagnostic { failure }))), _ => None } },
    source(_this) { None }
}

#[derive(Clone)]
enum Outcome {
    Pending,
    Source {
        package: ModelingPackage,
        selection: EngineeringSourceSelection,
    },
    Prepared {
        prepared: PreparedOperation,
        selection: EngineeringPreparedSelection,
    },
    Run(RunHandle),
    PreparedStudy(Arc<study::PreparedStudy>),
    Study(crate::workflow::StudyHandle),
    StudyCohort(Arc<crate::workflow::FrozenStudyCohort>),
    Released,
    Retained,
    Failed(Arc<EngineeringServiceError>),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Source,
    Preparation,
    Run,
    StudyPreparation,
    Study,
    StudyCohort,
}
struct Operation {
    fingerprint: ContentHash,
    kind: Kind,
    cancel: CancelSource,
    outcome: Mutex<Outcome>,
    issued: Mutex<Option<EngineeringRunKey>>,
    issued_study: Mutex<Option<crate::workflow::IssuedStudyIdentity>>,
    changed: Notify,
    _reservation: datafusion::execution::memory_pool::MemoryReservation,
}

impl Operation {
    fn snapshot(&self) -> Result<Outcome, EngineeringServiceError> {
        Ok(self
            .outcome
            .lock()
            .map_err(|_| infrastructure("operation lock poisoned"))?
            .clone())
    }
    fn publish(&self, outcome: Outcome) -> Result<(), EngineeringServiceError> {
        let mut original = self
            .outcome
            .lock()
            .map_err(|_| infrastructure("operation lock poisoned"))?;
        // Local capture release closes admission permanently, including when the
        // original native freeze finishes after the release acknowledgement.
        if !matches!(*original, Outcome::Released) {
            *original = outcome;
        }
        drop(original);
        self.changed.notify_waiters();
        Ok(())
    }
    async fn completed(&self) -> Result<Outcome, EngineeringServiceError> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            match self.snapshot()? {
                Outcome::Pending => notified.await,
                outcome => return Ok(outcome),
            }
        }
    }

    async fn dependency(&self, cancel: &CancelSource) -> Result<Outcome, EngineeringServiceError> {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(owner(crate::RuntimeError::Cancelled)),
            outcome = self.completed() => {
                cancel.checkpoint().map_err(owner)?;
                outcome
            }
        }
    }
}

/// One explicitly prepared runtime and its retained original source/preparation/run owners.
///
/// Dropping an RPC waiter does not drop these owners. Call [`Self::shutdown`] before
/// releasing the service; it cancels frontend-owned work and awaits actual joins/drain.
/// Durable studies retain their separate original canonical and scientific-worker owners.
pub struct EngineeringService {
    runtime: Runtime,
    shared: Arc<SharedRuntime>,
    operations: Mutex<BTreeMap<String, Arc<Operation>>>,
    tasks: Mutex<tokio::task::JoinSet<()>>,
    closing: AtomicBool,
    shutdown_lock: tokio::sync::Mutex<()>,
    maximum_operations: usize,
    contract: Mutex<Option<Arc<contract::PreparedContract>>>,
}

impl std::fmt::Debug for EngineeringService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EngineeringService")
            .field("maximum_operations", &self.maximum_operations)
            .field("closing", &self.closing.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

/// Selected response bytes retain their accounted allocation through transport consumption.
#[derive(Debug)]
pub struct EngineeringResultOutput {
    /// Complete selected native result page.
    pub page: EngineeringResultPage,
    pub(super) reservation: Arc<datafusion::execution::memory_pool::MemoryReservation>,
    pub(super) maximum_bytes: usize,
}

impl EngineeringService {
    pub(super) fn ingress_pool(&self) -> Arc<dyn datafusion::execution::memory_pool::MemoryPool> {
        self.shared.pool()
    }

    /// Attach an explicitly prepared native runtime; this never starts a configured database.
    /// # Errors
    /// A zero retention extent refuses before any operation can be admitted.
    pub fn new(
        runtime: Runtime,
        shared: Arc<SharedRuntime>,
        maximum_operations: usize,
    ) -> Result<Arc<Self>, EngineeringServiceError> {
        if maximum_operations == 0 {
            return Err(resource("retained operation extent must be positive"));
        }
        if !Arc::ptr_eq(&runtime.shared, &shared) {
            return Err(request(
                "runtime and engineering service must retain the same original shared budget",
            ));
        }
        Ok(Arc::new(Self {
            runtime,
            shared,
            operations: Mutex::new(BTreeMap::new()),
            tasks: Mutex::new(tokio::task::JoinSet::new()),
            closing: AtomicBool::new(false),
            shutdown_lock: tokio::sync::Mutex::new(()),
            maximum_operations,
            contract: Mutex::new(None),
        }))
    }

    pub(super) fn inspection_source(
        &self,
        operation: String,
    ) -> Result<(ModelingPackage, crate::workflow::SourceServices), EngineeringServiceError> {
        EngineeringOperationKey {
            operation: operation.clone(),
        }
        .validate_projection()
        .map_err(owner)?;
        let Outcome::Source { package, .. } =
            self.operation(&operation, Kind::Source)?.snapshot()?
        else {
            return Err(request("original source admission is not ready"));
        };
        Ok((package, self.runtime.source_services()))
    }

    fn operation(&self, key: &str, kind: Kind) -> Result<Arc<Operation>, EngineeringServiceError> {
        let operations = self
            .operations
            .lock()
            .map_err(|_| infrastructure("operation inventory poisoned"))?;
        let operation = operations
            .get(key)
            .filter(|operation| operation.kind == kind)
            .ok_or_else(|| {
                request("original operation is unavailable in this service incarnation")
            })?;
        Ok(operation.clone())
    }

    fn issue<T: serde::Serialize>(
        &self,
        key: &str,
        kind: Kind,
        input: &T,
    ) -> Result<(Arc<Operation>, bool), EngineeringServiceError> {
        EngineeringOperationKey {
            operation: key.to_owned(),
        }
        .validate_projection()
        .map_err(owner)?;
        let fingerprint = fingerprint(input)?;
        let mut operations = self
            .operations
            .lock()
            .map_err(|_| infrastructure("operation inventory poisoned"))?;
        if self.closing.load(Ordering::Acquire) {
            return Err(request("service is closing"));
        }
        if let Some(original) = operations.get(key) {
            if original.kind != kind || original.fingerprint != fingerprint {
                return Err(request(
                    "original operation identity is already bound to another request",
                ));
            }
            return Ok((original.clone(), false));
        }
        if operations.len() >= self.maximum_operations {
            return Err(resource("retained operation inventory is full"));
        }
        let reservation = datafusion::execution::memory_pool::MemoryConsumer::new(
            "engineering:retained-operation",
        )
        .register(&self.shared.pool());
        reservation
            .try_grow(size_of::<Operation>() + key.len() + 4096)
            .map_err(owner)?;
        let operation = Arc::new(Operation {
            fingerprint,
            kind,
            cancel: CancelSource::new(),
            outcome: Mutex::new(Outcome::Pending),
            issued: Mutex::new(None),
            issued_study: Mutex::new(None),
            changed: Notify::new(),
            _reservation: reservation,
        });
        operations.insert(key.into(), operation.clone());
        Ok((operation, true))
    }

    fn spawn(
        self: &Arc<Self>,
        operation: Arc<Operation>,
        work: impl Future<Output = Result<Outcome, EngineeringServiceError>> + Send + 'static,
    ) -> Result<(), EngineeringServiceError> {
        let outcome = self.spawn_owned(operation.clone(), work);
        if let Err(error) = outcome {
            let error = Arc::new(error);
            operation.publish(Outcome::Failed(error.clone()))?;
            return Err(EngineeringServiceError::Original(error));
        }
        Ok(())
    }

    fn spawn_owned<F>(
        self: &Arc<Self>,
        operation: Arc<Operation>,
        work: F,
    ) -> Result<(), EngineeringServiceError>
    where
        F: Future<Output = Result<Outcome, EngineeringServiceError>> + Send + 'static,
    {
        operation
            ._reservation
            .try_grow(
                size_of::<F>()
                    .checked_add(1024)
                    .ok_or_else(|| resource("operation future extent overflow"))?,
            )
            .map_err(owner)?;
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|_| infrastructure("native task inventory poisoned"))?;
        // Reap completed original tasks, retaining operation outcomes in the inventory.
        while let Some(joined) = tasks.try_join_next() {
            if let Err(error) = joined {
                return Err(infrastructure(&format!(
                    "original operation task failed: {error}"
                )));
            }
        }
        if self.closing.load(Ordering::Acquire) {
            return Err(request("service is closing"));
        }
        tasks.spawn(async move {
            let outcome = match std::panic::AssertUnwindSafe(work).catch_unwind().await {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(error)) => Outcome::Failed(Arc::new(error)),
                Err(_) => {
                    Outcome::Failed(Arc::new(infrastructure("original operation task panicked")))
                }
            };
            let _published = operation.publish(outcome);
        });
        Ok(())
    }

    /// Retain the original request before starting native source/physical admission.
    /// # Errors
    /// A changed same-identity request, closed service or exhausted retention refuses.
    pub async fn admit_source(
        self: &Arc<Self>,
        input: EngineeringAdmitSource,
    ) -> Result<EngineeringSourceOutcome, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let mut extent = 4096usize;
        for package in input.physical.iter().chain(&input.modeling) {
            for (path, body) in &package.files {
                extent = extent
                    .checked_add(path.capacity())
                    .and_then(|bytes| bytes.checked_add(body.capacity()))
                    .and_then(|bytes| bytes.checked_add(256))
                    .ok_or_else(|| resource("source request extent overflow"))?;
            }
        }
        let input_reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("engineering:source-request")
                .register(&self.shared.pool());
        input_reservation.try_grow(extent).map_err(owner)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::Source, &input)?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let _input_reservation = input_reservation;
                let physical = ProjectDraftDocuments {
                    version: Default::default(),
                    packages: input
                        .physical
                        .into_iter()
                        .map(|package| package.files)
                        .collect(),
                };
                let modeling = ProjectDraftDocuments {
                    version: Default::default(),
                    packages: input
                        .modeling
                        .into_iter()
                        .map(|package| package.files)
                        .collect(),
                };
                let physical = admit_project_physical(
                    &service.runtime,
                    &service.shared,
                    &physical,
                    &original.cancel,
                )
                .await
                .map_err(owner)?;
                let package = admit_project_documents(
                    &service.runtime,
                    &service.shared,
                    &modeling,
                    physical,
                    &original.cancel,
                )
                .await
                .map_err(owner)?;
                let source = OperationSource::of(&package);
                let selection = EngineeringSourceSelection {
                    operation: input.operation,
                    local_revision: package.revision_id().into(),
                    source_revision: source.revision.as_id(),
                    physical_context: source.physical_context,
                };
                Ok(Outcome::Source { package, selection })
            })?;
        }
        source_outcome(operation.snapshot()?)
    }

    /// Observe the original source-admission outcome without replaying or constructing science.
    /// # Errors
    /// An absent or differently typed original identity refuses.
    pub fn source_status(
        &self,
        key: &EngineeringOperationKey,
    ) -> Result<EngineeringSourceOutcome, EngineeringServiceError> {
        source_outcome(self.operation(&key.operation, Kind::Source)?.snapshot()?)
    }

    /// Read a selected declaration page from the real source producer, without preparing science.
    /// # Errors
    /// Invalid pages, unavailable source and original producer/resource refusals propagate.
    pub async fn inspect_source(
        &self,
        input: EngineeringInspectSource,
        cancel: &CancelSource,
    ) -> Result<pse_columnar::Leased<EngineeringDeclarationPage>, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let Outcome::Source { package, selection } =
            self.operation(&input.operation, Kind::Source)?.snapshot()?
        else {
            return Err(request("original source admission is not ready"));
        };
        let page = package
            .declaration_headers_page(
                &self.runtime.preparation_services(),
                &crate::workflow::ModelingDeclarationRequest {
                    offset: usize::try_from(input.offset).map_err(owner)?,
                    limit: usize::try_from(input.limit).map_err(owner)?,
                },
                cancel,
            )
            .await
            .map_err(owner)?;
        let next = input
            .offset
            .checked_add(u64::from(input.limit))
            .ok_or_else(|| request("declaration continuation overflow"))?;
        let total = u64::try_from(page.0).map_err(owner)?;
        let response = EngineeringDeclarationPage {
            source: selection,
            total,
            offset: input.offset,
            next_offset: (next < total).then_some(next),
            declarations: page.1.clone(),
        };
        Ok(pse_columnar::Leased::new(Arc::new(response), page.owner()))
    }

    /// Prepare one actual authored case under its retained exact source/physical context.
    /// # Errors
    /// Changed identity, stale parent, native contextual refusal and resource failures propagate.
    pub async fn prepare_case(
        self: &Arc<Self>,
        input: EngineeringPrepareCase,
    ) -> Result<EngineeringPreparedSelection, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let source = self.operation(&input.source_operation, Kind::Source)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::Preparation, &input)?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let source = source.dependency(&original.cancel).await?;
                let Outcome::Source { package, selection } = source else {
                    return Err(request(
                        "source admission did not produce an admitted package",
                    ));
                };
                if package.revision_id() != input.expected_local_revision {
                    return Err(request("exact source parent is stale"));
                }
                let prepared = package
                    .prepare_operation(
                        &service.runtime.preparation_services(),
                        &OperationSource::of(&package),
                        &PreparationSettings::default(),
                        RuntimeOperationRequest::DeclaredCase {
                            case: pse_model::generated::identities::DeclarationId::from_id(
                                input.case,
                            ),
                            settings: Box::new(crate::math::settings::SolveSettings {
                                intent: input.intent,
                                backend: input.backend,
                                ..Default::default()
                            }),
                        },
                        None,
                        &original.cancel,
                    )
                    .await
                    .map_err(owner)?;
                let selection = EngineeringPreparedSelection {
                    operation: input.operation,
                    source: selection,
                    preparation: prepared.identity().map_err(owner)?,
                };
                Ok(Outcome::Prepared {
                    prepared,
                    selection,
                })
            })?;
        }
        match operation.completed().await? {
            Outcome::Prepared { selection, .. } => Ok(selection),
            Outcome::Failed(error) => Err(EngineeringServiceError::Original(error)),
            _ => Err(infrastructure(
                "prepared operation published an incompatible outcome",
            )),
        }
    }

    /// Start once under the original request identity and retain the native cancellation lease.
    /// # Errors
    /// Conflicting repetition, absent preparation and original native start failures refuse.
    pub async fn submit_run(
        self: &Arc<Self>,
        input: EngineeringSubmitRun,
    ) -> Result<EngineeringRunStatus, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let key = EngineeringRunKey {
            operation: input.operation.clone(),
            run: input.run,
            attempt: input.attempt,
        };
        if matches!(
            self.runtime.durability(),
            crate::workflow::Durability::Durable(_)
        ) && let Some((_header_owner, _record, _attempt, stored)) =
            self.find_original_record(&key).await?
        {
            if stored.submission != fingerprint(&input)? {
                return Err(request(
                    "original canonical submission belongs to another request envelope",
                ));
            }
            return self.run_status(&key).await;
        }
        let preparation = self.operation(&input.preparation_operation, Kind::Preparation)?;
        let (operation, fresh) = self.issue_run(&input, &key)?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let prepared = preparation.dependency(&original.cancel).await?;
                let Outcome::Prepared { prepared, .. } = prepared else {
                    return Err(request(
                        "preparation did not produce an executable native operation",
                    ));
                };
                if prepared.identity().map_err(owner)? != input.expected_preparation {
                    return Err(request("exact immutable preparation identity differs"));
                }
                let identity = crate::workflow::IssuedRunIdentity {
                    run: pse_model::generated::identities::RunId::from_id(input.run),
                    attempt: pse_model::generated::identities::AttemptId::from_id(input.attempt),
                    operation: input.operation,
                    submission: original.fingerprint,
                };
                Ok(
                    match prepared
                        .start_issued(
                            &service.runtime.execution_services(),
                            identity,
                            &original.cancel,
                        )
                        .await
                        .map_err(owner)?
                    {
                        crate::workflow::IssuedRunStart::Started(handle) => Outcome::Run(handle),
                        crate::workflow::IssuedRunStart::Retained { .. } => Outcome::Retained,
                    },
                )
            })?;
        }
        operation.completed().await?;
        self.run_status(&key).await
    }

    fn issue_run(
        &self,
        input: &EngineeringSubmitRun,
        key: &EngineeringRunKey,
    ) -> Result<(Arc<Operation>, bool), EngineeringServiceError> {
        let (operation, fresh) = self.issue(&input.operation, Kind::Run, input)?;
        {
            let mut original = operation
                .issued
                .lock()
                .map_err(|_| infrastructure("issued run identity lock poisoned"))?;
            if let Some(original) = original.as_ref() {
                if original != key {
                    return Err(request(
                        "original operation has another issued run or attempt",
                    ));
                }
            } else {
                *original = Some(key.clone());
            }
        }
        Ok((operation, fresh))
    }

    /// Read the actual original native result; an unjoined result remains unsettled.
    /// # Errors
    /// An absent/different original operation or corrupted custody inventory refuses.
    pub async fn run_status(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<EngineeringRunStatus, EngineeringServiceError> {
        key.validate_projection().map_err(owner)?;
        let operation = match self.operation(&key.operation, Kind::Run) {
            Ok(operation) => operation,
            Err(EngineeringServiceError::Request(_)) => return self.recorded_status(key).await,
            Err(error) => return Err(error),
        };
        check_issued(&operation, key)?;
        let cancellation_requested = operation.cancel.token().is_cancelled();
        Ok(match operation.snapshot()? {
            Outcome::Pending => EngineeringRunStatus {
                operation: key.operation.clone(),
                run: None,
                settled: false,
                usable: None,
                cancellation_requested,
                failure: None,
                canonical_run: None,
                canonical_attempt: None,
                progress: None,
            },
            Outcome::Run(handle) => {
                check_run(&handle, key)?;
                let result = handle.result();
                EngineeringRunStatus {
                    operation: key.operation.clone(),
                    run: Some(handle.run_id().as_id()),
                    settled: result.is_some(),
                    usable: result
                        .as_ref()
                        .and_then(|result| result.completion().ok())
                        .and_then(crate::workflow::Completion::governing_permission),
                    cancellation_requested,
                    failure: result
                        .as_ref()
                        .and_then(|result| result.report().err())
                        .map(|error| failure(error)),
                    canonical_run: handle.canonical_run_key().map(str::to_owned),
                    canonical_attempt: handle.canonical_attempt_key().map(str::to_owned),
                    progress: Some(run_progress(&handle)),
                }
            }
            Outcome::Retained => return self.recorded_status(key).await,
            Outcome::Failed(error) => EngineeringRunStatus {
                operation: key.operation.clone(),
                run: None,
                settled: false,
                usable: None,
                cancellation_requested,
                failure: Some(failure(error.as_ref())),
                canonical_run: None,
                canonical_attempt: None,
                progress: None,
            },
            _ => {
                return Err(infrastructure(
                    "run operation published an incompatible outcome",
                ));
            }
        })
    }

    /// Await the original completion receiver, retaining bounded sampling for passive
    /// checkpoint owners which deliberately have no asynchronous subscriber API.
    pub(super) async fn observation_changed(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<(), EngineeringServiceError> {
        let pause = tokio::time::sleep(std::time::Duration::from_millis(100));
        let operation = match self.operation(&key.operation, Kind::Run) {
            Ok(operation) => operation,
            Err(EngineeringServiceError::Request(_)) => {
                pause.await;
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        match operation.snapshot()? {
            Outcome::Run(handle) => {
                check_run(&handle, key)?;
                tokio::select! { result = handle.wait() => { result.map_err(owner)?; }, _ = pause => {} }
            }
            Outcome::Pending => {
                tokio::select! { _ = operation.changed.notified() => {}, _ = pause => {} }
            }
            _ => pause.await,
        }
        Ok(())
    }

    /// Request cooperative cancellation without claiming that native settlement has occurred.
    /// # Errors
    /// An absent/different original operation refuses rather than cancelling another identity.
    pub async fn cancel_run(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<EngineeringRunStatus, EngineeringServiceError> {
        key.validate_projection().map_err(owner)?;
        match self.operation(&key.operation, Kind::Run) {
            Ok(operation) => {
                check_issued(&operation, key)?;
                match operation.snapshot()? {
                    Outcome::Run(handle) => {
                        check_run(&handle, key)?;
                        handle.cancel();
                    }
                    Outcome::Retained => self.cancel_recorded(key).await?,
                    _ => {}
                }
                operation.cancel.cancel();
            }
            Err(EngineeringServiceError::Request(_)) => self.cancel_recorded(key).await?,
            Err(error) => return Err(error),
        }
        self.run_status(key).await
    }
    async fn cancel_recorded(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<(), EngineeringServiceError> {
        let (reservation, original, _) = self.original_record(key).await?;
        pse_operations::canonical::response_ownership_scope(
            reservation,
            self.runtime
                .canonical_store()
                .map_err(owner)?
                .cancel_run(&original.key, &format!("issued-cancel:{}", key.operation)),
        )
        .await
        .map_err(owner)?;
        Ok(())
    }

    async fn original_record(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<
        (
            Arc<datafusion::execution::memory_pool::MemoryReservation>,
            pse_operations::canonical_execution::CanonicalRun,
            String,
        ),
        EngineeringServiceError,
    > {
        let Some((reservation, original, attempt, _)) = self.find_original_record(key).await?
        else {
            return Err(request(
                "original canonical registration is unavailable; absence does not establish no effect",
            ));
        };
        Ok((reservation, original, attempt))
    }

    async fn find_original_record(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<
        Option<(
            Arc<datafusion::execution::memory_pool::MemoryReservation>,
            pse_operations::canonical_execution::CanonicalRun,
            String,
            crate::workflow::IssuedRunIdentity,
        )>,
        EngineeringServiceError,
    > {
        key.validate_projection().map_err(owner)?;
        let (run, attempt) = crate::workflow::IssuedRunIdentity::canonical_keys_for(
            pse_model::generated::identities::RunId::from_id(key.run),
            &key.operation,
        );
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(
                "engineering:recorded-original-header",
            )
            .register(&self.shared.pool()),
        );
        reservation
            .try_grow(pse_operations::canonical::MetadataResponse::Header.allocation_bytes())
            .map_err(owner)?;
        let Some(original) = pse_operations::canonical::response_ownership_scope(
            reservation.clone(),
            self.runtime
                .canonical_store()
                .map_err(owner)?
                .canonical_run(&run),
        )
        .await
        .map_err(owner)?
        else {
            return Ok(None);
        };
        let stored =
            crate::workflow::IssuedRunIdentity::from_canonical_request(original.request.as_slice())
                .map_err(|error| {
                    EngineeringServiceError::Interpretation(format!(
                        "original canonical execution has another issuance interpretation: {error}"
                    ))
                })?;
        if stored.run.as_id() != key.run
            || stored.attempt.as_id() != key.attempt
            || stored.operation != key.operation
        {
            return Err(request(
                "canonical original run belongs to another issued request",
            ));
        }
        Ok(Some((reservation, original, attempt, stored)))
    }

    async fn recorded_status(
        &self,
        key: &EngineeringRunKey,
    ) -> Result<EngineeringRunStatus, EngineeringServiceError> {
        let (reservation, original, attempt) = self.original_record(key).await?;
        let record = pse_operations::canonical::response_ownership_scope(
            reservation,
            self.runtime
                .canonical_store()
                .map_err(owner)?
                .canonical_attempt(&attempt),
        )
        .await
        .map_err(owner)?;
        let settled = record.as_ref().is_some_and(|record| {
            matches!(
                record.phase,
                pse_model::generated::enums::CanonicalAttemptPhase::Terminal
                    | pse_model::generated::enums::CanonicalAttemptPhase::TerminalCompacted
            )
        });
        let completion = if settled {
            Some(
                self.runtime
                    .retained_data_services()
                    .stored_completion(&original.key, &attempt)
                    .await
                    .map_err(owner)?,
            )
        } else {
            None
        };
        let usable = completion
            .as_ref()
            .and_then(|completion| completion.completion.as_ref())
            .and_then(crate::workflow::Completion::governing_permission);
        let failure =
            completion
                .as_ref()
                .and_then(|completion| match &completion.termination.cause {
                    crate::workflow::TerminationCause::Error { diagnostic }
                    | crate::workflow::TerminationCause::Infrastructure { diagnostic } => {
                        Some(failure(diagnostic))
                    }
                    _ => None,
                });
        Ok(EngineeringRunStatus {
            operation: key.operation.clone(),
            run: Some(key.run),
            settled,
            usable,
            cancellation_requested: original.cancelled,
            failure,
            canonical_run: Some(original.key),
            canonical_attempt: Some(attempt),
            progress: None,
        })
    }

    /// Materialize only the selected original row range, with an atomic bounded IPC answer.
    /// # Errors
    /// Unsettled/unavailable products, invalid ranges, producer failure and extent exhaustion refuse.
    pub async fn read_result(
        &self,
        input: EngineeringReadResult,
        cancel: &CancelSource,
    ) -> Result<EngineeringResultOutput, EngineeringServiceError> {
        use datafusion::arrow::ipc::writer::StreamWriter;
        input.validate_projection().map_err(owner)?;
        let start = usize::try_from(input.start).map_err(owner)?;
        let end = usize::try_from(input.end).map_err(owner)?;
        if end < start || end - start > 4096 {
            return Err(request(
                "select an ordered range of at most 4096 original rows",
            ));
        }
        let maximum_bytes = usize::try_from(input.maximum_bytes).map_err(owner)?;
        let key = EngineeringRunKey {
            operation: input.operation.clone(),
            run: input.run,
            attempt: input.attempt,
        };
        let result = match self.operation(&input.operation, Kind::Run) {
            Ok(operation) => {
                check_issued(&operation, &key)?;
                match operation.snapshot()? {
                    Outcome::Run(handle) => {
                        check_run(&handle, &key)?;
                        Some(
                            handle
                                .result()
                                .ok_or_else(|| request("original native run is not settled"))?,
                        )
                    }
                    Outcome::Retained => None,
                    _ => return Err(request("original run has not started")),
                }
            }
            Err(EngineeringServiceError::Request(_)) => None,
            Err(error) => return Err(error),
        };
        let mut cursor = if let Some(result) = &result {
            SelectedCursor::Local(
                result
                    .table_range(&input.relation, start..end, 1024)
                    .map_err(owner)?,
            )
        } else {
            let (_header, original, attempt) = self.original_record(&key).await?;
            SelectedCursor::Stored(
                self.runtime
                    .retained_data_services()
                    .results(
                        &original.key,
                        &attempt,
                        &input.relation,
                        input.start,
                        input.end,
                        cancel.token(),
                    )
                    .await
                    .map_err(owner)?,
            )
        };
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(
                "engineering:selected-result-response",
            )
            .register(&self.shared.pool()),
        );
        let base = maximum_bytes
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| resource("response allocation extent overflow"))?;
        reservation.try_grow(base).map_err(owner)?;
        let mut sink = pse_ids::BoundedSink::new(Vec::new(), maximum_bytes);
        let mut rows = 0u64;
        {
            let mut writer =
                StreamWriter::try_new(&mut sink, cursor.schema().as_ref()).map_err(owner)?;
            while let Some(batch) = cursor.next_batch().await? {
                cancel.checkpoint().map_err(owner)?;
                let working = pse_columnar::ipc_file::writer_extent(
                    std::slice::from_ref(&batch),
                    maximum_bytes,
                )
                .map_err(owner)?;
                let extent = base
                    .checked_add(working)
                    .ok_or_else(|| resource("writer allocation extent overflow"))?;
                reservation
                    .try_grow(extent.saturating_sub(reservation.size()))
                    .map_err(owner)?;
                writer.write(&batch).map_err(owner)?;
                rows = rows
                    .checked_add(u64::try_from(batch.num_rows()).map_err(owner)?)
                    .ok_or_else(|| resource("selected row extent overflow"))?;
            }
            cancel.checkpoint().map_err(owner)?;
            writer.finish().map_err(owner)?;
        }
        if !cursor.complete() {
            return Err(infrastructure(
                "selected producer ended without complete range exhaustion",
            ));
        }
        Ok(EngineeringResultOutput {
            page: EngineeringResultPage {
                run: input.run,
                relation: input.relation,
                start: input.start,
                end: input.end,
                rows,
                ipc: sink.into_inner(),
            },
            reservation,
            maximum_bytes,
        })
    }

    /// Stop admission, cancel frontend-owned runs, join original creation tasks and native lanes.
    /// Durable study cancellation remains an explicit command to its canonical owner.
    /// # Errors
    /// Every join/native drain failure is retained independently.
    pub async fn shutdown(&self) -> Result<(), EngineeringServiceError> {
        let _shutdown = self.shutdown_lock.lock().await;
        self.closing.store(true, Ordering::Release);
        let mut failures = Vec::new();
        let operations = match self.operations.lock() {
            Ok(operations) => operations.values().cloned().collect::<Vec<_>>(),
            Err(poisoned) => {
                failures.push(infrastructure("operation inventory poisoned"));
                poisoned.into_inner().values().cloned().collect()
            }
        };
        for operation in &operations {
            // Canonical study creation retains its original owner through join.
            // Frontend retirement never becomes a cancellation request for the
            // durable Study now owned by the canonical service and its worker.
            if operation.kind != Kind::Study {
                operation.cancel.cancel();
            }
            match operation.snapshot() {
                Ok(Outcome::Run(handle)) => handle.cancel(),
                Err(error) => failures.push(error),
                _ => {}
            }
        }
        self.runtime.close();
        let mut tasks = match self.tasks.lock() {
            Ok(mut tasks) => std::mem::take(&mut *tasks),
            Err(poisoned) => {
                failures.push(infrastructure("native task inventory poisoned"));
                std::mem::take(&mut *poisoned.into_inner())
            }
        };
        while let Some(joined) = tasks.join_next().await {
            if let Err(error) = joined {
                failures.push(infrastructure(&format!(
                    "original operation join failed: {error}"
                )));
            }
        }
        for operation in operations {
            match operation.snapshot() {
                Ok(Outcome::Run(handle)) => {
                    if let Err(error) = handle.wait().await {
                        failures.push(owner(error));
                    }
                }
                Err(error) => failures.push(error),
                _ => {}
            }
        }
        if let Err(error) = self.shared.math().shutdown_native_lanes().await {
            failures.push(owner(error));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(EngineeringServiceError::Shutdown(failures))
        }
    }
}

fn source_outcome(outcome: Outcome) -> Result<EngineeringSourceOutcome, EngineeringServiceError> {
    Ok(match outcome {
        Outcome::Pending => EngineeringSourceOutcome::Pending,
        Outcome::Source { selection, .. } => EngineeringSourceOutcome::Ready(selection),
        Outcome::Failed(error) => EngineeringSourceOutcome::Failed(failure(error.as_ref())),
        _ => {
            return Err(infrastructure(
                "source operation published an incompatible outcome",
            ));
        }
    })
}
enum SelectedCursor<'a> {
    Local(crate::workflow::ResultCursor<'a>),
    Stored(crate::workflow::CanonicalResultReader),
}
impl SelectedCursor<'_> {
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        match self {
            Self::Local(cursor) => cursor.schema(),
            Self::Stored(cursor) => cursor.schema(),
        }
    }
    async fn next_batch(
        &mut self,
    ) -> Result<Option<datafusion::arrow::record_batch::RecordBatch>, EngineeringServiceError> {
        match self {
            Self::Local(cursor) => cursor
                .next_batch()
                .await
                .map(|batch| batch.map(|batch| batch.into_batch()))
                .map_err(owner),
            Self::Stored(cursor) => cursor.next_batch().await.map_err(owner),
        }
    }
    fn complete(&self) -> bool {
        match self {
            Self::Local(cursor) => cursor.complete(),
            Self::Stored(_) => true,
        }
    }
}
fn check_run(handle: &RunHandle, key: &EngineeringRunKey) -> Result<(), EngineeringServiceError> {
    if handle.run_id().as_id() != key.run
        || handle
            .attempt_id()
            .is_some_and(|attempt| attempt.as_id() != key.attempt)
    {
        return Err(request(
            "original operation has another run or scientific attempt identity",
        ));
    }
    Ok(())
}
fn fingerprint(input: &impl serde::Serialize) -> Result<ContentHash, EngineeringServiceError> {
    let mut digest = EncodingHasher::new();
    serde_json::to_writer(&mut digest, input).map_err(owner)?;
    Ok(digest.finish().content_hash())
}
fn check_issued(
    operation: &Operation,
    key: &EngineeringRunKey,
) -> Result<(), EngineeringServiceError> {
    let original = operation
        .issued
        .lock()
        .map_err(|_| infrastructure("issued run identity lock poisoned"))?;
    if original.as_ref() != Some(key) {
        return Err(request(
            "original operation has another or not yet retained issued run identity",
        ));
    }
    Ok(())
}
fn run_progress(handle: &RunHandle) -> EngineeringRunProgress {
    progress_checkpoint(
        &handle.native_progress_source(),
        handle.observations().as_ref(),
    )
}
fn progress_checkpoint(
    progress: &pse_backend_native::solve::Progress,
    observations: Option<&pse_diagnostics::observation::OperationReader>,
) -> EngineeringRunProgress {
    use pse_backend_native::solve::Metric;
    let current = progress.current();
    let (generation, highest_issued_sequence) = progress.source_identity();
    let source = current.as_ref().and_then(|event| event.source.as_ref());
    let snapshot =
        observations.map(|reader| reader.snapshot(std::time::Duration::from_millis(100)));
    EngineeringRunProgress {
        generation,
        highest_issued_sequence,
        dropped: progress.dropped_count(),
        current_is_latest: source.is_some_and(|source| {
            source.generation == generation && source.sequence == highest_issued_sequence
        }),
        phase: current.as_ref().map(|event| event.phase.clone()),
        elapsed_seconds_bits: current
            .as_ref()
            .map(|event| event.elapsed.as_secs_f64().to_bits()),
        sequence: source.map(|source| source.sequence),
        values: current.map_or_else(BTreeMap::new, |event| {
            event
                .values
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        match value {
                            Metric::Integer(value) => EngineeringProgressValue::Integer(value),
                            Metric::Real(value) => {
                                EngineeringProgressValue::RealBits(value.to_bits())
                            }
                            Metric::Text(value) => EngineeringProgressValue::Text(value),
                            Metric::Bool(value) => EngineeringProgressValue::Boolean(value),
                            Metric::Unavailable(value) => {
                                EngineeringProgressValue::Unavailable(value)
                            }
                        },
                    )
                })
                .collect()
        }),
        observations_available: snapshot
            .as_ref()
            .map(|snapshot| snapshot.coverage.available),
        observation_updates_dropped: snapshot
            .as_ref()
            .map(|snapshot| snapshot.coverage.dropped_updates),
    }
}
pub(super) fn failure(error: &dyn pse_diagnostics::TypedDiagnostic) -> EngineeringFailure {
    let facts = error.diagnostic_facts();
    let mut original: &(dyn std::error::Error + 'static) = error;
    let mut field_path = None;
    let mut field_rule = None;
    for _ in 0..16 {
        if let Some(pse_model::boundary_projection::AdapterError::Refused { path, .. }) =
            original.downcast_ref()
        {
            field_path = Some(path.clone());
            break;
        }
        if let Some(pse_model::boundary_projection::ValidationError::Rule { path, rule, .. }) =
            original.downcast_ref()
        {
            field_path = Some(path.clone());
            field_rule = Some((*rule).into());
            break;
        }
        let Some(source) = original.source() else {
            break;
        };
        original = source;
    }
    EngineeringFailure {
        code: error
            .diagnostic_code()
            .unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure)
            .to_string(),
        message: error.to_string(),
        sources: facts
            .sources
            .into_iter()
            .map(pse_ids::SemanticId::from_bytes)
            .collect(),
        rule: facts.rule.map(|rule| rule.to_string()),
        field_path,
        field_rule,
    }
}
fn request(message: &str) -> EngineeringServiceError {
    EngineeringServiceError::Request(message.into())
}
fn resource(message: &str) -> EngineeringServiceError {
    EngineeringServiceError::Resource(message.into())
}
fn infrastructure(message: &str) -> EngineeringServiceError {
    EngineeringServiceError::Infrastructure(message.into())
}
fn owner(error: impl Into<OwnerError>) -> EngineeringServiceError {
    let source = match error
        .into()
        .downcast::<datafusion::common::DataFusionError>()
    {
        Ok(error) => Box::new(pse_engine::EngineError::from(*error)),
        Err(source) => source,
    };
    EngineeringServiceError::Owner(source)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "disposable source fixtures and exact original-owner assertions"
    )]
    use super::*;
    use std::{num::NonZeroUsize, path::Path, time::Duration};

    fn edit_input(operation: &str, source: &str) -> EngineeringEditSource {
        EngineeringEditSource {
            operation: operation.into(),
            source_operation: source.into(),
            delta: SourceFileDelta {
                parent: "original-parent".into(),
                edits: vec![SourceFileEdit {
                    package: pse_ids::SemanticId::from_bytes([41; 16]),
                    path: "models/add.pse".into(),
                    change: SourceFileChange::Add(b"package local {}".to_vec()),
                }],
            },
        }
    }

    #[tokio::test]
    async fn engineering_source_edit_refuses_self_and_future_dependencies_before_issue() {
        let (service, _spill) = service();
        for input in [edit_input("self", "self"), edit_input("first", "future")] {
            let operation = input.operation.clone();
            assert!(matches!(
                service.edit_source(input).await,
                Err(EngineeringServiceError::Request(_))
            ));
            assert!(service.operation(&operation, Kind::Source).is_err());
        }
        tokio::time::timeout(Duration::from_secs(10), service.shutdown())
            .await
            .expect("invalid dependency admission must not leave shutdown work")
            .unwrap();
    }

    #[tokio::test]
    async fn engineering_shutdown_settles_pending_dependency_chains() {
        let (service, _spill) = service();
        let source = source_input();
        let (_, fresh) = service
            .issue(&source.operation, Kind::Source, &source)
            .unwrap();
        assert!(fresh);
        // An intentionally unpublished original source isolates dependency waits
        // from source parsing and native computation.
        for (operation, dependency) in [
            ("edit-first", source.operation.as_str()),
            ("edit-second", "edit-first"),
        ] {
            assert!(matches!(
                service
                    .edit_source(edit_input(operation, dependency))
                    .await
                    .unwrap(),
                EngineeringSourceOutcome::Pending
            ));
        }
        let preparation_service = service.clone();
        let prepare = tokio::spawn(async move {
            preparation_service
                .prepare_case(EngineeringPrepareCase {
                    operation: "dependency-preparation".into(),
                    source_operation: "edit-second".into(),
                    expected_local_revision: "original-parent".into(),
                    case: pse_ids::SemanticId::from_bytes([42; 16]),
                    intent: pse_model::generated::enums::NativeSolveIntent::Root,
                    backend: None,
                })
                .await
        });
        tokio::time::timeout(Duration::from_secs(10), async {
            while service
                .operation("dependency-preparation", Kind::Preparation)
                .is_err()
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let submit_service = service.clone();
        let submit = tokio::spawn(async move {
            submit_service
                .submit_run(EngineeringSubmitRun {
                    operation: "dependency-run".into(),
                    preparation_operation: "dependency-preparation".into(),
                    run: pse_ids::SemanticId::from_bytes([43; 16]),
                    attempt: pse_ids::SemanticId::from_bytes([44; 16]),
                    expected_preparation: ContentHash::from_bytes([45; 32]),
                })
                .await
        });
        tokio::time::timeout(Duration::from_secs(10), async {
            while service.operation("dependency-run", Kind::Run).is_err() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        tokio::time::timeout(Duration::from_secs(10), service.shutdown())
            .await
            .expect("shutdown must cancel retained dependency waiters")
            .unwrap();
        assert!(prepare.await.unwrap().is_err());
        assert!(submit.await.unwrap().unwrap().failure.is_some());
        for (key, kind) in [
            ("edit-first", Kind::Source),
            ("edit-second", Kind::Source),
            ("dependency-preparation", Kind::Preparation),
            ("dependency-run", Kind::Run),
        ] {
            let Outcome::Failed(error) = service.operation(key, kind).unwrap().snapshot().unwrap()
            else {
                panic!("cancelled dependency must publish its terminal refusal");
            };
            assert_eq!(
                pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()),
                Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
            );
        }
    }

    #[tokio::test]
    async fn pending_run_cancellation_checks_original_issued_coordinates_before_acknowledgement() {
        let (service, _spill) = service();
        let input = EngineeringSubmitRun {
            operation: "pending-original".into(),
            preparation_operation: "preparation".into(),
            run: pse_ids::SemanticId::from_bytes([51; 16]),
            attempt: pse_ids::SemanticId::from_bytes([52; 16]),
            expected_preparation: ContentHash::from_bytes([54; 32]),
        };
        let key = EngineeringRunKey {
            operation: input.operation.clone(),
            run: input.run,
            attempt: input.attempt,
        };
        let (original, fresh) = service.issue_run(&input, &key).unwrap();
        assert!(fresh);
        let mut changed = key.clone();
        changed.attempt = pse_ids::SemanticId::from_bytes([53; 16]);
        assert!(service.cancel_run(&changed).await.is_err());
        assert!(service.run_status(&changed).await.is_err());
        assert!(!original.cancel.token().is_cancelled());
        let cancelled = service.cancel_run(&key).await.unwrap();
        assert!(cancelled.cancellation_requested);
        assert!(!cancelled.settled);
        assert!(cancelled.run.is_none());
        assert!(cancelled.usable.is_none());
        assert!(original.cancel.token().is_cancelled());
        assert!(!service.issue_run(&input, &key).unwrap().1);
        service.shutdown().await.unwrap();
    }

    #[test]
    fn original_progress_survives_zero_history_and_keeps_exact_metrics_and_loss() {
        use pse_backend_native::solve::{Event, Metric, Progress};
        let source = Arc::new(Progress::new(0));
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("engineering.passive-progress");
        let reader = operation.reader();
        let nan_bits = 0x7ff8_0000_0000_0042_u64;
        source.push(Event {
            source: None,
            phase: "iteration".into(),
            elapsed: Duration::from_millis(125),
            values: BTreeMap::from([
                ("count".into(), Metric::Integer(9_007_199_254_740_993)),
                ("evidence".into(), Metric::Real(f64::from_bits(nan_bits))),
            ]),
            incumbent: None,
        });
        let first = progress_checkpoint(&source, Some(&reader));
        assert!(source.snapshot().0.is_empty());
        assert_eq!(first.phase.as_deref(), Some("iteration"));
        assert_eq!(first.sequence, Some(1));
        assert!(first.current_is_latest);
        assert_eq!(
            first.values["count"],
            EngineeringProgressValue::Integer(9_007_199_254_740_993)
        );
        assert_eq!(
            first.values["evidence"],
            EngineeringProgressValue::RealBits(nan_bits)
        );
        assert_eq!(first.observations_available, Some(true));
        drop(reader);
        source.push(Event {
            source: None,
            phase: "native-next".into(),
            elapsed: Duration::from_millis(250),
            values: BTreeMap::new(),
            incumbent: None,
        });
        let next = progress_checkpoint(&source, None);
        assert_eq!(next.generation, first.generation);
        assert_eq!(next.highest_issued_sequence, 2);
        assert_eq!(next.sequence, Some(2));
        assert_eq!(next.phase.as_deref(), Some("native-next"));
        assert_eq!(next.dropped, source.dropped_count());
        assert_eq!(next.observations_available, None);
    }

    fn service() -> (Arc<EngineeringService>, tempfile::TempDir) {
        service_on(None)
    }
    pub(super) fn service_on(
        store: Option<pse_operations::canonical::CanonicalStore>,
    ) -> (Arc<EngineeringService>, tempfile::TempDir) {
        let spill = tempfile::tempdir().unwrap();
        let n = |value| NonZeroUsize::new(value).unwrap();
        let shared = SharedRuntime::build(crate::ResourceBudget {
            memory_limit_bytes: n(4usize << 30),
            spill_dir: spill.path().into(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: n(16),
            threads: pse_engine::ThreadBudget {
                pool_threads: n(16),
                target_partitions: n(16),
            },
            execution: Default::default(),
            cache: crate::CacheBudget::disabled(1024),
            math: crate::math::MathPolicy::for_pool(4usize << 30),
        })
        .unwrap();
        let sessions = Arc::new(
            shared
                .session_factory(pse_engine::session::native_engine_profile())
                .unwrap(),
        );
        let model = sessions
            .bind_model(pse_schema::shared_registry().unwrap())
            .unwrap();
        let runtime = match store {
            None => Runtime::from_shared_local(shared.clone(), model).unwrap(),
            Some(store) => {
                let mut artifact = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
                let mut digest = EncodingHasher::new();
                std::io::copy(&mut artifact, &mut digest).unwrap();
                Runtime::from_shared(
                    shared.clone(),
                    model,
                    crate::workflow::CanonicalDeployment::new(
                        store,
                        crate::workflow::OuterAttestation {
                            source: None,
                            build: digest.finish().content_hash(),
                        },
                        None,
                    ),
                )
                .unwrap()
            }
        };
        (EngineeringService::new(runtime, shared, 16).unwrap(), spill)
    }
    fn source_input() -> EngineeringAdmitSource {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
        let physical = root.join("physical-primitives");
        let files = [
            "package.toml",
            "materials/physical.yaml",
            "materials/time.yaml",
        ]
        .into_iter()
        .map(|path| (path.into(), std::fs::read(physical.join(path)).unwrap()))
        .collect();
        let manifest = std::fs::read_to_string(root.join("minimal_named/package.toml")).unwrap()
            .replace("dependencies = []", "dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        EngineeringAdmitSource { operation: "source-original".into(), physical: vec![EngineeringPackage { files }],
            modeling: vec![EngineeringPackage { files: BTreeMap::from([("package.toml".into(), manifest.into_bytes()),
                ("models/root.pse".into(), b"package local { test square fixture { dof 0; } { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(2); } }".to_vec())]) }] }
    }
    pub(super) async fn admitted(service: &Arc<EngineeringService>) -> EngineeringSourceSelection {
        let input = source_input();
        let _ = service.admit_source(input.clone()).await.unwrap();
        let original = service.operation(&input.operation, Kind::Source).unwrap();
        let Outcome::Source { selection, .. } =
            tokio::time::timeout(Duration::from_secs(60), original.completed())
                .await
                .unwrap()
                .unwrap()
        else {
            panic!("source admission must produce its original package")
        };
        selection
    }

    #[tokio::test]
    async fn engineering_original_source_survives_dropped_waiter_and_rejects_changed_identity() {
        let (service, _spill) = service();
        let result = std::panic::AssertUnwindSafe(async {
            let selection = admitted(&service).await;
            let repeated = service.admit_source(source_input()).await.unwrap();
            assert_eq!(repeated, EngineeringSourceOutcome::Ready(selection.clone()));
            let mut changed = source_input();
            changed.modeling[0].files.insert(
                "models/root.pse".into(),
                b"different authored source".to_vec(),
            );
            assert!(matches!(
                service.admit_source(changed).await,
                Err(EngineeringServiceError::Request(_))
            ));
            assert_eq!(
                service
                    .source_status(&EngineeringOperationKey {
                        operation: selection.operation.clone()
                    })
                    .unwrap(),
                EngineeringSourceOutcome::Ready(selection.clone())
            );
            let request = EngineeringInspectSource {
                operation: selection.operation,
                offset: 0,
                limit: 1,
            };
            let cancelled = CancelSource::new();
            cancelled.cancel();
            let error = service
                .inspect_source(request.clone(), &cancelled)
                .await
                .unwrap_err();
            assert_eq!(
                pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
                Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
            );
            let page = service
                .inspect_source(request, &CancelSource::new())
                .await
                .unwrap();
            assert_eq!(page.declarations.len(), 1);
            assert!(page.total > 1);
            assert_eq!(page.next_offset, Some(1));
        })
        .catch_unwind()
        .await;
        service.shutdown().await.unwrap();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    #[tokio::test]
    async fn engineering_shutdown_settles_pending_original_work_and_seals_new_admission() {
        let (service, _spill) = service();
        let input = source_input();
        let _ = service.admit_source(input.clone()).await.unwrap();
        service.shutdown().await.unwrap();
        assert!(!matches!(
            service
                .source_status(&EngineeringOperationKey {
                    operation: input.operation.clone()
                })
                .unwrap(),
            EngineeringSourceOutcome::Pending
        ));
        let mut changed = input;
        changed.operation = "new-after-close".into();
        assert!(service.admit_source(changed).await.is_err());
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn engineering_actual_native_run_identity_replay_and_selected_result_readback() {
        let (service, _spill) = service();
        let result = std::panic::AssertUnwindSafe(async {
            let source = admitted(&service).await;
            let page = service
                .inspect_source(
                    EngineeringInspectSource {
                        operation: source.operation.clone(),
                        offset: 0,
                        limit: 128,
                    },
                    &CancelSource::new(),
                )
                .await
                .unwrap();
            let case = page
                .declarations
                .iter()
                .find(|declaration| declaration.name == "square")
                .unwrap()
                .declaration;
            let mut preparation = EngineeringPrepareCase {
                operation: "prepare-original".into(),
                source_operation: source.operation,
                expected_local_revision: "stale-parent".into(),
                case,
                intent: pse_model::generated::enums::NativeSolveIntent::Root,
                backend: Some(pse_model::generated::enums::NativeBackend::Kinsol),
            };
            assert!(service.prepare_case(preparation.clone()).await.is_err());
            preparation.operation = "prepare-current".into();
            preparation.expected_local_revision = source.local_revision;
            let prepared = service.prepare_case(preparation).await.unwrap();
            let submit = EngineeringSubmitRun {
                operation: "run-original".into(),
                preparation_operation: prepared.operation,
                run: pse_ids::SemanticId::from_bytes([21; 16]),
                attempt: pse_ids::SemanticId::from_bytes([22; 16]),
                expected_preparation: prepared.preparation,
            };
            let first = service.submit_run(submit.clone()).await.unwrap();
            let second = service.submit_run(submit.clone()).await.unwrap();
            assert_eq!(first.run, second.run);
            let original = service.operation(&submit.operation, Kind::Run).unwrap();
            let Outcome::Run(handle) = original.snapshot().unwrap() else {
                panic!("actual run handle is retained")
            };
            drop(handle.observations());
            let completed = tokio::time::timeout(Duration::from_secs(60), handle.wait())
                .await
                .unwrap()
                .unwrap();
            assert!(completed.usable());
            let status = service
                .run_status(&EngineeringRunKey {
                    operation: submit.operation.clone(),
                    run: submit.run,
                    attempt: submit.attempt,
                })
                .await
                .unwrap();
            assert!(status.settled);
            assert_eq!(status.usable, Some(true));
            let read = EngineeringReadResult {
                operation: submit.operation,
                relation: "runtime.solve_variables".into(),
                start: 0,
                end: 1,
                maximum_bytes: 1 << 20,
                run: submit.run,
                attempt: submit.attempt,
            };
            let output = service
                .read_result(read.clone(), &CancelSource::new())
                .await
                .unwrap();
            assert_eq!(output.page.rows, 1);
            let mut foreign_attempt = read.clone();
            foreign_attempt.attempt = pse_ids::SemanticId::from_bytes([23; 16]);
            assert!(matches!(
                service
                    .read_result(foreign_attempt, &CancelSource::new())
                    .await,
                Err(EngineeringServiceError::Request(_))
            ));
            let mut stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
                std::io::Cursor::new(&output.page.ipc),
                None,
            )
            .unwrap();
            assert_eq!(stream.next().unwrap().unwrap().num_rows(), 1);
            assert!(stream.next().is_none());
            let mut too_small = read;
            too_small.maximum_bytes = 1;
            assert!(
                service
                    .read_result(too_small, &CancelSource::new())
                    .await
                    .is_err()
            );
        })
        .catch_unwind()
        .await;
        service.shutdown().await.unwrap();
        if let Err(panic) = result {
            std::panic::resume_unwind(panic);
        }
    }

    #[cfg(all(feature = "canonical-tests", feature = "solver-kinsol"))]
    #[tokio::test]
    async fn canonical_engineering_original_run_survives_lost_waiter_and_service_retirement() {
        let store = pse_operations::testing::canonical_fixture_store().unwrap();
        let (service, _spill) = service_on(Some(store.clone()));
        let source = admitted(&service).await;
        let page = service
            .inspect_source(
                EngineeringInspectSource {
                    operation: source.operation.clone(),
                    offset: 0,
                    limit: 128,
                },
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let case = page
            .declarations
            .iter()
            .find(|declaration| declaration.name == "square")
            .unwrap()
            .declaration;
        let prepare = EngineeringPrepareCase {
            operation: "prepare-recorded".into(),
            source_operation: source.operation,
            expected_local_revision: source.local_revision,
            case,
            intent: pse_model::generated::enums::NativeSolveIntent::Root,
            backend: Some(pse_model::generated::enums::NativeBackend::Kinsol),
        };
        let prepared = service.prepare_case(prepare).await.unwrap();
        let submit = EngineeringSubmitRun {
            operation: "issued-recorded".into(),
            preparation_operation: prepared.operation,
            run: pse_ids::SemanticId::from_bytes([31; 16]),
            attempt: pse_ids::SemanticId::from_bytes([32; 16]),
            expected_preparation: prepared.preparation,
        };
        let key = EngineeringRunKey {
            operation: submit.operation.clone(),
            run: submit.run,
            attempt: submit.attempt,
        };
        // The canonical preflight is read-only. Wait until actual service custody
        // exists, then lose the acknowledgement without inspecting its value.
        let caller = service.clone();
        let submitted = submit.clone();
        let waiter = tokio::spawn(async move { caller.submit_run(submitted).await });
        let original = tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                if let Ok(original) = service.operation(&submit.operation, Kind::Run) {
                    break original;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        waiter.abort();
        let _discarded_acknowledgement = waiter.await;
        let outcome = tokio::time::timeout(Duration::from_secs(90), original.completed())
            .await
            .unwrap()
            .unwrap();
        let Outcome::Run(handle) = outcome else {
            panic!("original task retains its native run")
        };
        let result = tokio::time::timeout(Duration::from_secs(90), handle.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(result.usable());
        assert_eq!(handle.run_id().as_id(), submit.run);
        assert_eq!(handle.attempt_id().unwrap().as_id(), submit.attempt);
        let first = service.submit_run(submit.clone()).await.unwrap();
        assert!(first.settled);
        assert!(first.canonical_run.is_some());
        assert!(first.canonical_attempt.is_some());
        assert!(first.progress.is_some());
        service.shutdown().await.unwrap();
        drop(handle);
        drop(original);
        drop(result);
        drop(service);

        let (reader, _reader_spill) = service_on(Some(store.clone()));
        // Neither the original source nor its preparation exists in this new
        // incarnation. Exact submission resolves only the canonical original.
        let mut changed_submit = submit.clone();
        changed_submit.preparation_operation = "another-preparation".into();
        assert!(reader.submit_run(changed_submit).await.is_err());
        let resubmitted = reader.submit_run(submit.clone()).await.unwrap();
        assert_eq!(resubmitted.canonical_run, first.canonical_run);
        assert_eq!(resubmitted.canonical_attempt, first.canonical_attempt);
        assert!(
            reader.operations.lock().unwrap().is_empty(),
            "canonical replay must not create another native operation"
        );
        let mut changed_source = source_input();
        let body = changed_source.modeling[0]
            .files
            .get_mut("models/root.pse")
            .unwrap();
        let text = std::str::from_utf8(body).unwrap();
        assert!(text.contains("p:Scalar=4"));
        *body = text.replace("p:Scalar=4", "p:Scalar=16").into_bytes();
        let changed_operation = changed_source.operation.clone();
        reader.admit_source(changed_source).await.unwrap();
        let Outcome::Source {
            selection: source, ..
        } = tokio::time::timeout(
            Duration::from_secs(90),
            reader
                .operation(&changed_operation, Kind::Source)
                .unwrap()
                .completed(),
        )
        .await
        .unwrap()
        .unwrap()
        else {
            panic!("new incarnation retains the changed source")
        };
        let prepared = reader
            .prepare_case(EngineeringPrepareCase {
                operation: submit.preparation_operation.clone(),
                source_operation: source.operation,
                expected_local_revision: source.local_revision,
                case,
                intent: pse_model::generated::enums::NativeSolveIntent::Root,
                backend: Some(pse_model::generated::enums::NativeBackend::Kinsol),
            })
            .await
            .unwrap();
        assert_ne!(prepared.preparation, submit.expected_preparation);
        let mut rebound = submit;
        rebound.expected_preparation = prepared.preparation;
        assert!(matches!(
            reader.submit_run(rebound).await,
            Err(EngineeringServiceError::Request(_))
        ));
        let observed = reader.run_status(&key).await.unwrap();
        assert_eq!(observed.run, first.run);
        assert_eq!(observed.usable, Some(true));
        assert!(observed.settled);
        assert!(
            observed.progress.is_none(),
            "retirement must not fabricate live source observations"
        );
        assert_eq!(observed.canonical_attempt, first.canonical_attempt);
        let page = reader
            .read_result(
                EngineeringReadResult {
                    operation: key.operation.clone(),
                    run: key.run,
                    attempt: key.attempt,
                    relation: "runtime.solve_variables".into(),
                    start: 0,
                    end: 1,
                    maximum_bytes: 1 << 20,
                },
                &CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(page.page.rows, 1);
        let mut different = key;
        different.attempt = pse_ids::SemanticId::from_bytes([33; 16]);
        assert!(reader.run_status(&different).await.is_err());
        drop(page);
        reader.shutdown().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}
