// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original experiment assessment evidence, independently of whole-fit qualification.
use super::InstanceId;
use crate::workflow::{ModelingCheck, ModelingReport};
use pse_model::diagnostic::BoundaryDiagnostic;
use std::{ops::Range, sync::Arc};

/// Registry-owned original assessment status, independent of scientific permission.
pub use pse_model::generated::enums::FitExperimentAssessmentState;

#[derive(Clone, Debug)]
struct ExperimentAssessment {
    experiment: InstanceId,
    state: FitExperimentAssessmentState,
    failure: Option<Arc<BoundaryDiagnostic>>,
    checks: Range<usize>,
    reports: Range<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowError, contract};
    use pse_model::diagnostic::BoundaryClass;

    #[test]
    fn fitting_experiment_assessment_preserves_every_failure_and_terminal_primary() {
        let ids = [
            InstanceId::from(pse_ids::SemanticId::from_bytes([1; 16])),
            InstanceId::from(pse_ids::SemanticId::from_bytes([2; 16])),
        ];
        let mut assessment = FitAssessment::unattempted(ids);
        let mut first = contract("first independent numerical check").boundary_diagnostic();
        first.class = BoundaryClass::Numerical;
        let mut second = contract("second independent terminal check").boundary_diagnostic();
        second.class = BoundaryClass::Infrastructure;
        assessment.record(0, vec![], vec![], Some(first));
        assessment.record(1, vec![], vec![], Some(second));
        assert!(!assessment.complete());
        assert_eq!(
            assessment
                .failures()
                .map(|(id, _)| id.unwrap())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(
            assessment.failure().unwrap().class,
            BoundaryClass::Infrastructure
        );
        let observation = assessment.observation().unwrap();
        assert_eq!(observation.experiments.len(), 2);
        assert!(observation.experiments.iter().all(|member| member.state
            == FitExperimentAssessmentState::Failed
            && member.failure.is_some()));
        let original_bytes = assessment.bytes();
        assert!(original_bytes > 2 * size_of::<ExperimentAssessment>());
        assert!(assessment.observation_working_bytes() > 0);
        let mut numerical_interruption =
            contract("assessment-wide numerical interruption").boundary_diagnostic();
        numerical_interruption.class = BoundaryClass::Numerical;
        assessment.interrupt(numerical_interruption);
        assert_eq!(
            assessment.failure().unwrap().class,
            BoundaryClass::Infrastructure
        );
        let cancelled =
            WorkflowError::Math(crate::math::MathRuntimeError::Cancelled).boundary_diagnostic();
        assessment.interrupt(cancelled);
        assert_eq!(assessment.failures().count(), 3);
        assert_eq!(
            assessment.failure().unwrap().class,
            BoundaryClass::Cancelled
        );
        assert_eq!(
            assessment
                .observation()
                .unwrap()
                .interruption
                .unwrap()
                .class,
            BoundaryClass::Cancelled
        );
        let ambiguous = FitAssessment::unattempted([ids[0], ids[0]]);
        assert!(ambiguous.experiment(ids[0]).is_none());
    }
}

impl PartialEq for ExperimentAssessment {
    fn eq(&self, other: &Self) -> bool {
        self.experiment == other.experiment
            && self.state == other.state
            && self.checks == other.checks
            && self.reports == other.reports
            && match (&self.failure, &other.failure) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
    }
}

/// One original candidate's assessment. Member ranges refer to these sole owned values.
/// No method issues local experiment, trajectory, or global estimate permission.
#[derive(Clone, Debug)]
pub struct FitAssessment {
    checks: Vec<ModelingCheck>,
    reports: Vec<ModelingReport>,
    experiments: Vec<ExperimentAssessment>,
    interruption: Option<Arc<BoundaryDiagnostic>>,
}
impl PartialEq for FitAssessment {
    fn eq(&self, other: &Self) -> bool {
        self.checks == other.checks
            && self.reports == other.reports
            && self.experiments == other.experiments
            && match (&self.interruption, &other.interruption) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
    }
}

