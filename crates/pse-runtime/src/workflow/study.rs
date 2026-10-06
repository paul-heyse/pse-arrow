// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical study definitions, scoped occurrence dispatch and retained outcomes.
use super::{
    AdmittedBinding, OperationRequest, OperationSource, Operations, PointOverlay,
    PreparationSettings, Runtime, StudyOperation, WorkflowError, contract,
};
use pse_ids::SemanticId;
use pse_model::generated::{
    enums::{AttemptState, StudyPointState, StudyState},
    identities::{RunId, StudyId},
};
pub use pse_model::study::{PointAttemptOutcome, PointOutcome};
use pse_model::{
    document::Version,
    study::{OccurrenceGraph, PointPolicy},
};
use pse_operations::canonical_execution::RunRequest as StoredRequest;
use pse_operations::canonical_studies::{
    NewOccurrence, ScopedStudyPoint, StudyPoint as CanonicalPoint, StudyScope, StudySummary,
    point_facts, point_outcome, point_policy,
};
use std::{collections::BTreeMap, time::Duration};

/// The most points one study holds.
pub const MAXIMUM_STUDY_POINTS: usize = 100_000;

/// The authored sources of a package closure, each document's exact bytes by path (a
/// data document's included, ADR-0125): initial authored ingress only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSources {
    /// The physical package's documents.
    pub physical: BTreeMap<String, Vec<u8>>,
    /// The modeling package closure's documents, one map per package in load order.
    pub modeling: Vec<BTreeMap<String, Vec<u8>>>,
}

/// A ready operation and the actual seed fact consumed by shared policy. Numerical
/// preparation is deferred until immediate dependencies have settled.
pub(crate) struct PreparedStudyCandidate {
    pub(crate) operation: super::PreparedStudyOperation,
    pub(crate) seed: Option<pse_model::study::SeedFact>,
    prediction: Option<StudyPrediction>,
}

struct StudyPrediction {
    package: super::ModelingPackage,
    predecessor: ScopedStudyPoint,
    seed: pse_model::generated::identities::SolutionId,
    original: crate::math::solves::PreparedSolve,
}

