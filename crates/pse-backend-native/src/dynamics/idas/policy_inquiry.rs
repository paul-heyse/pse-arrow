// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded controls for current ODE adjoint rates and actual native initial steps.
use super::*;

#[derive(Debug)]
struct Linear {
    contract: Contract,
    stiffness: f64,
    fail_derivatives: Arc<std::sync::atomic::AtomicBool>,
    nonlinear: bool,
}
impl Linear {
    fn new(stiffness: f64) -> Self {
        let id = |byte| SemanticId::from_bytes([byte; 16]);
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([174; 32]),
                states: vec![id(1), id(2)],
                differential: vec![true; 2],
                parameters: vec![id(3), id(4)],
                outputs: vec![id(5)],
                events: vec![vec![]],
                signs: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
                quadratures: vec![],
                balances: vec![],
            },
            stiffness,
            fail_derivatives: Arc::default(),
            nonlinear: false,
        }
    }
}
impl Oracle for Linear {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        let pairs: &[(usize, usize)] = match function {
            Function::Rhs => &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1), (1, 2)],
            Function::Output => &[(0, 0), (0, 1)],
            _ => &[],
        };
        pairs
            .iter()
            .map(|&(r, c)| SupportEntry::new(OriginalRow::new(r), OriginalCol::new(c)))
            .collect()
    }
    fn evaluate(
        &mut self,
        _: usize,
        function: Function,
        time: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        if derivatives
            && self
                .fail_derivatives
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(ProblemError::numerical("current ODE derivative refused"));
        }
        let (values, entries) = match function {
            Function::Initial => (vec![2., 1.], vec![]),
            Function::Rhs if self.nonlinear => (
                vec![
                    -p[0] * x[0] + (p[1] + time) * x[1] * x[1],
                    0.2 * x[0] - self.stiffness * x[1],
                ],
                vec![
                    (0, 0, -p[0]),
                    (0, 1, 2. * (p[1] + time) * x[1]),
                    (0, 2, -x[0]),
                    (0, 3, x[1] * x[1]),
                    (1, 0, 0.2),
                    (1, 1, -self.stiffness),
                ],
            ),
            Function::Rhs => (
                vec![-p[0] * x[0] + p[1], -self.stiffness * x[1] + p[0]],
                vec![
                    (0, 0, -p[0]),
                    (0, 2, -x[0]),
                    (0, 3, 1.),
                    (1, 1, -self.stiffness),
                    (1, 2, 1.),
                ],
            ),
            Function::Output => (vec![x[0] + 0.3 * x[1]], vec![(0, 0, 1.), (0, 1, 0.3)]),
            _ => (vec![], vec![]),
        };
        let jacobian = derivatives.then(|| {
            Arc::new(
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    4,
                    &entries
                        .into_iter()
                        .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            )
            .into()
        });
        Ok(Evaluation { values, jacobian })
    }
}
fn profile(scheduled: bool, sensitivity: DynamicSensitivity) -> Profile {
    Profile {
        method: Method::Idas,
        end: 0.6,
        samples: vec![0., 0.1, 0.3, 0.45, 0.6],
        // Keep native-IC versus exact-rate comparisons at the same explicitly
        // configured first step. The separate h0 control selects both policies.
        idas: IdasSettings {
            native_initial_step: IdasInitialStep::Configured,
            ..Default::default()
        },
        rtol: 0.001,
        atol: vec![0.001; 2],
        initial_step: 1e-4,
        time_limit: Duration::from_secs(5),
        sensitivity,
        parameter_scales: vec![1.; 2],
        schedule: if scheduled {
            vec![ScheduledInput {
                parameter: 1,
                times: vec![0.3],
            }]
        } else {
            vec![]
        },
        ..Profile::default()
    }
}
fn analytic(p: &[f64], time: f64, stiffness: f64) -> f64 {
    let k = p[0];
    let first_time = time.min(0.3);
    let x = 2. * (-k * first_time).exp() + p[1] / k * (1. - (-k * first_time).exp());
    let x = if time > 0.3 {
        let u = if p.len() == 3 { p[2] } else { p[1] };
        x * (-k * (time - 0.3)).exp() + u / k * (1. - (-k * (time - 0.3)).exp())
    } else {
        x
    };
    let z = (-stiffness * time).exp() + k / stiffness * (1. - (-stiffness * time).exp());
    x + 0.3 * z
}
fn functional(p: &[f64], profile: &Profile, stiffness: f64) -> f64 {
    profile
        .samples
        .iter()
        .enumerate()
        .map(|(i, &t)| (i + 1) as f64 * analytic(p, t, stiffness))
        .sum()
}
fn derivative(p: &[f64], axis: usize, profile: &Profile, stiffness: f64) -> f64 {
    let mut plus = p.to_vec();
    let mut minus = p.to_vec();
    let h = 1e-5;
    plus[axis] += h;
    minus[axis] -= h;
    (functional(&plus, profile, stiffness) - functional(&minus, profile, stiffness)) / (2. * h)
}
#[test]
fn pure_ode_adjoint_rates_match_native_ic_with_original_accuracy_classification() {
    for stiffness in [1., 100.] {
        for scheduled in [false, true] {
            let p = profile(scheduled, DynamicSensitivity::Adjoint);
            let parameters = if scheduled {
                vec![0.8, 0.2, 1.1]
            } else {
                vec![0.8, 0.2]
            };
            let mut gradients = Vec::new();
            let mut original_classes = Vec::new();
            for full_ic in [true, false] {
                let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
                let operation = hub.start("ODE adjoint initialization comparison");
                let context = operation.context();
                let _entered = context.enter();
                let mut oracle = Linear::new(stiffness);
                let layout = oracle.prepared_layout(128 << 20).unwrap();
                let mut observer = IgnoreSamples;
                let mut output = SampleOutput {
                    retention: SampleRetention::Full,
                    observer: &mut observer,
                };
                let started = Instant::now();
                let (report, result, _) = attempt(
                    NumericalRequest {
                        oracle: &mut oracle,
                        profile: &p,
                        integration: &parameters,
                        directions: &[],
                        forward_quadratures: false,
                        admission: Some((layout, 128 << 20)),
                        current_execution: None,
                        workspace: None,
                    },
                    Cancellation::default(),
                    Arc::new(Progress::new(256)),
                    &mut output,
                    |session, report| {
                        session.force_backward_ic = full_ic;
                        let weights = (1..=report.samples.len())
                            .map(|i| i as f64)
                            .collect::<Vec<_>>();
                        session
                            .backward(&p, report, &weights)
                            .map(|value| (value.0, session.backward_statistics()))
                    },
                );
                let elapsed = started.elapsed();
                assert_eq!(
                    report.termination,
                    Termination::Completed,
                    "{:?}",
                    report.error
                );
                let classes=report.samples.iter().map(|sample| {
                    let expected=analytic(&parameters,sample.time,stiffness);
                    let error=(sample.outputs[0]-expected).abs();
                    let budget=0.001*(1.+expected.abs());
                    assert!(error.is_finite());
                    eprintln!("ode_original_output stiff={stiffness} scheduled={scheduled} full_ic={full_ic} t={} error={error} budget={budget} usable={}",sample.time,error<=budget);
                    error<=budget
                }).collect::<Vec<_>>();
                if stiffness == 1. {
                    assert!(classes.iter().all(|usable| *usable));
                }
                original_classes.push(classes);
                let (gradient, stats) = result.unwrap();
                let snapshot = context.reader().snapshot(Duration::from_secs(60));
                let count = |name: &str| {
                    snapshot
                        .metrics
                        .iter()
                        .find(|metric| metric.name == name)
                        .map_or(0, |metric| match metric.cumulative {
                            pse_diagnostics::observation::Value::Count(n) => n,
                            _ => 0,
                        })
                };
                assert!(
                    count(if full_ic {
                        "idas.adjoint.consistent_initializations"
                    } else {
                        "idas.adjoint.exact_ode_rate_initializations"
                    }) > 0
                );
                assert_eq!(
                    count(if full_ic {
                        "idas.adjoint.exact_ode_rate_initializations"
                    } else {
                        "idas.adjoint.consistent_initializations"
                    }),
                    0
                );
                for (axis, &actual) in gradient.iter().enumerate() {
                    let expected = derivative(&parameters, axis, &p, stiffness);
                    assert!(
                        (actual - expected).abs() * 0.1
                            <= 0.001 * (1. + functional(&parameters, &p, stiffness).abs()),
                        "stiff={stiffness} schedule={scheduled} fullIC={full_ic} axis={axis}: {actual} vs {expected}"
                    );
                }
                eprintln!(
                    "ode_adjoint_ic stiff={stiffness} scheduled={scheduled} full_ic={full_ic} elapsed_us={} gradient={gradient:?} backward={stats}",
                    elapsed.as_micros()
                );
                gradients.push(gradient);
            }
            assert_eq!(
                original_classes[0], original_classes[1],
                "native convergence does not promote an original-invalid stiff trajectory"
            );
            for (actual, reference) in gradients[1].iter().zip(&gradients[0]) {
                assert!(
                    (actual - reference).abs() * 0.1
                        <= 0.001 * (1. + functional(&parameters, &p, stiffness).abs())
                );
            }
        }
    }
}
#[test]
fn native_initial_step_choice_preserves_ic_and_reports_distinct_original_error_classifications() {
    for stiffness in [1., 100.] {
        let mut classifications = Vec::new();
        for automatic in [false, true] {
            let mut oracle = Linear::new(stiffness);
            let mut p = profile(false, DynamicSensitivity::Forward);
            p.idas.native_initial_step = if automatic {
                IdasInitialStep::Automatic
            } else {
                IdasInitialStep::Configured
            };
            let parameters = [0.8, 0.2];
            let mut execution = Execution::new(Cancellation::default(), &Default::default());
            execution.memory = Some(128 << 20);
            let mut session =
                Session::new(&mut oracle, &parameters, &p, &[], None, execution, false).unwrap();
            let mut report = Report::new(&p, SampleRetention::Full, parameters.len());
            let mut observer = IgnoreSamples;
            let mut output = SampleOutput {
                retention: SampleRetention::Full,
                observer: &mut observer,
            };
            let started = Instant::now();
            session.run(&p, &mut report, &mut output).unwrap();
            assert_eq!(
                report.termination,
                Termination::Completed,
                "{:?}",
                report.error
            );
            let classes=report.samples.iter().map(|sample| {
                let expected=analytic(&parameters,sample.time,stiffness);
                let error=(sample.outputs[0]-expected).abs();
                let budget=0.001*(1.+expected.abs());
                assert!(error.is_finite());
                eprintln!("ode_h0_original_output stiff={stiffness} automatic={automatic} t={} error={error} budget={budget} usable={}",sample.time,error<=budget);
                error<=budget
            }).collect::<Vec<_>>();
            if stiffness == 1. {
                assert!(classes.iter().all(|usable| *usable));
            }
            classifications.push(classes);
            let stats = session.integrator_statistics().unwrap();
            assert!(stats.initial_step > 0. && stats.initial_step.is_finite());
            eprintln!(
                "ode_initial_step stiff={stiffness} automatic={automatic} elapsed_us={} statistics={stats:?}",
                started.elapsed().as_micros()
            );
        }
        if stiffness == 1. {
            assert_eq!(classifications[0], classifications[1]);
        } else {
            assert_eq!(
                classifications[0],
                vec![true, true, false, false, false],
                "retain the independently observed configured-step original-invalid outcome"
            );
            assert!(
                classifications[1].iter().all(|usable| *usable),
                "explicit native h0 improves this fixture without changing its error budget"
            );
        }
    }
}

