// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded native-control observations with an independent analytic cone distance.
use super::*;

#[test]
fn bounded_observed_cone_refinement_probe() {
    for magnitude in [1.0_f64, 1e6, 1e9] {
        let mut contract = crate::solver_tests::contract();
        contract.variables[0].lower = 0.;
        contract.rows.push(crate::solver_tests::id(3));
        let problem = ConicProblem {
            contract,
            quadratic: SparseMatrix::zeros(1, 1),
            objective: vec![1.],
            constraints: SparseMatrix::new(2, 1, vec![0, 1], vec![0], vec![-1.]),
            rhs: vec![0., magnitude],
            cones: vec![Cone::SecondOrder { dimension: 2 }],
            objective_constant: 0.,
        };
        let zero = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
        let certificate = crate::solver_tests::certify(&zero, 1.);
        let controls = Controls {
            iterations: 100,
            time_limit: std::time::Duration::from_secs(2),
            ..Default::default()
        };
        let accuracy = ResolvedAccuracy {
            feasibility: 0.001,
            gap_absolute: 0.001,
            gap_relative: 0.001,
            ..ResolvedAccuracy::verification()
        };
        let tolerances = Tolerances {
            variables: vec![0.001],
            rows: vec![0.001; 2],
            integrality: 0.001,
        };
        let settings = Settings::default();
        let execution = Execution::new(Default::default(), &controls);
        let mut session = Session::new(
            &problem,
            &certificate,
            &controls,
            &accuracy,
            &settings,
            crate::solver_tests::stamp(Backend::Clarabel),
        )
        .unwrap();
        let first = session
            .solve_once(
                &problem,
                &controls,
                &accuracy,
                &settings,
                execution.clone(),
                &tolerances,
            )
            .unwrap();
        let quality = first.quality.as_ref().unwrap();
        let ratio = quality.normalized_max;
        if ratio <= 1. {
            continue;
        }
        let working = first.evidence.conic.as_ref().unwrap().primal_residual / (8. * ratio);
        assert!(working.is_finite() && working > 0. && working < accuracy.feasibility);
        let remaining = Controls {
            iterations: controls.iterations
                - u32::try_from(first.evidence.work.iterations.unwrap()).unwrap(),
            time_limit: controls
                .time_limit
                .saturating_sub(execution.started.elapsed()),
            ..controls.clone()
        };
        let second = session
            .solve_once(
                &problem,
                &remaining,
                &ResolvedAccuracy {
                    feasibility: working,
                    ..accuracy
                },
                &settings,
                execution,
                &tolerances,
            )
            .unwrap();
        let distance = second
            .candidate
            .as_ref()
            .map(|candidate| (magnitude - candidate.primal[0]).max(0.));
        println!(
            "observed-refinement magnitude={magnitude} first={} ratio={ratio} working={working} second={} iterations={:?}+{:?} original_distance={distance:?}",
            first.termination.name,
            second.termination.name,
            first.evidence.work.iterations,
            second.evidence.work.iterations
        );
        assert!(second.evidence.work.iterations.unwrap() <= u64::from(remaining.iterations));
        if magnitude == 1e6 {
            assert!(second.quality.as_ref().unwrap().feasible());
        }
    }
}

