// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded library derivative checks through the existing physical TNLP adapter.
#[path = "derivative_dynamics.rs"]
mod dynamic;
use crate::{
    NlpOracle, ProblemError, callback::CallbackState, nlp_pattern::Pattern, solve::Execution,
    tnlp::Adapter,
};
pub use dynamic::{DynamicSample, analyze_dynamic};
use pounce_nlp::derivative_test::{
    self, DerivativeTest, DerivativeTestOptions, DerivativeTestReport,
};
use pse_math::normalization::Normalization;

/// First derivative and sparsity sample policy, in normalized coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Finite-difference step, in (0, 1).
    pub perturbation: f64,
    /// Relative disagreement above which an entry is reported as suspicious.
    pub relative_tolerance: f64,
    /// Bound the dense inspection, including structurally omitted entries.
    pub maximum_cells: usize,
}
impl Policy {
    /// Validate the policy and return the allocation allowance it implies.
    pub fn allowance(self) -> Result<usize, ProblemError> {
        if !self.perturbation.is_finite()
            || self.perturbation <= 0.
            || self.perturbation >= 1.
            || !self.relative_tolerance.is_finite()
            || self.relative_tolerance <= 0.
            || self.maximum_cells == 0
        {
            return Err(ProblemError::Contract(
                "derivative diagnostic policy".into(),
            ));
        }
        self.maximum_cells
            .checked_mul(1024)
            .ok_or_else(|| pse_math::MathError::Limit("derivative diagnostic allocation").into())
    }
    fn check_extent(self, variables: usize, rows: usize) -> Result<(), ProblemError> {
        if rows
            .checked_add(1)
            .and_then(|m| variables.checked_mul(m))
            .is_none_or(|cells| cells > self.maximum_cells)
        {
            return Err(pse_math::MathError::Limit("derivative diagnostic cells").into());
        }
        Ok(())
    }
}
/// Numerical sampling evidence; never a proof of global derivative correctness.
#[derive(Debug)]
pub struct Report {
    /// Library comparison report, absent when no sample completed.
    pub sample: Option<DerivativeTestReport>,
    /// Whether every expected comparison ran without rejection or termination.
    pub complete: bool,
    /// Callback evaluations the oracle rejected during sampling.
    pub rejected_evaluations: usize,
    /// Termination that interrupted sampling, if any.
    pub terminal: Option<crate::solve::Termination>,
}
impl Report {
    /// Whether the sample completed with at least one comparison and no suspicious entry.
    pub fn passed(&self) -> bool {
        self.complete
            && self
                .sample
                .as_ref()
                .is_some_and(|s| s.checked > 0 && s.clean())
    }
    /// Library comparison details retain the analytic and sampled values, not just counts.
    pub fn summary(&self) -> String {
        self.sample.as_ref().map_or_else(
            || "no completed derivative sample".into(),
            |sample| format!(
                "{} comparisons; {} suspicious entries; {} missing sparsity entries; {} rejected evaluations\n{}",
                sample.checked, sample.suspicious, sample.missing_structure,
                self.rejected_evaluations, sample.lines.join("\n"),
            ),
        )
    }
}
/// Sample first derivatives and sparsity at `initial` within the policy's bounds.
pub fn analyze(
    oracle: Box<dyn NlpOracle>,
    initial: Vec<f64>,
    normalization: Normalization,
    policy: Policy,
    execution: Execution,
) -> Result<Report, ProblemError> {
    policy.allowance()?;
    crate::validate_nlp(oracle.as_ref(), pse_kernels::DerivativeOrder::First)?;
    let n = oracle.contract().variables.len();
    let m = oracle.contract().rows.len();
    if initial.len() != n || initial.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract("derivative diagnostic point".into()));
    }
    policy.check_extent(n, m)?;
    normalization.validate(n, m)?;
    let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
    // A coordinate its box fixes has no direction to difference along, so the library
    // compares neither its gradient entry nor its Jacobian column. Every other coordinate
    // owes both; one the step cannot move inside its box leaves the sample incomplete.
    let moves: Vec<bool> = oracle
        .contract()
        .variables
        .iter()
        .map(|v| v.lower < v.upper)
        .collect();
    let expected = moves.iter().filter(|m| **m).count()
        + jac
            .columns
            .iter()
            .filter(|c| usize::try_from(**c).is_ok_and(|c| moves[c]))
            .count();
    // No Hessian demand or native solver construction is needed for this sample.
    let mut adapter = Adapter {
        normalization,
        certification_budget: None,
        normalize_affine: false,
        oracle,
        state: CallbackState::new(execution.clone()),
        jac,
        hess: Pattern {
            rows: vec![],
            columns: vec![],
        },
        initial,
        duals: None,
        solution: None,
    };
    let sample = derivative_test::run(
        &mut adapter,
        &DerivativeTestOptions {
            mode: DerivativeTest::FirstOrder,
            perturbation: policy.perturbation,
            tol: policy.relative_tolerance,
            first_index: -2,
            print_all: false,
        },
    );
    let terminal = adapter
        .state
        .terminal
        .as_ref()
        .map(|v| v.0)
        .or_else(|| execution.stopped());
    let rejected_evaluations = adapter.state.rejected_evaluations;
    let complete = terminal.is_none()
        && rejected_evaluations == 0
        && expected > 0
        && sample.as_ref().is_some_and(|r| r.checked == expected);
    Ok(Report {
        sample,
        complete,
        rejected_evaluations,
        terminal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derivative_sample_budget_refusal_is_not_an_invalid_model() {
        let run = |initial| {
            analyze(
                Box::new(crate::solver_tests::Polynomial::new()),
                initial,
                Normalization::identity(1, 1),
                Policy {
                    perturbation: 1e-6,
                    relative_tolerance: 1e-4,
                    maximum_cells: 1,
                },
                Execution::new(
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    &Default::default(),
                ),
            )
        };
        assert!(matches!(
            run(vec![2.]),
            Err(ProblemError::Math(pse_math::MathError::Limit(
                "derivative diagnostic cells"
            )))
        ));
        assert!(matches!(
            run(vec![f64::NAN]),
            Err(ProblemError::Contract(_))
        ));
    }
    #[test]
    fn library_derivative_sample_requires_complete_callbacks_and_nonempty_comparisons() {
        let policy = Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 10,
        };
        let run = |oracle: crate::solver_tests::Polynomial, flag: bool| {
            analyze(
                Box::new(oracle),
                vec![2.],
                Normalization::identity(1, 1),
                policy,
                Execution::new(
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(flag)),
                    &Default::default(),
                ),
            )
            .unwrap()
        };
        assert!(run(crate::solver_tests::Polynomial::new(), false).passed());
        let mut bad = crate::solver_tests::Polynomial::new();
        bad.fail = true;
        let failed = run(bad, false);
        assert!(!failed.complete);
        assert!(!failed.passed());
        assert!(failed.rejected_evaluations > 0);
        let mut fixed = crate::solver_tests::Polynomial::new();
        fixed.c.variables[0].lower = 2.;
        fixed.c.variables[0].upper = 2.;
        assert!(!run(fixed, false).complete);
        let stopped = run(crate::solver_tests::Polynomial::new(), true);
        assert!(!stopped.complete);
        assert_eq!(stopped.terminal, Some(crate::solve::Termination::Cancelled));
    }
    /// `min x₀² + x₁²` subject to `x₀·x₁ = 1`, with the box of `x₁` given per case.
    #[derive(Debug)]
    struct Pair {
        contract: crate::OracleContract,
        pattern: faer::sparse::SparseColMat<usize, f64>,
        bounds: Vec<(f64, f64)>,
    }
    impl Pair {
        fn new(lower: f64, upper: f64) -> Self {
            let variable = |n: u8, lower, upper| crate::Variable {
                id: pse_ids::SemanticId::from_bytes([n; 16]),
                lower,
                upper,
            };
            Self {
                contract: crate::OracleContract {
                    identity: pse_ids::ContentHash::from_bytes([7; 32]),
                    variables: vec![
                        variable(1, f64::NEG_INFINITY, f64::INFINITY),
                        variable(2, lower, upper),
                    ],
                    rows: vec![pse_ids::SemanticId::from_bytes([3; 16])],
                    derivatives: pse_kernels::DerivativeOrder::First,
                    smoothness: pse_kernels::DerivativeOrder::First,
                },
                pattern: faer::sparse::SparseColMat::try_new_from_triplets(
                    1,
                    2,
                    &[
                        faer::sparse::Triplet::new(0, 0, 0.),
                        faer::sparse::Triplet::new(0, 1, 0.),
                    ],
                )
                .unwrap(),
                bounds: vec![(1., 1.)],
            }
        }
    }
    impl NlpOracle for Pair {
        fn contract(&self) -> &crate::OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.pattern.symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            None
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &self.bounds
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(x[0] * x[0] + x[1] * x[1])
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out.copy_from_slice(&[2. * x[0], 2. * x[1]]);
            Ok(())
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] * x[1];
            Ok(())
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out.copy_from_slice(&[x[1], x[0]]);
            Ok(())
        }
        fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
            Err(ProblemError::Contract("first-order sample".into()))
        }
    }
    /// A coordinate its box fixes (equal bounds, as an authored `bounds x(T, T)` gives) has
    /// no direction; the sample expects no comparison for it, and completes with the others.
    #[test]
    fn derivative_sample_expects_no_comparison_along_a_coordinate_its_box_fixes() {
        let run = |oracle: Pair| {
            analyze(
                Box::new(oracle),
                vec![2., 0.5],
                Normalization::identity(2, 1),
                Policy {
                    perturbation: 1e-6,
                    relative_tolerance: 1e-4,
                    maximum_cells: 100,
                },
                Execution::new(
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    &Default::default(),
                ),
            )
            .unwrap()
        };
        let fixed = run(Pair::new(0.5, 0.5));
        let sample = fixed.sample.as_ref().unwrap();
        // The gradient entry and Jacobian entry of the free coordinate only.
        assert_eq!(sample.checked, 2);
        assert!(fixed.complete, "{:?}", fixed.sample);
        assert!(fixed.passed());
        // Control: a free second coordinate owes its own two comparisons.
        let free = run(Pair::new(0., 1.));
        assert_eq!(free.sample.as_ref().unwrap().checked, 4);
        assert!(free.complete && free.passed());
    }
}