fn document(value: &impl serde::Serialize) -> Result<Vec<u8>, WorkflowError> {
    serde_json::to_vec(value).map_err(|error| contract(error.to_string()))
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StudyMetadata {
    version: Version<1>,
    physical: super::PhysicalSource,
}

type StoredPoint = (StudyOperation, pse_ids::roles::BindingHash, AdmittedBinding);
fn definition_of(point: &CanonicalPoint) -> Result<StudyPointDefinition, WorkflowError> {
    let (operation, binding_hash, binding): StoredPoint =
        serde_json::from_slice(point.descriptor.as_slice())
            .map_err(|error| contract(format!("immutable point descriptor: {error}")))?;
    if binding.identity() != binding_hash {
        return Err(contract("stored binding content differs"));
    }
    Ok(StudyPointDefinition {
        operation,
        binding_hash,
        binding,
        policy: point_policy(point)?,
    })
}

impl Runtime {
    /// The canonical execution owner; ephemerality is an explicit kernel adapter.
    pub(crate) fn operations(&self) -> Result<&Operations, WorkflowError> {
        match &self.durability {
            super::Durability::Durable(operations) => Ok(operations),
            super::Durability::Ephemeral => {
                Err(contract("canonical execution requires durable runtime"))
            }
        }
    }

    /// Admit authored ingress once and retain one canonical study before preparing ready occurrences.
    pub async fn start_study(&self, plan: StudyPlan) -> Result<StudyHandle, WorkflowError> {
        let operations = self.operations()?;
        let physical = operations.put_sources(&plan.sources.physical).await?;
        let cancel = crate::CancelSource::new();
        let package = self
            .package_from_sources(
                &plan.sources.modeling,
                self.physical_source(&physical, &cancel).await?,
            )
            .await?;
        let definition = package
            .admit_study_points(physical, &plan.points, &cancel)
            .await?;
        self.start_defined_study(plan.sources.physical, definition)
            .await
    }

    /// Persist the exact admitted immutable definition in bounded occurrence batches.
    pub async fn start_defined_study(
        &self,
        physical_sources: BTreeMap<String, Vec<u8>>,
        definition: StudyDefinition,
    ) -> Result<StudyHandle, WorkflowError> {
        let operations = self.operations()?;
        let physical = operations.put_sources(&physical_sources).await?;
        if physical != definition.physical {
            return Err(contract("study physical source identity differs"));
        }
        definition.validate_roles()?;
        if definition.points.is_empty() || definition.points.len() > MAXIMUM_STUDY_POINTS {
            return Err(contract("bounded admitted study extent"));
        }
        let revision = self
            .canonical_store()
            .revision(&definition.modeling_revision)
            .await?
            .ok_or_else(|| contract("canonical study revision absent"))?;
        let cancel = crate::CancelSource::new();
        let package = self
            .modeling_revision(
                revision.clone(),
                self.physical_source(&physical, &cancel).await?,
                BTreeMap::new(),
            )
            .await?;
        // Immutable scientific entry admission is complete before any claim. This
        // checks bindings and seed roles without constructing native solve views.
        for point in &definition.points {
            if point.binding.identity() != point.binding_hash {
                return Err(contract("study binding content differs"));
            }
            if point
                .operation
                .admit_binding_seed_need(&package, &point.binding, &cancel)
                .await?
                != point.policy.seed_need
            {
                return Err(contract(
                    "study seed consumption differs from its scientific owner",
                ));
            }
        }
        let study_id: StudyId = pse_operations::mint_id();
        let run_id: RunId = pse_operations::mint_id();
        let key = study_id.to_string();
        let run = format!("run:{run_id}");
        let physical_revision = self
            .canonical_store()
            .revision(&physical.revision)
            .await?
            .ok_or_else(|| contract("canonical physical source absent"))?;
        let attestation = self.canonical.attestation();
        let attestation = document(&(attestation.source, attestation.build))?;
        let source_selection = document(&(1_u8, &definition.modeling_revision, &physical))?;
        let request = StoredRequest {
            key: run,
            revision: revision.clone(),
            sources: vec![physical_revision.clone()],
            request: document(&(
                1_u8,
                study_id,
                run_id,
                pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &definition)
                    .map_err(|error| contract(error.to_string()))?,
            ))?,
            source_selection: source_selection.clone(),
            attestation: attestation.clone(),
        };
        let metadata = document(&StudyMetadata {
            version: Version::<1>,
            physical: physical.clone(),
        })?;
        let points = definition
            .points
            .iter()
            .map(|point| {
                let point_run: RunId = pse_operations::mint_id();
                let descriptor = document(&(&point.operation, point.binding_hash, &point.binding))?;
                Ok(NewOccurrence {
                    policy: point.policy.clone(),
                    descriptor: descriptor.clone(),
                    run: StoredRequest {
                        key: format!("run:{point_run}"),
                        revision: revision.clone(),
                        sources: vec![physical_revision.clone()],
                        request: document(&(
                            1_u8,
                            point_run,
                            study_id,
                            point.policy.key,
                            pse_ids::document::of(
                                pse_ids::Frame::DurableStudyRequestV5,
                                &(&point.operation, point.binding_hash, &point.policy),
                            )
                            .map_err(|error| contract(error.to_string()))?,
                        ))?,
                        source_selection: source_selection.clone(),
                        attestation: attestation.clone(),
                    },
                })
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        self.canonical_store()
            .create_study(&key, &request, &metadata, &points)
            .await?;
        Ok(self.study(study_id))
    }

    /// Exact handle; equal bindings never merge study identities or occurrences.
    pub fn study(&self, study_id: StudyId) -> StudyHandle {
        StudyHandle {
            runtime: self.clone(),
            study_id,
        }
    }

    /// Reconstruct only one structurally ready candidate under protected immutable sources.
    pub(crate) async fn prepare_study_candidate(
        &self,
        scope: &StudyScope,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedStudyCandidate, WorkflowError> {
        use pse_model::study::{ActionKind, SeedAvailability, SeedFact, StartPolicy};
        let point = self
            .canonical_store()
            .canonical_study_point(&scope.point().key)
            .await?
            .ok_or_else(|| contract("study occurrence absent"))?;
        let definition = definition_of(&point)?;
        let unresolved = match &definition.policy.start {
            StartPolicy::Fresh => None,
            StartPolicy::Explicit { role, .. } => Some(SeedFact {
                role: *role,
                availability: SeedAvailability::Unresolved,
            }),
            StartPolicy::Continuation(edge) => Some(SeedFact {
                role: edge.role,
                availability: SeedAvailability::Unresolved,
            }),
        };
        let preliminary = scope.action(unresolved)?;
        if !matches!(
            preliminary.kind,
            ActionKind::Start(_)
                | ActionKind::Wait(pse_model::study::WaitReason::SeedResolution { .. })
        ) {
            return Err(contract("candidate is not structurally ready"));
        }
        let study = self
            .canonical_store()
            .canonical_study(&point.study)
            .await?
            .ok_or_else(|| contract("study header absent"))?;
        let StudyMetadata { physical, .. } = serde_json::from_slice(study.metadata.as_slice())
            .map_err(|error| contract(format!("study interpretation: {error}")))?;
        let physical = self.physical_source(&physical, cancel).await?;
        let revision = self
            .canonical_store()
            .revision(&study.revision)
            .await?
            .ok_or_else(|| contract("study revision absent"))?;
        let package = self
            .modeling_revision(revision, physical, BTreeMap::new())
            .await?;
        let mut operation = package
            .prepare_bound_operation(&definition.operation, &definition.binding, cancel)
            .await?;
        if operation.seed_need() != definition.policy.seed_need {
            return Err(contract(
                "ready native seed consumption differs from admission",
            ));
        }
        let original = match &operation {
            super::PreparedStudyOperation::DeclaredCase(case)
                if case
                    .solve
                    .composition_request()
                    .recovery
                    .contains(&pse_model::strategy::StartOrigin::Predicted)
                    && case.solve.numerical_strategy().start.policy
                        != pse_backend_native::solve::StartPolicy::Explicit =>
            {
                Some(case.solve.clone())
            }
            _ => None,
        };
        let mut prediction = None;
        let seed = match &definition.policy.start {
            StartPolicy::Fresh => None,
            start => {
                let (role, selected) = match start {
                    StartPolicy::Explicit { role, seed } => (*role, Some(*seed)),
                    StartPolicy::Continuation(edge) => {
                        let predecessor = scope
                            .predecessors()
                            .iter()
                            .find(|point| point.occurrence == u64::from(edge.predecessor.0))
                            .ok_or_else(|| contract("immediate seed predecessor absent"))?;
                        let selected = if let (
                            super::PreparedStudyOperation::DeclaredCase(case),
                            Some(attempt),
                        ) = (&operation, predecessor.attempt.as_deref())
                        {
                            if let (Some(target), Some(preparation)) = (
                                case.solve.compatibility(),
                                case.solve.seed_preparation_identity(),
                            ) {
                                self.operations()?
                                    .latest_seed(target, &preparation, Some(attempt))
                                    .await?
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (edge.role, selected)
                    }
                    StartPolicy::Fresh => {
                        return Err(contract("fresh policy cannot request a seed"));
                    }
                };
                let availability = if let Some(selected) = selected {
                    if let super::PreparedStudyOperation::DeclaredCase(case) = &operation {
                        match case
                            .as_ref()
                            .clone()
                            .with_stored_start(
                                self.operations()?,
                                super::StoredStart::Solution(selected),
                            )
                            .await
                        {
                            Ok(seeded) => {
                                if let (StartPolicy::Continuation(edge), Some(original)) =
                                    (start, original.as_ref())
                                {
                                    let predecessor = scope
                                        .predecessors()
                                        .iter()
                                        .find(|point| {
                                            point.occurrence == u64::from(edge.predecessor.0)
                                        })
                                        .ok_or_else(|| {
                                            contract(
                                                "prediction source outside immediate claim scope",
                                            )
                                        })?;
                                    prediction = Some(StudyPrediction {
                                        package: package.clone(),
                                        predecessor: predecessor.clone(),
                                        seed: selected,
                                        original: original.clone(),
                                    });
                                }
                                operation =
                                    super::PreparedStudyOperation::DeclaredCase(Box::new(seeded));
                                SeedAvailability::Compatible { seed: selected }
                            }
                            Err(WorkflowError::SeedRead(
                                super::durable::SeedReadError::Missing { .. },
                            )) => SeedAvailability::Absent,
                            Err(error)
                                if error.boundary_diagnostic().class
                                    == pse_model::diagnostic::BoundaryClass::Incompatible =>
                            {
                                operation =
                                    super::PreparedStudyOperation::DeclaredCase(case.clone());
                                SeedAvailability::Incompatible
                            }
                            Err(error) => return Err(error),
                        }
                    } else {
                        SeedAvailability::Incompatible
                    }
                } else {
                    SeedAvailability::Absent
                };
                Some(SeedFact { role, availability })
            }
        };
        Ok(PreparedStudyCandidate {
            operation,
            seed,
            prediction,
        })
    }

    /// Numerical proposal selection and original screening run only after native claim.
    pub(crate) async fn apply_study_prediction(
        &self,
        prepared: &mut PreparedStudyCandidate,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        let Some(input) = prepared.prediction.take() else {
            return Ok(());
        };
        let super::PreparedStudyOperation::DeclaredCase(target) = &mut prepared.operation else {
            return Err(contract("prediction target ceased to be an original case"));
        };
        let source = self
            .prediction_sample(&input.package, &input.predecessor, input.seed, cancel)
            .await?;
        let Some(mut source) = source else {
            return Ok(());
        };
        let deadline = std::time::Instant::now()
            .checked_add(input.original.time_limit())
            .ok_or_else(|| contract("prediction deadline extent"))?;
        let scope = input.original.task_scope().unwrap_or_else(|| {
            pse_kernels::ExecutionScope::new(std::sync::Arc::default(), Some(deadline))
        });
        let original = self
            .native()
            .admit_proposal_task(input.original, scope.clone())
            .map_err(crate::math::MathRuntimeError::from)?;
        if let Some(root) = &source.point.root {
            let primal = source
                .point
                .primal
                .iter()
                .copied()
                .map(f64::from_bits)
                .collect();
            let values = root.values.iter().copied().map(f64::from_bits).collect();
            source.root = self
                .native()
                .restore_root_predictor(
                    source.prepared.solve.clone(),
                    primal,
                    values,
                    root.key,
                    scope.clone(),
                    original.task_admission(),
                    cancel,
                )
                .await
                .map_err(|error| {
                    pse_backend_native::square_response::Withheld::Cause(std::sync::Arc::new(
                        error.into_problem(),
                    ))
                });
        }
        let older = match input
            .predecessor
            .start
            .as_ref()
            .map(|bytes| {
                serde_json::from_slice::<pse_model::study::StartProvenance>(bytes.as_slice())
                    .map_err(|error| contract(error.to_string()))
            })
            .transpose()?
        {
            Some(pse_model::study::StartProvenance::Continuation {
                predecessor, seed, ..
            }) => {
                let key = pse_operations::canonical_studies::point_key(
                    &input.predecessor.study,
                    predecessor,
                );
                let point = self
                    .canonical_store()
                    .canonical_study_point(&key)
                    .await?
                    .ok_or_else(|| contract("selected older prediction occurrence absent"))?;
                self.prediction_sample(
                    &input.package,
                    &ScopedStudyPoint::from(&point),
                    seed,
                    cancel,
                )
                .await?
            }
            _ => None,
        };
        let mut execution = pse_backend_native::solve::Execution::within(
            scope.cancellation().clone(),
            &pse_backend_native::solve::Controls::default(),
            scope.clone(),
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        execution.work_admission = original
            .task_admission()
            .map(|owner| -> std::sync::Arc<dyn pse_backend_native::solve::WorkAdmission> { owner });
        let mut destination = target.as_ref().clone();
        destination.solve = original;
        let branch = destination.solve.composition_request().branch;
        match source.available_prediction(older.as_ref(), &destination, branch, &execution) {
            Ok(proposal) => {
                let screened = self
                    .native()
                    .screen_start(destination.solve.clone(), proposal, branch, scope, cancel)
                    .await?;
                target.solve = destination
                    .solve
                    .with_screened_start(&screened)
                    .map_err(crate::math::MathRuntimeError::from)?;
            }
            Err(error)
                if matches!(
                    error.boundary_diagnostic().class,
                    pse_model::diagnostic::BoundaryClass::Unsupported
                        | pse_model::diagnostic::BoundaryClass::Incompatible
                        | pse_model::diagnostic::BoundaryClass::Numerical
                ) => {}
            Err(error) => return Err(error),
        }
        Ok(())
    }
    async fn prediction_sample(
        &self,
        package: &super::ModelingPackage,
        point: &ScopedStudyPoint,
        seed: pse_model::generated::identities::SolutionId,
        cancel: &crate::CancelSource,
    ) -> Result<Option<super::modeling::results::PredictionSample>, WorkflowError> {
        let Some((header, portable)) = self.operations()?.prediction(seed).await? else {
            return Ok(None);
        };
        if header.run != point.run || header.step != 0 {
            return Err(contract(
                "prediction anchor escaped its selected occurrence",
            ));
        }
        let full = self
            .canonical_store()
            .canonical_study_point(&point.key)
            .await?
            .ok_or_else(|| contract("prediction source occurrence absent"))?;
        let definition = definition_of(&full)?;
        let operation = package
            .prepare_bound_operation(&definition.operation, &definition.binding, cancel)
            .await?;
        let super::PreparedStudyOperation::DeclaredCase(source) = operation else {
            return Err(contract("prediction source is not an original case"));
        };
        if source
            .solve
            .seed_preparation_identity()
            .map(|identity| identity.to_string())
            .as_deref()
            != Some(header.preparation.as_str())
        {
            return Err(contract(
                "prediction original preparation differs from selected seed",
            ));
        }
        Ok(Some(super::modeling::results::PredictionSample {
            prepared: *source,
            runtime: self.clone(),
            point: portable,
            root: Err(pse_backend_native::square_response::Withheld::Neighborhood(
                "no retained original root factor".into(),
            )),
        }))
    }
    /// Settle an effect-free shared-policy refusal under its exact immediate read set.
    pub(crate) async fn settle_study_candidate(
        &self,
        scope: &StudyScope,
        action: &pse_model::study::PointAction,
        error: Option<WorkflowError>,
    ) -> Result<CanonicalPoint, WorkflowError> {
        let diagnostic =
            error
                .map(|error| error.boundary_diagnostic())
                .or_else(|| match &action.kind {
                    pse_model::study::ActionKind::Refuse(refusal) => {
                        Some(super::study_execution::policy_refusal(refusal))
                    }
                    pse_model::study::ActionKind::Cancel => Some(
                        WorkflowError::Math(crate::math::MathRuntimeError::Cancelled)
                            .boundary_diagnostic(),
                    ),
                    _ => None,
                });
        Ok(self
            .canonical_store()
            .refuse_study_point(scope, action, diagnostic)
            .await?)
    }

    /// Explicitly recover an assigned occurrence without invoking a numerical kernel.
    /// Native expiration/cancellation authority closes the old writer before replay.
    #[allow(
        unsafe_code,
        reason = "owning recovery projects only admitted completion or truthful worker-loss facts"
    )]
    pub(crate) async fn recover_study_point(&self, key: &str) -> Result<bool, WorkflowError> {
        let point = self
            .canonical_store()
            .canonical_study_point(key)
            .await?
            .ok_or_else(|| contract("assigned study occurrence absent"))?;
        let point = &point;
        let Some(attempt_key) = point.attempt.as_deref() else {
            return Err(contract("assigned study occurrence has no attempt"));
        };
        let actual = self
            .canonical_store()
            .canonical_attempt(attempt_key)
            .await?
            .ok_or_else(|| contract("assigned attempt absent"))?;
        let study = self
            .canonical_store()
            .canonical_study(&point.study)
            .await?
            .ok_or_else(|| contract("study header absent"))?;
        let run = self
            .canonical_store()
            .canonical_run(&point.run)
            .await?
            .ok_or_else(|| contract("point run absent"))?;
        if !actual.terminal
            && !study.cancelled
            && !run.cancelled
            && actual.expires_at > chrono::Utc::now().timestamp_micros()
        {
            return Ok(false);
        }
        self.operations()?
            .recover(
                &point.run,
                &format!("study-recover:{}:{}", point.key, actual.generation),
            )
            .await?;
        let record = self.operations()?.record(&point.run, attempt_key).await?;
        let stored = record
            .completion
            .as_ref()
            .ok_or_else(|| contract("recovered completion absent"))?;
        let mut facts = point_facts(point)?;
        facts.lifecycle = match stored.state {
            AttemptState::Cancelled => StudyPointState::Cancelled,
            AttemptState::Failed => StudyPointState::Failed,
            _ => StudyPointState::Completed,
        };
        facts.retry_failure = stored.termination.retry_failure;
        facts.effect = stored.termination.effect;
        facts.scientific = stored.completion.as_ref().map_or_else(
            pse_model::study::ScientificFacts::default,
            |completion| {
                let single = match completion.assessments.as_slice() {
                    [one] => Some(one),
                    _ => None,
                };
                pse_model::study::ScientificFacts {
                    usable: !completion.assessments.is_empty()
                        && completion.assessments.iter().all(|row| row.permits_result),
                    candidate_use: single.map(|row| row.usability),
                    seed_permission: single.is_some_and(|row| row.permits_seed),
                }
            },
        );
        let start = point
            .start
            .as_ref()
            .map(|bytes| {
                serde_json::from_slice(bytes.as_slice())
                    .map_err(|error| contract(error.to_string()))
            })
            .transpose()?;
        let diagnostic = match &stored.termination.cause {
            super::TerminationCause::Error { diagnostic } => Some(diagnostic.clone()),
            _ => None,
        };
        let mut outcome = point_outcome(point)?.unwrap_or(PointOutcome {
            key: facts.key,
            lifecycle: facts.lifecycle,
            scientific: facts.scientific.clone(),
            diagnostic: None,
            start: start.clone(),
            effect: facts.effect,
            attempts: vec![],
        });
        outcome.lifecycle = facts.lifecycle;
        outcome.scientific = facts.scientific.clone();
        outcome.effect = facts.effect;
        outcome.diagnostic = diagnostic.clone();
        outcome.start = start.clone();
        outcome.attempts.push(PointAttemptOutcome {
            attempt_id: Some(record.attempt_id),
            lifecycle: Some(stored.state),
            diagnostic,
            scientific: facts.scientific.clone(),
            start,
            effect: facts.effect,
        });
        let settled = !pse_operations::study_policy::may_retry(
            &point_policy(point)?,
            &facts,
            study.cancelled,
        );
        // SAFETY: facts and outcome derive from this exact attempt's admitted durable
        // completion or truthful worker-loss receipt; shared policy alone decides retry.
        unsafe {
            self.canonical_store()
                .observe_study_point(point, &facts, &outcome, settled)
                .await
        }?;
        Ok(true)
    }

    /// Publish bounded occurrence summary blocks after every scientific occurrence
    /// settles. Reconciliation and terminal admission precede the study conclusion.
    #[allow(
        unsafe_code,
        reason = "owning study policy projects complete admitted occurrence observations"
    )]
    pub(crate) async fn finalize_canonical_study(&self, key: &str) -> Result<bool, WorkflowError> {
        use pse_model::study::{Availability, StudyLifecycle};
        use pse_operations::canonical_execution::{
            TerminalClass, result_batch_key, result_payload_digest, result_set_key,
        };
        let store = self.canonical_store();
        let study = store
            .canonical_study(key)
            .await?
            .ok_or_else(|| contract("study absent"))?;
        if study.terminal {
            return Ok(false);
        }
        let mut parent = store
            .canonical_run(&study.run)
            .await?
            .ok_or_else(|| contract("study parent run absent"))?;
        if let Some(attempt) = &parent.current_attempt {
            let actual = store
                .canonical_attempt(attempt)
                .await?
                .ok_or_else(|| contract("parent attempt absent"))?;
            if !actual.terminal {
                if actual.expires_at > chrono::Utc::now().timestamp_micros() {
                    return Ok(false);
                }
                // A summary has no scientific effects. Freeze the lost writer and retain
                // its truthful worker-loss receipt before rebuilding from settled points.
                self.operations()?
                    .recover(
                        &study.run,
                        &format!("study-summary-recover:{}", actual.generation),
                    )
                    .await?;
                parent = store
                    .canonical_run(&study.run)
                    .await?
                    .ok_or_else(|| contract("recovered parent run absent"))?;
            }
        }
        if let Some(attempt) = &parent.terminal_attempt {
            let actual = store
                .canonical_attempt(attempt)
                .await?
                .ok_or_else(|| contract("parent terminal attempt absent"))?;
            if actual.completion.as_ref().is_some_and(|bytes| {
                serde_json::from_slice::<(u8, pse_model::study::Conclusion)>(bytes.as_slice())
                    .is_ok()
            }) {
                store.conclude_study(key).await?;
                return Ok(true);
            }
            // Only the typed lost-writer completion permits rebuilding this effect-free
            // summary. An unknown payload must never become a successful conclusion.
            let recovered = self.operations()?.record(&study.run, attempt).await?;
            let lost = recovered.completion.as_ref().is_some_and(|stored| {
                stored.completion.is_none()
                    && stored.termination.retry_failure
                        == Some(pse_model::study::RetryFailure::Transient)
                    && stored.termination.effect == pse_model::study::EffectState::Absent
            });
            if !lost {
                return Err(contract(
                    "parent terminal observation does not authorize summary recovery",
                ));
            }
        }
        let operations = self.operations()?;
        let Some(fence) = store
            .begin_study_finalization(
                &StudySummary::from(&study),
                operations.worker(),
                operations.policy().lease,
            )
            .await?
        else {
            return Ok(false);
        };
        let study_id = StudyId::from_id(
            SemanticId::parse_hex(key).map_err(|error| contract(error.to_string()))?,
        );
        let relation = pse_relations::generated::runtime::study_outcomes::RELATION_ID;
        let set = result_set_key(fence.attempt(), &relation.to_string());
        let mut after = None;
        let mut ordinal = 0_u64;
        let mut offset = 0_u64;
        let mut usable = 0_u64;
        loop {
            let page = store.study_point_page(key, after).await?;
            if page.is_empty() {
                break;
            }
            let mut rows = Vec::with_capacity(page.len());
            for point in page {
                after = Some(point.ordinal);
                if !point.settled {
                    return Err(contract("study observation reopened during finalization"));
                }
                let outcome = point
                    .outcome()?
                    .ok_or_else(|| contract("settled study outcome absent"))?;
                usable += u64::from(outcome.scientific.usable);
                let full = store
                    .canonical_study_point(&point.key)
                    .await?
                    .ok_or_else(|| contract("study point absent"))?;
                rows.push(super::study_tables::outcome_row(
                    study_id,
                    &definition_of(&full)?,
                    &outcome,
                ));
            }
            let batch = super::study_tables::export::<
                pse_model::generated::runtime::study_outcomes::Row,
            >(self, rows)?;
            let columns = batch.batch().num_columns() as u64;
            let base = offset;
            offset += batch.batch().num_rows() as u64;
            super::result_blocks::visit_result_blocks_async(
                batch.batch(),
                |start, rows, payload| {
                    let current = ordinal;
                    ordinal += 1;
                    let start = base + start as u64;
                    let block = pse_model::generated::runtime::canonical_result_blocks::Row {
                        key: result_batch_key(fence.attempt(), &set, current),
                        result_set: set.clone(),
                        batch: result_batch_key(fence.attempt(), &set, current),
                        output: relation.to_string(),
                        partition: "0".into(),
                        ordinal: current,
                        start,
                        end: start + rows as u64,
                        rows: rows as u64,
                        columns,
                        coordinate_min: None,
                        coordinate_max: None,
                        payload_bytes: payload.len() as u64,
                        payload_digest: result_payload_digest(&payload),
                        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                    };
                    let fence = &fence;
                    async move {
                        store
                            .append_result_block(
                                fence,
                                &format!("study-summary:{}:{current}", fence.attempt()),
                                &relation.to_string(),
                                current,
                                &payload,
                                rows as u64,
                                &block,
                            )
                            .await?;
                        Ok::<_, WorkflowError>(())
                    }
                },
            )
            .await?;
            store
                .renew_attempt(&fence, operations.policy().lease)
                .await?;
        }
        if offset != study.point_count {
            return Err(contract("study summary coverage differs"));
        }
        let availability = if usable == study.point_count {
            Availability::Complete
        } else if usable > 0 {
            Availability::Partial
        } else {
            Availability::None
        };
        let conclusion = pse_model::study::Conclusion {
            availability,
            lifecycle: if study.cancelled {
                StudyLifecycle::Cancelled
            } else {
                StudyLifecycle::Terminal
            },
        };
        let class = if study.cancelled {
            TerminalClass::Cancelled
        } else {
            match availability {
                Availability::Complete => TerminalClass::Succeeded,
                Availability::Partial => TerminalClass::Partial,
                Availability::None => TerminalClass::Failed,
            }
        };
        let closed = store
            .close_result_ingestion(&fence, &format!("study-summary-close:{}", fence.attempt()))
            .await?;
        let manifest = store.reconcile_closed_attempt(&closed).await?;
        let operation = format!("study-summary-terminal:{}", fence.attempt());
        // SAFETY: the study owner assembled every settled occurrence with exact
        // coverage above; the manifest and conclusion describe those admitted observations.
        if let Err(error) = unsafe {
            store
                .seal_study_summary(
                    key,
                    &manifest,
                    &operation,
                    class,
                    &document(&(1_u8, conclusion))?,
                )
                .await
        } {
            let current = store
                .canonical_study(key)
                .await?
                .ok_or_else(|| contract("study summary header disappeared"))?;
            if !current.cancelled || study.cancelled {
                return Err(error.into());
            }
            let cancelled = pse_model::study::Conclusion {
                availability,
                lifecycle: StudyLifecycle::Cancelled,
            };
            // SAFETY: the same frozen complete observations remain authoritative;
            // only the conclusion changes to reflect the freshly observed cancellation.
            unsafe {
                store
                    .seal_study_summary(
                        key,
                        &manifest,
                        &operation,
                        TerminalClass::Cancelled,
                        &document(&(1_u8, cancelled))?,
                    )
                    .await
            }?;
        }
        store.conclude_study(key).await?;
        Ok(true)
    }

    /// Record completion only after the actual scientific result has drained and its
    /// exact canonical manifest was admitted.
    #[allow(
        unsafe_code,
        reason = "owning runtime projects actual admitted scientific completion, without pointer or ABI operations"
    )]
    pub(crate) async fn record_study_attempt(
        &self,
        point: &CanonicalPoint,
        start: &pse_model::study::StartProvenance,
        result: &super::RunResult,
        record: &super::DurableRecord,
    ) -> Result<CanonicalPoint, WorkflowError> {
        let terminal = record
            .attempt
            .as_ref()
            .map_err(|error| contract(format!("study result retention failed: {error}")))?;
        if point.attempt.as_deref() != Some(terminal.key.as_str()) {
            return Err(contract(
                "study observation does not name its actual canonical attempt",
            ));
        }
        let stored = record
            .completion
            .as_ref()
            .ok_or_else(|| contract("scientific completion absent"))?;
        let mut facts = point_facts(point)?;
        facts.native_started = true;
        facts.lifecycle = match stored.state {
            AttemptState::Cancelled => StudyPointState::Cancelled,
            AttemptState::Failed => StudyPointState::Failed,
            _ => StudyPointState::Completed,
        };
        facts.scientific = super::study_operations::scientific_facts(result);
        facts.retry_failure = stored.termination.retry_failure;
        facts.effect = stored.termination.effect;
        let diagnostic = super::study_execution::result_diagnostic(result);
        let attempt = PointAttemptOutcome {
            attempt_id: Some(record.attempt_id),
            lifecycle: Some(stored.state),
            diagnostic: diagnostic.clone(),
            scientific: facts.scientific.clone(),
            start: Some(start.clone()),
            effect: facts.effect,
        };
        let mut outcome = point_outcome(point)?.unwrap_or_else(|| PointOutcome {
            key: facts.key,
            lifecycle: facts.lifecycle,
            scientific: facts.scientific.clone(),
            diagnostic: diagnostic.clone(),
            start: Some(start.clone()),
            effect: facts.effect,
            attempts: vec![],
        });
        outcome.lifecycle = facts.lifecycle;
        outcome.scientific = facts.scientific.clone();
        outcome.diagnostic = diagnostic;
        outcome.start = Some(start.clone());
        outcome.effect = facts.effect;
        outcome.attempts.push(attempt);
        let policy = point_policy(point)?;
        let study = self
            .canonical_store()
            .canonical_study(&point.study)
            .await?
            .ok_or_else(|| contract("study absent"))?;
        let settled = !pse_operations::study_policy::may_retry(&policy, &facts, study.cancelled);
        // SAFETY: this owner projects the drained scientific result and its admitted
        // durable completion after checking that the point names this exact attempt.
        Ok(unsafe {
            self.canonical_store()
                .observe_study_point(point, &facts, &outcome, settled)
                .await
        }?)
    }
}