#[test]
fn relative_feasibility_and_original_soc_distance_probe() {
    for magnitude in [1.0_f64, 1e6, 1e9] {
        for working in [0.001, 0.001 / (1.0 + magnitude)] {
            let mut contract = crate::solver_tests::contract();
            contract.variables[0].lower = 0.0;
            contract.rows.push(crate::solver_tests::id(3));
            // min x, [x, magnitude] in SOC: original feasibility is exactly
            // max(magnitude - x, 0), independently of Clarabel's residual norm.
            let problem = ConicProblem {
                contract,
                quadratic: SparseMatrix::zeros(1, 1),
                objective: vec![1.0],
                constraints: SparseMatrix::new(2, 1, vec![0, 1], vec![0], vec![-1.0]),
                rhs: vec![0.0, magnitude],
                cones: vec![Cone::SecondOrder { dimension: 2 }],
                objective_constant: 0.0,
            };
            let quadratic = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
            let certificate = crate::solver_tests::certify(&quadratic, 1.0);
            let controls = Controls {
                iterations: 100,
                time_limit: std::time::Duration::from_secs(2),
                ..Controls::default()
            };
            let accuracy = ResolvedAccuracy {
                feasibility: working,
                gap_absolute: 0.001,
                gap_relative: 0.001,
                ..ResolvedAccuracy::verification()
            };
            let tolerances = Tolerances {
                variables: vec![0.001],
                rows: vec![0.001; 2],
                integrality: 0.001,
            };
            let settings = Settings::default();
            let mut session = Session::new(
                &problem,
                &certificate,
                &controls,
                &accuracy,
                &settings,
                crate::solver_tests::stamp(Backend::Clarabel),
            )
            .unwrap();
            let report = session
                .solve_once(
                    &problem,
                    &controls,
                    &accuracy,
                    &settings,
                    Execution::new(Default::default(), &controls),
                    &tolerances,
                )
                .unwrap();
            assert!(
                !report.evidence.start_submitted,
                "Clarabel accepts no external iterate"
            );
            if let Some(candidate) = report.candidate.as_ref() {
                let distance = (magnitude - candidate.primal[0]).max(0.0);
                assert_eq!(
                    report.quality.as_ref().unwrap().feasible(),
                    distance <= 0.001
                );
                println!(
                    "SOC magnitude={magnitude} working={working} status={} iterations={:?} primal={} original_distance={distance}",
                    report.termination.name, report.evidence.work.iterations, candidate.primal[0]
                );
            } else {
                println!(
                    "SOC magnitude={magnitude} working={working} status={} iterations={:?} no_candidate",
                    report.termination.name, report.evidence.work.iterations
                );
            }
        }
    }
}

#[test]
fn relative_control_probe_across_coefficients_and_original_soc_boundaries() {
    for magnitude in [1.0_f64, 1e6] {
        for coefficient in [1e-6_f64, 1.0, 1e6] {
            for boundary in [0.0_f64, 1e-3] {
                for working in [0.001, 0.001 / (1.0 + magnitude)] {
                    let mut contract = crate::solver_tests::contract();
                    contract.variables[0].lower = 0.;
                    contract.rows = vec![
                        crate::solver_tests::id(3),
                        crate::solver_tests::id(4),
                        crate::solver_tests::id(5),
                    ];
                    let problem = ConicProblem {
                        contract,
                        quadratic: SparseMatrix::zeros(1, 1),
                        objective: vec![coefficient],
                        constraints: SparseMatrix::new(
                            3,
                            1,
                            vec![0, 1],
                            vec![0],
                            vec![-coefficient],
                        ),
                        rhs: vec![0., magnitude, magnitude * boundary],
                        cones: vec![Cone::SecondOrder { dimension: 3 }],
                        objective_constant: 0.,
                    };
                    let zero =
                        faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
                    let certificate = crate::solver_tests::certify(&zero, 1.);
                    let controls = Controls {
                        iterations: 100,
                        time_limit: std::time::Duration::from_secs(2),
                        ..Default::default()
                    };
                    let accuracy = ResolvedAccuracy {
                        feasibility: working,
                        gap_absolute: 0.001,
                        gap_relative: 0.001,
                        ..ResolvedAccuracy::verification()
                    };
                    let tolerance = Tolerances {
                        variables: vec![0.001 / coefficient],
                        rows: vec![0.001; 3],
                        integrality: 0.001,
                    };
                    let settings = Settings::default();
                    let mut session = Session::new(
                        &problem,
                        &certificate,
                        &controls,
                        &accuracy,
                        &settings,
                        crate::solver_tests::stamp(Backend::Clarabel),
                    )
                    .unwrap();
                    let report = session
                        .solve_once(
                            &problem,
                            &controls,
                            &accuracy,
                            &settings,
                            Execution::new(Default::default(), &controls),
                            &tolerance,
                        )
                        .unwrap();
                    if let Some(candidate) = &report.candidate {
                        let physical = candidate.primal[0] * coefficient;
                        let distance = (magnitude.hypot(magnitude * boundary) - physical).max(0.);
                        let quality = report.quality.as_ref().unwrap();
                        assert!(
                            (quality.rows.iter().map(|r| r.physical).fold(0., f64::max) - distance)
                                .abs()
                                <= 16. * f64::EPSILON * magnitude.max(physical.abs()).max(1.)
                        );
                        assert_eq!(quality.feasible(), distance <= 0.001 && physical >= -0.001);
                        println!(
                            "SOC coefficient={coefficient} magnitude={magnitude} boundary={boundary} working={working} status={} iterations={:?} original_distance={distance}",
                            report.termination.name, report.evidence.work.iterations
                        );
                    }
                }
            }
        }
    }
}

