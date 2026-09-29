// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Single-run measurements of the large-KKT linear solvers (Plan 22 Q1). Not a benchmark
//! campaign: each solve runs once and prints its wall time, so an order-of-magnitude
//! difference is visible and a close one is not claimed.
//!
//! The problem is a nonlinear control on an `m × m` grid: minimize `‖x − t‖² + α‖u‖²`
//! subject to `4x_r − Σ x_neighbours + h²x_r³ = h²u_r` at every interior node, so the
//! Hessian is diagonal and the Jacobian carries the five-point stencil. With `m = 100` the
//! KKT matrix has `n_KKT = 3m² = 30 000`; `PSE_MEASURE_GRID` sets another `m`. The NLP
//! runs on Ipopt with MUMPS (METIS), SPRAL and MKL Pardiso, and on POUNCE with FERAL; the
//! linearized problem (no cubic term) runs on Clarabel with QDLDL and with MKL Pardiso on
//! several thread counts.
//!
//! Run on demand: `just native-test -E 'test(/measure_/)' --run-ignored only --no-capture`.
use crate::{
    CoefficientProblem, NlpOracle, OracleContract, ProblemError, Variable,
    conic::Direct,
    execution::{self, BackendSettings, LINKED, Nlp, Retained, Step},
    presolve::Policy,
    quality::Tolerances,
    settings::ipopt::{
        Linear, MumpsOrdering, PardisoMatching, PardisoOrdering, SpralOrdering, SpralPivot,
        SpralScaling,
    },
    solve::*,
};
use faer::sparse::{SparseColMat, Triplet};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{binding::ObjectiveSense, normalization::Normalization};
use pse_model::generated::enums::ModelingVariableDomain;
use std::time::Instant;

const ALPHA: f64 = 1e-2;

/// The grid side `m`: `PSE_MEASURE_GRID`, else 100, read once.
fn side() -> usize {
    static SIDE: std::sync::LazyLock<usize> = std::sync::LazyLock::new(|| {
        std::env::var("PSE_MEASURE_GRID")
            .ok()
            .and_then(|m| m.parse().ok())
            .unwrap_or(100)
    });
    *SIDE
}

fn identity(tag: u8, index: usize) -> SemanticId {
    let mut bytes = [tag; 16];
    bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
    SemanticId::from_bytes(bytes)
}
/// The grid's interior nodes, row-major, and each node's neighbours.
fn neighbours(r: usize) -> impl Iterator<Item = usize> {
    let m = side();
    let (i, j) = (r / m, r % m);
    [
        (i > 0).then(|| r - m),
        (i + 1 < m).then(|| r + m),
        (j > 0).then(|| r - 1),
        (j + 1 < m).then(|| r + 1),
    ]
    .into_iter()
    .flatten()
}
fn target(r: usize) -> f64 {
    let m = side();
    let (i, j) = ((r / m) as f64 + 1.0, (r % m) as f64 + 1.0);
    let s = std::f64::consts::PI / (side() as f64 + 1.0);
    (s * i).sin() * (s * j).sin()
}
/// The stencil's `(row, column, value)` entries for the `x` block and `−h²` on `u`;
/// `cubic` adds the diagonal `3h²x²` at `x`.
fn stencil(x: Option<&[f64]>) -> Vec<(usize, usize, f64)> {
    let n = side() * side();
    let h2 = (1.0 / (side() as f64 + 1.0)).powi(2);
    let mut entries = Vec::with_capacity(6 * n);
    for r in 0..n {
        entries.push((r, r, 4.0 + x.map_or(0.0, |x| 3.0 * h2 * x[r] * x[r])));
        entries.extend(neighbours(r).map(|c| (r, c, -1.0)));
        entries.push((r, n + r, -h2));
    }
    entries
}
fn csc(rows: usize, columns: usize, entries: &[(usize, usize, f64)]) -> SparseColMat<usize, f64> {
    SparseColMat::try_new_from_triplets(
        rows,
        columns,
        &entries
            .iter()
            .map(|&(r, c, v)| Triplet::new(r, c, v))
            .collect::<Vec<_>>(),
    )
    .unwrap()
}
fn contract() -> OracleContract {
    let n = side() * side();
    OracleContract {
        identity: ContentHash::from_bytes([11; 32]),
        variables: (0..2 * n)
            .map(|k| Variable {
                id: identity(21, k),
                lower: if k < n { f64::NEG_INFINITY } else { -50.0 },
                upper: if k < n { f64::INFINITY } else { 50.0 },
            })
            .collect(),
        rows: (0..n).map(|r| identity(22, r)).collect(),
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    }
}

