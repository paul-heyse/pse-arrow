// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{OracleContract, Variable, quality::Observation, solve::CandidateKind};
use faer::sparse::{SparseColMat, Triplet};
use pounce_sens_core::DenseLuBacksolver;

/// `f = ½ xᵀQx` with linear rows `g = A x`; `q` holds the lower triangle of `Q`.
#[derive(Debug)]
struct Dense {
    contract: OracleContract,
    q: Vec<(usize, usize, f64)>,
    a: Vec<(usize, usize, f64)>,
    rows: Vec<(f64, f64)>,
    jacobian: SparseColMat<usize, f64>,
    hessian: Option<SparseColMat<usize, f64>>,
}
impl Dense {
    fn new(
        bounds: &[(f64, f64)],
        q: &[(usize, usize, f64)],
        a: &[(usize, usize, f64)],
        rows: &[(f64, f64)],
    ) -> Self {
        let n = bounds.len();
        let pattern = |entries: &[(usize, usize, f64)], nrows| {
            let triplets: Vec<_> = entries
                .iter()
                .map(|&(i, j, _)| Triplet::new(i, j, 1.0))
                .collect();
            SparseColMat::try_new_from_triplets(nrows, n, &triplets).unwrap()
        };
        let sort = |entries: &[(usize, usize, f64)]| {
            // Storage order of a CSC pattern: by column, then row.
            let mut sorted = entries.to_vec();
            sorted.sort_by_key(|&(i, j, _)| (j, i));
            sorted
        };
        Self {
            contract: OracleContract {
                identity: pse_ids::ContentHash::from_bytes([8; 32]),
                variables: bounds
                    .iter()
                    .enumerate()
                    .map(|(i, &(lower, upper))| Variable {
                        id: pse_ids::SemanticId::from_bytes([40 + i as u8; 16]),
                        lower,
                        upper,
                    })
                    .collect(),
                rows: (0..rows.len())
                    .map(|r| pse_ids::SemanticId::from_bytes([60 + r as u8; 16]))
                    .collect(),
                derivatives: pse_kernels::DerivativeOrder::Second,
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            jacobian: pattern(a, rows.len()),
            hessian: Some(pattern(q, n)),
            q: sort(q),
            a: sort(a),
            rows: rows.to_vec(),
        }
    }
    fn values(&self, x: &[f64]) -> Vec<f64> {
        let mut g = vec![0.0; self.rows.len()];
        for &(r, j, v) in &self.a {
            g[r] += v * x[j];
        }
        g
    }
}
impl NlpOracle for Dense {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jacobian.symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.hessian.as_ref().map(|h| h.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.rows
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(self
            .q
            .iter()
            .map(|&(i, j, v)| if i == j { 0.5 * v * x[i] * x[i] } else { v * x[i] * x[j] })
            .sum())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&self.values(x));
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.0);
        for &(i, j, v) in &self.q {
            out[i] += v * x[j];
            if i != j {
                out[j] += v * x[i];
            }
        }
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (o, (_, _, v)) in out.iter_mut().zip(&self.a) {
            *o = *v;
        }
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        for (o, (_, _, v)) in out.iter_mut().zip(&self.q) {
            *o = weight * v;
        }
        Ok(())
    }
}

/// A candidate at `x` with the given multipliers, observed against `oracle`'s rows.
fn at(
    oracle: &Dense,
    x: &[f64],
    lambda: &[f64],
    (zl, zu): (&[f64], &[f64]),
) -> (Candidate, Observation) {
    let candidate = Candidate {
        kind: CandidateKind::FinalIterate,
        primal: x.to_vec(),
        objective: Some(0.0),
        row_dual: Some(lambda.to_vec()),
        bound_dual: Some((zl.to_vec(), zu.to_vec())),
        reduced_costs: None,
        slacks: None,
    };
    let observation =
        Observation::from_values(Some(0.0), oracle.values(x), oracle.rows.clone()).unwrap();
    (candidate, observation)
}
fn tolerances(n: usize, m: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; m],
        integrality: 1e-8,
    }
}
const BUDGET: Budget = Budget {
    dual: 1e-9,
    limit: 1000,
};
fn run(
    oracle: &mut Dense,
    x: &[f64],
    lambda: &[f64],
    z: (&[f64], &[f64]),
    normalization: &Normalization,
) -> Result<(KktPoint, KktFactor), Unavailable> {
    let (candidate, observation) = at(oracle, x, lambda, z);
    let (n, m) = (x.len(), lambda.len());
    analyse(
        oracle,
        &candidate,
        &observation,
        normalization,
        &tolerances(n, m),
        BUDGET,
    )
}
/// The analysis of `½ xᵀ diag(q) x` over two variables and the inactive row `x₀ + x₁`, at
/// the stationary point `x = 0` with zero multipliers.
fn at_origin(q: [f64; 2], bounds: [(f64, f64); 2]) -> KktPoint {
    let mut oracle = Dense::new(
        &bounds,
        &[(0, 0, q[0]), (1, 1, q[1])],
        &[(0, 0, 1.0), (0, 1, 1.0)],
        &[(-10.0, 10.0)],
    );
    run(
        &mut oracle,
        &[0.0; 2],
        &[0.0],
        (&[0.0; 2], &[0.0; 2]),
        &Normalization::identity(2, 1),
    )
    .unwrap()
    .0
}

