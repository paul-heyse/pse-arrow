// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit cones, bounded data reuse and source-space postprocessing. The boundary
//! vocabulary ([`SparseMatrix`], [`Cone`], [`Settings`]) is pse-owned: it is the Python wire
//! format and the request identity, and it maps to Clarabel only inside this adapter, so a
//! Clarabel upgrade cannot change a Python contract or an identity (F09).
use crate::{
    ConicProblem, ProblemError,
    quality::{Quality, Tolerances, Violation, interval},
    solve::{
        Assurance, Backend, Candidate, CertificateAccuracy, CertificateKind, Compatibility,
        ConicEvidence, Controls, Event, Execution, InfeasibilityCertificate, Metric,
        NativeTermination, RayCoordinate, RayEntry, ResolvedAccuracy, SolveReport, Termination,
    },
};
use clarabel::solver::traits::Settings as _;
use clarabel::{
    algebra::CscMatrix,
    solver::{DefaultInfo, DefaultSettings, DefaultSolver, IPSolver, SolverStatus, SupportedConeT},
};
use std::collections::BTreeMap;

pub(crate) mod lowering;
pub(crate) mod recognized;
pub use lowering::{Lowered, LoweredRow, RowSide};
pub use recognized::{Recognized, lower};

/// Compressed-sparse-column matrix of the conic boundary.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SparseMatrix {
    /// Row count.
    pub rows: usize,
    /// Column count.
    pub columns: usize,
    /// Offsets of each column's entries: `columns + 1` of them, from zero.
    pub column_starts: Vec<usize>,
    /// Row of each entry, strictly increasing within a column.
    pub row_indices: Vec<usize>,
    /// Value of each entry.
    pub values: Vec<f64>,
}
impl SparseMatrix {
    /// A matrix from complete CSC storage; [`Self::validate`] checks it.
    pub fn new(
        rows: usize,
        columns: usize,
        column_starts: Vec<usize>,
        row_indices: Vec<usize>,
        values: Vec<f64>,
    ) -> Self {
        Self {
            rows,
            columns,
            column_starts,
            row_indices,
            values,
        }
    }
    /// A `rows` by `columns` matrix without entries.
    pub fn zeros(rows: usize, columns: usize) -> Self {
        Self::new(rows, columns, vec![0; columns + 1], vec![], vec![])
    }
    /// The `n` by `n` identity.
    pub fn identity(n: usize) -> Self {
        Self::new(n, n, (0..=n).collect(), (0..n).collect(), vec![1.0; n])
    }
    /// Validate the CSC storage: `columns + 1` nondecreasing offsets from zero to the entry
    /// count, one row per value, and rows in range and strictly increasing per column.
    ///
    /// # Errors
    /// Malformed storage.
    pub fn validate(&self) -> Result<(), ProblemError> {
        let invalid = |what: &str| Err(ProblemError::Contract(format!("sparse matrix {what}")));
        if self.column_starts.len() != self.columns.saturating_add(1)
            || self.column_starts.first() != Some(&0)
            || self.column_starts.last() != Some(&self.row_indices.len())
            || self.row_indices.len() != self.values.len()
        {
            return invalid("dimensions");
        }
        if self.column_starts.windows(2).any(|w| w[0] > w[1]) {
            return invalid("column offsets");
        }
        for c in 0..self.columns {
            let rows = &self.row_indices[self.column(c)];
            if rows.windows(2).any(|w| w[0] >= w[1]) || rows.iter().any(|r| *r >= self.rows) {
                return invalid("row indices");
            }
        }
        Ok(())
    }
    /// The entry range of column `c` in validated storage.
    pub fn column(&self, c: usize) -> std::ops::Range<usize> {
        self.column_starts[c]..self.column_starts[c + 1]
    }
    pub(crate) fn to_clarabel(&self) -> CscMatrix<f64> {
        CscMatrix::new(
            self.rows,
            self.columns,
            self.column_starts.clone(),
            self.row_indices.clone(),
            self.values.clone(),
        )
    }
}

/// One explicit cone block of the conic boundary.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Cone {
    /// `s = 0`.
    Zero {
        /// Rows in the block.
        dimension: usize,
    },
    /// `s >= 0`.
    Nonnegative {
        /// Rows in the block.
        dimension: usize,
    },
    /// `s_0 >= ||(s_1, ...)||`.
    SecondOrder {
        /// Rows in the block, including the leading one.
        dimension: usize,
    },
    /// The closed exponential cone over three rows.
    Exponential,
    /// The three-row power cone with exponent `alpha` in (0, 1).
    Power {
        /// Exponent.
        alpha: f64,
    },
    /// The generalized power cone: `alpha.len()` scaled rows, then `dimension` rows under
    /// the norm.
    GeneralizedPower {
        /// Positive exponents summing to one.
        alpha: Vec<f64>,
        /// Rows under the norm.
        dimension: usize,
    },
    /// Symmetric positive semidefinite matrices of `order`, as the scaled upper triangle.
    /// A build without the SDP profile refuses it.
    PsdTriangle {
        /// Matrix order.
        order: usize,
    },
}
impl Cone {
    /// Rows occupied by this block, saturating; [`ConicProblem::validate`] checks overflow.
    pub fn dim(&self) -> usize {
        match self {
            Self::Zero { dimension }
            | Self::Nonnegative { dimension }
            | Self::SecondOrder { dimension } => *dimension,
            Self::Exponential | Self::Power { .. } => 3,
            Self::GeneralizedPower { alpha, dimension } => alpha.len().saturating_add(*dimension),
            Self::PsdTriangle { order } => order.saturating_mul(order.saturating_add(1)) / 2,
        }
    }
    /// The Clarabel cone of this block.
    pub(crate) fn to_clarabel(&self) -> Result<SupportedConeT<f64>, ProblemError> {
        Ok(match self {
            Self::Zero { dimension } => SupportedConeT::ZeroConeT(*dimension),
            Self::Nonnegative { dimension } => SupportedConeT::NonnegativeConeT(*dimension),
            Self::SecondOrder { dimension } => SupportedConeT::SecondOrderConeT(*dimension),
            Self::Exponential => SupportedConeT::ExponentialConeT(),
            Self::Power { alpha } => SupportedConeT::PowerConeT(*alpha),
            Self::GeneralizedPower { alpha, dimension } => {
                SupportedConeT::GenPowerConeT(alpha.clone(), *dimension)
            }
            #[cfg(feature = "sdp")]
            Self::PsdTriangle { order } => SupportedConeT::PSDTriangleConeT(*order),
            #[cfg(not(feature = "sdp"))]
            Self::PsdTriangle { .. } => {
                return Err(ProblemError::Unsupported(
                    "PSD cones need the Clarabel SDP profile, which this build does not link"
                        .into(),
                ));
            }
        })
    }
}
fn clarabel_cones(cones: &[Cone]) -> Result<Vec<SupportedConeT<f64>>, ProblemError> {
    cones.iter().map(Cone::to_clarabel).collect()
}
/// Layout identity of a cone sequence: every kind, dimension and exponent bit of the pse
/// encoding, never Clarabel's.
///
/// # Errors
/// The identity serializer refused a value.
pub fn cone_key(cones: &[Cone]) -> Result<pse_ids::ContentHash, ProblemError> {
    crate::identity::of(pse_ids::Frame::ConeLayoutV3, cones)
}
/// The KKT direct solver: Clarabel's serial QDLDL, or oneMKL Pardiso from the process's one
/// linked oneMKL (ADR-0108), which admits more than one thread (blueprint §18.8). A
/// registry vocabulary (ADR-0115 Outcome 3).
pub use pse_model::generated::enums::ClarabelDirect as Direct;
/// Native preprocessing and mutable-data reuse are distinct execution profiles
/// (`ReusableData` disables native presolve, input zero-dropping and chordal decomposition
/// whatever [`Settings`] request), and the clique merging of the chordal decomposition:
/// registry vocabularies (ADR-0115 Outcome 3).
pub use pse_model::generated::enums::{ClarabelMergeMethod as MergeMethod, ClarabelMode as Mode};
/// Clarabel's native `chordal_decomposition_merge_method` value.
#[cfg_attr(
    not(feature = "sdp"),
    expect(dead_code, reason = "the native merge setting exists only with SDP")
)]
const fn merge_method(method: MergeMethod) -> &'static str {
    match method {
        MergeMethod::None => "none",
        MergeMethod::ParentChild => "parent_child",
        MergeMethod::CliqueGraph => "clique_graph",
    }
}
/// The chordal-decomposition defaults: Clarabel's `sdp` builder defaults. They are stated
/// once because a build without SDP has no native fields to read them from; the
/// `clarabel_boundary_types_are_pse_owned` test checks them against the library.
const CHORDAL_DEFAULTS: (bool, MergeMethod, bool, bool) =
    (true, MergeMethod::CliqueGraph, true, true);