/// The nonlinear grid control as an NLP oracle.
#[derive(Debug)]
struct Grid {
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    jacobian: SparseColMat<usize, f64>,
    hessian: SparseColMat<usize, f64>,
    normalization: Normalization,
}
impl Grid {
    fn new() -> Self {
        let n = side() * side();
        Self {
            contract: contract(),
            bounds: vec![(0.0, 0.0); n],
            jacobian: csc(n, 2 * n, &stencil(Some(&vec![0.0; n]))),
            hessian: csc(
                2 * n,
                2 * n,
                &(0..2 * n).map(|k| (k, k, 1.0)).collect::<Vec<_>>(),
            ),
            normalization: Normalization {
                variables: vec![1.0; 2 * n],
                rows: vec![1.0; n],
                objective: 1.0,
            },
        }
    }
}
impl NlpOracle for Grid {
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
    fn objective(&mut self, v: &[f64]) -> Result<f64, ProblemError> {
        let n = side() * side();
        Ok((0..n)
            .map(|r| (v[r] - target(r)).powi(2) + ALPHA * v[n + r] * v[n + r])
            .sum())
    }
    fn constraints(&mut self, v: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let n = side() * side();
        let h2 = (1.0 / (side() as f64 + 1.0)).powi(2);
        for (r, o) in out.iter_mut().enumerate() {
            *o = 4.0 * v[r] - neighbours(r).map(|c| v[c]).sum::<f64>() + h2 * v[r].powi(3)
                - h2 * v[n + r];
        }
        Ok(())
    }
    fn gradient(&mut self, v: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let n = side() * side();
        for r in 0..n {
            out[r] = 2.0 * (v[r] - target(r));
            out[n + r] = 2.0 * ALPHA * v[n + r];
        }
        Ok(())
    }
    fn jacobian(&mut self, v: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let values = csc(
            side() * side(),
            2 * side() * side(),
            &stencil(Some(&v[..side() * side()])),
        );
        out.copy_from_slice(values.val());
        Ok(())
    }
    fn hessian(
        &mut self,
        v: &[f64],
        weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let n = side() * side();
        let h2 = (1.0 / (side() as f64 + 1.0)).powi(2);
        for r in 0..n {
            out[r] = 2.0 * weight + multipliers[r] * 6.0 * h2 * v[r];
            out[n + r] = 2.0 * ALPHA * weight;
        }
        Ok(())
    }
}

fn tolerances(variables: usize, rows: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-8; variables],
        rows: vec![1e-8; rows],
        integrality: 1e-8,
    }
}
/// One NLP solve through the shared runner with `analysis` at the candidate; the wall time
/// and the report.
fn nlp(
    backend: Backend,
    settings: &BackendSettings,
    threads: usize,
    analysis: execution::Analysis,
) -> (f64, SolveReport) {
    let n = side() * side();
    let controls = Controls {
        reuse: ReusePolicy::Fresh,
        threads,
        ..Controls::default()
    };
    let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-8).unwrap();
    let oracle = Grid::new();
    let normalization = oracle.normalization.clone();
    let tolerances = tolerances(2 * n, n);
    let initial = vec![0.0; 2 * n];
    let started = Instant::now();
    let report = execution::nlp(
        Step {
            adapter: LINKED.get(backend).unwrap(),
            settings,
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
            oracle: Box::new(oracle),
            initial: &initial,
            presolve: &Policy::Off,
            intent: SolveIntent::Optimize,
            sense: ObjectiveSense::Minimize,
            limit: 1 << 24,
            analysis,
        },
    )
    .unwrap();
    (started.elapsed().as_secs_f64(), report)
}

