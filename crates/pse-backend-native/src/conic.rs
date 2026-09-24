// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit Clarabel cones, bounded data reuse and source-space postprocessing.
use crate::{
    ConicProblem, GramCertificate, ProblemError,
    quality::{Quality, Tolerances, Violation, interval},
    solve::*,
};
/// Complete native cone vocabulary.
pub use clarabel::solver::SupportedConeT as Cone;
use clarabel::{
    algebra::CscMatrix,
    solver::{DefaultInfo, DefaultSettings, DefaultSolver, IPSolver, SolverStatus, SupportedConeT},
};
use std::collections::BTreeMap;
/// Complete pinned Clarabel settings.
pub type Settings = DefaultSettings<f64>;
use clarabel::solver::traits::Settings as _;
/// Stable typed cone identity, including every feature-gated dimension and exponent.
pub fn cone_key(cones: &[Cone<f64>]) -> pse_ids::ContentHash {
    let mut h = pse_ids::FramedHasher::new("pse.cone.layout.v1");
    h.u64(cones.len() as u64);
    for c in cones {
        use Cone::*;
        match c {
            ZeroConeT(n) => {
                h.u64(0).u64(*n as u64);
            }
            NonnegativeConeT(n) => {
                h.u64(1).u64(*n as u64);
            }
            SecondOrderConeT(n) => {
                h.u64(2).u64(*n as u64);
            }
            ExponentialConeT() => {
                h.u64(3);
            }
            PowerConeT(a) => {
                h.u64(4).u64(a.to_bits());
            }
            GenPowerConeT(a, d) => {
                h.u64(5).u64(*d as u64).u64(a.len() as u64);
                for v in a {
                    h.u64(v.to_bits());
                }
            }
            #[cfg(feature = "sdp")]
            PSDTriangleConeT(n) => {
                h.u64(6).u64(*n as u64);
            }
        }
    }
    h.finish_hash()
}
/// Native preprocessing and mutable-data reuse are distinct execution profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Native presolve/chordal preprocessing may change the native layout.
    SingleSolve,
    /// Preserve structure to use Clarabel's data update API.
    ReusableData,
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
fn data(p: &ConicProblem) -> Data {
    let mut bounds = Vec::new();
    let mut rhs = p.rhs.clone();
    let mut cones = p.cones.clone();
    for (i, v) in p.contract.variables.iter().enumerate() {
        if v.lower.is_finite() {
            bounds.push((i, true));
            rhs.push(-v.lower)
        }
        if v.upper.is_finite() {
            bounds.push((i, false));
            rhs.push(v.upper)
        }
    }
    if !bounds.is_empty() {
        cones.push(SupportedConeT::NonnegativeConeT(bounds.len()));
    }
    let mut colptr = vec![0];
    let mut rowval = Vec::new();
    let mut nzval = Vec::new();
    for c in 0..p.constraints.n {
        for k in p.constraints.colptr[c]..p.constraints.colptr[c + 1] {
            rowval.push(p.constraints.rowval[k]);
            nzval.push(p.constraints.nzval[k]);
        }
        for (r, &(v, lower)) in bounds.iter().enumerate() {
            if c == v {
                rowval.push(p.constraints.m + r);
                nzval.push(if lower { -1.0 } else { 1.0 });
            }
        }
        colptr.push(rowval.len());
    }
    Data {
        a: CscMatrix::new(rhs.len(), p.constraints.n, colptr, rowval, nzval),
        rhs,
        cones,
        bounds,
    }
}
fn settings(
    mut settings: DefaultSettings<f64>,
    controls: &Controls,
    mode: Mode,
) -> Result<DefaultSettings<f64>, ProblemError> {
    controls.validate()?;
    // Full settings are exposed directly; there is no unvalidated string option channel.
    if !controls.options.is_empty() {
        return Err(ProblemError::Contract(
            "Clarabel uses typed DefaultSettings instead of native option strings".into(),
        ));
    }
    if controls.threads != 1 {
        return Err(ProblemError::Contract(
            "Clarabel QDLDL/serial-netlib profile requires one core".into(),
        ));
    }
    settings.max_iter = controls.iterations;
    settings.time_limit = controls.time_limit.as_secs_f64();
    settings.max_threads = 1;
    settings.tol_gap_abs = controls.tolerance;
    settings.tol_gap_rel = controls.tolerance;
    settings.tol_feas = controls.tolerance;
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
        certificate: &GramCertificate,
        controls: &Controls,
        native: DefaultSettings<f64>,
        mode: Mode,
        compatibility: Compatibility,
    ) -> Result<Self, ProblemError> {
        p.validate(certificate)?;
        let settings = settings(native, controls, mode)?;
        let d = data(p);
        let solver =
            DefaultSolver::new(&p.quadratic, &p.objective, &d.a, &d.rhs, &d.cones, settings)
                .map_err(|e| ProblemError::Contract(format!("Clarabel problem: {e}")))?;
        Ok(Self {
            solver,
            compatibility,
            mode,
            rows: p.rhs.len(),
            bounds: d.bounds,
            signature: d.cones,
            a_pattern: (d.a.colptr, d.a.rowval),
            p_pattern: (p.quadratic.colptr.clone(), p.quadratic.rowval.clone()),
        })
    }
    /// Native update restrictions are checked before updating any part of the model.
    pub fn update(
        &mut self,
        p: &ConicProblem,
        certificate: &GramCertificate,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        p.validate(certificate)?;
        let d = data(p);
        if self.mode != Mode::ReusableData
            || !self.solver.is_data_update_allowed()
            || self.compatibility.layout != compatibility.layout
            || compatibility.backend != Backend::Clarabel
            || self.signature != d.cones
            || self.bounds != d.bounds
            || self.a_pattern != (d.a.colptr.clone(), d.a.rowval.clone())
            || self.p_pattern != (p.quadratic.colptr.clone(), p.quadratic.rowval.clone())
        {
            return Err(ProblemError::Contract(
                "Clarabel update changes layout or requires disabled preprocessing".into(),
            ));
        }
        self.solver
            .update_data(&p.quadratic, &p.objective, &d.a, &d.rhs)
            .map_err(|e| ProblemError::Contract(format!("Clarabel update: {e}")))?;
        self.compatibility = compatibility;
        Ok(())
    }
    /// Solve using native preprocessing/postprocessing. No external iterate warm start is claimed.
    pub fn solve(
        &mut self,
        p: &ConicProblem,
        controls: &Controls,
        native: DefaultSettings<f64>,
        execution: Execution,
        tolerances: &Tolerances,
    ) -> Result<SolveReport, ProblemError> {
        tolerances.validate(p.contract.variables.len(), p.contract.rows.len())?;
        let settings = settings(native, controls, self.mode)?;
        let settings_json =
            serde_json::to_string(&settings).map_err(|e| ProblemError::Contract(e.to_string()))?;
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
        report.provenance.insert(
            "native".into(),
            if cfg!(feature = "sdp") {
                "Clarabel 0.11.1; QDLDL; SDP serial netlib LP64"
            } else {
                "Clarabel 0.11.1; QDLDL; SDP unavailable"
            }
            .into(),
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
            report.termination.assurance = Assurance::None
        }
        if matches!(
            solution.status,
            SolverStatus::PrimalInfeasible
                | SolverStatus::DualInfeasible
                | SolverStatus::AlmostPrimalInfeasible
                | SolverStatus::AlmostDualInfeasible
        ) {
            report.certificate = Some(Certificate {
                kind: format!("{:?}", solution.status),
                primal: matches!(
                    solution.status,
                    SolverStatus::DualInfeasible | SolverStatus::AlmostDualInfeasible
                )
                .then(|| solution.x.clone()),
                dual: matches!(
                    solution.status,
                    SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible
                )
                .then(|| solution.z.clone()),
            });
            // Certificate duals include appended bound rows, explicitly described.
            report.provenance.insert("certificate.rows".into(),"original conic rows then finite variable bounds in source variable order, lower before upper".into());
        } else if solution.status != SolverStatus::Unsolved
            && solution.x.iter().all(|v| v.is_finite())
            && solution.obj_val.is_finite()
        {
            let mut lower = vec![0.0; solution.x.len()];
            let mut upper = lower.clone();
            for (k, &(i, l)) in self.bounds.iter().enumerate() {
                if l {
                    lower[i] = solution.z[self.rows + k]
                } else {
                    upper[i] = solution.z[self.rows + k]
                }
            }
            report.candidate = Some(Candidate {
                primal: solution.x.clone(),
                objective: Some(solution.obj_val + p.objective_constant),
                row_dual: Some(solution.z[..self.rows].to_vec()),
                bound_dual: Some((lower, upper)),
                reduced_costs: None,
                slacks: Some(solution.s[..self.rows].to_vec()),
            });
            match crate::quality::contained(|| quality(p, &solution.x, tolerances)) {
                Ok(q) => {
                    if !q.feasible() {
                        report.termination.assurance = Assurance::None
                    }
                    report.quality = Some(q)
                }
                Err(e) => {
                    report.validation_error = Some(e.to_string());
                    report.termination.assurance = Assurance::None
                }
            }
        } else {
            report.termination.assurance = Assurance::None
        }
        Ok(report)
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
/// Native conic statuses retain certificate versus candidate distinctions.
pub fn termination(status: SolverStatus) -> NativeTermination {
    let (category, assurance) = match status {
        SolverStatus::Solved => (Termination::Success, Assurance::NativeOptimal),
        SolverStatus::AlmostSolved => (Termination::Acceptable, Assurance::NativeOptimal),
        SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible => {
            (Termination::Infeasible, Assurance::Certificate)
        }
        SolverStatus::DualInfeasible | SolverStatus::AlmostDualInfeasible => {
            (Termination::Unbounded, Assurance::Certificate)
        }
        SolverStatus::MaxIterations => (Termination::Limit, Assurance::None),
        SolverStatus::MaxTime => (Termination::TimeLimit, Assurance::None),
        SolverStatus::CallbackTerminated => (Termination::Cancelled, Assurance::None),
        SolverStatus::NumericalError | SolverStatus::InsufficientProgress => {
            (Termination::Numerical, Assurance::None)
        }
        SolverStatus::Unsolved => (Termination::Invalid, Assurance::None),
    };
    NativeTermination {
        code: status as i64,
        name: format!("{status:?}"),
        message: None,
        category,
        assurance,
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
fn cone_violation(cone: &SupportedConeT<f64>, s: &[f64]) -> Result<f64, ProblemError> {
    use SupportedConeT::*;
    let violation = match cone {
        ZeroConeT(_) => s.iter().map(|v| v.abs()).fold(0.0, f64::max),
        NonnegativeConeT(_) => s.iter().map(|v| -v).fold(0.0, f64::max),
        SecondOrderConeT(_) => s[1..].iter().fold(0.0f64, |a, v| a.hypot(*v)) - s[0],
        ExponentialConeT() => {
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
        PowerConeT(a) => power_violation(&[*a, 1.0 - *a], &s[..2], s[2].abs()),
        GenPowerConeT(a, d) => power_violation(
            a,
            &s[..a.len()],
            s[a.len()..a.len() + d]
                .iter()
                .fold(0.0f64, |v, x| v.hypot(*x)),
        ),
        #[cfg(feature = "sdp")]
        PSDTriangleConeT(n) => {
            let mut m = faer::Mat::zeros(*n, *n);
            let mut k = 0;
            for c in 0..*n {
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
                .map_err(|e| ProblemError::Contract(format!("PSD quality eigensolve: {e:?}")))?;
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
fn dim(cone: &SupportedConeT<f64>) -> usize {
    use SupportedConeT::*;
    match cone {
        ZeroConeT(n) | NonnegativeConeT(n) | SecondOrderConeT(n) => *n,
        ExponentialConeT() | PowerConeT(_) => 3,
        GenPowerConeT(a, d) => a.len() + d,
        #[cfg(feature = "sdp")]
        PSDTriangleConeT(n) => n * (n + 1) / 2,
    }
}
fn quality(p: &ConicProblem, x: &[f64], t: &Tolerances) -> Result<Quality, ProblemError> {
    let mut s = p.rhs.clone();
    for (c, &x) in x.iter().enumerate() {
        for k in p.constraints.colptr[c]..p.constraints.colptr[c + 1] {
            s[p.constraints.rowval[k]] -= p.constraints.nzval[k] * x;
        }
    }
    if s.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract("nonfinite conic residual".into()));
    }
    let mut rows = Vec::new();
    let mut start = 0;
    for cone in &p.cones {
        let end = start + dim(cone);
        let tolerance = t.rows[start];
        if t.rows[start..end].iter().any(|v| *v != tolerance) {
            return Err(ProblemError::Contract(
                "cone coordinates need one explicit common physical normalization".into(),
            ));
        }
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
            cone_violation(&SupportedConeT::ExponentialConeT(), &[-1.0, 0.0, 2.0]).unwrap(),
            0.0
        );
        assert_eq!(
            cone_violation(&SupportedConeT::PowerConeT(0.5), &[1.0, 1.0, -1.0]).unwrap(),
            0.0
        );
        assert!(
            cone_violation(&SupportedConeT::ExponentialConeT(), &[2.0, 1.0, 1.0]).unwrap() > 0.0
        );
    }
    #[test]
    fn all_cone_quality_profiles_have_finite_boundary_measurements() {
        use SupportedConeT::*;
        for (cone, point) in [
            (ZeroConeT(2), vec![0., 0.]),
            (NonnegativeConeT(2), vec![0., 2.]),
            (SecondOrderConeT(3), vec![5., 3., 4.]),
            (ExponentialConeT(), vec![0., 1., 1.]),
            (PowerConeT(0.25), vec![1., 1., 1.]),
            (GenPowerConeT(vec![0.5, 0.5], 2), vec![1., 1., 0.6, 0.8]),
        ] {
            assert!(cone_violation(&cone, &point).unwrap().abs() < 1e-12);
        }
        assert!(
            cone_violation(&ExponentialConeT(), &[2.0, -1.0, -1.0])
                .unwrap()
                .is_finite()
        );
        #[cfg(feature = "sdp")]
        {
            assert_eq!(
                cone_violation(&PSDTriangleConeT(2), &[1., 0., 2.]).unwrap(),
                0.
            );
            assert!(cone_violation(&PSDTriangleConeT(2), &[-1., 0., 2.]).unwrap() > 0.);
        }
    }
    #[test]
    fn native_conic_bounds_and_data_reuse_preserve_original_rows() {
        let mut contract = crate::solver_tests::contract();
        contract.variables[0].lower = 0.0;
        contract.variables[0].upper = 4.0;
        let mut p = ConicProblem {
            contract,
            quadratic: CscMatrix::zeros((1, 1)),
            objective: vec![1.0],
            constraints: CscMatrix::new(1, 1, vec![0, 1], vec![0], vec![-1.0]),
            rhs: vec![0.0],
            cones: vec![SupportedConeT::NonnegativeConeT(1)],
            objective_constant: 3.0,
        };
        let q = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
        let certificate = GramCertificate::new(&q, 1.0, &faer::Mat::zeros(0, 1), &[], 10).unwrap();
        let stamp = crate::solver_tests::stamp(Backend::Clarabel);
        let mut session = Session::new(
            &p,
            &certificate,
            &Controls::default(),
            Settings::default(),
            Mode::ReusableData,
            stamp.clone(),
        )
        .unwrap();
        assert_eq!(session.bounds.len(), 2);
        assert_eq!(session.rows, 1);
        p.objective[0] = 2.0;
        p.contract.variables[0].upper = 3.0;
        session.update(&p, &certificate, stamp.clone()).unwrap();
        p.contract.variables[0].upper = f64::INFINITY;
        assert!(session.update(&p, &certificate, stamp).is_err());
    }
}
