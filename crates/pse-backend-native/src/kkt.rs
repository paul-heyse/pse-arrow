// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The KKT-point analysis at a qualified NLP candidate (Plan 22 S0, I1; PS-10, PS-12).
//!
//! One active-set KKT system is assembled from the original model's derivative programs at
//! the recovered candidate, in the layout `[x; active rows; active bound rows]`. Every
//! variable keeps its `x` row, and an active bound, including an `l == u` pin, stays a row
//! of its own, so the factor can name the variable behind each bound multiplier
//! ([`SensBacksolver::bound_rows`]). A lower bound's row is `−eⱼ` and an upper bound's
//! `+eⱼ`, the minimization convention `L = f + λᵀg − z_Lᵀx + z_Uᵀx` in which the candidate's
//! multipliers are recovered; a pin is oriented as a lower bound and carries `z_L − z_U`.
//!
//! **Coordinates.** The derivatives, the candidate and its multipliers are original
//! (physical) values; the matrix FERAL factors is the normalized one,
//! `K̃ = P·K·P / S_f` with `P = diag(S_x, S_f/S_r, S_f/S_x)` over the three blocks, which
//! is `[S_x H S_x / S_f, S_x Aᵀ S_r⁻¹; S_r⁻¹ A S_x, 0]` with unit bound rows. Pivoting,
//! zero-pivot detection and the condition estimate are scale dependent, and the normalized
//! matrix is the one the numerical policy declares well scaled (DP-10); inertia is invariant
//! under the congruence, so every verdict is coordinate free. [`KktFactor`] answers in the
//! original frame through the explicit back-map `K⁻¹ r = P·K̃⁻¹·(P r)/S_f`, so its solves
//! and the candidate's multipliers share one frame and no natural-units factor applies
//! (DP-11).
//!
//! **Verdicts.** With `A` the active gradients, of rank `r` among `a` active constraints,
//! `In(K) = In(ZᵀHZ) + (r, r, a − r)` for `Z` a basis of their null space. LICQ is read from
//! the inertia of `[I Aᵀ; A 0]`, `(n, r, a − r)`, and the reduced Hessian's inertia is the
//! difference, so a dependent active set and a singular reduced Hessian are distinct
//! verdicts. The null space of all active constraints lies inside the critical cone:
//! negative curvature there refutes a local minimizer, and positive definiteness there is
//! second-order sufficiency under strict complementarity. When weakly active constraints
//! exist, the same assembly with their rows released (couplings removed, diagonal `−1`)
//! tests the null space of the strongly active constraints, which contains the cone.
//!
//! **Sensitivities.** A sensitivity request (Plan 22 S1) runs the same analysis over the
//! solve's model with the requested parameters appended as pinned columns, after the report
//! is qualified, and reads the parametric step, the optimal value's derivatives and the
//! reduced Hessian from that factor through `pounce-sens-core` (the `sensitivity` module).
//! Each quantity is computed or withheld with its typed reason.
//!
//! **Inverse reduced Hessian.** A request over some of the solve's own columns (Plan 22 S3)
//! reads `B·K⁻¹·Bᵀ` with `B` selecting their rows of the `x` block from the step's factor,
//! after qualification. With `Z` a basis of the null space of the active gradients,
//! `[K⁻¹]ₓₓ = Z(ZᵀHZ)⁻¹Zᵀ`, so the block is the inverse reduced Hessian over the selected
//! columns: sIPOPT's reduced Hessian over free variables. Over a fit's parameter columns it
//! is the exact covariance of the estimate (ADR-0118 item 8).
use crate::{
    NlpOracle, ProblemError,
    quality::{self, Observation, Tolerances},
    solve::{Candidate, SolveIntent, SolveReport},
};
use pounce_sens_core::{SensBacksolver, backsolver::BoundRow};
use pse_math::{
    index::{Entry, OriginalCol, OriginalRow, TiVec},
    normalization::Normalization,
};
use std::sync::Arc;

mod sensitivity;
pub use sensitivity::{
    InverseReducedHessian, Parametric, ReducedHessian, Sensitivities, Sensitivity, Withheld,
};
pub(crate) use sensitivity::{derive, invert};

