// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opt-in native diagnostics preserve their scope and never replace the original solve.
use super::*;
use enum_map::EnumMap;
use pse_ids::SemanticId;

pub use crate::settings::highs::{Penalties, Request};
/// The LP of a MIP with its discrete columns fixed at the MIP solution, solved separately.
/// Its duals price the constraints conditional on that commitment (PS-12); they are not
/// duals of the MIP, whose discrete decisions have none. When it reaches the MIP
/// candidate's objective, they become the candidate's multipliers ([`FixedLp::price`]).
#[derive(Clone, Debug)]
pub struct FixedLp {
    /// Discrete columns and the closed boxes the LP commits them to.
    pub commitment: crate::transform::Commitment,
    /// Native termination of the fixed LP.
    pub termination: NativeTermination,
    /// Objective of the fixed LP, in the authored sense.
    pub objective: Option<f64>,
    /// Row multipliers conditional on the commitment, in the authored sense.
    pub row_dual: Option<Vec<f64>>,
    /// Reduced costs conditional on the commitment.
    pub reduced_costs: Option<Vec<f64>>,
}
impl FixedLp {
    /// Price the MIP `candidate` by this LP (ADR-0118 item 9): its multipliers become the
    /// candidate's, which then carries the commitment they are conditional on. The LP must
    /// have reached the candidate's own objective within the continuous gap budget, so its
    /// multipliers price that point. Both are in the native model's coordinates.
    ///
    /// # Errors
    /// Why the candidate cannot be priced; it is left unchanged.
    pub fn price(
        &self,
        candidate: &mut Candidate,
        accuracy: &ResolvedAccuracy,
    ) -> Result<(), String> {
        if self.termination.category != Termination::Success {
            return Err(format!(
                "the fixed-commitment LP ended {}",
                self.termination.category.as_str()
            ));
        }
        let (Some(lp), Some(mip)) = (self.objective, candidate.objective) else {
            return Err("an objective of the fixed-commitment LP or the candidate is unavailable".into());
        };
        if (lp - mip).abs() > accuracy.gap_absolute.max(accuracy.gap_relative * mip.abs()) {
            return Err(format!(
                "the fixed-commitment LP optimum {lp} differs from the candidate objective {mip}"
            ));
        }
        let (Some(rows), Some(columns)) = (&self.row_dual, &self.reduced_costs) else {
            return Err("the fixed-commitment LP has no multipliers".into());
        };
        candidate.row_dual = Some(rows.clone());
        candidate.reduced_costs = Some(columns.clone());
        candidate.commitment = Some(self.commitment.clone());
        Ok(())
    }
}
/// The variable basic at one basis position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basic {
    /// A structural column.
    Column(SemanticId),
    /// The slack of a row.
    Row(SemanticId),
}
/// Basic variables and rows of `B⁻¹`, in the native (normalized) model's coordinates.
#[derive(Clone, Debug)]
pub struct BasisInverse {
    /// The basic variable at every basis position.
    pub basic: Vec<Basic>,
    /// Requested rows of `B⁻¹` by basis position, as `(row index, value)` entries.
    pub rows: Vec<(usize, Vec<(usize, f64)>)>,
}
/// Native presolve of a copied model. The presolved LP is in presolve's own columns and
/// rows of the native (normalized) model.
#[derive(Clone, Debug)]
pub struct Presolved {
    /// Presolved columns.
    pub columns: usize,
    /// Presolved rows.
    pub rows: usize,
    /// Presolved nonzeros.
    pub nonzeros: usize,
    /// Objective offset of the presolved LP.
    pub offset: f64,
    /// Presolved costs.
    pub cost: Vec<f64>,
    /// Presolved column bounds.
    pub column_bounds: Vec<(f64, f64)>,
    /// Presolved row bounds.
    pub row_bounds: Vec<(f64, f64)>,
    /// Column starts of the presolved matrix.
    pub start: Vec<usize>,
    /// Row indices of the presolved matrix.
    pub index: Vec<usize>,
    /// Values of the presolved matrix.
    pub value: Vec<f64>,
    /// The presolved LP's optimal solution mapped back by native postsolve, in the model's
    /// columns; `None` for a discrete or quadratic model or without an optimal solution.
    pub postsolved: Option<Vec<f64>>,
}
/// One cut `lower ≤ Σ value·x[index] ≤ upper`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    /// Lower side.
    pub lower: f64,
    /// Upper side.
    pub upper: f64,
    /// `(column, coefficient)` entries.
    pub entries: Vec<(usize, f64)>,
}
/// The MIP solver's cut pool after root cut generation. Its column indices refer to the
/// MIP solver's presolved LP, not to the model's columns.
#[derive(Clone, Debug, PartialEq)]
pub struct CutPool {
    /// Columns of the presolved LP the cuts are stated in.
    pub columns: usize,
    /// Retained cuts, in pool order.
    pub cuts: Vec<Cut>,
    /// Cuts beyond the retention bound, counted only.
    pub dropped: usize,
}
impl CutPool {
    /// Entries retained per pool: 4 MiB of `(index, value)` pairs.
    const ENTRIES: usize = 1 << 18;
    /// Copy the callback's pool; `None` when the native arrays are inconsistent.
    pub(super) fn from_callback(out: &ffi::HighsCallbackDataOut) -> Option<Self> {
        let cuts = usize::try_from(out.cutpool_num_cut).ok()?;
        let nonzeros = usize::try_from(out.cutpool_num_nz).ok()?;
        let columns = usize::try_from(out.cutpool_num_col).ok()?;
        if cuts == 0 {
            return Some(Self {
                columns,
                cuts: vec![],
                dropped: 0,
            });
        }
        if out.cutpool_start.is_null()
            || out.cutpool_lower.is_null()
            || out.cutpool_upper.is_null()
            || nonzeros > 0 && (out.cutpool_index.is_null() || out.cutpool_value.is_null())
        {
            return None;
        }
        let (start, lower, upper) = unsafe {
            (
                std::slice::from_raw_parts(out.cutpool_start, cuts + 1),
                std::slice::from_raw_parts(out.cutpool_lower, cuts),
                std::slice::from_raw_parts(out.cutpool_upper, cuts),
            )
        };
        let (index, value): (&[i32], &[f64]) = if nonzeros == 0 {
            (&[], &[])
        } else {
            unsafe {
                (
                    std::slice::from_raw_parts(out.cutpool_index, nonzeros),
                    std::slice::from_raw_parts(out.cutpool_value, nonzeros),
                )
            }
        };
        let mut pool = Self {
            columns,
            cuts: vec![],
            dropped: 0,
        };
        let mut retained = 0usize;
        for k in 0..cuts {
            let (from, to) = (
                usize::try_from(start[k]).ok()?,
                usize::try_from(start[k + 1]).ok()?,
            );
            if from > to || to > nonzeros {
                return None;
            }
            if retained + (to - from) > Self::ENTRIES {
                pool.dropped = cuts - k;
                break;
            }
            retained += to - from;
            pool.cuts.push(Cut {
                lower: lower[k],
                upper: upper[k],
                entries: (from..to)
                    .map(|i| Some((usize::try_from(index[i]).ok()?, value[i])))
                    .collect::<Option<_>>()?,
            });
        }
        Some(pool)
    }
}
impl Penalties {
    /// Validate complete dimensions and finite per-entry weights before native mutation;
    /// the global weights are finite by type.
    pub fn validate(&self, n: usize, m: usize) -> Result<(), ProblemError> {
        if [(&self.lower, n), (&self.upper, n), (&self.rows, m)]
            .iter()
            .any(|(v, n)| {
                v.as_ref()
                    .is_some_and(|v| v.len() != *n || v.iter().any(|v| !v.is_finite()))
            })
        {
            return Err(ProblemError::Contract(
                "relaxation penalty dimensions/values".into(),
            ));
        }
        Ok(())
    }
}
/// IIS bound membership in original source coordinates; integer codes are retained.
#[derive(Clone, Debug)]
pub struct Iis {
    /// Original variable IDs and native IIS bound code.
    pub columns: Vec<(SemanticId, i32)>,
    /// Original row IDs and native IIS bound code.
    pub rows: Vec<(SemanticId, i32)>,
    /// Native status for every original column.
    pub column_status: Vec<i32>,
    /// Native status for every original row.
    pub row_status: Vec<i32>,
    /// Whether the diagnostic concerns a continuous relaxation of a discrete model.
    pub relaxation_only: bool,
}
/// One of `HiGHS`'s six basis sensitivity ranging families.
///
/// Variants are declared in the order of their published spellings, which is the order in
/// which results and metrics list them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, enum_map::Enum)]
pub enum RangeFamily {
    /// How far each column's upper bound may decrease before the basis changes.
    ColumnBoundDown,
    /// How far each column's lower bound may increase before the basis changes.
    ColumnBoundUp,
    /// How far each column's cost may decrease before the basis changes.
    ColumnCostDown,
    /// How far each column's cost may increase before the basis changes.
    ColumnCostUp,
    /// How far each row's bound may decrease before the basis changes.
    RowBoundDown,
    /// How far each row's bound may increase before the basis changes.
    RowBoundUp,
}
/// The coordinates a ranging family's entries are indexed by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeSide {
    /// One entry per original column.
    Column,
    /// One entry per original row.
    Row,
}
impl RangeFamily {
    /// The published spelling, as results and metrics name the family.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ColumnBoundDown => "column_bound_down",
            Self::ColumnBoundUp => "column_bound_up",
            Self::ColumnCostDown => "column_cost_down",
            Self::ColumnCostUp => "column_cost_up",
            Self::RowBoundDown => "row_bound_down",
            Self::RowBoundUp => "row_bound_up",
        }
    }
    /// The coordinates this family's entries are indexed by.
    pub const fn side(self) -> RangeSide {
        match self {
            Self::ColumnBoundDown
            | Self::ColumnBoundUp
            | Self::ColumnCostDown
            | Self::ColumnCostUp => RangeSide::Column,
            Self::RowBoundDown | Self::RowBoundUp => RangeSide::Row,
        }
    }
    /// Whether the family's values are cost coefficients, which carry objective units per
    /// variable unit, rather than bounds.
    pub const fn is_cost(self) -> bool {
        matches!(self, Self::ColumnCostDown | Self::ColumnCostUp)
    }
}
impl std::fmt::Display for RangeFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
/// One native ranging family, indexed by original columns or rows as its family's side
/// states.
#[derive(Clone, Debug)]
pub struct Range {
    /// Source variable or row coordinate.
    pub ids: Vec<SemanticId>,
    /// Limiting coefficient/bound value (infinity remains meaningful).
    pub value: Vec<f64>,
    /// Authored-sense objective at that limit.
    pub objective: Vec<f64>,
    /// Native entering variable index; negatives are native sentinels and indices
    /// >= number of columns refer to row slacks.
    pub entering: Vec<i32>,
    /// Native leaving variable index in the same documented augmented ordering.
    pub leaving: Vec<i32>,
}
impl Range {
    fn new(ids: Vec<SemanticId>) -> Self {
        let n = ids.len();
        Self {
            ids,
            value: vec![0.0; n],
            objective: vec![0.0; n],
            entering: vec![0; n],
            leaving: vec![0; n],
        }
    }
}
/// Separate relaxed attempt, explicitly outside original-model assurance.
#[derive(Clone, Debug)]
pub struct Relaxation {
    /// Raw native operation status.
    pub operation_status: i32,
    /// HiGHS restores the original model status after relaxation; this is not the
    /// relaxed solve's termination. A fresh copied model commonly reports NotSet.
    pub restored_status: NativeTermination,
    /// Weighted infeasibility objective retained by a successful native operation.
    pub penalty: Option<f64>,
    /// Available relaxed coordinates; never the original solve's candidate.
    pub primal: Option<Vec<f64>>,
}
/// Available native diagnostics and explicit reasons for requested unavailable data.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Native primal unbounded direction in original variable order.
    pub primal_ray: Option<Vec<f64>>,
    /// Native dual infeasibility ray in original row order.
    pub dual_ray: Option<Vec<f64>>,
    /// Native irreducible subsystem if found.
    pub iis: Option<Iis>,
    /// The six native cost/bound range families, when the basis supports them.
    pub ranging: Option<EnumMap<RangeFamily, Range>>,
    /// Separate explicitly penalized relaxation attempt.
    pub relaxation: Option<Relaxation>,
    /// The fixed-commitment LP and its conditional duals.
    pub fixed_lp: Option<FixedLp>,
    /// Basic variables and requested rows of the basis inverse.
    pub basis_inverse: Option<BasisInverse>,
    /// The presolved model and the postsolved solution of its LP.
    pub presolved: Option<Presolved>,
    /// The root cut pool captured during the solve.
    pub cut_pool: Option<CutPool>,
    /// Unavailable/refused/failed diagnostic reasons; absence never means zero.
    pub unavailable: BTreeMap<String, String>,
}
impl Session {
    /// Ask native diagnostics under the same owner/scheduler gate. A relaxation owns
    /// a separate model and cannot alter this session's coefficients or candidate.
    pub fn diagnose(
        &mut self,
        p: &CoefficientProblem,
        request: &Request,
        execution: &Execution,
    ) -> Result<Report, ProblemError> {
        let n = p.contract.variables.len();
        let m = p.contract.rows.len();
        if let Some(v) = &request.relaxation {
            v.validate(n, m)?;
        }
        let mut report = Report::default();
        if execution.stopped().is_some() {
            report.unavailable.insert(
                "all".into(),
                "attempt cancelled or deadline exhausted".into(),
            );
            return Ok(report);
        }
        let ptr = self.model()?.as_mut_ptr();
        check(
            unsafe {
                ffi::Highs_setDoubleOptionValue(
                    ptr,
                    c"time_limit".as_ptr(),
                    execution
                        .time_limit
                        .saturating_sub(execution.started.elapsed())
                        .as_secs_f64(),
                )
            },
            "remaining diagnostic time",
        )?;
        let _callbacks = CallbackBinding::new(ptr, execution.clone())?;
        let discrete = p
            .domains
            .iter()
            .any(|d| *d != ModelingVariableDomain::Continuous);
        let quadratic = p
            .hessian
            .as_ref()
            .is_some_and(|v| v.val().iter().any(|v| *v != 0.0));
        if request.rays {
            if discrete || quadratic {
                report.unavailable.insert(
                    "rays".into(),
                    "adapter exposes rays only for continuous LP models".into(),
                );
            } else {
                for (primal, len) in [(true, n), (false, m)] {
                    let mut has = 0;
                    let mut ray = vec![0.0; len];
                    let status = unsafe {
                        if primal {
                            ffi::Highs_getPrimalRay(ptr, &mut has, ray.as_mut_ptr())
                        } else {
                            ffi::Highs_getDualRay(ptr, &mut has, ray.as_mut_ptr())
                        }
                    };
                    let key = if primal { "primal_ray" } else { "dual_ray" };
                    if status == 0 && has != 0 && ray.iter().all(|v| v.is_finite()) {
                        if primal {
                            report.primal_ray = Some(ray)
                        } else {
                            report.dual_ray = Some(ray)
                        }
                    } else {
                        report.unavailable.insert(
                            key.into(),
                            format!("native status={status}, available={has}"),
                        );
                    }
                }
            }
        }
        if request.iis && discrete {
            // The diagnostic contract is the continuous relaxation. Give HiGHS an
            // explicitly continuous copy instead of labelling a mixed-integer IIS as one.
            let mut relaxation = p.clone();
            for (variable, domain) in relaxation.contract.variables.iter_mut().zip(&p.domains) {
                if *domain == ModelingVariableDomain::Binary {
                    variable.lower = variable.lower.max(0.);
                    variable.upper = variable.upper.min(1.);
                } else if domain.is_semi() {
                    variable.lower = variable.lower.min(0.);
                    variable.upper = variable.upper.max(0.);
                }
            }
            relaxation.domains.fill(ModelingVariableDomain::Continuous);
            let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::HighsIisRelaxationV1);
            hash.hash(&p.contract.identity).hash(&p.assumptions);
            relaxation.contract.identity = hash.finish_hash();
            let mut model = upload(&relaxation)?;
            let _binding = CallbackBinding::new(model.as_mut_ptr(), execution.clone())?;
            let mut result = Report::default();
            collect_iis(&mut model, &relaxation, execution, &mut result)?;
            if let Some(iis) = &mut result.iis {
                iis.relaxation_only = true;
            }
            report.iis = result.iis;
            report.unavailable.extend(result.unavailable);
        } else if request.iis {
            collect_iis(self.model()?, p, execution, &mut report)?;
        }
        if request.ranging {
            if discrete
                || quadratic
                || unsafe { ffi::Highs_getModelStatus(ptr) } != 7
                || !matches!(info(ptr, "basis_validity")?, Some(Metric::Integer(1)))
            {
                report.unavailable.insert(
                    "ranging".into(),
                    "requires an optimal continuous LP and native valid basis".into(),
                );
            } else {
                let ids: Vec<_> = p.contract.variables.iter().map(|v| v.id).collect();
                let mut ranging = EnumMap::from_fn(|family: RangeFamily| {
                    Range::new(match family.side() {
                        RangeSide::Column => ids.clone(),
                        RangeSide::Row => p.contract.rows.clone(),
                    })
                });
                // The native argument order names each family; the map owns each buffer.
                let (cu, cd, bu, bd, ru, rd) = (
                    RangeFamily::ColumnCostUp,
                    RangeFamily::ColumnCostDown,
                    RangeFamily::ColumnBoundUp,
                    RangeFamily::ColumnBoundDown,
                    RangeFamily::RowBoundUp,
                    RangeFamily::RowBoundDown,
                );
                let status = unsafe {
                    ffi::Highs_getRanging(
                        ptr,
                        ranging[cu].value.as_mut_ptr(),
                        ranging[cu].objective.as_mut_ptr(),
                        ranging[cu].entering.as_mut_ptr(),
                        ranging[cu].leaving.as_mut_ptr(),
                        ranging[cd].value.as_mut_ptr(),
                        ranging[cd].objective.as_mut_ptr(),
                        ranging[cd].entering.as_mut_ptr(),
                        ranging[cd].leaving.as_mut_ptr(),
                        ranging[bu].value.as_mut_ptr(),
                        ranging[bu].objective.as_mut_ptr(),
                        ranging[bu].entering.as_mut_ptr(),
                        ranging[bu].leaving.as_mut_ptr(),
                        ranging[bd].value.as_mut_ptr(),
                        ranging[bd].objective.as_mut_ptr(),
                        ranging[bd].entering.as_mut_ptr(),
                        ranging[bd].leaving.as_mut_ptr(),
                        ranging[ru].value.as_mut_ptr(),
                        ranging[ru].objective.as_mut_ptr(),
                        ranging[ru].entering.as_mut_ptr(),
                        ranging[ru].leaving.as_mut_ptr(),
                        ranging[rd].value.as_mut_ptr(),
                        ranging[rd].objective.as_mut_ptr(),
                        ranging[rd].entering.as_mut_ptr(),
                        ranging[rd].leaving.as_mut_ptr(),
                    )
                };
                if status == 0 {
                    report.ranging = Some(ranging);
                } else {
                    report
                        .unavailable
                        .insert("ranging".into(), format!("native status={status}"));
                }
            }
        }
        if request.fixed_lp {
            match fixed_lp(ptr, p, discrete, execution) {
                Ok(fixed) => report.fixed_lp = Some(fixed),
                Err(reason) => {
                    report.unavailable.insert("fixed_lp".into(), reason);
                }
            }
        }
        if let Some(positions) = &request.basis_inverse {
            if discrete
                || quadratic
                || unsafe { ffi::Highs_getModelStatus(ptr) } != ffi::kHighsModelStatusOptimal
                || !matches!(info(ptr, "basis_validity")?, Some(Metric::Integer(1)))
            {
                report.unavailable.insert(
                    "basis_inverse".into(),
                    "requires an optimal continuous LP and native valid basis".into(),
                );
            } else {
                match basis_inverse(ptr, p, positions) {
                    Ok(view) => report.basis_inverse = Some(view),
                    Err(reason) => {
                        report.unavailable.insert("basis_inverse".into(), reason);
                    }
                }
            }
        }
        if request.presolve {
            match presolve(p, discrete || quadratic, execution)? {
                Ok(view) => report.presolved = Some(view),
                Err(reason) => {
                    report.unavailable.insert("presolve".into(), reason);
                }
            }
        }
        if request.cut_pool {
            match self.cut_pool.take() {
                Some(pool) => report.cut_pool = Some(pool),
                None => {
                    report.unavailable.insert(
                        "cut_pool".into(),
                        "no root cut pool: the model is continuous, the MIP ended before root \
                         cut generation, or the solve did not request it"
                            .into(),
                    );
                }
            }
        }
        if let Some(v) = &request.relaxation {
            if quadratic || execution.stopped().is_some() {
                report.unavailable.insert(
                    "relaxation".into(),
                    "requires LP/MIP and remaining attempt time".into(),
                );
            } else {
                let mut model = upload(p)?;
                model
                    .try_set_option(
                        "time_limit",
                        execution
                            .time_limit
                            .saturating_sub(execution.started.elapsed())
                            .as_secs_f64(),
                    )
                    .map_err(|_| ProblemError::Internal("diagnostic time limit".into()))?;
                let ptr = model.as_mut_ptr();
                // The copied diagnostic model minimizes weighted violations.
                // HiGHS preserves the original objective offset through its elastic
                // operation; carrying that offset would contaminate the penalty.
                check(
                    unsafe { ffi::Highs_changeObjectiveSense(ptr, ffi::kHighsObjSenseMinimize) },
                    "relaxation objective sense",
                )?;
                check(
                    unsafe { ffi::Highs_changeObjectiveOffset(ptr, 0.) },
                    "relaxation objective offset",
                )?;
                let binding = CallbackBinding::new(ptr, execution.clone())?;
                let data =
                    |v: &Option<Vec<f64>>| v.as_ref().map_or(std::ptr::null(), |v| v.as_ptr());
                let status = unsafe {
                    ffi::Highs_feasibilityRelaxation(
                        ptr,
                        v.global[0].into_inner(),
                        v.global[1].into_inner(),
                        v.global[2].into_inner(),
                        data(&v.lower),
                        data(&v.upper),
                        data(&v.rows),
                    )
                };
                drop(binding);
                let restored_status = termination(unsafe { ffi::Highs_getModelStatus(ptr) });
                let penalty = if status == 0 {
                    match info(ptr, "objective_function_value")? {
                        Some(Metric::Real(v)) if v.is_finite() => Some(v),
                        _ => None,
                    }
                } else {
                    None
                };
                let mut x = vec![0.0; n];
                let primal = if matches!(
                    info(ptr, "primal_solution_status")?,
                    Some(Metric::Integer(1 | 2))
                ) && unsafe {
                    ffi::Highs_getSolution(
                        ptr,
                        x.as_mut_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                } == 0
                    && x.iter().all(|v| v.is_finite())
                {
                    Some(x)
                } else {
                    None
                };
                report.relaxation = Some(Relaxation {
                    operation_status: status,
                    restored_status,
                    penalty,
                    primal,
                });
            }
        }
        Ok(report)
    }
    /// Native sparse MIP start. Unspecified coordinates stay unspecified; no zeros
    /// are fabricated. The start is restricted to declared source variable IDs.
    pub fn sparse_start(
        &mut self,
        p: &CoefficientProblem,
        values: &BTreeMap<SemanticId, f64>,
    ) -> Result<(), ProblemError> {
        if !p
            .domains
            .iter()
            .any(|d| *d != ModelingVariableDomain::Continuous)
        {
            return Err(ProblemError::Unsupported(
                "sparse start requires mixed-linear model".into(),
            ));
        }
        let map: BTreeMap<_, _> = p
            .contract
            .variables
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id, i))
            .collect();
        let mut indices = vec![];
        let mut x = vec![];
        for (id, v) in values {
            let i = *map
                .get(id)
                .ok_or_else(|| ProblemError::Contract("sparse start unknown source".into()))?;
            if !v.is_finite() {
                return Err(ProblemError::Contract("nonfinite sparse start".into()));
            }
            indices.push(index(i)?);
            x.push(*v);
        }
        self.pending_sparse = Some(values.clone());
        Ok(())
    }
}

