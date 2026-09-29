// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Clarabel on coefficient programs, its KKT solvers and chordal decomposition, and typed
//! certificates verified in original coordinates (Plan 22 C4, I10–I11).
use crate::{
    CoefficientProblem, OracleContract, ProblemError, Variable,
    conic::{self, Direct},
    execution::{self, BackendSettings, Coefficients, Evaluation, OriginalModel, Retained, Step},
    quality::{self, Tolerances},
    solve::*,
    solver_tests::stamp,
};
#[cfg(feature = "sdp")]
use crate::{
    ConicProblem,
    conic::{Cone, SparseMatrix},
};
use faer::sparse::{SparseColMat, Triplet};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    binding::ObjectiveSense, convexity::QuadraticEvidence, normalization::Normalization,
};
use pse_model::generated::enums::ModelingVariableDomain;
use std::sync::{Arc, atomic::AtomicBool};

pub(crate) fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
/// The coefficient data is its own original model.
pub(crate) struct Affine<'p>(pub(crate) &'p CoefficientProblem);
impl OriginalModel for Affine<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<Evaluation, ProblemError> {
        let p = self.0;
        let mut constraints = vec![0.0; p.bounds.len()];
        for (c, x) in primal.iter().enumerate().take(p.constraints.ncols()) {
            for (r, v) in p
                .constraints
                .row_idx_of_col(c)
                .zip(p.constraints.val_of_col(c))
            {
                constraints[r] += v * x;
            }
        }
        Ok(Evaluation {
            constraints,
            objective: Some(p.objective_at(primal)),
            sources: vec![],
        })
    }
}
pub(crate) fn problem(
    bounds: &[(f64, f64)],
    rows: &[(f64, f64)],
    entries: &[(usize, usize, f64)],
    objective: Vec<f64>,
    sense: ObjectiveSense,
) -> CoefficientProblem {
    let n = bounds.len();
    CoefficientProblem {
        contract: OracleContract {
            identity: ContentHash::from_bytes([1; 32]),
            variables: bounds
                .iter()
                .enumerate()
                .map(|(i, (lower, upper))| Variable {
                    id: id(1 + i as u8),
                    lower: *lower,
                    upper: *upper,
                })
                .collect(),
            rows: (0..rows.len()).map(|i| id(100 + i as u8)).collect(),
            derivatives: pse_kernels::DerivativeOrder::Second,
            smoothness: pse_kernels::DerivativeOrder::Second,
        },
        objective,
        objective_constant: 4.0,
        sense,
        domains: vec![ModelingVariableDomain::Continuous; n],
        assumptions: stamp(Backend::Highs).data,
        constraints: SparseColMat::try_new_from_triplets(
            rows.len(),
            n,
            &entries
                .iter()
                .map(|(r, c, v)| Triplet::new(*r, *c, *v))
                .collect::<Vec<_>>(),
        )
        .unwrap(),
        hessian: None,
        bounds: rows.to_vec(),
        objectives: Vec::new(),
    }
}
pub(crate) fn budgets(p: &CoefficientProblem, budget: f64) -> Tolerances {
    Tolerances {
        variables: vec![budget; p.contract.variables.len()],
        rows: vec![budget; p.bounds.len()],
        integrality: 1e-7,
    }
}
/// The coefficient runner on one backend, in the given coordinates.
pub(crate) fn run(
    p: &CoefficientProblem,
    evidence: Option<&dyn QuadraticEvidence>,
    backend: Backend,
    settings: &BackendSettings,
    controls: &Controls,
    normalization: &Normalization,
    tolerances: &Tolerances,
) -> Result<SolveReport, ProblemError> {
    let accuracy = ResolvedAccuracy::nominal();
    let constants = vec![0.0; p.bounds.len()];
    execution::coefficients(
        Step {
            adapter: execution::adapter(backend),
            settings,
            controls,
            accuracy: &accuracy,
            execution: Execution::new(Arc::new(AtomicBool::new(false)), controls),
            tolerances,
            normalization,
            compatibility: stamp(backend),
            warm: None,
        },
        &mut Retained::default(),
        Coefficients {
            problem: p,
            certificate: evidence,
            row_constants: &constants,
            row_bounds: p.bounds.clone(),
            original: &mut Affine(p),
        },
    )
}
fn clarabel(direct: Direct) -> BackendSettings {
    BackendSettings::Clarabel(conic::Settings {
        direct,
        ..conic::Settings::default()
    })
}
/// max 3x + 2y + z + 4 over an equality, a ranged, an upper and a lower row and bounds:
/// the unique vertex x = 6, y = 3, z = -5 with objective 23.
pub(crate) fn mixed_lp() -> CoefficientProblem {
    problem(
        &[(0.0, 10.0), (0.0, f64::INFINITY), (-5.0, 5.0)],
        &[
            (4.0, 4.0),
            (1.0, 3.0),
            (f64::NEG_INFINITY, 5.0),
            (0.5, f64::INFINITY),
        ],
        &[
            (0, 0, 1.0),
            (0, 1, 1.0),
            (0, 2, 1.0),
            (1, 0, 1.0),
            (1, 1, -1.0),
            (2, 0, 1.0),
            (2, 2, 2.0),
            (3, 1, 1.0),
        ],
        vec![3.0, 2.0, 1.0],
        ObjectiveSense::Maximize,
    )
}
/// Distinct positive scales, so native and original coordinates differ.
pub(crate) fn scaled(p: &CoefficientProblem) -> Normalization {
    let n = p.contract.variables.len();
    let m = p.bounds.len();
    Normalization {
        variables: (0..n).map(|i| [2.0, 0.5, 4.0][i % 3]).collect(),
        rows: (0..m).map(|i| [3.0, 0.25, 8.0, 1.0][i % 4]).collect(),
        objective: 10.0,
    }
}
pub(crate) fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} vs {b}");
}

