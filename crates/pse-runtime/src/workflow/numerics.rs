// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Numerical preparation and immutable completion assessment, shared by public workflows.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_model::{
    generated::enums::{CandidateUse, ClosureAssessment, ClosurePolicy, NativeTermination},
    numerics::ResolvedNumericalPolicy,
};
use pse_relations::generated::runtime::{
    candidate_assessments as assessments, resolved_numerics as resolved,
};
use std::collections::BTreeMap;

fn assessment(
    native_ok: bool,
    numerical: Option<bool>,
    required: bool,
    checks: &[Option<bool>],
    policy: ClosurePolicy,
) -> (ClosureAssessment, CandidateUse, &'static str) {
    let closure = if checks.is_empty() {
        ClosureAssessment::NotRequired
    } else if checks.contains(&None) {
        ClosureAssessment::Unavailable
    } else if checks.contains(&Some(false)) {
        ClosureAssessment::Unclosed
    } else {
        ClosureAssessment::Closed
    };
    let (use_, reason) = if !native_ok {
        (
            CandidateUse::Unusable,
            "native outcome does not permit candidate use",
        )
    } else if numerical == Some(false) || required && numerical.is_none() {
        (
            CandidateUse::Unusable,
            "original numerical acceptance failed or is unavailable",
        )
    } else {
        match closure {
            ClosureAssessment::Unavailable => (
                CandidateUse::Unusable,
                "required physical closure is unavailable",
            ),
            ClosureAssessment::Unclosed if policy == ClosurePolicy::AllowUnclosed => (
                CandidateUse::QualifiedUnclosed,
                "explicit policy permits a retained physically unclosed candidate",
            ),
            ClosureAssessment::Unclosed => (
                CandidateUse::Unusable,
                "physical closure failed the frozen budget",
            ),
            _ => (
                CandidateUse::Usable,
                "required original-coordinate checks passed",
            ),
        }
    };
    (closure, use_, reason)
}
fn model_checks(checks: &[super::ModelingCheck], complete: bool) -> (Vec<Option<bool>>, bool) {
    use pse_model::generated::enums::ModelingCheckKind;
    let mut closure = Vec::new();
    let mut mandatory = complete;
    for check in checks {
        if check.kind == ModelingCheckKind::Closure { closure.push(Some(check.satisfied)); }
        else { mandatory &= check.satisfied; }
    }
    if !complete { closure.push(None); }
    (closure, mandatory)
}
fn permits(t: NativeTermination) -> bool {
    matches!(
        t,
        NativeTermination::Success
            | NativeTermination::Acceptable
            | NativeTermination::FeasibleOnly
    )
}
impl RunResult {
    pub(super) fn assess_candidates(&self) -> Vec<assessments::Row> {
        let count = match &self.request {
            RunRequest::Modeling(s) => s.len(),
            _ => 1,
        };
        (0..count)
            .map(|step| {
                let (native, native_ok, numerical, required, policy) =
                    match (&self.request, &self.report) {
                        (RunRequest::Fit(p), Ok(RunReport::Fit(r))) => {
                            let native = r.solve.as_ref().map(|s| s.termination.category);
                            (
                                native,
                                r.candidate.is_some()
                                    && native.is_none_or(permits)
                                    && r.solve
                                        .as_ref()
                                        .is_none_or(|s| s.validation_error.is_none()),
                                r.quality.as_ref().map(|q| q.feasible()),
                                true,
                                p.problem.numerics.policy.closure,
                            )
                        }
                        (RunRequest::Modeling(p), Ok(RunReport::Modeling(results))) if results.get(step).is_some() => {
                            let r=&results[step];
                            let (native,ok,num)=match &r.outcome {
                                crate::math::solves::Outcome::Native(n)=>(Some(n.termination.category),permits(n.termination.category)&&n.candidate.is_some()&&n.validation_error.is_none(),n.quality.as_ref().map(|q|q.feasible())),
                                crate::math::solves::Outcome::Constant(c)=>(None,true,Some(c.quality.feasible())),
                                _=>(None,false,None),
                            };
                            (native, ok, num, true, p[step].solve.numerics().policy.closure)
                        }
                        (RunRequest::Simulation(p), Ok(RunReport::Simulation(r))) => {
                            (None, r.accepted, Some(r.accepted), true, p.numerics().policy.closure)
                        }
                        _ => (None, false, None, true, ClosurePolicy::RequireClosed),
                    };
                let (checks, model_ok) = match &self.report {
                    Ok(RunReport::Modeling(results)) => results.get(step).map(|r| model_checks(&r.checks, r.validation_error.is_none())).unwrap_or_else(|| (vec![None], false)),
                    Ok(RunReport::Fit(r)) => model_checks(&r.checks, r.checks_complete && r.validation_error.is_none()),
                    Ok(RunReport::Simulation(r)) => model_checks(&r.checks, r.checks_complete && r.validation_error.is_none()),
                    Err(_) => (vec![None], false),
                };
                let (closure, usability, mut reason) =
                    assessment(native_ok && model_ok, numerical, required, &checks, policy);
                if !model_ok { reason = "required original-model checks failed or are unavailable"; }
                assessments::Row {
                    run_id: self.run_id,
                    step: step as i64,
                    native_termination: native,
                    numerically_feasible: numerical,
                    closure,
                    policy,
                    usability,
                    reason: reason.into(),
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
            RunRequest::Modeling(p) => p.iter().map(|p|p.solve.numerics()).collect(),
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
                                field: p.field.into(),
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
    #[test]
    fn candidate_assessment_distinguishes_numerical_closure_and_usability() {
        let check = |native, numerical, checks: &[Option<bool>], policy| {
            assessment(native, numerical, true, checks, policy)
        };
        assert_eq!(
            check(
                true,
                Some(true),
                &[Some(false)],
                ClosurePolicy::RequireClosed
            )
            .1,
            CandidateUse::Unusable
        );
        assert_eq!(
            check(
                true,
                Some(true),
                &[Some(false)],
                ClosurePolicy::AllowUnclosed
            )
            .1,
            CandidateUse::QualifiedUnclosed
        );
        assert_eq!(
            check(true, Some(true), &[None], ClosurePolicy::AllowUnclosed).1,
            CandidateUse::Unusable
        );
        assert_eq!(
            check(
                true,
                Some(false),
                &[Some(true)],
                ClosurePolicy::AllowUnclosed
            )
            .1,
            CandidateUse::Unusable
        );
        assert_eq!(
            check(false, Some(true), &[], ClosurePolicy::AllowUnclosed).1,
            CandidateUse::Unusable
        );
        assert_eq!(
            check(true, Some(true), &[], ClosurePolicy::RequireClosed).1,
            CandidateUse::Usable
        );
    }
}
