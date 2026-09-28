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
    let expected = n
        .checked_add(jac.rows.len())
        .ok_or_else(|| ProblemError::Contract("derivative comparison extent".into()))?;
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
}
