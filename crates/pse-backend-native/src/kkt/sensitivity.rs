// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parametric sensitivity and the reduced Hessian at a qualified candidate (Plan 22 S1;
//! ADR-0118 items 5–7; PS-12).
//!
//! **Pins.** The request's callbacks are the solve's model with one more column per
//! parameter. The analysis pins those columns at the parameter values
//! ([`crate::transform::Pinned`]), sIPOPT's pin formulation, and assembles one KKT system
//! over them at the candidate. A pin is oriented as a lower bound, so its row is `−eₚ` and
//! its multiplier `ν = z_L − z_U` is the original-coordinate stationarity of the parameter
//! column, `ν = ∂f/∂p + (∂g/∂p)ᵀλ`: what a pinned solve returns as its bound multiplier,
//! and by the envelope theorem the derivative of the optimal value, `df*/dp = ν`.
//!
//! **Step.** Perturbing a pin by `Δp` changes its row's right-hand side by `−Δp`, so with
//! `IndexSchurData` selecting the pin rows with sign `−1`, `parametric_step` returns
//! `Δw = K⁻¹·(−e_pin)·Δp`: the primal step and the steps of every active multiplier, in
//! original units because the factor answers through the explicit normalization back-map.
//!
//! **Reduced Hessian.** Over the pin rows, `B K⁻¹ Bᵀ = [K⁻¹]_νν = −H_R` with
//! `H_R = d²f*/dp²`, so `compute_reduced_hessian` returns `−H_R` (gh#937: the pin path
//! selects multiplier rows, where the block of `K⁻¹` is minus the reduced Hessian itself,
//! not an inverse). It is computed on the normalized factor, where it is the dimensionless
//! `Ĥ = S_p·H_R·S_p / S_f` whose eigen-decomposition does not depend on the choice of
//! units, and mapped to physical units through the normalization.
//!
//! **Validity.** The quantities are withheld, with the failed condition recorded, unless
//! the candidate is qualified stationary or better in original coordinates with a
//! multiplier for every original row that passes original-coordinate complementarity
//! (F01: presolve is never switched off to recover one), and the parametric KKT point has
//! independent active gradients, strict complementarity and second-order sufficiency.
//!
//! **Inverse reduced Hessian** (Plan 22 S3). Over some of the solve's own columns,
//! `IndexSchurData` selects their rows of the `x` block with sign `+1`, and
//! `compute_reduced_hessian` returns `B·K⁻¹·Bᵀ`, the block of `Z(ZᵀHZ)⁻¹Zᵀ`: upstream
//! sIPOPT's reduced Hessian over free variables, an inverse (gh#937). It is read from the
//! normalized factor of the step's own analysis under the same validity, and mapped to
//! physical units through the normalization.
use super::{Budget, Curvature, KktFactor, KktPoint, Licq, Side, Unavailable};
use crate::{
    NlpOracle, OracleContract, ProblemError,
    quality::{self, Observation, Tolerances},
    solve::{Candidate, Qualification, SolveReport},
};
use pounce_sens_core::{IndexSchurData, SensApplication, SensOptions};
use pse_ids::SemanticId;
use pse_math::{binding::ObjectiveSense, index::OriginalCol};

/// A parametric sensitivity request: the parametric callbacks, the parameters and their
/// values, whether the reduced Hessian is wanted, and whether the factor is kept for an
/// advanced step. The runner computes it after qualification, while the analysis factor is
/// alive, and drops the factor with the step unless it is kept.
#[derive(Debug)]
pub struct Sensitivity {
    /// Callbacks over the solve's columns followed by one column per parameter, in the
    /// order of `parameters`, with the solve's rows and row bounds and the normalization
    /// of every column. The parameter columns may be unbounded: the analysis pins them.
    pub oracle: Box<dyn NlpOracle>,
    /// Each parameter's identity and value, in request order.
    pub parameters: Vec<(SemanticId, f64)>,
    /// Also compute the reduced Hessian over the parameters.
    pub reduced_hessian: bool,
    /// Keep the pinned parametric factor in the worker's retained state after the step, for
    /// an advanced-step prediction (Plan 22 Y5c2), when the sensitivities are certified.
    pub retain: bool,
}

