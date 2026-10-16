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
use std::{collections::BTreeMap, mem::size_of, sync::Arc, time::Duration};

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
    pub(crate) output: super::StudyOutputDemand,
    pub(crate) seed: Option<pse_model::study::SeedFact>,
    prediction: Option<StudyPrediction>,
}

struct StudyPrediction {
    package: super::ModelingPackage,
    predecessor: ScopedStudyPoint,
    seed: super::AdmittedSeed,
    original: super::ModelingSolvePreparation,
    operation: pse_ids::ContentHash,
    binding_members: Vec<SemanticId>,
    _binding_members_owner: Arc<pse_columnar::AllocationLease>,
}

/// Rebuild one related prediction source from the already checked structural view when
/// its only changes are admitted parameter bindings. All value-dependent numerical
/// producers and optional solve adapters remain on the complete reconstruction path.
pub(in crate::workflow) async fn rebind_prediction_source(
    package: &super::ModelingPackage,
    template: &super::ModelingSolvePreparation,
    binding: &AdmittedBinding,
    binding_members: &[SemanticId],
    portable: Arc<pse_columnar::Leased<super::modeling::results::PortablePrediction>>,
    cancel: &crate::CancelSource,
) -> Result<Option<super::modeling::results::PredictionSample>, WorkflowError> {
    let product = template.model.model.compiled();
    let can_rebind = binding.entries.len() == binding_members.len()
        && binding
            .entries
            .keys()
            .copied()
            .eq(binding_members.iter().copied())
        && binding.entries.values().all(|entry| entry.parameter)
        && template.profile.reconstruction.is_none()
        && template.profile.numerics.goals.is_empty()
        && template.profile.sensitivity.is_none()
        && template.providers.is_empty()
        && product.admitted.implicit_systems().next().is_none()
        && portable.key.numerical_policy == Some(template.solve.numerics().key);
    if !can_rebind {
        return Ok(None);
    }
    package.validate_binding(product, binding)?;

    // Charge the rebound value/start projections before their maps and coordinate view
    // are allocated. The prepared compiler product and portable point keep their own
    // owners; this lease covers only the additional source-valued wrappers below.
    let value_extent = template
        .model
        .model
        .compiled()
        .model
        .symbols
        .len()
        .max(template.model.values.scalars.len());
    let start_extent = template
        .model
        .model
        .compiled()
        .model
        .symbols
        .len()
        .max(template.starts.len());
    let scalar_node = size_of::<(SemanticId, f64)>()
        .checked_add(4 * size_of::<usize>())
        .ok_or_else(|| contract("prediction scalar metadata extent"))?;
    let start_node = size_of::<(SemanticId, super::modeling::StartSource)>()
        .checked_add(4 * size_of::<usize>())
        .ok_or_else(|| contract("prediction start metadata extent"))?;
    let start_strings = template.starts.values().try_fold(0usize, |total, start| {
        let bytes = match start {
            super::modeling::StartSource::Case { path } => path.capacity(),
            _ => 0,
        };
        total.checked_add(bytes)
    });
    let projection_bytes = value_extent
        .checked_mul(scalar_node)
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| bytes.checked_add(2 * size_of::<BTreeMap<SemanticId, f64>>()))
        .and_then(|bytes| bytes.checked_add(size_of::<pse_math::binding::CaseValues>()))
        .and_then(|bytes| {
            bytes.checked_add(size_of::<BTreeMap<SemanticId, super::modeling::StartSource>>())
        })
        .and_then(|bytes| {
            start_extent
                .checked_mul(start_node)
                .and_then(|starts| bytes.checked_add(starts))
        })
        .and_then(|bytes| start_strings.and_then(|strings| bytes.checked_add(strings)))
        .and_then(|bytes| {
            portable
                .primal
                .len()
                .checked_mul(size_of::<f64>())
                .and_then(|coordinates| bytes.checked_add(coordinates))
        })
        .and_then(|bytes| {
            template
                .model
                .tightenings
                .capacity()
                .checked_mul(size_of::<pse_modeling::DomainTightening>())
                .and_then(|tightenings| bytes.checked_add(tightenings))
        })
        .ok_or_else(|| contract("prediction source projection extent"))?;
    let projection_owner = package
        .runtime
        .shared
        .math()
        .reserve("study:prediction-source-view", projection_bytes)?;
    cancel
        .token()
        .checkpoint()
        .map_err(pse_engine::EngineError::from)?;

    // The operation descriptor and equal binding-member set establish that only the
    // admitted parameter values differ. Replace every bound parameter in the template
    // before asking the existing math owner to perform its bounded value rebind.
    let mut values = template.model.values.clone();
    for (member, entry) in &binding.entries {
        values.scalars.insert(*member, entry.canonical.into_inner());
    }
    let service = package.runtime.shared.math();
    let case = match service
        .rebind(&template.model.case, values.clone(), cancel)
        .await
    {
        Ok(case) => case,
        Err(error) if cancel.token().is_cancelled() => return Err(error.into()),
        Err(_) => return Ok(None),
    };
    let values = case.compiled().complete(&values);
    let model = crate::math::modeling::ModelingCasePreparation {
        model: template.model.model.clone(),
        case: case.clone(),
        values: values.clone(),
        tightenings: template.model.tightenings.clone(),
    };
    let preparation = service.prepare_resolved(
        case,
        values.clone(),
        template.providers.clone(),
        template.profile.clone(),
        template.solve.shared_numerics(),
        BTreeMap::new(),
    );
    tokio::pin!(preparation);
    let solve = tokio::select! {
        biased;
        () = cancel.cancelled() => return Err(crate::math::MathRuntimeError::Cancelled.into()),
        result = &mut preparation => match result {
            Ok(solve) => solve,
            Err(_) => return Ok(None),
        },
    };

    let plan_columns = model.case.compiled().plan.columns();
    if portable.coordinates.as_slice() != plan_columns {
        return Ok(None);
    }
    let primal = portable
        .primal
        .iter()
        .copied()
        .map(f64::from_bits)
        .collect::<Vec<_>>();
    if !matches!(solve.semantic_point_key(&primal), Ok(key) if key == portable.key) {
        return Ok(None);
    }
    cancel
        .token()
        .checkpoint()
        .map_err(pse_engine::EngineError::from)?;

    let mut starts = template.starts.clone();
    for member in binding.entries.keys() {
        starts.insert(
            *member,
            super::modeling::StartSource::Binding { member: *member },
        );
    }
    let prepared = super::ModelingSolvePreparation {
        original_inputs: template.original_inputs.clone(),
        stored_seed_owner: None,
        bound_source: Some(package.retain_bound_source(binding)?),
        model,
        solve,
        starts,
        providers: template.providers.clone(),
        source: package.clone(),
        compiler: template.compiler,
        profile: template.profile.clone(),
    };
    Ok(Some(super::modeling::results::PredictionSample {
        prepared,
        runtime: package.runtime.clone(),
        point: portable,
        root: Err(pse_backend_native::square_response::Withheld::Neighborhood(
            "no retained original root factor".into(),
        )),
        _projection_owner: Some(projection_owner),
    }))
}

pub(super) fn creation_checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    cancel
        .token()
        .checkpoint()
        .map_err(pse_engine::EngineError::from)?;
    Ok(())
}

