// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded existing integrations at practical accuracy; no backend selection threshold.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Shape {
    Dense,
    Banded,
    Skewed,
}
#[derive(Debug)]
struct QuadraticQuartic {
    contract: OracleContract,
    jacobian: SparseColMat<usize, f64>,
    hessian: SparseColMat<usize, f64>,
    bounds: Vec<(f64, f64)>,
    normalization: Normalization,
    target: Vec<f64>,
    shape: Shape,
    generation: u8,
}
impl QuadraticQuartic {
    fn new(n: usize, shape: Shape, generation: u8) -> Self {
        let contract = OracleContract {
            identity: ContentHash::from_bytes([91; 32]),
            variables: (0..n)
                .map(|i| Variable {
                    id: identity(91, i),
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                })
                .collect(),
            rows: (0..n / 2).map(|i| identity(92, i)).collect(),
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        let target = (0..n)
            .map(|i| i as f64 * 0.01 + f64::from(generation) * 0.1)
            .collect::<Vec<_>>();
        let bounds = target
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| (pair[0] + pair[1], pair[0] + pair[1]))
            .collect();
        let jacobian = csc(
            n / 2,
            n,
            &(0..n).map(|col| (col / 2, col, 1.)).collect::<Vec<_>>(),
        );
        let hessian = csc(
            n,
            n,
            &(0..n)
                .flat_map(|col| {
                    (col..n).filter_map(move |row| {
                        (row == col || matches!(shape, Shape::Dense) || row.abs_diff(col) == 1)
                            .then_some((row, col, 1.))
                    })
                })
                .collect::<Vec<_>>(),
        );
        Self {
            contract,
            jacobian,
            hessian,
            bounds,
            normalization: Normalization::identity(n, n / 2),
            target,
            shape,
            generation,
        }
    }
    fn coefficient(&self, row: usize, col: usize) -> f64 {
        if row == col {
            (if matches!(self.shape, Shape::Skewed) {
                10_f64.powi((row % 4) as i32)
            } else {
                2.
            }) + f64::from(self.generation) * 0.25
        } else if matches!(self.shape, Shape::Dense) {
            0.1 / self.target.len() as f64
        } else if row.abs_diff(col) == 1 {
            0.05
        } else {
            0.
        }
    }
    fn original_gradient(&self, x: &[f64], out: &mut [f64]) {
        for (row, value) in out.iter_mut().enumerate() {
            *value = 0.1 * (x[row] - self.target[row]).powi(3)
                + (0..x.len())
                    .map(|col| self.coefficient(row, col) * (x[col] - self.target[col]))
                    .sum::<f64>();
        }
    }
}
impl NlpOracle for QuadraticQuartic {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.normalization)
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
        let mut value = 0.;
        for row in 0..x.len() {
            let d = x[row] - self.target[row];
            value += 0.025 * d.powi(4);
            for col in 0..x.len() {
                value += 0.5 * d * self.coefficient(row, col) * (x[col] - self.target[col]);
            }
        }
        Ok(value)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (pair, value) in x.as_chunks::<2>().0.iter().zip(out) {
            *value = pair[0] + pair[1];
        }
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.original_gradient(x, out);
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(1.);
        Ok(())
    }
    fn hessian(
        &mut self,
        x: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        for col in 0..x.len() {
            let start = self.hessian.col_ptr()[col];
            let end = self.hessian.col_ptr()[col + 1];
            for (value, &row) in out[start..end]
                .iter_mut()
                .zip(&self.hessian.row_idx()[start..end])
            {
                *value = weight
                    * (self.coefficient(row, col)
                        + if row == col {
                            0.3 * (x[col] - self.target[col]).powi(2)
                        } else {
                            0.
                        });
            }
        }
        Ok(())
    }
}