/// What the local analysis of an NLP step computes at its candidate. The caller selects it
/// (PS-11); the runner never derives it from the model or the intent.
#[derive(Debug, Default)]
pub struct Analysis {
    /// The KKT-point analysis of an optimizing candidate: activity, LICQ, second-order
    /// curvature, inertia, the condition estimate and the backsolve residual.
    pub second_order: bool,
    /// Parametric sensitivities and, on request, the reduced Hessian over the named
    /// parameters (Plan 22 S1).
    pub sensitivity: Option<Sensitivity>,
    /// The inverse reduced Hessian over these of the solve's own original columns, read
    /// from the step's KKT factor after qualification (Plan 22 S3): over a fit's parameter
    /// columns, the exact covariance of its estimate. It needs `second_order`.
    pub inverse_reduced_hessian: Option<Vec<OriginalCol>>,
}
impl Analysis {
    /// No analysis.
    pub const NONE: Self = Self {
        second_order: false,
        sensitivity: None,
        inverse_reduced_hessian: None,
    };
    /// The standing selection for a solve of `intent`: the KKT-point analysis for an
    /// optimization, and none for the feasibility purposes, which solve a constant
    /// objective whose curvature says nothing.
    pub fn for_intent(intent: SolveIntent) -> Self {
        Self {
            second_order: intent == SolveIntent::Optimize,
            sensitivity: None,
            inverse_reduced_hessian: None,
        }
    }
}

/// Budgets of the analysis, resolved by the runner from the step.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Budget {
    /// Normalized multiplier magnitude at or below which an active constraint is weakly
    /// active: the dual stationarity budget, below which a multiplier is not
    /// distinguishable from zero.
    pub dual: f64,
    /// Entry ceiling of the KKT matrix, the NLP run's dimension ceiling.
    pub limit: usize,
}

