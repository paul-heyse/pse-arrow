// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original completion accumulation and bounded, conditional detail retention.
#[cfg(feature = "solver-diffsol")]
use crate::workflow::{
    self, Completion, ModelingSolvePreparation, PreparationServices, RunResult, StudyOutputDemand,
    WorkflowError, contract,
};
use pse_ids::SemanticId;
#[cfg(feature = "solver-diffsol")]
use pse_relations::columnar::FieldCheckedBatch;
#[cfg(feature = "solver-diffsol")]
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[cfg(feature = "solver-diffsol")]
pub(super) type Detail = BTreeMap<SemanticId, Vec<FieldCheckedBatch>>;

#[cfg(feature = "solver-diffsol")]
type PrefixResult = (
    Completion,
    Result<Detail, Arc<WorkflowError>>,
    Detail,
    Vec<Arc<pse_columnar::AllocationLease>>,
    Vec<pse_modeling::DomainTightening>,
    Option<Arc<WorkflowError>>,
    Vec<SemanticId>,
);

#[cfg(feature = "solver-diffsol")]
struct SelectedDetail {
    demand: workflow::StudyDetailDemand,
    allowance: workflow::study_demand::DetailAllowance,
    chunks: Detail,
    _owner: Arc<pse_columnar::AllocationLease>,
}
#[cfg(feature = "solver-diffsol")]
impl SelectedDetail {
    fn new(
        services: &PreparationServices,
        demand: &workflow::StudyDetailDemand,
    ) -> Result<Self, WorkflowError> {
        let bytes = (demand.maximum_rows as usize)
            .checked_mul(size_of::<FieldCheckedBatch>() + 128)
            .and_then(|bytes| bytes.checked_add(demand.relation_names().count().checked_mul(1024)?))
            .ok_or_else(|| contract("horizon selected detail envelope extent"))?;
        let owner = services.math().reserve("horizon:selected-detail", bytes)?;
        let mut chunks = BTreeMap::new();
        for name in demand.relation_names().collect::<BTreeSet<_>>() {
            let id = services
                .registry()
                .relation(name)
                .ok_or_else(|| contract("horizon detail relation absent"))?
                .id;
            chunks.insert(id, Vec::new());
        }
        Ok(Self {
            demand: demand.clone(),
            allowance: workflow::study_demand::DetailAllowance::new(demand),
            chunks,
            _owner: owner,
        })
    }
    async fn append(&mut self, result: &RunResult) -> Result<(), WorkflowError> {
        for name in self.demand.relation_names().collect::<BTreeSet<_>>() {
            let id = result
                .services
                .preparation()
                .registry()
                .relation(name)
                .ok_or_else(|| contract("horizon detail relation absent"))?
                .id;
            let cursor = result
                .cursor_by_id(id, 1024, workflow::ResultOrder::Canonical)
                .map_err(WorkflowError::Shared)?;
            let mut cursor = self.demand.select_cursor(
                name,
                id,
                cursor,
                result.services.preparation().pool(),
            )?;
            while let Some(batch) = cursor.next_batch().await.map_err(WorkflowError::Shared)? {
                if batch.batch().num_rows() == 0 {
                    continue;
                }
                self.allowance.accept(&batch)?;
                self.chunks
                    .get_mut(&id)
                    .ok_or_else(|| contract("horizon detail membership changed"))?
                    .push(batch.with_export_owner(self._owner.clone()));
            }
            if !cursor.complete() {
                return Err(contract("horizon detail producer incomplete"));
            }
        }
        Ok(())
    }
}

/// Membership comes from the modeling summary contract, independently of whether
/// a first scientific step was reached. Empty members keep their declared schema.
pub(in crate::workflow) fn summary_ids() -> [SemanticId; 10] {
    use pse_relations::generated::runtime::*;
    [
        candidate_assessments::RELATION_ID,
        accuracy_goal_assessments::RELATION_ID,
        resolved_numerics::RELATION_ID,
        run_lineage::RELATION_ID,
        solve_runs::RELATION_ID,
        modeling_checks::RELATION_ID,
        modeling_reports::RELATION_ID,
        modeling_findings::RELATION_ID,
        route_decisions::RELATION_ID,
        structural_assessments::RELATION_ID,
    ]
}

