// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional original-coordinate accuracy work, admitted after original model checks.
use super::*;
pub(super) mod arithmetic;
use pse_math::engineering_accuracy::GoalResult;
use pse_model::{
    engineering_accuracy::BoundGoal,
    generated::enums::{
        AccuracyGoalSubject, AccuracyObservation, AccuracyUnavailableReason as U, NumericalTarget,
    },
};

fn absent(goals: &[pse_model::engineering_accuracy::AccuracyGoal], reason: U) -> Vec<GoalResult> {
    goals
        .iter()
        .cloned()
        .map(|g| GoalResult::unavailable(g, reason))
        .collect()
}
pub(crate) fn optional_failure(error: &ProblemError) -> bool {
    matches!(
        error,
        ProblemError::Numerical { .. }
            | ProblemError::Unsupported(_)
            | ProblemError::Native {
                kind: native::NativeFailureKind::Numerical,
                ..
            }
            | ProblemError::Linear {
                kind: native::LinearFailureKind::Numerical,
                ..
            }
    )
}
pub(super) fn retained_failure(
    error: Arc<ProblemError>,
    budget: &Arc<WorkerBudget>,
) -> Result<pse_math::engineering_accuracy::GoalFailure, ProblemError> {
    let bytes = error.retained_bytes();
    let cause = pse_model::diagnostic::DiagnosticCause::from_shared(error);
    let bytes = bytes
        .saturating_add(cause.allocation_overhead())
        .saturating_add(size_of::<pse_model::diagnostic::DiagnosticCause>());
    let owner = budget
        .charge(bytes)
        .map_err(MathRuntimeError::into_problem)?;
    Ok(pse_math::engineering_accuracy::GoalFailure::new(cause, bytes).with_owner(Arc::new(owner)))
}
impl PreparedSolve {
    /// Produce selected-coordinate evidence with one shared fresh original-square action.
    /// Direct coordinate projection has no arithmetic evaluator or opaque supplier of its
    /// own. Selected provider-free authored Observables use the retained function plan's
    /// First evaluation and exact-real arithmetic enclosure at this same physical point.
    pub(crate) fn coordinate_accuracy(
        &self,
        service: &MathService,
        outcome: &Outcome,
        values: &CaseValues,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Vec<GoalResult>, ProblemError> {
        let mut results =
            match self.selected_coordinate_accuracy(service, outcome, values, scope, budget) {
                Ok(results) => results,
                Err(error) if optional_failure(&error) => {
                    let failure = retained_failure(Arc::new(error), budget)?;
                    let mut results = absent(&self.numerics.policy.goals, U::EvaluatorUncertainty);
                    for result in &mut results {
                        result.failure = Some(failure.clone());
                    }
                    results
                }
                Err(error) => return Err(error),
            };
        match self.exact_objective_accuracy(outcome, values, scope, budget, &mut results) {
            Ok(()) => {}
            Err(error) if optional_failure(&error) => {
                let failure = retained_failure(Arc::new(error), budget)?;
                for result in &mut results {
                    if result.goal.subject == AccuracyGoalSubject::OptimalObjective
                        && result.evidence.is_none()
                    {
                        result.failure = Some(failure.clone());
                    }
                }
            }
            Err(error) => return Err(error),
        }
        objective_accuracy::retain_objective_uncertainty(&mut results);
        Ok(results)
    }

    fn selected_coordinate_accuracy(
        &self,
        service: &MathService,
        outcome: &Outcome,
        values: &CaseValues,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Vec<GoalResult>, ProblemError> {
        let goals = &self.numerics.policy.goals;
        if goals.is_empty() {
            return Ok(vec![]);
        }
        scope.check().map_err(ProblemError::from)?;
        if let Outcome::Constant(report) = outcome
            && let Some(receipt) = &report.certified_reconstruction
            && let Some(mut results) = receipt.assess_coordinates(self, values)?
        {
            self.certified_observable_accuracy(
                service,
                receipt,
                values,
                scope,
                budget,
                &mut results,
            )?;
            return Ok(results);
        }
        if let Some(accuracy) =
            self.kkt_coordinate_accuracy(service, outcome, values, scope, budget)?
        {
            return Ok(accuracy);
        }
        let Representation::Algebraic(case) = &self.representation else {
            return Ok(absent(goals, U::Unsupported));
        };
        // Expanded original equations retain selected supplier registrations as
        // metadata. Actual opaque arithmetic is refused by the exact projection
        // below; an unused registration does not withhold a provider-free source.
        let plan = &case.prepared.compiled().plan;
        let point = plan
            .columns()
            .iter()
            .map(|id| values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ProblemError::Contract("goal point lacks original coordinate".into()))?;
        let source = self.semantic_point_key(&point)?;
        let bound = goals
            .iter()
            .map(|g| {
                let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                product
                    .str("original-selected-output")
                    .id(&g.target_id)
                    .hash(&source.binding);
                if let Some(point) = source.point {
                    product.hash(&point);
                }
                if g.target_kind == NumericalTarget::Observable
                    && let Some(selected) = self.selected_output_program()
                {
                    product.hash(&selected.identity);
                    if let Some(row) = selected.rows.get(&g.target_id) {
                        product.id(row);
                    }
                }
                let mut unit = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                unit.str("physical-output-error")
                    .id(&g.quantity_id)
                    .id(&g.unit_id);
                BoundGoal {
                    declaration: g.clone(),
                    source,
                    product: product.finish_hash(),
                    normalization: unit.finish_hash(),
                }
            })
            .collect::<Vec<_>>();
        let mut supported = bound
            .iter()
            .map(|g| {
                g.declaration.subject == AccuracyGoalSubject::SelectedOutput
                    && g.declaration.observation == AccuracyObservation::Steady
                    && ((g.declaration.target_kind == NumericalTarget::Variable
                        && plan.columns().contains(&g.declaration.target_id))
                        || (g.declaration.target_kind == NumericalTarget::Observable
                            && self.selected_output_program().is_some_and(|program| {
                                program.rows.contains_key(&g.declaration.target_id)
                            })))
            })
            .collect::<Vec<_>>();
        if supported.iter().all(|s| !s) {
            return Ok(goals
                .iter()
                .map(|goal| {
                    GoalResult::unavailable(
                        goal.clone(),
                        if goal.target_kind == NumericalTarget::Observable
                            && goal.subject == AccuracyGoalSubject::SelectedOutput
                            && goal.observation == AccuracyObservation::Steady
                        {
                            U::EvaluatorUncertainty
                        } else {
                            U::UnsupportedObservation
                        },
                    )
                })
                .collect());
        }
        let Outcome::Native(report) = outcome else {
            return Ok(absent(goals, U::Unsupported));
        };
        if report.quality.as_ref().is_none_or(|q| !q.feasible())
            || report.validation_failure().is_some()
            || report.callback_failure().is_some()
            || report.candidate.as_ref().is_none_or(|c| c.primal != point)
        {
            return Ok(absent(goals, U::InvalidValidity));
        }
        let Some(observed) = &report.observation else {
            return Ok(absent(goals, U::MissingEvidence));
        };
        let Some(residual) = observed
            .equality_residuals
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(absent(goals, U::Unsupported));
        };
        let contract = native::assembled::contract(plan);
        let bounds = plan
            .structure()
            .rows()
            .iter()
            .map(|r| (r.lower, r.upper))
            .collect::<Vec<_>>();
        let square = match native::square_response::SquareScope::admit(
            &contract,
            plan.jacobian_pattern(),
            &bounds,
            Some(&case.prepared.compiled().structure),
        ) {
            Ok(scope) => scope,
            Err(_) => return Ok(absent(goals, U::Regularity)),
        };
        let n = point.len();
        let bytes = native::square_response::SparseFactor::allowance(n)
            .and_then(|b| b.checked_add(n.checked_mul(goals.len())?.checked_mul(32)?))
            .ok_or_else(|| ProblemError::memory("engineering output action extent"))?;
        let charge = budget
            .charge(bytes)
            .map_err(MathRuntimeError::into_problem)?;
        let mut worker = service
            .case_worker(case.case.clone(), case.providers.clone(), scope, budget)
            .map_err(MathRuntimeError::into_problem)?;
        let jacobian = budget
            .evaluate(|| {
                worker
                    .worker()
                    .jacobian(values)
                    .cloned()
                    .map_err(MathRuntimeError::from)
            })
            .map_err(MathRuntimeError::into_problem)?;
        let mut execution = Execution::within(
            scope.cancellation().clone(),
            &self.profile.controls,
            scope.clone(),
        )?;
        execution.work_admission = budget
            .admission()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let observable =
            self.evaluated_observable_outputs(service, values, scope, budget, &execution)?;
        for (goal, supported) in bound.iter().zip(&mut supported) {
            if goal.declaration.target_kind == NumericalTarget::Observable {
                *supported &= observable.as_ref().is_some_and(|outputs| {
                    outputs.outputs.contains_key(&goal.declaration.target_id)
                });
            }
        }
        let Some(arithmetic) = arithmetic::original_point(
            plan, values, &point, &residual, &jacobian, &execution, budget,
        )?
        else {
            return Ok(absent(goals, U::EvaluatorUncertainty));
        };
        let factor = match native::square_response::SparseFactor::prepare(
            native::square_response::SparseRequest {
                scope: &square,
                point: &point,
                values: &observed.values,
                tolerances: &self.tolerances,
                normalization: &self.normalization,
                key: source,
                bytes,
            },
            || Ok(jacobian),
            &execution,
        ) {
            Ok(factor) => factor.with_owner(Arc::new(charge)),
            Err(native::square_response::Withheld::Cause(cause)) if optional_failure(&cause) => {
                let failure = retained_failure(cause, budget)?;
                let mut results = absent(goals, U::Regularity);
                for result in &mut results {
                    result.failure = Some(failure.clone());
                }
                return Ok(results);
            }
            Err(native::square_response::Withheld::Cause(cause)) => {
                return Err(ProblemError::Math(pse_math::MathError::Typed {
                    retained: cause.retained_bytes(),
                    cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
                }));
            }
            Err(_) => return Ok(absent(goals, U::Regularity)),
        };
        // This receipt names completed derivative/guard evaluation and the factor's actual
        // original matching, physical feasibility and inactive-bound admission.
        let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        witness
            .str("admitted-original-first-coordinate-action")
            .hash(&source.structure)
            .hash(&source.binding)
            .hash(
                &source
                    .point
                    .ok_or_else(|| ProblemError::Contract("goal point key".into()))?,
            )
            .hash(&self.normalization.key());
        witness.hash(&arithmetic.witness);
        if let Some(outputs) = &observable {
            witness.hash(&outputs.projection).hash(&outputs.witness);
        }
        let validity = native::engineering_accuracy::SquareValidity {
            source,
            witness: witness.finish_hash(),
        };
        let gradients = bound
            .iter()
            .map(|g| {
                let mut gradient = vec![0.; n];
                if let Some(i) = plan
                    .columns()
                    .iter()
                    .position(|id| *id == g.declaration.target_id)
                {
                    gradient[i] = 1.;
                }
                gradient
            })
            .collect::<Vec<_>>();
        let zero_gradient_uncertainty = vec![0.; n];
        let outputs = bound
            .iter()
            .zip(&gradients)
            .zip(&supported)
            .filter_map(|((g, gradient), supported)| {
                if !supported {
                    return None;
                }
                if g.declaration.target_kind == NumericalTarget::Observable {
                    let output = observable.as_ref()?.outputs.get(&g.declaration.target_id)?;
                    return Some(native::engineering_accuracy::SquareOutput {
                        goal: g,
                        product: g.product,
                        normalization: g.normalization,
                        derivative_source: source,
                        value: output.value,
                        gradient: &output.gradient,
                        gradient_uncertainty: Some(&output.gradient_uncertainty),
                        uncertainty: Some(output.uncertainty),
                    });
                }
                let q = values.scalars[&g.declaration.target_id];
                Some(native::engineering_accuracy::SquareOutput {
                    goal: g,
                    product: g.product,
                    normalization: g.normalization,
                    derivative_source: source,
                    value: q,
                    gradient,
                    gradient_uncertainty: Some(&zero_gradient_uncertainty),
                    uncertainty: Some(0.0),
                })
            })
            .collect::<Vec<_>>();
        let evidence = native::engineering_accuracy::estimate_square(
            &factor,
            native::engineering_accuracy::SquareResidual {
                source,
                values: &residual,
                residual_uncertainty: Some(&arithmetic.residual),
                jacobian_uncertainty: Some(&arithmetic.jacobian),
            },
            Some(validity),
            &outputs,
            &execution,
        );
        let failure = match evidence.failure {
            Some(error) if optional_failure(&error) => {
                Some(retained_failure(Arc::new(error), budget)?)
            }
            Some(error) => return Err(error),
            None => None,
        };
        let mut actual = evidence.outputs.into_iter();
        let mut demands = evidence.work_demands.into_iter();
        let mut results = bound
            .into_iter()
            .zip(supported)
            .map(|(g, supported)| {
                if !supported {
                    let reason = if g.declaration.target_kind == NumericalTarget::Observable
                        && g.declaration.subject == AccuracyGoalSubject::SelectedOutput
                        && g.declaration.observation == AccuracyObservation::Steady
                    {
                        U::EvaluatorUncertainty
                    } else {
                        U::UnsupportedObservation
                    };
                    return GoalResult::unavailable(g.declaration, reason);
                }
                let demand = demands.next().flatten();
                match actual.next() {
                    Some(Ok(e)) => {
                        let mut result = GoalResult::assess(&g, Some(e));
                        result.work_demand = demand;
                        result
                    }
                    Some(Err(reason)) => GoalResult::unavailable(g.declaration, reason),
                    None => GoalResult::unavailable(g.declaration, U::MissingEvidence),
                }
            })
            .collect::<Vec<_>>();
        for result in &mut results {
            if result.classification.status
                == pse_model::generated::enums::AccuracyGoalStatus::Unresolved
            {
                result.failure = failure.clone();
            }
        }
        Ok(results)
    }
}

impl PreparedSolve {
    /// Prepare a same-backend attempt from actual output-adjoint row demands. Only the
    /// operational primal stopping budget and work identity change; scientific numerics,
    /// physical acceptance, normalization, source point and task scope remain frozen.
    pub(crate) fn refine_coordinate_accuracy(
        &self,
        goals: &[GoalResult],
        point: &[f64],
    ) -> Result<Option<Self>, ProblemError> {
        let Representation::Algebraic(case) = &self.representation else {
            return Ok(None);
        };
        let plan = &case.prepared.compiled().plan;
        let row_ids = plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        if row_ids.len() != self.normalization.rows.len() {
            return Err(ProblemError::Contract(
                "accuracy demand rows differ from prepared physical row scaling".into(),
            ));
        }
        let source = self.semantic_point_key(point)?;
        let mut row_demands = Vec::new();
        for goal in goals {
            let Some(demand) = &goal.work_demand else {
                continue;
            };
            let Some(evidence) = goal.evidence.as_ref() else {
                continue;
            };
            let bound = BoundGoal {
                declaration: goal.goal.clone(),
                source: evidence.source,
                product: evidence.accuracy.product,
                normalization: evidence.accuracy.normalization,
            };
            let actual_classification =
                pse_math::engineering_accuracy::classify(&bound, Some(evidence));
            let Some(value) = evidence.value else {
                continue;
            };
            let Some(allowance) =
                pse_math::engineering_accuracy::refinement_allowance(&bound, value)
            else {
                continue;
            };
            // A work receipt is tied to its exact original point and input lineage.
            // Stale or cross-point receipts cannot tighten this attempt.
            if demand.source != source
                || demand.product != evidence.accuracy.product
                || !demand.physical_output_allowance.is_finite()
                || demand.physical_output_allowance <= 0.0
                || demand.physical_output_allowance.to_bits() != allowance.to_bits()
                || !goal.goal.refine
                || goal.goal.required_class != pse_model::strategy::AccuracyClass::Estimated
                || actual_classification.status
                    != pse_model::generated::enums::AccuracyGoalStatus::Unresolved
                || goal.classification != actual_classification
            {
                continue;
            }
            if demand.rows.is_empty() {
                return Err(ProblemError::Contract(
                    "accuracy work receipt contains no residual-row demands".into(),
                ));
            }
            for row in &demand.rows {
                let Some(index) = row_ids.iter().position(|id| *id == row.row) else {
                    return Err(ProblemError::Contract(
                        "accuracy work receipt names a row outside the original problem".into(),
                    ));
                };
                if !row.physical_allowance.is_finite() || row.physical_allowance <= 0.0 {
                    return Err(ProblemError::Contract(
                        "accuracy work receipt has a nonpositive physical allowance".into(),
                    ));
                }
                row_demands.push((index, row.physical_allowance));
            }
        }
        if row_demands.is_empty() {
            return Ok(None);
        }
        let Some(backend) = self.backend() else {
            return Ok(None);
        };
        let projected = execution::adapter(backend).residual_work_tolerance(
            &self.profile.backend,
            execution::Budgets {
                accuracy: &self.accuracy,
                tolerances: &self.tolerances,
                normalization: &self.normalization,
            },
            &row_demands,
        )?;
        let Some(requested) = projected.map(|value| value.min(self.accuracy.feasibility)) else {
            return Ok(None);
        };
        if requested >= self.accuracy.feasibility {
            return Ok(None);
        }
        let mut refined = self.clone();
        refined.accuracy.feasibility = requested;
        let precision = pse_ids::document::of(
            pse_ids::Frame::SolverProfileV5,
            &("goal-work-precision", &refined.accuracy),
        )
        .map_err(|error| ProblemError::Internal(error.to_string()))?;
        refined.work_precision = Some(precision);
        if let Some(compatibility) = &mut refined.compatibility {
            compatibility.profile = pse_ids::document::of(
                pse_ids::Frame::SolverSessionV3,
                &("goal-work-precision", compatibility.profile, precision),
            )
            .map_err(|error| ProblemError::Internal(error.to_string()))?;
        }
        Ok(Some(refined))
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    #[test]
    fn engineering_accuracy_optional_failure_preserves_terminal_precedence_and_shared_storage() {
        assert!(optional_failure(&ProblemError::numerical(
            "optional action backward error"
        )));
        assert!(optional_failure(&ProblemError::Unsupported(
            "optional arithmetic capability".into()
        )));
        for error in [
            ProblemError::Cancelled,
            ProblemError::memory("enclosing grant"),
            ProblemError::Internal("required invariant".into()),
            ProblemError::Math(pse_math::MathError::Contract("required expression".into())),
        ] {
            assert!(!optional_failure(&error));
        }
        let budget = WorkerBudget::new(4096);
        let failure = retained_failure(
            Arc::new(ProblemError::numerical("optional backsolve")),
            &budget,
        )
        .unwrap();
        assert_eq!(budget.used(), failure.bytes);
        let shared = failure.clone();
        drop(failure);
        assert_eq!(budget.used(), shared.bytes);
        drop(shared);
        assert_eq!(budget.used(), 0);
    }
}

#[cfg(all(test, feature = "solver-kinsol", feature = "solver-root-isolation"))]
mod refinement_tests {
    use super::*;
    use crate::{math::strategy::admission::TaskAdmission, workflow::tests as fixture};
    use pse_authoring::{
        ParseBudget,
        language::{self, IdentityPolicy},
    };
    use pse_compiler::workspace::ModelingCaseBindings;
    use pse_ids::SemanticId;
    use pse_kernels::DerivativeOrder;
    use pse_modeling::{Bindings, Limits};
    use std::sync::Arc;