impl Sensitivity {
    /// Admit the request against the callbacks the step solves: at least one parameter,
    /// none repeated or among the solve's columns, every value finite, and parametric
    /// callbacks whose columns are the solve's followed by the parameters and whose rows
    /// are the solve's.
    ///
    /// # Errors
    /// A contract error naming the violated rule.
    pub fn admit(&self, solve: &OracleContract) -> Result<(), ProblemError> {
        let parametric = self.oracle.contract();
        let ids: std::collections::BTreeSet<_> = self.parameters.iter().map(|(id, _)| id).collect();
        let n = solve.variables.len();
        if self.parameters.is_empty()
            || ids.len() != self.parameters.len()
            || self.parameters.iter().any(|(_, v)| !v.is_finite())
            || solve.variables.iter().any(|v| ids.contains(&v.id))
        {
            return Err(ProblemError::Contract(
                "a sensitivity names distinct parameters with finite values, none a solved column"
                    .into(),
            ));
        }
        if parametric.variables.len() != n + self.parameters.len()
            || parametric.rows != solve.rows
            || parametric.variables[..n]
                .iter()
                .zip(&solve.variables)
                .any(|(p, s)| p.id != s.id)
            || parametric.variables[n..]
                .iter()
                .zip(&self.parameters)
                .any(|(p, (id, _))| p.id != *id)
        {
            return Err(ProblemError::Contract(
                "the parametric callbacks are the solve's columns followed by the parameters, over the solve's rows".into(),
            ));
        }
        Ok(())
    }
}

/// Why a requested derived quantity was withheld (PS-12). The runtime maps it onto the
/// registry `WithheldReason` at the publication boundary.
#[derive(Clone, Debug)]
pub enum Withheld {
    /// No candidate was observed in original coordinates.
    NoCandidate,
    /// A multiplier of some original row or bound is missing or failed recovery, postsolve
    /// included.
    Multipliers,
    /// The recovered multipliers fail original-coordinate complementarity (F01).
    Complementarity,
    /// The candidate is not qualified stationary or better in original coordinates.
    Unqualified(Qualification),
    /// The KKT-point analysis of the parametric system produced no point.
    Analysis(Unavailable),
    /// The active constraint gradients are linearly dependent.
    Licq {
        /// Active constraints beyond the rank of their gradients.
        deficiency: usize,
    },
    /// Active constraints with a multiplier within the dual budget of zero: strict
    /// complementarity fails, and the active set may change under any perturbation.
    WeaklyActive {
        /// Weakly active rows and bounds.
        count: usize,
    },
    /// Second-order sufficiency does not hold.
    SecondOrder(Curvature),
    /// A backsolve or the eigen-decomposition against the factor failed.
    Backsolve,
}
impl std::fmt::Display for Withheld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCandidate => f.write_str("no candidate was observed in original coordinates"),
            Self::Multipliers => f.write_str("a multiplier is missing or failed recovery"),
            Self::Complementarity => {
                f.write_str("the recovered multipliers fail original-coordinate complementarity")
            }
            Self::Unqualified(q) => write!(f, "the candidate is qualified only as {}", q.as_str()),
            Self::Analysis(cause) => write!(f, "no KKT point: {cause}"),
            Self::Licq { deficiency } => {
                write!(f, "the active gradients are dependent (deficiency {deficiency})")
            }
            Self::WeaklyActive { count } => {
                write!(f, "{count} weakly active constraints: strict complementarity fails")
            }
            Self::SecondOrder(curvature) => {
                write!(f, "second-order sufficiency fails: {curvature:?}")
            }
            Self::Backsolve => f.write_str("a backsolve against the KKT factor failed"),
        }
    }
}