#[test]
fn bounded_existing_linear_portfolio_preserves_original_kkt_at_standard_accuracy() {
    let ipopt = |linear| {
        BackendSettings::Ipopt(crate::settings::ipopt::Settings {
            linear,
            ..Default::default()
        })
    };
    let routes = [
        ("mumps", Backend::Ipopt, ipopt(Linear::default())),
        (
            "spral",
            Backend::Ipopt,
            ipopt(Linear::Spral {
                ordering: SpralOrdering::Metis,
                scaling: SpralScaling::Matching,
                pivot: SpralPivot::Block,
            }),
        ),
        (
            "pardiso",
            Backend::Ipopt,
            ipopt(Linear::PardisoMkl {
                ordering: PardisoOrdering::Metis,
                matching: PardisoMatching::CompletePlus2x2,
            }),
        ),
        (
            "pounce-feral",
            Backend::Pounce,
            BackendSettings::Pounce(Default::default()),
        ),
    ];
    let controls = Controls {
        threads: 1,
        ..Controls::default()
    };
    let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 0.001).unwrap();
    for (shape, n) in [(Shape::Dense, 16), (Shape::Banded, 64), (Shape::Skewed, 32)] {
        for (name, backend, settings) in &routes {
            execution::scoped(&[LINKED.get(*backend).unwrap()], 1, 64 << 20, || {
                let mut retained = Retained::default();
                for generation in 0..2 {
                    let complete = Instant::now();
                    let oracle = QuadraticQuartic::new(n, shape, generation);
                    let normalization = oracle.normalization.clone();
                    let initial = oracle.target.iter().map(|x| x + 0.5).collect::<Vec<_>>();
                    let tolerances = Tolerances { variables: vec![0.001; n], rows: vec![0.001; n / 2], integrality: 0.001 };
                    let preparation_ns = complete.elapsed().as_nanos();
                    let started = Instant::now();
                    let report = execution::nlp(Step {
                        snapshot: &execution::Snapshot::observe(&LINKED), structure: None,
                        adapter: LINKED.get(*backend).unwrap(), settings, controls: &controls, accuracy: &accuracy,
                        execution: Execution::new(Default::default(), &controls), tolerances: &tolerances,
                        normalization: &normalization, compatibility: Compatibility {
                            layout: ContentHash::from_bytes([93; 32]), profile: ContentHash::from_bytes([94; 32]),
                            data: ContentHash::from_bytes([generation; 32]), backend: *backend }, warm: None,
                    }, &mut retained, Nlp { oracle: Box::new(oracle), initial: &initial, presolve: &Policy::Off,
                        intent: SolveIntent::Optimize, sense: ObjectiveSense::Minimize, limit: 1 << 24,
                        analysis: execution::Analysis::for_intent(SolveIntent::Optimize) })?;
                    let solve_ns = started.elapsed().as_nanos();
                    assert_eq!(report.termination.category, Termination::Success, "{name} {shape:?}");
                    assert!(report.quality.as_ref().unwrap().feasible());
                    let candidate = report.candidate.as_ref().unwrap();
                    // Independently re-evaluate every original equality and reduced stationarity
                    // (the equality nullspace direction is [1,-1] for each pair).
                    let source = QuadraticQuartic::new(n, shape, generation);
                    let x = &candidate.primal;
                    let original_max = x.as_chunks::<2>().0.iter().zip(&source.bounds).map(|(x, bounds)|
                        (x[0] + x[1] - bounds.0).abs()).fold(0., f64::max);
                    let mut gradient = vec![0.; n]; source.original_gradient(x, &mut gradient);
                    let stationarity = gradient.as_chunks::<2>().0.iter().map(|g| 0.5 * (g[0] - g[1]).abs()).fold(0., f64::max);
                    assert!(original_max <= 0.001, "{name} original residual {original_max}");
                    assert!(stationarity <= accuracy.stationarity, "{name} original reduced stationarity {stationarity}");
                    let local = report.evidence.local.as_ref().expect("requested original KKT analysis")
                        .as_ref().expect("qualified positive-definite reduced Hessian");
                    let backsolve = local.residual.expect("original KKT analysis backsolve");
                    assert!(backsolve.is_finite() && backsolve <= 0.001);
                    let phases = report.metrics.iter().filter(|(key, _)| key.starts_with("timing.native.")
                        || key.contains("memory") || key.contains("factor") || key.contains("nnz")).collect::<Vec<_>>();
                    eprintln!("portfolio shape={shape:?} n={n} kkt={} route={name} threads=1 generation={generation} preparation_ns={preparation_ns} solve_ns={solve_ns} complete_with_independent_check_ns={} original_max={original_max} reduced_stationarity={stationarity} analysis_backsolve={backsolve} analysis_inertia={:?} analysis_condition={:?} iterations={} phases={phases:?}",
                        n + n / 2, complete.elapsed().as_nanos(), local.inertia, local.condition_1norm, crate::restart_tests::iterations(&report));
                }
                Ok::<_, ProblemError>(())
            }).unwrap();
        }
    }
}
