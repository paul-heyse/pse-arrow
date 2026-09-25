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
    candidate_assessments as assessments, physical_checks as checks, resolved_numerics as resolved,
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
            RunRequest::Solves(s) => s.len(),
            _ => 1,
        };
        (0..count)
            .map(|step| {
                let (native, native_ok, numerical, required, policy) =
                    match (&self.request, &self.report) {
                        (RunRequest::Solves(steps), Ok(RunReport::Solves(r))) => {
                            let (native, ok, num) = match r.outcomes.get(step) {
                                Some(crate::math::solves::Outcome::Native(r)) => (
                                    Some(r.termination.category),
                                    permits(r.termination.category)
                                        && r.candidate.is_some()
                                        && r.validation_error.is_none(),
                                    r.quality.as_ref().map(|q| q.feasible()),
                                ),
                                Some(crate::math::solves::Outcome::Constant(r)) => {
                                    (None, true, Some(r.quality.feasible()))
                                }
                                _ => (None, false, None),
                            };
                            (
                                native,
                                ok,
                                num,
                                true,
                                steps[step].solve.numerics().policy.closure,
                            )
                        }
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
                        (RunRequest::Simulation(p), Ok(RunReport::Simulation(r))) => {
                            let ok = matches!(
                                r.termination,
                                pse_backend_native::dynamics::Termination::Completed
                                    | pse_backend_native::dynamics::Termination::Event
                            ) && r.error.is_none()
                                && !r.samples.is_empty();
                            (None, ok, None, false, p.numerics.policy.closure)
                        }
                        _ => (None, false, None, true, ClosurePolicy::RequireClosed),
                    };
                let checks = match &self.physical {
                    Ok(rows) => rows
                        .iter()
                        .filter(|r| {
                            !matches!(self.request, RunRequest::Solves(_)) || r.step == step as i64
                        })
                        .map(|r| r.accepted)
                        .collect::<Vec<_>>(),
                    Err(_) => vec![None],
                };
                let (closure, usability, reason) =
                    assessment(native_ok, numerical, required, &checks, policy);
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
        let rows = self
            .physical
            .as_ref()
            .map_err(|e| contract(format!("physical assessment unavailable: {e}")))?;
        let mut physical =
            checks::Builder::with_registry(registry, rows.len()).map_err(relation)?;
        for row in rows {
            physical.push(row.clone()).map_err(relation)?;
        }
        batches.insert(checks::RELATION_ID, physical.finish().map_err(relation)?);
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
            RunRequest::Solves(s) => s.iter().map(|s| s.solve.numerics()).collect(),
            RunRequest::Fit(f) => vec![&f.problem.numerics],
            RunRequest::Simulation(p) => vec![&p.numerics],
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

impl super::ModelRevision {
    pub(super) fn property_numerics(
        &self,
        case: SemanticId,
        targets: &[pse_math::numerics::TargetSpec],
    ) -> Result<Vec<pse_math::numerics::SourcedRequirement>, WorkflowError> {
        let mut out = Vec::new();
        for binding in self
            .0
            .resolved_sources
            .scaling_bindings
            .iter()
            .filter(|b| b.case_id.is_none_or(|id| id == case))
        {
            let provider = self
                .0
                .providers
                .get(&binding.provider)
                .ok_or_else(|| contract("scaling default names an unbound native provider"))?;
            let index = usize::try_from(binding.output)
                .map_err(|_| contract("negative provider scaling output"))?;
            if index != provider.output {
                return Err(contract(
                    "scaling default does not name the selected provider output",
                ));
            }
            let spec = provider.registration.spec();
            let output = spec
                .outputs
                .get(index)
                .ok_or_else(|| contract("provider scaling output absent"))?;
            let target = targets
                .iter()
                .find(|t| t.id == binding.target_id && t.kind == binding.target_kind)
                .ok_or_else(|| contract("property scaling target not selected"))?;
            let default = self
                .0
                .resolved_sources
                .scaling_defaults
                .iter()
                .find(|d| {
                    d.property_package_id == binding.property_package_id
                        && d.property_kind_id == binding.property_kind_id
                        && d.index == binding.index
                })
                .ok_or_else(|| contract("explicitly bound property scaling default absent"))?;
            out.push(property_requirement(
                binding,
                output,
                target,
                default.scaling_factor,
                &self.0.physical.quantities,
            )?);
        }
        Ok(out)
    }
}

fn property_requirement(
    binding: &pse_model::generated::authored::provider_scaling_bindings::Row,
    output: &pse_kernels::Port,
    target: &pse_math::numerics::TargetSpec,
    factor: f64,
    registry: &pse_quantity::QuantityRegistry,
) -> Result<pse_math::numerics::SourcedRequirement, WorkflowError> {
    use pse_model::{
        generated::enums::{NumericalCoordinates, NumericalSource},
        numerics::NumericalRequirement,
    };
    pse_quantity::admission::require_same_contract(output.quantity, target.quantity, registry)
        .map_err(super::math)?;
    if binding.provenance.trim().is_empty() {
        return Err(contract("property scaling provenance required"));
    }
    Ok(pse_math::numerics::SourcedRequirement {
        source: NumericalSource::PropertyDefault,
        declaration: NumericalRequirement {
            requirement_id: binding.binding_id,
            model_id: binding.model_id,
            case_id: binding.case_id,
            target_id: binding.target_id,
            target_kind: binding.target_kind,
            nominal: None,
            scaling_factor: Some(factor),
            absolute_tolerance: None,
            relative_tolerance: None,
            unit_id: Some(output.unit.as_id()),
            coordinates: NumericalCoordinates::Physical,
            priority: 0,
            required: false,
            provenance: binding.provenance.clone(),
        },
    })
}
#[cfg(test)]
mod property_tests {
    use super::*;
    #[test]
    fn property_default_binding_preserves_magnitude_units_and_explicit_origin() {
        use pse_model::generated::enums::NumericalTarget;
        use pse_quantity::standard::{ids, standard_registry};
        let registry = standard_registry().unwrap();
        let id = SemanticId::from_bytes([1; 16]);
        let quantity = ids::quantity("temperature.point");
        let target = pse_math::numerics::TargetSpec {
            id,
            kind: NumericalTarget::Variable,
            quantity,
            unit: ids::unit("degC"),
            integer: false,
            declared_tolerance: None,
        };
        let output = pse_kernels::Port {
            id,
            quantity,
            unit: ids::unit("K"),
        };
        let mut binding:pse_model::generated::authored::provider_scaling_bindings::Row=serde_json::from_value(serde_json::json!({"binding_id":id,"model_id":id,"case_id":null,"provider":"selected","output":0,"target_id":id,"target_kind":"variable","property_package_id":id,"property_kind_id":id,"index":[],"provenance":"declared package property output"})).unwrap();
        let requirement =
            property_requirement(&binding, &output, &target, 0.01, &registry).unwrap();
        let result = pse_math::numerics::resolve(
            &registry,
            std::slice::from_ref(&target),
            &[requirement],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(result.targets[0].nominal, 100.0);
        assert!(result.targets[0].provenance.iter().any(|p| p.selected
            && p.declaration == Some(id)
            && p.description == binding.provenance));
        binding.provenance.clear();
        assert!(property_requirement(&binding, &output, &target, 0.01, &registry).is_err());
        let other = pse_kernels::Port {
            quantity: ids::quantity("neutral"),
            ..output
        };
        assert!(property_requirement(&binding, &other, &target, 0.01, &registry).is_err());
    }
}