#[cfg(feature = "highs")]
#[test]
fn clarabel_lp_matches_highs() {
    let p = mixed_lp();
    let t = budgets(&p, 1e-7);
    let n = scaled(&p);
    let controls = Controls::default();
    let highs = run(
        &p,
        None,
        Backend::Highs,
        &BackendSettings::Default,
        &controls,
        &n,
        &t,
    )
    .unwrap();
    let cone = run(
        &p,
        None,
        Backend::Clarabel,
        &BackendSettings::Default,
        &controls,
        &n,
        &t,
    )
    .unwrap();
    for r in [&highs, &cone] {
        assert_eq!(r.termination.category, Termination::Success, "{r:?}");
        assert!(r.quality.as_ref().unwrap().feasible(), "{r:?}");
        assert!(r.validation_failure().is_none(), "{r:?}");
        assert_eq!(r.rows, p.contract.rows);
    }
    let (h, c) = (highs.candidate.unwrap(), cone.candidate.unwrap());
    for (a, b) in h.primal.iter().zip(&c.primal).chain([(&6.0, &c.primal[0])]) {
        near(*a, *b, 1e-6);
    }
    near(c.objective.unwrap(), 23.0, 1e-6);
    near(h.objective.unwrap(), c.objective.unwrap(), 1e-6);
    // One authored-sense multiplier per coefficient row, and reduced costs, with c = Aᵀy + d.
    let (y, d) = (c.row_dual.unwrap(), c.reduced_costs.unwrap());
    assert_eq!(y.len(), p.bounds.len());
    for (j, (cj, dj)) in p.objective.iter().zip(&d).enumerate() {
        let aty: f64 = p
            .constraints
            .row_idx_of_col(j)
            .zip(p.constraints.val_of_col(j))
            .map(|(r, v)| v * y[r])
            .sum();
        near(*cj, aty + dj, 1e-6);
    }
    for (a, b) in h.row_dual.unwrap().iter().zip(&y) {
        near(*a, *b, 1e-6);
    }
    assert_eq!(cone.backend, Backend::Clarabel);
    assert!(cone.provenance["lowering"].contains("zero cone"));
}