#[test]
fn ode_adjoint_rate_eligibility_keeps_dae_second_order_failure_and_stop_distinct() {
    let mut oracle = Linear::new(2.);
    let failure = oracle.fail_derivatives.clone();
    let p = profile(false, DynamicSensitivity::Adjoint);
    let parameters = [0.8, 0.2];
    let cancellation = Cancellation::default();
    let mut execution = Execution::new(cancellation.clone(), &Default::default());
    execution.memory = Some(128 << 20);
    let mut session =
        Session::new(&mut oracle, &parameters, &p, &[], None, execution, false).unwrap();
    let state = [2., 1.];
    let lambda = vec![vec![3., 4.]];
    let rates = session
        .ode_backward_rates(0., &state, &lambda)
        .unwrap()
        .unwrap();
    assert!((rates[0][0] - 2.4).abs() < 1e-14);
    assert_eq!(rates[0][1], 8.);
    session.callback.contract.differential[1] = false;
    assert!(
        session
            .ode_backward_rates(0., &state, &lambda)
            .unwrap()
            .is_none()
    );
    session.callback.contract.differential[1] = true;
    session.backward_mut().unwrap().directions.push(0);
    assert!(
        session
            .ode_backward_rates(0., &state, &lambda)
            .unwrap()
            .is_none()
    );
    session.backward_mut().unwrap().directions.clear();
    assert!(session.ode_backward_rates(0., &state, &[vec![3.]]).is_err());
    failure.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(session.ode_backward_rates(0., &state, &lambda).is_err());
    failure.store(false, std::sync::atomic::Ordering::Relaxed);
    cancellation.cancel();
    assert!(session.ode_backward_rates(0., &state, &lambda).is_err());
}