// Only input-only acquisition may be interrupted this way. Publication and
// protection owners settle their issued effects through creation_effect instead.
pub(in crate::workflow) async fn creation_input<T, E: Into<WorkflowError>>(
    cancel: &crate::CancelSource,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, WorkflowError> {
    creation_checkpoint(cancel)?;
    tokio::select! {
        biased;
        () = cancel.cancelled() => {
            creation_checkpoint(cancel)?;
            Err(contract("creation cancellation wake without cancellation"))
        }
        result = work => {
            creation_checkpoint(cancel)?;
            result.map_err(Into::into)
        }
    }
}

// Publication and source-protection owners perform issued effects before this
// checkpoint. Their futures must not be dropped when the caller asks to stop.
pub(in crate::workflow) async fn creation_effect<T, E: Into<WorkflowError>>(
    cancel: &crate::CancelSource,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, WorkflowError> {
    creation_checkpoint(cancel)?;
    let result = work.await.map_err(Into::into)?;
    creation_checkpoint(cancel)?;
    Ok(result)
}

fn document(value: &impl serde::Serialize) -> Result<Vec<u8>, WorkflowError> {
    serde_json::to_vec(value).map_err(|error| contract(error.to_string()))
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StudyMetadata {
    version: Version<2>,
    physical: super::PhysicalSource,
    variants: Option<super::LazyStudyDefinition>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct VariantOccurrence {
    variant: usize,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StudyAdmission {
    version: Version<1>,
    definition: Option<StudyPointDefinition>,
    revision: Option<String>,
    diagnostic: Option<pse_model::diagnostic::BoundaryDiagnostic>,
}
type StoredPoint = (StudyOperation, pse_ids::roles::BindingHash, AdmittedBinding);
fn definition_of(point: &CanonicalPoint) -> Result<StudyPointDefinition, WorkflowError> {
    let policy = point_policy(point)?;
    let definition = optional_definition(
        point.descriptor.as_slice(),
        point.admission.as_ref(),
        policy,
    )?
    .ok_or_else(|| contract("selected scientific admission absent or refused"))?;
    if point_facts(point)?
        .admitted_seed_need
        .is_some_and(|need| need != definition.policy.seed_need)
    {
        return Err(contract("stored scientific admission capability differs"));
    }
    Ok(definition)
}
fn definition_from_parts(
    descriptor: &[u8],
    policy: PointPolicy,
) -> Result<StudyPointDefinition, WorkflowError> {
    let (operation, binding_hash, binding): StoredPoint = serde_json::from_slice(descriptor)
        .map_err(|error| contract(format!("immutable point descriptor: {error}")))?;
    if binding.identity() != binding_hash {
        return Err(contract("stored binding content differs"));
    }
    Ok(StudyPointDefinition {
        operation,
        binding_hash,
        binding,
        policy,
    })
}

pub(in crate::workflow) fn optional_definition(
    descriptor: &[u8],
    admission: Option<&pse_model::Bytes>,
    mut policy: PointPolicy,
) -> Result<Option<StudyPointDefinition>, WorkflowError> {
    if policy.seed_need != pse_model::study::SeedNeed::Pending {
        if admission.is_some() {
            return Err(contract("explicit study point has a lazy admission"));
        }
        return definition_from_parts(descriptor, policy).map(Some);
    }
    let Some(bytes) = admission else {
        return Ok(None);
    };
    let admitted: StudyAdmission =
        serde_json::from_slice(bytes.as_slice()).map_err(|e| contract(e.to_string()))?;
    match (admitted.definition, admitted.revision, admitted.diagnostic) {
        (Some(definition), Some(_), None) => {
            policy.seed_need = definition.policy.seed_need;
            if policy.seed_need == pse_model::study::SeedNeed::Pending
                || policy != definition.policy
                || definition.binding_hash != definition.binding.identity()
            {
                return Err(contract(
                    "immutable selected science differs from occurrence policy or bindings",
                ));
            }
            Ok(Some(definition))
        }
        (None, None, Some(_)) => Ok(None),
        _ => Err(contract("incomplete selected scientific admission")),
    }
}

fn stored_scientific_facts(
    stored: &super::durable::StoredCompletion,
) -> pse_model::study::ScientificFacts {
    stored.completion.as_ref().map_or_else(
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
    )
}
fn stored_diagnostic(
    stored: &super::durable::StoredCompletion,
) -> Option<pse_model::diagnostic::BoundaryDiagnostic> {
    match &stored.termination.cause {
        super::TerminationCause::Error { diagnostic }
        | super::TerminationCause::Infrastructure { diagnostic } => Some(diagnostic.clone()),
        super::TerminationCause::Assessment { diagnostics, .. } => match diagnostics.as_slice() {
            [] => None,
            [diagnostic] => Some(diagnostic.clone()),
            diagnostics => {
                let mut envelope = pse_model::diagnostic::BoundaryDiagnostic::new(
                    pse_model::diagnostic::BoundaryClass::Conflict,
                    pse_model::diagnostic::DiagnosticStage::StudyPolicy,
                    [],
                    pse_model::diagnostic::DiagnosticRule::DiagnosticAggregate,
                );
                envelope.causes = diagnostics.to_vec();
                Some(envelope)
            }
        },
    }
}

impl Runtime {
    /// Actual recording/read owner, independent of default scientific execution durability.
    pub(crate) fn operations(&self) -> Result<&Operations, WorkflowError> {
        self.recording_operations
            .as_deref()
            .ok_or(WorkflowError::RecordingUnavailable)
    }

    /// Publish the exact retained scientific inputs, without repeating admission or
    /// expanding lazy variants. Missing byte owners or sinks refuse before effects.
    pub async fn record_study(
        &self,
        local: &LocalStudy,
        cancel: &crate::CancelSource,
    ) -> Result<StudyDefinition, WorkflowError> {
        let documents = local.package.physical.documents.as_ref().ok_or_else(|| {
            contract("study recording requires exact admitted physical document bytes")
        })?;
        let operations = self.operations()?;
        self.canonical_store()?;
        creation_checkpoint(cancel)?;
        local.science.validate_roles()?;
        let extent = documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .try_fold(0usize, |total, document| {
                total
                    .checked_add(document.bytes().len())
                    .and_then(|n| n.checked_add(document.path.len() + 256))
            })
            .ok_or_else(|| contract("study physical document extent"))?;
        let _owner = self
            .shared
            .math()
            .reserve("study:physical-recording", extent)?;
        let mut sources = BTreeMap::new();
        for document in documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
        {
            if sources
                .insert(document.path.clone(), document.bytes().to_vec())
                .is_some()
            {
                return Err(contract("study physical document path is ambiguous"));
            }
        }
        if sources.is_empty() {
            return Err(contract("study recording requires physical document bytes"));
        }
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let revision = local.package.record_study_source(self, cancel).await?;
                let physical = creation_effect(cancel, operations.put_sources(&sources)).await?;
                let mut science = local.science.clone();
                science.modeling_revision = revision;
                if let Some(lazy) = &mut science.variants {
                    lazy.variants.retain_parent(&science.modeling_revision);
                }
                Ok(StudyDefinition {
                    version: Version,
                    physical,
                    modeling_revision: science.modeling_revision,
                    variants: science.variants,
                    points: science.points,
                })
            })
            .await
    }

    fn study_creation_operations(&self) -> Result<&Operations, WorkflowError> {
        match self.durability() {
            super::Durability::Durable(operations) => Ok(operations),
            super::Durability::Ephemeral => Err(contract(
                "durable study creation requires explicit durable execution",
            )),
        }
    }
    /// Admit authored ingress once and retain one canonical study before preparing ready occurrences.
    pub async fn start_study(
        &self,
        plan: StudyPlan,
        cancel: &crate::CancelSource,
    ) -> Result<StudyHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                self.study_creation_operations()?;
                let context = self
                    .physical_from_sources(&plan.sources.physical, cancel)
                    .await?;
                let package = creation_effect(
                    cancel,
                    self.package_from_sources(&plan.sources.modeling, context, cancel),
                )
                .await?;
                let definition = creation_effect(
                    cancel,
                    package.admit_study(&plan.inputs, &plan.output, cancel),
                )
                .await?;
                let definition = self.record_study(&definition, cancel).await?;
                self.start_recorded_study(definition, cancel).await
            })
            .await
    }

    /// Persist the exact admitted immutable definition in bounded occurrence batches.
    pub async fn start_recorded_study(
        &self,
        definition: StudyDefinition,
        cancel: &crate::CancelSource,
    ) -> Result<StudyHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                creation_checkpoint(cancel)?;
                let operations = self.study_creation_operations()?;
                definition.validate_roles()?;
                if definition.point_count() == 0 || definition.point_count() > MAXIMUM_STUDY_POINTS
                {
                    return Err(contract("bounded admitted study extent"));
                }
                let revision = creation_input(
                    cancel,
                    self.canonical_store()?
                        .revision(&definition.modeling_revision),
                )
                .await?
                .ok_or_else(|| contract("canonical study revision absent"))?;
                let physical = definition.physical.clone();
                let context =
                    creation_effect(cancel, self.physical_source(&physical, cancel)).await?;
                let package = creation_effect(
                    cancel,
                    self.modeling_revision(revision.clone(), context, BTreeMap::new()),
                )
                .await?;
                // Binding/role admission reuses structure. Explicit starts also validate
                // compatibility against one prepared target before study activation.
                let study_id: StudyId = pse_operations::mint_id();
                let key = study_id.to_string();
                let mut admitted_seeds = BTreeMap::new();
                let mut seed_lane = super::study_operations::StudyPreparationLane::new();
                let mut seed_preparation: Option<(
                    super::study_operations::StudyBindingKey,
                    super::ModelingSolvePreparation,
                )> = None;
                let mut admission = None;
                for point in &definition.points {
                    creation_checkpoint(cancel)?;
                    if point.binding.identity() != point.binding_hash {
                        return Err(contract("study binding content differs"));
                    }
                    if creation_effect(
                        cancel,
                        point.operation.admit_binding_seed_need(
                            &package,
                            &point.binding,
                            &mut admission,
                            cancel,
                        ),
                    )
                    .await?
                        != point.policy.seed_need
                    {
                        return Err(contract(
                            "study seed consumption differs from its scientific owner",
                        ));
                    }
                    if let pse_model::study::StartPolicy::Explicit { seed, .. } =
                        &point.policy.start
                    {
                        if !admitted_seeds.contains_key(seed) {
                            let selected = creation_input(cancel, operations.seed(*seed)).await?;
                            admitted_seeds.insert(*seed, selected);
                        }
                        let target_key =
                            super::study_operations::DeclaredStudyAdmission::binding_key(
                                &point.operation,
                                &point.binding,
                            )?;
                        if !seed_preparation
                            .as_ref()
                            .is_some_and(|(key, _)| key == &target_key)
                        {
                            let target = creation_effect(
                                cancel,
                                seed_lane.prepare(
                                    &package,
                                    &point.operation,
                                    &point.binding,
                                    cancel,
                                ),
                            )
                            .await?;
                            let super::PreparedStudyOperation::DeclaredCase(target) = target else {
                                return Err(contract(
                                    "explicit study seed requires a declared scientific case",
                                ));
                            };
                            seed_preparation = Some((target_key, *target));
                        }
                        let target = &seed_preparation
                            .as_ref()
                            .ok_or_else(|| contract("study seed target absent"))?
                            .1;
                        target.clone().with_admitted_start(
                            admitted_seeds
                                .get(seed)
                                .ok_or_else(|| contract("study explicit seed absent"))?,
                        )?;
                    }
                }
                if let Some(lazy) = &definition.variants {
                    lazy.variants.source.check(&package)?;
                    for selected in &lazy.variants.selected {
                        if let pse_model::study::StartPolicy::Explicit { seed, .. } =
                            &selected.point.policy.start
                            && !admitted_seeds.contains_key(seed)
                        {
                            let selected_seed =
                                creation_input(cancel, operations.seed(*seed)).await?;
                            admitted_seeds.insert(*seed, selected_seed);
                        }
                    }
                }
                drop(seed_preparation);
                seed_lane.close().await?;
                drop(admission);
                creation_checkpoint(cancel)?;
                let run_id: RunId = pse_operations::mint_id();
                let run = format!("run:{run_id}");
                let physical_revision =
                    creation_input(cancel, self.canonical_store()?.revision(&physical.revision))
                        .await?
                        .ok_or_else(|| contract("canonical physical source absent"))?;
                let attestation = self.canonical()?.attestation();
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
                    version: Version::<2>,
                    physical: physical.clone(),
                    variants: definition.variants.clone(),
                })?;
                let graph = definition.graph();
                let points = graph
                    .points
                    .iter()
                    .enumerate()
                    .map(|(index, policy)| {
                        creation_checkpoint(cancel)?;
                        let point_run: RunId = pse_operations::mint_id();
                        let descriptor = match &definition.variants {
                            Some(_) => document(&VariantOccurrence { variant: index })?,
                            None => {
                                let point = &definition.points[index];
                                document(&(&point.operation, point.binding_hash, &point.binding))?
                            }
                        };
                        Ok(NewOccurrence {
                            policy: policy.clone(),
                            descriptor: descriptor.clone(),
                            run: StoredRequest {
                                key: format!("run:{point_run}"),
                                revision: revision.clone(),
                                sources: vec![physical_revision.clone()],
                                request: document(&(
                                    2_u8,
                                    point_run,
                                    study_id,
                                    policy.key,
                                    pse_ids::document::of(
                                        pse_ids::Frame::DurableStudyRequestV5,
                                        &(&descriptor, policy),
                                    )
                                    .map_err(|e| contract(e.to_string()))?,
                                ))?,
                                source_selection: source_selection.clone(),
                                attestation: attestation.clone(),
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, WorkflowError>>()?;
                let creation = async {
                    for solution in admitted_seeds.keys() {
                        creation_checkpoint(cancel)?;
                        let artifact = creation_effect(
                            cancel,
                            operations.promote_seed(*solution, "study", &key),
                        )
                        .await?;
                        creation_effect(
                            cancel,
                            self.canonical_store()?.retain_study_input(
                                &key,
                                &solution.to_string(),
                                &artifact.key,
                            ),
                        )
                        .await?;
                    }
                    let activated = self
                        .canonical_store()?
                        .create_study(&key, &request, &metadata, &points, &|| {
                            cancel.token().is_cancelled()
                        })
                        .await?;
                    if activated.is_none() {
                        creation_checkpoint(cancel)?;
                        return Err(contract("inactive study creation was interrupted"));
                    }
                    Ok::<(), WorkflowError>(())
                }
                .await;
                if let Err(source) = creation {
                    // Issued effects are awaited. Absence is proved under the same
                    // study guard before permanent abandonment; a header is never
                    // inferred inactive or fabricated terminal by this route.
                    loop {
                        match self.canonical_store()?.abandon_study_inputs(&key).await {
                            Ok(true) => return Err(source),
                            Ok(false) => continue,
                            Err(cleanup) => {
                                return Err(WorkflowError::StudyCreationUnsettled {
                                    study: study_id,
                                    source: Box::new(source),
                                    cleanup: Box::new(cleanup),
                                });
                            }
                        }
                    }
                }
                // Catch cancellation after the owner's final observation and before handing
                // an active handle to the caller. Never interrupt the durable cancel write.
                if cancel.token().is_cancelled() {
                    self.canonical_store()?.cancel_study(&key).await?;
                }
                Ok(self.study(study_id))
            })
            .await
    }

    /// Exact handle; equal bindings never merge study identities or occurrences.
    pub fn study(&self, study_id: StudyId) -> StudyHandle {
        StudyHandle {
            runtime: self.clone(),
            study_id,
        }
    }

    pub(in crate::workflow) async fn retained_study_admissions(
        &self,
        study: &str,
    ) -> Result<BTreeMap<usize, StudyPointDefinition>, WorkflowError> {
        let mut result = BTreeMap::new();
        let mut after = None;
        loop {
            let page = self
                .canonical_store()?
                .study_outcome_page(study, after)
                .await?;
            if page.is_empty() {
                break;
            }
            for point in page {
                after = Some(point.ordinal);
                let policy = point.policy()?;
                if policy.seed_need != pse_model::study::SeedNeed::Pending {
                    continue;
                }
                if let Some(admission) = optional_definition(
                    point.descriptor.as_slice(),
                    point.admission.as_ref(),
                    policy,
                )? {
                    result.insert(point.ordinal as usize, admission);
                }
            }
        }
        Ok(result)
    }
    /// Resolve a selected lazy occurrence under its separate execution lease. Publication
    /// acknowledgements are retained before the immutable point admission is exposed.
    pub(in crate::workflow) async fn admit_study_candidate(
        &self,
        scope: &StudyScope,
        lane: &mut StudyCandidateLane,
        cancel: &crate::CancelSource,
    ) -> Result<bool, WorkflowError> {
        let operations = self.operations()?;
        let Some(fence) = self
            .canonical_store()?
            .claim_study_admission(scope, operations.worker(), operations.policy().lease)
            .await?
        else {
            return Ok(false);
        };
        let child = cancel.child_with_deadline(None);
        let work = self.resolve_study_candidate_admission(scope, &fence, lane, &child);
        tokio::pin!(work);
        let renewal = (operations.policy().lease / 3).max(Duration::from_millis(1));
        let mut renewal_error = None;
        let result = loop {
            tokio::select! {
                result = &mut work => break result,
                () = tokio::time::sleep(renewal), if renewal_error.is_none() => {
                    if let Err(error) = self.canonical_store()?.renew_attempt(&fence, operations.policy().lease).await {
                        child.cancel(); renewal_error = Some(error);
                    }
                }
            }
        };
        if let Some(error) = renewal_error {
            return Err(error.into());
        }
        result
    }
    async fn resolve_study_candidate_admission(
        &self,
        scope: &StudyScope,
        fence: &pse_operations::canonical_execution::AttemptFence,
        lane: &mut StudyCandidateLane,
        cancel: &crate::CancelSource,
    ) -> Result<bool, WorkflowError> {
        let store = self.canonical_store()?;
        let point = store
            .canonical_study_point(&scope.point().key)
            .await?
            .ok_or_else(|| contract("lazy occurrence absent"))?;
        if point.admission.is_some() || point.settled {
            return Ok(false);
        }
        let selected: VariantOccurrence = serde_json::from_slice(point.descriptor.as_slice())
            .map_err(|e| contract(e.to_string()))?;
        if selected.variant as u64 != point.ordinal {
            return Err(contract("lazy selected occurrence ordinal differs"));
        }
        if !lane
            .input
            .as_ref()
            .is_some_and(|input| input.study == point.study)
        {
            let study = store
                .canonical_study(&point.study)
                .await?
                .ok_or_else(|| contract("lazy study absent"))?;
            let extent = study
                .metadata
                .len()
                .checked_mul(4)
                .and_then(|n| n.checked_add(4096))
                .filter(|_| study.metadata.len() <= pse_operations::canonical::PAYLOAD_BYTES)
                .ok_or_else(|| contract("lazy descriptor extent"))?;
            let owner = self
                .shared
                .math()
                .reserve("study:lazy-input-owner", extent)?;
            let metadata: StudyMetadata = serde_json::from_slice(study.metadata.as_slice())
                .map_err(|e| contract(e.to_string()))?;
            let lazy = metadata
                .variants
                .ok_or_else(|| contract("lazy descriptor absent"))?;
            lazy.variants.check_retained_parent(&study.revision)?;
            let physical = self.physical_source(&metadata.physical, cancel).await?;
            let base_revision = store
                .revision(&study.revision)
                .await?
                .ok_or_else(|| contract("lazy base revision absent"))?;
            let package = self
                .modeling_revision(base_revision, physical, BTreeMap::new())
                .await?;
            lazy.variants.source.check(&package)?;
            lane.input = Some(StudyVariantOwner {
                study: point.study.clone(),
                lazy,
                physical: metadata.physical,
                package,
                _owner: owner,
            });
        }
        let input = lane
            .input
            .as_ref()
            .ok_or_else(|| contract("lazy input owner absent"))?;
        if input.lazy.variants.selected_policy(selected.variant)? != point_policy(&point)? {
            return Err(contract(
                "lazy selected policy differs from the immutable occurrence",
            ));
        }
        let authority =
            pse_operations::canonical_studies::StudyAdmissionAuthority::new(scope, fence)?;
        let admitted = input
            .lazy
            .admit_selected(&input.package, selected.variant, Some(&authority), cancel)
            .await;
        let (admission, seed_need, diagnostic) = match admitted {
            Ok((package, definition)) => {
                let revision = package.recorded_revision()?.clone();
                let physical_revision = store
                    .revision(&input.physical.revision)
                    .await?
                    .ok_or_else(|| contract("lazy physical receipt absent"))?;
                let admitted = StudyAdmission {
                    version: Version,
                    definition: Some(definition.clone()),
                    revision: Some(revision.key.clone()),
                    diagnostic: None,
                };
                let input_run = store
                    .canonical_run(fence.run())
                    .await?
                    .ok_or_else(|| contract("lazy admission run absent"))?;
                let request = StoredRequest {
                    key: point.run.clone(),
                    revision,
                    sources: vec![physical_revision],
                    request: document(&(
                        2_u8,
                        &point.run,
                        &point.study,
                        definition.policy.key,
                        pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &definition)
                            .map_err(|e| contract(e.to_string()))?,
                    ))?,
                    source_selection: document(&(2_u8, &admitted.revision, &input.physical))?,
                    attestation: input_run.attestation.as_slice().to_vec(),
                };
                creation_effect(
                    cancel,
                    store.begin_run_with_study_admission(&request, &authority),
                )
                .await?;
                (admitted, Some(definition.policy.seed_need), None)
            }
            Err(error) => {
                let diagnostic = error.boundary_diagnostic();
                if matches!(
                    diagnostic.class,
                    pse_model::diagnostic::BoundaryClass::Infrastructure
                        | pse_model::diagnostic::BoundaryClass::Internal
                        | pse_model::diagnostic::BoundaryClass::Cancelled
                ) {
                    return Err(error);
                }
                (
                    StudyAdmission {
                        version: Version,
                        definition: None,
                        revision: None,
                        diagnostic: Some(diagnostic.clone()),
                    },
                    None,
                    Some(diagnostic),
                )
            }
        };
        let admission = document(&admission)?;
        creation_effect(
            cancel,
            store.complete_study_admission(scope, fence, &admission, seed_need, diagnostic),
        )
        .await?;
        Ok(true)
    }

    /// Prepare one ready point using its explicit drained lane's source owner.
    pub(in crate::workflow) async fn prepare_study_candidate_in_lane(
        &self,
        scope: &StudyScope,
        lane: &mut StudyCandidateLane,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedStudyCandidate, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                use pse_model::study::{ActionKind, SeedAvailability, SeedFact, StartPolicy};
                let point = self
                    .canonical_store()?
                    .canonical_study_point(&scope.point().key)
                    .await?
                    .ok_or_else(|| contract("study occurrence absent"))?;
                let definition = definition_of(&point)?;
                let selected_revision = point
                    .admission
                    .as_ref()
                    .map(|bytes| {
                        serde_json::from_slice::<StudyAdmission>(bytes.as_slice())
                            .map_err(|e| contract(e.to_string()))
                    })
                    .transpose()?
                    .and_then(|admission| admission.revision);
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
                    .canonical_store()?
                    .canonical_study(&point.study)
                    .await?
                    .ok_or_else(|| contract("study header absent"))?;
                let StudyMetadata { physical, .. } =
                    serde_json::from_slice(study.metadata.as_slice())
                        .map_err(|error| contract(format!("study interpretation: {error}")))?;
                let actual_revision = selected_revision.as_deref().unwrap_or(&study.revision);
                let same_source = lane.source.as_ref().is_some_and(|(revision, receipt, _)| {
                    revision == actual_revision && receipt == &physical
                });
                if !same_source {
                    lane.preparation.close().await?;
                    lane.source = None;
                    let context = self.physical_source(&physical, cancel).await?;
                    let revision = self
                        .canonical_store()?
                        .revision(actual_revision)
                        .await?
                        .ok_or_else(|| contract("study revision absent"))?;
                    let package = self
                        .modeling_revision(revision, context, BTreeMap::new())
                        .await?;
                    lane.source = Some((actual_revision.into(), physical, package));
                }
                let package = lane
                    .source
                    .as_ref()
                    .ok_or_else(|| contract("study lane source absent"))?
                    .2
                    .clone();
                let mut operation = lane
                    .preparation
                    .prepare(&package, &definition.operation, &definition.binding, cancel)
                    .await?;
                if operation.seed_need() != definition.policy.seed_need {
                    return Err(contract(
                        "ready native seed consumption differs from admission",
                    ));
                }
                let original_prediction_eligible = matches!(&operation,
                    super::PreparedStudyOperation::DeclaredCase(case)
                        if case
                            .solve
                            .composition_request()
                            .recovery
                            .contains(&pse_model::strategy::StartOrigin::Predicted)
                            && case.solve.numerical_strategy().start.policy
                                != pse_backend_native::solve::StartPolicy::Explicit
                );
                let operation_descriptor = super::study_operations::DeclaredStudyAdmission::key(
                    &definition.operation,
                    &PointOverlay::default(),
                )?
                .0;
                let mut prediction = None;
                let seed = match &definition.policy.start {
                    StartPolicy::Fresh => None,
                    start => {
                        let (role, selected) = match start {
                            StartPolicy::Explicit { role, seed } => {
                                let retained = self
                                    .canonical_store()?
                                    .study_input(&point.study, &seed.to_string())
                                    .await?;
                                let selected =
                                    match self.operations()?.artifact_seed(&retained).await {
                                        Ok(admitted) => Some(admitted),
                                        Err(WorkflowError::SeedRead(
                                            super::durable::SeedReadError::Missing { .. },
                                        )) => None,
                                        Err(error) => return Err(error),
                                    };
                                (*role, selected)
                            }
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
                                match case.as_ref().clone().with_admitted_start(&selected) {
                            Ok(seeded) => {
                                let unseeded = match std::mem::replace(
                                    &mut operation,
                                    super::PreparedStudyOperation::DeclaredCase(Box::new(seeded)),
                                ) {
                                    super::PreparedStudyOperation::DeclaredCase(case) => *case,
                                    _ => return Err(contract("seeded study case changed owner")),
                                };
                                if let StartPolicy::Continuation(edge) = start
                                    && original_prediction_eligible
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
                                    let member_bytes = definition
                                        .binding
                                        .entries
                                        .len()
                                        .checked_mul(size_of::<SemanticId>())
                                        .and_then(|bytes| {
                                            bytes.checked_add(size_of::<Vec<SemanticId>>())
                                        })
                                        .ok_or_else(|| {
                                            contract("study prediction member extent")
                                        })?;
                                    let binding_members_owner = self
                                        .shared
                                        .math()
                                        .reserve("study:prediction-members", member_bytes)?;
                                    let binding_members = definition
                                        .binding
                                        .entries
                                        .keys()
                                        .copied()
                                        .collect::<Vec<_>>();
                                    prediction = Some(StudyPrediction {
                                        package: package.clone(),
                                        predecessor: predecessor.clone(),
                                        seed: selected.clone(),
                                        original: unseeded,
                                        operation: operation_descriptor,
                                        binding_members,
                                        _binding_members_owner: binding_members_owner,
                                    });
                                }
                                SeedAvailability::Compatible {
                                    seed: selected.solution(),
                                }
                            }
                            Err(WorkflowError::SeedRead(
                                super::durable::SeedReadError::Missing { .. },
                            )) => SeedAvailability::Absent,
                            Err(error)
                                if error.boundary_diagnostic().class
                                    == pse_model::diagnostic::BoundaryClass::Incompatible =>
                            {
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
                    output: definition.operation.output,
                    seed,
                    prediction,
                })
            })
            .await
    }

    /// Numerical proposal selection and original screening run only after native claim.
    pub(crate) async fn apply_study_prediction(
        &self,
        prepared: &mut PreparedStudyCandidate,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let Some(input) = prepared.prediction.take() else {
                    return Ok(());
                };
                let super::PreparedStudyOperation::DeclaredCase(target) = &mut prepared.operation
                else {
                    return Err(contract("prediction target ceased to be an original case"));
                };
                let source = self
                    .prediction_sample(
                        &input.package,
                        &input.predecessor,
                        &input.seed,
                        &input,
                        cancel,
                    )
                    .await?;
                let Some(mut source) = source else {
                    return Ok(());
                };
                let deadline = std::time::Instant::now()
                    .checked_add(input.original.solve.time_limit())
                    .ok_or_else(|| contract("prediction deadline extent"))?;
                let scope = input.original.solve.task_scope().unwrap_or_else(|| {
                    pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline))
                });
                let original = self
                    .native()
                    .admit_proposal_task(input.original.solve.clone(), scope.clone())
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
                            pse_backend_native::square_response::Withheld::Cause(Arc::new(
                                error.into_problem(),
                            ))
                        });
                }
                let older = match input
                    .predecessor
                    .start
                    .as_ref()
                    .map(|bytes| {
                        serde_json::from_slice::<pse_model::study::StartProvenance>(
                            bytes.as_slice(),
                        )
                        .map_err(|error| contract(error.to_string()))
                    })
                    .transpose()?
                {
                    Some(pse_model::study::StartProvenance::Continuation {
                        predecessor,
                        seed,
                        ..
                    }) => {
                        let key = pse_operations::canonical_studies::point_key(
                            &input.predecessor.study,
                            predecessor,
                        );
                        let point = self
                            .canonical_store()?
                            .canonical_study_point(&key)
                            .await?
                            .ok_or_else(|| {
                                contract("selected older prediction occurrence absent")
                            })?;
                        let admitted = self.operations()?.seed(seed).await?;
                        self.prediction_sample(
                            &input.package,
                            &ScopedStudyPoint::from(&point),
                            &admitted,
                            &input,
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
                    .map(|owner| -> Arc<dyn pse_backend_native::solve::WorkAdmission> { owner });
                let mut destination = input.original.clone();
                destination.solve = original;
                let branch = destination.solve.composition_request().branch;
                match source.available_prediction(older.as_ref(), &destination, branch, &execution)
                {
                    Ok(proposal) => {
                        let screened = self
                            .native()
                            .screen_start(
                                destination.solve.clone(),
                                proposal,
                                branch,
                                scope,
                                cancel,
                            )
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
            })
            .await
    }
    async fn prediction_sample(
        &self,
        package: &super::ModelingPackage,
        point: &ScopedStudyPoint,
        seed: &super::AdmittedSeed,
        template: &StudyPrediction,
        cancel: &crate::CancelSource,
    ) -> Result<Option<super::modeling::results::PredictionSample>, WorkflowError> {
        let header = seed.header();
        let Some(portable) = seed.prediction.clone() else {
            return Ok(None);
        };
        if header.run != point.run
            || point.attempt.as_deref() != Some(header.attempt.as_str())
            || header.step != 0
        {
            return Err(contract(
                "prediction anchor escaped its selected occurrence",
            ));
        }
        let full = self
            .canonical_store()?
            .canonical_study_point(&point.key)
            .await?
            .ok_or_else(|| contract("prediction source occurrence absent"))?;
        let definition = definition_of(&full)?;
        definition.operation.source.check(package)?;
        let descriptor = super::study_operations::DeclaredStudyAdmission::key(
            &definition.operation,
            &PointOverlay::default(),
        )?
        .0;
        if descriptor == template.operation
            && let Some(sample) = rebind_prediction_source(
                package,
                &template.original,
                &definition.binding,
                &template.binding_members,
                portable.clone(),
                cancel,
            )
            .await?
            && sample
                .prepared
                .solve
                .seed_preparation_identity()
                .map(|identity| identity.to_string())
                .as_deref()
                == Some(header.preparation.as_str())
        {
            return Ok(Some(sample));
        }
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
            _projection_owner: None,
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
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
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
                    .canonical_store()?
                    .refuse_study_point(scope, action, diagnostic)
                    .await?)
            })
            .await
    }

    /// Explicitly recover an assigned occurrence without invoking a numerical kernel.
    /// Expiration fences departed writers; cancellation alone is not native drain.
    #[allow(
        unsafe_code,
        reason = "owning recovery projects only admitted completion or truthful worker-loss facts"
    )]
    pub(crate) async fn recover_study_point(&self, key: &str) -> Result<bool, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let (point, actual, study, _run) =
                    self.canonical_store()?.recovery_snapshot(key).await?;
                let point = &point;
                // Assigned-page discovery is only a hint. The original owner may have
                // recorded this terminal attempt before this fresh snapshot, including
                // a retryable observation that normal claim policy must handle next.
                if !point.assigned || point.settled {
                    return Ok(false);
                }
                let attempt_key = point
                    .attempt
                    .as_deref()
                    .ok_or_else(|| contract("assigned study occurrence has no attempt"))?;
                // A live owner receives typed cancellation, drains, and settles its own
                // original completion. An independent recovery worker cannot infer that
                // physical handoff from the cancellation bit alone.
                if !actual.terminal && actual.expires_at > chrono::Utc::now().timestamp_micros() {
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
                facts.scientific = stored_scientific_facts(stored);
                let start = point
                    .start
                    .as_ref()
                    .map(|bytes| {
                        serde_json::from_slice(bytes.as_slice())
                            .map_err(|error| contract(error.to_string()))
                    })
                    .transpose()?;
                let diagnostic = stored_diagnostic(stored);
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
                let settled = !pse_model::study_policy::may_retry(
                    &point_policy(point)?,
                    &facts,
                    study.cancelled,
                );
                // SAFETY: facts and outcome derive from this exact attempt's admitted durable
                // completion or truthful worker-loss receipt; shared policy alone decides retry.
                unsafe {
                    self.canonical_store()?
                        .observe_study_point(point, &facts, &outcome, settled)
                        .await
                }?;
                Ok(true)
            })
            .await
    }

    /// Publish bounded occurrence summary blocks after every scientific occurrence
    /// settles. Reconciliation and terminal admission precede the study conclusion.
    #[allow(
        unsafe_code,
        reason = "owning study policy projects complete admitted occurrence observations"
    )]
    pub(crate) async fn finalize_canonical_study(&self, key: &str) -> Result<bool, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                use pse_model::study::{Availability, StudyLifecycle};
                use pse_operations::canonical_execution::{
                    TerminalClass, result_batch_key, result_set_key,
                };
                let store = self.canonical_store()?;
                let study = store
                    .canonical_study(key)
                    .await?
                    .ok_or_else(|| contract("study absent"))?;
                if study.terminal {
                    return Ok(false);
                }
                if store.study_has_pending(key).await? {
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
                        serde_json::from_slice::<(u8, pse_model::study::Conclusion)>(
                            bytes.as_slice(),
                        )
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
                    let page = store.study_outcome_page(key, after).await?;
                    if page.is_empty() {
                        break;
                    }
                    let mut rows = Vec::with_capacity(page.len());
                    for point in page {
                        after = Some(point.ordinal);
                        let outcome = point.outcome()?;
                        usable += u64::from(outcome.scientific.usable);
                        let admitted = optional_definition(
                            point.descriptor.as_slice(),
                            point.admission.as_ref(),
                            point.policy()?,
                        )?;
                        rows.push(super::study_tables::outcome_row(
                            study_id,
                            admitted.as_ref(),
                            &outcome,
                        ));
                    }
                    let batch = super::study_tables::export::<
                        pse_model::generated::runtime::study_outcomes::Row,
                    >(self, rows)?;
                    let columns = batch.batch().num_columns() as u64;
                    let base = offset;
                    offset += batch.batch().num_rows() as u64;
                    super::result_files::visit_result_files_async(
                        batch.batch(),
                        |start, rows, payload| {
                            let current = ordinal;
                            ordinal += 1;
                            let start = base + start as u64;

                            let set = set.clone();
                            let fence = &fence;
                            async move {
                                let operation = format!("study-summary:{}:{current}", fence.attempt());
                                let digest = pse_operations::canonical_execution::result_payload_digest(&payload);
                                let intent = store.reserve_result_file(fence, &operation, &digest).await?;
                                let sealed = store.seal_file(&intent, &payload, pse_operations::canonical_files::ARROW_IPC_FILE, 1, &relation.to_string()).await?;
                                let payload = pse_operations::canonical_execution::ResultPayload::from_sealed(sealed, 0)?;
                                let block =
                                    pse_model::generated::runtime::canonical_result_blocks::Row {
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
                                        payload_bytes: payload.byte_len(),
                                        payload_digest: payload.digest().to_owned(),
                                        interpretation:
                                            pse_operations::generated::surreal::INTERPRETATION
                                                .into(),
                                    };
                                store
                                    .append_result_block(
                                        fence,
                                        &format!("study-summary:{}:{current}", fence.attempt()),
                                        &relation.to_string(),
                                        current,
                                        payload,
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
                    .close_result_ingestion(
                        &fence,
                        &format!("study-summary-close:{}", fence.attempt()),
                    )
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
            })
            .await
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
        record: &super::DurableRecord,
    ) -> Result<CanonicalPoint, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let terminal = record.attempt.as_ref().map_err(|error| {
                    super::diagnostics::operation_context(
                        WorkflowError::Shared(error.clone()),
                        "study.record_study_attempt.retained_result",
                    )
                })?;
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
                facts.scientific = stored_scientific_facts(stored);
                facts.retry_failure = stored.termination.retry_failure;
                facts.effect = stored.termination.effect;
                let diagnostic = stored_diagnostic(stored);
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
                    .canonical_store()?
                    .canonical_study(&point.study)
                    .await?
                    .ok_or_else(|| contract("study absent"))?;
                let settled = !pse_model::study_policy::may_retry(&policy, &facts, study.cancelled);
                // SAFETY: this owner projects the drained scientific result and its admitted
                // durable completion after checking that the point names this exact attempt.
                Ok(unsafe {
                    self.canonical_store()?
                        .observe_study_point(point, &facts, &outcome, settled)
                        .await
                }?)
            })
            .await
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
    #[serde(skip)]
    _owner: Option<Arc<dyn pse_columnar::PayloadOwner>>,
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