/// One canonical point's state and exact persisted result identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointStatus {
    /// Authored occurrence identity.
    pub point_index: u32,
    /// Actual occurrence lifecycle.
    pub state: StudyPointState,
    /// Retained scientific outcome, when observed.
    pub outcome: Option<PointOutcome>,
    /// Exact canonical occurrence execution.
    pub run: String,
    /// Exact assigned native attempt, when claimed.
    pub attempt: Option<String>,
    /// Shared policy no longer permits a retry.
    pub settled: bool,
}
/// Study metadata and explicit user-requested point summary; no native reports or source bundle.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyStatus {
    /// Stable requested study identity.
    pub study_id: StudyId,
    /// Actual header lifecycle.
    pub state: StudyState,
    /// Exact canonical parent execution.
    pub run: String,
    /// Admitted parent summary attempt, when available.
    pub result_attempt: Option<String>,
    /// Persistent study cancellation authority.
    pub cancelled: bool,
    /// Explicitly requested compact occurrence observations.
    pub points: Vec<PointStatus>,
}
/// Canonical parent result and exact occurrence results, available after restart.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyResults {
    /// Exact canonical parent execution.
    pub run: String,
    /// Admitted parent summary attempt.
    pub attempt: String,
    /// Occurrence identity, execution and optional native attempt.
    pub points: Vec<(u32, String, Option<String>)>,
}
/// Explicit cancellation receipt; native process drain remains the worker's obligation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCancel {
    /// Exact requested study identity.
    pub study_id: StudyId,
    /// Current native cancellation generation.
    pub generation: u64,
    /// A concluded study cannot revoke admitted history.
    pub already_concluded: bool,
}