/// Borrowed evidence attributed to one exact declared experiment by its original producer.
#[derive(Clone, Copy, Debug)]
pub struct FitExperimentAssessment<'a> {
    assessment: &'a FitAssessment,
    member: &'a ExperimentAssessment,
}
impl FitExperimentAssessment<'_> {
    /// Exact original declared instance, never a row position or generated alias.
    pub fn experiment_id(&self) -> InstanceId {
        self.member.experiment
    }
    /// Coverage of the original assessment, independent of check satisfaction.
    pub fn state(&self) -> FitExperimentAssessmentState {
        self.member.state
    }
    /// Original attributable failure; unattempted declarations invent no error.
    pub fn failure(&self) -> Option<&BoundaryDiagnostic> {
        self.member.failure.as_deref()
    }
    /// Already-produced original checks, including partial evidence after failure.
    pub fn checks(&self) -> &[ModelingCheck] {
        &self.assessment.checks[self.member.checks.clone()]
    }
    /// Already-produced original reports, including partial evidence after failure.
    pub fn reports(&self) -> &[ModelingReport] {
        &self.assessment.reports[self.member.reports.clone()]
    }
    pub(super) fn observation_working_bytes(&self) -> usize {
        size_of::<pse_model::generated::structures::FitExperimentAssessmentObservation>()
            .saturating_add(self.failure().map_or(0, diagnostic_working_bytes))
    }
    pub(super) fn observation(
        &self,
    ) -> Result<
        pse_model::generated::structures::FitExperimentAssessmentObservation,
        crate::workflow::WorkflowError,
    > {
        let ordinal = |value| {
            i64::try_from(value)
                .map_err(|_| crate::workflow::contract("original experiment assessment row extent"))
        };
        Ok(
            pse_model::generated::structures::FitExperimentAssessmentObservation {
                experiment_id: self.member.experiment,
                state: self.member.state,
                check_start: ordinal(self.member.checks.start)?,
                check_end: ordinal(self.member.checks.end)?,
                report_start: ordinal(self.member.reports.start)?,
                report_end: ordinal(self.member.reports.end)?,
                failure: self
                    .failure()
                    .map(crate::workflow::diagnostic_rows::fit_experiment_assessment_failure),
            },
        )
    }
}
impl serde::Serialize for FitAssessment {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(
            &self.observation().map_err(serde::ser::Error::custom)?,
            serializer,
        )
    }
}
impl FitAssessment {
    pub(super) fn unattempted(experiments: impl IntoIterator<Item = InstanceId>) -> Self {
        Self {
            checks: Vec::new(),
            reports: Vec::new(),
            interruption: None,
            experiments: experiments
                .into_iter()
                .map(|experiment| ExperimentAssessment {
                    experiment,
                    state: FitExperimentAssessmentState::Unattempted,
                    failure: None,
                    checks: 0..0,
                    reports: 0..0,
                })
                .collect(),
        }
    }
    /// The original declared-order population, including failed and unattempted members.
    pub fn experiments(&self) -> impl ExactSizeIterator<Item = FitExperimentAssessment<'_>> {
        self.experiments
            .iter()
            .map(|member| FitExperimentAssessment {
                assessment: self,
                member,
            })
    }
    /// Select only an unambiguous original member. Duplicate identity never picks a row.
    pub fn experiment(&self, experiment: InstanceId) -> Option<FitExperimentAssessment<'_>> {
        let mut selected = self
            .experiments()
            .filter(|member| member.experiment_id() == experiment);
        let result = selected.next()?;
        selected.next().is_none().then_some(result)
    }
    /// Sole owned aggregate original checks in declaration/assessment order.
    pub fn checks(&self) -> &[ModelingCheck] {
        &self.checks
    }
    /// Sole owned aggregate original reports in declaration/assessment order.
    pub fn reports(&self) -> &[ModelingReport] {
        &self.reports
    }
    /// Registry-owned passive observation; native row buffers remain with this assessment.
    pub fn observation(
        &self,
    ) -> Result<
        pse_model::generated::structures::FitOriginalAssessmentObservation,
        crate::workflow::WorkflowError,
    > {
        Ok(
            pse_model::generated::structures::FitOriginalAssessmentObservation {
                complete: self.complete(),
                experiments: self
                    .experiments()
                    .map(|member| member.observation())
                    .collect::<Result<_, _>>()?,
                interruption: self
                    .interruption
                    .as_deref()
                    .map(crate::workflow::diagnostic_rows::fit_experiment_assessment_failure),
            },
        )
    }
    /// Working population of this mechanical projection, excluding unprojected row values.
    pub(super) fn observation_working_bytes(&self) -> usize {
        use pse_model::generated::structures::{
            FitExperimentAssessmentObservation, FitOriginalAssessmentObservation,
        };
        let diagnostics = self
            .failures()
            .map(|(_, diagnostic)| diagnostic_working_bytes(diagnostic))
            .fold(0usize, usize::saturating_add);
        size_of::<FitOriginalAssessmentObservation>()
            .saturating_add(
                self.experiments
                    .len()
                    .saturating_mul(size_of::<FitExperimentAssessmentObservation>()),
            )
            .saturating_add(diagnostics)
    }
    /// Every declared experiment completed original assessment without interruption.
    pub fn complete(&self) -> bool {
        self.interruption.is_none()
            && !self.experiments.is_empty()
            && self
                .experiments
                .iter()
                .all(|member| matches!(member.state, FitExperimentAssessmentState::Completed))
    }
    /// Original member failure, or an assessment-wide refusal before/between members.
    pub fn failure(&self) -> Option<&BoundaryDiagnostic> {
        let failures = || {
            self.interruption.as_deref().into_iter().chain(
                self.experiments
                    .iter()
                    .filter_map(|member| member.failure.as_deref()),
            )
        };
        failures()
            .find(|failure| {
                !crate::math::strategy::permits_numerical_continuation(
                    crate::math::strategy::boundary_failure(failure.class),
                )
            })
            .or_else(|| failures().next())
    }
    pub(in crate::workflow) fn failures(
        &self,
    ) -> impl Iterator<Item = (Option<InstanceId>, &BoundaryDiagnostic)> {
        self.experiments
            .iter()
            .filter_map(|member| {
                member
                    .failure
                    .as_deref()
                    .map(|error| (Some(member.experiment), error))
            })
            .chain(self.interruption.as_deref().map(|error| (None, error)))
    }
    pub(super) fn interrupt(&mut self, error: BoundaryDiagnostic) {
        self.interruption = Some(Arc::new(error));
    }
    pub(super) fn record(
        &mut self,
        index: usize,
        checks: Vec<ModelingCheck>,
        reports: Vec<ModelingReport>,
        failure: Option<BoundaryDiagnostic>,
    ) {
        let check_start = self.checks.len();
        let report_start = self.reports.len();
        self.checks.extend(checks);
        self.reports.extend(reports);
        let member = &mut self.experiments[index];
        member.checks = check_start..self.checks.len();
        member.reports = report_start..self.reports.len();
        member.state = if failure.is_some() {
            FitExperimentAssessmentState::Failed
        } else {
            FitExperimentAssessmentState::Completed
        };
        member.failure = failure.map(Arc::new);
    }
    pub(super) fn bytes(&self) -> usize {
        use pse_model::HeapUsage;
        self.checks.capacity() * size_of::<ModelingCheck>()
            + self.checks.iter().map(HeapUsage::heap_bytes).sum::<usize>()
            + self.reports.capacity() * size_of::<ModelingReport>()
            + self
                .reports
                .iter()
                .map(HeapUsage::heap_bytes)
                .sum::<usize>()
            + self.experiments.capacity() * size_of::<ExperimentAssessment>()
            + self
                .experiments
                .iter()
                .filter_map(|member| member.failure.as_ref())
                .chain(self.interruption.iter())
                .map(|error| size_of::<BoundaryDiagnostic>() + error.heap_bytes())
                .sum::<usize>()
    }
}