/// The limit that binds an active row or bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The lower limit.
    Lower,
    /// The upper limit.
    Upper,
    /// Equal limits: an equality row or a pinned variable.
    Equal,
}
/// Whether a row or a variable's bounds bind the candidate, and how firmly (PS-12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// No limit binds within its tolerance.
    Inactive,
    /// A limit binds and its normalized multiplier exceeds the dual budget, or the limits
    /// are equal: strict complementarity holds.
    Strong(Side),
    /// A limit binds and its normalized multiplier is within the dual budget of zero:
    /// strict complementarity fails.
    Weak(Side),
}
impl Activity {
    /// The binding limit of an active constraint.
    pub const fn side(self) -> Option<Side> {
        match self {
            Self::Inactive => None,
            Self::Strong(side) | Self::Weak(side) => Some(side),
        }
    }
    /// Whether a limit binds.
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::Inactive)
    }
    /// Whether a limit binds with a multiplier indistinguishable from zero.
    pub const fn is_weak(self) -> bool {
        matches!(self, Self::Weak(_))
    }
}
/// Linear independence of the active constraint gradients, read from the inertia of
/// `[I Aᵀ; A 0]` (PS-12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Licq {
    /// The active gradients are linearly independent.
    Independent,
    /// The active gradients span `active − deficiency` dimensions.
    Dependent {
        /// Number of active constraints beyond the rank of their gradients.
        deficiency: usize,
    },
}
/// The second-order verdict at the candidate, in the minimization convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Curvature {
    /// The reduced Hessian is positive definite on the null space of the strongly active
    /// constraints, which contains the critical cone: the candidate is a strict local
    /// minimizer (second-order sufficient conditions).
    Sufficient,
    /// A direction satisfying every active constraint has negative curvature: the candidate
    /// is not a local minimizer.
    #[serde(rename = "negative_curvature")]
    Negative,
    /// The reduced Hessian on the null space of the active gradients has a zero eigenvalue.
    /// A dependent active set is the separate [`Licq`] verdict.
    Singular,
    /// Weakly active constraints leave the critical cone between the two null spaces
    /// tested, and neither test decides.
    Undecided,
}
/// The local analysis at a qualified candidate: plain data, independent of the factor it
/// was read from.
#[derive(Clone, Debug, PartialEq)]
pub struct KktPoint {
    /// Activity of every row, keyed by its original row: the position of its identity in
    /// the report's `rows`.
    pub rows: TiVec<OriginalRow, Activity>,
    /// Activity of every variable's bounds, keyed by its original column: the position of
    /// its identity in the report's `variables`.
    pub bounds: TiVec<OriginalCol, Activity>,
    /// Independence of the active gradients.
    pub licq: Licq,
    /// What the inertia decides.
    pub curvature: Curvature,
    /// Certified inertia (positive, negative, zero) of the normalized KKT matrix over every
    /// variable and every active constraint.
    pub inertia: (usize, usize, usize),
    /// Inertia of the reduced Hessian on the null space of the active gradients.
    pub reduced: (usize, usize, usize),
    /// Hager–Higham estimate of the normalized KKT matrix's 1-norm condition number, a
    /// lower bound; `None` when unavailable.
    pub condition_1norm: Option<f64>,
    /// Normwise backward error `‖b − K̃w‖∞ / (‖K̃‖∞‖w‖∞ + ‖b‖∞)` of a refined backsolve of the
    /// consistent system `K̃w = K̃·1`; `None` when the solve failed or was not finite.
    pub residual: Option<f64>,
}
impl KktPoint {
    /// Active rows and bounds.
    pub fn active(&self) -> usize {
        self.rows
            .iter()
            .chain(self.bounds.iter())
            .filter(|a| a.is_active())
            .count()
    }
    /// Active rows and bounds whose multiplier is within the dual budget of zero.
    pub fn weakly_active(&self) -> usize {
        self.rows
            .iter()
            .chain(self.bounds.iter())
            .filter(|a| a.is_weak())
            .count()
    }
}
/// Why a requested local analysis produced no [`KktPoint`].
#[derive(Clone, Debug)]
pub enum Unavailable {
    /// The candidate carries no multipliers, or they failed recovery or their sign checks.
    Multipliers,
    /// The candidate violates its original tolerances.
    Infeasible,
    /// The profile evaluates no exact Hessian.
    Hessian,
    /// The KKT matrix exceeds the run's entry ceiling.
    Limit {
        /// Structural entries of the KKT matrix.
        entries: usize,
        /// The run's ceiling.
        limit: usize,
    },
    /// Evaluation or factorization failed; the typed cause is kept (DP-21).
    Failed(Arc<ProblemError>),
}
impl std::fmt::Display for Unavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Multipliers => f.write_str("the candidate has no qualified multipliers"),
            Self::Infeasible => f.write_str("the candidate is not feasible"),
            Self::Hessian => f.write_str("the profile has no exact Hessian"),
            Self::Limit { entries, limit } => write!(
                f,
                "the KKT matrix has {entries} entries, above the ceiling of {limit}"
            ),
            Self::Failed(error) => write!(f, "{error}"),
        }
    }
}
impl From<ProblemError> for Unavailable {
    fn from(error: ProblemError) -> Self {
        Self::Failed(Arc::new(error))
    }
}
/// A requested local analysis: its point, or why there is none.
pub type Local = Result<KktPoint, Unavailable>;

/// Where each variable and active constraint sits in the KKT system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// Variables: the `x` block holds every original column, in order.
    pub variables: usize,
    /// Active rows in KKT order, after the `x` block.
    pub rows: Vec<(OriginalRow, Side)>,
    /// Active bounds in KKT order, after the active rows.
    pub bounds: Vec<(OriginalCol, Side)>,
}
impl Layout {
    /// Order of the KKT system.
    pub fn dim(&self) -> usize {
        self.variables + self.rows.len() + self.bounds.len()
    }
    /// KKT row of an active original row.
    pub fn row(&self, row: OriginalRow) -> Option<usize> {
        self.rows
            .iter()
            .position(|(r, _)| *r == row)
            .map(|k| self.variables + k)
    }
    /// KKT row of an active bound on an original column.
    pub fn bound(&self, col: OriginalCol) -> Option<usize> {
        self.bounds
            .iter()
            .position(|(j, _)| *j == col)
            .map(|k| self.variables + self.rows.len() + k)
    }
}