#[test]
#[ignore = "single-run measurement, run on demand"]
fn measure_large_kkt_linear_solvers() {
    let ipopt = |linear| {
        BackendSettings::Ipopt(crate::settings::ipopt::Settings {
            linear,
            ..Default::default()
        })
    };
    let routes = [
        (
            "ipopt mumps metis",
            Backend::Ipopt,
            ipopt(Linear::Mumps {
                ordering: MumpsOrdering::Metis,
            }),
            1,
        ),
        (
            "ipopt spral (1 thread)",
            Backend::Ipopt,
            ipopt(Linear::Spral {
                ordering: SpralOrdering::Metis,
                scaling: SpralScaling::Matching,
                pivot: SpralPivot::Block,
            }),
            1,
        ),
        (
            "ipopt spral (8 threads)",
            Backend::Ipopt,
            ipopt(Linear::Spral {
                ordering: SpralOrdering::Metis,
                scaling: SpralScaling::Matching,
                pivot: SpralPivot::Block,
            }),
            8,
        ),
        (
            "ipopt mkl pardiso (1 thread)",
            Backend::Ipopt,
            ipopt(Linear::PardisoMkl {
                ordering: PardisoOrdering::Metis,
                matching: PardisoMatching::CompletePlus2x2,
            }),
            1,
        ),
        (
            "ipopt mkl pardiso (8 threads)",
            Backend::Ipopt,
            ipopt(Linear::PardisoMkl {
                ordering: PardisoOrdering::Metis,
                matching: PardisoMatching::CompletePlus2x2,
            }),
            8,
        ),
        (
            "pounce feral (8 threads)",
            Backend::Pounce,
            BackendSettings::Pounce(Default::default()),
            8,
        ),
        (
            "pounce feral",
            Backend::Pounce,
            BackendSettings::Pounce(Default::default()),
            1,
        ),
    ];
    let mut objectives = Vec::new();
    // Every route with the standing KKT-point analysis of an optimization, then the first
    // and the last without it, which isolates the analysis cost.
    let runs = routes
        .iter()
        .map(|route| (route, true))
        .chain([0, routes.len() - 1].map(|k| (&routes[k], false)));
    for ((name, backend, settings, threads), analysed) in runs {
        // A route runs inside its adapter's admitted thread scope, as the runtime enters it.
        let (seconds, report) = execution::scoped(
            &[LINKED.get(*backend).unwrap()],
            *threads,
            64 << 20,
            || {
                let analysis = if analysed {
                    execution::Analysis::for_intent(SolveIntent::Optimize)
                } else {
                    execution::Analysis::NONE
                };
                Ok::<_, ProblemError>(nlp(*backend, settings, *threads, analysis))
            },
        )
        .unwrap();
        let objective = report.candidate.as_ref().and_then(|c| c.objective);
        eprintln!(
            "measure n_kkt={} {name}{}: {seconds:.3} s, {} iterations, {:?}, objective {objective:?}",
            3 * side() * side(),
            if analysed { "" } else { " (no analysis)" },
            crate::restart_tests::iterations(&report),
            report.termination.category,
        );
        assert_eq!(report.termination.category, Termination::Success, "{name}");
        objectives.push(objective.unwrap());
    }
    for objective in &objectives {
        assert!((objective - objectives[0]).abs() <= 1e-6 * objectives[0].abs().max(1.0));
    }
}

/// The linearized grid control as a coefficient problem: `½vᵀQv + cᵀv` with
/// `Q = diag(2, …, 2α, …)` and `c = −2t` on `x`.
fn linearized() -> (CoefficientProblem, SparseColMat<usize, f64>) {
    let n = side() * side();
    let q = csc(
        2 * n,
        2 * n,
        &(0..2 * n)
            .map(|k| (k, k, if k < n { 2.0 } else { 2.0 * ALPHA }))
            .collect::<Vec<_>>(),
    );
    let problem = CoefficientProblem {
        contract: contract(),
        objective: (0..2 * n)
            .map(|k| if k < n { -2.0 * target(k) } else { 0.0 })
            .collect(),
        objective_constant: (0..n).map(|r| target(r).powi(2)).sum(),
        sense: ObjectiveSense::Minimize,
        domains: vec![ModelingVariableDomain::Continuous; 2 * n],
        assumptions: crate::solver_tests::stamp(Backend::Clarabel).data,
        constraints: csc(n, 2 * n, &stencil(None)),
        hessian: Some(q.clone()),
        bounds: vec![(0.0, 0.0); n],
        objectives: Vec::new(),
    };
    (problem, q)
}

#[test]
#[ignore = "single-run measurement, run on demand"]
fn measure_clarabel_qdldl_against_mkl_pardiso() {
    let (problem, q) = linearized();
    let n = side() * side();
    let proof = crate::solver_tests::certify(&q, 1.0);
    let normalization = Normalization::identity(2 * n, n);
    let tolerances = tolerances(2 * n, n);
    let mut objectives = Vec::new();
    for (direct, threads) in [
        (Direct::Qdldl, 1),
        (Direct::MklPardiso, 1),
        (Direct::MklPardiso, 4),
        (Direct::MklPardiso, 8),
    ] {
        let controls = Controls {
            threads,
            ..Controls::default()
        };
        let settings = BackendSettings::Clarabel(crate::conic::Settings {
            direct,
            ..crate::conic::Settings::default()
        });
        let started = Instant::now();
        let report = crate::clarabel_tests::run(
            &problem,
            Some(&proof),
            Backend::Clarabel,
            &settings,
            &controls,
            &normalization,
            &tolerances,
        )
        .unwrap();
        let seconds = started.elapsed().as_secs_f64();
        let objective = report.candidate.as_ref().and_then(|c| c.objective);
        eprintln!(
            "measure n_kkt={} clarabel {direct:?} ({threads} threads): {seconds:.3} s \
             (native {:?} s, {:?} iterations, nnz(L) {:?}), {:?}, objective {objective:?}",
            3 * n,
            report.metrics.get("solve_time"),
            report.metrics.get("iterations"),
            report.metrics.get("linear.nnzL"),
            report.termination.category,
        );
        assert_eq!(report.termination.category, Termination::Success);
        objectives.push(objective.unwrap());
    }
    for objective in &objectives {
        assert!((objective - objectives[0]).abs() <= 1e-6 * objectives[0].abs().max(1.0));
    }
}