    #[tokio::test]
    async fn selected_scalar_observable_has_actual_first_arithmetic_and_report_only_is_pure() {
        let rows = language::parse(
            "package p { def Root { var x:Scalar; eq root:x*x==4; annotation start x(1.5); let q:Scalar=x*x/3; } }",
            SemanticId::NIL, IdentityPolicy::Named, ParseBudget::default(),
        ).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let runtime = fixture::runtime();
        let physical = fixture::physical();
        let quantities = physical.quantities.clone();
        let package = runtime.modeling_package(rows, physical).unwrap();
        let cancel = crate::CancelSource::new();
        let solver = SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Kinsol),
            presolve: native::presolve::Policy::Off,
            ..Default::default()
        };
        let mut bindings = Bindings::default();
        bindings.demand.push("q".into());
        let report_only = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                bindings.clone(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver.clone(),
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert!(report_only.solve.selected_output_program().is_none());
        let source = report_only.model.model.compiled();
        let target = source
            .model
            .symbols
            .iter()
            .find(|(_, symbol)| symbol.lineage.path.ends_with(".q"))
            .map(|(id, _)| *id)
            .unwrap();
        let pse_modeling::Type::Quantity(quantity) = &source.model.symbols[&target].ty else {
            panic!("physical scalar output");
        };
        let quantity = quantity.resolve(&quantities, &BTreeMap::new()).unwrap();
        let unit = quantities.quantity_type(quantity).unwrap().canonical_unit;
        let mut solver = solver;
        solver
            .numerics
            .goals
            .push(pse_model::engineering_accuracy::AccuracyGoal {
                goal_id: SemanticId::from_bytes([32; 16]).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: target,
                target_kind: NumericalTarget::Observable,
                quantity_id: quantity.as_id(),
                unit_id: unit.as_id(),
                subject: AccuracyGoalSubject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: Some(0.1),
                criterion_lower: None,
                criterion_upper: None,
                required_class: pse_model::strategy::AccuracyClass::Estimated,
                use_policy: pse_model::generated::enums::AccuracyGoalUse::Assess,
                refine: false,
                source: pse_model::generated::enums::NumericalSource::Analysis,
                priority: 0,
                provenance: "independent x=2, q=4/3 physical Observable oracle".into(),
            });
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                bindings,
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let Representation::Algebraic(case) = &prepared.solve.representation else {
            panic!("algebraic root");
        };
        let selected = prepared.solve.selected_output_program().unwrap();
        assert_eq!(
            selected.executable.assembly.columns(),
            case.prepared.compiled().plan.columns()
        );
        assert_eq!(selected.executable.assembly.structure().rows().len(), 1);
        let original_tolerances = prepared.solve.tolerances.clone();
        let original_normalization = prepared.solve.normalization.clone();
        let original_key = prepared.solve.numerics.key;
        let finished = runtime
            .native()
            .solve(prepared.solve.clone())
            .unwrap()
            .finish()
            .await
            .unwrap();
        let Outcome::Native(report) = &finished.outcome else {
            panic!("native root");
        };
        assert!(report.quality.as_ref().unwrap().feasible());
        let point = &report.candidate.as_ref().unwrap().primal;
        let mut values = case.values.clone();
        for (id, value) in case.prepared.compiled().plan.columns().iter().zip(point) {
            values.scalars.insert(*id, *value);
        }
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), None);
        let budget = WorkerBudget::new(64 << 20);
        let execution = Execution::within(
            scope.cancellation().clone(),
            &prepared.solve.profile.controls,
            scope.clone(),
        )
        .unwrap();
        let evaluated = prepared
            .solve
            .evaluated_observable_outputs(runtime.native(), &values, &scope, &budget, &execution)
            .unwrap()
            .unwrap();
        let output = &evaluated.outputs[&target];
        assert!((output.value - 4. / 3.).abs() < 0.1);
        assert!((output.gradient[0] - 4. / 3.).abs() < 0.1);
        assert!(output.gradient_uncertainty[0] > 0.);
        assert!(output.uncertainty >= 0.);
        drop(evaluated);
        let assessed = prepared
            .solve
            .coordinate_accuracy(
                runtime.native(),
                &finished.outcome,
                &values,
                &scope,
                &budget,
            )
            .unwrap();
        let evidence = assessed[0].evidence.as_ref().unwrap();
        assert_eq!(evidence.target, target);
        assert_eq!(
            evidence.method,
            pse_model::generated::enums::AccuracyEvidenceMethod::SquareCorrection
        );
        assert!((evidence.value.unwrap() - 4. / 3.).abs() < 0.1);
        assert_eq!(
            evidence.accuracy.class,
            pse_model::strategy::AccuracyClass::Estimated
        );
        assert!(evidence.accuracy.error.unwrap() > 0.);
        assert_eq!(
            prepared.solve.tolerances.variables,
            original_tolerances.variables
        );
        assert_eq!(prepared.solve.tolerances.rows, original_tolerances.rows);
        assert_eq!(
            prepared.solve.tolerances.integrality,
            original_tolerances.integrality
        );
        assert_eq!(prepared.solve.normalization, original_normalization);
        assert_eq!(prepared.solve.numerics.key, original_key);
        assert_eq!(budget.used(), 0);
    }

    #[tokio::test]
    async fn authored_original_square_goal_drives_same_contract_tighter_native_attempt() {
        let source = r#"package p {
          def Root {
            var x: Scalar;
            eq root: 0.0001 * (x*x - 4) == 0;
            annotation start x(1);
          }
        }"#;
        let rows = language::parse(
            source,
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let runtime = fixture::runtime();
        let package = runtime.modeling_package(rows, fixture::physical()).unwrap();
        let cancel = crate::CancelSource::new();
        let baseline = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                SolverProfile {
                    intent: SolveIntent::Root,
                    selection: SolverSelection::Explicit(Backend::Kinsol),
                    presolve: native::presolve::Policy::Off,
                    ..Default::default()
                },
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let Representation::Algebraic(baseline_case) = &baseline.solve.representation else {
            panic!("expected authored algebraic root");
        };
        let variable_id = baseline_case.prepared.compiled().plan.columns()[0];
        let variable = baseline_case
            .prepared
            .compiled()
            .plan
            .structure()
            .variables()
            .iter()
            .find(|variable| variable.port.id == variable_id)
            .expect("selected original coordinate");
        let mut solver = SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Kinsol),
            presolve: native::presolve::Policy::Off,
            ..Default::default()
        };
        solver
            .numerics
            .goals
            .push(pse_model::engineering_accuracy::AccuracyGoal {
                goal_id: SemanticId::from_bytes([31; 16]).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: variable_id,
                target_kind: NumericalTarget::Variable,
                quantity_id: variable.port.quantity.as_id(),
                unit_id: variable.port.unit.as_id(),
                subject: AccuracyGoalSubject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: Some(0.1),
                criterion_lower: None,
                criterion_upper: None,
                required_class: pse_model::strategy::AccuracyClass::Estimated,
                use_policy: pse_model::generated::enums::AccuracyGoalUse::Assess,
                refine: true,
                source: pse_model::generated::enums::NumericalSource::Analysis,
                priority: 0,
                provenance: "bounded original-coordinate accuracy refinement test".into(),
            });
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let scope = pse_kernels::ExecutionScope::new(
            Arc::default(),
            Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
        );
        let admission = TaskAdmission::new(
            pse_model::strategy::WorkLimits {
                attempts: 4,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
            scope.clone(),
            Some(runtime.native().pool.clone()),
            false,
        );
        let original = prepared
            .solve
            .within_admitted_task(scope.clone(), admission.clone())
            .unwrap();
        assert_eq!(original.numerics().policy.goals.len(), 1);
        let original_identity = original.original_identity().unwrap();
        let original_key = original.numerics().key;
        let original_normalization = original.normalization.clone();
        let original_tolerances = original.tolerances.clone();
        let original_work = original.accuracy().feasibility;
        assert!(
            original
                .refine_coordinate_accuracy(&[], &[1.0])
                .unwrap()
                .is_none()
        );

        let first = runtime
            .native()
            .solve(original.clone())
            .unwrap()
            .finish()
            .await
            .unwrap();
        let Outcome::Native(report) = &first.outcome else {
            panic!(
                "expected the selected native root solve: {:?}",
                first.outcome
            );
        };
        let point = report
            .candidate
            .as_ref()
            .expect("native root candidate")
            .primal
            .clone();
        let Representation::Algebraic(case) = &original.representation else {
            panic!("expected authored algebraic root");
        };
        let mut values = case.values.clone();
        for (id, value) in case.prepared.compiled().plan.columns().iter().zip(&point) {
            values.scalars.insert(*id, *value);
        }
        let budget = WorkerBudget::new(64 << 20);
        let observed = original
            .coordinate_accuracy(runtime.native(), &first.outcome, &values, &scope, &budget)
            .unwrap();
        assert_eq!(observed.len(), 1);
        assert_eq!(
            observed[0].classification.status,
            pse_model::generated::enums::AccuracyGoalStatus::Unresolved
        );
        assert!(observed[0].work_demand.is_some());

        let refined = original
            .refine_coordinate_accuracy(&observed, &point)
            .unwrap()
            .expect("actual unresolved output adjoint demands tighter native feasibility");
        assert!(refined.accuracy().feasibility < original_work);
        assert_eq!(refined.numerics().key, original_key);
        assert_eq!(refined.normalization, original_normalization);
        assert_eq!(refined.tolerances.variables, original_tolerances.variables);
        assert_eq!(refined.tolerances.rows, original_tolerances.rows);
        assert_eq!(
            refined.tolerances.integrality,
            original_tolerances.integrality
        );
        assert_eq!(refined.original_identity().unwrap(), original_identity);
        assert_ne!(
            refined.strategy_profile().unwrap(),
            original.strategy_profile().unwrap()
        );
        assert!(refined.work_precision().is_some());
        assert_eq!(
            refined.task_scope.as_ref().and_then(|task| task.deadline()),
            original
                .task_scope
                .as_ref()
                .and_then(|task| task.deadline()),
        );
        assert!(Arc::ptr_eq(
            refined.task_admission.as_ref().unwrap(),
            original.task_admission.as_ref().unwrap(),
        ));

        let second = runtime
            .native()
            .solve(refined.clone())
            .unwrap()
            .finish()
            .await
            .unwrap();
        let Outcome::Native(second_report) = &second.outcome else {
            panic!("expected refined native root solve: {:?}", second.outcome);
        };
        let point = second_report
            .candidate
            .as_ref()
            .expect("refined candidate")
            .primal
            .clone();
        for (id, value) in case.prepared.compiled().plan.columns().iter().zip(&point) {
            values.scalars.insert(*id, *value);
        }
        let final_goals = refined
            .coordinate_accuracy(runtime.native(), &second.outcome, &values, &scope, &budget)
            .unwrap();
        assert_eq!(final_goals.len(), 1);
        let final_goal = &final_goals[0];
        assert_eq!(
            final_goal.classification.resolution,
            pse_model::generated::enums::AccuracyResolutionStatus::Met,
            "this supported original-square refinement must meet its authored resolution: {final_goal:?}"
        );
    }
}
