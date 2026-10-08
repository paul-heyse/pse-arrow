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
use pounce_nlp::derivative_test::DerivativeTestReport;
use pounce_nlp::tnlp::{BoundsInfo, SparsityRequest, TNLP};
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
    /// Actual dimension-based diagnostic construction, with the original cell
    /// maximum remaining an eligibility guard rather than initial demand.
    /// # Errors
    /// Invalid policy, oversized dimensions or allocation overflow.
    pub fn construction_allocation_bound(
        self,
        variables: usize,
        rows: usize,
    ) -> Result<usize, ProblemError> {
        self.allowance()?;
        self.check_extent(variables, rows)?;
        rows.checked_add(1)
            .and_then(|rows| variables.checked_mul(rows))
            .and_then(|cells| cells.checked_mul(1024))
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
    /// Original typed callback witness when sampling cannot continue.
    /// A rejected sample must not erase its domain, selector or task-scope cause.
    pub failure: Option<ProblemError>,
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
            || {
                let cause = self.failure.as_ref().map_or_else(
                    || "no callback cause".into(),
                    ToString::to_string,
                );
                format!(
                    "no completed derivative sample; {} rejected evaluations; termination {:?}; {cause}",
                    self.rejected_evaluations, self.terminal,
                )
            },
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
    let sample = bounded_sample(&mut adapter, policy);
    let terminal = adapter
        .state
        .terminal
        .as_ref()
        .map(|v| v.0)
        .or_else(|| execution.stopped());
    let rejected_evaluations = adapter.state.rejected_evaluations;
    let failure = adapter
        .state
        .terminal_error()
        .or_else(|| adapter.state.last_failure.take());
    let complete = terminal.is_none()
        && rejected_evaluations == 0
        && expected > 0
        && sample.as_ref().is_some_and(|r| r.checked == expected);
    Ok(Report {
        sample,
        complete,
        rejected_evaluations,
        terminal,
        failure,
    })
}