/// A durable study's exact canonical selection.
#[derive(Clone, Debug)]
pub struct StudyHandle {
    runtime: Runtime,
    study_id: StudyId,
}
impl StudyHandle {
    /// Stable requested study identity.
    pub const fn study_id(&self) -> StudyId {
        self.study_id
    }
    /// Metadata and compact occurrence outcomes, fetched only on explicit request.
    pub async fn status(&self) -> Result<StudyStatus, WorkflowError> {
        let store = self.runtime.canonical_store();
        let study = store
            .canonical_study(&self.study_id.to_string())
            .await?
            .ok_or_else(|| contract("canonical study absent"))?;
        let run = store
            .canonical_run(&study.run)
            .await?
            .ok_or_else(|| contract("canonical study run absent"))?;
        let mut points = Vec::new();
        let mut after = None;
        loop {
            let page = store.study_point_page(&study.key, after).await?;
            if page.is_empty() {
                break;
            }
            for point in page {
                after = Some(point.ordinal);
                let facts = point.facts()?;
                points.push(PointStatus {
                    point_index: u32::try_from(point.occurrence)
                        .map_err(|_| contract("occurrence overflow"))?,
                    state: facts.lifecycle,
                    outcome: point.outcome()?,
                    run: point.run,
                    attempt: point.attempt,
                    settled: point.settled,
                });
            }
        }
        Ok(StudyStatus {
            study_id: self.study_id,
            state: if study.terminal {
                StudyState::Concluded
            } else {
                StudyState::Open
            },
            run: study.run,
            result_attempt: run.terminal_attempt,
            cancelled: study.cancelled,
            points,
        })
    }
    /// Revoke study claims and every assigned writer through one guarded header;
    /// native renewal and ingestion observe this cancellation immediately.
    pub async fn cancel(&self) -> Result<StudyCancel, WorkflowError> {
        let store = self.runtime.canonical_store();
        let key = self.study_id.to_string();
        let old = store
            .canonical_study(&key)
            .await?
            .ok_or_else(|| contract("canonical study absent"))?;
        if old.terminal {
            return Ok(StudyCancel {
                study_id: self.study_id,
                generation: old.generation,
                already_concluded: true,
            });
        }
        let study = store.cancel_study(&key).await?;
        Ok(StudyCancel {
            study_id: self.study_id,
            generation: study.generation,
            already_concluded: false,
        })
    }
    /// Sealed canonical handles; incomplete private staging is never returned as a result.
    pub async fn result(&self) -> Result<Option<StudyResults>, WorkflowError> {
        let status = self.status().await?;
        let Some(attempt) = status.result_attempt else {
            return Ok(None);
        };
        self.runtime
            .canonical_store()
            .read_results(&status.run, &attempt, Duration::from_secs(60))
            .await?;
        Ok(Some(StudyResults {
            run: status.run,
            attempt,
            points: status
                .points
                .into_iter()
                .map(|point| (point.point_index, point.run, point.attempt))
                .collect(),
        }))
    }
    /// Wait for retained finalization, with optional managed workers continuing dispatch.
    pub async fn wait(&self, poll: Duration) -> Result<StudyResults, WorkflowError> {
        if poll.is_zero() {
            return Err(contract("study poll interval must be positive"));
        }
        loop {
            if let Some(result) = self.result().await? {
                return Ok(result);
            }
            tokio::time::sleep(poll).await;
        }
    }
}