#[test]
fn second_order_verdicts_follow_the_inertia() {
    let free = [(-1.0, 1.0); 2];
    // A saddle: stationary, not a minimizer.
    let saddle = at_origin([2.0, -2.0], free);
    assert_eq!(saddle.curvature, Curvature::Negative);
    assert_eq!((saddle.active(), saddle.inertia), (0, (1, 1, 0)));
    assert_eq!(saddle.rows[OriginalRow::new(0)], Activity::Inactive);
    // A flat direction: the reduced Hessian is singular.
    assert_eq!(at_origin([2.0, 0.0], free).curvature, Curvature::Singular);
    // A weakly active bound (zero multiplier) under positive curvature: the released test
    // on the larger subspace still certifies sufficiency.
    let weak = at_origin([2.0, 2.0], [(-1.0, 1.0), (0.0, 1.0)]);
    assert_eq!(weak.curvature, Curvature::Sufficient);
    assert_eq!(weak.weakly_active(), 1);
    assert_eq!(weak.bounds[OriginalCol::new(1)], Activity::Weak(Side::Lower));
    // The bound stays a row of the KKT: In(K) = In(ZᵀHZ) + (1, 1, 0).
    assert_eq!((weak.inertia, weak.reduced), ((2, 1, 0), (1, 0, 0)));
    // A weakly active bound with negative curvature along it: neither test decides.
    let open = at_origin([2.0, -2.0], [(-1.0, 1.0), (0.0, 1.0)]);
    assert_eq!(open.curvature, Curvature::Undecided);
    // A pinned variable is a strongly active bound row.
    let fixed = at_origin([2.0, -2.0], [(-1.0, 1.0), (0.0, 0.0)]);
    assert_eq!(fixed.curvature, Curvature::Sufficient);
    assert_eq!(fixed.active(), 1);
    assert_eq!(fixed.bounds[OriginalCol::new(1)], Activity::Strong(Side::Equal));
    assert_eq!(fixed.licq, Licq::Independent);
}

#[test]
fn licq_failure_distinct_from_singular_curvature() {
    // Two equality rows with parallel gradients over positive curvature: the active set is
    // dependent, and the reduced Hessian on {x₀ + x₁ = 0} is still positive definite.
    let mut dependent = Dense::new(
        &[(-1.0, 1.0); 3],
        &[(0, 0, 1.0), (1, 1, 1.0), (2, 2, 1.0)],
        &[(0, 0, 1.0), (0, 1, 1.0), (1, 0, 2.0), (1, 1, 2.0)],
        &[(0.0, 0.0), (0.0, 0.0)],
    );
    let (point, _) = run(
        &mut dependent,
        &[0.0; 3],
        &[0.0; 2],
        (&[0.0; 3], &[0.0; 3]),
        &Normalization::identity(3, 2),
    )
    .unwrap();
    assert_eq!(point.licq, Licq::Dependent { deficiency: 1 });
    assert_eq!(point.curvature, Curvature::Sufficient);
    // In(K) = (2, 0, 0) + (r, r, a − r) with r = 1, a = 2.
    assert_eq!((point.inertia, point.reduced), ((3, 1, 1), (2, 0, 0)));
    // One independent equality row over a flat direction: the gradients are independent
    // and the reduced Hessian is singular.
    let mut flat = Dense::new(
        &[(-1.0, 1.0); 2],
        &[(0, 0, 1.0), (1, 1, 0.0)],
        &[(0, 0, 1.0)],
        &[(0.0, 0.0)],
    );
    let (point, _) = run(
        &mut flat,
        &[0.0; 2],
        &[0.0],
        (&[0.0; 2], &[0.0; 2]),
        &Normalization::identity(2, 1),
    )
    .unwrap();
    assert_eq!(point.licq, Licq::Independent);
    assert_eq!(point.curvature, Curvature::Singular);
    assert_eq!(point.reduced, (0, 0, 1));
}