/// The library owns the differences. A one-dimensional chart supplies exactly the
/// declared normalized step; central differences apply inside the box, and forward
/// differences point inward at an active bound. No analytic derivative enters the
/// library's value callback, including for structurally absent entries.
fn bounded_sample(adapter: &mut Adapter, policy: Policy) -> Option<DerivativeTestReport> {
    let n = adapter.initial.len();
    let m = adapter.oracle.contract().rows.len();
    let mut x = adapter
        .normalization
        .normalized_point(&adapter.initial)
        .ok()?;
    let (mut lower, mut upper) = (vec![0.; n], vec![0.; n]);
    let (mut gl, mut gu) = (vec![0.; m], vec![0.; m]);
    if !adapter.get_bounds_info(BoundsInfo {
        x_l: &mut lower,
        x_u: &mut upper,
        g_l: &mut gl,
        g_u: &mut gu,
    }) {
        return None;
    }
    for i in 0..n {
        x[i] = x[i].clamp(lower[i], upper[i]);
    }
    let mut gradient = vec![0.; n];
    let mut jacobian = vec![0.; adapter.jac.rows.len()];
    if adapter.eval_f(&x, true).is_none()
        || !adapter.eval_grad_f(&x, false, &mut gradient)
        || !adapter.eval_jac_g(
            Some(&x),
            false,
            SparsityRequest::Values {
                values: &mut jacobian,
            },
        )
    {
        return None;
    }
    let entries = adapter
        .jac
        .rows
        .iter()
        .zip(&adapter.jac.columns)
        .enumerate()
        .map(|(index, (row, column))| {
            Some((
                (usize::try_from(*row).ok()?, usize::try_from(*column).ok()?),
                index,
            ))
        })
        .collect::<Option<std::collections::BTreeMap<_, _>>>()?;
    let mut report = DerivativeTestReport {
        evaluations: 3,
        ..Default::default()
    };
    report.lines.push(format!("Library derivative sample: central inside bounds, forward inward at active bounds; normalized perturbation {:.1e}, tolerance {:.1e}.",policy.perturbation,policy.relative_tolerance));
    for column in 0..n {
        if lower[column] == upper[column] {
            continue;
        }
        let h = policy.perturbation * x[column].abs().max(1.);
        let up = x[column] + h <= upper[column] && x[column] + h != x[column];
        let down = x[column] - h >= lower[column] && x[column] - h != x[column];
        if !up && !down {
            continue;
        }
        let central = up && down;
        let epsilon = if central {
            f64::EPSILON.cbrt()
        } else {
            f64::EPSILON.sqrt()
        };
        let signed = if up { h } else { -h };
        let calls = std::cell::Cell::new(0usize);
        let numeric = {
            let callback = std::cell::RefCell::new(&mut *adapter);
            let values = |offset: &Vec<f64>| {
                let mut point = x.clone();
                point[column] += offset[0];
                let mut adapter = callback.borrow_mut();
                calls.set(calls.get() + 1 + usize::from(m > 0));
                let Some(objective) = adapter.eval_f(&point, true) else {
                    return Err(std::io::Error::other("derivative value callback rejected").into());
                };
                let mut result = vec![0.; m + 1];
                result[0] = objective;
                if m > 0 && !adapter.eval_g(&point, false, &mut result[1..]) {
                    return Err(std::io::Error::other("derivative row callback rejected").into());
                }
                Ok(result)
            };
            let direction = vec![signed / epsilon];
            if central {
                finitediff::vec::central_jacobian_vec_prod(&values)(&vec![0.], &direction)
            } else {
                finitediff::vec::forward_jacobian_vec_prod(&values)(&vec![0.], &direction)
            }
        }
        .ok()?;
        report.evaluations += calls.get();
        if numeric.len() != m + 1 {
            return None;
        }
        let numeric = numeric
            .into_iter()
            .map(|value| value * epsilon / signed)
            .collect::<Vec<_>>();
        if !numeric
            .iter()
            .chain(&gradient)
            .chain(&jacobian)
            .all(|value| value.is_finite())
        {
            return None;
        }
        comparison(
            &mut report,
            format!("grad_f[{column:5}]"),
            gradient[column],
            numeric[0],
            policy.relative_tolerance,
        );
        for row in 0..m {
            if let Some(index) = entries.get(&(row, column)) {
                comparison(
                    &mut report,
                    format!("jac_g [{row:5},{column:5}]"),
                    jacobian[*index],
                    numeric[row + 1],
                    policy.relative_tolerance,
                );
            } else if numeric[row + 1].abs() > policy.relative_tolerance {
                report.missing_structure += 1;
                report.lines.push(format!(
                    "! jac_g [{row:5},{column:5}] is absent from sparsity, sampled {:.6e}",
                    numeric[row + 1]
                ));
            }
        }
    }
    Some(report)
}
fn comparison(
    report: &mut DerivativeTestReport,
    label: String,
    analytic: f64,
    numeric: f64,
    tolerance: f64,
) {
    report.checked += 1;
    let relative = (analytic - numeric).abs() / numeric.abs().max(1.);
    if relative > tolerance {
        report.suspicious += 1;
        report.lines.push(format!(
            "* {label} = {analytic:.16e} ~ {numeric:.16e} [{relative:.3e}]"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_demand_uses_dimensions_without_changing_original_cell_guard() {
        let policy = Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 100_000_000,
        };
        assert_eq!(
            policy.construction_allocation_bound(2, 1).unwrap(),
            4 * 1024
        );
        assert_eq!(
            Policy {
                maximum_cells: 4,
                ..policy
            }
            .construction_allocation_bound(2, 1)
            .unwrap(),
            4 * 1024
        );
        assert!(
            Policy {
                maximum_cells: 3,
                ..policy
            }
            .construction_allocation_bound(2, 1)
            .is_err()
        );
        assert!(policy.construction_allocation_bound(usize::MAX, 1).is_err());
        assert!(policy.construction_allocation_bound(1, usize::MAX).is_err());
    }
    #[test]
    fn interior_central_sample_avoids_forward_truncation_bias() {
        // For x^3 at x=2, a forward step of 2e-4 incurs O(h) relative bias;
        // the declared central sample incurs O(h^2), below the same strict tolerance.
        let report = analyze(
            Box::new(crate::solver_tests::Polynomial::new()),
            vec![2.],
            Normalization::identity(1, 1),
            Policy {
                perturbation: 1e-4,
                relative_tolerance: 1e-8,
                maximum_cells: 2,
            },
            Execution::new(
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                &Default::default(),
            ),
        )
        .unwrap();
        assert!(report.passed(), "{}", report.summary());
        assert_eq!(report.sample.unwrap().checked, 2);
    }
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
        assert!(failed.failure.is_some());
        assert!(failed.summary().contains("rejected evaluations"));
        assert!(
            failed
                .summary()
                .contains(&failed.failure.as_ref().unwrap().to_string())
        );
        let mut fixed = crate::solver_tests::Polynomial::new();
        fixed.c.variables[0].lower = 2.;
        fixed.c.variables[0].upper = 2.;
        assert!(!run(fixed, false).complete);
        let stopped = run(crate::solver_tests::Polynomial::new(), true);
        assert!(!stopped.complete);
        assert_eq!(stopped.terminal, Some(crate::solve::Termination::Cancelled));
        assert!(matches!(stopped.failure, Some(ProblemError::Cancelled)));
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
        fn hessian(
            &mut self,
            _: &[f64],
            _: f64,
            _: &[f64],
            _: &mut [f64],
        ) -> Result<(), ProblemError> {
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
