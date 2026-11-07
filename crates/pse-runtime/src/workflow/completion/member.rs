// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original whole-member permission; descriptive observations cannot mint native custody.
use super::{Completion, ScientificPermission};
use crate::workflow::{
    FitExperimentAssessmentState, ModelingSimulation, ModelingTrajectory, RunDurability,
    RunOutcome, RunResult, WorkflowError,
};
use pse_model::generated::{
    enums::{CandidateRefusal, CandidateUse},
    identities::{AttemptId, InstanceId, RunId},
};
use std::sync::Arc;

#[derive(Debug)]
struct MemberCustody {
    native: NativeMember,
    _metadata: Arc<pse_columnar::AllocationLease>,
}

#[derive(Debug)]
enum NativeMember {
    Fit {
        report: Arc<pse_backend_native::dynamics::Report>,
        assessment: Arc<crate::workflow::fitting::TrajectoryAssessment>,
        _result: Option<Arc<pse_columnar::AllocationLease>>,
    },
    Simulation(ModelingTrajectory),
}
impl NativeMember {
    fn report(&self) -> &pse_backend_native::dynamics::Report {
        match self {
            Self::Fit { report, .. } => report,
            Self::Simulation(trajectory) => trajectory.report(),
        }
    }
    fn completion(&self) -> &crate::workflow::numerics::Completed {
        match self {
            Self::Fit { assessment, .. } => &assessment.completion,
            Self::Simulation(trajectory) => trajectory.completion(),
        }
    }
}

fn permission_bytes(local: &crate::workflow::numerics::Completed) -> Option<usize> {
    (local.decision.refusals.len().checked_add(1)?)
        .checked_mul(size_of::<CandidateRefusal>())?
        .checked_add(
            local
                .decision
                .qualifiers
                .len()
                .checked_mul(size_of::<pse_model::generated::enums::CandidateQualifier>())?,
        )?
        .checked_add(
            size_of::<MemberScientificPermission>()
                + size_of::<MemberCustody>()
                + 2 * size_of::<usize>(),
        )
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
    /// Compare recorded original identity only. This neither requires nor issues live proof.
    pub(in crate::workflow) fn records_original(
        &self,
        run_id: RunId,
        attempt_id: Option<AttemptId>,
        member: InstanceId,
        source: pse_ids::ContentHash,
        physical: pse_ids::ContentHash,
        preparation: pse_ids::ContentHash,
    ) -> bool {
        self.run_id == run_id
            && self.attempt_id == attempt_id
            && self.member == member
            && self.source == source
            && self.physical == physical
            && self.preparation == preparation
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
        self.matches_context(producer, member, source)
            && self
                .custody
                .as_ref()
                .is_some_and(|custody| match &custody.native {
                    NativeMember::Fit {
                        report: original,
                        assessment: local,
                        ..
                    } => Arc::ptr_eq(original, report) && Arc::ptr_eq(local, assessment),
                    NativeMember::Simulation(_) => false,
                })
    }
    fn matches_context(
        &self,
        producer: &RunResult,
        member: InstanceId,
        source: &ModelingSimulation,
    ) -> bool {
        let attempt = match producer.durability() {
            RunDurability::Ephemeral => None,
            RunDurability::Durable(record) => Some(record.attempt_id),
        };
        self.records_original(
            producer.run_id,
            attempt,
            member,
            source.source.revision.identity().as_id(),
            source.source.physical.identity(),
            source.identity(),
        ) && source.solved.instance() == member
    }
    pub(in crate::workflow) fn matches_simulation(
        &self,
        producer: &RunResult,
        trajectory: &ModelingTrajectory,
    ) -> bool {
        let RunOutcome::Simulation {
            request,
            result: Ok(original),
        } = &producer.outcome
        else {
            return false;
        };
        trajectory.run_id() == producer.run_id
            && original.same_original(trajectory)
            && trajectory.matches_preparation(request)
            && self.matches_context(producer, request.solved.instance(), request)
            && self
                .custody
                .as_ref()
                .is_some_and(|custody| match &custody.native {
                    NativeMember::Simulation(retained) => retained.same_original(trajectory),
                    NativeMember::Fit { .. } => false,
                })
    }
    fn capture(
        run_id: RunId,
        attempt_id: Option<AttemptId>,
        source: &ModelingSimulation,
        succeeded: bool,
        native: NativeMember,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        let local = native.completion();
        let mut refusals = Vec::with_capacity(local.decision.refusals.len() + 1);
        refusals.extend_from_slice(&local.decision.refusals);
        if !succeeded && !refusals.contains(&CandidateRefusal::NativeOutcome) {
            refusals.push(CandidateRefusal::NativeOutcome);
        }
        Self {
            run_id,
            attempt_id,
            member: source.solved.instance(),
            source: source.source.revision.identity().as_id(),
            physical: source.source.physical.identity(),
            preparation: source.identity(),
            permission: ScientificPermission {
                reference_point: None,
                step: None,
                requested: source.profile().samples.len(),
                observed: native.report().coverage.produced(),
                report_succeeded: succeeded,
                decision: if succeeded {
                    local.decision.usability
                } else {
                    CandidateUse::Unusable
                },
                refusals,
                qualifiers: local.decision.qualifiers.clone(),
            },
            custody: Some(Arc::new(MemberCustody {
                native,
                _metadata: owner,
            })),
        }
    }
}

impl RunResult {
    pub(super) fn capture_member_permissions(
        &self,
        completion: &mut Completion,
        attempt_id: Option<AttemptId>,
    ) -> Result<(), WorkflowError> {
        if let RunOutcome::Simulation {
            request,
            result: Ok(trajectory),
        } = &self.outcome
        {
            if trajectory.run_id() != self.run_id || !trajectory.matches_preparation(request) {
                return Ok(());
            }
            let bytes = permission_bytes(trajectory.completion())
                .ok_or_else(|| crate::workflow::contract("original member permission extent"))?;
            let owner = self
                .services
                .preparation()
                .math()
                .reserve("completion:original-member-permission", bytes)?;
            completion.member_permissions = vec![MemberScientificPermission::capture(
                self.run_id,
                attempt_id,
                request,
                trajectory.report().error.is_none()
                    && trajectory.validation_error().is_none()
                    && trajectory.checks_complete(),
                NativeMember::Simulation(trajectory.as_ref().clone()),
                owner,
            )];
            return Ok(());
        }
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
                Some((
                    count.checked_add(1)?,
                    bytes.checked_add(permission_bytes(&local.completion)?)?,
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
        for (_, source, state, local, trajectory) in selected() {
            let succeeded =
                state == FitExperimentAssessmentState::Completed && trajectory.error.is_none();
            permissions.push(MemberScientificPermission::capture(
                self.run_id,
                attempt_id,
                source,
                succeeded,
                NativeMember::Fit {
                    report: trajectory.clone(),
                    assessment: local.clone(),
                    _result: self._owner.clone(),
                },
                owner.clone(),
            ));
        }
        completion.member_permissions = permissions;
        Ok(())
    }
}