/// min x² + y² with x + y >= 3 over x, y in [0, 1]: infeasible, so Clarabel returns a
/// Farkas ray over the lowered row and the bounds, verified in original coordinates.
#[test]
fn clarabel_qp_farkas_certificate() {
    let mut p = problem(
        &[(0.0, 1.0), (0.0, 1.0)],
        &[(3.0, f64::INFINITY)],
        &[(0, 0, 1.0), (0, 1, 1.0)],
        vec![0.0, 0.0],
        ObjectiveSense::Minimize,
    );
    let q = SparseColMat::try_new_from_triplets(
        2,
        2,
        &[Triplet::new(0, 0, 2.0), Triplet::new(1, 1, 2.0)],
    )
    .unwrap();
    let proof = crate::solver_tests::certify(&q, 1.0);
    p.hessian = Some(q);
    let t = budgets(&p, 1e-7);
    let r = run(
        &p,
        Some(&proof),
        Backend::Clarabel,
        &BackendSettings::Default,
        &Controls::default(),
        &scaled(&p),
        &t,
    )
    .unwrap();
    assert_eq!(r.termination.category, Termination::Infeasible, "{r:?}");
    assert!(r.candidate.is_none());
    let c = r.certificate.as_ref().unwrap();
    assert_eq!(c.kind, CertificateKind::PrimalInfeasible);
    assert_eq!(c.accuracy, CertificateAccuracy::Full);
    // The lowered row's lower side, then both variables' lower and upper bounds.
    assert_eq!(
        c.ray
            .iter()
            .map(|e| (e.coordinate, e.id))
            .collect::<Vec<_>>(),
        [
            (RayCoordinate::RowLower, id(100)),
            (RayCoordinate::VariableLower, id(1)),
            (RayCoordinate::VariableUpper, id(1)),
            (RayCoordinate::VariableLower, id(2)),
            (RayCoordinate::VariableUpper, id(2)),
        ]
    );
    let v = c.verification.unwrap();
    assert!(v.verified && v.margin > 0.0 && v.objective < 0.0, "{v:?}");
    assert!(c.certified());
    assert_eq!(r.termination.assurance, Assurance::Certificate);
}

