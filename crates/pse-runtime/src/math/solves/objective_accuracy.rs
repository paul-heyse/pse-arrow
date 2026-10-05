// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original optimum goals require material uncertainty for both primal and bound.
use pse_math::engineering_accuracy::GoalResult;
use pse_model::generated::enums::{AccuracyGoalSubject, AccuracyObservation,
    AccuracyUnavailableReason as U, NumericalTarget};
use super::*;

impl PreparedSolve {
    /// Optional exact Optimal objective evidence, after fresh original admission.
    pub(super) fn exact_objective_accuracy(&self, outcome: &Outcome, values: &CaseValues,
        scope: &pse_kernels::ExecutionScope, budget: &Arc<WorkerBudget>, results: &mut [GoalResult])
        -> Result<(), ProblemError> {
        let eligible = |result: &GoalResult| result.goal.subject == AccuracyGoalSubject::OptimalObjective
            && result.goal.observation == AccuracyObservation::Steady
            && result.goal.target_kind == NumericalTarget::Objective && result.evidence.is_none()
            && result.classification.unavailable != Some(U::InvalidValidity);
        if !results.iter().any(eligible) { return Ok(()); }
        #[cfg(feature = "solver-root-isolation")]
        {
            use native::root_isolation::{Ibex, PointArithmeticEvidence};
            use native::engineering_accuracy::{QualifiedScalar, objective_interval};
            use pse_model::{engineering_accuracy::BoundGoal, strategy::AccuracyClass};
            use pse_kernels::DerivativeOrder;
            let (Representation::Algebraic(case), Outcome::Native(report)) = (&self.representation, outcome)
                else { return Ok(()); };
            let Some(global) = report.evidence.global.as_ref() else { return Ok(()); };
            let Some(receipt) = report.global.as_ref().and_then(|record| record.exact_objective_transport.as_ref())
                else { return Ok(()); };
            if !global.exact || !global.readback || global.fidelity != pse_math::factorable::Fidelity::Exact
                || global.infeasible || global.sense != receipt.sense() || report.backend != Backend::Scip
                || report.quality.as_ref().is_none_or(|q| !q.feasible())
                || report.validation_failure().is_some() || report.callback_failure().is_some()
            { return Ok(()); }
            let Some(candidate) = &report.candidate else { return Ok(()); };
            let plan = &case.prepared.compiled().plan;
            let _point = budget.charge(plan.columns().len().saturating_mul(24))
                .map_err(MathRuntimeError::into_problem)?;
            let Some(point) = plan.columns().iter().map(|id| values.scalars.get(id).copied())
                .collect::<Option<Vec<_>>>() else { return Ok(()); };
            if point.iter().map(|v| v.to_bits()).ne(candidate.primal.iter().map(|v| v.to_bits())) { return Ok(()); }
            let Some(value) = report.observation.as_ref().and_then(|observed| observed.objective)
                .filter(|v| v.is_finite()) else { return Ok(()); };
            let mut execution = Execution::within(scope.cancellation().clone(), &self.profile.controls, scope.clone())?;
            execution.work_admission = budget.admission().map(|owner| -> Arc<dyn WorkAdmission> { owner });
            execution.check()?;
            let scratch = receipt.correspondence_bytes();
            let _scratch = budget.charge(scratch).map_err(MathRuntimeError::into_problem)?;
            let available = budget.capacity().saturating_sub(budget.used());
            let construction = budget.charge(available).map_err(MathRuntimeError::into_problem)?;
            let limit = available / 256;
            if limit == 0 { return Ok(()); }
            let original = plan.exact_value_factorable_program(values, limit, &execution.cancel)
                .map_err(|error| match error {
                    pse_math::factorable::FactorableError::Math(error) => ProblemError::Math(error),
                    error => ProblemError::Unsupported(error.to_string()),
                })?;
            let Some(bound_interval) = receipt.original_interval(&original, &point, &execution)? else { return Ok(()); };
            drop(original);
            drop(construction);
            drop(_scratch);
            let available = budget.capacity().saturating_sub(budget.used());
            let construction = budget.charge(available).map_err(MathRuntimeError::into_problem)?;
            let Some(program) = plan.point_arithmetic_program_for_order(values, DerivativeOrder::Value,
                available / 256, &execution.cancel).map_err(|error| match error {
                    pse_math::factorable::FactorableError::Math(error) => ProblemError::Math(error),
                    error => ProblemError::Unsupported(error.to_string()),
                })? else { return Ok(()); };
            drop(construction);
            let _program = budget.charge(program.retained_bytes()).map_err(MathRuntimeError::into_problem)?;
            let Some(objective) = program.objective else { return Ok(()); };
            let bytes = Ibex.point_arithmetic_workspace_bytes_for_order(&program, DerivativeOrder::Value)?;
            let _workspace = budget.charge(bytes).map_err(MathRuntimeError::into_problem)?;
            let _outputs = budget.charge(program.graph.residuals.len().saturating_mul(24))
                .map_err(MathRuntimeError::into_problem)?;
            let region = point.iter().map(|v| pse_math::implicit::ProofInterval {lower: *v, upper: *v}).collect::<Vec<_>>();
            let enclosed = Ibex.enclose_arithmetic_box_values_with_execution(&program, &region, bytes, &execution)?;
            let intervals = match enclosed {
                PointArithmeticEvidence::Enclosed {values, ..} => values,
                PointArithmeticEvidence::Incomplete {..} => return Ok(()),
                PointArithmeticEvidence::Interrupted {..} => return Err(ProblemError::Cancelled),
            };
            execution.check()?;
            let Some(primal_interval) = intervals.get(objective) else { return Ok(()); };
            // Native exact optimality proves the rational optimum independently.
            // A primal/dual interval is used only if the actual primal enclosure
            // reaches the whole transported bound enclosure in the authored sense.
            if match receipt.sense() {
                ObjectiveSense::Minimize => primal_interval.upper < bound_interval.upper,
                ObjectiveSense::Maximize => primal_interval.lower > bound_interval.lower,
            } { return Ok(()); }
            let radius = |lower: f64, upper: f64, center: f64| {
                let error = (lower - center).abs().max((upper - center).abs());
                if error == 0. { 0. } else { error.next_up() }
            };
            let primal_error = radius(primal_interval.lower, primal_interval.upper, value);
            let dual_value = bound_interval.lower;
            let dual_error = radius(bound_interval.lower, bound_interval.upper, dual_value);
            if !primal_error.is_finite() || !dual_error.is_finite() { return Ok(()); }
            let source = self.semantic_point_key(&point)?;
            let mut validity = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
            validity.str("original-exact-optimal-objective-correspondence")
                .hash(&receipt.validity()).hash(&program.key).hash(&program.graph.identity());
            let validity = validity.finish_hash();
            for result in results.iter_mut().filter(|result| eligible(result)) {
                let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                product.str("original-optimal-objective").id(&result.goal.target_id).hash(&validity);
                let mut normalization = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                normalization.str("physical-optimal-objective-error").id(&result.goal.quantity_id).id(&result.goal.unit_id);
                let goal = BoundGoal {declaration: result.goal.clone(), source,
                    product: product.finish_hash(), normalization: normalization.finish_hash()};
                let primal = QualifiedScalar {source, value, uncertainty: Some(primal_error),
                    class: AccuracyClass::Certified, validity};
                let dual = QualifiedScalar {source, value: dual_value, uncertainty: Some(dual_error),
                    class: AccuracyClass::Certified, validity};
                if let Ok(evidence) = objective_interval(&goal, primal, dual, receipt.sense()) {
                    *result = GoalResult::assess(&goal, Some(evidence));
                }
            }
        }
        #[cfg(not(feature = "solver-root-isolation"))]
        let _ = (outcome, values, scope, budget);
        Ok(())
    }
}