#[test]
fn ode_adjoint_rates_use_current_nonsymmetric_state_time_and_parameter_jacobian() {
    let mut oracle = Linear::new(2.);
    oracle.nonlinear = true;
    let p = profile(false, DynamicSensitivity::Adjoint);
    let mut execution = Execution::new(Cancellation::default(), &Default::default());
    execution.memory = Some(128 << 20);
    let mut session =
        Session::new(&mut oracle, &[0.8, 0.2], &p, &[], None, execution, false).unwrap();
    for (time, state, parameters) in [(0., [2., 1.], [0.8, 0.2]), (0.4, [3., 2.], [1.2, 0.7])] {
        session.callback.parameters = parameters.to_vec();
        let lambda = vec![vec![3., 4.]];
        let actual = session
            .ode_backward_rates(time, &state, &lambda)
            .unwrap()
            .unwrap();
        let expected = [
            3. * parameters[0] - 0.8,
            8. - 6. * (parameters[1] + time) * state[1],
        ];
        for (a, b) in actual[0].iter().zip(expected) {
            assert!((a - b).abs() < 1e-13, "{a} vs {b}");
        }
        // This independently expanded residual uses both nonsymmetric cross terms.
        assert!((actual[0][0] - 3. * parameters[0] + 0.2 * 4.).abs() < 1e-13);
        assert!(
            (actual[0][1] + 2. * (parameters[1] + time) * state[1] * 3. - 2. * 4.).abs() < 1e-13
        );
    }
}
