// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed original fit trajectories retain their actual joined producer.

use super::{Experiment, InstanceId, PreparedFit, native};
use crate::workflow::{
    MemberScientificPermission, ModelingSimulation, RunOutcome, RunResult, ScientificPermission,
    WorkflowError,
};
use pse_model::generated::identities::RunId;
use std::sync::Arc;

/// Raw trajectory selected from one original experiment of an exact joined fit.
///
/// The borrow retains the actual producer, including its final report selection and
/// allocation owner. It is not a portable identity or a scientific product grant.
#[derive(Clone, Copy, Debug)]
pub struct FitTrajectory<'a> {
    producer: &'a RunResult,
    assessment: super::FitExperimentAssessment<'a>,
    report: &'a Arc<native::dynamics::Report>,
    source: &'a ModelingSimulation,
}

impl FitTrajectory<'_> {
    /// Original scientific execution identity; retries may share this run identity.
    pub fn run_id(&self) -> RunId {
        self.producer.run_id
    }

    /// Original canonical run when registration was acknowledged.
    pub fn canonical_run_key(&self) -> Option<&str> {
        self.producer.canonical_run_key()
    }

    /// Exact canonical attempt when the producer recorded one.
    /// Ephemeral observations do not invent a durable attempt identity.
    pub fn canonical_attempt_key(&self) -> Option<&str> {
        self.producer.canonical_attempt_key()
    }

    /// Original experiment, not an ordinal or a trajectory-map position.
    pub fn experiment_id(&self) -> InstanceId {
        self.assessment.experiment_id()
    }

    /// Original local assessment evidence. Completion is not a scientific use grant.
    pub fn assessment(&self) -> super::FitExperimentAssessment<'_> {
        self.assessment
    }

    /// Already-produced raw trajectory, including incomplete or unusable evidence.
    /// Reading it never repeats integration or revises final selection.
    pub fn report(&self) -> &native::dynamics::Report {
        self.report
    }
    /// Exact local integration inputs used by original assessment, not fit-estimate permission.
    pub fn parameters(&self) -> Option<&[f64]> {
        self.assessment
            .trajectory()
            .map(|local| local.parameters.as_slice())
    }
    /// Original source/layout coordinates with the actually applied integration inputs.
    pub fn scientific_coordinates(
        &self,
    ) -> Result<crate::workflow::ReviewScientificCoordinates, WorkflowError> {
        self.source.review_coordinates_with_parameters(
            self.run_id(),
            self.parameters()
                .ok_or_else(|| crate::workflow::contract("original member inputs unavailable"))?,
        )
    }
    /// Independent original whole-member permission, bound to this actual producer/report.
    pub fn governing_permission(&self) -> Option<&MemberScientificPermission> {
        let assessment = self.assessment.trajectory()?;
        self.producer
            .completion()
            .ok()?
            .member_permission(self.experiment_id())
            .filter(|permission| {
                permission.matches(
                    self.producer,
                    self.experiment_id(),
                    self.source,
                    self.report,
                    assessment,
                )
            })
    }

    /// Original trajectory available for scientific use under its own native decision.
    /// Raw failed and partial observations remain available through [`Self::report`].
    pub fn qualified_report(&self) -> Option<&native::dynamics::Report> {
        self.governing_permission()
            .filter(|permission| permission.permits_result())
            .map(|_| self.report())
    }

    /// This producer's original whole-fit decision, if captured.
    ///
    /// This observation does not issue experiment/trajectory product permission or
    /// promote a sample's local validity to scientific use.
    pub fn governing_fit_permission(&self) -> Option<&ScientificPermission> {
        self.producer.completion().ok()?.permission.as_ref()
    }

    /// Whether this reference belongs to this exact joined producer.
    /// Equal run keys alone cannot establish the selected attempt/report.
    pub fn belongs_to(&self, producer: &RunResult) -> bool {
        std::ptr::eq(self.producer, producer)
    }
}

impl RunResult {
    /// Inspect the retained final trajectory of one original transient experiment.
    ///
    /// Foreign/ambiguous experiment declarations, steady experiments, absent final
    /// reports and missing trajectories remain unavailable. Governing fit failure
    /// does not hide raw evidence. Only this joined outcome supplies the selection;
    /// optimizer callback caches and caller-supplied maps cannot construct it.
    pub fn fit_trajectory(&self, experiment: InstanceId) -> Option<FitTrajectory<'_>> {
        let RunOutcome::Fit {
            request,
            result: Ok(report),
        } = &self.outcome
        else {
            return None;
        };
        let mut declarations = request
            .problem
            .declaration
            .experiments
            .iter()
            .enumerate()
            .filter(|(_, declaration)| declaration.experiment_id == experiment);
        let (index, _) = declarations.next()?;
        if declarations.next().is_some()
            || !matches!(
                request.problem.experiments.get(index),
                Some(Experiment::Transient(_))
            )
        {
            return None;
        }
        Some(FitTrajectory {
            producer: self,
            assessment: report.assessment.experiment(experiment)?,
            report: report.trajectories.get(&experiment)?,
            source: request.trajectory_source(experiment)?,
        })
    }
}

impl PreparedFit {
    pub(in crate::workflow) fn trajectory_source(
        &self,
        member: InstanceId,
    ) -> Option<&ModelingSimulation> {
        let mut declarations = self
            .problem
            .declaration
            .experiments
            .iter()
            .enumerate()
            .filter(|(_, declaration)| declaration.experiment_id == member);
        let (index, _) = declarations.next()?;
        if declarations.next().is_some()
            || !matches!(
                self.problem.experiments.get(index),
                Some(Experiment::Transient(_))
            )
        {
            return None;
        }
        match self.assessments.get(index)? {
            super::modeling::Assessment::Transient(source)
                if source.solved.instance() == member =>
            {
                Some(source)
            }
            _ => None,
        }
    }
}