/// Authored policy choices; seed consumption is supplied only by operation admission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointPolicy {
    /// Requested occurrence identity.
    pub key: pse_model::study::OccurrenceKey,
    /// Explicit ordering and scientific-result dependencies.
    pub dependencies: Vec<pse_model::study::Dependency>,
    /// Explicit start/seed selection.
    pub start: pse_model::study::StartPolicy,
    /// Maximum tries including the first.
    pub attempt_limit: u32,
}

/// One point of a study to start.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPoint {
    /// Supported existing operation and its complete inputs.
    pub operation: OperationRequest,
    /// Existing preparation controls, serialized without a second default authority.
    pub preparation: PreparationSettings,
    /// Submitted physical assignments, admitted once before any point is scheduled.
    pub overlay: PointOverlay,
    /// Occurrence identity and explicit dependency/start policy.
    pub policy: StudyPointPolicy,
}

/// Raw request admitted once into the immutable executable definition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyRequest {
    /// Document version.
    pub version: Version<4>,
    /// Ordered occurrence requests.
    pub points: Vec<StudyPoint>,
}

/// A durable study to start.
#[derive(Clone, Debug)]
pub struct StudyPlan {
    /// Initial authored ingress; modeling sources are persisted once into a canonical revision.
    pub sources: PackageSources,
    /// The points, in index order.
    pub points: Vec<StudyPoint>,
}

