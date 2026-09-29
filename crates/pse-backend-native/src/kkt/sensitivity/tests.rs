// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parametric sensitivity units (Plan 22 S1): an analytic NLP solved through the one NLP
//! runner, and negative controls read at exact candidates through the runner's tail
//! (original observation, KKT evidence, qualification, derivation).
use super::*;
use crate::{
    OracleContract, Variable,
    quality::Tolerances,
    solve::{
        Assurance, Backend, CandidateKind, Controls, Execution, NativeTermination,
        ResolvedAccuracy, Termination,
    },
    solver_tests::id,
};
use faer::sparse::{SparseColMat, Triplet};
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::normalization::Normalization;

/// `f = ½x₁² + ½c·x₂² − p₂x₁ + ½x₃² + p₁x₃` subject to `g = x₁ + x₂ − p₁ = 0`,
/// `h = x₁ − x₂ ≤ 10`, optionally the duplicate `2g = 0`, `x₁, x₂ ∈ [−10, 10]` and
/// `x₃ ∈ [0, 10]`. The solve view holds `p` fixed; the parametric view appends `p₁, p₂`
/// as unbounded columns.
///
/// With `c = 2` and `p = (1, 2)` the minimizer is `x = (4/3, −1/3, 0)` with `λ_g = 2/3`
/// and `z_L,3 = p₁ = 1`, and `x₁ = (p₂ + 2p₁)/3`, `x₂ = (p₁ − p₂)/3`,
/// `λ_g = 2(p₂ − p₁)/3`, `df*/dp = (−λ_g + x₃, −x₁)`, `d²f*/dp² = [[2/3, −2/3], [−2/3, −1/3]]`.
#[derive(Debug)]
struct Analytic {
    p: [f64; 2],
    c: f64,
    parametric: bool,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    jacobian: SparseColMat<usize, f64>,
    jacobian_values: Vec<f64>,
    hessian: SparseColMat<usize, f64>,
    hessian_values: Vec<f64>,
    normalization: Normalization,
}
/// Coordinate scales of the parametric view: `x₁, x₂, x₃, p₁, p₂`.
const SCALES: [f64; 5] = [2.0, 0.5, 1.0, 4.0, 0.25];
const OBJECTIVE_SCALE: f64 = 5.0;
impl Analytic {
    fn new(p: [f64; 2], c: f64, duplicate: bool, parametric: bool) -> Self {
        let n = if parametric { 5 } else { 3 };
        let mut variables: Vec<Variable> = [(-10.0, 10.0), (-10.0, 10.0), (0.0, 10.0)]
            .iter()
            .enumerate()
            .map(|(j, &(lower, upper))| Variable {
                id: id(40 + u8::try_from(j).unwrap()),
                lower,
                upper,
            })
            .collect();
        if parametric {
            variables.extend((0..2).map(|k| Variable {
                id: id(50 + k),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }));
        }
        let mut jacobian = vec![(0, 0, 1.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, -1.0)];
        let mut bounds = vec![(0.0, 0.0), (f64::NEG_INFINITY, 10.0)];
        if parametric {
            jacobian.push((0, 3, -1.0));
        }
        if duplicate {
            jacobian.extend([(2, 0, 2.0), (2, 1, 2.0)]);
            if parametric {
                jacobian.push((2, 3, -2.0));
            }
            bounds.push((0.0, 0.0));
        }
        let mut hessian = vec![(0, 0, 1.0), (1, 1, c), (2, 2, 1.0)];
        if parametric {
            hessian.extend([(3, 2, 1.0), (4, 0, -1.0)]);
        }
        let m = bounds.len();
        let (jacobian, jacobian_values) = csc(m, n, &jacobian);
        let (hessian, hessian_values) = csc(n, n, &hessian);
        Self {
            p,
            c,
            parametric,
            contract: OracleContract {
                identity: ContentHash::from_bytes([61; 32]),
                variables,
                rows: (0..m).map(|r| id(60 + u8::try_from(r).unwrap())).collect(),
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            bounds,
            jacobian,
            jacobian_values,
            hessian,
            hessian_values,
            normalization: Normalization {
                variables: SCALES[..n].to_vec(),
                rows: vec![3.0; m],
                objective: OBJECTIVE_SCALE,
            },
        }
    }
    fn parameters(&self, x: &[f64]) -> (f64, f64) {
        if self.parametric {
            (x[3], x[4])
        } else {
            (self.p[0], self.p[1])
        }
    }
}
/// A CSC pattern and its values in storage order.
fn csc(rows: usize, cols: usize, entries: &[(usize, usize, f64)]) -> (SparseColMat<usize, f64>, Vec<f64>) {
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|&(i, j, _)| (j, i));
    let triplets: Vec<_> = sorted.iter().map(|&(i, j, v)| Triplet::new(i, j, v)).collect();
    let matrix = SparseColMat::try_new_from_triplets(rows, cols, &triplets).unwrap();
    (matrix, sorted.iter().map(|&(_, _, v)| v).collect())
}
impl NlpOracle for Analytic {
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.normalization)
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jacobian.symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.hessian.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        let (p1, p2) = self.parameters(x);
        Ok(0.5 * x[0] * x[0] + 0.5 * self.c * x[1] * x[1] - p2 * x[0]
            + 0.5 * x[2] * x[2]
            + p1 * x[2])
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let (p1, _) = self.parameters(x);
        let g = x[0] + x[1] - p1;
        out[0] = g;
        out[1] = x[0] - x[1];
        if let Some(duplicate) = out.get_mut(2) {
            *duplicate = 2.0 * g;
        }
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let (p1, p2) = self.parameters(x);
        out[..3].copy_from_slice(&[x[0] - p2, self.c * x[1], x[2] + p1]);
        if self.parametric {
            out[3] = x[2];
            out[4] = -x[0];
        }
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&self.jacobian_values);
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        for (o, v) in out.iter_mut().zip(&self.hessian_values) {
            *o = weight * v;
        }
        Ok(())
    }
}

