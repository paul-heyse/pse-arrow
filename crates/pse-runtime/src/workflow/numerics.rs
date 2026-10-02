// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Numerical preparation and immutable completion assessment, shared by public workflows.
use super::{RunReport, RunRequest, RunResult, WorkflowError, relation};
use pse_backend_native::solve::SolveReport;
use pse_ids::SemanticId;
use pse_model::{
    generated::enums::{
        CandidateBoundOrigin, CandidateQualifier, CandidateRefusal, CandidateUse,
        ClosureAssessment, ClosurePolicy, IncumbentPolicy, NativeTermination,
    },
    numerics::ResolvedNumericalPolicy,
};
use pse_relations::generated::runtime::{
    candidate_assessments as assessments, resolved_numerics as resolved,
};
use std::collections::BTreeMap;

/// Independent original-objective bound facts retained with the completion decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoundEvidence {
    pub(crate) origin: CandidateBoundOrigin,
    pub(crate) bound: f64,
    pub(crate) absolute_gap: f64,
    pub(crate) relative_gap: Option<f64>,
    pub(crate) within_gap: bool,
}
/// One composed decision; projections consume permissions without reconstructing science.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CandidateDecision {
    pub(crate) usability: CandidateUse,
    pub(crate) qualifiers: Vec<CandidateQualifier>,
    pub(crate) refusals: Vec<CandidateRefusal>,
    pub(crate) bound: Option<BoundEvidence>,
}
impl CandidateDecision {
    fn new(usability: CandidateUse) -> Self {
        Self {
            usability,
            qualifiers: Vec::new(),
            refusals: Vec::new(),
            bound: None,
        }
    }
    pub(crate) const fn permits_use(&self) -> bool {
        matches!(
            self.usability,
            CandidateUse::Usable | CandidateUse::QualifiedUnclosed
        )
    }
    pub(crate) const fn permits_seed(&self) -> bool {
        self.permits_use() || matches!(self.usability, CandidateUse::SeedOnly)
    }
    pub(crate) fn refuse(&mut self, reason: CandidateRefusal) {
        if !self.refusals.contains(&reason) {
            self.refusals.push(reason);
        }
        if !matches!(self.usability, CandidateUse::DiagnosticOnly) {
            self.usability = CandidateUse::Unusable;
        }
    }
    /// Derived presentation; typed reasons remain the authority.
    pub(crate) fn reason(&self) -> String {
        let parts: Vec<_> = self
            .qualifiers
            .iter()
            .map(|r| r.as_str())
            .chain(self.refusals.iter().map(|r| r.as_str()))
            .collect();
        if parts.is_empty() {
            "required original-coordinate checks passed".into()
        } else {
            parts.join("; ")
        }
    }
}
/// The existing absolute OR same-sign relative criterion in original objective units.
fn global_bound(report: &SolveReport) -> Option<BoundEvidence> {
    let (origin, sense, bound, absolute, relative) = if let Some(g) = report.evidence.global {
        if !g.readback || report.evidence.contradiction.is_some() {
            return None;
        }
        (
            CandidateBoundOrigin::GlobalExport,
            g.sense,
            g.dual_bound?,
            g.gap_absolute,
            g.gap_relative,
        )
    } else {
        let bound = report.evidence.original_bound?;
        (
            bound.origin,
            bound.sense,
            bound.value,
            bound.absolute_tolerance,
            bound.relative_tolerance,
        )
    };
    let objective = report
        .observation
        .as_ref()?
        .objective
        .filter(|v| v.is_finite())?;
    let ordered = match sense {
        pse_math::binding::ObjectiveSense::Minimize => bound <= objective,
        pse_math::binding::ObjectiveSense::Maximize => bound >= objective,
    };
    if !ordered
        || !bound.is_finite()
        || !absolute.is_finite()
        || absolute < 0.
        || !relative.is_finite()
        || relative < 0.
    {
        return None;
    }
    let absolute_gap = (objective - bound).abs();
    if !absolute_gap.is_finite() {
        return None;
    }
    let denominator = objective.abs().min(bound.abs());
    let relative_gap = if objective.signum() == bound.signum() && denominator > 0. {
        (absolute_gap / denominator)
            .is_finite()
            .then_some(absolute_gap / denominator)
    } else if absolute_gap == 0. {
        Some(0.)
    } else {
        None
    };
    Some(BoundEvidence {
        origin,
        bound,
        absolute_gap,
        relative_gap,
        within_gap: absolute_gap <= absolute || relative_gap.is_some_and(|gap| gap <= relative),
    })
}
/// Native termination remains independent from validated original quality and policy.
pub(crate) fn native_use(
    report: &SolveReport,
    policy: &pse_model::numerics::NumericalPolicy,
) -> CandidateDecision {
    use CandidateRefusal as R;
    use NativeTermination as T;
    let mut decision = CandidateDecision::new(CandidateUse::Usable);
    decision.bound = global_bound(report);
    if report.candidate.is_none() {
        decision.refuse(R::NoCandidate);
    }
    if report.validation_failure().is_some() || report.callback_failure().is_some() {
        decision.refuse(R::ValidationFailed);
    }
    if !report
        .quality
        .as_ref()
        .is_some_and(pse_backend_native::quality::Quality::feasible)
    {
        decision.refuse(R::Infeasible);
    }
    if report.qualification == pse_backend_native::solve::Qualification::Unqualified {
        decision.refuse(R::Unqualified);
    }
    let diagnostic = if report
        .evidence
        .global
        .is_some_and(|g| g.primal == pse_backend_native::solve::PrimalSource::RelaxedIncumbent)
    {
        Some(R::RelaxedIncumbent)
    } else if report.termination.category == T::Infeasible {
        Some(R::LeastInfeasible)
    } else {
        None
    };
    if let Some(reason) = diagnostic {
        decision.refusals.push(reason);
        decision.usability = if report.candidate.is_some() {
            CandidateUse::DiagnosticOnly
        } else {
            CandidateUse::Unusable
        };
        return decision;
    }
    match report.termination.category {
        T::Success | T::Acceptable | T::FeasibleOnly => {}
        T::TimeLimit | T::NodeLimit | T::IterationLimit | T::SolutionLimit => {
            if decision.permits_use() {
                match policy.incumbent {
                    IncumbentPolicy::Refuse => {
                        decision.usability = CandidateUse::SeedOnly;
                        decision.refusals.push(R::IncumbentRefused);
                    }
                    IncumbentPolicy::AcceptFeasible => decision
                        .qualifiers
                        .push(CandidateQualifier::AcceptedIncumbentFeasible),
                    IncumbentPolicy::AcceptWithinGap => match decision.bound {
                        Some(bound) if bound.within_gap => decision
                            .qualifiers
                            .push(CandidateQualifier::AcceptedIncumbentWithinGap),
                        Some(_) => {
                            decision.usability = CandidateUse::SeedOnly;
                            decision.refusals.push(R::GapExceeded);
                        }
                        None => {
                            decision.usability = CandidateUse::SeedOnly;
                            decision.refusals.push(R::BoundUnavailable);
                        }
                    },
                }
            }
        }
        T::Limit | T::ObjectiveLimit | T::ResourceExhausted | T::Inconclusive | T::Numerical => {
            if decision.permits_use() {
                decision.usability = CandidateUse::SeedOnly;
            }
            decision.refusals.push(R::NativeOutcome);
        }
        T::Cancelled
        | T::Evaluation
        | T::Panic
        | T::Invalid
        | T::Unbounded
        | T::InfeasibleOrUnbounded => decision.refuse(R::NativeOutcome),
        T::Infeasible => {} // handled as an independent diagnostic source above
    }
    decision
}
pub(crate) fn constant_use(quality: &pse_backend_native::quality::Quality) -> CandidateDecision {
    if quality.feasible() {
        CandidateDecision::new(CandidateUse::Usable)
    } else {
        refused(CandidateRefusal::Infeasible)
    }
}
/// An event-ended prefix grants a result only for its admitted actual endpoint.
pub(crate) fn trajectory_use(
    report: &pse_backend_native::dynamics::Report,
    endpoint_satisfied: bool,
) -> CandidateDecision {
    if report.error.is_some() {
        return refused(CandidateRefusal::NativeOutcome);
    }
    if !endpoint_satisfied {
        return refused(CandidateRefusal::EndpointUnavailable);
    }
    match report.termination {
        pse_backend_native::dynamics::Termination::Completed
        | pse_backend_native::dynamics::Termination::Event => {
            CandidateDecision::new(CandidateUse::Usable)
        }
        _ => refused(CandidateRefusal::NativeOutcome),
    }
}
pub(crate) fn refused(reason: CandidateRefusal) -> CandidateDecision {
    let mut decision = CandidateDecision::new(CandidateUse::Unusable);
    decision.refusals.push(reason);
    decision
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Completed {
    pub(crate) closure: ClosureAssessment,
    pub(crate) decision: CandidateDecision,
}
impl Completed {
    /// Copy the immutable decision and independent source evidence into its registry row.
    pub(crate) fn assessment_row(
        &self,
        run_id: pse_model::generated::identities::RunId,
        step: i64,
        policy: &pse_model::numerics::NumericalPolicy,
        report: Option<&SolveReport>,
        numerical: Option<bool>,
        constant: bool,
    ) -> assessments::Row {
        use pse_model::generated::enums::{NativeCandidateKind, NativeQualification};
        let decision = &self.decision;
        assessments::Row {
            run_id,
            step,
            native_termination: report.map(|r| r.termination.category),
            numerically_feasible: numerical,
            closure: self.closure,
            policy: policy.closure,
            usability: decision.usability,
            reason: decision.reason(),
            incumbent_policy: policy.incumbent,
            candidate_kind: report
                .and_then(|r| r.candidate.as_ref())
                .map(|c| c.kind)
                .or_else(|| constant.then_some(NativeCandidateKind::ConstantEvaluation)),
            qualification: report.map(|r| r.qualification).or_else(|| {
                constant.then_some(if numerical == Some(true) {
                    NativeQualification::Feasible
                } else {
                    NativeQualification::Unqualified
                })
            }),
            validated: report
                .map(|r| {
                    r.quality.is_some()
                        && r.validation_failure().is_none()
                        && r.callback_failure().is_none()
                })
                .or_else(|| constant.then_some(numerical.is_some())),
            bound_origin: decision.bound.map(|b| b.origin),
            bound: decision.bound.map(|b| b.bound),
            absolute_gap: decision.bound.map(|b| b.absolute_gap),
            relative_gap: decision.bound.and_then(|b| b.relative_gap),
            permits_result: decision.permits_use(),
            permits_seed: decision.permits_seed(),
            qualifiers: decision.qualifiers.clone(),
            refusals: decision.refusals.clone(),
        }
    }
    pub(crate) const fn permits_use(&self) -> bool {
        self.decision.permits_use()
    }
}
/// Explicit requirements prevent missing closure evidence from becoming NotRequired.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CompletionEvidence<'a> {
    pub(crate) checks: &'a [super::ModelingCheck],
    pub(crate) checks_complete: bool,
    pub(crate) required_closure: usize,
    pub(crate) endpoint_satisfied: Option<bool>,
    pub(crate) coverage_complete: bool,
}
impl<'a> CompletionEvidence<'a> {
    pub(crate) const fn point(
        checks: &'a [super::ModelingCheck],
        checks_complete: bool,
        required_closure: usize,
    ) -> Self {
        Self {
            checks,
            checks_complete,
            required_closure,
            endpoint_satisfied: None,
            coverage_complete: true,
        }
    }
}
/// Compose independent native evidence, original obligations, closure and applicability once.
pub(crate) fn complete(
    mut native: CandidateDecision,
    evidence: CompletionEvidence<'_>,
    policy: &pse_model::numerics::NumericalPolicy,
) -> Completed {
    use pse_model::generated::enums::{ModelingApplicabilityOutcome as A, ModelingCheckKind as K};
    let mut closure_count = 0;
    let mut closure_failed = false;
    if !evidence.checks_complete {
        native.refuse(CandidateRefusal::ModelChecks);
    }
    if evidence.endpoint_satisfied == Some(false) {
        native.refuse(CandidateRefusal::EndpointUnavailable);
    }
    if !evidence.coverage_complete {
        native.refuse(CandidateRefusal::CoverageUnavailable);
    }
    for check in evidence.checks {
        match check.kind {
            K::Closure => {
                closure_count += 1;
                closure_failed |= !check.satisfied;
            }
            K::Applicability => {
                if !check.satisfied {
                    native.refuse(CandidateRefusal::ApplicabilityDenied);
                }
                match check.applicability_outcome {
                    Some(A::UnknownEvidence)
                        if check.satisfied && check.unknown_allowed == Some(true) =>
                    {
                        if !native
                            .qualifiers
                            .contains(&CandidateQualifier::ApplicabilityUnknownAllowed)
                        {
                            native
                                .qualifiers
                                .push(CandidateQualifier::ApplicabilityUnknownAllowed);
                        }
                    }
                    Some(A::OutsideRegion)
                        if check.satisfied && check.extrapolation_allowed == Some(true) =>
                    {
                        if !native
                            .qualifiers
                            .contains(&CandidateQualifier::ApplicabilityExtrapolationAllowed)
                        {
                            native
                                .qualifiers
                                .push(CandidateQualifier::ApplicabilityExtrapolationAllowed);
                        }
                    }
                    Some(A::UnknownEvidence) if check.applicability_required == Some(true) => {
                        native.refuse(CandidateRefusal::ApplicabilityDenied)
                    }
                    Some(A::OutsideRegion) => native.refuse(CandidateRefusal::ApplicabilityDenied),
                    None if check.applicability_required == Some(true) => {
                        native.refuse(CandidateRefusal::ApplicabilityUnavailable)
                    }
                    _ => {}
                }
            }
            _ if !check.satisfied => native.refuse(CandidateRefusal::ModelChecks),
            _ => {}
        }
    }
    let closure = if evidence.required_closure > closure_count
        || (evidence.required_closure > 0 && !evidence.checks_complete)
    {
        native.refuse(CandidateRefusal::ClosureUnavailable);
        ClosureAssessment::Unavailable
    } else if closure_count == 0 {
        ClosureAssessment::NotRequired
    } else if closure_failed {
        if policy.closure == ClosurePolicy::AllowUnclosed {
            native.qualifiers.push(CandidateQualifier::ClosureAllowed);
            if native.permits_use() {
                native.usability = CandidateUse::QualifiedUnclosed;
            }
        } else {
            native.refuse(CandidateRefusal::ClosureUnclosed);
        }
        ClosureAssessment::Unclosed
    } else {
        ClosureAssessment::Closed
    };
    Completed {
        closure,
        decision: native,
    }
}
impl RunResult {
    pub(super) fn assess_candidates(&self) -> Vec<assessments::Row> {
        let count = match &self.request {
            RunRequest::Modeling(s) => s.len(),
            _ => 1,
        };
        let default = pse_model::numerics::NumericalPolicy::default();
        (0..count)
            .map(|step| {
                let unavailable = || Completed {
                    closure: ClosureAssessment::Unavailable,
                    decision: refused(CandidateRefusal::ModelChecks),
                };
                let (report, numerical, constant, policy, completed) =
                    match (&self.request, &self.report) {
                        (RunRequest::Fit(p), Ok(RunReport::Fit(r))) => {
                            let policy = &p.problem.numerics.policy;
                            (
                                r.solve.as_ref(),
                                r.quality.as_ref().map(|q| q.feasible()),
                                r.solve.is_none() && r.candidate.is_some(),
                                policy,
                                complete(
                                    r.candidate_use(policy),
                                    CompletionEvidence::point(
                                        &r.checks,
                                        r.checks_complete && r.validation_error.is_none(),
                                        p.required_closure_checks(),
                                    ),
                                    policy,
                                ),
                            )
                        }
                        (RunRequest::Modeling(p), Ok(RunReport::Modeling(results)))
                            if results.get(step).is_some() =>
                        {
                            let r = &results[step];
                            let (report, numerical, constant) = match &r.outcome {
                                crate::math::solves::Outcome::Native(n) => (
                                    Some(n.as_ref()),
                                    n.quality.as_ref().map(|q| q.feasible()),
                                    false,
                                ),
                                crate::math::solves::Outcome::Constant(c) => {
                                    (None, Some(c.quality.feasible()), true)
                                }
                                crate::math::solves::Outcome::Rejected(_) => (None, None, false),
                            };
                            (
                                report,
                                numerical,
                                constant,
                                &p[step].solve.numerics().policy,
                                r.completion.clone(),
                            )
                        }
                        (RunRequest::Simulation(p), Ok(RunReport::Simulation(r))) => (
                            None,
                            None,
                            false,
                            &p.numerics().policy,
                            r.completion.clone(),
                        ),
                        #[cfg(feature = "solver-diffsol")]
                        (RunRequest::Shooting { problem: p, .. }, Ok(RunReport::Shooting(r))) => (
                            r.solve.as_ref(),
                            r.quality.as_ref().map(|q| q.feasible()),
                            r.solve.is_none() && r.candidate.is_some(),
                            &p.numerics().policy,
                            r.completion.clone(),
                        ),
                        _ => (None, None, false, &default, unavailable()),
                    };
                completed.assessment_row(
                    self.run_id,
                    step as i64,
                    policy,
                    report,
                    numerical,
                    constant,
                )
            })
            .collect()
    }
    pub(super) fn numerical_tables(
        &self,
        batches: &mut BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>,
    ) -> Result<(), WorkflowError> {
        let registry = &self.runtime.registry;
        let validation = self.runtime.validation_context()?;
        let mut candidates =
            assessments::Builder::with_registry(registry, self.assessments.len(), &validation)
                .map_err(relation)?;
        for row in &self.assessments {
            candidates.push(row.clone()).map_err(relation)?;
        }
        batches.insert(
            assessments::RELATION_ID,
            candidates.finish().map_err(relation)?,
        );
        let policies: Vec<&ResolvedNumericalPolicy> = match &self.request {
            RunRequest::Fit(f) => vec![&f.problem.numerics],
            RunRequest::Simulation(p) => vec![p.numerics()],
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { problem: p, .. } => vec![p.numerics()],
            RunRequest::Modeling(p) => p.iter().map(|p| p.solve.numerics()).collect(),
        };
        let mut resolved =
            resolved::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        for (step, policy) in policies.iter().enumerate() {
            for t in &policy.targets {
                resolved
                    .push(resolved::Row {
                        run_id: self.run_id,
                        step: step as i64,
                        target_id: t.id,
                        target_kind: t.kind,
                        quantity_id: t.quantity,
                        unit_id: t.unit,
                        nominal: t.nominal,
                        coordinate_scale: t.coordinate_scale,
                        absolute: t.absolute,
                        relative: t.relative,
                        budget: t.budget,
                        provenance: t
                            .provenance
                            .iter()
                            .map(|p| resolved::RuntimeResolvedNumericsFieldProvenanceItem {
                                declaration: p.declaration,
                                source: p.source,
                                field: p.field,
                                selected: p.selected,
                                value: p.value,
                                description: p.description.clone(),
                            })
                            .collect(),
                    })
                    .map_err(relation)?;
            }
        }
        batches.insert(resolved::RELATION_ID, resolved.finish().map_err(relation)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::{enums::ModelingCheckKind, identities::RunId};
    fn check(kind: ModelingCheckKind, satisfied: bool) -> super::super::ModelingCheck {
        super::super::ModelingCheck {
            run_id: RunId::from_bytes([1; 16]),
            step: 0,
            sample_index: 0,
            time: None,
            target_id: SemanticId::from_bytes([4; 16]),
            source_id: pse_modeling::DeclarationId::from_bytes([3; 16]),
            kind,
            value: 0.0,
            tolerance: None,
            satisfied,
            within_validity: None,
            extrapolation_allowed: None,
            basis: pse_model::generated::enums::ModelingCheckBasis::Point,
            layer: None,
            claim_id: None,
            claim_owner: None,
            claim_owner_lineage: Vec::new(),
            coverage_id: None,
            evidence_id: None,
            form_id: None,
            call_id: None,
            selected_records: Vec::new(),
            dependencies: Vec::new(),
            input_values: Vec::new(),
            applicability_outcome: None,
            applicability_basis: None,
            permission_ids: Vec::new(),
            unknown_allowed: None,
            observation_instance: None,
            applicability_required: None,
            applicability_reason: None,
            applicability_permissions: Vec::new(),
        }
    }
    fn report(category: NativeTermination) -> SolveReport {
        use pse_backend_native::{self as native, solve::*};
        let id = |v: u8| SemanticId::from_bytes([v; 16]);
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![native::Variable {
                id: id(1),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut report = SolveReport::new(
            Backend::Ipopt,
            &contract,
            NativeTermination {
                code: -1,
                name: "fixture".into(),
                message: None,
                category,
                assurance: Assurance::None,
            },
            &Execution::new(std::sync::Arc::default(), &Controls::default()),
        );
        report.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: vec![2.0],
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
            commitment: None,
        });
        report.quality = Some(native::quality::Quality::new(vec![], vec![], vec![]).unwrap());
        native::quality::qualify(
            &mut report,
            &ResolvedAccuracy::from_policy(&Default::default(), 1e-8).unwrap(),
        );
        report
    }
    fn bound(report: &mut SolveReport, objective: f64, value: f64, absolute: f64, relative: f64) {
        report.observation = Some(
            pse_backend_native::quality::Observation::from_values(Some(objective), vec![], vec![])
                .unwrap(),
        );
        report.evidence.original_bound = Some(pse_backend_native::solve::OriginalObjectiveBound {
            origin: CandidateBoundOrigin::MixedInteger,
            sense: pse_math::binding::ObjectiveSense::Minimize,
            value,
            absolute_tolerance: absolute,
            relative_tolerance: relative,
        });
    }
    #[test]
    fn incumbent_policy_stop_matrix_never_upgrades_failure() {
        use NativeTermination as T;
        for stop in T::ALL {
            for incumbent in IncumbentPolicy::ALL {
                let policy = pse_model::numerics::NumericalPolicy {
                    incumbent,
                    ..Default::default()
                };
                let mut native = report(stop);
                bound(&mut native, 10., 9.99, 0.02, 0.);
                let completed = complete(
                    native_use(&native, &policy),
                    CompletionEvidence::point(&[], true, 0),
                    &policy,
                );
                let supported_limit = matches!(
                    stop,
                    T::TimeLimit | T::NodeLimit | T::IterationLimit | T::SolutionLimit
                );
                let normal = matches!(stop, T::Success | T::Acceptable | T::FeasibleOnly);
                assert_eq!(
                    completed.permits_use(),
                    normal || (supported_limit && incumbent != IncumbentPolicy::Refuse),
                    "{stop:?} {incumbent:?}"
                );
                if supported_limit && incumbent == IncumbentPolicy::Refuse {
                    assert!(completed.decision.permits_seed());
                }
                assert_eq!(native.termination.category, stop);
                assert!(native.quality.as_ref().unwrap().feasible());
            }
        }
    }
    #[test]
    fn incumbent_gap_requires_valid_original_bound_and_existing_or_criterion() {
        let policy = pse_model::numerics::NumericalPolicy {
            incumbent: IncumbentPolicy::AcceptWithinGap,
            ..Default::default()
        };
        let mut native = report(NativeTermination::TimeLimit);
        assert!(
            native_use(&native, &policy)
                .refusals
                .contains(&CandidateRefusal::BoundUnavailable)
        );
        bound(&mut native, 10., 9., 0.01, 0.01);
        assert!(
            native_use(&native, &policy)
                .refusals
                .contains(&CandidateRefusal::GapExceeded)
        );
        bound(&mut native, 10., 9., 1.1, 0.);
        assert!(native_use(&native, &policy).permits_use());
        bound(&mut native, 1000., 998., 0.1, 0.003);
        assert!(native_use(&native, &policy).permits_use());
        for value in [f64::NAN, f64::INFINITY, 1001.] {
            native.evidence.original_bound.as_mut().unwrap().value = value;
            assert!(!native_use(&native, &policy).permits_use());
        }
        bound(&mut native, 1e308, -1e308, 1., 1.);
        assert!(
            native_use(&native, &policy)
                .refusals
                .contains(&CandidateRefusal::BoundUnavailable)
        );
        bound(&mut native, 1., -1., 0.1, 10.);
        assert!(!native_use(&native, &policy).permits_use());
    }
    #[test]
    fn global_bound_and_original_candidate_sources_remain_independent() {
        use pse_backend_native::solve::{BoundSource, GlobalEvidence, PrimalSource};
        let policy = pse_model::numerics::NumericalPolicy {
            incumbent: IncumbentPolicy::AcceptWithinGap,
            ..Default::default()
        };
        let mut native = report(NativeTermination::TimeLimit);
        bound(&mut native, 10., 9.99, 0.02, 0.);
        native.evidence.global = Some(GlobalEvidence {
            fidelity: pse_math::factorable::Fidelity::Exact,
            domain: pse_ids::ContentHash::from_bytes([2; 32]),
            sense: pse_math::binding::ObjectiveSense::Minimize,
            feasibility: 1e-8,
            gap_relative: 0.,
            gap_absolute: 0.02,
            dual_bound: Some(9.99),
            primal_bound: Some(10.),
            gap: Some(0.001),
            nodes: 1,
            readback: true,
            dual: BoundSource::ExactExport,
            primal: PrimalSource::FixedAssignment,
            infeasible: false,
            exact: false,
        });
        assert!(native_use(&native, &policy).permits_use());
        native.evidence.global.as_mut().unwrap().readback = false;
        assert!(
            native_use(&native, &policy)
                .refusals
                .contains(&CandidateRefusal::BoundUnavailable)
        );
        native.evidence.global.as_mut().unwrap().readback = true;
        native.evidence.global.as_mut().unwrap().primal = PrimalSource::RelaxedIncumbent;
        for incumbent in IncumbentPolicy::ALL {
            let policy = pse_model::numerics::NumericalPolicy {
                incumbent,
                ..policy.clone()
            };
            let decision = native_use(&native, &policy);
            assert_eq!(decision.usability, CandidateUse::DiagnosticOnly);
            assert!(!decision.permits_seed());
            assert!(
                decision
                    .refusals
                    .contains(&CandidateRefusal::RelaxedIncumbent)
            );
        }
        native.evidence.global.as_mut().unwrap().primal = PrimalSource::Backend;
        native.evidence.global.as_mut().unwrap().sense =
            pse_math::binding::ObjectiveSense::Maximize;
        native.evidence.global.as_mut().unwrap().dual_bound = Some(10.01);
        assert!(native_use(&native, &policy).permits_use());
        native.evidence.global.as_mut().unwrap().dual_bound = Some(9.99);
        assert!(!native_use(&native, &policy).permits_use());
    }
    #[test]
    fn composed_acceptance_retains_incumbent_and_closure_qualifiers_and_missing_evidence() {
        let policy = pse_model::numerics::NumericalPolicy {
            incumbent: IncumbentPolicy::AcceptFeasible,
            closure: ClosurePolicy::AllowUnclosed,
            ..Default::default()
        };
        let mut closure = check(ModelingCheckKind::Closure, false);
        closure.value = 4.5;
        closure.tolerance = Some(0.1);
        let checks = vec![closure];
        let native = report(NativeTermination::NodeLimit);
        let completed = complete(
            native_use(&native, &policy),
            CompletionEvidence::point(&checks, true, 1),
            &policy,
        );
        assert_eq!(completed.closure, ClosureAssessment::Unclosed);
        assert_eq!(
            completed.decision.usability,
            CandidateUse::QualifiedUnclosed
        );
        assert_eq!(
            completed.decision.qualifiers,
            [
                CandidateQualifier::AcceptedIncumbentFeasible,
                CandidateQualifier::ClosureAllowed
            ]
        );
        assert_eq!(checks[0].value, 4.5);
        for (observations, required, done) in [
            (&checks[..], 2, true),
            (&[][..], 1, true),
            (&checks[..], 1, false),
        ] {
            let completed = complete(
                native_use(&native, &policy),
                CompletionEvidence::point(observations, done, required),
                &policy,
            );
            assert!(!completed.permits_use());
            assert_eq!(completed.closure, ClosureAssessment::Unavailable);
            assert!(
                completed
                    .decision
                    .refusals
                    .contains(&CandidateRefusal::ClosureUnavailable)
            );
        }
        let completed = complete(
            native_use(&native, &policy),
            CompletionEvidence::point(&[], true, 0),
            &policy,
        );
        assert_eq!(completed.closure, ClosureAssessment::NotRequired);
    }
    #[test]
    fn original_checks_source_quality_and_qualification_cannot_be_waived() {
        let policy = pse_model::numerics::NumericalPolicy {
            incumbent: IncumbentPolicy::AcceptFeasible,
            closure: ClosurePolicy::AllowUnclosed,
            ..Default::default()
        };
        let native = report(NativeTermination::SolutionLimit);
        let failed = [check(ModelingCheckKind::OriginalEquation, false)];
        assert!(
            !complete(
                native_use(&native, &policy),
                CompletionEvidence::point(&failed, true, 0),
                &policy
            )
            .permits_use()
        );
        let mut native = native;
        native.qualification = pse_backend_native::solve::Qualification::Unqualified;
        assert!(!native_use(&native, &policy).permits_use());
        native.qualification = pse_backend_native::solve::Qualification::Feasible;
        native.record_validation_failure(pse_backend_native::ProblemError::numerical(
            "original observation",
        ));
        assert!(!native_use(&native, &policy).permits_seed());
        native.candidate = None;
        assert!(
            native_use(&native, &policy)
                .refusals
                .contains(&CandidateRefusal::NoCandidate)
        );
    }
    #[test]
    fn applicability_and_endpoint_coverage_remain_independent() {
        use pse_model::generated::enums::ModelingApplicabilityOutcome as A;
        let policy = pse_model::numerics::NumericalPolicy::default();
        let mut unknown = check(ModelingCheckKind::Applicability, true);
        unknown.applicability_outcome = Some(A::UnknownEvidence);
        unknown.applicability_required = Some(true);
        unknown.unknown_allowed = Some(true);
        let mut outside = unknown.clone();
        outside.applicability_outcome = Some(A::OutsideRegion);
        outside.extrapolation_allowed = Some(true);
        let checks = [unknown, outside];
        let completed = complete(
            native_use(&report(NativeTermination::Success), &policy),
            CompletionEvidence::point(&checks, true, 0),
            &policy,
        );
        assert!(completed.permits_use());
        assert!(
            completed
                .decision
                .qualifiers
                .contains(&CandidateQualifier::ApplicabilityUnknownAllowed)
        );
        assert!(
            completed
                .decision
                .qualifiers
                .contains(&CandidateQualifier::ApplicabilityExtrapolationAllowed)
        );
        for denied in [
            CandidateRefusal::EndpointUnavailable,
            CandidateRefusal::CoverageUnavailable,
        ] {
            let mut evidence = CompletionEvidence::point(&checks, true, 0);
            evidence.endpoint_satisfied = Some(denied != CandidateRefusal::EndpointUnavailable);
            evidence.coverage_complete = denied != CandidateRefusal::CoverageUnavailable;
            let completed = complete(
                native_use(&report(NativeTermination::Success), &policy),
                evidence,
                &policy,
            );
            assert!(!completed.permits_use());
            assert!(completed.decision.refusals.contains(&denied));
        }
        let mut absent_permission = checks[0].clone();
        absent_permission.unknown_allowed = None;
        assert!(
            !complete(
                native_use(&report(NativeTermination::Success), &policy),
                CompletionEvidence::point(&[absent_permission], true, 0),
                &policy
            )
            .permits_use()
        );
        let mut missing = checks[0].clone();
        missing.applicability_outcome = None;
        assert!(
            !complete(
                native_use(&report(NativeTermination::Success), &policy),
                CompletionEvidence::point(&[missing], true, 0),
                &policy
            )
            .permits_use()
        );
        let mut denied = checks[0].clone();
        denied.satisfied = false;
        assert!(
            !complete(
                native_use(&report(NativeTermination::Success), &policy),
                CompletionEvidence::point(&[denied], true, 0),
                &policy
            )
            .permits_use()
        );
    }
}
