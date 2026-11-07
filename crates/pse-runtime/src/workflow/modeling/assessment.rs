// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-model assessment of a step's candidate: prepared once per structure, then
//! evaluated on the step's own worker, so every step of a staged sequence is qualified the
//! same way without a nested job (A6).
use super::results::{AssessmentScope, CertifiedBound, assess_observations, assessment_units};
use super::*;
use crate::math::{ExecutableCase, WorkerBudget, solves::Outcome};
use pse_math::binding::CaseValues;
use pse_model::diagnostic::DiagnosticProjection;
use pse_model::generated::identities::RunId;
use std::sync::Arc;

fn exposure_check(
    run_id: RunId,
    source: DeclarationId,
    target: SemanticId,
    actual: f64,
    bound: f64,
) -> ModelingCheck {
    let mut check = results::temporal_closure_check(run_id, source, target, 0, 0.0, actual, bound);
    check.time = None;
    check.kind = pse_model::generated::enums::ModelingCheckKind::Check;
    check.satisfied = actual.is_finite() && actual >= 0.0 && actual <= bound;
    check
}

struct Exposure {
    quantity: SemanticId,
    bound: f64,
    instance: Option<SemanticId>,
    dependencies: std::collections::BTreeSet<SemanticId>,
}
fn exposure_bounds(
    registry: &pse_quantity::QuantityRegistry,
    policy: &pse_model::numerics::NumericalPolicy,
) -> Result<BTreeMap<(SemanticId, SemanticId), Exposure>, WorkflowError> {
    let mut exposures: BTreeMap<(SemanticId, SemanticId), Exposure> = BTreeMap::new();
    for requirement in &policy.caloric_requirements {
        for consumer in &requirement.consumers {
            let quantity = registry
                .quantity_type(consumer.exposure_quantity_id.into())
                .map_err(|e| contract(e.to_string()))?;
            let from = registry
                .unit(consumer.exposure_unit_id.into())
                .map_err(|e| contract(e.to_string()))?;
            let to = registry
                .unit(quantity.canonical_unit)
                .map_err(|e| contract(e.to_string()))?;
            let conversion = pse_quantity::convert_spec_for_type(from, to, &quantity.key)
                .map_err(|e| contract(e.to_string()))?;
            let bound = consumer.aggregate_exposure * conversion.scale + conversion.offset;
            if !bound.is_finite() || bound < 0.0 {
                return Err(contract("energy exposure bound conversion is invalid"));
            }
            let instance = requirement.instance_id.map(Into::into);
            let entry = exposures
                .entry((consumer.declaration_id, consumer.actual_flow_id))
                .or_insert_with(|| Exposure {
                    quantity: consumer.exposure_quantity_id,
                    bound,
                    instance,
                    dependencies: Default::default(),
                });
            pse_quantity::admission::require_same_contract(
                entry.quantity.into(),
                consumer.exposure_quantity_id.into(),
                registry,
            )
            .map_err(|e| contract(e.to_string()))?;
            if entry.instance != instance {
                return Err(contract("conflicting energy exposure instance"));
            }
            entry.bound = entry.bound.min(bound);
            entry
                .dependencies
                .extend([consumer.target_id, consumer.power_rule_id]);
        }
    }
    Ok(exposures)
}