const P: [f64; 2] = [1.0, 2.0];
/// The analytic sensitivities at `P`: `dx/dp` by column, `dλ_g/dp`, `d z_L,3/dp`,
/// `df*/dp` and `d²f*/dp²` (row-major).
const DX: [[f64; 3]; 2] = [[2.0 / 3.0, 1.0 / 3.0, 0.0], [1.0 / 3.0, -1.0 / 3.0, 0.0]];
const DLAMBDA: [f64; 2] = [-2.0 / 3.0, 2.0 / 3.0];
const DZ: [f64; 2] = [1.0, 0.0];
const DF: [f64; 2] = [-2.0 / 3.0, -4.0 / 3.0];
const H: [f64; 4] = [2.0 / 3.0, -2.0 / 3.0, -2.0 / 3.0, -1.0 / 3.0];

fn request(parametric: Analytic, reduced_hessian: bool) -> Sensitivity {
    Sensitivity {
        parameters: vec![(id(50), parametric.p[0]), (id(51), parametric.p[1])],
        oracle: Box::new(parametric),
        reduced_hessian,
    }
}
fn tolerances(n: usize, m: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; m],
        integrality: 1e-8,
    }
}
fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1e-6 * (1.0 + expected.abs())
}
/// The runner's tail at an exact candidate: original observation, KKT evidence,
/// qualification and the requested derivation.
fn tail(
    mut solve: Analytic,
    parametric: Analytic,
    x: [f64; 3],
    lambda: &[f64],
    zl: [f64; 3],
) -> SolveReport {
    let (n, m) = (3, solve.bounds.len());
    let execution = Execution::new(Default::default(), &Controls::default());
    let mut report = SolveReport::new(
        Backend::Ipopt,
        &solve.contract,
        NativeTermination {
            code: 0,
            name: "synthetic".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::None,
        },
        &execution,
    );
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: x.to_vec(),
        objective: None,
        row_dual: Some(lambda.to_vec()),
        bound_dual: Some((zl.to_vec(), vec![0.0; n])),
        reduced_costs: None,
        slacks: None,
        commitment: None,
    });
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(n, m);
    let normalization = solve.normalization.clone();
    quality::attach_nlp(&mut report, &mut solve, &tolerances, ObjectiveSense::Minimize);
    quality::record_kkt(&mut report, &normalization, &accuracy);
    quality::qualify(&mut report, &accuracy);
    derive(
        &mut report,
        request(parametric, true),
        &tolerances,
        ObjectiveSense::Minimize,
        Budget {
            dual: accuracy.stationarity,
            limit: 10_000,
        },
    );
    report
}
fn withheld(report: &SolveReport) -> &Withheld {
    let parametric = report.evidence.sensitivity.as_ref().unwrap();
    // The reduced Hessian is withheld for the same reason.
    assert!(matches!(parametric.reduced_hessian, Some(Err(_))));
    match &parametric.sensitivities {
        Err(withheld) => withheld,
        Ok(_) => panic!("{parametric:?}"),
    }
}

