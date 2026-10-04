// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-domain barrier and multiplier transport controls.
use super::*;
use pounce_nlp::expression_provider::ExpressionProvider;

#[derive(Clone, Copy, Debug)]
enum Model {
    Quadratic,
    LowerRow,
    Affine,
}
#[derive(Debug)]
struct Source {
    model: Model,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    facts: Facts,
    j: AssemblyMatrix,
    h: AssemblyMatrix,
}
impl Source {
    fn new(model: Model) -> Self {
        let mut base = Mixed::new();
        let (variables, bounds, affine, tape) = match model {
            Model::Quadratic => (
                vec![Variable {
                    id: id(1),
                    lower: 1e-10,
                    upper: 1.0,
                }],
                (0.25, 0.25),
                None,
                vec![Op::Var(0), Op::PowInt(0, 2)],
            ),
            Model::LowerRow => (
                vec![Variable {
                    id: id(1),
                    lower: 0.0,
                    upper: 10.0,
                }],
                (2.0, f64::INFINITY),
                Some(AffineRow {
                    entries: BTreeMap::from([(0, 1.0)]),
                    constant: 0.0,
                }),
                vec![Op::Var(0)],
            ),
            Model::Affine => (
                vec![
                    Variable {
                        id: id(1),
                        lower: 0.0,
                        upper: 10.0,
                    },
                    Variable {
                        id: id(2),
                        lower: 0.0,
                        upper: 4.0,
                    },
                ],
                (0.0, 0.0),
                Some(AffineRow {
                    entries: BTreeMap::from([(0, -2.0), (1, 1.0)]),
                    constant: 0.0,
                }),
                vec![
                    Op::Var(0),
                    Op::Const(2.0),
                    Op::Mul(0, 1),
                    Op::Var(1),
                    Op::Sub(3, 2),
                ],
            ),
        };
        let n = variables.len();
        base.contract.variables = variables;
        base.contract.rows = vec![id(3)];
        base.facts.affine = vec![affine];
        base.facts.tapes = vec![FbbtTape { ops: tape }];
        base.facts.complete = vec![true];
        base.facts.row_sources = vec![vec![]];
        base.facts.objective_linear = vec![true; n];
        let entries = (0..n)
            .map(|c| Entry::new(OriginalRow::new(0), OriginalCol::new(c)))
            .collect::<Vec<_>>();
        Self {
            model,
            contract: base.contract,
            bounds: vec![bounds],
            facts: base.facts,
            j: AssemblyMatrix::new(1, n, &entries, 100).unwrap(),
            h: hessian(n, &(0..n).map(|c| (c, c)).collect::<Vec<_>>()),
        }
    }
    fn tolerances(&self) -> Tolerances {
        Tolerances {
            variables: vec![1e-8; self.contract.variables.len()],
            rows: vec![1e-8],
            integrality: 1e-8,
        }
    }
}
impl ExpressionProvider for Source {
    fn constraint_expression(&self, i: usize) -> Option<FbbtTape> {
        self.facts.tapes.get(i).cloned()
    }
}
impl NlpOracle for Source {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn presolve_facts(&self) -> Option<&Facts> {
        Some(&self.facts)
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.j.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.h.matrix().symbolic())
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(if matches!(self.model, Model::Affine) {
            -x[0]
        } else {
            x[0]
        })
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.0);
        out[0] = if matches!(self.model, Model::Affine) {
            -1.0
        } else {
            1.0
        };
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = match self.model {
            Model::Quadratic => x[0] * x[0],
            Model::LowerRow => x[0],
            Model::Affine => x[1] - 2.0 * x[0],
        };
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        match self.model {
            Model::Quadratic => out[0] = 2.0 * x[0],
            Model::LowerRow => out[0] = 1.0,
            Model::Affine => out.copy_from_slice(&[-2.0, 1.0]),
        }
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        _: f64,
        lambda: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.fill(0.0);
        if matches!(self.model, Model::Quadratic) {
            out[0] = 2.0 * lambda[0];
        }
        Ok(())
    }
}

#[test]
fn nlp_presolve_refuses_required_untracked_bound_transport_before_dispatch() {
    for pass in [Pass::Fbbt, Pass::LinearBounds] {
        let source = Source::new(Model::LowerRow);
        let tolerance = source.tolerances();
        let policy = Policy::Explicit {
            options: PresolveOptions {
                enabled: true,
                fbbt: true,
                ..PresolveOptions::defaults()
            },
            required: BTreeSet::from([pass]),
        };
        assert!(matches!(
            Pipeline::new(
                Box::new(source),
                &[3.0],
                &policy,
                &tolerance,
                &ResolvedAccuracy::nominal(),
                execution(),
                None,
                stamp(),
                1000
            ),
            Err(ProblemError::Unsupported(_))
        ));
    }
}

#[test]
fn nlp_presolve_retains_provenance_bearing_affine_bound_transfer() {
    let source = Source::new(Model::Affine);
    let tolerance = source.tolerances();
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[1.0, 2.0],
        &Policy::Auto,
        &tolerance,
        &ResolvedAccuracy::nominal(),
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    assert!(pipeline.report().passes[&Pass::AffineElimination].applied);
    assert!(pipeline.report().passes[&Pass::RedundantRows].applied);
    let native = pipeline.take_oracle().unwrap();
    assert_eq!(native.contract().variables.len(), 1);
    assert!(native.contract().rows.is_empty());
    let variable = &native.contract().variables[0];
    assert_eq!(variable.lower, 0.0);
    assert_eq!(
        variable.upper,
        if variable.id == id(1) {
            2.0
        } else {
            assert_eq!(variable.id, id(2));
            4.0
        }
    );
}