/// Parametric sensitivities at the candidate, one vector per parameter in request order,
/// in original physical units per parameter unit.
#[derive(Clone, Debug, PartialEq)]
pub struct Sensitivities {
    /// `dxⱼ/dpₖ` over the report's variables: `primal[k][j]`.
    pub primal: Vec<Vec<f64>>,
    /// `d(z_L − z_U)ⱼ/dpₖ` of each variable's bound multiplier; zero where no bound is
    /// active, since that multiplier stays zero.
    pub bounds: Vec<Vec<f64>>,
    /// `dλᵣ/dpₖ` of each original row's multiplier, in the minimization convention of the
    /// candidate's multipliers; zero for an inactive row.
    pub rows: Vec<Vec<f64>>,
    /// `df*/dpₖ` of the optimal value in the authored objective sense.
    pub objective: Vec<f64>,
}

/// The reduced Hessian over the parameters, `d²f*/dp²` in the authored objective sense,
/// with the eigen-decomposition of its normalized form.
#[derive(Clone, Debug, PartialEq)]
pub struct ReducedHessian {
    /// Row-major `np × np`, in objective units per parameter unit per parameter unit.
    pub values: Vec<f64>,
    /// Row-major dimensionless `Ĥ = S_p·H·S_p / S_f` under the declared coordinate scales.
    pub normalized: Vec<f64>,
    /// Eigenvalues of `normalized`, ascending.
    pub eigenvalues: Vec<f64>,
    /// Unit eigenvectors of `normalized`, column-major: column `k` belongs to eigenvalue
    /// `k`, its entries over the parameters in request order.
    pub eigenvectors: Vec<f64>,
    /// The coordinate scale `S_p` of each parameter, in request order.
    pub coordinate_scales: Vec<f64>,
    /// The objective's coordinate scale `S_f`.
    pub objective_scale: f64,
}

/// The inverse reduced Hessian over selected columns of the solve, `B·K⁻¹·Bᵀ` in the
/// authored objective sense: over a fit's parameter columns, the exact covariance of the
/// estimate (ADR-0118 item 8).
#[derive(Clone, Debug, PartialEq)]
pub struct InverseReducedHessian {
    /// The selected columns, in request order: the order of every matrix.
    pub columns: Vec<OriginalCol>,
    /// Row-major `k × k`, in column unit per column unit per objective unit.
    pub values: Vec<f64>,
    /// Row-major dimensionless block `S_f·S_c⁻¹·(B·K⁻¹·Bᵀ)·S_c⁻¹` under the declared
    /// coordinate scales.
    pub normalized: Vec<f64>,
    /// The coordinate scale `S_c` of each column, in request order.
    pub coordinate_scales: Vec<f64>,
    /// The objective's coordinate scale `S_f`.
    pub objective_scale: f64,
}

/// The derived quantities a sensitivity request asked for, each computed or withheld with
/// its reason (PS-12).
#[derive(Clone, Debug)]
pub struct Parametric {
    /// The parameters, in request order.
    pub parameters: Vec<SemanticId>,
    /// The KKT point of the pinned parametric system, when the analysis ran.
    pub point: Option<KktPoint>,
    /// The parametric sensitivities.
    pub sensitivities: Result<Sensitivities, Withheld>,
    /// The reduced Hessian; `None` when it was not requested.
    pub reduced_hessian: Option<Result<ReducedHessian, Withheld>>,
    /// Bytes of the parametric factor kept for an advanced-step prediction and charged to
    /// the job's allowance (Plan 22 Y5c2); `None` when none was kept.
    pub retained: Option<usize>,
}

impl Parametric {
    /// Every requested quantity withheld for one reason, before any analysis ran.
    pub fn withheld(parameters: Vec<SemanticId>, reduced_hessian: bool, reason: Withheld) -> Self {
        Self {
            parameters,
            point: None,
            sensitivities: Err(reason.clone()),
            reduced_hessian: reduced_hessian.then_some(Err(reason)),
            retained: None,
        }
    }
}