#[test]
fn exact_candidate_sensitivities_match_the_analytic_nlp() {
    let report = tail(
        Analytic::new(P, 2.0, false, false),
        Analytic::new(P, 2.0, false, true),
        [4.0 / 3.0, -1.0 / 3.0, 0.0],
        &[2.0 / 3.0, 0.0],
        [0.0, 0.0, 1.0],
    );
    assert_eq!(report.qualification, Qualification::Stationary);
    let parametric = report.evidence.sensitivity.as_ref().unwrap();
    let s = parametric.sensitivities.as_ref().unwrap();
    for k in 0..2 {
        for j in 0..3 {
            assert!(close(s.primal[k][j], DX[k][j]), "{s:?}");
        }
        assert!(close(s.rows[k][0], DLAMBDA[k]) && s.rows[k][1] == 0.0, "{s:?}");
        assert!(close(s.bounds[k][2], DZ[k]), "{s:?}");
        assert!(close(s.objective[k], DF[k]), "{s:?}");
    }
    let hessian = parametric.reduced_hessian.as_ref().unwrap().as_ref().unwrap();
    for (actual, expected) in hessian.values.iter().zip(H) {
        assert!(close(*actual, expected), "{hessian:?}");
    }
}

#[test]
fn sensitivity_withheld_when_sosc_fails() {
    // c = −2: x = (0, 1, 0) with λ_g = 2 is a KKT point, but the null space of the active
    // gradients, along (1, −1, 0), has curvature 1 − 2 < 0: not a minimizer.
    let report = tail(
        Analytic::new(P, -2.0, false, false),
        Analytic::new(P, -2.0, false, true),
        [0.0, 1.0, 0.0],
        &[2.0, 0.0],
        [0.0, 0.0, 1.0],
    );
    assert_eq!(report.qualification, Qualification::Stationary);
    assert!(matches!(
        withheld(&report),
        Withheld::SecondOrder(Curvature::Negative)
    ));
    let point = report.evidence.sensitivity.as_ref().unwrap().point.as_ref().unwrap();
    assert_eq!(point.curvature, Curvature::Negative);
}

#[test]
fn sensitivity_withheld_when_weakly_active() {
    // p₁ = 0: x₃ = 0 at its bound with z_L,3 = p₁ = 0, strict complementarity fails.
    let p = [0.0, 2.0];
    let lambda = 2.0 * (p[1] - p[0]) / 3.0;
    let report = tail(
        Analytic::new(p, 2.0, false, false),
        Analytic::new(p, 2.0, false, true),
        [p[1] - lambda, -lambda / 2.0, 0.0],
        &[lambda, 0.0],
        [0.0; 3],
    );
    assert_eq!(report.qualification, Qualification::Stationary);
    assert!(matches!(
        withheld(&report),
        Withheld::WeaklyActive { count: 1 }
    ));
}