fn diagnostic_working_bytes(diagnostic: &BoundaryDiagnostic) -> usize {
    use pse_model::{
        HeapUsage,
        generated::structures::{
            FitExperimentAssessmentFailure, FitExperimentAssessmentFailureCausesItem,
        },
    };
    fn tree(diagnostic: &BoundaryDiagnostic, depth: usize) -> (usize, usize, usize) {
        diagnostic
            .causes
            .iter()
            .fold((1, depth, depth), |(nodes, paths, deepest), child| {
                let (n, p, d) = tree(child, depth.saturating_add(1));
                (
                    nodes.saturating_add(n),
                    paths.saturating_add(p),
                    deepest.max(d),
                )
            })
    }
    let (nodes, path_items, depth) = tree(diagnostic, 0);
    // Every source dynamic value is copied once. Output envelopes and the
    // flattening scratch/path Vec populations are additional known Rust storage.
    diagnostic
        .heap_bytes()
        .saturating_add(size_of::<FitExperimentAssessmentFailure>())
        .saturating_add(
            nodes.saturating_mul(
                size_of::<FitExperimentAssessmentFailureCausesItem>()
                    .saturating_add(2 * size_of::<(Vec<i64>, &BoundaryDiagnostic)>()),
            ),
        )
        .saturating_add(
            path_items
                .saturating_add(depth)
                .saturating_mul(2 * size_of::<i64>()),
        )
}