#[test]
fn kkt_factor_backsolve_matches_dense_reference() {
    // Q is positive definite; row 0 is active at its lower limit and x₂ at its lower bound;
    // row 1 is inactive. Nonunit scales exercise the normalization back-map.
    let q = [(0, 0, 4.0), (1, 0, 1.0), (1, 1, 3.0), (2, 2, 2.0)];
    let a = [(0, 0, 1.0), (0, 1, 2.0), (0, 2, -1.0), (1, 1, 1.0), (1, 2, 1.0)];
    let mut oracle = Dense::new(
        &[(-5.0, 5.0), (-5.0, 5.0), (0.0, 5.0)],
        &q,
        &a,
        &[(1.0, 5.0), (f64::NEG_INFINITY, 10.0)],
    );
    let normalization = Normalization {
        variables: vec![2.0, 0.5, 10.0],
        rows: vec![3.0, 7.0],
        objective: 4.0,
    };
    let (point, factor) = run(
        &mut oracle,
        &[1.0, 0.0, 0.0],
        &[-1.5, 0.0],
        (&[0.0, 0.0, 0.8], &[0.0; 3]),
        &normalization,
    )
    .unwrap();
    assert_eq!(point.rows[OriginalRow::new(0)], Activity::Strong(Side::Lower));
    assert_eq!(point.rows[OriginalRow::new(1)], Activity::Inactive);
    assert_eq!(point.bounds[OriginalCol::new(2)], Activity::Strong(Side::Lower));
    assert_eq!((point.licq, point.curvature), (Licq::Independent, Curvature::Sufficient));
    assert_eq!(point.inertia, (3, 2, 0));
    assert!(point.residual.is_some_and(|r| r < 1e-12), "{:?}", point.residual);
    assert!(point.condition_1norm.is_some_and(|c| c >= 1.0));
    // Layout [x; row 0; bound on x₂], and the bound row names its variable and side.
    let layout = factor.layout();
    assert_eq!(layout.dim(), 5);
    assert_eq!(layout.row(OriginalRow::new(0)), Some(3));
    assert_eq!(layout.row(OriginalRow::new(1)), None);
    assert_eq!(layout.bound(OriginalCol::new(2)), Some(4));
    assert_eq!(
        factor.bound_rows(),
        Some(
            &[BoundRow {
                row: 4,
                var_row: 2,
                lower: true
            }][..]
        )
    );
    // The original-coordinate KKT matrix: [Q Aᵀ; A 0] over row 0 and the bound row −e₂.
    let mut k = [[0.0; 5]; 5];
    for &(i, j, v) in &q {
        k[i][j] = v;
        k[j][i] = v;
    }
    for (row, gradient) in [(3, [1.0, 2.0, -1.0]), (4, [0.0, 0.0, -1.0])] {
        for (j, v) in gradient.into_iter().enumerate() {
            k[row][j] = v;
            k[j][row] = v;
        }
    }
    let dense = DenseLuBacksolver::from_dense(5, k.as_flattened()).unwrap();
    for rhs in [
        [1.0, 0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 0.0, 1.0],
        [0.3, -2.0, 7.5, 1e-3, -4.0],
    ] {
        let (mut expected, mut actual) = ([0.0; 5], [0.0; 5]);
        assert!(dense.solve(&rhs, &mut expected));
        assert!(factor.solve(&rhs, &mut actual));
        for (e, a) in expected.iter().zip(&actual) {
            assert!((e - a).abs() <= 1e-10 * (1.0 + e.abs()), "{expected:?} {actual:?}");
        }
    }
    // Clones share the factor, as the sensitivity machinery requires.
    let shared = factor.clone();
    let (mut first, mut second) = ([0.0; 5], [0.0; 5]);
    assert!(factor.solve(&[1.0; 5], &mut first) && shared.solve(&[1.0; 5], &mut second));
    assert_eq!(first, second);
    assert!(!factor.solve(&[1.0; 4], &mut [0.0; 4]));
}

