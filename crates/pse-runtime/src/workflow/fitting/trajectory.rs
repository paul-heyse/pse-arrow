// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed original fit trajectories retain their actual joined producer.

use super::{Experiment, InstanceId, native};
use crate::workflow::{RunOutcome, RunResult, ScientificPermission};
use pse_model::generated::identities::RunId;

/// Raw trajectory selected from one original experiment of an exact joined fit.
///
/// The borrow retains the actual producer, including its final report selection and
/// allocation owner. It is not a portable identity or a scientific product grant.
#[derive(Clone, Copy, Debug)]
pub struct FitTrajectory<'a> {
    producer: &'a RunResult,
    assessment: super::FitExperimentAssessment<'a>,
    report: &'a native::dynamics::Report,
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
            report: report.trajectories.get(&experiment)?.as_ref(),
        })
    }
}
