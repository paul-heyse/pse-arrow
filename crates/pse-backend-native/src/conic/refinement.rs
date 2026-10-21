// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One observed-quality native correction, never a new nonlinear iteration owner.
use super::{
    ConicProblem, Controls, Execution, ProblemError, ResolvedAccuracy, Session, Settings,
    SolveReport, Termination, Tolerances,
};

#[derive(serde::Serialize)]
struct Attempt {
    native_invoked: bool,
    native: crate::solve::NativeTermination,
    working_feasibility: f64,
    original_quality_ratio: Option<f64>,
    iterations: Option<u64>,
    native_primal_residual: Option<f64>,
}
impl Attempt {
    fn from_report(report: &SolveReport, feasibility: f64) -> Self {
        Self {
            native_invoked: report.evidence.conic.is_some(),
            native: report.termination.clone(),
            working_feasibility: feasibility,
            original_quality_ratio: report
                .quality
                .as_ref()
                .map(|quality| quality.normalized_max),
            iterations: report.evidence.work.iterations,
            native_primal_residual: report.evidence.conic.map(|conic| conic.primal_residual),
        }
    }
}

// This is a working-control heuristic. No residual-norm equivalence or guaranteed
// refinement success is inferred; the next original assessment remains authoritative.
fn next_controls(
    report: &SolveReport,
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    execution: &Execution,
) -> Option<(Controls, ResolvedAccuracy)> {
    if report.termination.category != Termination::Success
        || report.certificate.is_some()
        || report.validation_failure().is_some()
        || execution.stopped().is_some()
    {
        return None;
    }
    let ratio = report.quality.as_ref()?.normalized_max;
    let residual = report.evidence.conic?.primal_residual;
    if !(ratio.is_finite() && ratio > 1. && residual.is_finite() && residual > 0.) {
        return None;
    }
    let working = (residual / ratio / 8.).min(accuracy.feasibility / 8.);
    if !(working.is_finite() && working > 0. && working < accuracy.feasibility) {
        return None;
    }
    let used = u32::try_from(report.evidence.work.iterations?).ok()?;
    let iterations = controls
        .iterations
        .checked_sub(used)
        .filter(|remaining| *remaining > 0)?;
    let elapsed = execution.started.elapsed();
    let time_limit = controls
        .time_limit
        .min(execution.time_limit)
        .saturating_sub(elapsed);
    if time_limit.is_zero() {
        return None;
    }
    Some((
        Controls {
            iterations,
            time_limit,
            ..controls.clone()
        },
        ResolvedAccuracy {
            feasibility: working,
            ..accuracy.clone()
        },
    ))
}

fn late_stop(report: &mut SolveReport, execution: &Execution) {
    if let Some(stop) = execution.stopped() {
        report.termination.category = stop;
        report.termination.assurance = crate::solve::Assurance::None;
    }
}