/// Compute the request's quantities at the report's qualified candidate, or record why
/// each is withheld. The factor lives for this call only, unless the request keeps it for
/// an advanced step and the sensitivities are certified: then it is returned, with what a
/// prediction needs, for the worker's retained state.
pub(crate) fn derive(
    report: &mut SolveReport,
    request: Sensitivity,
    tolerances: &Tolerances,
    sense: ObjectiveSense,
    budget: Budget,
) -> Option<super::Advance> {
    let parameters: Vec<SemanticId> = request.parameters.iter().map(|(id, _)| *id).collect();
    let values = request.parameters.clone();
    let (reduced, retain) = (request.reduced_hessian, request.retain);
    let (point, result, advance) = match analysed(report, request, tolerances, budget) {
        Ok(analysed) => {
            let result = certify(&analysed.point).and_then(|()| {
                compute(report, &analysed.factor, &analysed.multipliers, reduced, sense)
            });
            let advance = (retain && result.is_ok())
                .then(|| super::Advance::new(report, &analysed, values, tolerances))
                .flatten();
            (Some(analysed.point), result, advance)
        }
        Err(withheld) => (None, Err(withheld), None),
    };
    let (sensitivities, reduced_hessian) = match result {
        Ok((sensitivities, hessian)) => (Ok(sensitivities), hessian),
        Err(withheld) => (Err(withheld.clone()), reduced.then_some(Err(withheld))),
    };
    report.evidence.sensitivity = Some(Parametric {
        parameters,
        point,
        sensitivities,
        reduced_hessian,
        retained: advance.as_ref().map(super::Advance::bytes),
    });
    advance
}

/// The pinned parametric system at a qualified candidate, and what its analysis read there.
pub(super) struct Analysed {
    /// The KKT point of the pinned system.
    pub point: KktPoint,
    /// Its factor.
    pub factor: KktFactor,
    /// The pin multipliers, in parameter order.
    pub multipliers: Vec<f64>,
    /// The parametric Jacobian at the candidate, `(row, column, value)` over the solve's
    /// columns followed by the parameters.
    pub jacobian: Vec<(usize, usize, f64)>,
    /// The bounds of the solve's columns.
    pub bounds: Vec<(f64, f64)>,
}

/// The candidate's qualification: stationary or better with every multiplier recovered
/// and passing original-coordinate complementarity.
fn qualified(report: &SolveReport) -> Result<(&Candidate, &Observation), Withheld> {
    let (Some(candidate), Some(observation)) = (&report.candidate, &report.observation) else {
        return Err(Withheld::NoCandidate);
    };
    if observation.dual_error.is_some()
        || candidate.row_dual.is_none()
        || candidate.bound_dual.is_none()
    {
        return Err(Withheld::Multipliers);
    }
    if report
        .evidence
        .kkt
        .is_some_and(|k| k.complementarity == Some(false))
    {
        return Err(Withheld::Complementarity);
    }
    match report.qualification {
        Qualification::Stationary
        | Qualification::OptimalWithinTolerance
        | Qualification::GapQualified => Ok((candidate, observation)),
        other => Err(Withheld::Unqualified(other)),
    }
}

