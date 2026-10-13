// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact affine elimination over the original guarded residual, with library sparse LU.
use super::*;

/// An admitted residual affine in its leading unknown coordinates. Coefficients may
/// depend nonlinearly on external inputs; original guards remain active at every trial.
#[derive(Debug, Clone)]
pub struct Affine {
    unknowns: usize,
    inputs: usize,
}
impl Affine {
    /// Admit `body` when it is affine in its first `unknowns` inputs, one output per unknown.
    pub fn new(body: &crate::guarded::PreparedBody, unknowns: usize) -> Result<Self, MathError> {
        Self::with_cancel(body, unknowns, &Arc::new(AtomicBool::new(false)))
    }
    pub(super) fn with_cancel(
        body: &crate::guarded::PreparedBody,
        unknowns: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
        if unknowns == 0 || body.output_count() != unknowns || body.input_count() < unknowns {
            return Err(MathError::Contract("affine residual extent".into()));
        }
        let coordinates = (0..unknowns).collect::<Vec<_>>();
        let support = body.prepare_support(
            &(0..body.output_count()).collect::<Vec<_>>(),
            &coordinates,
            DerivativeOrder::Second,
            cancel,
        )?;
        if body.available_order_for_outputs(support.outputs(), &coordinates)?
            != DerivativeOrder::Second
            || support
                .support()
                .second
                .iter()
                .any(|row| row.iter().any(|(i, j)| *i < unknowns && *j < unknowns))
        {
            return Err(MathError::Contract(
                "direct elimination requires a proven affine residual in the unknown coordinates".into(),
            ));
        }
        if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
        Ok(Self {
            unknowns,
            inputs: body.input_count() - unknowns,
        })
    }
}
impl InnerSolver for Affine {
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::First
    }
    fn identity(&self) -> ContentHash {
        solver_identity("faer.affine.v1")
    }

    fn worker(&self) -> Box<dyn InnerWorker> {
        Box::new(self.clone())
    }
}
impl InnerWorker for Affine {
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, MathError> {
        problem.validate_options(options)?;
        if problem.unknowns.len() != self.unknowns || parameters.len() != self.inputs {
            return Err(MathError::Contract("affine coordinate extent".into()));
        }
        let started = std::time::Instant::now();
        let n = self.unknowns;
        let width = n + self.inputs;
        let jet = problem.evaluate(parameters, &options.start, DerivativeOrder::First, cancel)?;
        if jet.values.len() != n || jet.jacobian.len() != n * width {
            return Err(MathError::Contract("affine residual jet extent".into()));
        }
        let values = (0..n)
            .flat_map(|j| {
                problem
                    .pattern
                    .as_ref()
                    .row_idx_of_col(j)
                    .map(move |i| (i, j))
            })
            .map(|(i, j)| jet.jacobian[i * width + j])
            .collect();
        let matrix = SparseColMat::new(problem.pattern.clone(), values);
        let mut factor = problem.factorize_matrix(&matrix, "affine residual matrix is singular")?;
        let mut delta = Mat::from_fn(n, 1, |i, _| -jet.values[i]);
        let rhs = delta.clone();
        problem.solve_factor(&mut factor, delta.as_mut());
        check_solve(
            &jet.jacobian,
            width,
            &delta,
            &rhs,
            options.derivative_tolerance,
            problem.id,
        )?;
        drop(factor);
        let point = (0..n)
            .map(|i| options.start[i] + delta[(i, 0)])
            .collect::<Vec<_>>();
        problem.verify(parameters, &point, options, cancel)?;
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if started.elapsed() > options.time_limit {
            return Err(MathError::Limit("affine elimination time"));
        }
        Ok(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registered_affine_square_system_refactors_parameters_and_refuses_singular_or_out_of_bounds_roots() {
        use crate::guarded::{PreparedBody, Stage};
        use crate::implicit::accelerators::Accelerators;
        use symbolica::atom::{Atom, AtomCore};
        crate::initialize().unwrap();
        let id = SemanticId::from_bytes([148; 16]);
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let p = crate::library::formal(2).unwrap();
        let body = PreparedBody::new(3, 5, vec![3, 4], vec![Stage::Block {
            expressions: vec![p * x.clone() + y.clone() - Atom::num(1), x - y],
            outputs: vec![3, 4], source: id,
        }], DerivativeOrder::Second).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let solver = Accelerators::standard().admit("affine", &body, 2,
            crate::jets::EvaluationLimits::default(), &cancel).unwrap();
        let mut worker = solver.worker();
        let compiled = Arc::new(body.compile(&[0, 1], &[0, 1, 2], DerivativeOrder::Second,
            crate::library::Optimization::default(), crate::jets::EvaluationLimits::default(), &cancel).unwrap());
        let problem = Arc::new(Problem::new(id, ContentHash::from_bytes([0; 32]),
            vec![Unknown { id, lower: 0.0, upper: 2.0 }, Unknown {
                id: pse_ids::named_id(id, "y"), lower: 0.0, upper: 2.0 }],
            vec![pse_ids::named_id(id, "first"), pse_ids::named_id(id, "second")], 1, compiled, 100).unwrap());
        let options = Options { start: vec![0.0, 0.0], variable_nominals: vec![1.0; 2],
            variable_tolerance: vec![1e-10; 2], residual_tolerance: vec![1e-10; 2],
            iterations: 1, time_limit: Duration::from_secs(1), derivative_tolerance: 1e-12 };
        for (parameter, value, first, second) in [(1.0, 0.5, -0.25, 0.25), (3.0, 0.25, -0.0625, 0.03125)] {
            let root = worker.solve(problem.clone(), &[parameter], &options, &cancel).unwrap();
            let jet = problem.derivatives(&[parameter], &root, DerivativeOrder::Second, &options, &cancel).unwrap();
            for actual in jet.values { assert!((actual - value).abs() < 1e-12); }
            for actual in jet.jacobian { assert!((actual - first).abs() < 1e-12); }
            for actual in jet.hessians { assert!((actual - second).abs() < 1e-12); }
        }
        assert!(worker.solve(problem.clone(), &[-1.0], &options, &cancel).is_err(), "singular original matrix");
        assert!(worker.solve(problem.clone(), &[-2.0], &options, &cancel).is_err(), "original bound violation");
        cancel.store(true, Ordering::Release);
        assert!(matches!(worker.solve(problem, &[1.0], &options, &cancel), Err(MathError::Cancelled)));
    }
    #[test]
    fn affine_rate_elimination_preserves_second_implicit_jets_and_guards() {
        use crate::typed::{Binary, BodyBuilder, BodyLimits};
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let id = SemanticId::from_bytes([147; 16]);
        let mut b = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let x = b.input(0, q, pse_quantity::IndexSet::new(), id).unwrap();
        let p = b.input(1, q, pse_quantity::IndexSet::new(), id).unwrap();
        let one = b
            .literal(
                1.,
                registry
                    .unit(registry.quantity_type(q).unwrap().canonical_unit)
                    .unwrap(),
                pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                id,
            )
            .unwrap();
        let px = b.binary(Binary::Mul, p, x, None, id).unwrap();
        let r = b.binary(Binary::Sub, px, one, None, id).unwrap();
        let body = b.prepare(&[r]).unwrap();
        let mut solver = Affine::new(&body, 1).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let compiled = Arc::new(
            body.compile(
                &[0],
                &[0, 1],
                DerivativeOrder::Second,
                crate::library::Optimization::default(),
                crate::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap(),
        );
        let problem = Arc::new(
            Problem::new(
                id,
                ContentHash::from_bytes([0; 32]),
                vec![Unknown {
                    id,
                    lower: 0.,
                    upper: 2.,
                }],
                vec![pse_ids::named_id(id, "row")],
                1,
                compiled,
                100,
            )
            .unwrap(),
        );
        let options = Options {
            start: vec![0.],
            variable_nominals: vec![1.],
            variable_tolerance: vec![1e-10],
            residual_tolerance: vec![1e-10],
            iterations: 1,
            time_limit: Duration::from_secs(1),
            derivative_tolerance: 1e-12,
        };
        let root = solver
            .solve(problem.clone(), &[2.], &options, &cancel)
            .unwrap();
        let jet = problem
            .derivatives(&[2.], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(jet.values, vec![0.5]);
        assert_eq!(jet.jacobian, vec![-0.25]);
        assert_eq!(jet.hessians, vec![0.25]);
        assert!(
            solver
                .solve(problem.clone(), &[0.], &options, &cancel)
                .is_err()
        );
        assert!(
            solver
                .solve(problem.clone(), &[-1.], &options, &cancel)
                .is_err()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            solver.solve(problem, &[2.], &options, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}