const STUDY_DEFINITION_VERSION: u32 = 8;

/// Version 8 of a study's definition: the store's `definition` document and the content of
/// the study's request identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyDefinition {
    /// Document version.
    pub version: Version<STUDY_DEFINITION_VERSION>,
    /// The source bundle of the physical package.
    pub physical: super::PhysicalSource,
    /// The exact immutable canonical modeling revision.
    pub modeling_revision: String,
    /// The points, in index order.
    pub points: Vec<StudyPointDefinition>,
}

/// One point of a study's definition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointDefinition {
    /// Reconstructable admitted descriptor of one existing operation.
    pub operation: StudyOperation,
    /// The hash of the point's value bindings.
    pub binding_hash: pse_ids::roles::BindingHash,
    /// Canonical physically admitted member assignments; original paths are attribution.
    pub binding: AdmittedBinding,
    /// Shared occurrence/dependency/start policy.
    pub policy: PointPolicy,
}
impl StudyDefinition {
    /// Read a current durable definition, refusing historical contracts before their
    /// nested scientific inputs are decoded. Stored bytes are never rewritten.
    pub fn readmission(document: &str) -> Result<Self, WorkflowError> {
        #[derive(serde::Deserialize)]
        struct Header {
            version: u32,
        }
        let header: Header = serde_json::from_str(document)
            .map_err(|error| contract(format!("immutable study definition version: {error}")))?;
        if header.version != Version::<STUDY_DEFINITION_VERSION>::NUMBER {
            return Err(contract(format!(
                "study definition version {} is unsupported (current: {}); explicit readmission is required",
                header.version,
                Version::<STUDY_DEFINITION_VERSION>::NUMBER
            )));
        }
        serde_json::from_str(document)
            .map_err(|error| contract(format!("immutable study definition: {error}")))
    }

