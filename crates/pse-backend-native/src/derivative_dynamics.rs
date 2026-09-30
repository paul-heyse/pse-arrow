// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A fixed dynamic function sample adapts to the existing library derivative checker.
use super::{Policy, Report, analyze};
use crate::{
    NlpOracle, OracleContract, ProblemError, Variable,
    dynamics::{self, Function},
    solve::Execution,
};
use faer::sparse::{SparseColMat, SymbolicSparseColMatRef, Triplet};
use pse_kernels::DerivativeOrder;

/// A local function sample; time and active mode remain fixed during perturbations.
#[derive(Clone, Debug)]
pub struct DynamicSample {
    /// Explicit active mode; perturbations cannot select another one.
    pub mode: usize,
    /// Existing compiled dynamic function to sample.
    pub function: Function,
    /// Physical time held fixed during this local comparison.
    pub time: f64,
    /// Normalized state coordinates.
    pub state: Vec<f64>,
    /// Parameters in the dynamic oracle's declared coordinates.
    pub parameters: Vec<f64>,
    /// The box each coordinate, states then parameters, stays inside in the same
    /// coordinates: the range the oracle's obligations admit, unbounded where none applies.
    /// A difference step never leaves it, so at an active bound it is one-sided into the
    /// box: backward at an upper bound, forward at a lower one.
    pub bounds: Vec<(f64, f64)>,
}
/// Compare raw state/parameter partials and declared sparsity with bounded library differences.
/// This is local numerical evidence; it does not establish smoothness across mode changes.
pub fn analyze_dynamic(
    oracle: Box<dyn dynamics::Oracle>,
    sample: DynamicSample,
    normalization: pse_math::normalization::Normalization,
    policy: Policy,
    execution: Execution,
) -> Result<Report, ProblemError> {
    policy.allowance()?;
    let contract = oracle.contract();
    contract.validate()?;
    let rows = match sample.function {
        Function::Rhs | Function::Initial | Function::Reset(_) => contract.states.clone(),
        Function::Output => contract.outputs.clone(),
        Function::QuadratureFlux => contract.quadratures.clone(),
        Function::Roots => contract
            .events
            .get(sample.mode)
            .ok_or_else(|| invalid("dynamic sample mode"))?
            .iter()
            .map(|e| e.id)
            .collect(),
    };
    let n = contract.states.len() + contract.parameters.len();
    if sample.mode >= contract.events.len()
        || !sample.time.is_finite()
        || sample.state.len() != contract.states.len()
        || sample.parameters.len() != contract.parameters.len()
        || sample.bounds.len() != n
        || sample
            .bounds
            .iter()
            .any(|(l, u)| l.is_nan() || u.is_nan() || l > u)
    {
        return Err(invalid("dynamic derivative sample dimensions"));
    }
    policy.check_extent(n, rows.len())?;
    let pattern = SparseColMat::try_new_from_triplets(
        rows.len(),
        n,
        &oracle
            .support(sample.mode, sample.function)
            .into_iter()
            .map(|entry| Triplet::new(entry.row.get(), entry.col.get(), 0.))
            .collect::<Vec<_>>(),
    )
    .map_err(|_| invalid("dynamic derivative support"))?;
    let contract = OracleContract {
        identity: contract.identity,
        variables: contract
            .states
            .iter()
            .chain(&contract.parameters)
            .zip(&sample.bounds)
            .map(|(id, &(lower, upper))| Variable {
                id: *id,
                lower,
                upper,
            })
            .collect(),
        rows,
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    let bounds = vec![(f64::NEG_INFINITY, f64::INFINITY); contract.rows.len()];
    let initial = sample
        .state
        .iter()
        .chain(&sample.parameters)
        .copied()
        .collect();
    analyze(
        Box::new(DynamicFunction {
            oracle,
            sample,
            contract,
            pattern,
            bounds,
        }),
        initial,
        normalization,
        policy,
        execution,
    )
}
fn invalid(message: &str) -> ProblemError {
    ProblemError::Contract(message.into())
}
#[derive(Debug)]
struct DynamicFunction {
    oracle: Box<dyn dynamics::Oracle>,
    sample: DynamicSample,
    contract: OracleContract,
    pattern: SparseColMat<usize, f64>,
    bounds: Vec<(f64, f64)>,
}
impl DynamicFunction {
    fn evaluate(
        &mut self,
        x: &[f64],
        derivatives: bool,
    ) -> Result<dynamics::Evaluation, ProblemError> {
        if x.len() != self.contract.variables.len() {
            return Err(invalid("dynamic sample coordinates"));
        }
        let (state, parameters) = x.split_at(self.sample.state.len());
        self.oracle.evaluate(
            self.sample.mode,
            self.sample.function,
            self.sample.time,
            state,
            parameters,
            derivatives,
        )
    }
}
impl NlpOracle for DynamicFunction {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> SymbolicSparseColMatRef<'_, usize> {
        self.pattern.symbolic()
    }
    fn hessian_pattern(&self) -> Option<SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.)
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let result = self.evaluate(x, false)?;
        if result.values.len() != out.len() {
            return Err(invalid("dynamic sample values"));
        }
        out.copy_from_slice(&result.values);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let result = self
            .evaluate(x, true)?
            .jacobian
            .ok_or_else(|| invalid("dynamic partials absent"))?;
        let pattern = self.pattern.symbolic();
        if result.nrows() != pattern.nrows()
            || result.ncols() != pattern.ncols()
            || out.len() != pattern.row_idx().len()
        {
            return Err(invalid("dynamic partial dimensions"));
        }
        for column in 0..pattern.ncols() {
            for index in pattern.col_range(column) {
                out[index] = result
                    .get(pattern.row_idx()[index], column)
                    .copied()
                    .unwrap_or(0.);
            }
        }
        Ok(())
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(invalid("dynamic derivative samples are first order"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{ContentHash, SemanticId};
    use std::sync::{Arc, atomic::AtomicBool};
    #[derive(Debug)]
    struct Polynomial {
        contract: dynamics::Contract,
        wrong: bool,
        missing: bool,
        /// The parameter range outside which an evaluation is refused, as an authored
        /// bound's guard refuses it.
        domain: (f64, f64),
    }
    impl dynamics::Oracle for Polynomial {
        fn contract(&self) -> &dynamics::Contract {
            &self.contract
        }
        fn support(&self, _: usize, _: Function) -> Vec<dynamics::SupportEntry> {
            dynamics::entries(if self.missing {
                vec![(0, 0)]
            } else {
                vec![(0, 0), (0, 1)]
            })
        }
        fn evaluate(
            &mut self,
            _: usize,
            _: Function,
            _: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<dynamics::Evaluation, ProblemError> {
            if parameters[0] < self.domain.0 || parameters[0] > self.domain.1 {
                return Err(ProblemError::Math(pse_math::MathError::OutsideRange {
                    source_id: SemanticId::from_bytes([2; 16]),
                    target: SemanticId::from_bytes([2; 16]),
                    value: parameters[0],
                    lower: Some(self.domain.0),
                    upper: Some(self.domain.1),
                }));
            }
            Ok(dynamics::Evaluation {
                values: vec![state[0] * state[0] * parameters[0]],
                jacobian: derivatives.then(|| {
                    SparseColMat::try_new_from_triplets(
                        1,
                        2,
                        &[
                            Triplet::new(
                                0,
                                0,
                                if self.wrong {
                                    0.
                                } else {
                                    2. * state[0] * parameters[0]
                                },
                            ),
                            Triplet::new(0, 1, state[0] * state[0]),
                        ],
                    )
                    .unwrap()
                }),
            })
        }
    }
    fn polynomial(wrong: bool, missing: bool, domain: (f64, f64)) -> Polynomial {
        let id = |n| SemanticId::from_bytes([n; 16]);
        Polynomial {
            contract: dynamics::Contract {
                identity: ContentHash::from_bytes([8; 32]),
                states: vec![id(1)],
                differential: vec![true],
                parameters: vec![id(2)],
                outputs: vec![id(3)],
                events: vec![vec![]],
                signs: vec![],
                quadratures: vec![],
                balances: vec![],
                derivatives: DerivativeOrder::First,
            },
            wrong,
            missing,
            domain,
        }
    }
    /// Sample the output at state 2 and `parameter` within `bounds`.
    fn sample(oracle: Polynomial, parameter: f64, bounds: Vec<(f64, f64)>) -> Report {
        analyze_dynamic(
            Box::new(oracle),
            DynamicSample {
                mode: 0,
                function: Function::Output,
                time: 0.5,
                state: vec![2.],
                parameters: vec![parameter],
                bounds,
            },
            pse_math::normalization::Normalization::identity(2, 1),
            Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                maximum_cells: 100,
            },
            Execution::new(
                Arc::new(AtomicBool::new(false)),
                &crate::solve::Controls::default(),
            ),
        )
        .unwrap()
    }
    const FREE: (f64, f64) = (f64::NEG_INFINITY, f64::INFINITY);
    #[test]
    fn dynamic_derivative_sampling_detects_wrong_partials_and_missing_support() {
        for (wrong, missing) in [(false, false), (true, false), (false, true)] {
            let report = sample(polynomial(wrong, missing, FREE), 3., vec![FREE; 2]);
            assert!(report.complete);
            assert_eq!(report.passed(), !wrong && !missing, "{:?}", report.sample);
        }
    }
    /// An input held at the edge of the range its guard admits is differenced into the
    /// range: backward at its upper bound and forward at its lower one, never across it.
    #[test]
    fn derivative_sample_uses_one_sided_step_at_an_active_bound() {
        let domain = (0., 1.);
        for parameter in [1., 0.] {
            let report = sample(
                polynomial(false, false, domain),
                parameter,
                vec![FREE, domain],
            );
            assert_eq!(report.rejected_evaluations, 0, "{parameter}");
            assert!(report.complete, "{parameter}: {:?}", report.sample);
            assert!(report.passed(), "{parameter}: {:?}", report.sample);
        }
        // Control: without the box the step at the upper bound leaves the range, the
        // guard refuses the trial, and the sample is incomplete.
        let unbounded = sample(polynomial(false, false, domain), 1., vec![FREE; 2]);
        assert!(unbounded.rejected_evaluations > 0);
        assert!(!unbounded.complete);
        // A box of the wrong extent is refused.
        let refused = analyze_dynamic(
            Box::new(polynomial(false, false, domain)),
            DynamicSample {
                mode: 0,
                function: Function::Output,
                time: 0.5,
                state: vec![2.],
                parameters: vec![1.],
                bounds: vec![FREE],
            },
            pse_math::normalization::Normalization::identity(2, 1),
            Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                maximum_cells: 100,
            },
            Execution::new(
                Arc::new(AtomicBool::new(false)),
                &crate::solve::Controls::default(),
            ),
        );
        assert!(matches!(refused, Err(ProblemError::Contract(_))));
    }
}