/// The Clarabel adapter's settings type: the mode, the KKT direct solver and every admitted
/// native control. Iteration and time budgets, stopping tolerances, equilibration and the
/// thread count are owned by the shared controls and the resolved accuracy, so they are not
/// fields. Native defaults are the pinned library's. Identity derives from serde.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "ClarabelSettings")]
pub struct Settings {
    /// Preprocessing or data-update mode.
    pub mode: Mode,
    /// KKT direct solver; only MKL Pardiso admits more than one thread.
    pub direct: Direct,
    /// Maximum interior step fraction.
    pub max_step_fraction: f64,
    /// Absolute infeasibility tolerance.
    pub tol_infeas_abs: f64,
    /// Relative infeasibility tolerance.
    pub tol_infeas_rel: f64,
    /// KKT ratio tolerance.
    pub tol_ktratio: f64,
    /// Reduced absolute infeasibility tolerance.
    pub reduced_tol_infeas_abs: f64,
    /// Reduced relative infeasibility tolerance.
    pub reduced_tol_infeas_rel: f64,
    /// Reduced KKT ratio tolerance.
    pub reduced_tol_ktratio: f64,
    /// Equilibration iterations.
    pub equilibrate_max_iter: u32,
    /// Minimum equilibration scaling.
    pub equilibrate_min_scaling: f64,
    /// Maximum equilibration scaling.
    pub equilibrate_max_scaling: f64,
    /// Line-search backtracking factor.
    pub linesearch_backtrack_step: f64,
    /// Minimum step length before switching to symmetric scaling.
    pub min_switch_step_length: f64,
    /// Minimum step length before termination.
    pub min_terminate_step_length: f64,
    /// Static KKT regularization.
    pub static_regularization_enable: bool,
    /// Constant static regularization.
    pub static_regularization_constant: f64,
    /// Proportional static regularization.
    pub static_regularization_proportional: f64,
    /// Dynamic KKT regularization.
    pub dynamic_regularization_enable: bool,
    /// Dynamic regularization threshold.
    pub dynamic_regularization_eps: f64,
    /// Dynamic regularization shift.
    pub dynamic_regularization_delta: f64,
    /// KKT iterative refinement.
    pub iterative_refinement_enable: bool,
    /// Refinement relative tolerance.
    pub iterative_refinement_reltol: f64,
    /// Refinement absolute tolerance.
    pub iterative_refinement_abstol: f64,
    /// Refinement iterations.
    pub iterative_refinement_max_iter: u32,
    /// Refinement stall ratio.
    pub iterative_refinement_stop_ratio: f64,
    /// Native presolve; `ReusableData` disables it.
    pub presolve_enable: bool,
    /// Drop explicit input zeros; `ReusableData` disables it.
    pub input_sparse_dropzeros: bool,
    /// Chordal decomposition of PSD cones; `ReusableData` disables it. Without the SDP
    /// profile only the chordal defaults are admitted.
    pub chordal_decomposition_enable: bool,
    /// Clique merging.
    pub chordal_decomposition_merge_method: MergeMethod,
    /// Compact the decomposed problem.
    pub chordal_decomposition_compact: bool,
    /// Complete the dual of decomposed PSD cones.
    pub chordal_decomposition_complete_dual: bool,
}
impl Default for Settings {
    fn default() -> Self {
        let d = DefaultSettings::<f64>::default();
        let (enable, merge, compact, complete) = CHORDAL_DEFAULTS;
        Self {
            mode: Mode::SingleSolve,
            direct: Direct::Qdldl,
            max_step_fraction: d.max_step_fraction,
            tol_infeas_abs: d.tol_infeas_abs,
            tol_infeas_rel: d.tol_infeas_rel,
            tol_ktratio: d.tol_ktratio,
            reduced_tol_infeas_abs: d.reduced_tol_infeas_abs,
            reduced_tol_infeas_rel: d.reduced_tol_infeas_rel,
            reduced_tol_ktratio: d.reduced_tol_ktratio,
            equilibrate_max_iter: d.equilibrate_max_iter,
            equilibrate_min_scaling: d.equilibrate_min_scaling,
            equilibrate_max_scaling: d.equilibrate_max_scaling,
            linesearch_backtrack_step: d.linesearch_backtrack_step,
            min_switch_step_length: d.min_switch_step_length,
            min_terminate_step_length: d.min_terminate_step_length,
            static_regularization_enable: d.static_regularization_enable,
            static_regularization_constant: d.static_regularization_constant,
            static_regularization_proportional: d.static_regularization_proportional,
            dynamic_regularization_enable: d.dynamic_regularization_enable,
            dynamic_regularization_eps: d.dynamic_regularization_eps,
            dynamic_regularization_delta: d.dynamic_regularization_delta,
            iterative_refinement_enable: d.iterative_refinement_enable,
            iterative_refinement_reltol: d.iterative_refinement_reltol,
            iterative_refinement_abstol: d.iterative_refinement_abstol,
            iterative_refinement_max_iter: d.iterative_refinement_max_iter,
            iterative_refinement_stop_ratio: d.iterative_refinement_stop_ratio,
            presolve_enable: d.presolve_enable,
            input_sparse_dropzeros: d.input_sparse_dropzeros,
            chordal_decomposition_enable: enable,
            chordal_decomposition_merge_method: merge,
            chordal_decomposition_compact: compact,
            chordal_decomposition_complete_dual: complete,
        }
    }
}
/// A native Clarabel model owned by one admitted worker.
pub struct Session {
    solver: DefaultSolver<f64>,
    compatibility: Compatibility,
    mode: Mode,
    rows: usize,
    bounds: Vec<(usize, bool)>,
    signature: Vec<SupportedConeT<f64>>,
    a_pattern: (Vec<usize>, Vec<usize>),
    p_pattern: (Vec<usize>, Vec<usize>),
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarabelSession")
            .field("compatibility", &self.compatibility)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}
