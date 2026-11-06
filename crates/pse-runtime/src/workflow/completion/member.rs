// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original whole-member permission; descriptive observations cannot mint native custody.
use super::{Completion, ScientificPermission};
use crate::workflow::{
    FitExperimentAssessmentState, ModelingSimulation, RunDurability, RunOutcome, RunResult,
    WorkflowError,
};
use pse_model::generated::{
    enums::{CandidateRefusal, CandidateUse},
    identities::{AttemptId, InstanceId, RunId},
};
use std::sync::Arc;

#[derive(Debug)]
struct MemberCustody {
    report: Arc<pse_backend_native::dynamics::Report>,
    assessment: Arc<crate::workflow::fitting::TrajectoryAssessment>,
    _result: Option<Arc<pse_columnar::AllocationLease>>,
    _metadata: Arc<pse_columnar::AllocationLease>,
}

pse_contract::projection_struct! {
/// Recorded original-member decision with optional process-local native custody.
/// Decoded facts remain readable, but cannot grant native use.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MemberScientificPermission {
    projection: { name: "ScientificMemberPermissionSnapshot", rust_path: crate::workflow::MemberScientificPermission,
        version: 1, direction: pse_contract::BoundaryDirection::Output, construction: ObservationOnly,
        reserved_tags: [], reserved_names: [] };
    /// Original producer run, not a mathematical content identity.
    run_id: RunId => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actual durable attempt; genuinely ephemeral runs retain absence.
    attempt_id: Option<AttemptId> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exact originally declared instance.
    member: InstanceId => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original authored source interpretation.
    source: pse_ids::ContentHash => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original physical definition, including quantity/unit/datum meaning.
    physical: pse_ids::ContentHash => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original source/layout and integration preparation.
    preparation: pse_ids::ContentHash => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Recorded original scientific decision. Reading it is distinct from current native proof.
    permission: ScientificPermission => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    @local {
    #[serde(skip)]
    #[schemars(skip)]
    custody: Option<Arc<MemberCustody>>;
    }
}
}
impl MemberScientificPermission {
    /// Exact declared member of the original producer.
    pub fn member(&self) -> InstanceId {
        self.member
    }
    /// Original producer identity.
    pub fn run_id(&self) -> RunId {
        self.run_id
    }
    /// Original durable attempt, absent for ephemeral execution.
    pub fn attempt_id(&self) -> Option<AttemptId> {
        self.attempt_id
    }
    /// Recorded observation, independently of process-local native availability.
    pub fn recorded_permission(&self) -> &ScientificPermission {
        &self.permission
    }
    /// Decoding cannot manufacture native custody.
    pub fn has_native_proof(&self) -> bool {
        self.custody.is_some()
    }
    /// This retained original native member permits scientific result use.
    pub fn permits_result(&self) -> bool {
        self.has_native_proof() && self.permission.permits_result()
    }
    pub(super) fn native_permission(&self) -> Option<&ScientificPermission> {
        self.custody.as_ref().map(|_| &self.permission)
    }
    pub(in crate::workflow) fn matches(
        &self,
        producer: &RunResult,
        member: InstanceId,
        source: &ModelingSimulation,
        report: &Arc<pse_backend_native::dynamics::Report>,
        assessment: &Arc<crate::workflow::fitting::TrajectoryAssessment>,
    ) -> bool {
        let attempt = match producer.durability() {
            RunDurability::Ephemeral => None,
            RunDurability::Durable(record) => Some(record.attempt_id),
        };
        self.run_id == producer.run_id
            && self.attempt_id == attempt
            && self.member == member
            && self.source == source.source.revision.identity().as_id()
            && self.physical == source.source.physical.identity()
            && self.preparation == source.identity()
            && source.solved.instance() == member
            && self.custody.as_ref().is_some_and(|custody| {
                Arc::ptr_eq(&custody.report, report) && Arc::ptr_eq(&custody.assessment, assessment)
            })
    }
}

impl RunResult {
    pub(super) fn capture_fit_member_permissions(
        &self,
        completion: &mut Completion,
        attempt_id: Option<AttemptId>,
    ) -> Result<(), WorkflowError> {
        let RunOutcome::Fit {
            request,
            result: Ok(report),
        } = &self.outcome
        else {
            return Ok(());
        };
        let selected = || {
            request
                .declaration()
                .experiments
                .iter()
                .filter_map(|declaration| {
                    let member = declaration.experiment_id;
                    let source = request.trajectory_source(member)?;
                    let assessment = report.assessment.experiment(member)?;
                    let local = assessment.trajectory()?;
                    let trajectory = report.trajectories.get(&member)?;
                    Some((member, source, assessment.state(), local, trajectory))
                })
        };
        let (count, bytes) = selected()
            .try_fold((0usize, 0usize), |(count, bytes), (_, _, _, local, _)| {
                let dynamic = (local.completion.decision.refusals.len() + 1)
                    .checked_mul(size_of::<CandidateRefusal>())?
                    .checked_add(local.completion.decision.qualifiers.len().checked_mul(
                        size_of::<pse_model::generated::enums::CandidateQualifier>(),
                    )?)?;
                Some((
                    count.checked_add(1)?,
                    bytes
                        .checked_add(
                            size_of::<MemberScientificPermission>()
                                + size_of::<MemberCustody>()
                                + 2 * size_of::<usize>(),
                        )?
                        .checked_add(dynamic)?,
                ))
            })
            .ok_or_else(|| crate::workflow::contract("original member permission extent"))?;
        if count == 0 {
            return Ok(());
        }
        let owner = self
            .services
            .preparation()
            .math()
            .reserve("completion:original-member-permission", bytes)?;
        let mut permissions = Vec::with_capacity(count);
        for (member, source, state, local, trajectory) in selected() {
            let succeeded =
                state == FitExperimentAssessmentState::Completed && trajectory.error.is_none();
            let mut refusals = Vec::with_capacity(local.completion.decision.refusals.len() + 1);
            refusals.extend_from_slice(&local.completion.decision.refusals);
            if !succeeded && !refusals.contains(&CandidateRefusal::NativeOutcome) {
                refusals.push(CandidateRefusal::NativeOutcome);
            }
            permissions.push(MemberScientificPermission {
                run_id: self.run_id,
                attempt_id,
                member,
                source: source.source.revision.identity().as_id(),
                physical: source.source.physical.identity(),
                preparation: source.identity(),
                permission: ScientificPermission {
                    reference_point: None,
                    step: None,
                    requested: source.profile().samples.len(),
                    observed: trajectory.coverage.produced(),
                    report_succeeded: succeeded,
                    decision: if succeeded {
                        local.completion.decision.usability
                    } else {
                        CandidateUse::Unusable
                    },
                    refusals,
                    qualifiers: local.completion.decision.qualifiers.clone(),
                },
                custody: Some(Arc::new(MemberCustody {
                    report: trajectory.clone(),
                    assessment: local.clone(),
                    _result: self._owner.clone(),
                    _metadata: owner.clone(),
                })),
            });
        }
        completion.member_permissions = permissions;
        Ok(())
    }
}