/// The factored normalized KKT matrix of one step's analysis, answering in original
/// coordinates. Cloning shares the factor.
///
/// It lives for the step's analysis only and never enters a `SolveReport`; moving it into
/// `Retained`, charged to the job allowance, is the Y5c seam (I14).
#[derive(Clone)]
pub struct KktFactor {
    solver: Arc<feral::Solver>,
    matrix: Arc<feral::CscMatrix>,
    layout: Arc<Layout>,
    /// `P`: the original value of each KKT unknown per normalized unit.
    scales: Arc<[f64]>,
    /// `S_f`, the objective nominal.
    objective: f64,
    bound_rows: Arc<[BoundRow]>,
}
impl std::fmt::Debug for KktFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KktFactor")
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}
impl KktFactor {
    /// Where each variable and active constraint sits.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    /// The coordinate scale `S_x` of an original column.
    pub fn column_scale(&self, col: OriginalCol) -> f64 {
        self.scales[col.get()]
    }
    /// The objective nominal `S_f`.
    pub fn objective_scale(&self) -> f64 {
        self.objective
    }
    /// The same factor answering in normalized coordinates: `K̃ w = r` solved directly.
    pub fn normalized(&self) -> NormalizedFactor {
        NormalizedFactor(self.clone())
    }
}
/// A [`KktFactor`] answering in the normalized coordinates of the matrix it factored,
/// `K̃ = P·K·P / S_f`, where every active bound row is a unit row. A quantity read from it
/// is dimensionless under the declared coordinate scales.
#[derive(Clone, Debug)]
pub struct NormalizedFactor(KktFactor);
impl SensBacksolver for NormalizedFactor {
    fn dim(&self) -> usize {
        self.0.dim()
    }
    fn solve(&self, rhs: &[f64], lhs: &mut [f64]) -> bool {
        let n = self.dim();
        if rhs.len() != n || lhs.len() != n {
            return false;
        }
        let Ok(w) = self.0.solver.solve_refined(&self.0.matrix, rhs) else {
            return false;
        };
        if w.len() != n {
            return false;
        }
        lhs.copy_from_slice(&w);
        lhs.iter().all(|v| v.is_finite())
    }
}
impl SensBacksolver for KktFactor {
    fn dim(&self) -> usize {
        self.layout.dim()
    }
    /// `K lhs = rhs` in original coordinates: `lhs = P·K̃⁻¹·(P rhs)/S_f`, refined against the
    /// unperturbed normalized matrix.
    fn solve(&self, rhs: &[f64], lhs: &mut [f64]) -> bool {
        let n = self.dim();
        if rhs.len() != n || lhs.len() != n {
            return false;
        }
        let scaled: Vec<f64> = rhs
            .iter()
            .zip(self.scales.iter())
            .map(|(r, p)| r * p / self.objective)
            .collect();
        let Ok(w) = self.solver.solve_refined(&self.matrix, &scaled) else {
            return false;
        };
        if w.len() != n {
            return false;
        }
        for ((l, w), p) in lhs.iter_mut().zip(&w).zip(self.scales.iter()) {
            *l = w * p;
        }
        lhs.iter().all(|v| v.is_finite())
    }
    fn bound_rows(&self) -> Option<&[BoundRow]> {
        Some(&self.bound_rows)
    }
}

/// Record the requested local analysis of a report's candidate against the original model.
/// A report without a candidate or an original observation gets none; an unqualified one
/// gets the typed reason. Returns the factor when a quantity is still to be read from it
/// after qualification (an inverse reduced Hessian); otherwise it is dropped here.
pub(crate) fn attach(
    report: &mut SolveReport,
    oracle: &mut dyn NlpOracle,
    normalization: &Normalization,
    tolerances: &Tolerances,
    analysis: &Analysis,
    budget: Budget,
) -> Option<KktFactor> {
    if !analysis.second_order {
        return None;
    }
    let (Some(candidate), Some(observation)) = (&report.candidate, &report.observation) else {
        return None;
    };
    let local = if observation.dual_error.is_some() {
        Err(Unavailable::Multipliers)
    } else if !report
        .quality
        .as_ref()
        .is_some_and(quality::Quality::feasible)
    {
        Err(Unavailable::Infeasible)
    } else {
        analyse(
            oracle,
            candidate,
            observation,
            normalization,
            tolerances,
            budget,
        )
    };
    // The factor serves this step's analysis and is dropped with it; it never enters the
    // report. Keeping it for a later step (Y5c) moves it into `Retained` here, charged to
    // the job allowance (I14).
    let (local, factor) = match local {
        Ok((point, factor)) => (Ok(point), Some(factor)),
        Err(unavailable) => (Err(unavailable), None),
    };
    report.evidence.local = Some(local);
    factor.filter(|_| analysis.inverse_reduced_hessian.is_some())
}