impl Session {
    /// Solve with native preprocessing and at most one original-quality correction.
    /// Both native calls share the original execution clock and iteration allowance.
    /// No external iterate warm-start capability is claimed.
    pub fn solve(
        &mut self,
        problem: &ConicProblem,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        settings: &Settings,
        execution: Execution,
        tolerances: &Tolerances,
    ) -> Result<SolveReport, ProblemError> {
        let mut first = self.solve_once(
            problem,
            controls,
            accuracy,
            settings,
            execution.clone(),
            tolerances,
        )?;
        let Some((remaining, working)) = next_controls(&first, controls, accuracy, &execution)
        else {
            late_stop(&mut first, &execution);
            return Ok(first);
        };
        let first_attempt = Attempt::from_report(&first, accuracy.feasibility);
        let used = first.evidence.work.iterations;
        // Retire the first candidate, row-quality arrays and copied event history
        // before constructing another result. Only fixed-size attempt summaries remain.
        drop(first);
        let mut final_report = crate::observation::scoped("clarabel.refinement", || {
            self.solve_once(
                problem,
                &remaining,
                &working,
                settings,
                execution.clone(),
                tolerances,
            )
        })?;
        let final_attempt = Attempt::from_report(&final_report, working.feasibility);
        final_report.evidence.work.iterations = used
            .zip(final_report.evidence.work.iterations)
            .and_then(|(first, second)| first.checked_add(second));
        final_report.metrics.insert(
            "native.refinement.calls".into(),
            crate::solve::Metric::Integer(1 + i64::from(final_attempt.native_invoked)),
        );
        if let Some(iterations) = final_report
            .evidence
            .work
            .iterations
            .and_then(|n| i64::try_from(n).ok())
        {
            final_report.metrics.insert(
                "native.refinement.total_iterations".into(),
                crate::solve::Metric::Integer(iterations),
            );
        }
        final_report.provenance.insert(
            "native.refinement.attempts".into(),
            serde_json::to_string(&[first_attempt, final_attempt])
                .map_err(|error| ProblemError::Internal(error.to_string()))?,
        );
        final_report.provenance.insert("native.refinement.policy".into(),
            "one observed-primal/original-quality correction; shared original clock and iterations; fresh native iterate; original physical budgets unchanged".into());
        late_stop(&mut final_report, &execution);
        Ok(final_report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        conic::{Cone, SparseMatrix},
        solve::{Assurance, Backend},
    };

    fn fixture(
        magnitude: f64,
        coefficient: f64,
        psd: bool,
    ) -> (ConicProblem, Controls, ResolvedAccuracy, Tolerances) {
        let mut contract = crate::solver_tests::contract();
        contract.variables[0].lower = 0.;
        contract.rows = (0..if psd { 3 } else { 2 })
            .map(|i| crate::solver_tests::id(3 + i))
            .collect();
        let (constraints, rhs, cones) = if psd {
            (
                SparseMatrix::new(3, 1, vec![0, 2], vec![0, 2], vec![-coefficient; 2]),
                vec![0., std::f64::consts::SQRT_2 * magnitude, 0.],
                vec![Cone::PsdTriangle { order: 2 }],
            )
        } else {
            (
                SparseMatrix::new(2, 1, vec![0, 1], vec![0], vec![-coefficient]),
                vec![0., magnitude],
                vec![Cone::SecondOrder { dimension: 2 }],
            )
        };
        let rows = rhs.len();
        (
            ConicProblem {
                contract,
                quadratic: SparseMatrix::zeros(1, 1),
                objective: vec![coefficient],
                constraints,
                rhs,
                cones,
                objective_constant: 0.,
            },
            Controls {
                iterations: 100,
                time_limit: std::time::Duration::from_secs(2),
                ..Default::default()
            },
            ResolvedAccuracy {
                feasibility: 0.001,
                gap_absolute: 0.001,
                gap_relative: 0.001,
                ..ResolvedAccuracy::verification()
            },
            Tolerances {
                variables: vec![0.001 / coefficient],
                rows: vec![0.001; rows],
                integrality: 0.001,
            },
        )
    }
    fn session(
        problem: &ConicProblem,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
    ) -> Session {
        let zero = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
        let certificate = crate::solver_tests::certify(&zero, 1.);
        Session::new(
            problem,
            &certificate,
            controls,
            accuracy,
            &Settings::default(),
            crate::solver_tests::stamp(Backend::Clarabel),
        )
        .unwrap()
    }

    #[test]
    fn bounded_refinement_preserves_physical_budget_status_and_total_iterations() {
        for magnitude in [1., 1e6, 1e9] {
            for coefficient in [1., 1e6] {
                let (problem, controls, accuracy, tolerances) =
                    fixture(magnitude, coefficient, false);
                let report = session(&problem, &controls, &accuracy)
                    .solve(
                        &problem,
                        &controls,
                        &accuracy,
                        &Settings::default(),
                        Execution::new(Default::default(), &controls),
                        &tolerances,
                    )
                    .unwrap();
                if let (Some(candidate), Some(quality)) = (&report.candidate, &report.quality) {
                    let distance = (magnitude - coefficient * candidate.primal[0]).max(0.);
                    assert_eq!(
                        quality.feasible(),
                        distance <= 0.001 && candidate.primal[0] >= -0.001 / coefficient
                    );
                } else {
                    assert_ne!(report.termination.category, Termination::Success);
                }
                assert!(report.evidence.work.iterations.unwrap() <= u64::from(controls.iterations));
                if magnitude == 1e6 && coefficient == 1. {
                    assert!(report.provenance.contains_key("native.refinement.attempts"));
                }
                if magnitude == 1. {
                    assert!(!report.provenance.contains_key("native.refinement.attempts"));
                } else if report.provenance.contains_key("native.refinement.attempts") {
                    let attempts: serde_json::Value =
                        serde_json::from_str(&report.provenance["native.refinement.attempts"])
                            .unwrap();
                    assert_eq!(attempts.as_array().unwrap().len(), 2);
                    assert_eq!(attempts[0]["native"]["name"], "Solved");
                    assert_eq!(attempts[0]["working_feasibility"], 0.001);
                    assert!(attempts[0]["original_quality_ratio"].as_f64().unwrap() > 1.);
                    assert!(attempts[1]["working_feasibility"].as_f64().unwrap() < 0.001);
                    assert_eq!(
                        report.evidence.work.iterations.unwrap(),
                        attempts[0]["iterations"].as_u64().unwrap()
                            + attempts[1]["iterations"].as_u64().unwrap()
                    );
                    assert_eq!(attempts[1]["native"]["name"], report.termination.name);
                    if magnitude == 1e6 && coefficient == 1. {
                        assert!(report.quality.as_ref().unwrap().feasible());
                        assert_eq!(report.termination.category, Termination::Success);
                    }
                    if report.termination.category != Termination::Success {
                        assert_eq!(report.termination.assurance, Assurance::None);
                    }
                }
            }
        }
    }

    #[test]
    fn refinement_never_renews_exhausted_effort_or_retries_unknown_quality() {
        let (problem, controls, accuracy, tolerances) = fixture(1e6, 1., false);
        let execution = Execution::new(Default::default(), &controls);
        let mut owner = session(&problem, &controls, &accuracy);
        let first = owner
            .solve_once(
                &problem,
                &controls,
                &accuracy,
                &Settings::default(),
                execution.clone(),
                &tolerances,
            )
            .unwrap();
        let before = execution.started.elapsed();
        let (remaining, _) = next_controls(&first, &controls, &accuracy, &execution).unwrap();
        let after = execution.started.elapsed();
        assert_eq!(
            u64::from(remaining.iterations) + first.evidence.work.iterations.unwrap(),
            u64::from(controls.iterations)
        );
        assert!(remaining.time_limit <= controls.time_limit.saturating_sub(before));
        assert!(remaining.time_limit >= controls.time_limit.saturating_sub(after));
        let exhausted = Controls {
            iterations: u32::try_from(first.evidence.work.iterations.unwrap()).unwrap(),
            ..controls.clone()
        };
        assert!(next_controls(&first, &exhausted, &accuracy, &execution).is_none());
        let mut unknown = first.clone();
        unknown.quality = None;
        assert!(next_controls(&unknown, &controls, &accuracy, &execution).is_none());
        unknown = first.clone();
        unknown.evidence.conic.as_mut().unwrap().primal_residual = 0.;
        assert!(next_controls(&unknown, &controls, &accuracy, &execution).is_none());
        unknown = first.clone();
        unknown.termination.category = Termination::Numerical;
        assert!(next_controls(&unknown, &controls, &accuracy, &execution).is_none());
        let mut expired = execution.clone();
        expired.time_limit = std::time::Duration::ZERO;
        assert!(next_controls(&first, &controls, &accuracy, &expired).is_none());
        execution.cancellation.cancel();
        assert!(next_controls(&first, &controls, &accuracy, &execution).is_none());
        let stopped = owner
            .solve(
                &problem,
                &controls,
                &accuracy,
                &Settings::default(),
                execution,
                &tolerances,
            )
            .unwrap();
        assert_eq!(stopped.termination.category, Termination::Cancelled);
        assert_eq!(stopped.evidence.work.iterations, Some(0));
        assert!(stopped.evidence.conic.is_none());
        assert!(
            stopped.candidate.is_none(),
            "prior native candidate cannot escape a cancelled invocation"
        );
        assert!(
            !stopped
                .provenance
                .contains_key("native.refinement.attempts")
        );
    }

    #[test]
    fn refinement_reports_remaining_iteration_limit_without_promoting_first_success() {
        let (problem, mut controls, accuracy, tolerances) = fixture(1e6, 1., false);
        controls.iterations = 5;
        let report = session(&problem, &controls, &accuracy)
            .solve(
                &problem,
                &controls,
                &accuracy,
                &Settings::default(),
                Execution::new(Default::default(), &controls),
                &tolerances,
            )
            .unwrap();
        assert!(report.provenance.contains_key("native.refinement.attempts"));
        assert_eq!(report.termination.category, Termination::IterationLimit);
        assert_eq!(report.evidence.work.iterations, Some(5));
        assert_eq!(report.termination.assurance, Assurance::None);
        assert!(!report.quality.unwrap().feasible());
    }

    #[test]
    fn report_publication_cancellation_vetoes_both_native_return_paths() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        #[derive(Debug)]
        struct StopAtReturn {
            cancel: pse_diagnostics::Cancellation,
            target: usize,
            returned: AtomicUsize,
        }
        impl crate::solve::ProgressTap for StopAtReturn {
            fn observe(&self, event: &crate::solve::Event) {
                if event
                    .source
                    .as_ref()
                    .is_some_and(|source| source.family == "native_owner_return.v1")
                    && self.returned.fetch_add(1, Ordering::Relaxed) + 1 == self.target
                {
                    self.cancel.cancel();
                }
            }
        }
        for (magnitude, target) in [(1., 1), (1e6, 2)] {
            let (problem, controls, accuracy, tolerances) = fixture(magnitude, 1., false);
            let cancel = pse_diagnostics::Cancellation::default();
            let tap = Arc::new(StopAtReturn {
                cancel: cancel.clone(),
                target,
                returned: AtomicUsize::new(0),
            });
            let mut execution = Execution::new(cancel, &controls);
            execution.progress = Arc::new(crate::solve::Progress::tapped(16, tap.clone()));
            let report = session(&problem, &controls, &accuracy)
                .solve(
                    &problem,
                    &controls,
                    &accuracy,
                    &Settings::default(),
                    execution,
                    &tolerances,
                )
                .unwrap();
            assert_eq!(tap.returned.load(Ordering::Relaxed), target);
            assert_eq!(report.termination.category, Termination::Cancelled);
            assert_eq!(report.termination.assurance, Assurance::None);
            assert!(report.evidence.work.iterations.unwrap() > 0);
            assert_eq!(
                report.provenance.contains_key("native.refinement.attempts"),
                target == 2
            );
        }
    }

    #[cfg(feature = "sdp")]
    #[test]
    fn bounded_psd_refinement_keeps_independent_original_eigenvalue_check() {
        let (problem, controls, accuracy, tolerances) = fixture(1e6, 1., true);
        let report = session(&problem, &controls, &accuracy)
            .solve(
                &problem,
                &controls,
                &accuracy,
                &Settings::default(),
                Execution::new(Default::default(), &controls),
                &tolerances,
            )
            .unwrap();
        let candidate = report.candidate.as_ref().unwrap();
        let negative_eigenvalue = (1e6 - candidate.primal[0]).max(0.);
        assert_eq!(
            report.quality.as_ref().unwrap().feasible(),
            negative_eigenvalue <= 0.001 && candidate.primal[0] >= -0.001
        );
        assert!(report.provenance.contains_key("native.refinement.attempts"));
        assert!(report.evidence.work.iterations.unwrap() <= u64::from(controls.iterations));
    }
}