/// The pinned parametric system at the candidate: its point, its factor, the pin
/// multipliers and the parametric Jacobian.
fn analysed(
    report: &SolveReport,
    request: Sensitivity,
    tolerances: &Tolerances,
    budget: Budget,
) -> Result<Analysed, Withheld> {
    let (candidate, observation) = qualified(report)?;
    let failed = |message: &str| {
        Withheld::Analysis(Unavailable::from(ProblemError::internal(message.to_owned())))
    };
    let (n, np) = (report.variables.len(), request.parameters.len());
    let contract = request.oracle.contract();
    if contract.variables.len() != n + np
        || contract.rows != report.rows
        || contract.variables[..n]
            .iter()
            .map(|v| v.id)
            .ne(report.variables.iter().copied())
        || contract.variables[n..]
            .iter()
            .map(|v| v.id)
            .ne(request.parameters.iter().map(|(id, _)| *id))
    {
        return Err(failed(
            "the parametric callbacks differ from the solve's columns and parameters",
        ));
    }
    let normalization = request
        .oracle
        .normalization()
        .cloned()
        .ok_or_else(|| failed("the parametric callbacks carry no normalization"))?;
    let bounds = contract.variables[..n]
        .iter()
        .map(|v| (v.lower, v.upper))
        .collect();
    let pins: Vec<(usize, f64)> = request
        .parameters
        .iter()
        .enumerate()
        .map(|(k, (_, value))| (n + k, *value))
        .collect();
    let mut oracle = crate::transform::Pinned::new(request.oracle, &pins)
        .map_err(|e| Withheld::Analysis(e.into()))?;
    let (Some(lambda), Some((zl, zu))) = (&candidate.row_dual, &candidate.bound_dual) else {
        return Err(Withheld::Multipliers);
    };
    let mut primal = candidate.primal.clone();
    primal.extend(pins.iter().map(|(_, value)| value));
    // The pin multipliers: the stationarity of each parameter column at the candidate.
    let (multipliers, jacobian) = quality::contained(|| {
        let mut gradient = vec![0.0; n + np];
        oracle.gradient(&primal, &mut gradient)?;
        let (starts, rows) = {
            let pattern = oracle.jacobian_pattern();
            (pattern.col_ptr().to_vec(), pattern.row_idx().to_vec())
        };
        let mut jacobian = vec![0.0; rows.len()];
        oracle.jacobian(&primal, &mut jacobian)?;
        let multipliers = (n..n + np)
            .map(|col| {
                gradient[col]
                    + (starts[col]..starts[col + 1])
                        .map(|k| jacobian[k] * lambda[rows[k]])
                        .sum::<f64>()
            })
            .collect::<Vec<f64>>();
        let entries = (0..n + np)
            .flat_map(|col| (starts[col]..starts[col + 1]).map(move |k| (col, k)))
            .map(|(col, k)| (rows[k], col, jacobian[k]))
            .collect::<Vec<_>>();
        Ok((multipliers, entries))
    })
    .map_err(|e| Withheld::Analysis(e.into()))?;
    if multipliers.iter().any(|v| !v.is_finite()) {
        return Err(failed("nonfinite pin multiplier"));
    }
    let extended = Candidate {
        primal,
        row_dual: Some(lambda.clone()),
        bound_dual: Some((
            zl.iter()
                .copied()
                .chain(multipliers.iter().map(|v| v.max(0.0)))
                .collect(),
            zu.iter()
                .copied()
                .chain(multipliers.iter().map(|v| (-v).max(0.0)))
                .collect(),
        )),
        ..candidate.clone()
    };
    // A pin binds by its degenerate box whatever its tolerance.
    let tolerances = Tolerances {
        variables: tolerances
            .variables
            .iter()
            .copied()
            .chain(std::iter::repeat_n(0.0, np))
            .collect(),
        ..tolerances.clone()
    };
    let (point, factor) = super::analyse(
        &mut oracle,
        &extended,
        observation,
        &normalization,
        &tolerances,
        budget,
    )
    .map_err(Withheld::Analysis)?;
    Ok(Analysed {
        point,
        factor,
        multipliers,
        jacobian,
        bounds,
    })
}

/// Read the inverse reduced Hessian over `columns` from the step's own factor at its
/// qualified, certified candidate, or record why it is withheld. The factor is dropped
/// with this call.
pub(crate) fn invert(
    report: &mut SolveReport,
    factor: Option<KktFactor>,
    columns: &[OriginalCol],
    sense: ObjectiveSense,
) {
    let result = inverse(report, factor, columns, sense);
    report.evidence.inverse_reduced_hessian = Some(result);
}