/// Every stored entry of a faer pattern, in storage order, in the spaces `R` and `C`.
fn entries<R: From<usize>, C: From<usize>>(
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
) -> Vec<Entry<R, C>> {
    (0..pattern.ncols())
        .flat_map(|j| {
            pattern
                .row_idx_of_col(j)
                .map(move |i| Entry::new(R::from(i), C::from(j)))
        })
        .collect()
}

/// The activity of a constraint on `side` whose normalized multiplier is `multiplier`.
fn activity(side: Option<Side>, multiplier: f64, dual: f64) -> Activity {
    match side {
        None => Activity::Inactive,
        Some(Side::Equal) => Activity::Strong(Side::Equal),
        Some(side) if multiplier.abs() > dual => Activity::Strong(side),
        Some(side) => Activity::Weak(side),
    }
}
/// The binding side of `value` within `[lower, upper]`; `upward` breaks a tie between two
/// limits closer than the tolerance.
fn side(value: f64, (lower, upper): (f64, f64), tolerance: f64, upward: bool) -> Option<Side> {
    if lower == upper {
        return Some(Side::Equal);
    }
    let at_lower = lower.is_finite() && value - lower <= tolerance;
    let at_upper = upper.is_finite() && upper - value <= tolerance;
    match (at_lower, at_upper) {
        (true, true) if upward => Some(Side::Upper),
        (true, _) => Some(Side::Lower),
        (false, true) => Some(Side::Upper),
        (false, false) => None,
    }
}

/// Lower-triangle triplets of a symmetric matrix of order `order`.
#[derive(Clone, Debug, Default)]
struct Triplets {
    rows: Vec<usize>,
    cols: Vec<usize>,
    values: Vec<f64>,
}
impl Triplets {
    fn push(&mut self, i: usize, j: usize, value: f64) {
        self.rows.push(i.max(j));
        self.cols.push(i.min(j));
        self.values.push(value);
    }
    fn matrix(&self, order: usize) -> Result<feral::CscMatrix, ProblemError> {
        if self.values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite KKT entry"));
        }
        feral::CscMatrix::from_triplets(order, &self.rows, &self.cols, &self.values)
            .map_err(crate::conditioning::native)
    }
}

