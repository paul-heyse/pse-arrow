// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned LM over the existing original weighted residual/response authority.
use super::*;
use levenberg_marquardt::{LeastSquaresProblem, TerminationReason};
use nalgebra::{DMatrix, DVector, Dyn, storage::Owned};
use std::cell::{Cell, RefCell};

pub(in crate::workflow::fitting) fn request_eligible(profile: &FitProfile) -> bool {
    profile.solver.intent == SolveIntent::Optimize
        && profile.solver.sensitivity.is_none()
        && profile.derivatives == FitDerivatives::Responses
        && matches!(
            profile.solver.controls.hessian,
            HessianMode::Auto | HessianMode::GaussNewton
        )
        && profile.solver.selection == native::solve::SolverSelection::Auto
        && profile.solver.composition.policy == pse_model::strategy::CompositionPolicy::Auto
        && profile.solver.composition.advice.is_none()
        && profile.solver.composition.recovery.is_empty()
        && profile.solver.composition.branch.connected.is_none()
        && profile.solver.composition.limits.is_none_or(|limits| {
            limits.iterations.is_none()
                && limits.factorizations.is_none()
                && limits.proof_steps.is_none()
        })
        && profile.solver.block_overrides.is_empty()
        && profile.solver.block_workers == 1
        && profile.solver.reconstruction.is_none()
        && profile.solver.controls.options.is_empty()
        && matches!(
            profile.solver.backend,
            native::execution::BackendSettings::Default
        )
        && !matches!(&profile.solver.presolve, native::presolve::Policy::Explicit { required, .. } if !required.is_empty())
}
pub(in crate::workflow::fitting) fn eligible(p: &FitProblem) -> bool {
    let n = p.contract.variables.len();
    let m = p.measurements.iter().filter(|m| m.included).count();
    n > 0
        && m >= n
        && p.bounds.is_empty()
        && p.contract
            .variables
            .iter()
            .all(|v| !v.lower.is_finite() && !v.upper.is_finite())
        && request_eligible(&p.profile)
        && workspace_eligible(
            m,
            n,
            p.profile.max_cells,
            p.runtime
                .shared
                .budget()
                .math
                .foreign_allowance(&p.profile.solver.controls),
        )
}
pub(in crate::workflow::fitting) fn workspace_eligible(
    m: usize,
    n: usize,
    max_cells: usize,
    allowance: usize,
) -> bool {
    m.checked_mul(n).is_some_and(|cells| cells <= max_cells)
        && matrix_bytes(m, n).is_ok_and(|bytes| bytes <= allowance)
}
/// All simultaneously retained dense QR, permutation, diagonal and candidate buffers.
/// The opaque foreign envelope remains the hard owner; this is a conservative shape refusal.
pub(in crate::workflow::fitting) fn matrix_bytes(
    m: usize,
    n: usize,
) -> Result<usize, ProblemError> {
    m.checked_mul(n)
        .and_then(|v| v.checked_mul(16))
        .and_then(|v| n.checked_mul(n)?.checked_mul(16)?.checked_add(v))
        .and_then(|v| m.checked_add(n)?.checked_mul(128)?.checked_add(v))
        .and_then(|v| v.checked_mul(size_of::<f64>()))
        .and_then(|v| v.checked_add(64 << 10))
        .ok_or_else(|| ProblemError::memory("LM dense QR/candidate extent"))
}
struct ResidualProblem {
    oracle: RefCell<FitOracle>,
    z: DVector<f64>,
    physical: RefCell<Vec<f64>>,
    free: Vec<usize>,
    error: RefCell<Option<Arc<ProblemError>>>,
    evaluations: Cell<usize>,
    jacobians: Cell<usize>,
    updates: Cell<usize>,
    update_limit: usize,
    evaluation_limit: usize,
}
impl ResidualProblem {
    fn retain<T>(&self, result: Result<T, ProblemError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                let mut first = self.error.borrow_mut();
                if first.is_none() {
                    *first = Some(Arc::new(error));
                }
                None
            }
        }
    }
    fn physical_point(&self, p: &FitProblem) -> Result<(), ProblemError> {
        let mut point = self.physical.borrow_mut();
        for (&column, value) in self.free.iter().zip(self.z.iter()) {
            let value = value * p.normalization.variables[column];
            if !value.is_finite() {
                return Err(ProblemError::numerical("LM current physical coordinate"));
            }
            point[column] = value;
        }
        Ok(())
    }
}
impl LeastSquaresProblem<f64, Dyn, Dyn> for ResidualProblem {
    type ParameterStorage = Owned<f64, Dyn>;
    type ResidualStorage = Owned<f64, Dyn>;
    type JacobianStorage = Owned<f64, Dyn, Dyn>;
    fn set_params(&mut self, z: &DVector<f64>) {
        if self.error.borrow().is_some() {
            return;
        }
        let result = (|| {
            self.oracle.borrow().execution.check()?;
            if self.updates.get() >= self.update_limit {
                return Err(ProblemError::Limit {
                    kind: native::LimitKind::Work,
                    detail: "LM conservative parameter-update allowance".into(),
                });
            }
            if z.len() != self.z.len() || z.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("LM parameter update extent/value"));
            }
            self.updates.set(self.updates.get() + 1);
            self.z.copy_from(z);
            Ok(())
        })();
        self.retain(result);
    }
    fn params(&self) -> DVector<f64> {
        self.z.clone()
    }
    fn residuals(&self) -> Option<DVector<f64>> {
        if self.error.borrow().is_some() {
            return None;
        }
        self.retain((|| {
            let mut oracle = self.oracle.borrow_mut();
            let p = oracle.prepared.clone();
            self.physical_point(&p)?;
            if self.evaluations.get() >= self.evaluation_limit {
                return Err(ProblemError::Limit {
                    kind: native::LimitKind::Work,
                    detail: "LM residual evaluation allowance".into(),
                });
            }
            let execution = oracle.execution.clone();
            execution.counted(
                native::solve::WorkEvidence {
                    evaluations: Some(1),
                    ..Default::default()
                },
                || {
                    self.evaluations.set(self.evaluations.get() + 1);
                    let point =
                        oracle.evaluate_demand(&self.physical.borrow(), Demand::default())?;
                    let scale = p.normalization.objective.sqrt();
                    let values = p
                        .measurements
                        .iter()
                        .zip(&point.predictions)
                        .filter(|(m, _)| m.included)
                        .map(|(m, prediction)| {
                            FitOracle::residual(m, *prediction).map(|v| v.0 / scale)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if values.iter().any(|v| !v.is_finite()) {
                        return Err(ProblemError::numerical("LM normalized residual"));
                    }
                    Ok(DVector::from_vec(values))
                },
            )
        })())
    }
    fn jacobian(&self) -> Option<DMatrix<f64>> {
        if self.error.borrow().is_some() {
            return None;
        }
        self.retain((|| {
            let mut oracle = self.oracle.borrow_mut();
            let p = oracle.prepared.clone();
            self.physical_point(&p)?;
            let execution = oracle.execution.clone();
            execution.counted(
                native::solve::WorkEvidence {
                    evaluations: Some(1),
                    ..Default::default()
                },
                || {
                    self.jacobians.set(self.jacobians.get() + 1);
                    let point =
                        oracle.evaluate_demand(&self.physical.borrow(), Demand::responses())?;
                    let mut mapping = vec![None; p.measurements.len()];
                    let mut rows = 0;
                    for (i, (measurement, prediction)) in
                        p.measurements.iter().zip(&point.predictions).enumerate()
                    {
                        if measurement.included {
                            mapping[i] =
                                Some((rows, FitOracle::residual(measurement, *prediction)?.1));
                            rows += 1;
                        }
                    }
                    let source = point.responses.matrix();
                    let mut result = DMatrix::zeros(rows, self.free.len());
                    for (j, &column) in self.free.iter().enumerate() {
                        let scale =
                            p.normalization.variables[column] / p.normalization.objective.sqrt();
                        for entry in source.col_range(column) {
                            if let Some((row, weight)) = mapping[source.row_idx()[entry]] {
                                result[(row, j)] = source.val()[entry] * weight * scale;
                            }
                        }
                    }
                    if result.iter().any(|v| !v.is_finite()) {
                        return Err(ProblemError::numerical("LM normalized response Jacobian"));
                    }
                    Ok(result)
                },
            )
        })())
    }
}
fn termination(reason: TerminationReason) -> FitResidualTermination {
    match reason {
        TerminationReason::User(detail) => FitResidualTermination::User {
            detail: detail.into(),
        },
        TerminationReason::Numerical(detail) => FitResidualTermination::Numerical {
            detail: detail.into(),
        },
        TerminationReason::ResidualsZero => FitResidualTermination::ResidualsZero,
        TerminationReason::Orthogonal => FitResidualTermination::Orthogonal,
        TerminationReason::Converged { ftol, xtol } => {
            FitResidualTermination::Converged { ftol, xtol }
        }
        TerminationReason::NoImprovementPossible(detail) => {
            FitResidualTermination::NoImprovementPossible {
                detail: detail.into(),
            }
        }
        TerminationReason::LostPatience => FitResidualTermination::EvaluationLimit,
        TerminationReason::NoParameters => FitResidualTermination::Invalid {
            detail: "no free parameters".into(),
        },
        TerminationReason::NoResiduals => FitResidualTermination::Invalid {
            detail: "no residuals".into(),
        },
        TerminationReason::WrongDimensions(detail) => FitResidualTermination::Invalid {
            detail: detail.into(),
        },
    }
}
pub(in crate::workflow::fitting) fn solve(
    p: &Arc<FitProblem>,
    execution: Execution,
    initial: &[f64],
    pin: Option<(usize, f64)>,
    scratch: Option<Arc<FitScratch>>,
) -> Result<(FitResidualReport, Vec<f64>), ProblemError> {
    solve_using(p, execution, initial, pin, scratch,
        p.residual_lm_criteria)
}
/// The same callback, normalization, work and join owner under an explicit library policy.
pub(in crate::workflow::fitting) fn solve_using(
    p: &Arc<FitProblem>, execution: Execution, initial: &[f64], pin: Option<(usize, f64)>,
    scratch: Option<Arc<FitScratch>>, criteria: ResidualLmCriteria,
) -> Result<(FitResidualReport, Vec<f64>), ProblemError> {
    if !eligible(p) {
        return Err(ProblemError::unsupported("residual LM eligibility changed"));
    }
    execution.check()?;
    let m = p.measurements.iter().filter(|m| m.included).count();
    let n = initial
        .len()
        .checked_sub(usize::from(pin.is_some()))
        .ok_or_else(|| ProblemError::internal("LM pin extent"))?;
    let bytes = matrix_bytes(m, n)?;
    if execution.memory.is_none_or(|allowance| bytes > allowance) {
        return Err(ProblemError::memory(
            "LM dense workspace exceeds actual admitted foreign allowance",
        ));
    }
    if let Some(admission) = &execution.work_admission {
        admission.admit_storage(
            native::solve::NativeStorageScope::Linear,
            "fit.lm.workspace",
            bytes,
            false,
        )?;
    }
    let mut physical = initial.to_vec();
    if let Some((column, value)) = pin {
        let target = physical
            .get_mut(column)
            .ok_or_else(|| ProblemError::internal("LM pinned column"))?;
        if !value.is_finite() {
            return Err(ProblemError::numerical("LM pinned value"));
        }
        *target = value;
    }
    if n == 0 {
        let _ = assess_pin(
            p,
            execution,
            &physical,
            pin.map_or(usize::MAX, |p| p.0),
            scratch,
        )?;
        return Ok((
            FitResidualReport {
                stopping: None,
                termination: FitResidualTermination::FixedEvaluation,
                residual_evaluations: 1,
                jacobian_evaluations: 1,
                parameter_updates: 0,
                failure: None,
                stationarity: Some(0.),
                qualification: native::solve::Qualification::Stationary,
            },
            physical,
        ));
    }
    let free: Vec<_> = (0..initial.len())
        .filter(|column| pin.is_none_or(|(pinned, _)| *column != pinned))
        .collect();
    let z = DVector::from_iterator(
        free.len(),
        free.iter()
            .map(|&column| physical[column] / p.normalization.variables[column]),
    );
    let update_limit = p.profile.solver.controls.iterations as usize;
    let evaluation_limit = update_limit
        .checked_mul(
            n.checked_add(1)
                .ok_or_else(|| ProblemError::memory("LM evaluation extent"))?,
        )
        .ok_or_else(|| ProblemError::memory("LM evaluation allowance"))?;
    let problem = ResidualProblem {
        oracle: RefCell::new(FitOracle::with_scratch(p.clone(), execution, scratch)?),
        z,
        physical: RefCell::new(physical),
        free,
        error: RefCell::new(None),
        evaluations: Cell::new(0),
        jacobians: Cell::new(0),
        updates: Cell::new(0),
        update_limit,
        evaluation_limit,
    };
    let (problem, report) = criteria.algorithm()
        .with_scale_diag(p.numerics.policy.native_scaling)
        .with_patience(update_limit)
        .minimize(problem);
    let checked = problem.oracle.borrow().execution.check();
    problem.retain(checked);
    let p = problem.oracle.borrow().prepared.clone();
    // The literal final library parameters, including rejected-trial restoration, define the candidate.
    problem.physical_point(&p)?;
    let candidate = problem.physical.borrow().clone();
    let evidence = FitResidualReport {
        stopping: Some(ResidualLmStopping { criteria, parameter_update_limit: update_limit, residual_evaluation_limit: evaluation_limit }),
        termination: termination(report.termination),
        residual_evaluations: problem.evaluations.get(),
        jacobian_evaluations: problem.jacobians.get(),
        parameter_updates: problem.updates.get(),
        failure: problem.error.borrow().clone().map(ProfileFailure::shared),
        stationarity: None,
        qualification: native::solve::Qualification::Unqualified,
    };
    Ok((evidence, candidate))
}

/// Original scaled gradient from one complete current residual/response point.
pub(super) fn stationarity(p: &FitProblem, point: &Point, pin: Option<usize>) -> f64 {
    let source = point.responses.matrix();
    let mut maximum = 0.0_f64;
    for column in 0..source.ncols() {
        if pin == Some(column) {
            continue;
        }
        let mut gradient = 0.;
        for entry in source.col_range(column) {
            let row = source.row_idx()[entry];
            if p.measurements[row].included {
                let Ok((residual, weight)) =
                    FitOracle::residual(&p.measurements[row], point.predictions[row])
                else {
                    return f64::INFINITY;
                };
                gradient += source.val()[entry] * residual * weight;
            }
        }
        let normalized =
            (gradient * p.normalization.variables[column] / p.normalization.objective).abs();
        if !normalized.is_finite() {
            return f64::INFINITY;
        }
        maximum = maximum.max(normalized);
    }
    maximum
}

/// Independent original point for a pinned likelihood solve; no optimizer point is reused.
pub(in crate::workflow::fitting) fn assess_pin(
    p: &Arc<FitProblem>,
    execution: Execution,
    candidate: &[f64],
    pin: usize,
    scratch: Option<Arc<FitScratch>>,
) -> Result<(f64, f64, bool), ProblemError> {
    let mut oracle = FitOracle::with_scratch(p.clone(), execution, scratch)?;
    let point = oracle.evaluate_demand(candidate, Demand::responses())?;
    let residuals = p
        .measurements
        .iter()
        .zip(&point.predictions)
        .filter(|(m, _)| m.included)
        .map(|(m, value)| FitOracle::residual(m, *value).map(|v| v.0 * v.0))
        .collect::<Result<Vec<_>, _>>()?;
    let objective = 0.5 * residuals.iter().sum::<f64>();
    if !objective.is_finite() {
        return Err(ProblemError::numerical("LM pinned original objective"));
    }
    let quality = native::quality::observed(
        &p.contract,
        &p.bounds,
        candidate,
        &point.constraints,
        &p.tolerances,
    )?;
    Ok((
        objective,
        stationarity(p, point, Some(pin)),
        quality.feasible(),
    ))
}