/// One terminal realization; its reservation follows retained consumers.
pub(in crate::workflow) struct TerminalStudyInventory {
    pub status: StudyStatus,
    pub facts: Vec<pse_model::study::PointFacts>,
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}

#[cfg(test)]
tokio::task_local! {
    pub(in crate::workflow) static TERMINAL_PAGE_OBSERVER: Box<dyn Fn() + Send + Sync>;
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
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let store = self.runtime.canonical_store()?;
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
            })
            .await
    }
    /// Revoke study claims and every assigned writer through one guarded header;
    /// native renewal and ingestion observe this cancellation immediately.
    pub async fn cancel(&self) -> Result<StudyCancel, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let store = self.runtime.canonical_store()?;
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
            })
            .await
    }
    /// Hydrate terminal occurrences once. Open polling reads only the compact header.
    pub(in crate::workflow) async fn terminal_inventory(
        &self,
    ) -> Result<Option<TerminalStudyInventory>, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let store = self.runtime.canonical_store()?;
                let study = store
                    .canonical_study_summary(&self.study_id.to_string())
                    .await?
                    .ok_or_else(|| contract("canonical study absent"))?;
                if !study.terminal {
                    return Ok(None);
                }
                let run = store
                    .canonical_run(&study.run)
                    .await?
                    .ok_or_else(|| contract("canonical study run absent"))?;
                let Some(attempt) = run.terminal_attempt else {
                    return Ok(None);
                };
                if study.key != self.study_id.to_string()
                    || !study.active
                    || study.next_ordinal != study.point_count
                {
                    return Err(contract("terminal study inventory header disagrees"));
                }
                let count = usize::try_from(study.point_count)
                    .map_err(|_| contract("study point extent"))?;
                if count > MAXIMUM_STUDY_POINTS {
                    return Err(contract("study point extent"));
                }
                let owner_count = count.div_ceil(64) + 2;
                let bytes = count
                    .checked_mul(
                        size_of::<PointStatus>() + size_of::<pse_model::study::PointFacts>(),
                    )
                    .and_then(|bytes| {
                        bytes.checked_add(
                            owner_count * size_of::<Arc<pse_columnar::AllocationLease>>(),
                        )
                    })
                    .ok_or_else(|| contract("study point extent"))?;
                let base_owner = self
                    .runtime
                    .shared
                    .math()
                    .reserve("study:retained-occurrence-summaries", bytes)?;
                let mut owners = Vec::with_capacity(owner_count);
                owners.push(base_owner);
                // A response and its decoded page coexist only during this realization.
                let _page_owner = self.runtime.shared.math().reserve(
                    "study:occurrence-page",
                    pse_operations::canonical::MESSAGE_BYTES * 4,
                )?;
                let mut points = Vec::with_capacity(count);
                let mut facts = Vec::with_capacity(count);
                let mut after = None;
                loop {
                    #[cfg(test)]
                    let _ = TERMINAL_PAGE_OBSERVER.try_with(|observer| observer());
                    let page = store.study_point_page(&study.key, after).await?;
                    if page.is_empty() {
                        break;
                    }
                    let extent = page.iter().try_fold(0usize, |sum, point| {
                        let variable = point.key.len()
                            + point.study.len()
                            + point.run.len()
                            + point.attempt.as_ref().map_or(0, String::len)
                            + point.facts.as_slice().len()
                            + point
                                .outcome
                                .as_ref()
                                .map_or(0, |value| value.as_slice().len());
                        sum.checked_add(
                            variable
                                .checked_mul(4)
                                .ok_or_else(|| contract("study point extent"))?,
                        )
                        .ok_or_else(|| contract("study point extent"))
                    })?;
                    owners.push(
                        self.runtime
                            .shared
                            .math()
                            .reserve("study:retained-occurrence-page", extent)?,
                    );
                    for point in page {
                        if points.len() >= count
                            || point.study != study.key
                            || point.ordinal != points.len() as u64
                            || point.key
                                != pse_operations::canonical_studies::point_key(
                                    &study.key,
                                    pse_model::study::OccurrenceKey(
                                        u32::try_from(point.occurrence)
                                            .map_err(|_| contract("occurrence overflow"))?,
                                    ),
                                )
                            || !point.settled
                        {
                            return Err(contract("terminal occurrence membership disagrees"));
                        }
                        let fact = point.facts()?;
                        let outcome = point
                            .outcome()?
                            .ok_or_else(|| contract("terminal occurrence outcome absent"))?;
                        if outcome.key != fact.key || outcome.lifecycle != fact.lifecycle {
                            return Err(contract("terminal occurrence outcome disagrees"));
                        }
                        after = Some(point.ordinal);
                        points.push(PointStatus {
                            point_index: fact.key.0,
                            state: fact.lifecycle,
                            outcome: Some(outcome),
                            run: point.run,
                            attempt: point.attempt,
                            settled: point.settled,
                        });
                        facts.push(fact);
                    }
                }
                if points.len() != count {
                    return Err(contract("terminal occurrence inventory incomplete"));
                }
                // Recheck the immutable terminal header before publishing the realization.
                if store.canonical_study_summary(&study.key).await?.as_ref() != Some(&study) {
                    return Err(contract("terminal study changed during occurrence read"));
                }
                store.read_results(&study.run, &attempt).await?;
                Ok(Some(TerminalStudyInventory {
                    status: StudyStatus {
                        study_id: self.study_id,
                        state: StudyState::Concluded,
                        run: study.run,
                        result_attempt: Some(attempt),
                        cancelled: study.cancelled,
                        points,
                    },
                    facts,
                    owner: Arc::new(owners),
                }))
            })
            .await
    }
    /// Sealed canonical handles; incomplete private staging is never returned as a result.
    pub async fn result(&self) -> Result<Option<StudyResults>, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                let Some(inventory) = self.terminal_inventory().await? else {
                    return Ok(None);
                };
                let status = inventory.status;
                let Some(attempt) = status.result_attempt else {
                    return Ok(None);
                };
                let projection_bytes = status
                    .points
                    .len()
                    .checked_mul(size_of::<(u32, String, Option<String>)>())
                    .ok_or_else(|| contract("study result extent"))?;
                let projection_owner = self
                    .runtime
                    .shared
                    .math()
                    .reserve("study:terminal-results", projection_bytes)?;
                let owners: Vec<Arc<dyn pse_columnar::PayloadOwner>> =
                    vec![inventory.owner, projection_owner];
                Ok(Some(StudyResults {
                    run: status.run,
                    attempt,
                    points: status
                        .points
                        .into_iter()
                        .map(|point| (point.point_index, point.run, point.attempt))
                        .collect(),
                    _owner: Some(Arc::new(owners)),
                }))
            })
            .await
    }
    /// Wait for retained finalization, with optional managed workers continuing dispatch.
    pub async fn wait(&self, poll: Duration) -> Result<StudyResults, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                if poll.is_zero() {
                    return Err(contract("study poll interval must be positive"));
                }
                loop {
                    if let Some(result) = self.result().await? {
                        return Ok(result);
                    }
                    tokio::time::sleep(poll).await;
                }
            })
            .await
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
    pub version: Version<7>,
    /// Explicit original-summary and occurrence-detail demand.
    pub output: super::StudyOutputDemand,
    /// Explicit points or a bounded immutable lazy structural input.
    pub inputs: super::StudyInputs,
}

