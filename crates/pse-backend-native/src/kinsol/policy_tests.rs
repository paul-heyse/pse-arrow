// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native control comparisons against independently evaluated original equations.
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct TranslatedCubic {
    contract: OracleContract,
    pattern: faer::sparse::SparseColMat<usize, f64>,
    origin: f64,
    multiplier: f64,
    points: Arc<Mutex<Vec<f64>>>,
}
impl NleOracle for TranslatedCubic {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.points.lock().unwrap().push(x[0]);
        out[0] = self.multiplier * ((x[0] - self.origin).powi(3) - 1.0);
        Ok(())
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = 3.0 * self.multiplier * (x[0] - self.origin).powi(2);
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        v: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.jacobian(x, out)?;
        out[0] *= v[0];
        Ok(())
    }
}

fn run(
    origin: f64,
    multiplier: f64,
    shifted: bool,
    legacy: bool,
    warm: bool,
    accepted: bool,
    maximum_step: Option<f64>,
) -> SolveReport {
    let points = Arc::new(Mutex::new(Vec::new()));
    let mut contract = crate::solver_tests::contract();
    if shifted {
        contract.variables[0].lower = origin;
    }
    let oracle = TranslatedCubic {
        contract,
        pattern: faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 1.0)],
        )
        .unwrap(),
        origin,
        multiplier,
        points: points.clone(),
    };
    let tolerances = Tolerances {
        variables: vec![0.001],
        rows: vec![0.001],
        integrality: 0.001,
    };
    let accuracy = ResolvedAccuracy {
        feasibility: 0.001,
        ..ResolvedAccuracy::verification()
    };
    let mut settings = Settings::from_policy(
        Method {
            strategy: Strategy::Newton,
            setup_interval: 1,
            max_newton_step: maximum_step,
            ..Method::default()
        },
        &tolerances,
        &pse_math::normalization::Normalization::identity(1, 1),
        accuracy.feasibility,
    );
    if legacy {
        settings.step_tolerance = accuracy.feasibility;
    }
    let controls = Controls {
        iterations: 50,
        time_limit: std::time::Duration::from_secs(2),
        ..Controls::default()
    };
    let execution = Execution::new(pse_diagnostics::Cancellation::default(), &controls);
    let compatibility = crate::solver_tests::stamp(Backend::Kinsol);
    let mut session = Session::new(
        Function::Equations(Box::new(oracle)),
        settings,
        execution.clone(),
        compatibility.clone(),
    )
    .unwrap();
    let expected = origin
        + if accepted {
            1.0
        } else if maximum_step.is_some() {
            100.0
        } else if warm {
            1.5
        } else {
            2.0
        };
    let seed = warm.then(|| WarmStart {
        origin: None,
        compatibility,
        payload: WarmPayload::Root(vec![expected]),
    });
    let report = session
        .solve(
            &[if warm { f64::NAN } else { expected }],
            &controls,
            &accuracy,
            execution,
            &tolerances,
            seed.as_ref(),
        )
        .unwrap();
    assert_eq!(points.lock().unwrap().first().copied(), Some(expected));
    assert!(report.evidence.start_submitted);
    assert_eq!(report.metrics["start.submitted"], Metric::Bool(true));
    assert_eq!(
        report.metrics["KINSetScaledStepTol.requested"],
        Metric::Real(if legacy { 0.001 } else { 0.0 })
    );
    assert_eq!(
        report.provenance["scaled_step_policy"],
        if legacy {
            "explicit_native_relative_stagnation"
        } else {
            "native_roundoff_default"
        }
    );
    assert!(!report.evidence.reused_native_state);
    if let Some(candidate) = &report.candidate {
        let original = multiplier * ((candidate.primal[0] - origin).powi(3) - 1.0);
        assert_eq!(
            report.quality.as_ref().unwrap().feasible(),
            original.abs() <= 0.001
        );
    }
    report
}

#[test]
fn native_roundoff_stagnation_avoids_false_large_coordinate_limit() {
    let previous = run(1e8, 1.0, false, true, false, false, None);
    assert_eq!(previous.termination.name, "KIN_STEP_LT_STPTOL");
    assert!(!previous.quality.as_ref().unwrap().feasible());
    let corrected = run(1e8, 1.0, false, false, false, false, None);
    assert_eq!(corrected.termination.category, Termination::Success);
    assert!(corrected.quality.as_ref().unwrap().feasible());
    assert!(
        corrected.evidence.work.iterations.unwrap() > previous.evidence.work.iterations.unwrap()
    );
}

