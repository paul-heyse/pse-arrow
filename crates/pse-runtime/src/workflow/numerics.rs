// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Numerical preparation and immutable completion assessment, shared by public workflows.
use super::{RunReport, RunRequest, RunResult, WorkflowError, relation};
use pse_backend_native::solve::SolveReport;
use pse_ids::SemanticId;
use pse_model::{
    generated::enums::{CandidateUse, ClosureAssessment, ClosurePolicy, NativeTermination},
    numerics::ResolvedNumericalPolicy,
};
use pse_relations::generated::runtime::{
    candidate_assessments as assessments, resolved_numerics as resolved,
};
use std::collections::BTreeMap;

/// Stable reason for a candidate-use decision; published as text, never parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CandidateReason {
    /// Every required original-coordinate check passed and the native stop permits use.
    Accepted,
    /// The native API supplied no candidate.
    NoCandidate,
    /// Independent original-model validation failed.
    ValidationFailed,
    /// Original-coordinate feasibility failed or is unavailable.
    Infeasible,
    /// Qualification was not established for the reported facts.
    Unqualified,
    /// A limit, numerical or inconclusive stop left an original-feasible iterate.
    StoppedFeasible,
    /// A refusal, failure, cancellation or unboundedness stop.
    NativeOutcome,
    /// The point a local infeasibility stop returns.
    LeastInfeasible,
    /// An incumbent of a relaxed global export: an assignment proposal (ADR-0105 §2).
    RelaxedIncumbent,
    /// Required original-model checks failed or are unavailable.
    ModelChecks,
    /// Required physical closure is unavailable.
    ClosureUnavailable,
    /// Physical closure failed the frozen budget.
    ClosureUnclosed,
    /// Closure failed and an explicit policy permits the unclosed candidate.
    ClosureAllowed,
}
impl CandidateReason {
    /// Stable published reason.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "required original-coordinate checks passed",
            Self::NoCandidate => "native attempt supplied no candidate",
            Self::ValidationFailed => "original-model validation failed",
            Self::Infeasible => "original numerical acceptance failed or is unavailable",
            Self::Unqualified => "original-space qualification was not established",
            Self::StoppedFeasible => {
                "feasible iterate after a native stop that forbids use; seed only"
            }
            Self::NativeOutcome => "native outcome does not permit candidate use",
            Self::LeastInfeasible => {
                "least-infeasible point after a local infeasibility stop; diagnostic only"
            }
            Self::RelaxedIncumbent => {
                "incumbent of a relaxed global export; an assignment proposal, diagnostic only"
            }
            Self::ModelChecks => "required original-model checks failed or are unavailable",
            Self::ClosureUnavailable => "required physical closure is unavailable",
            Self::ClosureUnclosed => "physical closure failed the frozen budget",
            Self::ClosureAllowed => {
                "explicit policy permits a retained physically unclosed candidate"
            }
        }
    }
}
/// One immutable candidate-use decision (§16.6, ADR-0106). Solve sequences,
/// initialization, homotopy, studies, fitting, diagnostics and publication consume it;
/// no workflow re-derives acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CandidateDecision {
    pub(crate) usability: CandidateUse,
    pub(crate) reason: CandidateReason,
}
impl CandidateDecision {
    const fn new(usability: CandidateUse, reason: CandidateReason) -> Self {
        Self { usability, reason }
    }
    /// The candidate may be a result: it may be committed, published or advanced from.
    pub(crate) const fn permits_use(self) -> bool {
        matches!(
            self.usability,
            CandidateUse::Usable | CandidateUse::QualifiedUnclosed
        )
    }
    /// The candidate may seed a later step; a seed-only candidate is never a result.
    pub(crate) const fn permits_seed(self) -> bool {
        self.permits_use() || matches!(self.usability, CandidateUse::SeedOnly)
    }
}
/// What a native stop permits for an original-feasible candidate. The match is
/// exhaustive: a new shared stop category must be assigned here.
const fn stop_use(category: NativeTermination) -> Option<CandidateDecision> {
    use NativeTermination as T;
    match category {
        // Success, a declared acceptable budget, or the feasible point of a square system.
        T::Success | T::Acceptable | T::FeasibleOnly => None,
        // The method stopped before establishing its result; the iterate may seed.
        T::Limit
        | T::IterationLimit
        | T::TimeLimit
        | T::SolutionLimit
        | T::ObjectiveLimit
        | T::ResourceExhausted
        | T::Inconclusive
        | T::Numerical => Some(CandidateDecision::new(
            CandidateUse::SeedOnly,
            CandidateReason::StoppedFeasible,
        )),
        T::Infeasible => Some(CandidateDecision::new(
            CandidateUse::DiagnosticOnly,
            CandidateReason::LeastInfeasible,
        )),
        T::Unbounded
        | T::InfeasibleOrUnbounded
        | T::Cancelled
        | T::Evaluation
        | T::Panic
        | T::Invalid => Some(CandidateDecision::new(
            CandidateUse::Unusable,
            CandidateReason::NativeOutcome,
        )),
    }
}
/// The native decision over termination, qualification, quality and validation.
pub(crate) fn native_use(report: &SolveReport) -> CandidateDecision {
    use pse_backend_native::solve::Qualification;
    let refused = |reason| CandidateDecision::new(CandidateUse::Unusable, reason);
    if report.validation_failure().is_some() {
        return refused(CandidateReason::ValidationFailed);
    }
    if report.candidate.is_none() {
        return refused(CandidateReason::NoCandidate);
    }
    if report.termination.category == NativeTermination::Infeasible {
        return CandidateDecision::new(
            CandidateUse::DiagnosticOnly,
            CandidateReason::LeastInfeasible,
        );
    }
    if report
        .evidence
        .global
        .is_some_and(|g| g.primal == pse_backend_native::solve::PrimalSource::RelaxedIncumbent)
    {
        return CandidateDecision::new(
            CandidateUse::DiagnosticOnly,
            CandidateReason::RelaxedIncumbent,
        );
    }
    if !report
        .quality
        .as_ref()
        .is_some_and(pse_backend_native::quality::Quality::feasible)
    {
        return refused(CandidateReason::Infeasible);
    }
    if report.qualification == Qualification::Unqualified {
        return refused(CandidateReason::Unqualified);
    }
    stop_use(report.termination.category).unwrap_or(CandidateDecision::new(
        CandidateUse::Usable,
        CandidateReason::Accepted,
    ))
}
/// All-fixed evaluation has no native stop; original quality alone decides.
pub(crate) fn constant_use(quality: &pse_backend_native::quality::Quality) -> CandidateDecision {
    if quality.feasible() {
        CandidateDecision::new(CandidateUse::Usable, CandidateReason::Accepted)
    } else {
        CandidateDecision::new(CandidateUse::Unusable, CandidateReason::Infeasible)
    }
}
/// A trajectory is usable only after a completed integration without a typed failure.
pub(crate) fn trajectory_use(report: &pse_backend_native::dynamics::Report) -> CandidateDecision {
    if report.termination == pse_backend_native::dynamics::Termination::Completed
        && report.error.is_none()
    {
        CandidateDecision::new(CandidateUse::Usable, CandidateReason::Accepted)
    } else {
        CandidateDecision::new(CandidateUse::Unusable, CandidateReason::NativeOutcome)
    }
}
/// A refusal with no native report.
pub(crate) const fn refused(reason: CandidateReason) -> CandidateDecision {
    CandidateDecision::new(CandidateUse::Unusable, reason)
}
/// The completed decision for one candidate: the native decision, required
/// original-model checks and physical closure (§16.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Completed {
    pub(crate) closure: ClosureAssessment,
    pub(crate) decision: CandidateDecision,
}
impl Completed {
    /// See [`CandidateDecision::permits_use`].
    pub(crate) const fn permits_use(self) -> bool {
        self.decision.permits_use()
    }
}
/// The workflow completion owner (§16.6). Checks and closure can only refuse or qualify
/// a usable native decision; they never upgrade a seed-only or diagnostic point.
pub(crate) fn complete(
    native: CandidateDecision,
    checks: &[super::ModelingCheck],
    checks_complete: bool,
    policy: ClosurePolicy,
) -> Completed {
    use pse_model::generated::enums::ModelingCheckKind;
    let mut closure_checks = Vec::new();
    let mut mandatory = checks_complete;
    for check in checks {
        if check.kind == ModelingCheckKind::Closure {
            closure_checks.push(Some(check.satisfied));
        } else {
            mandatory &= check.satisfied;
        }
    }
    if !checks_complete {
        closure_checks.push(None);
    }
    let closure = if closure_checks.is_empty() {
        ClosureAssessment::NotRequired
    } else if closure_checks.contains(&None) {
        ClosureAssessment::Unavailable
    } else if closure_checks.contains(&Some(false)) {
        ClosureAssessment::Unclosed
    } else {
        ClosureAssessment::Closed
    };
    let decision = if native.usability != CandidateUse::Usable {
        native
    } else if !mandatory {
        refused(CandidateReason::ModelChecks)
    } else {
        match closure {
            ClosureAssessment::Unavailable => refused(CandidateReason::ClosureUnavailable),
            ClosureAssessment::Unclosed if policy == ClosurePolicy::AllowUnclosed => {
                CandidateDecision::new(
                    CandidateUse::QualifiedUnclosed,
                    CandidateReason::ClosureAllowed,
                )
            }
            ClosureAssessment::Unclosed => refused(CandidateReason::ClosureUnclosed),
            ClosureAssessment::NotRequired | ClosureAssessment::Closed => native,
        }
    };
    Completed { closure, decision }
}
impl RunResult {
    pub(super) fn assess_candidates(&self) -> Vec<assessments::Row> {
        let count = match &self.request {
            RunRequest::Modeling(s) => s.len(),
            _ => 1,
        };
        let unavailable = || Completed {
            closure: ClosureAssessment::Unavailable,
            decision: refused(CandidateReason::ModelChecks),
        };
        (0..count)
            .map(|step| {
                let (native, numerical, policy, completed) = match (&self.request, &self.report) {
                    (RunRequest::Fit(p), Ok(RunReport::Fit(r))) => {
                        let policy = p.problem.numerics.policy.closure;
                        (
                            r.solve.as_ref().map(|s| s.termination.category),
                            r.quality.as_ref().map(|q| q.feasible()),
                            policy,
                            complete(
                                r.candidate_use(),
                                &r.checks,
                                r.checks_complete && r.validation_error.is_none(),
                                policy,
                            ),
                        )
                    }
                    (RunRequest::Modeling(p), Ok(RunReport::Modeling(results)))
                        if results.get(step).is_some() =>
                    {
                        let r = &results[step];
                        let (native, numerical) = match &r.outcome {
                            crate::math::solves::Outcome::Native(n) => (
                                Some(n.termination.category),
                                n.quality.as_ref().map(|q| q.feasible()),
                            ),
                            crate::math::solves::Outcome::Constant(c) => {
                                (None, Some(c.quality.feasible()))
                            }
                            crate::math::solves::Outcome::Rejected(_) => (None, None),
                        };
                        (
                            native,
                            numerical,
                            p[step].solve.numerics().policy.closure,
                            r.completion,
                        )
                    }
                    (RunRequest::Simulation(p), Ok(RunReport::Simulation(r))) => {
                        let policy = p.numerics().policy.closure;
                        (
                            None,
                            Some(r.accepted),
                            policy,
                            complete(
                                r.candidate_use(),
                                &r.checks,
                                r.checks_complete && r.validation_error.is_none(),
                                policy,
                            ),
                        )
                    }
                    _ => (None, None, ClosurePolicy::RequireClosed, unavailable()),
                };
                assessments::Row {
                    run_id: self.run_id,
                    step: step as i64,
                    native_termination: native,
                    numerically_feasible: numerical,
                    closure: completed.closure,
                    policy,
                    usability: completed.decision.usability,
                    reason: completed.decision.reason.as_str().into(),
                }
            })
            .collect()
    }
    pub(super) fn numerical_tables(
        &self,
        batches: &mut BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>,
    ) -> Result<(), WorkflowError> {
        let registry = &self.runtime.registry;
        let mut candidates = assessments::Builder::with_registry(registry, self.assessments.len())
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
            RunRequest::Modeling(p) => p.iter().map(|p| p.solve.numerics()).collect(),
        };
        let mut resolved = resolved::Builder::with_registry(registry, 0).map_err(relation)?;
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
        }
    }
    #[test]
    fn candidate_assessment_distinguishes_native_closure_and_usability() {
        let accepted = CandidateDecision::new(CandidateUse::Usable, CandidateReason::Accepted);
        let seed = CandidateDecision::new(CandidateUse::SeedOnly, CandidateReason::StoppedFeasible);
        let refusal = refused(CandidateReason::NativeOutcome);
        let closed = [check(ModelingCheckKind::Closure, true)];
        let unclosed = [check(ModelingCheckKind::Closure, false)];
        let failed = [check(ModelingCheckKind::OriginalEquation, false)];
        let run = |native, checks: &[_], checked, policy| {
            complete(native, checks, checked, policy).decision.usability
        };
        use ClosurePolicy::{AllowUnclosed as Allow, RequireClosed as Require};
        assert_eq!(
            run(accepted, &unclosed, true, Require),
            CandidateUse::Unusable
        );
        assert_eq!(
            run(accepted, &unclosed, true, Allow),
            CandidateUse::QualifiedUnclosed
        );
        assert_eq!(run(accepted, &closed, false, Allow), CandidateUse::Unusable);
        assert_eq!(run(accepted, &failed, true, Allow), CandidateUse::Unusable);
        assert_eq!(run(refusal, &closed, true, Allow), CandidateUse::Unusable);
        assert_eq!(
            complete(refusal, &closed, true, Allow).decision.reason,
            CandidateReason::NativeOutcome
        );
        // Checks and closure never upgrade a seed-only point.
        assert_eq!(run(seed, &closed, true, Allow), CandidateUse::SeedOnly);
        assert_eq!(run(seed, &failed, true, Allow), CandidateUse::SeedOnly);
        assert_eq!(run(accepted, &[], true, Require), CandidateUse::Usable);
        assert_eq!(run(accepted, &closed, true, Require), CandidateUse::Usable);
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
    #[test]
    fn native_stops_map_to_one_candidate_use() {
        use NativeTermination as T;
        for (stop, usability) in [
            (T::Success, CandidateUse::Usable),
            (T::Acceptable, CandidateUse::Usable),
            (T::FeasibleOnly, CandidateUse::Usable),
            (T::IterationLimit, CandidateUse::SeedOnly),
            (T::TimeLimit, CandidateUse::SeedOnly),
            (T::Numerical, CandidateUse::SeedOnly),
            (T::Infeasible, CandidateUse::DiagnosticOnly),
            (T::Cancelled, CandidateUse::Unusable),
            (T::Evaluation, CandidateUse::Unusable),
            (T::Unbounded, CandidateUse::Unusable),
        ] {
            assert_eq!(native_use(&report(stop)).usability, usability, "{stop:?}");
        }
        let mut r = report(T::Success);
        r.qualification = pse_backend_native::solve::Qualification::Unqualified;
        assert_eq!(native_use(&r).reason, CandidateReason::Unqualified);
        r.quality = None;
        assert_eq!(native_use(&r).reason, CandidateReason::Infeasible);
        r.record_validation_failure(pse_backend_native::ProblemError::numerical("observation"));
        assert_eq!(native_use(&r).reason, CandidateReason::ValidationFailed);
        r.candidate = None;
        assert!(!native_use(&r).permits_seed());
    }
    #[test]
    fn candidate_use_iteration_limited_feasible_is_seed_only_everywhere() {
        use crate::math::solves::Outcome;
        // An iteration-limited run whose final iterate is feasible in original coordinates.
        let mut report = report(NativeTermination::IterationLimit);
        // Feasibility is retained as qualification evidence, never as a result.
        assert_eq!(
            report.qualification,
            pse_backend_native::solve::Qualification::Feasible
        );
        // Block initialization commits through the same native decision.
        #[cfg(feature = "solver-kinsol")]
        let commit = |report: &SolveReport| {
            let id = |v: u8| SemanticId::from_bytes([v; 16]);
            let block = pse_structural::initialization::Block {
                id: pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([2; 32])),
                members: pse_structural::incidence::Part {
                    rows: vec![id(2)],
                    columns: vec![id(1)],
                },
                inputs: vec![],
            };
            let mut values = pse_math::binding::CaseValues {
                scalars: BTreeMap::from([(id(1), 1.0)]),
            };
            crate::math::initialization::commit_block(&mut values, &block, Some(report))
        };
        #[cfg(not(feature = "solver-kinsol"))]
        let commit = |report: &SolveReport| native_use(report).permits_use();
        let decide = |report: &SolveReport| {
            let outcome = Outcome::Native(Box::new(report.clone()));
            // Solve-sequence seeding and the nonlinear explanation read the native decision.
            let native = outcome.candidate_use();
            // Modeling acceptance (homotopy advance, study predecessors, initialization
            // stages) and the published assessment read the same completion.
            let completed = complete(native, &[], true, ClosurePolicy::AllowUnclosed);
            (native, completed, commit(report))
        };
        let (native, completed, committed) = decide(&report);
        assert_eq!(native.usability, CandidateUse::SeedOnly);
        assert_eq!(completed.decision.usability, CandidateUse::SeedOnly);
        assert_eq!(completed.decision.reason, CandidateReason::StoppedFeasible);
        // It may seed a later step, and it is never a result, a commit or an advance.
        assert!(native.permits_seed() && completed.decision.permits_seed());
        assert!(!native.permits_use() && !completed.permits_use());
        assert!(!committed);
        // The same facts after a permitted stop are a result in every consumer.
        report.termination.category = NativeTermination::Success;
        pse_backend_native::quality::qualify(
            &mut report,
            &pse_backend_native::solve::ResolvedAccuracy::from_policy(&Default::default(), 1e-8)
                .unwrap(),
        );
        let (native, completed, committed) = decide(&report);
        assert!(native.permits_use() && completed.permits_use() && committed);
        assert_eq!(completed.decision.usability, CandidateUse::Usable);
    }
}