/// A durable study to start.
#[derive(Clone, Debug)]
pub struct StudyPlan {
    /// Explicit result retention and selected detail.
    pub output: super::StudyOutputDemand,
    /// Initial authored ingress; modeling sources are persisted once into a canonical revision.
    pub sources: PackageSources,
    /// Explicit points or a bounded immutable lazy structural input.
    pub inputs: super::StudyInputs,
}

const STUDY_DEFINITION_VERSION: u32 = 11;

/// Version 11 of a study's definition: the store's `definition` document and the content of
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
    /// Lazy science retained once; when present explicit admitted points are empty.
    pub variants: Option<super::LazyStudyDefinition>,
    /// The explicit admitted points, in index order.
    pub points: Vec<StudyPointDefinition>,
}

/// Scientific descriptors shared by local execution and recorded transport.
/// This payload contains no claimed physical publication receipt.
#[derive(Clone, Debug)]
pub struct AdmittedStudy {
    /// Immutable source attribution, rebased only after acknowledged recording.
    pub modeling_revision: String,
    /// Selected-only lazy scientific input.
    pub variants: Option<super::LazyStudyDefinition>,
    /// Explicit physically admitted occurrences in authored order.
    pub points: Vec<StudyPointDefinition>,
}

/// Runtime-local admitted study retaining the actual immutable scientific owners.
/// Deliberately has no serialization contract.
#[derive(Clone, Debug)]
pub struct LocalStudy {
    pub(in crate::workflow) package: super::ModelingPackage,
    pub(in crate::workflow) science: AdmittedStudy,
}
impl std::ops::Deref for LocalStudy {
    type Target = AdmittedStudy;
    fn deref(&self) -> &Self::Target {
        &self.science
    }
}
#[cfg(test)]
impl std::ops::DerefMut for LocalStudy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.science
    }
}
impl LocalStudy {
    /// Execute with the retained model and physical context, independent of store setup.
    pub async fn execute(
        &self,
        maximum_points: usize,
        cancel: &crate::CancelSource,
    ) -> Result<super::StudyReport, WorkflowError> {
        self.package.study(self, maximum_points, cancel).await
    }
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