#[cfg(feature = "sdp")]
#[test]
fn relative_control_probe_retains_original_psd_eigenvalue_and_native_status() {
    for magnitude in [1.0_f64, 1e6, 1e9] {
        for working in [0.001, 0.001 / (1. + magnitude)] {
            let mut contract = crate::solver_tests::contract();
            contract.variables[0].lower = 0.;
            contract.rows = vec![
                crate::solver_tests::id(3),
                crate::solver_tests::id(4),
                crate::solver_tests::id(5),
            ];
            // [[x, magnitude], [magnitude, x]] has eigenvalues x +/- magnitude.
            let problem = ConicProblem {
                contract,
                quadratic: SparseMatrix::zeros(1, 1),
                objective: vec![1.],
                constraints: SparseMatrix::new(3, 1, vec![0, 2], vec![0, 2], vec![-1., -1.]),
                rhs: vec![0., std::f64::consts::SQRT_2 * magnitude, 0.],
                cones: vec![Cone::PsdTriangle { order: 2 }],
                objective_constant: 0.,
            };
            let zero = faer::sparse::SparseColMat::try_new_from_triplets(1, 1, &[]).unwrap();
            let certificate = crate::solver_tests::certify(&zero, 1.);
            let controls = Controls {
                iterations: 100,
                time_limit: std::time::Duration::from_secs(2),
                ..Default::default()
            };
            let accuracy = ResolvedAccuracy {
                feasibility: working,
                gap_absolute: 0.001,
                gap_relative: 0.001,
                ..ResolvedAccuracy::verification()
            };
            let tolerance = Tolerances {
                variables: vec![0.001],
                rows: vec![0.001; 3],
                integrality: 0.001,
            };
            let settings = Settings::default();
            let mut session = Session::new(
                &problem,
                &certificate,
                &controls,
                &accuracy,
                &settings,
                crate::solver_tests::stamp(Backend::Clarabel),
            )
            .unwrap();
            let report = session
                .solve_once(
                    &problem,
                    &controls,
                    &accuracy,
                    &settings,
                    Execution::new(Default::default(), &controls),
                    &tolerance,
                )
                .unwrap();
            if let Some(candidate) = &report.candidate {
                let distance = (magnitude - candidate.primal[0]).max(0.);
                let quality = report.quality.as_ref().unwrap();
                assert!(
                    (quality.rows.iter().map(|r| r.physical).fold(0., f64::max) - distance).abs()
                        <= 16. * f64::EPSILON * magnitude.max(candidate.primal[0].abs()).max(1.)
                );
                println!(
                    "PSD magnitude={magnitude} working={working} status={} iterations={:?} original_distance={distance}",
                    report.termination.name, report.evidence.work.iterations
                );
            }
        }
    }
}
