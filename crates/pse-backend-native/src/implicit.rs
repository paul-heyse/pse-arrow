// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! KINSOL child execution on an already admitted outer worker. No runtime admission occurs here.
use crate::{
    NleOracle, OracleContract, ProblemError, Variable, kinsol, quality::Tolerances, solve::*,
};
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    implicit::{InnerSolver, Options, Problem},
};
use std::sync::{Arc, atomic::AtomicBool};

/// Native root capability injected into generic implicit evaluation.
#[derive(Debug)]
pub struct Kinsol;
#[derive(Debug)]
struct Oracle {
    problem: Arc<Problem>,
    parameters: Vec<f64>,
    contract: OracleContract,
    cancel: Arc<AtomicBool>,
}
impl NleOracle for Oracle {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.problem.pattern()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let jet =
            self.problem
                .evaluate(&self.parameters, x, DerivativeOrder::Value, &self.cancel)?;
        if jet.values.len() != out.len() {
            return Err(ProblemError::internal("implicit residual extent"));
        }
        out.copy_from_slice(&jet.values);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let n = self.problem.unknowns.len();
        let width = n + self.problem.inputs;
        let jet =
            self.problem
                .evaluate(&self.parameters, x, DerivativeOrder::First, &self.cancel)?;
        let pattern = self.problem.pattern();
        if jet.jacobian.len() != n * width || out.len() != pattern.compute_nnz() {
            return Err(ProblemError::internal("implicit Jacobian extent"));
        }
        for (slot, (i, j)) in (0..n)
            .flat_map(|j| pattern.row_idx_of_col(j).map(move |i| (i, j)))
            .enumerate()
        {
            out[slot] = jet.jacobian[i * width + j];
        }
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let n = self.problem.unknowns.len();
        if direction.len() != n || out.len() != n {
            return Err(ProblemError::internal("implicit JVP extent"));
        }
        let mut values = vec![0.0; self.problem.pattern().compute_nnz()];
        self.jacobian(x, &mut values)?;
        out.fill(0.);
        let pattern = self.problem.pattern();
        for (slot, (i, j)) in (0..n)
            .flat_map(|j| pattern.row_idx_of_col(j).map(move |i| (i, j)))
            .enumerate()
        {
            out[i] += values[slot] * direction[j];
        }
        Ok(())
    }
}
impl InnerSolver for Kinsol {
    fn identity(&self) -> pse_ids::ContentHash {
        pse_math::implicit::solver_identity("sundials.kinsol.v1")
    }
    fn solve(
        &self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, MathError> {
        problem.validate_options(options)?;
        let mut controls = Controls {
            iterations: options.iterations,
            time_limit: options.time_limit,
            ..Controls::default()
        };
        controls.accuracy.feasibility = 1.0;
        let contract = OracleContract {
            identity: problem.identity,
            variables: problem
                .unknowns
                .iter()
                .map(|u| Variable {
                    id: u.id,
                    lower: if u.lower >= 0.0 {
                        0.0
                    } else {
                        f64::NEG_INFINITY
                    },
                    upper: if u.upper <= 0.0 { 0.0 } else { f64::INFINITY },
                })
                .collect(),
            rows: problem.rows.clone(),
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        let oracle = Oracle {
            problem: problem.clone(),
            parameters: parameters.to_vec(),
            contract,
            cancel: cancel.clone(),
        };
        let settings = kinsol::Settings {
            strategy: kinsol::Strategy::LineSearch,
            linear: kinsol::Linear::Klu,
            variable_scales: options
                .variable_nominals
                .iter()
                .map(|v| v.recip())
                .collect(),
            residual_scales: options.residual_tolerance.iter().map(|t| 1.0 / t).collect(),
            anderson: 0,
            damping: 1.0,
            setup_interval: 1,
            step_tolerance: 1e-12,
        };
        let compatibility = Compatibility {
            layout: problem.identity,
            data: problem.identity,
            backend: Backend::Kinsol,
        };
        let execution = Execution::new(cancel.clone(), &controls);
        // Typed native causes, including structural rows and columns, stay attributable.
        let map = |e: ProblemError| match e {
            ProblemError::Math(e) => e,
            ProblemError::Cancelled => MathError::Cancelled,
            other => MathError::Native {
                source_id: problem.id,
                retained: other.retained_bytes(),
                cause: Box::new(other),
            },
        };
        let mut session = kinsol::Session::new(
            kinsol::Function::Equations(Box::new(oracle)),
            settings,
            execution.clone(),
            compatibility,
        )
        .map_err(map)?;
        let tolerances = Tolerances {
            variables: options.variable_tolerance.clone(),
            rows: options.residual_tolerance.clone(),
            integrality: f64::EPSILON,
        };
        let report = session
            .solve(&options.start, &controls, execution, &tolerances, None)
            .map_err(map)?;
        match report.termination.category {
            Termination::Cancelled => return Err(MathError::Cancelled),
            Termination::TimeLimit => return Err(MathError::Limit("implicit solve time")),
            Termination::Success | Termination::Acceptable => {}
            _ => {
                return Err(MathError::Domain {
                    source_id: problem.id,
                    requirement: "native inner solve did not converge",
                });
            }
        }
        let point = report
            .candidate
            .ok_or(MathError::Domain {
                source_id: problem.id,
                requirement: "inner solve returned no candidate",
            })?
            .primal;
        problem.verify(parameters, &point, options, cancel)?;
        Ok(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn problem(deficient: bool) -> Result<Problem, MathError> {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([96; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut builder = pse_math::typed::BodyBuilder::new(
                pse_math::initialize().unwrap(),
                &registry,
                &pse_quantity::standard::StandardInvariantChecker,
                3,
                pse_math::typed::BodyLimits::default(),
            )
            .unwrap();
            let x = builder
                .input(0, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let y = builder
                .input(1, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let p = builder
                .input(2, q, pse_quantity::IndexSet::new(), id)
                .unwrap();
            let a = builder
                .binary(pse_math::typed::Binary::Sub, x.clone(), p.clone(), None, id)
                .unwrap();
            let b = builder
                .binary(
                    pse_math::typed::Binary::Sub,
                    if deficient { x } else { y },
                    p,
                    None,
                    id,
                )
                .unwrap();
            let body = Arc::new(
                builder
                    .finish(
                        &[a, b],
                        DerivativeOrder::Second,
                        pse_math::library::Optimization::default(),
                        &cancel,
                    )
                    .unwrap(),
            );
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([96; 32]),
                vec![
                    pse_math::implicit::Unknown {
                        id: pse_ids::named_id(id, "x"),
                        lower: -10.,
                        upper: 10.,
                    },
                    pse_math::implicit::Unknown {
                        id: pse_ids::named_id(id, "y"),
                        lower: -10.,
                        upper: 10.,
                    },
                ],
                vec![pse_ids::named_id(id, "a"), pse_ids::named_id(id, "b")],
                1,
                body,
                100,
            )
        }
    }
    fn options() -> Options {
        Options {
            start: vec![1., 1.],
            variable_nominals: vec![1., 1.],
            variable_tolerance: vec![1e-8, 1e-8],
            residual_tolerance: vec![1e-8, 1e-8],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        }
    }
    #[test]
    fn structural_failure_keeps_rows_and_columns() {
        let cancel = Arc::new(AtomicBool::new(false));
        let deficient = Arc::new(problem(true).unwrap());
        let error = Kinsol
            .solve(deficient.clone(), &[3.], &options(), &cancel)
            .unwrap_err();
        let MathError::Native {
            source_id, cause, ..
        } = &error
        else {
            panic!("untyped inner failure: {error:?}");
        };
        assert_eq!(*source_id, deficient.id);
        let Some(ProblemError::Structural { rows, columns, .. }) =
            cause.downcast_ref::<ProblemError>()
        else {
            panic!("structural identities were not retained: {cause:?}");
        };
        let y = deficient.unknowns[1].id;
        assert_eq!(columns, &vec![y]);
        assert!(!rows.is_empty() && rows.iter().all(|r| deficient.rows.contains(r)));
        assert!(error.retained_bytes() > size_of::<MathError>());
    }
    #[test]
    fn implicit_kinsol_preserves_sparse_support_and_refuses_structural_deficiency() {
        let cancel = Arc::new(AtomicBool::new(false));
        let build = problem;
        let options = Options {
            start: vec![1., 1.],
            variable_nominals: vec![1., 1.],
            variable_tolerance: vec![1e-8, 1e-8],
            residual_tolerance: vec![1e-8, 1e-8],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let problem = Arc::new(build(false).unwrap());
        assert_eq!(problem.pattern().compute_nnz(), 2);
        let root = Kinsol
            .solve(problem.clone(), &[3.], &options, &cancel)
            .unwrap();
        assert!(root.iter().all(|x| (*x - 3.).abs() < 1e-8));
        let jet = problem
            .derivatives(&[3.], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(jet.jacobian, vec![1., 1.]);
        assert_eq!(jet.hessians, vec![0., 0.]);
        // Either faer's symbolic admission or the shared KINSOL structural check
        // must refuse a missing unknown column, even though row count is square.
        if let Ok(deficient) = build(true) {
            assert!(
                Kinsol
                    .solve(Arc::new(deficient), &[3.], &options, &cancel)
                    .is_err()
            );
        }
    }
    #[test]
    fn implicit_kinsol_uses_physical_variable_nominals() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([95; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = pse_math::typed::BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            pse_math::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let residual = builder
            .binary(pse_math::typed::Binary::Sub, y, p, None, id)
            .unwrap();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    pse_math::library::Optimization::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let problem = Arc::new(
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([95; 32]),
                vec![pse_math::implicit::Unknown {
                    id,
                    lower: 0.,
                    upper: f64::INFINITY,
                }],
                vec![pse_ids::named_id(id, "row")],
                1,
                body,
                100,
            )
            .unwrap(),
        );
        let mut options = Options {
            start: vec![1.],
            variable_nominals: vec![1e12],
            variable_tolerance: vec![1e-3],
            residual_tolerance: vec![1e-3],
            iterations: 20,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let root = Kinsol
            .solve(problem.clone(), &[1e12], &options, &cancel)
            .unwrap();
        assert!((root[0] - 1e12).abs() < 1e-3);
        options.variable_nominals[0] = 1.;
        assert!(
            Kinsol
                .solve(problem.clone(), &[1e12], &options, &cancel)
                .is_err()
        );
        options.variable_nominals[0] = 0.;
        assert!(problem.validate_options(&options).is_err());
    }
    #[test]
    fn implicit_kinsol_rechecks_roots_and_arbitrary_interval_guards() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = pse_ids::SemanticId::from_bytes([94; 16]);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = pse_math::typed::BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            pse_math::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let square = builder
            .binary(pse_math::typed::Binary::Mul, y.clone(), y, None, id)
            .unwrap();
        let residual = builder
            .binary(pse_math::typed::Binary::Sub, square, p, None, id)
            .unwrap();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    pse_math::library::Optimization::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let problem = Arc::new(
            Problem::new(
                id,
                pse_ids::ContentHash::from_bytes([94; 32]),
                vec![pse_math::implicit::Unknown {
                    id,
                    lower: 0.5,
                    upper: 2.5,
                }],
                vec![pse_ids::named_id(id, "row")],
                1,
                body,
                100,
            )
            .unwrap(),
        );
        let options = Options {
            start: vec![1.0],
            variable_nominals: vec![2.0],
            variable_tolerance: vec![1e-9],
            residual_tolerance: vec![1e-9],
            iterations: 50,
            time_limit: std::time::Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        let root = Kinsol
            .solve(problem.clone(), &[4.0], &options, &cancel)
            .unwrap();
        assert!((root[0] - 2.0).abs() < 1e-8);
        let jet = problem
            .derivatives(&[4.0], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert!((jet.jacobian[0] - 0.25).abs() < 1e-8);
        assert!(Kinsol.solve(problem, &[9.0], &options, &cancel).is_err());
    }
}