#[test]
fn sensitivity_withheld_when_licq_fails() {
    // The duplicate row 2g = 0 is active with g: the active gradients are dependent.
    let report = tail(
        Analytic::new(P, 2.0, true, false),
        Analytic::new(P, 2.0, true, true),
        [4.0 / 3.0, -1.0 / 3.0, 0.0],
        &[2.0 / 3.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    );
    assert_eq!(report.qualification, Qualification::Stationary);
    assert!(matches!(
        withheld(&report),
        Withheld::Licq { deficiency: 1 }
    ));
}

#[test]
fn sensitivity_withheld_when_multiplier_fails_complementarity() {
    // Negative control for F01 (ADR-0118 item 7): multipliers as a postsolve could recover
    // them for the wrong active set. The inactive row h = x₁ − x₂ ≤ 10 carries δ = 0.3;
    // stationarity holds with λ_g = (2(p₂ − p₁) − δ)/3, but δ·(10 − h) ≠ 0.
    let delta = 0.3;
    let lambda = (2.0 * (P[1] - P[0]) - delta) / 3.0;
    let x = [P[1] - lambda - delta, (delta - lambda) / 2.0, 0.0];
    let report = tail(
        Analytic::new(P, 2.0, false, false),
        Analytic::new(P, 2.0, false, true),
        x,
        &[lambda, delta],
        [0.0, 0.0, 1.0],
    );
    assert!(report.quality.as_ref().unwrap().feasible());
    let kkt = report.evidence.kkt.unwrap();
    assert_eq!((kkt.stationarity, kkt.complementarity), (Some(true), Some(false)));
    assert_eq!(report.qualification, Qualification::Feasible);
    assert!(matches!(withheld(&report), Withheld::Complementarity));
    assert!(report.evidence.sensitivity.as_ref().unwrap().point.is_none());
}

#[test]
fn pinned_columns_keep_their_coordinates() {
    use crate::transform::Pinned;
    let pinned = Pinned::new(Box::new(Analytic::new(P, 2.0, false, true)), &[(3, 1.0), (4, 2.0)])
        .unwrap();
    let variables = &pinned.contract().variables;
    assert_eq!(variables.len(), 5);
    assert_eq!((variables[3].lower, variables[3].upper), (1.0, 1.0));
    assert_eq!((variables[4].id, variables[4].lower), (id(51), 2.0));
    assert_eq!((variables[2].lower, variables[2].upper), (0.0, 10.0));
    // A pin narrows a box; it never widens one, repeats a column or leaves the contract.
    for pins in [
        &[(2, -1.0)][..],
        &[(3, 1.0), (3, 2.0)][..],
        &[(5, 0.0)][..],
        &[(3, f64::INFINITY)][..],
    ] {
        assert!(Pinned::new(Box::new(Analytic::new(P, 2.0, false, true)), pins).is_err());
    }
}

/// `Pinned::discrete` commits every integer-valued column at a member of its domain and
/// only committed columns (ADR-0118 item 9): x₃ ∈ [0, 10] is integer, x₂ ∈ [−10, 10]
/// semi-continuous and x₁ continuous.
#[test]
fn pinned_discrete_commits_integer_columns() {
    use crate::transform::{Commitment, Pinned, Relaxed};
    use pse_model::generated::enums::ModelingVariableDomain as D;
    let relaxed = || {
        Relaxed::new(
            Box::new(Analytic::new(P, 2.0, false, false)),
            vec![D::Continuous, D::Semicontinuous, D::Integer],
        )
        .unwrap()
    };
    let commit = |columns: &[(u8, f64)]| Commitment {
        columns: columns.iter().map(|(c, v)| (id(40 + c), *v)).collect(),
    };
    // x₃ committed at 3; the semi-continuous x₂ stays over its active interval.
    let pinned = Pinned::discrete(relaxed(), &commit(&[(2, 3.0)])).unwrap();
    let variables = &pinned.contract().variables;
    assert_eq!((variables[2].lower, variables[2].upper), (3.0, 3.0));
    assert_eq!((variables[1].lower, variables[1].upper), (-10.0, 10.0));
    // The semi-continuous off branch and a continuous column held at zero are pins too.
    let pinned = Pinned::discrete(relaxed(), &commit(&[(0, 0.0), (1, 0.0), (2, 3.0)])).unwrap();
    let variables = &pinned.contract().variables;
    assert_eq!((variables[0].lower, variables[1].upper), (0.0, 0.0));
    for columns in [
        // The integer column is free, off the lattice or outside its box.
        &[][..],
        &[(2, 2.5)][..],
        &[(2, 11.0)][..],
        // A continuous pin outside its box, a repeated and an unknown column.
        &[(2, 3.0), (0, 11.0)][..],
        &[(2, 3.0), (2, 4.0)][..],
        &[(2, 3.0), (9, 0.0)][..],
    ] {
        assert!(
            Pinned::discrete(relaxed(), &commit(columns)).is_err(),
            "{columns:?}"
        );
    }
}

#[test]
fn sensitivity_request_admission() {
    let solve = Analytic::new(P, 2.0, false, false);
    assert!(request(Analytic::new(P, 2.0, false, true), false)
        .admit(&solve.contract)
        .is_ok());
    // The parametric view must extend the solve's columns over the same rows.
    assert!(request(Analytic::new(P, 2.0, true, true), false)
        .admit(&solve.contract)
        .is_err());
    let mut repeated = request(Analytic::new(P, 2.0, false, true), false);
    repeated.parameters[1].0 = id(50);
    assert!(repeated.admit(&solve.contract).is_err());
    let mut nonfinite = request(Analytic::new(P, 2.0, false, true), false);
    nonfinite.parameters[0].1 = f64::NAN;
    assert!(nonfinite.admit(&solve.contract).is_err());
}

#[cfg(any(feature = "ipopt", feature = "pounce"))]
mod solved {
    use super::*;
    use crate::{
        execution::{self, BackendSettings, LINKED, Nlp, Retained, Step},
        kkt::Analysis,
        presolve::Policy,
        solve::Compatibility,
    };

    /// One solve of the analytic NLP through the one NLP runner with a sensitivity request.
    pub(super) fn solve(backend: Backend, p: [f64; 2], sense: ObjectiveSense) -> SolveReport {
        run(
            backend,
            p,
            sense,
            Analysis {
                second_order: true,
                sensitivity: Some(request(Analytic::new(p, 2.0, false, true), true)),
                inverse_reduced_hessian: None,
            },
        )
        .unwrap()
    }
    /// One solve of the analytic NLP through the one NLP runner with `analysis`.
    fn run(
        backend: Backend,
        p: [f64; 2],
        sense: ObjectiveSense,
        analysis: Analysis,
    ) -> Result<SolveReport, ProblemError> {
        let solve = Analytic::new(p, 2.0, false, false);
        let (n, m) = (3, solve.bounds.len());
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-9).unwrap();
        let normalization = solve.normalization.clone();
        let tolerances = tolerances(n, m);
        execution::nlp(
            Step {
                adapter: LINKED.get(backend).unwrap(),
                settings: &BackendSettings::Default,
                controls: &controls,
                accuracy: &accuracy,
                execution: Execution::new(Default::default(), &controls),
                tolerances: &tolerances,
                normalization: &normalization,
                compatibility: Compatibility {
                    layout: ContentHash::from_bytes([2; 32]),
                    profile: ContentHash::from_bytes([4; 32]),
                    data: ContentHash::from_bytes([3; 32]),
                    backend,
                },
                warm: None,
            },
            &mut Retained::default(),
            Nlp {
                oracle: Box::new(solve),
                initial: &[0.5, 0.5, 0.5],
                presolve: &Policy::Off,
                intent: crate::solve::SolveIntent::Optimize,
                sense,
                limit: 1 << 20,
                analysis,
            },
        )
    }

    /// Plan 22 S3: `B·K⁻¹·Bᵀ` over the solve's own columns is the inverse reduced Hessian.
    /// At `P` the row `g` and `x₃`'s lower bound are active, the null space of their
    /// gradients is spanned by `z = (1, −1, 0)` with `zᵀHz = 1 + c = 3`, so
    /// `[K⁻¹]ₓₓ = z·zᵀ/3`: `x₃`, held by its active bound, has none.
    #[test]
    fn inverse_reduced_hessian_over_solve_columns() {
        let columns: Vec<OriginalCol> = (0..3).map(OriginalCol::new).collect();
        let inverse = |second_order| {
            run(
                Backend::Ipopt,
                P,
                ObjectiveSense::Minimize,
                Analysis {
                    second_order,
                    sensitivity: None,
                    inverse_reduced_hessian: Some(columns.clone()),
                },
            )
        };
        let report = inverse(true).unwrap();
        assert_eq!(report.qualification, Qualification::Stationary);
        let block = report
            .evidence
            .inverse_reduced_hessian
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        let z = [1.0, -1.0, 0.0];
        for i in 0..3 {
            for j in 0..3 {
                assert!(close(block.values[3 * i + j], z[i] * z[j] / 3.0), "{block:?}");
                let normalized = block.values[3 * i + j] * OBJECTIVE_SCALE / (SCALES[i] * SCALES[j]);
                assert!(close(block.normalized[3 * i + j], normalized), "{block:?}");
            }
        }
        assert_eq!(block.columns, columns);
        // It is read from the step's own analysis, over distinct columns of the solve.
        assert!(matches!(inverse(false), Err(ProblemError::Contract(_))));
        let repeated = run(
            Backend::Ipopt,
            P,
            ObjectiveSense::Minimize,
            Analysis {
                second_order: true,
                sensitivity: None,
                inverse_reduced_hessian: Some(vec![OriginalCol::new(0), OriginalCol::new(0)]),
            },
        );
        assert!(matches!(repeated, Err(ProblemError::Contract(_))));
    }

    #[test]
    fn sensitivity_matches_analytic_nlp() {
        for backend in [Backend::Ipopt, Backend::Pounce] {
            if LINKED.get(backend).is_none() {
                continue;
            }
            let report = solve(backend, P, ObjectiveSense::Minimize);
            assert_eq!(report.qualification, Qualification::Stationary, "{backend:?}");
            let parametric = report.evidence.sensitivity.as_ref().unwrap();
            assert_eq!(parametric.parameters, vec![id(50), id(51)]);
            let s = parametric.sensitivities.as_ref().unwrap();
            for k in 0..2 {
                for j in 0..3 {
                    assert!(close(s.primal[k][j], DX[k][j]), "{backend:?} {s:?}");
                }
                assert!(close(s.rows[k][0], DLAMBDA[k]), "{backend:?} {s:?}");
                assert!(close(s.bounds[k][2], DZ[k]), "{backend:?} {s:?}");
                assert!(close(s.objective[k], DF[k]), "{backend:?} {s:?}");
            }
            // The pinned parametric point: the row, x₃'s bound and both pins are strongly
            // active, In(K) = In(ZᵀHZ) + (4, 4, 0) over a one-dimensional null space.
            let point = parametric.point.as_ref().unwrap();
            assert_eq!(
                point.bounds[OriginalCol::new(3)],
                crate::kkt::Activity::Strong(Side::Equal)
            );
            assert_eq!((point.inertia, point.reduced), ((5, 4, 0), (1, 0, 0)));
            // A first-order step predicts the solution at a perturbed parameter.
            let moved = solve(backend, [P[0] + 1e-3, P[1] - 2e-3], ObjectiveSense::Minimize);
            let (x0, x1) = (
                &report.candidate.as_ref().unwrap().primal,
                &moved.candidate.as_ref().unwrap().primal,
            );
            for j in 0..3 {
                let predicted = x0[j] + 1e-3 * s.primal[0][j] - 2e-3 * s.primal[1][j];
                assert!((predicted - x1[j]).abs() < 1e-7, "{backend:?} {predicted} {}", x1[j]);
            }
        }
    }

    #[test]
    fn reduced_hessian_sign_pinned() {
        // gh#937: over the pin rows the Schur reduction yields −H_R; the reported value is
        // d²f*/dp² itself, in the authored sense. H is indefinite and differs from −H and
        // from H⁻¹ = [[1/2, −1], [−1, −1]] entry by entry, so a sign or inversion slip fails.
        let report = solve(Backend::Ipopt, P, ObjectiveSense::Minimize);
        let parametric = report.evidence.sensitivity.as_ref().unwrap();
        let hessian = parametric.reduced_hessian.as_ref().unwrap().as_ref().unwrap();
        for (actual, expected) in hessian.values.iter().zip(H) {
            assert!(close(*actual, expected), "{hessian:?}");
        }
        assert!(hessian.values[3] < 0.0 && hessian.values[0] > 0.0);
        // Ĥ = S_p·H·S_p / S_f under the declared scales, with its eigenpairs ascending.
        let s = [SCALES[3], SCALES[4]];
        for i in 0..2 {
            for j in 0..2 {
                let expected = s[i] * H[2 * i + j] * s[j] / OBJECTIVE_SCALE;
                assert!(close(hessian.normalized[2 * i + j], expected), "{hessian:?}");
            }
        }
        assert!(hessian.eigenvalues[0] < 0.0 && hessian.eigenvalues[0] < hessian.eigenvalues[1]);
        for k in 0..2 {
            let v = &hessian.eigenvectors[2 * k..2 * k + 2];
            for i in 0..2 {
                let product: f64 = (0..2).map(|j| hessian.normalized[2 * i + j] * v[j]).sum();
                assert!((product - hessian.eigenvalues[k] * v[i]).abs() < 1e-9, "{hessian:?}");
            }
        }
        // The same minimizer reported as a maximization of −f: every derived quantity is
        // in the authored sense, so df*/dp and the reduced Hessian change sign.
        let maximized = solve(Backend::Ipopt, P, ObjectiveSense::Maximize);
        let flipped = maximized.evidence.sensitivity.as_ref().unwrap();
        let s = flipped.sensitivities.as_ref().unwrap();
        assert!(close(s.objective[0], -DF[0]) && close(s.objective[1], -DF[1]));
        let hessian = flipped.reduced_hessian.as_ref().unwrap().as_ref().unwrap();
        for (actual, expected) in hessian.values.iter().zip(H) {
            assert!(close(*actual, -expected), "{hessian:?}");
        }
    }
}