#[test]
fn native_default_preserves_shifted_scaled_and_accepted_cold_or_warm_inputs() {
    for origin in [0.0, 1e4] {
        for multiplier in [1.0, 1e6] {
            for shifted in [false, true] {
                for warm in [false, true] {
                    for accepted in [false, true] {
                        let report = run(origin, multiplier, shifted, false, warm, accepted, None);
                        assert_eq!(
                            report.termination.category,
                            Termination::Success,
                            "{origin}/{multiplier}/{shifted}/{warm}/{accepted}: {:?}",
                            report.termination
                        );
                        assert!(report.quality.as_ref().unwrap().feasible());
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_maximum_step_remains_a_distinct_native_effort_limit() {
    let report = run(0.0, 1.0, false, false, false, false, Some(1.0));
    assert_eq!(report.termination.name, "KIN_MXNEWT_5X_EXCEEDED");
    assert_ne!(report.termination.category, Termination::Success);
    assert_eq!(report.termination.assurance, Assurance::None);
    assert!(!report.quality.as_ref().unwrap().feasible());
}

#[derive(Debug)]
struct AffineTear {
    contract: OracleContract,
    pattern: faer::sparse::SymbolicSparseColMat<usize>,
    slope: f64,
    domain: f64,
    calls: Arc<std::sync::atomic::AtomicUsize>,
}
impl FixedPointOracle for AffineTear {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn physical_tear_coordinates(&self) -> bool {
        true
    }
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.as_ref()
    }
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        for (i, (x, out)) in x.iter().zip(out).enumerate() {
            if *x < self.domain {
                return Err(ProblemError::Provider(pse_kernels::ProviderError::Trial(
                    "declared tear-map domain".into(),
                )));
            }
            let slope = self.slope - 0.01 * f64::from(u32::try_from(i).unwrap());
            *out = slope * x + (1. - slope);
        }
        Ok(())
    }
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for (i, (x, out)) in x.iter().zip(out).enumerate() {
            if *x < self.domain {
                return Err(ProblemError::Provider(pse_kernels::ProviderError::Trial(
                    "declared original tear domain".into(),
                )));
            }
            let slope = self.slope - 0.01 * f64::from(u32::try_from(i).unwrap());
            *out = (1. - slope) * (1. - x);
        }
        Ok(())
    }
}

#[test]
fn native_recycle_history_damping_and_delay_probe_preserves_original_map_meaning() {
    for (slope, domain) in [
        (0.5, f64::NEG_INFINITY),
        (-0.9, f64::NEG_INFINITY),
        (0.99, f64::NEG_INFINITY),
        (1.1, -2.0),
    ] {
        for (history, damping, delay) in
            [(0, 1., 0), (1, 1., 0), (3, 1., 1), (5, 1., 1), (1, 0.5, 1)]
        {
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let (pattern, _) = faer::sparse::SymbolicSparseColMat::try_new_from_indices(
                6,
                6,
                &(0..6)
                    .map(|i| faer::sparse::Pair::new(i, i))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            let mut contract = crate::solver_tests::contract();
            contract.variables = (0..6_u8)
                .map(|i| crate::Variable {
                    id: crate::solver_tests::id(10 + i),
                    lower: f64::NEG_INFINITY,
                    upper: f64::INFINITY,
                })
                .collect();
            contract.rows = (0..6_u8).map(|i| crate::solver_tests::id(30 + i)).collect();
            let oracle = AffineTear {
                contract,
                pattern,
                slope,
                domain,
                calls: calls.clone(),
            };
            let method = Method {
                strategy: Strategy::FixedPoint,
                anderson: history,
                damping: pse_model::scalars::Fraction::try_new(damping).unwrap(),
                anderson_delay: delay,
                ..Method::default()
            };
            let tolerances = Tolerances {
                variables: vec![0.001; 6],
                rows: vec![0.001; 6],
                integrality: 0.001,
            };
            let accuracy = ResolvedAccuracy {
                feasibility: 0.001,
                ..ResolvedAccuracy::verification()
            };
            let settings = Settings::from_policy(
                method,
                &tolerances,
                &pse_math::normalization::Normalization::identity(6, 6),
                accuracy.feasibility,
            );
            let controls = Controls {
                iterations: 40,
                time_limit: std::time::Duration::from_secs(2),
                ..Default::default()
            };
            let mut execution = Execution::new(Default::default(), &controls);
            let history_bytes = anderson_storage(6, method).unwrap().0;
            execution.memory = Some((1 << 20) + history_bytes);
            let mut session = Session::new(
                Function::FixedPoint(Box::new(oracle)),
                settings,
                execution.clone(),
                crate::solver_tests::stamp(Backend::Kinsol),
            )
            .unwrap();
            let report = session
                .solve(&[0.; 6], &controls, &accuracy, execution, &tolerances, None)
                .unwrap();
            if let (Some(candidate), Some(quality)) = (&report.candidate, &report.quality) {
                let physical = candidate
                    .primal
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        ((1. - slope + 0.01 * f64::from(u32::try_from(i).unwrap())) * (1. - x))
                            .abs()
                    })
                    .fold(0., f64::max);
                assert_eq!(quality.feasible(), physical <= 0.001);
            }
            println!(
                "recycle slope={slope} domain={domain} history={history} damping={damping} delay={delay} history_bytes={history_bytes} map_calls={} iterations={:?} native={} physical={:?}",
                calls.load(std::sync::atomic::Ordering::SeqCst),
                report.evidence.work.iterations,
                report.termination.name,
                report.quality.as_ref().map(Quality::feasible)
            );
        }
    }
}