fn inverse(
    report: &SolveReport,
    factor: Option<KktFactor>,
    columns: &[OriginalCol],
    sense: ObjectiveSense,
) -> Result<InverseReducedHessian, Withheld> {
    qualified(report)?;
    let failed = |message: &str| {
        Withheld::Analysis(Unavailable::from(ProblemError::internal(message.to_owned())))
    };
    let point = match &report.evidence.local {
        Some(Ok(point)) => point,
        Some(Err(unavailable)) => return Err(Withheld::Analysis(unavailable.clone())),
        None => return Err(failed("no KKT-point analysis at the candidate")),
    };
    certify(point)?;
    let factor = factor.ok_or_else(|| failed("the analysis kept no factor"))?;
    let n = factor.layout().variables;
    let k = columns.len();
    let rows = columns
        .iter()
        .map(|c| (c.get() < n).then(|| i32::try_from(c.get()).ok()).flatten())
        .collect::<Option<Vec<i32>>>()
        .ok_or_else(|| failed("a selected column is not a column of the KKT system"))?;
    let selector =
        IndexSchurData::from_parts(rows, vec![1; k]).map_err(|_| Withheld::Backsolve)?;
    let mut app = SensApplication::new(
        selector,
        factor.normalized(),
        SensOptions {
            compute_red_hessian: true,
            ..SensOptions::default()
        },
    );
    let mut raw = vec![0.0; k * k];
    if !app.compute_reduced_hessian(&mut raw) || raw.iter().any(|v| !v.is_finite()) {
        return Err(Withheld::Backsolve);
    }
    // Over x rows the Schur reduction returns B·K̃⁻¹·Bᵀ itself (column-major, symmetric)
    // in the minimization convention; the authored sense multiplies by the objective's
    // sign, the inverse of the authored Hessian.
    let sign = sense.sign();
    let normalized: Vec<f64> = (0..k * k)
        .map(|index| sign * raw[(index % k) * k + index / k])
        .collect();
    // K⁻¹ = P·K̃⁻¹·P / S_f, and P is the coordinate scale on the x block.
    let coordinate_scales: Vec<f64> = columns.iter().map(|c| factor.column_scale(*c)).collect();
    let objective_scale = factor.objective_scale();
    let values = (0..k * k)
        .map(|index| {
            let (i, j) = (index / k, index % k);
            normalized[index] * coordinate_scales[i] * coordinate_scales[j] / objective_scale
        })
        .collect();
    Ok(InverseReducedHessian {
        columns: columns.to_vec(),
        values,
        normalized,
        coordinate_scales,
        objective_scale,
    })
}

/// The point's verdicts: independent active gradients, strict complementarity and
/// second-order sufficiency.
fn certify(point: &KktPoint) -> Result<(), Withheld> {
    if let Licq::Dependent { deficiency } = point.licq {
        return Err(Withheld::Licq { deficiency });
    }
    let count = point.weakly_active();
    if count > 0 {
        return Err(Withheld::WeaklyActive { count });
    }
    match point.curvature {
        Curvature::Sufficient => Ok(()),
        other => Err(Withheld::SecondOrder(other)),
    }
}

/// The KKT rows of the pins of the solve's `n` columns' `np` parameters, in parameter
/// order; `None` when a pin is not a row of the system.
pub(super) fn pin_rows(factor: &KktFactor, n: usize, np: usize) -> Option<Vec<i32>> {
    (0..np)
        .map(|k| {
            factor
                .layout()
                .bound(OriginalCol::new(n + k))
                .and_then(|row| i32::try_from(row).ok())
        })
        .collect()
}