struct Data {
    a: CscMatrix<f64>,
    rhs: Vec<f64>,
    cones: Vec<SupportedConeT<f64>>,
    bounds: Vec<(usize, bool)>,
}
/// The finite variable bounds of a cone form, as rows appended after the cone rows: in
/// variable order, lower (`-x + s = -l`) before upper (`x + s = u`), each `(variable, lower)`.
/// Every cone form, its certificates and their verification share this one layout.
pub(crate) fn bound_rows(variables: &[crate::Variable]) -> Vec<(usize, bool)> {
    let mut bounds = Vec::new();
    for (i, v) in variables.iter().enumerate() {
        if v.lower.is_finite() {
            bounds.push((i, true));
        }
        if v.upper.is_finite() {
            bounds.push((i, false));
        }
    }
    bounds
}
/// Clarabel's native data: pse cones and matrices mapped, finite bounds appended as rows.
fn data(p: &ConicProblem) -> Result<Data, ProblemError> {
    let bounds = bound_rows(&p.contract.variables);
    let mut rhs = p.rhs.clone();
    let mut cones = clarabel_cones(&p.cones)?;
    for &(i, lower) in &bounds {
        let v = &p.contract.variables[i];
        rhs.push(if lower { -v.lower } else { v.upper });
    }
    if !bounds.is_empty() {
        cones.push(SupportedConeT::NonnegativeConeT(bounds.len()));
    }
    let mut colptr = vec![0];
    let mut rowval = Vec::new();
    let mut nzval = Vec::new();
    let constraints = &p.constraints;
    // The bound rows are in variable order, so one cursor appends each column's own.
    let mut next = 0;
    for c in 0..constraints.columns {
        for k in constraints.column(c) {
            rowval.push(constraints.row_indices[k]);
            nzval.push(constraints.values[k]);
        }
        while let Some(&(v, lower)) = bounds.get(next)
            && v == c
        {
            rowval.push(constraints.rows + next);
            nzval.push(if lower { -1.0 } else { 1.0 });
            next += 1;
        }
        colptr.push(rowval.len());
    }
    Ok(Data {
        a: CscMatrix::new(rhs.len(), constraints.columns, colptr, rowval, nzval),
        rhs,
        cones,
        bounds,
    })
}
/// Thread admission of a direct solver: QDLDL factorizes on one thread; MKL Pardiso admits
/// the worker's native threads (blueprint §18.8) when this build links it.
///
/// # Errors
/// More than one thread for QDLDL, or MKL Pardiso in a build without it.
pub(crate) fn admit_threads(direct: Direct, threads: usize) -> Result<(), ProblemError> {
    match direct {
        Direct::Qdldl if threads != 1 => Err(ProblemError::Unsupported(
            "Clarabel's QDLDL factorizes on one thread; more threads need the MKL Pardiso KKT solver"
                .into(),
        )),
        Direct::MklPardiso if !cfg!(feature = "clarabel-pardiso") => {
            Err(ProblemError::Unsupported(
                "the MKL Pardiso KKT solver needs the clarabel-pardiso profile, which this build does not link"
                    .into(),
            ))
        }
        _ => Ok(()),
    }
}
/// Map the pse settings, the shared controls and the resolved accuracy to Clarabel's
/// complete native settings. Only this adapter sees `DefaultSettings`.
fn settings(
    pse: &Settings,
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    mode: Mode,
) -> Result<DefaultSettings<f64>, ProblemError> {
    controls.validate()?;
    // The typed settings are complete; there is no unvalidated string option channel.
    if !controls.options.is_empty() {
        return Err(ProblemError::Unsupported(
            "Clarabel uses typed settings instead of native option strings".into(),
        ));
    }
    admit_threads(pse.direct, controls.threads)?;
    let mut settings = DefaultSettings::<f64> {
        max_step_fraction: pse.max_step_fraction,
        tol_infeas_abs: pse.tol_infeas_abs,
        tol_infeas_rel: pse.tol_infeas_rel,
        tol_ktratio: pse.tol_ktratio,
        reduced_tol_infeas_abs: pse.reduced_tol_infeas_abs,
        reduced_tol_infeas_rel: pse.reduced_tol_infeas_rel,
        reduced_tol_ktratio: pse.reduced_tol_ktratio,
        equilibrate_max_iter: pse.equilibrate_max_iter,
        equilibrate_min_scaling: pse.equilibrate_min_scaling,
        equilibrate_max_scaling: pse.equilibrate_max_scaling,
        linesearch_backtrack_step: pse.linesearch_backtrack_step,
        min_switch_step_length: pse.min_switch_step_length,
        min_terminate_step_length: pse.min_terminate_step_length,
        static_regularization_enable: pse.static_regularization_enable,
        static_regularization_constant: pse.static_regularization_constant,
        static_regularization_proportional: pse.static_regularization_proportional,
        dynamic_regularization_enable: pse.dynamic_regularization_enable,
        dynamic_regularization_eps: pse.dynamic_regularization_eps,
        dynamic_regularization_delta: pse.dynamic_regularization_delta,
        iterative_refinement_enable: pse.iterative_refinement_enable,
        iterative_refinement_reltol: pse.iterative_refinement_reltol,
        iterative_refinement_abstol: pse.iterative_refinement_abstol,
        iterative_refinement_max_iter: pse.iterative_refinement_max_iter,
        iterative_refinement_stop_ratio: pse.iterative_refinement_stop_ratio,
        presolve_enable: pse.presolve_enable,
        input_sparse_dropzeros: pse.input_sparse_dropzeros,
        ..DefaultSettings::default()
    };
    #[cfg(feature = "sdp")]
    {
        settings.chordal_decomposition_enable = pse.chordal_decomposition_enable;
        settings.chordal_decomposition_merge_method =
            merge_method(pse.chordal_decomposition_merge_method).into();
        settings.chordal_decomposition_compact = pse.chordal_decomposition_compact;
        settings.chordal_decomposition_complete_dual = pse.chordal_decomposition_complete_dual;
    }
    #[cfg(not(feature = "sdp"))]
    if (
        pse.chordal_decomposition_enable,
        pse.chordal_decomposition_merge_method,
        pse.chordal_decomposition_compact,
        pse.chordal_decomposition_complete_dual,
    ) != CHORDAL_DEFAULTS
    {
        return Err(ProblemError::Unsupported(
            "chordal decomposition settings need the Clarabel SDP profile".into(),
        ));
    }
    settings.max_iter = controls.iterations;
    settings.time_limit = controls.time_limit.as_secs_f64();
    match pse.direct {
        Direct::Qdldl => {
            settings.direct_solve_method = "qdldl".into();
            settings.max_threads = 1;
        }
        Direct::MklPardiso => {
            // pardiso-wrapper resolves the process's one linked oneMKL before Clarabel
            // validates the method. Zero leaves Pardiso's thread count to the owning
            // worker's oneMKL-local setting (`mkl::Threads`) instead of the process-wide
            // Pardiso domain Clarabel would set.
            #[cfg(feature = "clarabel-pardiso")]
            crate::mkl::pardiso()?;
            settings.direct_solve_method = "mkl".into();
            settings.max_threads = 0;
        }
    }
    settings.tol_gap_abs = accuracy.gap_absolute;
    settings.tol_gap_rel = accuracy.gap_relative;
    settings.tol_feas = accuracy.feasibility;
    settings.reduced_tol_gap_abs = accuracy
        .acceptable
        .map_or(settings.tol_gap_abs, |k| k.complementarity);
    settings.reduced_tol_gap_rel = accuracy
        .acceptable
        .map_or(settings.tol_gap_rel, |k| k.complementarity);
    settings.reduced_tol_feas = settings.tol_feas;
    settings.equilibrate_enable = accuracy.native_scaling;
    settings.verbose = false;
    if mode == Mode::ReusableData {
        settings.presolve_enable = false;
        settings.input_sparse_dropzeros = false;
        #[cfg(feature = "sdp")]
        {
            settings.chordal_decomposition_enable = false;
        }
    }
    settings
        .validate()
        .map_err(|e| ProblemError::Contract(format!("Clarabel settings: {e}")))?;
    Ok(settings)
}
impl Session {
    /// Validate explicit cones and current convexity evidence before constructing native data.
    pub fn new(
        p: &ConicProblem,
        certificate: &dyn pse_math::convexity::QuadraticEvidence,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        pse: &Settings,
        compatibility: Compatibility,
    ) -> Result<Self, ProblemError> {
        p.validate(certificate)?;
        let settings = settings(pse, controls, accuracy, pse.mode)?;
        let d = data(p)?;
        let quadratic = p.quadratic.to_clarabel();
        let solver = DefaultSolver::new(&quadratic, &p.objective, &d.a, &d.rhs, &d.cones, settings)
            .map_err(|e| ProblemError::Contract(format!("Clarabel problem: {e}")))?;
        Ok(Self {
            solver,
            compatibility,
            mode: pse.mode,
            rows: p.rhs.len(),
            bounds: d.bounds,
            signature: d.cones,
            a_pattern: (d.a.colptr, d.a.rowval),
            p_pattern: (quadratic.colptr, quadratic.rowval),
        })
    }
    /// Native update restrictions, including the session's fixed mode, are checked before
    /// updating any part of the model.
    pub fn update(
        &mut self,
        p: &ConicProblem,
        certificate: &dyn pse_math::convexity::QuadraticEvidence,
        pse: &Settings,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        p.validate(certificate)?;
        let d = data(p)?;
        let quadratic = p.quadratic.to_clarabel();
        if self.mode != Mode::ReusableData
            || pse.mode != self.mode
            || !self.solver.is_data_update_allowed()
            || !self.compatibility.same_session(&compatibility)
            || compatibility.backend != Backend::Clarabel
            || self.signature != d.cones
            || self.bounds != d.bounds
            || self.a_pattern != (d.a.colptr.clone(), d.a.rowval.clone())
            || self.p_pattern != (quadratic.colptr.clone(), quadratic.rowval.clone())
        {
            return Err(ProblemError::Unsupported(
                "Clarabel update changes layout or requires disabled preprocessing".into(),
            ));
        }
        self.solver
            .update_data(&quadratic, &p.objective, &d.a, &d.rhs)
            .map_err(|e| ProblemError::Contract(format!("Clarabel update: {e}")))?;
        self.compatibility = compatibility;
        Ok(())
    }
    /// Solve using native preprocessing/postprocessing. No external iterate warm start is claimed.
    pub fn solve(
        &mut self,
        p: &ConicProblem,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        pse: &Settings,
        execution: Execution,
        tolerances: &Tolerances,
    ) -> Result<SolveReport, ProblemError> {
        tolerances.validate(p.contract.variables.len(), p.contract.rows.len())?;
        let settings = settings(pse, controls, accuracy, self.mode)?;
        let settings_json =
            serde_json::to_string(&settings).map_err(|e| ProblemError::Internal(e.to_string()))?;
        self.solver
            .update_settings(settings)
            .map_err(|e| ProblemError::Contract(format!("Clarabel settings: {e}")))?;
        let callback = execution.clone();
        self.solver
            .set_termination_callback(move |info: &DefaultInfo<f64>| {
                callback.progress.push(Event {
                    phase: "clarabel.iteration".into(),
                    elapsed: callback.started.elapsed(),
                    values: metrics(info),
                    incumbent: None,
                });
                callback.stopped().is_some()
            });
        if let Some(stop) = execution.stopped() {
            let mut report = SolveReport::new(
                Backend::Clarabel,
                &p.contract,
                termination(SolverStatus::Unsolved),
                &execution,
            );
            report.termination.category = stop;
            return Ok(report);
        }
        if execution.stopped().is_none() {
            self.solver.solve();
        }
        let solution = &self.solver.solution;
        let mut report = SolveReport::new(
            Backend::Clarabel,
            &p.contract,
            termination(solution.status),
            &execution,
        );
        report.metrics = metrics(&self.solver.info);
        // Nonfinite native residuals compare false against every budget.
        let info = &self.solver.info;
        report.evidence.conic = Some(ConicEvidence {
            primal_residual: info.res_primal,
            dual_residual: info.res_dual,
            gap_absolute: info.gap_abs,
            gap_relative: info.gap_rel,
        });
        report.provenance.insert(
            "native".into(),
            format!(
                "Clarabel 0.11.1; {}; {}",
                match pse.direct {
                    Direct::Qdldl => "QDLDL",
                    Direct::MklPardiso => "MKL Pardiso (the linked oneMKL, LP64, GNU threading)",
                },
                if cfg!(feature = "sdp") {
                    "SDP BLAS/LAPACK from the linked oneMKL (LP64, GNU threading)"
                } else {
                    "SDP unavailable"
                }
            ),
        );
        report
            .provenance
            .insert("settings.effective".into(), settings_json);
        report.provenance.insert(
            "warm_start".into(),
            "no external iterate seed API; data/allocation reuse only".into(),
        );
        if let Some(stop) = execution.stopped() {
            report.termination.category = stop;
            report.termination.assurance = Assurance::None;
        }
        if let Some(certificate) = self.certificate(p, solution.status, &solution.x, &solution.z) {
            report.certificate = Some(certificate);
        } else if solution.status != SolverStatus::Unsolved
            && solution.x.iter().all(|v| v.is_finite())
            && solution.obj_val.is_finite()
        {
            let mut lower = vec![0.0; solution.x.len()];
            let mut upper = lower.clone();
            for (k, &(i, l)) in self.bounds.iter().enumerate() {
                if l {
                    lower[i] = solution.z[self.rows + k];
                } else {
                    upper[i] = solution.z[self.rows + k];
                }
            }
            report.candidate = Some(Candidate {
                kind: crate::solve::CandidateKind::FinalIterate,
                primal: solution.x.clone(),
                objective: Some(solution.obj_val + p.objective_constant),
                row_dual: Some(solution.z[..self.rows].to_vec()),
                bound_dual: Some((lower, upper)),
                reduced_costs: None,
                slacks: Some(solution.s[..self.rows].to_vec()),
                commitment: None,
            });
            match crate::quality::contained(|| quality(p, &solution.x, tolerances)) {
                Ok(q) => {
                    if !q.feasible() {
                        report.termination.assurance = Assurance::None;
                    }
                    report.quality = Some(q);
                }
                Err(e) => {
                    report.record_validation_failure(e);
                    report.termination.assurance = Assurance::None;
                }
            }
        } else {
            report.termination.assurance = Assurance::None;
        }
        Ok(report)
    }
}
impl Session {
    /// The native ray of an infeasibility status over this session's cone form: a Farkas
    /// ray over the cone rows then the appended bound rows, or a recession direction over
    /// the variables. An almost status keeps its reduced accuracy.
    fn certificate(
        &self,
        p: &ConicProblem,
        status: SolverStatus,
        x: &[f64],
        z: &[f64],
    ) -> Option<InfeasibilityCertificate> {
        let (kind, accuracy) = match status {
            SolverStatus::PrimalInfeasible => {
                (CertificateKind::PrimalInfeasible, CertificateAccuracy::Full)
            }
            SolverStatus::AlmostPrimalInfeasible => (
                CertificateKind::PrimalInfeasible,
                CertificateAccuracy::Reduced,
            ),
            SolverStatus::DualInfeasible => {
                (CertificateKind::DualInfeasible, CertificateAccuracy::Full)
            }
            SolverStatus::AlmostDualInfeasible => (
                CertificateKind::DualInfeasible,
                CertificateAccuracy::Reduced,
            ),
            _ => return None,
        };
        let variables = &p.contract.variables;
        let ray = match kind {
            CertificateKind::PrimalInfeasible => p
                .contract
                .rows
                .iter()
                .map(|id| (RayCoordinate::Row, *id))
                .chain(self.bounds.iter().map(|&(i, lower)| {
                    let coordinate = if lower {
                        RayCoordinate::VariableLower
                    } else {
                        RayCoordinate::VariableUpper
                    };
                    (coordinate, variables[i].id)
                }))
                .zip(z)
                .map(|((coordinate, id), value)| RayEntry {
                    coordinate,
                    id,
                    value: *value,
                })
                .collect(),
            CertificateKind::DualInfeasible => variables
                .iter()
                .zip(x)
                .map(|(v, value)| RayEntry {
                    coordinate: RayCoordinate::Variable,
                    id: v.id,
                    value: *value,
                })
                .collect(),
        };
        Some(InfeasibilityCertificate {
            kind,
            accuracy,
            ray,
            verification: None,
        })
    }
}
fn metrics(info: &DefaultInfo<f64>) -> BTreeMap<String, Metric> {
    let mut values = BTreeMap::new();
    macro_rules! real{($($field:ident),*)=>{$(values.insert(stringify!($field).into(),Metric::Real(info.$field));)*}}
    real!(
        mu,
        sigma,
        step_length,
        cost_primal,
        cost_dual,
        res_primal,
        res_dual,
        res_primal_inf,
        res_dual_inf,
        gap_abs,
        gap_rel,
        ktratio,
        solve_time
    );
    values.insert(
        "iterations".into(),
        Metric::Integer(i64::from(info.iterations)),
    );
    values.insert(
        "linear.name".into(),
        Metric::Text(info.linsolver.name.clone()),
    );
    values.insert(
        "linear.threads".into(),
        Metric::Integer(info.linsolver.threads as i64),
    );
    values.insert("linear.direct".into(), Metric::Bool(info.linsolver.direct));
    values.insert(
        "linear.nnzA".into(),
        Metric::Integer(info.linsolver.nnzA as i64),
    );
    values.insert(
        "linear.nnzL".into(),
        Metric::Integer(info.linsolver.nnzL as i64),
    );
    values
}
/// Clarabel's symbolic name of `status`, stated here rather than taken from Rust `Debug`
/// output, so a library refactor cannot silently change a published name (F30).
pub const fn status_name(status: SolverStatus) -> &'static str {
    match status {
        SolverStatus::Unsolved => "Unsolved",
        SolverStatus::Solved => "Solved",
        SolverStatus::PrimalInfeasible => "PrimalInfeasible",
        SolverStatus::DualInfeasible => "DualInfeasible",
        SolverStatus::AlmostSolved => "AlmostSolved",
        SolverStatus::AlmostPrimalInfeasible => "AlmostPrimalInfeasible",
        SolverStatus::AlmostDualInfeasible => "AlmostDualInfeasible",
        SolverStatus::MaxIterations => "MaxIterations",
        SolverStatus::MaxTime => "MaxTime",
        SolverStatus::NumericalError => "NumericalError",
        SolverStatus::InsufficientProgress => "InsufficientProgress",
        SolverStatus::CallbackTerminated => "CallbackTerminated",
    }
}
/// Native conic statuses retain certificate versus candidate distinctions. A native
/// infeasibility status claims no assurance: the certificate assurance follows only from
/// the ray's verification in original coordinates (`quality::qualify`).
pub fn termination(status: SolverStatus) -> NativeTermination {
    let category = match status {
        SolverStatus::Solved => Termination::Success,
        SolverStatus::AlmostSolved => Termination::Acceptable,
        SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible => {
            Termination::Infeasible
        }
        SolverStatus::DualInfeasible | SolverStatus::AlmostDualInfeasible => Termination::Unbounded,
        SolverStatus::MaxIterations => Termination::IterationLimit,
        SolverStatus::MaxTime => Termination::TimeLimit,
        SolverStatus::CallbackTerminated => Termination::Cancelled,
        SolverStatus::NumericalError | SolverStatus::InsufficientProgress => Termination::Numerical,
        SolverStatus::Unsolved => Termination::Invalid,
    };
    NativeTermination {
        code: status as i64,
        name: status_name(status).into(),
        message: None,
        category,
        assurance: Assurance::None,
    }
}
/// Pack a symmetric matrix in Clarabel's upper-column svec convention. This small
/// conversion is necessary because Clarabel's equivalent helper is crate-private.
pub fn svec(matrix: faer::MatRef<'_, f64>) -> Result<Vec<f64>, ProblemError> {
    if matrix.nrows() != matrix.ncols() {
        return Err(ProblemError::Contract("PSD matrix is not square".into()));
    }
    let mut result = Vec::new();
    for c in 0..matrix.ncols() {
        for r in 0..=c {
            let v = matrix[(r, c)];
            if !v.is_finite() || v != matrix[(c, r)] {
                return Err(ProblemError::Contract(
                    "PSD matrix must be finite symmetric".into(),
                ));
            }
            result.push(
                v * if r == c {
                    1.0
                } else {
                    std::f64::consts::SQRT_2
                },
            );
        }
    }
    Ok(result)
}
/// Original-space cone violation. Symmetric eigenvalues use faer; nonsymmetric
/// closed-cone predicates are boundary glue because Clarabel's public margins
/// deliberately panic for exponential and power cones.
pub(crate) fn cone_violation(cone: &Cone, s: &[f64]) -> Result<f64, ProblemError> {
    let violation = match cone {
        Cone::Zero { .. } => s.iter().map(|v| v.abs()).fold(0.0, f64::max),
        Cone::Nonnegative { .. } => s.iter().map(|v| -v).fold(0.0, f64::max),
        Cone::SecondOrder { .. } => s[1..].iter().fold(0.0f64, |a, v| a.hypot(*v)) - s[0],
        Cone::Exponential => {
            let (x, y, z) = (s[0], s[1], s[2]);
            if y > 0.0 && z > 0.0 {
                (x - y * (z.ln() - y.ln())).max(-y).max(-z)
            } else if y == 0.0 {
                x.max(-z)
            } else {
                // Distance to a feasible closure point (min(x,0),0,max(z,0)).
                x.max(0.0).hypot(y).hypot((-z).max(0.0))
            }
        }
        Cone::Power { alpha } => power_violation(&[*alpha, 1.0 - *alpha], &s[..2], s[2].abs()),
        Cone::GeneralizedPower { alpha, dimension } => power_violation(
            alpha,
            &s[..alpha.len()],
            s[alpha.len()..alpha.len() + dimension]
                .iter()
                .fold(0.0f64, |v, x| v.hypot(*x)),
        ),
        Cone::PsdTriangle { order } => {
            let n = *order;
            let mut m = faer::Mat::zeros(n, n);
            let mut k = 0;
            for c in 0..n {
                for r in 0..=c {
                    let v = s[k]
                        / if r == c {
                            1.0
                        } else {
                            std::f64::consts::SQRT_2
                        };
                    m[(r, c)] = v;
                    m[(c, r)] = v;
                    k += 1;
                }
            }
            let eig = m
                .self_adjoint_eigenvalues(faer::Side::Lower)
                .map_err(|e| ProblemError::numerical(format!("PSD quality eigensolve: {e:?}")))?;
            eig.iter().map(|v| -v).fold(0.0, f64::max)
        }
    };
    Ok(violation.max(0.0))
}
fn power_violation(alpha: &[f64], x: &[f64], norm: f64) -> f64 {
    let negative = x.iter().map(|v| -v).fold(0.0, f64::max);
    if negative > 0.0 {
        return negative.max(norm);
    }
    let product = if x.contains(&0.0) {
        0.0
    } else {
        alpha
            .iter()
            .zip(x)
            .map(|(a, x)| a * x.ln())
            .sum::<f64>()
            .exp()
    };
    (norm - product).max(negative)
}
pub(crate) fn quality(
    p: &ConicProblem,
    x: &[f64],
    t: &Tolerances,
) -> Result<Quality, ProblemError> {
    let mut s = p.rhs.clone();
    for (c, &x) in x.iter().enumerate() {
        for k in p.constraints.column(c) {
            s[p.constraints.row_indices[k]] -= p.constraints.values[k] * x;
        }
    }
    if s.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite conic residual"));
    }
    let mut rows = Vec::new();
    let mut start = 0;
    for cone in &p.cones {
        let end = start + cone.dim();
        if matches!(cone, Cone::Zero { .. } | Cone::Nonnegative { .. }) {
            for (i, slack) in s.iter().enumerate().take(end).skip(start) {
                rows.push(Violation {
                    id: p.contract.rows[i],
                    physical: if matches!(cone, Cone::Zero { .. }) {
                        slack.abs()
                    } else {
                        (-slack).max(0.0)
                    },
                    tolerance: t.rows[i],
                });
            }
            start = end;
            continue;
        }
        // The nonlinear cone is one homogeneous geometric block in common normalized coordinates.
        let tolerance = t.rows[start..end]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        rows.push(Violation {
            id: p.contract.rows[start],
            physical: cone_violation(cone, &s[start..end])?,
            tolerance,
        });
        start = end;
    }
    let bounds = p
        .contract
        .variables
        .iter()
        .zip(x)
        .zip(&t.variables)
        .map(|((v, x), t)| Violation {
            id: v.id,
            physical: interval(*x, v.lower, v.upper),
            tolerance: *t,
        })
        .collect();
    Quality::new(rows, bounds, vec![])
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::GramCertificate;
    #[test]
    fn svec_preserves_trace_inner_product_and_column_order() {
        let a = faer::Mat::from_fn(2, 2, |r, c| if r == c { (r + 1) as f64 } else { 3.0 });
        let v = svec(a.as_ref()).unwrap();
        assert_eq!(v, vec![1.0, 3.0 * std::f64::consts::SQRT_2, 2.0]);
        assert!((v.iter().map(|x| x * x).sum::<f64>() - 23.0).abs() < 1e-12);
    }
    #[test]
    fn closed_nonsymmetric_cones_handle_boundaries_without_library_panics() {
        assert_eq!(
            cone_violation(&Cone::Exponential, &[-1.0, 0.0, 2.0]).unwrap(),
            0.0
        );
        assert_eq!(
            cone_violation(&Cone::Power { alpha: 0.5 }, &[1.0, 1.0, -1.0]).unwrap(),
            0.0
        );
        assert!(cone_violation(&Cone::Exponential, &[2.0, 1.0, 1.0]).unwrap() > 0.0);
    }
    #[test]
    fn all_cone_quality_profiles_have_finite_boundary_measurements() {
        for (cone, point) in [
            (Cone::Zero { dimension: 2 }, vec![0., 0.]),
            (Cone::Nonnegative { dimension: 2 }, vec![0., 2.]),
            (Cone::SecondOrder { dimension: 3 }, vec![5., 3., 4.]),
            (Cone::Exponential, vec![0., 1., 1.]),
            (Cone::Power { alpha: 0.25 }, vec![1., 1., 1.]),
            (
                Cone::GeneralizedPower {
                    alpha: vec![0.5, 0.5],
                    dimension: 2,
                },
                vec![1., 1., 0.6, 0.8],
            ),
            (Cone::PsdTriangle { order: 2 }, vec![1., 0., 2.]),
        ] {
            assert!(cone_violation(&cone, &point).unwrap().abs() < 1e-12);
        }
        assert!(
            cone_violation(&Cone::Exponential, &[2.0, -1.0, -1.0])
                .unwrap()
                .is_finite()
        );
        assert!(cone_violation(&Cone::PsdTriangle { order: 2 }, &[-1., 0., 2.]).unwrap() > 0.);
    }
    fn reuse_problem() -> (ConicProblem, GramCertificate) {
        let mut contract = crate::solver_tests::contract();
        contract.variables[0].lower = 0.0;
        contract.variables[0].upper = 4.0;
        let p = ConicProblem {
            contract,
            quadratic: SparseMatrix::zeros(1, 1),
            objective: vec![1.0],
            constraints: SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![-1.0]),
            rhs: vec![0.0],
            cones: vec![Cone::Nonnegative { dimension: 1 }],
            objective_constant: 3.0,
        };
        let q = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
        let certificate = crate::solver_tests::certify(&q, 1.0);
        (p, certificate)
    }
    #[test]
    fn native_conic_bounds_and_data_reuse_preserve_original_rows() {
        let (mut p, certificate) = reuse_problem();
        let stamp = crate::solver_tests::stamp(Backend::Clarabel);
        let reusable = Settings {
            mode: Mode::ReusableData,
            ..Settings::default()
        };
        let mut session = Session::new(
            &p,
            &certificate,
            &Controls::default(),
            &ResolvedAccuracy::nominal(),
            &reusable,
            stamp.clone(),
        )
        .unwrap();
        assert_eq!(session.bounds.len(), 2);
        assert_eq!(session.rows, 1);
        p.objective[0] = 2.0;
        p.contract.variables[0].upper = 3.0;
        session
            .update(&p, &certificate, &reusable, stamp.clone())
            .unwrap();
        // A different mode never updates a retained session.
        assert!(
            session
                .update(&p, &certificate, &Settings::default(), stamp.clone())
                .is_err()
        );
        p.contract.variables[0].upper = f64::INFINITY;
        assert!(session.update(&p, &certificate, &reusable, stamp).is_err());
    }
    /// The conic boundary vocabulary is pse-owned (F09): its encoding is pinned here, the
    /// layout and request identities derive from it alone, and Clarabel sees only the
    /// adapter's mapping. A Clarabel serde change therefore cannot move an identity or a
    /// Python contract.
    #[test]
    fn clarabel_boundary_types_are_pse_owned() {
        // (a) Pinned pse encoding of every cone kind and of the matrix.
        let cones = vec![
            Cone::Zero { dimension: 1 },
            Cone::Nonnegative { dimension: 2 },
            Cone::SecondOrder { dimension: 3 },
            Cone::Exponential,
            Cone::Power { alpha: 0.5 },
            Cone::GeneralizedPower {
                alpha: vec![0.25, 0.75],
                dimension: 1,
            },
            Cone::PsdTriangle { order: 2 },
        ];
        let encoded = serde_json::to_string(&cones).unwrap();
        assert_eq!(
            encoded,
            concat!(
                r#"[{"kind":"zero","dimension":1},{"kind":"nonnegative","dimension":2},"#,
                r#"{"kind":"second_order","dimension":3},{"kind":"exponential"},"#,
                r#"{"kind":"power","alpha":0.5},"#,
                r#"{"kind":"generalized_power","alpha":[0.25,0.75],"dimension":1},"#,
                r#"{"kind":"psd_triangle","order":2}]"#
            )
        );
        assert_eq!(serde_json::from_str::<Vec<Cone>>(&encoded).unwrap(), cones);
        assert!(
            serde_json::from_str::<Cone>(r#"{"kind":"zero","dimension":1,"extra":0}"#).is_err()
        );
        assert!(serde_json::from_str::<Cone>(r#"{"ZeroConeT":1}"#).is_err());
        let matrix = SparseMatrix::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.5, -2.0]);
        let encoded = serde_json::to_string(&matrix).unwrap();
        assert_eq!(
            encoded,
            r#"{"rows":2,"columns":2,"column_starts":[0,1,2],"row_indices":[0,1],"values":[1.5,-2.0]}"#
        );
        assert_eq!(
            serde_json::from_str::<SparseMatrix>(&encoded).unwrap(),
            matrix
        );
        assert!(
            serde_json::from_str::<SparseMatrix>(
                r#"{"m":1,"n":1,"colptr":[0,0],"rowval":[],"nzval":[]}"#
            )
            .is_err()
        );
        matrix.validate().unwrap();
        assert!(
            SparseMatrix::new(2, 1, vec![0, 2], vec![1, 0], vec![1., 1.])
                .validate()
                .is_err()
        );
        assert!(
            SparseMatrix::new(1, 1, vec![0, 1], vec![1], vec![1.])
                .validate()
                .is_err()
        );
        // Settings: defaults are the pinned library's, round trip exactly, and refuse
        // unknown or policy-owned fields.
        let defaults = Settings::default();
        let encoded = serde_json::to_string(&defaults).unwrap();
        assert_eq!(
            serde_json::from_str::<Settings>(&encoded).unwrap(),
            defaults
        );
        assert_eq!(serde_json::from_str::<Settings>("{}").unwrap(), defaults);
        assert!(serde_json::from_str::<Settings>(r#"{"tol_feas":1e-6}"#).is_err());
        assert!(serde_json::from_str::<Settings>(r#"{"max_iter":5}"#).is_err());
        let library = DefaultSettings::<f64>::default();
        assert_eq!(defaults.max_step_fraction, library.max_step_fraction);
        assert_eq!(defaults.presolve_enable, library.presolve_enable);
        #[cfg(feature = "sdp")]
        assert_eq!(
            CHORDAL_DEFAULTS,
            (
                library.chordal_decomposition_enable,
                [
                    MergeMethod::None,
                    MergeMethod::ParentChild,
                    MergeMethod::CliqueGraph
                ]
                .into_iter()
                .find(|m| merge_method(*m) == library.chordal_decomposition_merge_method)
                .unwrap(),
                library.chordal_decomposition_compact,
                library.chordal_decomposition_complete_dual,
            )
        );
        let reusable: Settings = serde_json::from_str(
            r#"{"mode":"reusable_data","max_step_fraction":0.9,"presolve_enable":true}"#,
        )
        .unwrap();
        assert_eq!(reusable.mode, Mode::ReusableData);

        // (b) Identities come from the pse encoding only, never from Clarabel's.
        let key = cone_key(&cones).unwrap();
        assert_eq!(
            key,
            crate::identity::of(pse_ids::Frame::ConeLayoutV3, &cones).unwrap()
        );
        assert_ne!(
            key,
            cone_key(&[Cone::Zero { dimension: 1 }]).unwrap(),
            "layout identity covers every block"
        );
        #[cfg(feature = "sdp")]
        {
            let mapped = clarabel_cones(&cones).unwrap();
            assert_ne!(
                serde_json::to_string(&cones).unwrap(),
                serde_json::to_string(&mapped).unwrap()
            );
        }
        assert_ne!(
            serde_json::to_string(&matrix).unwrap(),
            serde_json::to_string(&matrix.to_clarabel()).unwrap()
        );

        // (c) The mapping reaches Clarabel.
        for (cone, native) in [
            (Cone::Zero { dimension: 1 }, SupportedConeT::ZeroConeT(1)),
            (
                Cone::Nonnegative { dimension: 2 },
                SupportedConeT::NonnegativeConeT(2),
            ),
            (
                Cone::SecondOrder { dimension: 3 },
                SupportedConeT::SecondOrderConeT(3),
            ),
            (Cone::Exponential, SupportedConeT::ExponentialConeT()),
            (Cone::Power { alpha: 0.5 }, SupportedConeT::PowerConeT(0.5)),
            (
                Cone::GeneralizedPower {
                    alpha: vec![0.25, 0.75],
                    dimension: 1,
                },
                SupportedConeT::GenPowerConeT(vec![0.25, 0.75], 1),
            ),
        ] {
            assert_eq!(cone.to_clarabel().unwrap(), native);
        }
        #[cfg(feature = "sdp")]
        assert_eq!(
            Cone::PsdTriangle { order: 2 }.to_clarabel().unwrap(),
            SupportedConeT::PSDTriangleConeT(2)
        );
        #[cfg(not(feature = "sdp"))]
        assert!(matches!(
            Cone::PsdTriangle { order: 2 }.to_clarabel(),
            Err(ProblemError::Unsupported(_))
        ));
        let native = matrix.to_clarabel();
        assert_eq!(
            (
                native.m,
                native.n,
                &native.colptr,
                &native.rowval,
                &native.nzval
            ),
            (
                2,
                2,
                &matrix.column_starts,
                &matrix.row_indices,
                &matrix.values
            )
        );
        let custom = Settings {
            max_step_fraction: 0.9,
            presolve_enable: false,
            ..Settings::default()
        };
        let accuracy = ResolvedAccuracy::nominal();
        let mapped = settings(&custom, &Controls::default(), &accuracy, custom.mode).unwrap();
        assert_eq!(mapped.max_step_fraction, 0.9);
        assert!(!mapped.presolve_enable);
        assert_eq!(mapped.tol_feas, accuracy.feasibility);
        let single = settings(
            &reusable,
            &Controls::default(),
            &accuracy,
            Mode::SingleSolve,
        )
        .unwrap();
        assert!(single.presolve_enable);
        let reused = settings(
            &reusable,
            &Controls::default(),
            &accuracy,
            Mode::ReusableData,
        )
        .unwrap();
        assert!(!reused.presolve_enable && !reused.input_sparse_dropzeros);
        assert_eq!(reused.max_step_fraction, 0.9);
        #[cfg(not(feature = "sdp"))]
        assert!(matches!(
            settings(
                &Settings {
                    chordal_decomposition_enable: false,
                    ..Settings::default()
                },
                &Controls::default(),
                &accuracy,
                Mode::SingleSolve
            ),
            Err(ProblemError::Unsupported(_))
        ));
    }
}