/// A copied native model for diagnostic work: silent, and bounded by the attempt's remaining
/// time.
fn scratch(execution: &Execution) -> Result<highs::Model, ProblemError> {
    let mut model = highs::Model::try_new(highs::ColProblem::new())
        .map_err(|e| ProblemError::memory(format!("HiGHS allocation: {e:?}")))?;
    model
        .try_set_option("output_flag", false)
        .map_err(|_| ProblemError::Internal("diagnostic output option".into()))?;
    model
        .try_set_option(
            "time_limit",
            execution
                .time_limit
                .saturating_sub(execution.started.elapsed())
                .as_secs_f64(),
        )
        .map_err(|_| ProblemError::Internal("diagnostic time limit".into()))?;
    Ok(model)
}
/// Column-wise LP arrays as the C API passes them.
struct Lp {
    sense: i32,
    offset: f64,
    cost: Vec<f64>,
    lower: Vec<f64>,
    upper: Vec<f64>,
    row_lower: Vec<f64>,
    row_upper: Vec<f64>,
    start: Vec<i32>,
    index: Vec<i32>,
    value: Vec<f64>,
}
impl Lp {
    fn new(columns: usize, rows: usize, nonzeros: usize) -> Self {
        Self {
            sense: 1,
            offset: 0.0,
            cost: vec![0.0; columns],
            lower: vec![0.0; columns],
            upper: vec![0.0; columns],
            row_lower: vec![0.0; rows],
            row_upper: vec![0.0; rows],
            start: vec![0; columns + 1],
            index: vec![0; nonzeros],
            value: vec![0.0; nonzeros],
        }
    }
    /// Solve this LP on a scratch model; the solution `(x, column duals, row duals)` when
    /// it is optimal with feasible duals, and the termination either way.
    fn solve(
        &self,
        execution: &Execution,
    ) -> Result<
        (
            NativeTermination,
            Option<(Vec<f64>, Vec<f64>, Vec<f64>)>,
            Option<f64>,
        ),
        ProblemError,
    > {
        let (n, m) = (self.cost.len(), self.row_lower.len());
        let mut model = scratch(execution)?;
        let ptr = model.as_mut_ptr();
        check(
            unsafe {
                ffi::Highs_passLp(
                    ptr,
                    index(n)?,
                    index(m)?,
                    index(self.value.len())?,
                    ffi::kHighsMatrixFormatColwise,
                    self.sense,
                    self.offset,
                    self.cost.as_ptr(),
                    self.lower.as_ptr(),
                    self.upper.as_ptr(),
                    self.row_lower.as_ptr(),
                    self.row_upper.as_ptr(),
                    self.start.as_ptr(),
                    self.index.as_ptr(),
                    self.value.as_ptr(),
                )
            },
            "diagnostic LP upload",
        )?;
        let binding = CallbackBinding::new(ptr, execution.clone())?;
        let run = unsafe { ffi::Highs_run(ptr) };
        drop(binding);
        let status = unsafe { ffi::Highs_getModelStatus(ptr) };
        let solved = run == ffi::STATUS_OK
            && status == ffi::kHighsModelStatusOptimal
            && matches!(
                info(ptr, "dual_solution_status")?,
                Some(Metric::Integer(v)) if v == i64::from(ffi::kHighsSolutionStatusFeasible)
            );
        let objective = match info(ptr, "objective_function_value")? {
            Some(Metric::Real(v)) if solved && v.is_finite() => Some(v),
            _ => None,
        };
        let solution = if solved {
            let (mut x, mut cd, mut rd) = (vec![0.0; n], vec![0.0; n], vec![0.0; m]);
            check(
                unsafe {
                    ffi::Highs_getSolution(
                        ptr,
                        x.as_mut_ptr(),
                        cd.as_mut_ptr(),
                        std::ptr::null_mut(),
                        rd.as_mut_ptr(),
                    )
                },
                "diagnostic LP solution",
            )?;
            x.iter()
                .chain(&cd)
                .chain(&rd)
                .all(|v| v.is_finite())
                .then_some((x, cd, rd))
        } else {
            None
        };
        Ok((termination(status), solution, objective))
    }
}
/// The fixed-commitment LP of the session's MIP solution and its duals. `Err` carries the
/// reason it is unavailable.
fn fixed_lp(
    ptr: *mut c_void,
    p: &CoefficientProblem,
    discrete: bool,
    execution: &Execution,
) -> Result<FixedLp, String> {
    if !discrete {
        return Err("a fixed-commitment LP needs a model with discrete columns".into());
    }
    if !matches!(
        info(ptr, "primal_solution_status").map_err(|e| e.to_string())?,
        Some(Metric::Integer(v)) if v == i64::from(ffi::kHighsSolutionStatusFeasible)
    ) {
        return Err("no feasible MIP solution to commit to".into());
    }
    let (n, m) = (p.contract.variables.len(), p.contract.rows.len());
    let nonzeros = usize::try_from(unsafe { ffi::Highs_getNumNz(ptr) })
        .map_err(|_| "native nonzero count".to_string())?;
    let mut lp = Lp::new(n, m, nonzeros);
    let (mut nc, mut nr, mut nz) = (0, 0, 0);
    let status = unsafe {
        ffi::Highs_getFixedLp(
            ptr,
            ffi::kHighsMatrixFormatColwise,
            &raw mut nc,
            &raw mut nr,
            &raw mut nz,
            &raw mut lp.sense,
            &raw mut lp.offset,
            lp.cost.as_mut_ptr(),
            lp.lower.as_mut_ptr(),
            lp.upper.as_mut_ptr(),
            lp.row_lower.as_mut_ptr(),
            lp.row_upper.as_mut_ptr(),
            lp.start.as_mut_ptr(),
            lp.index.as_mut_ptr(),
            lp.value.as_mut_ptr(),
        )
    };
    // HiGHS warns when a discrete value is not integral and fixes it anyway: such a
    // commitment is not one, so the warning refuses the view.
    if status == ffi::kHighsStatusWarning {
        return Err("the MIP solution is not integral on its discrete columns".into());
    }
    if status != ffi::STATUS_OK || nc as usize != n || nr as usize != m || nz as usize != nonzeros {
        return Err(format!("native status={status}, columns={nc}, rows={nr}"));
    }
    lp.start[n] = nz;
    let commitment = crate::transform::Commitment {
        columns: p
            .contract
            .variables
            .iter()
            .zip(&p.domains)
            .enumerate()
            .filter(|(_, (_, d))| **d != ModelingVariableDomain::Continuous)
            // `Highs_getFixedLp` fixes integer, semi-integer and zero-branch
            // semicontinuous columns; an active semicontinuous column keeps its interval.
            .map(|(j, (v, _))| (v.id, (lp.lower[j], lp.upper[j])))
            .collect(),
    };
    let (termination, solution, objective) = lp.solve(execution).map_err(|e| e.to_string())?;
    let (row_dual, reduced_costs) = match solution {
        Some((_, cd, rd)) => (Some(rd), Some(cd)),
        None => (None, None),
    };
    Ok(FixedLp {
        commitment,
        termination,
        objective,
        row_dual,
        reduced_costs,
    })
}
/// Basic variables and the requested rows of `B⁻¹`.
fn basis_inverse(
    ptr: *mut c_void,
    p: &CoefficientProblem,
    positions: &[usize],
) -> Result<BasisInverse, String> {
    let m = p.contract.rows.len();
    if positions.iter().any(|r| *r >= m)
        || positions
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != positions.len()
    {
        return Err(format!("basis positions must be distinct and below {m}"));
    }
    let mut basic = vec![0; m];
    if unsafe { ffi::Highs_getBasicVariables(ptr, basic.as_mut_ptr()) } != ffi::STATUS_OK {
        return Err("native basic variables".into());
    }
    let basic = basic
        .into_iter()
        .map(|v| {
            if v >= 0 {
                p.contract
                    .variables
                    .get(v as usize)
                    .map(|c| Basic::Column(c.id))
            } else {
                p.contract
                    .rows
                    .get((-v - 1) as usize)
                    .map(|r| Basic::Row(*r))
            }
        })
        .collect::<Option<Vec<_>>>()
        .ok_or("native basic variable index")?;
    let mut rows = vec![];
    for &r in positions {
        let (mut values, mut indices, mut count) = (vec![0.0; m], vec![0; m], 0);
        if unsafe {
            ffi::Highs_getBasisInverseRow(
                ptr,
                r as i32,
                values.as_mut_ptr(),
                &raw mut count,
                indices.as_mut_ptr(),
            )
        } != ffi::STATUS_OK
            || count < 0
            || count as usize > m
        {
            return Err(format!("native basis inverse row {r}"));
        }
        let entries = (0..count as usize)
            .map(|k| (indices[k] as usize, values[indices[k] as usize]))
            .collect();
        rows.push((r, entries));
    }
    Ok(BasisInverse { basic, rows })
}
/// Presolve a copied model and, for a continuous LP, postsolve the presolved LP's solution.
/// The outer `Err` is an adapter failure; the inner one why the view is unavailable.
fn presolve(
    p: &CoefficientProblem,
    no_postsolve: bool,
    execution: &Execution,
) -> Result<Result<Presolved, String>, ProblemError> {
    let mut model = upload(p)?;
    model
        .try_set_option("output_flag", false)
        .map_err(|_| ProblemError::Internal("diagnostic output option".into()))?;
    model
        .try_set_option(
            "time_limit",
            execution
                .time_limit
                .saturating_sub(execution.started.elapsed())
                .as_secs_f64(),
        )
        .map_err(|_| ProblemError::Internal("diagnostic time limit".into()))?;
    let ptr = model.as_mut_ptr();
    let binding = CallbackBinding::new(ptr, execution.clone())?;
    let status = unsafe { ffi::Highs_presolve(ptr) };
    drop(binding);
    if status != ffi::STATUS_OK {
        return Ok(Err(format!("native presolve status={status}")));
    }
    let count =
        |v: i32| usize::try_from(v).map_err(|_| ProblemError::Internal("presolved size".into()));
    let (pc, pr, pz) = unsafe {
        (
            count(ffi::Highs_getPresolvedNumCol(ptr))?,
            count(ffi::Highs_getPresolvedNumRow(ptr))?,
            count(ffi::Highs_getPresolvedNumNz(ptr))?,
        )
    };
    let mut lp = Lp::new(pc, pr, pz);
    let mut integrality = vec![0; pc];
    let (mut nc, mut nr, mut nz) = (0, 0, 0);
    check(
        unsafe {
            ffi::Highs_getPresolvedLp(
                ptr,
                ffi::kHighsMatrixFormatColwise,
                &raw mut nc,
                &raw mut nr,
                &raw mut nz,
                &raw mut lp.sense,
                &raw mut lp.offset,
                lp.cost.as_mut_ptr(),
                lp.lower.as_mut_ptr(),
                lp.upper.as_mut_ptr(),
                lp.row_lower.as_mut_ptr(),
                lp.row_upper.as_mut_ptr(),
                lp.start.as_mut_ptr(),
                lp.index.as_mut_ptr(),
                lp.value.as_mut_ptr(),
                integrality.as_mut_ptr(),
            )
        },
        "presolved LP",
    )?;
    if (nc as usize, nr as usize, nz as usize) != (pc, pr, pz) {
        return Err(ProblemError::Internal("presolved LP dimensions".into()));
    }
    lp.start[pc] = nz;
    let postsolved = if no_postsolve {
        None
    } else {
        // A model presolve reduced to nothing has the empty solution; HiGHS refuses to
        // run an empty LP, so it is postsolved directly.
        let solved = if pc == 0 && pr == 0 {
            Some((vec![], vec![], vec![]))
        } else {
            lp.solve(execution)?.1
        };
        match solved {
            Some((x, cd, rd)) => {
                let n = p.contract.variables.len();
                let mut original = vec![0.0; n];
                (unsafe { ffi::Highs_postsolve(ptr, x.as_ptr(), cd.as_ptr(), rd.as_ptr()) }
                    == ffi::STATUS_OK
                    && unsafe {
                        ffi::Highs_getSolution(
                            ptr,
                            original.as_mut_ptr(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        )
                    } == ffi::STATUS_OK
                    && original.iter().all(|v| v.is_finite()))
                .then_some(original)
            }
            None => None,
        }
    };
    let usize_of = |v: &[i32]| -> Result<Vec<usize>, ProblemError> {
        v.iter()
            .map(|v| {
                usize::try_from(*v).map_err(|_| ProblemError::Internal("presolved index".into()))
            })
            .collect()
    };
    Ok(Ok(Presolved {
        columns: pc,
        rows: pr,
        nonzeros: pz,
        offset: lp.offset,
        column_bounds: lp
            .lower
            .iter()
            .copied()
            .zip(lp.upper.iter().copied())
            .collect(),
        row_bounds: lp
            .row_lower
            .iter()
            .copied()
            .zip(lp.row_upper.iter().copied())
            .collect(),
        start: usize_of(&lp.start)?,
        index: usize_of(&lp.index)?,
        value: lp.value,
        cost: lp.cost,
        postsolved,
    }))
}
// The session already owns the native scheduler gate, including separately uploaded models.
fn collect_iis(
    model: &mut highs::Model,
    p: &CoefficientProblem,
    execution: &Execution,
    report: &mut Report,
) -> Result<(), ProblemError> {
    let ptr = model.as_mut_ptr();
    let n = p.contract.variables.len();
    let m = p.contract.rows.len();
    // The pinned default is only a light bound test. An IIS request needs
    // HiGHS' elastic-LP reduction and its dedicated remaining time allowance.
    check(
        unsafe {
            ffi::Highs_setIntOptionValue(
                ptr,
                c"iis_strategy".as_ptr(),
                ffi::kHighsIisStrategyFromLpRowPriority,
            )
        },
        "irreducible IIS strategy",
    )?;
    check(
        unsafe {
            ffi::Highs_setDoubleOptionValue(
                ptr,
                c"iis_time_limit".as_ptr(),
                execution
                    .time_limit
                    .saturating_sub(execution.started.elapsed())
                    .as_secs_f64(),
            )
        },
        "remaining IIS time",
    )?;
    let (mut nc, mut nr) = (0, 0);
    let mut ci = vec![0; n];
    let mut ri = vec![0; m];
    let mut cb = vec![0; n];
    let mut rb = vec![0; m];
    let mut cs = vec![0; n];
    let mut rs = vec![0; m];
    let status = unsafe {
        ffi::Highs_getIis(
            ptr,
            &mut nc,
            &mut nr,
            ci.as_mut_ptr(),
            ri.as_mut_ptr(),
            cb.as_mut_ptr(),
            rb.as_mut_ptr(),
            cs.as_mut_ptr(),
            rs.as_mut_ptr(),
        )
    };
    if status == 0
        && nc >= 0
        && nr >= 0
        && nc as usize <= n
        && nr as usize <= m
        && (nc > 0 || nr > 0)
    {
        let columns = ci[..nc as usize]
            .iter()
            .zip(&cb)
            .map(|(&i, &b)| {
                p.contract
                    .variables
                    .get(i as usize)
                    .map(|v| (v.id, b))
                    .ok_or_else(|| ProblemError::Internal("native IIS column index".into()))
            })
            .collect::<Result<_, _>>()?;
        let rows = ri[..nr as usize]
            .iter()
            .zip(&rb)
            .map(|(&i, &b)| {
                p.contract
                    .rows
                    .get(i as usize)
                    .map(|v| (*v, b))
                    .ok_or_else(|| ProblemError::Internal("native IIS row index".into()))
            })
            .collect::<Result<_, _>>()?;
        report.iis = Some(Iis {
            columns,
            rows,
            column_status: cs,
            row_status: rs,
            relaxation_only: false,
        });
    } else {
        report.unavailable.insert(
            "iis".into(),
            format!("native status={status}, columns={nc}, rows={nr}"),
        );
    }
    Ok(())
}