#[test]
fn local_analysis_unavailable_is_typed() {
    let mut oracle = Dense::new(&[(-1.0, 1.0); 2], &[(0, 0, 1.0), (1, 1, 1.0)], &[(0, 0, 1.0)], &[(0.0, 0.0)]);
    let (mut candidate, observation) = at(&oracle, &[0.0; 2], &[0.0], (&[0.0; 2], &[0.0; 2]));
    let normalization = Normalization::identity(2, 1);
    let tolerances = tolerances(2, 1);
    // Above the entry ceiling: 3 diagonals, 2 Hessian entries and 1 coupling.
    let limited = Budget {
        dual: 1e-9,
        limit: 5,
    };
    assert!(matches!(
        analyse(&mut oracle, &candidate, &observation, &normalization, &tolerances, limited),
        Err(Unavailable::Limit {
            entries: 6,
            limit: 5
        })
    ));
    candidate.row_dual = None;
    assert!(matches!(
        analyse(&mut oracle, &candidate, &observation, &normalization, &tolerances, BUDGET),
        Err(Unavailable::Multipliers)
    ));
    candidate.row_dual = Some(vec![0.0]);
    oracle.hessian = None;
    assert!(matches!(
        analyse(&mut oracle, &candidate, &observation, &normalization, &tolerances, BUDGET),
        Err(Unavailable::Hessian)
    ));
}

#[cfg(all(feature = "ipopt", feature = "pounce"))]
#[test]
fn kkt_inertia_certifies_second_order() {
    use crate::{
        execution::BackendSettings,
        presolve::Policy,
        restart_tests::{Simplex, TARGET, run},
        solve::Backend,
    };
    for (backend, settings) in [
        (Backend::Ipopt, BackendSettings::Ipopt(Default::default())),
        (Backend::Pounce, BackendSettings::Pounce(Default::default())),
    ] {
        // The projection onto the simplex keeps three coordinates positive: the simplex row
        // and three lower bounds are active with positive multipliers, and every bound stays
        // a row, so In(K) = In(ZᵀHZ) + (4, 4, 0) = (6, 4, 0) with ZᵀHZ = 2·I/S_f on the
        // two-dimensional null space.
        let report = run(
            backend,
            &settings,
            Simplex::new(&TARGET, 1.0, 4.0),
            None,
            &Policy::Auto,
        );
        let point = match report.evidence.local {
            Some(Ok(point)) => point,
            other => panic!("{backend:?}: {other:?}"),
        };
        assert_eq!(point.curvature, Curvature::Sufficient, "{backend:?}");
        assert_eq!(point.licq, Licq::Independent, "{backend:?}");
        assert_eq!((point.active(), point.weakly_active()), (4, 0), "{backend:?}");
        assert_eq!(point.rows[OriginalRow::new(0)], Activity::Strong(Side::Equal));
        assert_eq!(
            point
                .bounds
                .iter()
                .filter(|a| **a == Activity::Strong(Side::Lower))
                .count(),
            3,
            "{backend:?}"
        );
        assert_eq!(point.inertia, (6, 4, 0), "{backend:?}");
        assert_eq!(point.reduced, (2, 0, 0), "{backend:?}");
        assert!(point.condition_1norm.is_some_and(|c| c >= 1.0));
        assert!(point.residual.is_some_and(|r| r < 1e-12), "{:?}", point.residual);
    }
}

#[cfg(feature = "ipopt")]
#[test]
fn second_order_analysis_needs_an_optimizing_intent() {
    use crate::{
        execution::{self, BackendSettings, LINKED, Nlp, Retained, Step},
        presolve::Policy,
        restart_tests::{Simplex, TARGET},
        solve::{Backend, Compatibility, Controls, Execution, ResolvedAccuracy},
    };
    let oracle = Simplex::new(&TARGET, 1.0, 1.0);
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = crate::restart_tests::tolerances(6);
    let error = execution::nlp(
        Step {
            adapter: LINKED.get(Backend::Ipopt).unwrap(),
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: &accuracy,
            execution: Execution::new(Default::default(), &controls),
            tolerances: &tolerances,
            normalization: &Normalization::identity(6, 1),
            compatibility: Compatibility {
                layout: pse_ids::ContentHash::from_bytes([2; 32]),
                profile: pse_ids::ContentHash::from_bytes([4; 32]),
                data: pse_ids::ContentHash::from_bytes([3; 32]),
                backend: Backend::Ipopt,
            },
            warm: None,
        },
        &mut Retained::default(),
        Nlp {
            oracle: Box::new(oracle),
            initial: &[1.0 / 6.0; 6],
            presolve: &Policy::Off,
            intent: SolveIntent::FeasiblePoint,
            sense: pse_math::binding::ObjectiveSense::Minimize,
            limit: 1 << 20,
            analysis: Analysis {
                second_order: true,
                sensitivity: None,
                inverse_reduced_hessian: None,
            },
        },
    )
    .unwrap_err();
    assert!(matches!(error, ProblemError::Contract(_)), "{error:?}");
}