#[test]
fn nlp_presolve_analysis_shows_singleton_geometry_without_materializing_it() {
    let source = Source::new(Model::Quadratic);
    let mut lower = vec![source.contract.variables[0].lower];
    let mut upper = vec![source.contract.variables[0].upper];
    let result = pounce_presolve::fbbt::run_fbbt(
        &source,
        1,
        1,
        &mut lower,
        &mut upper,
        &[0.25],
        &[0.25],
        None,
        &pounce_presolve::fbbt::FbbtConfig {
            tol: 1e-6,
            max_iter: 10,
            max_constraints: 1,
        },
    );
    assert!(result.bound_updates > 0);
    assert!(lower[0] <= 0.5 && upper[0] >= 0.5);
    assert!(upper[0] - lower[0] < 1e-14, "{lower:?} {upper:?}");
    let tolerance = source.tolerances();
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[0.6],
        &Policy::Auto,
        &tolerance,
        &ResolvedAccuracy::nominal(),
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let report = pipeline.report();
    assert!(matches!(report.requested, Policy::Auto));
    for pass in [Pass::LinearBounds, Pass::Fbbt] {
        let decision = &report.passes[&pass];
        assert!(decision.requested && !decision.eligible && !decision.applied);
        assert_eq!(decision.reason.as_deref(), Some(UNTRACKED_BOUND_DUALS));
    }
    assert!(!report.effective.warm_z_bounds);
    assert!(report.diagnostics.contains_key("admission.fbbt"));
    let native = pipeline.take_oracle().unwrap();
    assert_eq!(native.contract().variables[0].lower, 1e-10);
    assert_eq!(native.contract().variables[0].upper, 1.0);
    assert_eq!(native.contract().rows, vec![id(3)]);
    assert_eq!(native.constraint_bounds(), &[(0.25, 0.25)]);
}

#[cfg(feature = "ipopt")]
fn native_run(model: Model) -> SolveReport {
    let source = Source::new(model);
    let n = source.contract.variables.len();
    let tolerance = source.tolerances();
    let initial = match model {
        Model::Quadratic => vec![0.6],
        Model::LowerRow => vec![3.0],
        Model::Affine => vec![1.0, 2.0],
    };
    let accuracy = ResolvedAccuracy {
        feasibility: 1e-10,
        stationarity: 1e-9,
        complementarity: 1e-9,
        ..ResolvedAccuracy::nominal()
    };
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &initial,
        &Policy::Auto,
        &tolerance,
        &accuracy,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let mut native = pipeline.take_oracle().unwrap();
    if matches!(model, Model::Affine) {
        assert!(pipeline.report().passes[&Pass::AffineElimination].applied);
        assert_eq!(native.contract().variables.len(), 1);
        assert!(native.contract().rows.is_empty());
    } else {
        assert_eq!(native.contract().rows, vec![id(3)]);
        assert_eq!(
            native.contract().variables[0].upper,
            if matches!(model, Model::Quadratic) {
                1.0
            } else {
                10.0
            }
        );
    }
    let controls = Controls {
        iterations: 100,
        ..Controls::default()
    };
    let report = crate::ipopt::Session::new()
        .solve(
            &mut native,
            pipeline.initial(),
            ObjectiveSense::Minimize,
            &controls,
            &accuracy,
            &Default::default(),
            execution(),
            &pipeline.tolerances(&tolerance),
            pipeline.warm(),
            pipeline.native_compatibility().clone(),
        )
        .unwrap();
    let (mut report, _) = pipeline.finish(
        report,
        &tolerance,
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
    crate::quality::record_kkt(
        &mut report,
        &pse_math::normalization::Normalization::identity(n, 1),
        &accuracy,
    );
    crate::quality::qualify(&mut report, &accuracy);
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert_eq!(
        report.termination.assurance,
        Assurance::LocalStationary,
        "{report:?}"
    );
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert!(
        report
            .observation
            .as_ref()
            .unwrap()
            .complementarity
            .as_ref()
            .unwrap()
            .iter()
            .all(|v| *v <= 1e-9),
        "{report:?}"
    );
    report
}

#[cfg(feature = "ipopt")]
#[test]
fn native_nlp_presolve_quadratic_keeps_authored_barriers_and_original_complementarity() {
    let report = native_run(Model::Quadratic);
    let candidate = report.candidate.unwrap();
    assert!((candidate.primal[0] - 0.5).abs() < 1e-8);
    let (lower, upper) = candidate.bound_dual.unwrap();
    assert!(lower[0] < 1e-8 && upper[0] < 1e-8);
    assert!((candidate.row_dual.unwrap()[0] + 1.0).abs() < 1e-7);
}

#[cfg(feature = "ipopt")]
#[test]
fn native_nlp_presolve_lower_row_retains_original_row_multiplier() {
    let report = native_run(Model::LowerRow);
    let candidate = report.candidate.unwrap();
    assert!((candidate.primal[0] - 2.0).abs() < 1e-8);
    let (lower, upper) = candidate.bound_dual.unwrap();
    assert!(lower[0] < 1e-8 && upper[0] < 1e-8);
    assert!((candidate.row_dual.unwrap()[0] + 1.0).abs() < 1e-7);
}

#[cfg(feature = "ipopt")]
#[test]
fn native_nlp_presolve_affine_transfer_retains_bound_owner_and_original_complementarity() {
    let report = native_run(Model::Affine);
    let candidate = report.candidate.unwrap();
    assert!((candidate.primal[0] - 2.0).abs() < 1e-8);
    assert!((candidate.primal[1] - 4.0).abs() < 1e-8);
    let (lower, upper) = candidate.bound_dual.unwrap();
    assert!(lower.iter().all(|v| *v < 1e-8));
    assert!(upper[0] < 1e-8);
    assert!((upper[1] - 0.5).abs() < 1e-7);
}