/// The parametric steps and, on request, the reduced Hessian against the factor.
fn compute(
    report: &SolveReport,
    factor: &KktFactor,
    multipliers: &[f64],
    reduced: bool,
    sense: ObjectiveSense,
) -> Result<(Sensitivities, Option<Result<ReducedHessian, Withheld>>), Withheld> {
    let layout = factor.layout();
    let (n, m, np) = (report.variables.len(), report.rows.len(), multipliers.len());
    let pins = pin_rows(factor, n, np).ok_or_else(|| {
        Withheld::Analysis(Unavailable::from(ProblemError::internal(
            "a pin is not a row of the KKT system",
        )))
    })?;
    let selector = || {
        IndexSchurData::from_parts(pins.clone(), vec![-1; np]).map_err(|_| Withheld::Backsolve)
    };
    let sign = sense.sign();
    let steps = SensApplication::new(
        selector()?,
        factor.clone(),
        SensOptions {
            run_sens: true,
            ..SensOptions::default()
        },
    );
    let mut sensitivities = Sensitivities {
        primal: Vec::with_capacity(np),
        bounds: Vec::with_capacity(np),
        rows: Vec::with_capacity(np),
        objective: multipliers.iter().map(|v| sign * v).collect(),
    };
    let mut delta = vec![0.0; np];
    let mut step = vec![0.0; layout.dim()];
    for k in 0..np {
        delta.fill(0.0);
        delta[k] = 1.0;
        if !steps.parametric_step(&delta, &mut step) {
            return Err(Withheld::Backsolve);
        }
        sensitivities.primal.push(step[..n].to_vec());
        // Layout `[x; active rows; active bounds]`: every inactive multiplier stays zero.
        let mut rows = vec![0.0; m];
        for (k, (r, _)) in layout.rows.iter().enumerate() {
            rows[r.get()] = step[layout.variables + k];
        }
        sensitivities.rows.push(rows);
        let base = layout.variables + layout.rows.len();
        let mut bounds = vec![0.0; n];
        for (k, (j, side)) in layout.bounds.iter().enumerate() {
            if j.get() < n {
                // The row carries z_L for a lower bound or a pin and z_U for an upper one.
                let dz = step[base + k];
                bounds[j.get()] = if *side == Side::Upper { -dz } else { dz };
            }
        }
        sensitivities.bounds.push(bounds);
    }
    let hessian = reduced.then(|| reduced_hessian(factor, selector, sign, np));
    Ok((sensitivities, hessian))
}

/// The reduced Hessian over the pins from the normalized factor, mapped to physical units.
fn reduced_hessian(
    factor: &KktFactor,
    selector: impl Fn() -> Result<IndexSchurData, Withheld>,
    sign: f64,
    np: usize,
) -> Result<ReducedHessian, Withheld> {
    let mut app = SensApplication::new(
        selector()?,
        factor.normalized(),
        SensOptions {
            compute_red_hessian: true,
            rh_eigendecomp: true,
            ..SensOptions::default()
        },
    );
    let (mut raw, mut w, mut v) = (vec![0.0; np * np], vec![0.0; np], vec![0.0; np * np]);
    if !app.compute_reduced_hessian_eigen(&mut raw, &mut w, &mut v)
        || raw.iter().chain(&w).chain(&v).any(|x| !x.is_finite())
    {
        return Err(Withheld::Backsolve);
    }
    // gh#937: over the pin rows the Schur reduction returns −Ĥ in the minimization
    // convention; the authored sense multiplies by the objective's sign.
    let orient = -sign;
    let normalized: Vec<f64> = (0..np * np)
        .map(|index| orient * raw[(index % np) * np + index / np])
        .collect();
    // Negation keeps the eigenvectors and maps each eigenvalue w to orient·w.
    let mut order: Vec<usize> = (0..np).collect();
    order.sort_by(|a, b| (orient * w[*a]).total_cmp(&(orient * w[*b])));
    let eigenvalues = order.iter().map(|k| orient * w[*k]).collect();
    let eigenvectors = order
        .iter()
        .flat_map(|k| v[k * np..(k + 1) * np].iter().copied())
        .collect();
    // Ĥ = S_p·H·S_p / S_f, so H = S_f·Ĥ / (S_p·S_p) through the normalization.
    let first = factor.layout().variables - np;
    let coordinate_scales: Vec<f64> = (0..np)
        .map(|k| factor.column_scale(OriginalCol::new(first + k)))
        .collect();
    let objective_scale = factor.objective_scale();
    let values = (0..np * np)
        .map(|index| {
            let (i, j) = (index / np, index % np);
            normalized[index] * objective_scale / (coordinate_scales[i] * coordinate_scales[j])
        })
        .collect();
    Ok(ReducedHessian {
        values,
        normalized,
        eigenvalues,
        eigenvectors,
        coordinate_scales,
        objective_scale,
    })
}


#[cfg(test)]
mod tests;