/// The KKT-point analysis of a feasible candidate with multipliers, and the factor it was
/// read from (see the module documentation).
///
/// # Errors
/// Missing multipliers or Hessian, a KKT matrix above the entry ceiling, a failed
/// evaluation or a failed factorization.
pub(crate) fn analyse(
    oracle: &mut dyn NlpOracle,
    candidate: &Candidate,
    observation: &Observation,
    normalization: &Normalization,
    tolerances: &Tolerances,
    budget: Budget,
) -> Result<(KktPoint, KktFactor), Unavailable> {
    let n = oracle.contract().variables.len();
    let m = oracle.contract().rows.len();
    let (Some(lambda), Some((zl, zu))) = (&candidate.row_dual, &candidate.bound_dual) else {
        return Err(Unavailable::Multipliers);
    };
    let x = &candidate.primal;
    if x.len() != n
        || normalization.variables.len() != n
        || normalization.rows.len() != m
        || tolerances.variables.len() != n
        || tolerances.rows.len() != m
        || observation.values.len() != m
        || observation.bounds.len() != m
        || lambda.len() != m
        || zl.len() != n
        || zu.len() != n
    {
        return Err(ProblemError::internal("KKT analysis dimensions").into());
    }
    let (sx, sr, sf) = (
        &normalization.variables,
        &normalization.rows,
        normalization.objective,
    );
    // Activity by original identity, from normalized multipliers.
    let rows: TiVec<OriginalRow, Activity> = observation
        .bounds
        .iter()
        .zip(&observation.values)
        .enumerate()
        .map(|(r, (&limits, &value))| {
            activity(
                side(value, limits, tolerances.rows[r], lambda[r] > 0.0),
                lambda[r] * sr[r] / sf,
                budget.dual,
            )
        })
        .collect();
    let bounds: TiVec<OriginalCol, Activity> = oracle
        .contract()
        .variables
        .iter()
        .enumerate()
        .map(|(j, v)| {
            let side = side(x[j], (v.lower, v.upper), tolerances.variables[j], zu[j] > zl[j]);
            let multiplier = match side {
                Some(Side::Upper) => zu[j],
                Some(Side::Lower) => zl[j],
                Some(Side::Equal) | None => zl[j] - zu[j],
            };
            activity(side, multiplier * sx[j] / sf, budget.dual)
        })
        .collect();
    let layout = Layout {
        variables: n,
        rows: rows
            .iter_enumerated()
            .filter_map(|(r, a)| a.side().map(|s| (r, s)))
            .collect(),
        bounds: bounds
            .iter_enumerated()
            .filter_map(|(j, a)| a.side().map(|s| (j, s)))
            .collect(),
    };
    let order = layout.dim();
    let active = order - n;
    let mut position: TiVec<OriginalRow, Option<usize>> = vec![None; m].into();
    for (k, (r, _)) in layout.rows.iter().enumerate() {
        position[*r] = Some(n + k);
    }
    let hessian = entries::<OriginalCol, OriginalCol>(
        oracle.hessian_pattern().ok_or(Unavailable::Hessian)?,
    );
    let jacobian = entries::<OriginalRow, OriginalCol>(oracle.jacobian_pattern());
    let coupled = jacobian
        .iter()
        .filter(|e| position[e.row].is_some())
        .count();
    let size = order + hessian.len() + coupled + layout.bounds.len();
    if size > budget.limit {
        return Err(Unavailable::Limit {
            entries: size,
            limit: budget.limit,
        });
    }
    // Derivatives of the original Lagrangian at the original candidate, in one evaluation.
    let (h, a) = quality::contained(|| {
        let mut h = vec![0.0; hessian.len()];
        oracle.hessian(x, 1.0, lambda, &mut h)?;
        let mut a = vec![0.0; jacobian.len()];
        oracle.jacobian(x, &mut a)?;
        Ok((h, a))
    })?;
    // One assembly: the normalized KKT matrix, the matrix of the LICQ test, and each
    // active constraint's slots for a release.
    let mut kkt = Triplets::default();
    let mut licq = Triplets::default();
    for i in 0..order {
        kkt.push(i, i, 0.0);
        licq.push(i, i, if i < n { 1.0 } else { 0.0 });
    }
    for (e, v) in hessian.iter().zip(&h) {
        let (i, j) = (e.row.get(), e.col.get());
        kkt.push(i, j, sx[i] * v * sx[j] / sf);
    }
    // Per active constraint (KKT row minus n): the coupling entries of the KKT triplets.
    let mut couplings: Vec<Vec<usize>> = vec![vec![]; active];
    for (e, v) in jacobian.iter().zip(&a) {
        if let Some(row) = position[e.row] {
            let j = e.col.get();
            let value = v * sx[j] / sr[e.row.get()];
            couplings[row - n].push(kkt.values.len());
            kkt.push(row, j, value);
            licq.push(row, j, value);
        }
    }
    let base = n + layout.rows.len();
    for (k, (j, side)) in layout.bounds.iter().enumerate() {
        let value = if *side == Side::Upper { 1.0 } else { -1.0 };
        couplings[base + k - n].push(kkt.values.len());
        kkt.push(base + k, j.get(), value);
        licq.push(base + k, j.get(), value);
    }
    let matrix = kkt.matrix(order)?;
    // The rank of the active gradients: In([I Aᵀ; A 0]) = (n, r, a − r).
    let (mut diagnostic, (_, rank, _)) = crate::conditioning::factor(&licq.matrix(order)?)?;
    let deficiency = active
        .checked_sub(rank)
        .ok_or_else(|| ProblemError::numerical("LICQ inertia exceeds the active set"))?;
    let (solver, inertia) = crate::conditioning::factor(&matrix)?;
    let reduced = less(inertia, (rank, rank, deficiency)).ok_or_else(|| {
        ProblemError::numerical("KKT inertia disagrees with the rank of the active gradients")
    })?;
    let weak: Vec<usize> = layout
        .rows
        .iter()
        .map(|(r, _)| rows[*r])
        .chain(layout.bounds.iter().map(|(j, _)| bounds[*j]))
        .enumerate()
        .filter(|(_, a)| a.is_weak())
        .map(|(k, _)| k)
        .collect();
    let curvature = if reduced.1 > 0 {
        Curvature::Negative
    } else if reduced.2 > 0 {
        Curvature::Singular
    } else if weak.is_empty() {
        Curvature::Sufficient
    } else if deficiency > 0 {
        Curvature::Undecided
    } else {
        // The same assembly with the weak constraints released: each weak row reads
        // `−w = 0`, adding (0, 1, 0) per released row to the strongly active KKT inertia.
        let mut released = kkt.clone();
        for k in &weak {
            for slot in &couplings[*k] {
                released.values[*slot] = 0.0;
            }
            released.values[n + k] = -1.0;
        }
        let inertia =
            crate::conditioning::factor_into(&mut diagnostic, &released.matrix(order)?)?;
        // The strongly active gradients are independent, a subset of an independent set.
        let strong = active - weak.len();
        match less(inertia, (strong, strong + weak.len(), 0)) {
            Some((_, 0, 0)) => Curvature::Sufficient,
            Some(_) => Curvature::Undecided,
            None => {
                return Err(ProblemError::numerical(
                    "released KKT inertia disagrees with the strongly active set",
                )
                .into());
            }
        }
    };
    let condition_1norm = solver
        .estimate_condition_1norm(&matrix)
        .ok()
        .filter(|v| v.is_finite());
    let residual = backward_error(&solver, &matrix);
    let mut scales = Vec::with_capacity(order);
    scales.extend_from_slice(sx);
    scales.extend(layout.rows.iter().map(|(r, _)| sf / sr[r.get()]));
    scales.extend(layout.bounds.iter().map(|(j, _)| sf / sx[j.get()]));
    let bound_rows: Vec<BoundRow> = layout
        .bounds
        .iter()
        .enumerate()
        .map(|(k, (j, side))| BoundRow {
            row: base + k,
            var_row: j.get(),
            lower: *side != Side::Upper,
        })
        .collect();
    let point = KktPoint {
        rows,
        bounds,
        licq: if deficiency == 0 {
            Licq::Independent
        } else {
            Licq::Dependent { deficiency }
        },
        curvature,
        inertia,
        reduced,
        condition_1norm,
        residual,
    };
    let factor = KktFactor {
        solver: Arc::new(solver),
        matrix: Arc::new(matrix),
        layout: Arc::new(layout),
        scales: scales.into(),
        objective: sf,
        bound_rows: bound_rows.into(),
    };
    Ok((point, factor))
}