/// Preserve the precise current capability limit for recognized optimum goals.
/// Fresh original objective evaluation and a transported native original bound do
/// not supply independently admitted arithmetic error for either scalar. Native
/// gap, feasibility and stationarity budgets cannot fill that missing contract.
/// Original validity failures keep their stronger, independently relevant reason.
pub(super) fn retain_objective_uncertainty(results: &mut [GoalResult]) {
    for result in results {
        if result.goal.subject == AccuracyGoalSubject::OptimalObjective
            && result.goal.observation == AccuracyObservation::Steady
            && result.goal.target_kind == NumericalTarget::Objective
            && result.evidence.is_none()
            && matches!(result.classification.unavailable,
                Some(U::Unsupported | U::UnsupportedObservation | U::Regularity))
        {
            result.classification.unavailable = Some(U::EvaluatorUncertainty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::SemanticId;
    use pse_model::{engineering_accuracy::AccuracyGoal, generated::enums::*};
    fn goal() -> AccuracyGoal {
        AccuracyGoal {
            goal_id: SemanticId::from_bytes([1; 16]).into(), model_id: None,
            case_id: None, instance_id: None, fit_id: None,
            target_id: SemanticId::from_bytes([2; 16]), target_kind: NumericalTarget::Objective,
            quantity_id: SemanticId::from_bytes([3; 16]), unit_id: SemanticId::from_bytes([4; 16]),
            subject: AccuracyGoalSubject::OptimalObjective, observation: AccuracyObservation::Steady,
            time: None, resolution: Some(1.), criterion_lower: None, criterion_upper: None,
            required_class: NumericalAccuracyClass::Estimated, use_policy: AccuracyGoalUse::Assess,
            refine: false, source: NumericalSource::Analysis, priority: 0,
            provenance: "original optimum material uncertainty".into(),
        }
    }
    #[test]
    fn engineering_accuracy_objective_requires_material_bound_error_without_hiding_original_failure() {
        let mut selected = goal();
        selected.subject = AccuracyGoalSubject::SelectedOutput;
        selected.target_kind = NumericalTarget::Variable;
        let mut rows = vec![
            GoalResult::unavailable(goal(), U::UnsupportedObservation),
            GoalResult::unavailable(goal(), U::Regularity),
            GoalResult::unavailable(goal(), U::InvalidValidity),
            GoalResult::unavailable(selected, U::UnsupportedObservation),
        ];
        retain_objective_uncertainty(&mut rows);
        assert_eq!(rows[0].classification.unavailable, Some(U::EvaluatorUncertainty));
        assert_eq!(rows[1].classification.unavailable, Some(U::EvaluatorUncertainty));
        assert_eq!(rows[2].classification.unavailable, Some(U::InvalidValidity));
        assert_eq!(rows[3].classification.unavailable, Some(U::UnsupportedObservation));
        assert!(rows.iter().all(|row| row.evidence.is_none() && row.work_demand.is_none()));
    }

    #[cfg(all(feature = "solver-scip", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn engineering_accuracy_exact_original_optimum_uses_actual_scip_transport_and_value_arithmetic() {
        use crate::workflow::tests as fixture;
        use pse_backend_native::execution::{BackendSettings, ScipSettings};
        use pse_modeling::specialize::{Bindings, Limits};
        use pse_compiler::workspace::ModelingCaseBindings;
        use pse_kernels::DerivativeOrder;
        for (expression, sense, exact, certified, optimum) in [("3*x+2", "minimize", true, true, 5.),
            ("3*x+2", "maximize", true, true, 8.),
            ("x/3", "minimize", true, false, 1./3.), ("3*x+2", "minimize", false, false, 5.)] {
            let text = format!("package p {{ def Root {{ var x:Scalar; eq floor:x>=1;
                annotation bounds x(0,2); annotation start x(1);
                let f:Scalar={expression}; annotation objective f({sense}); annotation report f(\"f\");
                annotation accuracy_goal f(optimal_objective,steady,resolution=1e-6,
                    required_class=certified,use_policy=assess,refine=false); }} }}");
            let rows = pse_authoring::language::parse(&text, SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
            let root = rows.iter().find(|row| row.name == "Root").unwrap().declaration_id;
            let package = fixture::runtime_with(16 << 20, 16 << 20, 2 << 30)
                .modeling_package(rows, fixture::physical()).unwrap();
            let mut solver = fixture::profile();
            solver.intent = SolveIntent::Optimize;
            solver.selection = SolverSelection::Explicit(Backend::Scip);
            solver.backend = BackendSettings::Scip(ScipSettings {exact, ..ScipSettings::default()});
            solver.presolve = native::presolve::Policy::Off;
            solver.composition.policy = pse_model::strategy::CompositionPolicy::Declared;
            let cancel = crate::CancelSource::new();
            let compiler = fixture::compiler_profile();
            let mut prepared = package.prepare_solve(root, pse_modeling::specialize::root_instance(root),
                Bindings::default(), Limits::default(), ModelingCaseBindings::default(), DerivativeOrder::First,
                compiler, solver, NumericalInputs::default(), &cancel).await.unwrap();
            let mut declaration = prepared.solve.numerical_strategy();
            declaration.mechanisms[0].profile = prepared.solve.backend().map(|backend|
                pse_model::strategy::ProfileRef {backend, key: prepared.solve.strategy_profile().unwrap()});
            prepared.solve = prepared.solve.clone().with_strategy(declaration,
                vec![prepared.solve.clone().into()]).unwrap();
            let run = prepared.start().unwrap().wait().await.unwrap();
            let crate::workflow::RunReport::Modeling(results) = run.report().unwrap() else {
                panic!("expected actual modeling report");
            };
            let result = &results[0];
            let assessed = &run.completion().unwrap().accuracy_goals[0];
            let Outcome::Native(report) = &result.outcome else { panic!("{:?}", result.outcome); };
            assert_eq!(report.evidence.global.as_ref().unwrap().exact, exact);
            if certified {
                assert!(result.accepted,
                    "{expression} {sense} exact={exact}: validation={:?}, assessments={:?}, goal={assessed:?}, native={report:?}",
                    result.validation_error, run.completion().unwrap().assessments);
                assert_eq!(assessed.status, AccuracyGoalStatus::Satisfied, "{assessed:?}");
                assert_eq!(assessed.accuracy_class, Some(pse_model::strategy::AccuracyClass::Certified));
                assert_eq!(assessed.method, Some(AccuracyEvidenceMethod::QualifiedObjectiveInterval));
                assert_eq!(assessed.value, Some(optimum));
                let (lower, upper) = (assessed.interval_lower.unwrap(), assessed.interval_upper.unwrap());
                assert!(lower <= optimum && upper >= optimum);
                assert!(report.global.as_ref().unwrap().exact_objective_transport.is_some());
            } else {
                assert!(!result.accepted, "unresolved optimum evidence must preserve composed refusal");
                assert!(report.candidate.is_some() && report.quality.as_ref().unwrap().feasible());
                assert!(result.validation_error.is_none() && report.validation_failure().is_none());
                assert_eq!(assessed.unavailable, Some(U::EvaluatorUncertainty), "{assessed:?}");
                assert!(assessed.accuracy_class.is_none() && assessed.error.is_none());
            }
        }
    }
}