/// The certificate proves infeasibility of the original problem: Aᵀy + (bound rows) = 0 and
/// bᵀy < 0 hold in original coordinates for Clarabel's ray and for HiGHS's exported one,
/// solved in scaled coordinates; the native (scaled) ray does not satisfy the original data.
#[test]
fn farkas_certificate_verified_in_original_coordinates() {
    // x + 2y <= 1 and 3x + y >= 4 over x, y in [0, 1]: at most 1 + ... infeasible.
    let p = problem(
        &[(0.0, 1.0), (0.0, 1.0)],
        &[(f64::NEG_INFINITY, 1.0), (4.0, f64::INFINITY)],
        &[(0, 0, 1.0), (0, 1, 2.0), (1, 0, 3.0), (1, 1, 1.0)],
        vec![1.0, 1.0],
        ObjectiveSense::Minimize,
    );
    let t = budgets(&p, 1e-7);
    let n = scaled(&p);
    let mut settings = vec![(Backend::Clarabel, BackendSettings::Default)];
    if cfg!(feature = "highs") {
        settings.push((
            Backend::Highs,
            BackendSettings::Highs(crate::settings::highs::Settings {
                diagnostics: crate::settings::highs::Request {
                    rays: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
        ));
    }
    let (cone, _) = conic::lowering::data(&p).unwrap();
    let bounds = conic::bound_rows(&p.contract.variables);
    let original = |ray: &[f64]| {
        // Aᵀy over the cone rows then the bound rows, and bᵀy, in original data.
        let (rows, bound) = ray.split_at(cone.rhs.len());
        let mut residual = [0.0; 2];
        for (c, r) in residual.iter_mut().enumerate() {
            for k in cone.constraints.column(c) {
                *r += cone.constraints.values[k] * rows[cone.constraints.row_indices[k]];
            }
        }
        let mut objective: f64 = cone.rhs.iter().zip(rows).map(|(b, y)| b * y).sum();
        for (&(j, lower), y) in bounds.iter().zip(bound) {
            residual[j] += if lower { -y } else { *y };
            let v = &p.contract.variables[j];
            objective += if lower { -v.lower } else { v.upper } * y;
        }
        (residual, objective)
    };
    for (backend, settings) in settings {
        let r = run(&p, None, backend, &settings, &Controls::default(), &n, &t).unwrap();
        assert_eq!(r.termination.category, Termination::Infeasible, "{r:?}");
        let c = r.certificate.as_ref().unwrap_or_else(|| panic!("{r:?}"));
        assert_eq!(c.kind, CertificateKind::PrimalInfeasible);
        let ray: Vec<f64> = c.ray.iter().map(|e| e.value).collect();
        let scale = ray.iter().fold(0.0f64, |a, v| a.max(v.abs()));
        let (residual, objective) = original(&ray);
        for r in residual {
            assert!(r.abs() <= 1e-7 * scale, "{backend:?} {residual:?} {ray:?}");
        }
        assert!(objective < -1e-3 * scale, "{backend:?} {objective}");
        assert!(c.certified(), "{backend:?} {c:?}");
        assert_eq!(
            r.termination.assurance,
            Assurance::Certificate,
            "{backend:?}"
        );
        // The same values read as native coordinates break the original equations.
        let native: Vec<f64> = c
            .ray
            .iter()
            .map(|e| match e.coordinate {
                RayCoordinate::VariableLower | RayCoordinate::VariableUpper => {
                    let j = p
                        .contract
                        .variables
                        .iter()
                        .position(|v| v.id == e.id)
                        .unwrap();
                    e.value * n.variables[j]
                }
                _ => {
                    let i = p.contract.rows.iter().position(|r| *r == e.id).unwrap();
                    e.value * n.rows[i]
                }
            })
            .collect();
        let (residual, _) = original(&native);
        assert!(
            residual.iter().any(|r| r.abs() > 1e-3 * scale),
            "{backend:?} {residual:?}"
        );
    }
}

/// A problem infeasible by less than its acceptance budgets is never certified: its rows,
/// relaxed by their budgets, admit a point. Nor is a ray found at reduced accuracy.
#[test]
fn almost_infeasible_is_not_certified() {
    // x <= 1 and x >= 1 + 1e-9 with 1e-7 row budgets.
    let p = problem(
        &[(-10.0, 10.0)],
        &[(f64::NEG_INFINITY, 1.0), (1.0 + 1e-9, f64::INFINITY)],
        &[(0, 0, 1.0), (1, 0, 1.0)],
        vec![1.0],
        ObjectiveSense::Minimize,
    );
    let t = budgets(&p, 1e-7);
    let r = run(
        &p,
        None,
        Backend::Clarabel,
        &BackendSettings::Default,
        &Controls::default(),
        &Normalization::identity(1, 2),
        &t,
    )
    .unwrap();
    assert_ne!(r.termination.assurance, Assurance::Certificate, "{r:?}");
    assert!(
        r.certificate.as_ref().is_none_or(|c| !c.certified()),
        "{r:?}"
    );
    // The Farkas ray (1, 1) over the two rows: bᵀy = -1e-9, within the budgets 2e-7.
    let (cone, _) = conic::lowering::data(&p).unwrap();
    let farkas = |rows: [f64; 2]| {
        let entry = |coordinate, id, value| RayEntry {
            coordinate,
            id,
            value,
        };
        let mut c = InfeasibilityCertificate {
            kind: CertificateKind::PrimalInfeasible,
            accuracy: CertificateAccuracy::Full,
            ray: vec![
                entry(RayCoordinate::RowUpper, id(100), rows[0]),
                entry(RayCoordinate::RowLower, id(101), rows[1]),
                entry(RayCoordinate::VariableLower, id(1), 0.0),
                entry(RayCoordinate::VariableUpper, id(1), 0.0),
            ],
            verification: None,
        };
        crate::certificate::verify(
            &mut c,
            &cone,
            &conic::lowering::row_budgets(&conic::lowering::layout(&p.bounds), &t),
            ResolvedAccuracy::nominal().feasibility,
        );
        c
    };
    let almost = farkas([1.0, 1.0]);
    let v = almost.verification.unwrap();
    assert!(v.objective < 0.0 && v.margin < 0.0 && !v.verified, "{v:?}");
    assert!(!almost.certified());
    // Infeasible by more than the budgets: verified, but a reduced-accuracy ray still
    // carries no certificate.
    let clear = run(
        &problem(
            &[(-10.0, 10.0)],
            &[(f64::NEG_INFINITY, 1.0), (1.1, f64::INFINITY)],
            &[(0, 0, 1.0), (1, 0, 1.0)],
            vec![1.0],
            ObjectiveSense::Minimize,
        ),
        None,
        Backend::Clarabel,
        &BackendSettings::Default,
        &Controls::default(),
        &Normalization::identity(1, 2),
        &t,
    )
    .unwrap();
    let mut reduced = clear.clone();
    let certificate = reduced.certificate.as_mut().unwrap();
    assert!(certificate.certified(), "{certificate:?}");
    certificate.accuracy = CertificateAccuracy::Reduced;
    assert!(!certificate.certified());
    quality::qualify(&mut reduced, &ResolvedAccuracy::nominal());
    assert_eq!(reduced.termination.assurance, Assurance::None);
}

/// An LP and a convex QP on Clarabel with the requested KKT solver and threads.
fn solve_both(direct: Direct, threads: usize) -> [SolveReport; 2] {
    let lp = mixed_lp();
    let mut qp = problem(
        &[(-5.0, 5.0), (-5.0, 5.0)],
        &[(1.0, f64::INFINITY)],
        &[(0, 0, 1.0), (0, 1, 1.0)],
        vec![-1.0, 0.0],
        ObjectiveSense::Minimize,
    );
    let q = SparseColMat::try_new_from_triplets(
        2,
        2,
        &[
            Triplet::new(0, 0, 2.0),
            Triplet::new(0, 1, 1.0),
            Triplet::new(1, 0, 1.0),
            Triplet::new(1, 1, 2.0),
        ],
    )
    .unwrap();
    // Q = Lᵀ D L with L = [[1, 1/2], [0, 1]] and D = (2, 3/2), found by the exact LDLᵀ.
    let proof = crate::solver_tests::certify(&q, 1.0);
    qp.hessian = Some(q);
    let controls = Controls {
        threads,
        ..Controls::default()
    };
    let settings = clarabel(direct);
    execution::adapter(Backend::Clarabel)
        .admit_settings(&settings, &controls)
        .unwrap();
    let evidence: &dyn QuadraticEvidence = &proof;
    [(&lp, None), (&qp, Some(evidence))].map(|(p, evidence)| {
        let r = run(
            p,
            evidence,
            Backend::Clarabel,
            &settings,
            &controls,
            &scaled(p),
            &budgets(p, 1e-7),
        )
        .unwrap();
        assert_eq!(r.termination.category, Termination::Success, "{r:?}");
        assert!(r.quality.as_ref().unwrap().feasible(), "{r:?}");
        r
    })
}
/// A Clarabel MKL Pardiso solve on the worker's `threads`; loads the linked oneMKL's Pardiso.
#[cfg(feature = "clarabel-pardiso")]
pub(crate) fn solve_on_mkl_pardiso(threads: usize) -> [SolveReport; 2] {
    solve_both(Direct::MklPardiso, threads)
}

#[cfg(feature = "clarabel-pardiso")]
#[test]
fn clarabel_mkl_pardiso_matches_qdldl() {
    let qdldl = solve_both(Direct::Qdldl, 1);
    let pardiso = solve_on_mkl_pardiso(2);
    for (a, b) in qdldl.iter().zip(&pardiso) {
        let (x, y) = (a.candidate.as_ref().unwrap(), b.candidate.as_ref().unwrap());
        for (u, v) in x.primal.iter().zip(&y.primal) {
            near(*u, *v, 1e-7);
        }
        near(x.objective.unwrap(), y.objective.unwrap(), 1e-7);
        assert_eq!(a.metrics["linear.name"], Metric::Text("qdldl".into()));
        assert_eq!(b.metrics["linear.name"], Metric::Text("mkl".into()));
        assert!(b.provenance["native"].contains("MKL Pardiso"));
    }
    // min x² + xy + y² - x over x + y >= 1: on the active row it is (x - 1)², so x = (1, 0).
    let qp = pardiso[1].candidate.as_ref().unwrap();
    near(qp.primal[0], 1.0, 1e-7);
    near(qp.primal[1], 0.0, 1e-7);
    // QDLDL factorizes on one thread; MKL Pardiso admits more.
    let two = Controls {
        threads: 2,
        ..Controls::default()
    };
    let adapter = execution::adapter(Backend::Clarabel);
    assert!(matches!(
        adapter.admit_settings(&clarabel(Direct::Qdldl), &two),
        Err(ProblemError::Unsupported(_))
    ));
    adapter
        .admit_settings(&clarabel(Direct::MklPardiso), &two)
        .unwrap();
    assert!(adapter.capability().parallel);
}

/// Without the MKL Pardiso profile the setting is refused, and QDLDL stays serial.
#[cfg(not(feature = "clarabel-pardiso"))]
#[test]
fn clarabel_mkl_pardiso_refused_without_profile() {
    let adapter = execution::adapter(Backend::Clarabel);
    assert!(matches!(
        adapter.admit_settings(&clarabel(Direct::MklPardiso), &Controls::default()),
        Err(ProblemError::Unsupported(_))
    ));
    assert!(!adapter.capability().parallel);
    let [lp, _] = solve_both(Direct::Qdldl, 1);
    near(lp.candidate.unwrap().objective.unwrap(), 23.0, 1e-6);
}

/// min tr(CX) = 2 Σ X_ii - 2 Σ X_i,i+1 over tridiagonal X ⪰ 0 with tr X = 1: the chordal
/// decomposition splits the order-4 PSD cone into its three edge cliques and finds the same
/// optimum as the undecomposed cone. With v = (1, -1, 1, -1), vᵀXv = tr X - 2 Σ X_i,i+1 ≥ 0
/// bounds the off-diagonal sum by ½, which X = ¼[[1, 1], [1, 1]] ⊕ ¼[[1, 1], [1, 1]]
/// attains: the optimum is 1.
#[cfg(feature = "sdp")]
#[test]
fn clarabel_chordal_matches_undecomposed() {
    // Variables: the diagonal X_ii (0..4), then the off-diagonal X_i,i+1 (4..7).
    let diagonal = [0usize, 2, 5, 9];
    let offdiagonal = [1usize, 4, 8];
    let mut entries = Vec::new();
    for (j, row) in diagonal.iter().enumerate() {
        entries.push((0usize, j, 1.0));
        entries.push((1 + row, j, -1.0));
    }
    for (k, row) in offdiagonal.iter().enumerate() {
        entries.push((1 + row, 4 + k, -std::f64::consts::SQRT_2));
    }
    let mut column_starts = vec![0];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for c in 0..7 {
        let mut column: Vec<_> = entries.iter().filter(|e| e.1 == c).collect();
        column.sort_by_key(|e| e.0);
        for (r, _, v) in column {
            row_indices.push(*r);
            values.push(*v);
        }
        column_starts.push(row_indices.len());
    }
    let mut rhs = vec![0.0; 11];
    rhs[0] = 1.0;
    let p = ConicProblem {
        contract: OracleContract {
            identity: ContentHash::from_bytes([1; 32]),
            variables: (0..7)
                .map(|i| Variable {
                    id: id(1 + i),
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                })
                .collect(),
            rows: (0..11).map(|i| id(100 + i)).collect(),
            derivatives: pse_kernels::DerivativeOrder::Second,
            smoothness: pse_kernels::DerivativeOrder::Second,
        },
        quadratic: SparseMatrix::zeros(7, 7),
        objective: vec![2.0, 2.0, 2.0, 2.0, -2.0, -2.0, -2.0],
        constraints: SparseMatrix::new(11, 7, column_starts, row_indices, values),
        rhs,
        cones: vec![Cone::Zero { dimension: 1 }, Cone::PsdTriangle { order: 4 }],
        objective_constant: 0.0,
    };
    let zero = SparseColMat::try_new_from_triplets(7, 7, &[]).unwrap();
    let proof = crate::solver_tests::certify(&zero, 1.0);
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::nominal();
    let expected = 1.0;
    let solve = |chordal: bool| {
        let settings = conic::Settings {
            chordal_decomposition_enable: chordal,
            ..conic::Settings::default()
        };
        let mut session = conic::Session::new(
            &p,
            &proof,
            &controls,
            &accuracy,
            &settings,
            stamp(Backend::Clarabel),
        )
        .unwrap();
        let r = session
            .solve(
                &p,
                &controls,
                &accuracy,
                &settings,
                Execution::new(Arc::new(AtomicBool::new(false)), &controls),
                &Tolerances {
                    variables: vec![1e-6; 7],
                    rows: vec![1e-6; 11],
                    integrality: 1e-7,
                },
            )
            .unwrap();
        assert_eq!(r.termination.category, Termination::Success, "{r:?}");
        assert!(r.quality.as_ref().unwrap().feasible(), "{r:?}");
        r
    };
    let (decomposed, whole) = (solve(true), solve(false));
    let objective = |r: &SolveReport| r.candidate.as_ref().unwrap().objective.unwrap();
    near(objective(&decomposed), expected, 1e-6);
    near(objective(&whole), expected, 1e-6);
    // The decomposition changed the KKT system Clarabel factorized.
    assert_ne!(
        decomposed.metrics["linear.nnzA"],
        whole.metrics["linear.nnzA"]
    );
}