/// `inertia − known`, the inertia left to the reduced Hessian; `None` when a count would
/// be negative, which only an inconsistent factorization produces.
fn less(
    (p, q, z): (usize, usize, usize),
    (dp, dq, dz): (usize, usize, usize),
) -> Option<(usize, usize, usize)> {
    Some((p.checked_sub(dp)?, q.checked_sub(dq)?, z.checked_sub(dz)?))
}
/// `y = K x` for a symmetric matrix stored as its lower triangle, with `|K|` in place of `K`
/// when `absolute`.
fn symmetric_product(matrix: &feral::CscMatrix, x: &[f64], absolute: bool) -> Vec<f64> {
    let mut y = vec![0.0; matrix.n];
    for j in 0..matrix.n {
        for k in matrix.col_ptr[j]..matrix.col_ptr[j + 1] {
            let i = matrix.row_idx[k];
            let v = if absolute {
                matrix.values[k].abs()
            } else {
                matrix.values[k]
            };
            y[i] += v * x[j];
            if i != j {
                y[j] += v * x[i];
            }
        }
    }
    y
}
/// Normwise backward error of a refined solve of the consistent system `K w = K·1`.
fn backward_error(solver: &feral::Solver, matrix: &feral::CscMatrix) -> Option<f64> {
    let ones = vec![1.0; matrix.n];
    let b = symmetric_product(matrix, &ones, false);
    let w = solver.solve_refined(matrix, &b).ok()?;
    let r = symmetric_product(matrix, &w, false);
    let infinity = |v: &[f64]| v.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
    // ‖K‖∞ is the largest absolute row sum.
    let norm = infinity(&symmetric_product(matrix, &ones, true));
    let residual = infinity(&b.iter().zip(&r).map(|(b, r)| b - r).collect::<Vec<_>>());
    let scale = norm * infinity(&w) + infinity(&b);
    let error = if scale > 0.0 { residual / scale } else { residual };
    error.is_finite().then_some(error)
}

#[cfg(test)]
mod tests;