    /// Project the recorded transport into the common admitted scientific payload.
    pub fn science(&self) -> AdmittedStudy {
        AdmittedStudy {
            modeling_revision: self.modeling_revision.clone(),
            variants: self.variants.clone(),
            points: self.points.clone(),
        }
    }
    /// Validate the common scientific policy, without constructing lazy variants.
    pub fn validate_roles(&self) -> Result<(), WorkflowError> {
        self.view().validate_roles()
    }
    fn view(&self) -> StudyScience<'_> {
        StudyScience {
            modeling_revision: &self.modeling_revision,
            variants: self.variants.as_ref(),
            points: &self.points,
        }
    }
    /// Number of selected occurrences.
    pub fn point_count(&self) -> usize {
        self.variants
            .as_ref()
            .map_or(self.points.len(), |lazy| lazy.variants.selected.len())
    }
    /// Shared occurrence graph.
    pub fn graph(&self) -> OccurrenceGraph {
        self.view().graph()
    }
}
impl AdmittedStudy {
    fn view(&self) -> StudyScience<'_> {
        StudyScience {
            modeling_revision: &self.modeling_revision,
            variants: self.variants.as_ref(),
            points: &self.points,
        }
    }
    /// Validate the common occurrence policy without copying scientific descriptors.
    pub fn validate_roles(&self) -> Result<(), WorkflowError> {
        self.view().validate_roles()
    }
    /// Number of selected occurrences.
    pub fn point_count(&self) -> usize {
        self.variants
            .as_ref()
            .map_or(self.points.len(), |lazy| lazy.variants.selected.len())
    }
    /// Common occurrence graph.
    pub fn graph(&self) -> OccurrenceGraph {
        self.view().graph()
    }
}
struct StudyScience<'a> {
    modeling_revision: &'a str,
    variants: Option<&'a super::LazyStudyDefinition>,
    points: &'a [StudyPointDefinition],
}
impl StudyScience<'_> {
    /// Validate supported producer and consumer roles for every immutable descriptor.
    fn validate_roles(&self) -> Result<(), WorkflowError> {
        if let Some(lazy) = &self.variants {
            if !self.points.is_empty() {
                return Err(contract("study input forms are exclusive"));
            }
            lazy.variants
                .check_retained_parent(self.modeling_revision)?;
            return lazy.variants.validate();
        }
        pse_model::study_policy::admit(&self.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        for point in self.points {
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
    /// Mechanically derive immutable scientific policies without constructing lazy variants.
    fn graph(&self) -> OccurrenceGraph {
        if let Some(lazy) = &self.variants {
            return lazy.variants.graph();
        }
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
    /// Admit scientific intent using the exact retained model and physical context.
    /// No canonical store is initialized or contacted.
    pub async fn admit_study(
        &self,
        inputs: &super::StudyInputs,
        output: &super::StudyOutputDemand,
        cancel: &crate::CancelSource,
    ) -> Result<LocalStudy, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                // Local admission owns execution placement as well as scientific inputs.
                // Family preparations embed this Runtime, so projecting only at dispatch
                // would still let simulations/fitting/horizons create durable attempts.
                let mut package = self.clone();
                package.runtime = package
                    .runtime
                    .clone()
                    .with_durability(super::DurabilitySelection::Ephemeral)?;
                let science = match inputs {
                    super::StudyInputs::Explicit(points) => {
                        package.admit_study_points(points, output, cancel).await?
                    }
                    super::StudyInputs::Lazy(variants) => {
                        creation_checkpoint(cancel)?;
                        output.validate(&package.runtime)?;
                        variants.check_parent(&package)?;
                        let mut retained = variants.clone();
                        retained.retain_parent(package.source_attribution());
                        AdmittedStudy {
                            modeling_revision: package.source_attribution().into(),
                            points: vec![],
                            variants: Some(super::LazyStudyDefinition {
                                variants: retained,
                                output: output.clone(),
                            }),
                        }
                    }
                };
                Ok(LocalStudy { package, science })
            })
            .await
    }
    pub(in crate::workflow) async fn admit_one_study_point(
        &self,
        point: &StudyPoint,
        output: &super::StudyOutputDemand,
        cancel: &crate::CancelSource,
    ) -> Result<StudyPointDefinition, WorkflowError> {
        self.admit_one_study_point_reusing(point, output, &mut None, cancel)
            .await
    }
    async fn admit_one_study_point_reusing(
        &self,
        point: &StudyPoint,
        output: &super::StudyOutputDemand,
        admission: &mut Option<super::study_operations::DeclaredStudyAdmission>,
        cancel: &crate::CancelSource,
    ) -> Result<StudyPointDefinition, WorkflowError> {
        let mut operation = StudyOperation {
            output: output.clone(),
            version: Version,
            source: OperationSource::of(self),
            preparation: point.preparation.clone(),
            operation: point.operation.clone(),
            admitted_horizon: None,
        };
        if !matches!(&operation.operation, OperationRequest::DeclaredCase(_)) {
            *admission = None;
        }
        match &point.policy.start {
            pse_model::study::StartPolicy::Continuation(edge) => {
                operation.admit_seed_role(edge.role)?
            }
            pse_model::study::StartPolicy::Explicit { role, .. } => {
                operation.admit_seed_role(*role)?
            }
            pse_model::study::StartPolicy::Fresh => {}
        }
        operation.admit_horizon_values(self, cancel).await?;
        let (binding, seed_need) = self
            .admit_operation_overlay(&operation, &point.overlay, admission, cancel)
            .await?;
        let policy = PointPolicy {
            key: point.policy.key,
            dependencies: point.policy.dependencies.clone(),
            start: point.policy.start.clone(),
            attempt_limit: point.policy.attempt_limit,
            seed_need,
        };
        Ok(StudyPointDefinition {
            binding_hash: binding.identity(),
            binding,
            operation,
            policy,
        })
    }
    /// Admit operations and physical bindings before either executor schedules an occurrence.
    pub(in crate::workflow) async fn admit_study_points(
        &self,
        points: &[StudyPoint],
        output: &super::StudyOutputDemand,
        cancel: &crate::CancelSource,
    ) -> Result<AdmittedStudy, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("study.request");
                output.validate(&self.runtime)?;
                if points.is_empty() {
                    return Err(WorkflowError::Typed(
                        pse_model::diagnostic::DiagnosticCause::new(
                            pse_model::study_policy::PolicyError::Empty,
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
                let mut admission = None;
                for point in points {
                    admitted.push(
                        self.admit_one_study_point_reusing(point, output, &mut admission, cancel)
                            .await?,
                    );
                }
                drop(admission);
                let definition = AdmittedStudy {
                    modeling_revision: self.source_attribution().into(),
                    variants: None,
                    points: admitted,
                };
                pse_model::study_policy::admit(&definition.graph()).map_err(|error| {
                    WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
                })?;
                definition.validate_roles()?;
                Ok(definition)
            })
            .await
    }
}

struct StudyVariantOwner {
    study: String,
    lazy: super::LazyStudyDefinition,
    physical: super::PhysicalSource,
    package: super::ModelingPackage,
    _owner: Arc<pse_columnar::AllocationLease>,
}

/// Durable worker lane inputs survive adjacent points only until explicit drain.
pub(in crate::workflow) struct StudyCandidateLane {
    input: Option<StudyVariantOwner>,
    source: Option<(
        String,
        super::worker::PhysicalSource,
        super::ModelingPackage,
    )>,
    preparation: super::study_operations::StudyPreparationLane,
}
impl StudyCandidateLane {
    pub(in crate::workflow) fn new() -> Self {
        Self {
            input: None,
            source: None,
            preparation: super::study_operations::StudyPreparationLane::new(),
        }
    }
    pub(in crate::workflow) fn scope(
        &self,
    ) -> Option<Arc<super::modeling::engagement::HeldPreparationScope>> {
        self.preparation.scope()
    }
    pub(in crate::workflow) async fn close(&mut self) -> Result<(), WorkflowError> {
        self.preparation.close().await?;
        self.source = None;
        self.input = None;
        Ok(())
    }
}

#[cfg(test)]
mod creation_authority_tests {
    use super::*;

    fn refused_before_creation<T>(result: Result<T, WorkflowError>) {
        assert!(matches!(result, Err(WorkflowError::Input(message))
            if message == "durable study creation requires explicit durable execution"));
    }

    fn inputs() -> (StudyPlan, StudyDefinition) {
        let sources = BTreeMap::from([(
            format!(
                "unpublished-{}.pse",
                pse_operations::mint_id::<SemanticId>()
            ),
            b"not parsed unless durable execution was selected".to_vec(),
        )]);
        let identity = pse_admission::authoring_driver::document::package_checksum(&sources);
        let definition = StudyDefinition {
            variants: None,
            version: Version,
            physical: super::super::PhysicalSource {
                revision: format!("physical:{identity}:package"),
                identity,
            },
            modeling_revision: "unpublished-local-revision".into(),
            points: Vec::new(),
        };
        let plan = StudyPlan {
            output: super::super::StudyOutputDemand::Full {},
            sources: PackageSources {
                physical: sources,
                modeling: Vec::new(),
            },
            inputs: crate::workflow::StudyInputs::Explicit(Vec::new()),
        };
        (plan, definition)
    }

    #[tokio::test]
    async fn recorded_study_creation_requires_explicit_durable_execution() {
        let runtime = super::super::tests::runtime();
        let (plan, definition) = inputs();
        let cancel = crate::CancelSource::new();
        refused_before_creation(runtime.start_study(plan, &cancel).await);
        refused_before_creation(runtime.start_recorded_study(definition, &cancel).await);
        assert!(runtime.canonical().is_err());
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn recording_connection_does_not_authorize_study_source_publication() {
        let runtime = super::super::tests::runtime()
            .with_canonical(super::super::tests::canonical_deployment())
            .unwrap();
        assert!(matches!(
            runtime.durability(),
            super::super::Durability::Ephemeral
        ));
        let (plan, definition) = inputs();
        let receipt = definition.physical.revision.clone();
        let store = runtime.canonical_store().unwrap();
        assert!(store.revision(&receipt).await.unwrap().is_none());
        let before = store.study_page(None).await.unwrap();
        let cancel = crate::CancelSource::new();
        refused_before_creation(runtime.start_study(plan, &cancel).await);
        refused_before_creation(runtime.start_recorded_study(definition, &cancel).await);
        assert!(store.revision(&receipt).await.unwrap().is_none());
        assert_eq!(store.study_page(None).await.unwrap(), before);
        // Explicit recording reads retain their independent authority.
        assert!(
            runtime
                .source_settlement("never-issued", &receipt)
                .await
                .unwrap()
                .acknowledgement
                .is_none()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