/// One assessed candidate: the complete values it implies, its original-model checks and
/// reports, and why checks could not be evaluated.
#[derive(Debug)]
pub(in crate::workflow) struct AssessedPoint {
    pub values: CaseValues,
    pub checks: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Actual failed operation that prevented a candidate from reaching original checks.
    pub(in crate::workflow) unavailable_cause: Option<Arc<pse_backend_native::ProblemError>>,
    /// A candidate existed and was assessed; without one no check applies.
    pub complete: bool,
    pub required_closure: usize,
    pub owner: Arc<pse_columnar::AllocationLease>,
    pub work: Vec<pse_model::strategy::WorkCharge>,
    pub accuracy: Vec<pse_math::engineering_accuracy::GoalResult>,
    /// Original physical/native permission, before optional goal composition.
    pub original: Option<crate::workflow::numerics::CandidateDecision>,
    pub canonical_fallback: bool,
}
impl AssessedPoint {
    /// Preserve the original scientific refusal instead of deriving it from native status.
    pub(in crate::workflow) fn conclusion(
        &self,
        completion: &crate::workflow::numerics::Completed,
    ) -> crate::math::strategy::OriginalConclusion {
        use crate::math::strategy::OriginalConclusion;
        if let Some(error) = &self.error {
            OriginalConclusion::Unavailable {
                cause: crate::math::strategy::target::assessment_failure(error),
            }
        } else if !self.complete {
            OriginalConclusion::Unavailable {
                cause: self.unavailable_cause.clone().unwrap_or_else(|| {
                    Arc::new(pse_backend_native::ProblemError::numerical(
                        "native trajectory produced no assessable original candidate",
                    ))
                }),
            }
        } else if self
            .original
            .as_ref()
            .unwrap_or(&completion.decision)
            .permits_use()
        {
            OriginalConclusion::Satisfied
        } else {
            // Recovery may replace this candidate. Keep bounded original-check
            // evidence in the retained strategy cause, rather than preserving
            // only a generic closure refusal beside the final fallback point.
            let failed = self.checks.iter().filter(|check| !check.satisfied);
            let failed_count = failed.clone().count();
            let details = failed
                .take(8)
                .map(|check| {
                    format!(
                        "kind={:?}, source={}, target={}, value={}, tolerance={:?}",
                        check.kind, check.source_id, check.target_id, check.value, check.tolerance
                    )
                })
                .collect::<Vec<_>>();
            let reason = self
                .original
                .as_ref()
                .unwrap_or(&completion.decision)
                .reason();
            let closures = self.checks.iter().filter(|check| {
                check.kind == pse_relations::generated::enums::ModelingCheckKind::Closure
            });
            let closure_count = closures.clone().count();
            let closed_count = closures.filter(|check| check.satisfied).count();
            let reason = format!(
                "{reason}; original_closure_checks={closed_count}/{closure_count}, required={}; failed_original_checks={}/{failed_count}: {}",
                self.required_closure,
                details.len(),
                details.join("; ")
            );
            OriginalConclusion::Refused {
                cause: Arc::new(pse_backend_native::ProblemError::numerical(reason)),
            }
        }
    }
    /// Compose original checks with native permission through the shared completion owner.
    pub(in crate::workflow) fn completion(
        &self,
        outcome: &Outcome,
        policy: &pse_model::numerics::NumericalPolicy,
    ) -> crate::workflow::numerics::Completed {
        crate::workflow::numerics::complete(
            outcome.candidate_use(policy),
            crate::workflow::numerics::CompletionEvidence::point(
                &self.checks,
                self.complete && self.error.is_none(),
                self.required_closure,
            )
            .with_accuracy(&self.accuracy),
            policy,
        )
        .with_canonical_fallback(self.canonical_fallback)
    }
}
/// Which original-model obligations a step answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::workflow) enum Obligations {
    /// Intermediate specifications keep their model checks; final-fixture expectations
    /// apply only to the original specification.
    Intermediate,
    /// Every obligation, including final-fixture expectations.
    Final,
}
/// The prepared original-model assessment of one bound case.
#[derive(Debug)]
pub(in crate::workflow) struct Assessment {
    math: Arc<crate::math::MathService>,
    program: Option<Arc<ExecutableCase>>,
    rows: Vec<SemanticId>,
    scope: AssessmentScope,
    pub(in crate::workflow) required_closure: usize,
}
impl ModelingPackage {
    /// Prepare the assessment of `prepared` before its step runs. The observation program
    /// is value-independent and shared by every step of the same structure.
    pub(in crate::workflow) async fn assessment(
        &self,
        services: &crate::workflow::PreparationServices,
        prepared: &ModelingSolvePreparation,
        obligations: Obligations,
        cancel: &crate::CancelSource,
    ) -> Result<Assessment, WorkflowError> {
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let product = prepared.model.model.compiled();
                let mut units = assessment_units(product);
                if obligations == Obligations::Intermediate {
                    for expectation in product.model.expectations.values() {
                        units.remove(&(expectation.id, expectation.lineage.declaration));
                    }
                }
                let scope = units.keys().copied().collect();
                let rows = units
                    .into_values()
                    .flatten()
                    .collect::<std::collections::BTreeSet<_>>();
                let program = if rows.is_empty() {
                    None
                } else {
                    Some(
                        self.observation_program(
                            services,
                            &prepared.model.model,
                            &rows,
                            prepared.compiler,
                            cancel,
                        )
                        .await?,
                    )
                };
                let rows = program
                    .as_ref()
                    .map(|p| p.assembly.structure().rows().iter().map(|r| r.id).collect())
                    .unwrap_or_default();
                Ok(Assessment {
                    math: services.math().clone(),
                    program,
                    rows,
                    scope,
                    required_closure: product.model.required_closure_checks(),
                })
            })
            .await
    }
}
impl Assessment {
    /// Assess `outcome` against the original model on the step's worker. The evaluator
    /// observes the step's stop flag and is charged to the session's worker share (F31).
    /// Without a candidate nothing is evaluated and no check applies.
    #[expect(
        clippy::too_many_arguments,
        reason = "the step supplies its identity, stop flag, worker share and result owner"
    )]
    pub(in crate::workflow) fn assess(
        &self,
        prepared: &ModelingSolvePreparation,
        run_id: RunId,
        attempt: usize,
        outcome: &Outcome,
        execution: Option<&pse_kernels::ExecutionScope>,
        budget: &Arc<WorkerBudget>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> AssessedPoint {
        let mut point = AssessedPoint {
            values: prepared.model.values.clone(),
            checks: vec![],
            reports: vec![],
            error: None,
            unavailable_cause: None,
            complete: false,
            required_closure: self.required_closure,
            owner,
            work: Vec::new(),
            accuracy: Vec::new(),
            original: None,
            canonical_fallback: prepared
                .solve
                .numerics()
                .targets
                .iter()
                .any(|t| t.engineering.as_ref().is_some_and(|e| e.canonical_fallback)),
        };
        match outcome {
            Outcome::Constant(report) => {
                point
                    .values
                    .scalars
                    .extend(report.coordinates.iter().copied());
            }
            Outcome::Native(native) => match &native.candidate {
                Some(candidate) if candidate.primal.len() == native.variables.len() => {
                    point.values.scalars.extend(
                        native
                            .variables
                            .iter()
                            .copied()
                            .zip(candidate.primal.iter().copied()),
                    );
                }
                Some(_) => {
                    let _ = prepared.solve.clear_point_accuracy();
                    point.error =
                        Some(contract("native candidate coordinate extent").boundary_diagnostic());
                    return point;
                }
                None => {
                    let _ = prepared.solve.clear_point_accuracy();
                    point.unavailable_cause = crate::math::strategy::cause(outcome);
                    return point;
                }
            },
            Outcome::Rejected(_) => {
                let _ = prepared.solve.clear_point_accuracy();
                point.unavailable_cause = crate::math::strategy::cause(outcome);
                return point;
            }
        }
        let Some(execution) = execution else {
            let _ = prepared.solve.clear_point_accuracy();
            point.error =
                Some(contract("candidate assessment has no execution scope").boundary_diagnostic());
            return point;
        };
        point.complete = true;
        let certified = CertifiedBound::of(outcome);
        let mut evaluations = 0_u64;
        match self.evaluate(
            prepared,
            run_id,
            attempt,
            &point.values,
            certified,
            execution,
            budget,
            &mut evaluations,
            &mut point.checks,
            &mut point.reports,
        ) {
            Ok(()) => {}
            Err(error) => point.error = Some(error.boundary_diagnostic()),
        }
        let policy = &prepared.solve.numerics().policy;
        let original = crate::workflow::numerics::original_completion(
            outcome.candidate_use(policy),
            crate::workflow::numerics::CompletionEvidence::point(
                &point.checks,
                point.complete && point.error.is_none(),
                point.required_closure,
            ),
            policy,
        );
        point.original = Some(original.decision.clone());
        if !policy.goals.is_empty() {
            if original.permits_use() {
                match prepared.solve.coordinate_accuracy(
                    &self.math,
                    outcome,
                    &point.values,
                    execution,
                    budget,
                ) {
                    Ok(goals) => point.accuracy = goals,
                    Err(error) => {
                        point.error = Some(error.boundary_diagnostic(
                            pse_diagnostics::DiagnosticStage::ModelingQualification,
                        ));
                        point.unavailable_cause = Some(Arc::new(error));
                    }
                }
            } else {
                if let Err(error) = prepared.solve.clear_point_accuracy() {
                    point.error = Some(error.boundary_diagnostic(
                        pse_diagnostics::DiagnosticStage::ModelingQualification,
                    ));
                    point.unavailable_cause = Some(Arc::new(error));
                }
            }
        }
        let mut charging_owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
        if let Ok(original) = prepared.solve.original_identity() {
            charging_owner.hash(&original);
        }
        charging_owner
            .str("original-model-assessment")
            .str(&run_id.to_string())
            .u64(attempt as u64);
        point.work.push(pse_model::strategy::WorkCharge {
            phase: pse_model::strategy::Phase::Assessment,
            scope: pse_model::strategy::Scope::Task,
            charging_owner: charging_owner.finish_hash(),
            observed: pse_model::strategy::WorkObservation {
                attempts: 0,
                evaluations: Some(evaluations),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
        });
        point
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the step supplies its identity, point, certified bound, stop flag and worker share"
    )]
    fn evaluate(
        &self,
        prepared: &ModelingSolvePreparation,
        run_id: RunId,
        attempt: usize,
        values: &CaseValues,
        certified: Option<CertifiedBound>,
        execution: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
        checks: &mut Vec<ModelingCheck>,
        reports: &mut Vec<ModelingReport>,
    ) -> Result<(), WorkflowError> {
        let checkpoint = || {
            execution.check().map_err(|error| {
                crate::math::MathRuntimeError::from(pse_backend_native::ProblemError::Provider(
                    error,
                ))
            })
        };
        checkpoint()?;
        let mut evaluator = self
            .program
            .as_ref()
            .map(|program| {
                self.math.worker(
                    program.clone(),
                    &prepared.providers,
                    execution.clone(),
                    budget,
                )
            })
            .transpose()?;
        let check_start = checks.len();
        let report_start = reports.len();
        let assessed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut applicability = Vec::new();
        let observed = if let Some(evaluator) = evaluator.as_mut() {
            let observed = budget.evaluate(|| {
                *evaluations =
                    evaluations
                        .checked_add(1)
                        .ok_or(crate::math::MathRuntimeError::Limit(
                            "original assessment work counter overflow",
                        ))?;
                evaluator
                    .worker()
                    .constraints(values)
                    .map_err(crate::math::MathRuntimeError::from)
            })?;
            let observed = self
                .rows
                .iter()
                .copied()
                .zip(observed.iter().copied())
                .collect();
            applicability = evaluator.worker().applicability_observations();
            observed
        } else {
            BTreeMap::new()
        };
        assess_observations(
            run_id,
            prepared.model.model.compiled(),
            values,
            &observed,
            &applicability,
            prepared.solve.numerics(),
            &prepared.source.quantities,
            true,
            Some(&self.scope),
            certified,
            checks,
            reports,
        )?;
        // The frozen exposure is a bound, not a solved flow. Check the actual
        // original candidate variable at the same acceptance owner that settles
        // all other original-model obligations.
        let exposures = exposure_bounds(
            &prepared.source.quantities,
            &prepared.solve.numerics().policy,
        )?;
        for ((declaration, flow), exposure) in exposures {
            let actual = *values.scalars.get(&flow).ok_or_else(|| {
                contract("actual energy-exposure flow is absent from the candidate")
            })?;
            let mut check =
                exposure_check(run_id, declaration.into(), flow, actual, exposure.bound);
            check.dependencies = exposure.dependencies.into_iter().collect();
            check.observation_instance = exposure.instance;
            check.input_values.push(pse_model::generated::runtime::modeling_checks::RuntimeModelingChecksFieldInputValuesItem {
                    name: "frozen_exposure_bound".into(), value: exposure.bound,
                    quantity_type: exposure.quantity,
                });
            checks.push(check);
        }
        checkpoint()?;
        Ok(())
        })).unwrap_or_else(|payload| Err(WorkflowError::from(crate::math::MathRuntimeError::Panic(
            pse_diagnostics::panic::settle_payload(payload, "original assessment panic with opaque payload"),
        ))));
        for row in &mut checks[check_start..] {
            row.step = attempt as i64;
        }
        for row in &mut reports[report_start..] {
            row.step = attempt as i64;
        }
        WorkflowError::settle(
            assessed,
            evaluator.map_or(Ok(()), |worker| {
                worker.finish().map_err(WorkflowError::from)
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_energy_exposure_is_a_required_attributed_original_check() {
        let run = RunId::from_bytes([1; 16]);
        let source = DeclarationId::from_bytes([2; 16]);
        let target = SemanticId::from_bytes([3; 16]);
        for (actual, satisfied) in [
            (0.0, true),
            (4.0, true),
            (-0.01, false),
            (4.01, false),
            (f64::NAN, false),
            (f64::INFINITY, false),
        ] {
            let check = exposure_check(run, source, target, actual, 4.0);
            assert_eq!(
                (check.source_id, check.target_id, check.satisfied),
                (source, target, satisfied)
            );
            assert_eq!(check.tolerance, Some(4.0));
            assert_eq!(
                check.kind,
                pse_model::generated::enums::ModelingCheckKind::Check
            );
            let native = crate::workflow::numerics::CandidateDecision {
                usability: pse_model::generated::enums::CandidateUse::Usable,
                qualifiers: vec![],
                refusals: vec![],
                bound: None,
            };
            let completed = crate::workflow::numerics::original_completion(
                native,
                crate::workflow::numerics::CompletionEvidence::point(
                    std::slice::from_ref(&check),
                    true,
                    0,
                ),
                &pse_model::numerics::NumericalPolicy::default(),
            );
            assert_eq!(completed.permits_use(), satisfied);
        }
    }
    #[test]
    fn every_exposure_bound_remains_conjunctive_in_both_orders() {
        let run = RunId::from_bytes([1; 16]);
        let source = DeclarationId::from_bytes([2; 16]);
        let target = SemanticId::from_bytes([3; 16]);
        let loose = exposure_check(run, source, target, 3.0, 4.0);
        let tight = exposure_check(run, source, target, 3.0, 2.0);
        for checks in [
            [loose.clone(), tight.clone()],
            [tight.clone(), loose.clone()],
        ] {
            let native = crate::workflow::numerics::CandidateDecision {
                usability: pse_model::generated::enums::CandidateUse::Usable,
                qualifiers: vec![],
                refusals: vec![],
                bound: None,
            };
            let completed = crate::workflow::numerics::original_completion(
                native,
                crate::workflow::numerics::CompletionEvidence::point(&checks, true, 0),
                &pse_model::numerics::NumericalPolicy::default(),
            );
            assert!(!completed.permits_use());
        }
    }
    #[test]
    fn projected_exposure_copies_coalesce_to_one_tightest_check_key() {
        use pse_model::generated::{
            authored::caloric_requirements::AuthoredCaloricRequirementsFieldConsumersItem as Consumer,
            enums::{NumericalSource, NumericalTarget},
        };
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let flow = match registry.physical_name("Flow").unwrap() {
            pse_quantity::PhysicalName::QuantityType(id) => id,
            _ => panic!("flow quantity"),
        };
        let unit = registry.quantity_type(flow).unwrap().canonical_unit;
        let id = |n| SemanticId::from_bytes([n; 16]);
        let mut policy = pse_model::numerics::NumericalPolicy::default();
        let row = |target, consumer_target, bound| pse_model::numerics::CaloricRequirement {
            requirement_id: target,
            instance_id: None,
            target_id: target,
            target_kind: NumericalTarget::Variable,
            quantity_id: id(1),
            unit_id: id(2),
            source: NumericalSource::Model,
            science_revision: "revision".into(),
            science_records: vec![],
            response_quantity_id: None,
            response_unit_id: None,
            response: None,
            thermal_quantity_id: None,
            thermal_unit_id: None,
            thermal_resolution: None,
            consumers: vec![Consumer {
                target_id: consumer_target,
                declaration_id: id(3),
                power_rule_id: id(4),
                actual_flow_id: id(5),
                exposure_expression: "bound".into(),
                power_quantity_id: id(6),
                power_unit_id: id(7),
                power_allowance: 1.0,
                exposure_quantity_id: flow.as_id(),
                exposure_unit_id: unit.as_id(),
                aggregate_exposure: bound,
            }],
        };
        policy.caloric_requirements = vec![row(id(8), id(10), 4.0), row(id(9), id(11), 2.0)];
        for _ in 0..2 {
            let exposures = exposure_bounds(&registry, &policy).unwrap();
            assert_eq!(
                exposures.len(),
                1,
                "one retained modeling-check primary key"
            );
            let exposure = &exposures[&(id(3), id(5))];
            assert_eq!(exposure.bound, 2.0);
            assert!(
                exposure.dependencies.contains(&id(10)) && exposure.dependencies.contains(&id(11))
            );
            assert!(
                !exposure_check(
                    RunId::from_bytes([1; 16]),
                    id(3).into(),
                    id(5),
                    3.0,
                    exposure.bound
                )
                .satisfied
            );
            policy.caloric_requirements.reverse();
        }
    }
}