    /// Validate supported producer and consumer roles for every immutable descriptor.
    pub fn validate_roles(&self) -> Result<(), WorkflowError> {
        pse_operations::study_policy::admit(&self.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        for point in &self.points {
            match &point.policy.start {
                pse_model::study::StartPolicy::Continuation(edge) => {
                    point.operation.admit_seed_role(edge.role)?;
                    let predecessor = self
                        .points
                        .iter()
                        .find(|point| point.policy.key == edge.predecessor)
                        .ok_or_else(|| contract("missing admitted predecessor"))?;
                    predecessor.operation.admit_seed_role(edge.role)?;
                }
                pse_model::study::StartPolicy::Explicit { role, .. } => {
                    point.operation.admit_seed_role(*role)?
                }
                pse_model::study::StartPolicy::Fresh => {}
            }
        }
        Ok(())
    }
    /// Mechanically derive the shared policy graph from the one executable definition.
    pub fn graph(&self) -> OccurrenceGraph {
        OccurrenceGraph {
            points: self
                .points
                .iter()
                .map(|point| point.policy.clone())
                .collect(),
        }
    }
}

impl super::ModelingPackage {
    /// Admit exact authored physical sources through their canonical owner before
    /// deriving occurrence descriptors. Adapters never fabricate a storage receipt.
    pub async fn admit_study_sources(
        &self,
        sources: &BTreeMap<String, Vec<u8>>,
        points: &[StudyPoint],
        cancel: &crate::CancelSource,
    ) -> Result<StudyDefinition, WorkflowError> {
        let operations = Operations::from_store(
            self.runtime.canonical_store().clone(),
            Operations::process_worker("study-admission"),
            super::LeasePolicy::default(),
            self.runtime.shared.pool(),
        );
        let physical = operations.put_sources(sources).await?;
        self.admit_study_points(physical, points, cancel).await
    }
    /// Admit operations and physical bindings before either executor schedules an occurrence.
    pub async fn admit_study_points(
        &self,
        physical: super::PhysicalSource,
        points: &[StudyPoint],
        cancel: &crate::CancelSource,
    ) -> Result<StudyDefinition, WorkflowError> {
        if points.is_empty() {
            return Err(WorkflowError::Typed(
                pse_model::diagnostic::DiagnosticCause::new(
                    pse_operations::study_policy::PolicyError::Empty,
                ),
            ));
        }
        if points.len() > MAXIMUM_STUDY_POINTS {
            let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::InvalidModel,
                pse_diagnostics::DiagnosticStage::StudyAdmission,
                [],
                pse_diagnostics::DiagnosticRule::StudyPolicyAdmission,
            );
            diagnostic.observations.insert(
                "occurrences".into(),
                pse_model::diagnostic::Observation::Integer(
                    i64::try_from(points.len()).unwrap_or(i64::MAX),
                ),
            );
            diagnostic.observations.insert(
                "maximum_occurrences".into(),
                pse_model::diagnostic::Observation::Integer(MAXIMUM_STUDY_POINTS as i64),
            );
            return Err(diagnostic.into());
        }
        let mut admitted = Vec::with_capacity(points.len());
        for point in points {
            let mut operation = StudyOperation {
                version: Version,
                source: OperationSource::of(self),
                preparation: point.preparation.clone(),
                operation: point.operation.clone(),
                admitted_horizon: None,
            };
            operation.admit_horizon_values(self, cancel).await?;
            let binding = self
                .admit_operation_overlay(&operation, &point.overlay, cancel)
                .await?;
            let seed_need = operation
                .admit_binding_seed_need(self, &binding, cancel)
                .await?;
            let policy = PointPolicy {
                key: point.policy.key,
                dependencies: point.policy.dependencies.clone(),
                start: point.policy.start.clone(),
                attempt_limit: point.policy.attempt_limit,
                seed_need,
            };
            admitted.push(StudyPointDefinition {
                binding_hash: binding.identity(),
                binding,
                operation,
                policy,
            });
        }
        let definition = StudyDefinition {
            version: Version,
            physical,
            modeling_revision: self.canonical_revision().key.clone(),
            points: admitted,
        };
        pse_operations::study_policy::admit(&definition.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        definition.validate_roles()?;
        Ok(definition)
    }
}