/// The original grant stays unique until the original rows have been moved.
#[cfg(feature = "solver-diffsol")]
struct OriginalAdmission {
    goals: usize,
    bytes: usize,
    owner: usize,
    capacity_growth: usize,
}

#[cfg(feature = "solver-diffsol")]
pub(super) struct OriginalHistory {
    pub(super) completion: Completion,
    summaries: Detail,
    summary_error: Option<Arc<WorkflowError>>,
    owners: Vec<Arc<pse_columnar::AllocationLease>>,
    success: Option<SelectedDetail>,
    failed: Option<SelectedDetail>,
    success_error: Option<Arc<WorkflowError>>,
    failed_error: Option<Arc<WorkflowError>>,
    primary: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    accuracy: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    causes: Vec<Arc<pse_model::diagnostic::DiagnosticCause>>,
    tightenings: Vec<pse_modeling::DomainTightening>,
    admitted_goals: Option<OriginalAdmission>,
    compact: bool,
}
#[cfg(feature = "solver-diffsol")]
impl OriginalHistory {
    pub(super) fn new(
        services: &PreparationServices,
        demand: &StudyOutputDemand,
    ) -> Result<Self, WorkflowError> {
        let (success, failed) = match demand {
            StudyOutputDemand::Full {} => (None, None),
            StudyOutputDemand::Compact {
                detail,
                failed_detail,
            } => (
                Some(SelectedDetail::new(services, detail)?),
                Some(SelectedDetail::new(services, failed_detail)?),
            ),
        };
        let owner = services
            .math()
            .reserve("horizon:summary-membership", 64 * 1024)?;
        let summaries = summary_ids()
            .into_iter()
            .filter(|id| *id != pse_relations::generated::runtime::modeling_findings::RELATION_ID)
            .map(|id| (id, Vec::new()))
            .collect();
        Ok(Self {
            completion: Completion::default(),
            summaries,
            summary_error: None,
            owners: vec![owner],
            success,
            failed,
            success_error: None,
            failed_error: None,
            primary: Vec::with_capacity(1),
            accuracy: Vec::new(),
            causes: Vec::new(),
            tightenings: Vec::new(),
            admitted_goals: None,
            compact: matches!(demand, StudyOutputDemand::Compact { .. }),
        })
    }
    /// Reserve all aggregate row slots and their original producer's finite
    /// report envelope BEFORE dispatch. Post-science ingestion moves rows only.
    pub(super) fn admit_step(
        &mut self,
        services: &PreparationServices,
        prepared: &ModelingSolvePreparation,
    ) -> Result<(), WorkflowError> {
        if self.admitted_goals.is_some() {
            return Err(contract(
                "horizon original completion admission already live",
            ));
        }
        let goals = prepared.solve.numerics().policy.goals.len();
        let bytes = prepared
            .solve
            .result_bytes()?
            .checked_mul(2)
            .and_then(|bytes| {
                bytes.checked_add(
                    workflow::modeling::results::result_bytes(prepared)
                        .ok()?
                        .checked_mul(2)?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add((goals + 1).checked_mul(
                    size_of::<pse_model::diagnostic::BoundaryDiagnostic>() + 64 * 1024,
                )?)
            })
            .and_then(|bytes| bytes.checked_add(16 * 1024))
            .ok_or_else(|| contract("horizon original completion admission extent"))?;
        let descriptor_bytes =
            prepared
                .model
                .tightenings
                .iter()
                .try_fold(0usize, |bytes, tightening| {
                    bytes
                        .checked_add(
                            size_of::<pse_modeling::DomainTightening>()
                                + tightening.path.len()
                                + 128,
                        )
                        .ok_or_else(|| contract("horizon original tightening extent"))
                })?;
        let owner = services
            .math()
            .reserve("horizon:original-completion-before-science", bytes)?;
        let descriptor_owner = services
            .math()
            .reserve("horizon:original-tightening-descriptors", descriptor_bytes)?;
        let capacity_before = self.capacity_bytes()?;
        let owner_index = self.owners.len();
        self.owners.push(owner);
        self.owners.push(descriptor_owner);
        let tightening_start = self.tightenings.len();
        self.completion.assessments.reserve(1);
        self.completion.accuracy_goals.reserve(goals);
        self.completion.solves.reserve(1);
        self.completion.lineage.reserve(1);
        // Keep room for the final diagnostic concatenation without a final grant.
        self.primary.reserve(self.accuracy.len() + goals + 2);
        self.accuracy.reserve(goals);
        self.causes.reserve(goals);
        self.tightenings.reserve(prepared.model.tightenings.len());
        self.tightenings
            .extend(prepared.model.tightenings.iter().cloned());
        let descriptor_extent =
            self.tightenings[tightening_start..]
                .iter()
                .try_fold(0usize, |bytes, tightening| {
                    bytes
                        .checked_add(tightening.path.capacity())
                        .ok_or_else(|| contract("horizon retained tightening extent"))
                })?;
        let descriptor_owner = self
            .owners
            .pop()
            .ok_or_else(|| contract("horizon tightening owner absent"))?;
        self.owners
            .push(partition_retained(descriptor_owner, descriptor_extent));
        let capacity_growth = self
            .capacity_bytes()?
            .checked_sub(capacity_before)
            .ok_or_else(|| contract("horizon aggregate capacity shrank during admission"))?;
        if capacity_growth > bytes {
            return Err(contract(
                "horizon aggregate slots exceeded pre-science admission",
            ));
        }
        self.admitted_goals = Some(OriginalAdmission {
            goals,
            bytes,
            owner: owner_index,
            capacity_growth,
        });
        Ok(())
    }
    pub(super) fn validate_original(&self, result: &RunResult) -> Result<(), WorkflowError> {
        let admission = self
            .admitted_goals
            .as_ref()
            .ok_or_else(|| contract("horizon original completion lacked pre-science admission"))?;
        let (goals, bytes) = (admission.goals, admission.bytes);
        let completion = result
            .completion
            .as_ref()
            .map_err(|error| WorkflowError::Shared(error.clone()))?;
        if completion.assessments.len() != 1
            || completion.solves.len() != 1
            || completion.lineage.len() != 1
            || completion.accuracy_goals.len() > goals
            || completion.diagnostics.len() > goals + 1
            || completion.computation.is_some()
            || completion_retained_extent(completion, admission.capacity_growth)? > bytes
        {
            return Err(contract(
                "original modeling completion exceeded its admitted producer shape",
            ));
        }
        Ok(())
    }
    /// Publication may refuse after science. This phase never consumes the
    /// original completion; the caller transfers it on every exit below.
    pub(super) async fn export(&mut self, result: &RunResult) -> Result<(), WorkflowError> {
        if let Some(error) = &self.summary_error {
            return Err(WorkflowError::Shared(error.clone()));
        }
        let summaries = async {
            if self.compact {
                for id in summary_ids() {
                    if id == pse_relations::generated::runtime::modeling_findings::RELATION_ID {
                        continue;
                    }
                    let entry = self
                        .summaries
                        .get_mut(&id)
                        .ok_or_else(|| contract("horizon mandatory summary membership changed"))?;
                    let mut cursor = result
                        .cursor_by_id(id, 1024, workflow::ResultOrder::Canonical)
                        .map_err(WorkflowError::Shared)?;
                    while let Some(batch) =
                        cursor.next_batch().await.map_err(WorkflowError::Shared)?
                    {
                        if batch.batch().num_rows() == 0 {
                            continue;
                        }
                        let bytes = batch
                            .batch()
                            .get_array_memory_size()
                            .checked_add(size_of::<FieldCheckedBatch>() + 128)
                            .ok_or_else(|| contract("horizon summary retention extent"))?;
                        let owner = result
                            .services
                            .preparation()
                            .math()
                            .reserve("horizon:original-summary", bytes)?;
                        entry.push(batch.with_export_owner(owner));
                    }
                    if !cursor.complete() {
                        return Err(contract("horizon summary producer incomplete"));
                    }
                }
            }
            Ok::<_, WorkflowError>(())
        }
        .await;
        if let Err(error) = summaries {
            let error = Arc::new(error);
            self.summary_error = Some(error.clone());
            if self.success_error.is_none() {
                self.success_error = Some(error.clone());
            }
            if self.failed_error.is_none() {
                self.failed_error = Some(error.clone());
            }
            for selected in [&mut self.success, &mut self.failed].into_iter().flatten() {
                for chunks in selected.chunks.values_mut() {
                    *chunks = Vec::new();
                }
            }
            // Schema membership remains exact. A prefix missing any issued step
            // is incomplete, so these buffers cannot be advertised as a table.
            for chunks in self.summaries.values_mut() {
                *chunks = Vec::new();
            }
            return Err(WorkflowError::Shared(error));
        }
        if self.success_error.is_none()
            && let Some(selected) = &mut self.success
            && let Err(error) = selected.append(result).await
        {
            self.success_error = Some(Arc::new(error));
            for chunks in selected.chunks.values_mut() {
                *chunks = Vec::new();
            }
        }
        if self.failed_error.is_none()
            && let Some(selected) = &mut self.failed
            && let Err(error) = selected.append(result).await
        {
            self.failed_error = Some(Arc::new(error));
            for chunks in selected.chunks.values_mut() {
                *chunks = Vec::new();
            }
        }
        Ok(())
    }
    /// Validated producer shape plus preallocated aggregate slots makes transfer
    /// independent of every publication/allocation refusal after assessment.
    pub(super) fn take_original(&mut self, result: &mut RunResult) {
        let completion = std::mem::replace(
            &mut result.completion,
            Err(Arc::new(contract(
                "original completion transferred to horizon owner",
            ))),
        );
        let Ok(mut completion) = completion else {
            return;
        };
        // Validation already established these exact extents. Refuse no new
        // allocation here: partition transfers the pre-science reservation.
        let retained = self.admitted_goals.as_ref().and_then(|admission| {
            completion_retained_extent(&completion, admission.capacity_growth).ok()
        });
        self.completion
            .assessments
            .append(&mut completion.assessments);
        self.completion
            .accuracy_goals
            .append(&mut completion.accuracy_goals);
        self.completion.solves.append(&mut completion.solves);
        self.completion.lineage.append(&mut completion.lineage);
        let causes_before = self.causes.len();
        let mut diagnostics = completion.diagnostics.into_iter();
        if let Some(point) = result.modeling_result(0) {
            let local_causes = || {
                point
                    .completion
                    .accuracy
                    .iter()
                    .enumerate()
                    .filter_map(|(index, goal)| {
                        let failure = goal.failure.as_ref()?;
                        (!point.completion.accuracy[..index]
                            .iter()
                            .filter_map(|goal| goal.failure.as_ref())
                            .any(|earlier| Arc::ptr_eq(&earlier.cause, &failure.cause)))
                        .then_some(&failure.cause)
                    })
            };
            let primary_count = diagnostics.len().saturating_sub(local_causes().count());
            self.primary
                .extend(diagnostics.by_ref().take(primary_count));
            for (cause, diagnostic) in local_causes().zip(diagnostics) {
                if !self
                    .causes
                    .iter()
                    .any(|earlier| Arc::ptr_eq(earlier, cause))
                {
                    self.causes.push(cause.clone());
                    self.accuracy.push(diagnostic);
                }
            }
        } else {
            self.primary.extend(diagnostics);
        }
        if let Some(admission) = self.admitted_goals.take() {
            // DiagnosticCause owns a dyn DiagnosticProjection whose producer
            // payload has no HeapUsage contract. Only genuinely newly retained
            // opaque causes require the conservative producer envelope.
            if self.causes.len() == causes_before
                && let Some(retained) = retained
            {
                let placeholder = self.owners[0].clone();
                let owner = std::mem::replace(&mut self.owners[admission.owner], placeholder);
                self.owners[admission.owner] = partition_retained(owner, retained);
            }
        }
    }
    fn capacity_bytes(&self) -> Result<usize, WorkflowError> {
        fn capacity<T>(values: &Vec<T>) -> Option<usize> {
            values.capacity().checked_mul(size_of::<T>())
        }
        [
            capacity(&self.completion.assessments),
            capacity(&self.completion.accuracy_goals),
            capacity(&self.completion.solves),
            capacity(&self.completion.lineage),
            capacity(&self.primary),
            capacity(&self.accuracy),
            capacity(&self.causes),
            capacity(&self.tightenings),
            capacity(&self.owners),
        ]
        .into_iter()
        .try_fold(0usize, |bytes, extent| {
            extent.and_then(|extent| bytes.checked_add(extent))
        })
        .ok_or_else(|| contract("horizon aggregate capacity extent"))
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn retained_owner_bytes(&self) -> usize {
        self.owners.iter().map(|owner| owner.size()).sum()
    }
    pub(super) fn finish(
        mut self,
        usable: bool,
        final_error: Option<&WorkflowError>,
    ) -> PrefixResult {
        let (selected, error) = if usable {
            (self.success, self.success_error)
        } else {
            (self.failed, self.failed_error)
        };
        let detail_membership = selected
            .as_ref()
            .map(|selected| selected.chunks.keys().copied().collect())
            .unwrap_or_default();
        let detail = match error {
            Some(error) => Err(error),
            None => Ok(selected.map_or_else(BTreeMap::new, |selected| selected.chunks)),
        };
        if let Some(error) = final_error {
            self.primary.push(error.boundary_diagnostic());
        }
        self.primary.append(&mut self.accuracy);
        self.completion.diagnostics = self.primary;
        (
            self.completion,
            detail,
            self.summaries,
            self.owners,
            self.tightenings,
            self.summary_error,
            detail_membership,
        )
    }
}

/// Unique partitioning never releases/reacquires live capacity. A failed split
/// retains the original grant, so ownership is never lost on an accounting error.
#[cfg(feature = "solver-diffsol")]
fn partition_retained(
    owner: Arc<pse_columnar::AllocationLease>,
    bytes: usize,
) -> Arc<pse_columnar::AllocationLease> {
    match owner.partition(&[bytes]) {
        Ok(mut owners) => owners.remove(0),
        Err(owner) => owner,
    }
}

/// Rust-owned strings/nested capacities, aggregate Vec growth, and the eventual
/// separate RunResult assessment snapshot. Native/preparation report buffers are
/// owned elsewhere and are not part of this original Completion.
#[cfg(feature = "solver-diffsol")]
fn completion_retained_extent(
    completion: &Completion,
    capacity_growth: usize,
) -> Result<usize, WorkflowError> {
    use pse_model::HeapUsage;
    fn heap<T: HeapUsage>(rows: &[T]) -> Option<usize> {
        rows.iter()
            .try_fold(0usize, |bytes, row| bytes.checked_add(row.heap_bytes()))
    }
    let assessment_heap = heap(&completion.assessments)
        .ok_or_else(|| contract("horizon assessment payload extent"))?;
    let mut bytes = capacity_growth.checked_add(assessment_heap)
        .and_then(|bytes|bytes.checked_add(completion.assessments.len().checked_mul(size_of::<pse_model::generated::runtime::candidate_assessments::Row>())?))
        .and_then(|bytes|bytes.checked_add(assessment_heap))
        .and_then(|bytes|bytes.checked_add(128)) // lease allocation and its sharing counters
        .ok_or_else(||contract("horizon retained original extent"))?;
    for extent in [
        heap(&completion.accuracy_goals),
        heap(&completion.solves),
        heap(&completion.lineage),
        heap(&completion.diagnostics),
    ] {
        bytes = bytes
            .checked_add(extent.ok_or_else(|| contract("horizon original nested payload extent"))?)
            .ok_or_else(|| contract("horizon original completion extent"))?;
    }
    Ok(bytes)
}

/// Actual deep preparation copies are charged before their maps are cloned.
/// Shared mathematical/native owners continue carrying their original allocations.
#[cfg(feature = "solver-diffsol")]
pub(super) fn preparation_copy_owner(
    services: &PreparationServices,
    prepared: &ModelingSolvePreparation,
) -> Result<Vec<Arc<pse_columnar::AllocationLease>>, WorkflowError> {
    let mut owners = Vec::with_capacity(prepared.providers.len() + 2);
    owners.push(workflow::durable::completion_retention_owner(
        &services.pool(),
        &(
            &prepared.starts,
            &prepared.model.values.scalars,
            &prepared.profile.numerics,
            &prepared.profile.composition,
            prepared.profile.backend.document(),
            &prepared.profile.sensitivity,
            &prepared.profile.controls,
        ),
    )?);
    let tightening_bytes =
        prepared
            .model
            .tightenings
            .iter()
            .try_fold(0usize, |bytes, tightening| {
                bytes
                    .checked_add(
                        size_of::<pse_modeling::DomainTightening>() + tightening.path.len() + 128,
                    )
                    .ok_or_else(|| contract("horizon tightening copy extent"))
            })?;
    owners.push(services.math().reserve(
        "horizon:preparation-copy-envelope",
        size_of::<ModelingSolvePreparation>()
            + prepared.providers.len() * (size_of::<pse_kernels::Registration>() + 128)
            + tightening_bytes
            + 256,
    )?);
    for registration in prepared.providers.values() {
        let spec = registration.spec();
        let mut bytes = (spec.inputs.len() + spec.outputs.len()) * size_of::<pse_kernels::Port>()
            + registration.envelope().map_or(0, size_of_val)
            + 256;
        for shape in spec.shapes.inputs.iter().chain(&spec.shapes.outputs) {
            let coordinates = shape.coordinates.iter().try_fold(0usize, |bytes, row| {
                bytes
                    .checked_add(row.len() * size_of::<SemanticId>() + size_of::<Vec<SemanticId>>())
                    .ok_or_else(|| contract("horizon provider coordinate copy extent"))
            })?;
            bytes = bytes
                .checked_add(
                    size_of::<pse_kernels::ProviderShape>()
                        + (shape.axes.len() + shape.cells.len()) * size_of::<SemanticId>()
                        + coordinates,
                )
                .ok_or_else(|| contract("horizon provider shape copy extent"))?;
        }
        owners.push(services.math().reserve("horizon:provider-copy", bytes)?);
    }
    Ok(owners)
}

#[cfg(feature = "solver-diffsol")]
pub(super) fn step_copy_owner(
    services: &PreparationServices,
    measured: &[f64],
    applied: &[f64],
    reached: &[f64],
    activity: Option<&pse_backend_native::kkt::path::PathPrediction>,
) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
    let bytes = (measured.len() + applied.len() + reached.len()) * size_of::<f64>()
        + size_of::<super::HorizonStep>()
        + 256;
    let activity = activity.map_or(0, |activity| {
        activity.events.len() * size_of::<pse_backend_native::kkt::path::Event>()
            + activity.prediction.variables.len() * size_of::<SemanticId>()
            + (activity.prediction.primal.len()
                + activity.prediction.row_dual.len()
                + activity.prediction.step.len())
                * size_of::<f64>()
    });
    Ok(services.math().reserve(
        "horizon:full-sample-history",
        bytes
            .checked_add(activity)
            .ok_or_else(|| contract("horizon sample history extent"))?,
    )?)
}
