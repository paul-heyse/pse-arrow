// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected physical coordinates with complete nonzero values, coupled states,
//! parameter-dependent initialization and one scheduled interval boundary.
use super::*;

#[derive(Debug)]
struct Coupled(Contract);
impl Coupled {
    fn new() -> Self {
        let id = |v| SemanticId::from_bytes([v; 16]);
        Self(Contract {
            identity: ContentHash::from_bytes([101; 32]),
            states: vec![id(1), id(2)],
            differential: vec![true, true],
            parameters: vec![id(3), id(4), id(5), id(6)],
            outputs: vec![id(7)],
            events: vec![vec![]],
            signs: vec![],
            quadratures: vec![],
            balances: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
        })
    }
}
impl Oracle for Coupled {
    fn snapshot_owner(
        &self,
        bytes: usize,
    ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
        #[derive(Debug)]
        struct FixtureSnapshot {
            _bytes: usize,
        }
        Ok(Arc::new(FixtureSnapshot { _bytes: bytes }))
    }
    fn contract(&self) -> &Contract {
        &self.0
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        entries(match function {
            Function::Initial => vec![(0, 3), (0, 5)],
            Function::Rhs => vec![(0, 2), (0, 3), (1, 0), (1, 4)],
            Function::Output => vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        function: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivative: bool,
    ) -> Result<Evaluation, ProblemError> {
        // Contract order: scheduled rate, fixed nonzero forcing, coupling, initial factor.
        let (values, partials) = match function {
            Function::Initial => (vec![p[3] * p[1], 1.], vec![(0, 3, p[3]), (0, 5, p[1])]),
            Function::Rhs => (
                vec![p[0] + p[1], p[2] * x[0]],
                vec![(0, 2, 1.), (0, 3, 1.), (1, 0, p[2]), (1, 4, x[0])],
            ),
            Function::Output => (
                vec![x[0] + x[1] + 2. * p[0] + 3. * p[1]],
                vec![(0, 0, 1.), (0, 1, 1.), (0, 2, 2.), (0, 3, 3.)],
            ),
            _ => (vec![], vec![]),
        };
        let jacobian = derivative
            .then(|| {
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    6,
                    &partials
                        .into_iter()
                        .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                        .collect::<Vec<_>>(),
                )
                .map_err(|error| ProblemError::internal(format!("fixture Jacobian: {error:?}")))
            })
            .transpose()?;
        Ok(Evaluation {
            values,
            jacobian: jacobian.map(|j| Arc::new(j).into()),
        })
    }
}
fn methods() -> Vec<Method> {
    vec![
        #[cfg(feature = "diffsol")]
        Method::Diffsol,
        #[cfg(feature = "idas")]
        Method::Idas,
    ]
}
fn profile(method: Method, axes: Option<Vec<usize>>) -> Profile {
    Profile {
        method,
        samples: vec![0., 0.25, 0.5, 0.75, 1.],
        atol: vec![1e-10; 2],
        rtol: 1e-9,
        initial_step: 1e-5,
        sensitivity: DynamicSensitivity::Forward,
        forward_axes: axes,
        parameter_scales: vec![0.25, 8., 3., 0.6],
        schedule: vec![ScheduledInput {
            parameter: 0,
            times: vec![0.5],
        }],
        ..Default::default()
    }
}
// Physical integration order is forcing, coupling, initial factor, rate[0], rate[1].
const VALUES: [f64; 5] = [5., 2., 0.4, 1., 3.];
#[allow(
    clippy::unwrap_used,
    reason = "test helper fails on any fixture integration error"
)]
fn run(profile: &Profile, values: &[f64]) -> Report {
    let report = integrate(
        &mut Coupled::new(),
        profile,
        values,
        Cancellation::default(),
    )
    .unwrap();
    assert_eq!(
        report.termination,
        Termination::Completed,
        "{:?}: {:?}",
        profile.method,
        report.error
    );
    report
}
#[allow(
    clippy::unreachable,
    reason = "test analytic oracle accepts only the five declared physical axes"
)]
fn analytic(t: f64, axis: usize) -> (f64, f64, f64) {
    let later = (t - 0.5).max(0.);
    let initial = VALUES[2] * VALUES[0];
    let integral = initial * t
        + (VALUES[3] + VALUES[0]) * t * t / 2.
        + (VALUES[4] - VALUES[3]) * later * later / 2.;
    let (dx, dy, direct) = match axis {
        0 => (VALUES[2] + t, VALUES[1] * (VALUES[2] * t + t * t / 2.), 3.),
        1 => (0., integral, 0.),
        2 => (VALUES[0], VALUES[1] * VALUES[0] * t, 0.),
        3 => (
            t.min(0.5),
            VALUES[1] * (t * t - later * later) / 2.,
            if t < 0.5 { 2. } else { 0. },
        ),
        4 => (
            later,
            VALUES[1] * later * later / 2.,
            if t >= 0.5 { 2. } else { 0. },
        ),
        _ => unreachable!(),
    };
    (dx, dy, dx + dy + direct)
}
#[test]
fn selected_physical_axes_preserve_initial_coupled_scheduled_and_scaled_responses() {
    for method in methods() {
        let full = run(&profile(method, None), &VALUES);
        assert_eq!(full.forward_axes, vec![0, 1, 2, 3, 4]);
        for selected in [vec![4, 2, 3], vec![0, 1], vec![3], vec![4]] {
            let report = run(&profile(method, Some(selected.clone())), &VALUES);
            assert_eq!(report.forward_axes, selected);
            for (sample, reference) in report.samples.iter().zip(&full.samples) {
                let width = selected.len();
                assert_eq!(sample.state_sensitivities.len(), 2 * width);
                assert_eq!(sample.output_sensitivities.len(), width);
                assert!((sample.outputs[0] - reference.outputs[0]).abs() < 2e-5);
                for (column, axis) in selected.iter().enumerate() {
                    let expected = analytic(sample.time, *axis);
                    for (actual, exact) in [
                        (sample.state_sensitivities[column], expected.0),
                        (sample.state_sensitivities[width + column], expected.1),
                        (sample.output_sensitivities[column], expected.2),
                    ] {
                        assert!(
                            (actual - exact).abs() < 2e-5,
                            "{method:?} axis {axis} at {}: {actual} vs {exact}",
                            sample.time
                        );
                    }
                    assert!(
                        (sample.output_sensitivities[column]
                            - reference.output_sensitivities[*axis])
                            .abs()
                            < 2e-5
                    );
                }
            }
            // Independent value-only central differences at the initial point and endpoint.
            let mut primal = profile(method, Some(vec![]));
            primal.sensitivity = DynamicSensitivity::None;
            primal.forward_axes = None;
            for (column, axis) in selected.iter().enumerate() {
                let mut plus = VALUES;
                let mut minus = VALUES;
                plus[*axis] += 1e-3;
                minus[*axis] -= 1e-3;
                let plus = run(&primal, &plus);
                let minus = run(&primal, &minus);
                for k in [0, report.samples.len() - 1] {
                    let difference =
                        (plus.samples[k].outputs[0] - minus.samples[k].outputs[0]) / 2e-3;
                    assert!(
                        (difference - report.samples[k].output_sensitivities[column]).abs() < 5e-5,
                        "{method:?} axis {axis}: finite difference {difference}"
                    );
                }
            }
        }
    }
}
#[test]
fn selected_empty_axes_integrate_values_without_sensitivities_and_missing_is_unavailable() {
    for method in methods() {
        let full = run(&profile(method, None), &VALUES);
        let empty = run(&profile(method, Some(vec![])), &VALUES);
        assert!(empty.forward_axes.is_empty());
        assert!(empty.sensitivity_column(0).is_err());
        for (value, reference) in empty.samples.iter().zip(&full.samples) {
            assert!(value.state_sensitivities.is_empty() && value.output_sensitivities.is_empty());
            assert!((value.outputs[0] - reference.outputs[0]).abs() < 2e-5);
        }
        let partial = run(&profile(method, Some(vec![4, 2])), &VALUES);
        assert_eq!(partial.sensitivity_column(4).unwrap(), 0);
        assert_eq!(partial.sensitivity_column(2).unwrap(), 1);
        assert!(partial.sensitivity_column(0).is_err());
    }
}
#[test]
fn selected_anchor_and_later_interval_keep_fixed_anchor_values() {
    for method in methods() {
        let mut oracle = Anchored::new(Coupled::new(), false).unwrap();
        let mut profile = oracle.profile(&profile(method, None));
        // Unscheduled forcing, coupling, initial factor, x anchor, y anchor,
        // then the scheduled rate's two intervals.
        let values = [5., 2., 0.4, 2., 1., 1., 3.];
        profile.forward_axes = Some(vec![6, 3]);
        let report = integrate(&mut oracle, &profile, &values, Cancellation::default()).unwrap();
        assert_eq!(
            report.termination,
            Termination::Completed,
            "{method:?}: {:?}",
            report.error
        );
        assert_eq!(report.forward_axes, vec![6, 3]);
        for sample in &report.samples {
            let later = analytic(sample.time, 4);
            assert!((sample.output_sensitivities[0] - later.2).abs() < 2e-5);
            assert!((sample.state_sensitivities[1] - 1.).abs() < 2e-5);
            assert!((sample.state_sensitivities[3] - 2. * sample.time).abs() < 2e-5);
            assert!((sample.output_sensitivities[1] - (1. + 2. * sample.time)).abs() < 2e-5);
            assert!(
                report.sensitivity_column(4).is_err(),
                "fixed y anchor is a value, not a selected axis"
            );
        }
    }
}
#[test]
fn selected_axes_admission_and_identity_use_the_resolved_width() {
    let contract = Coupled::new().0;
    for method in methods() {
        let full = profile(method, None);
        let selected = profile(method, Some(vec![4, 2]));
        assert_ne!(profile_json(&full), profile_json(&selected));
        assert_eq!(
            profile_json(&selected)["forward_axes"],
            serde_json::json!([4, 2])
        );
        assert!(
            serde_json::to_value(&selected)
                .unwrap()
                .get("forward_axes")
                .is_none()
        );
        assert_eq!(selected.sample_cells(&contract, 5).unwrap(), 45);
        assert_eq!(full.sample_cells(&contract, 5).unwrap(), 90);
        let mut admitted = selected.clone();
        admitted.max_cells = selected.validate(&contract, &VALUES).unwrap();
        assert!(admitted.validate(&contract, &VALUES).is_ok());
        admitted.forward_axes = None;
        assert!(admitted.validate(&contract, &VALUES).is_err());
        for axes in [vec![1, 1], vec![5], vec![usize::MAX]] {
            let invalid = profile(method, Some(axes));
            assert!(invalid.validate(&contract, &VALUES).is_err());
            assert!(
                integrate(
                    &mut Coupled::new(),
                    &invalid,
                    &VALUES,
                    Cancellation::default()
                )
                .is_err()
            );
        }
        let mut invalid = selected;
        invalid.sensitivity = DynamicSensitivity::Adjoint;
        assert!(invalid.validate(&contract, &VALUES).is_err());
    }
}

/// Exact quadrature derivatives include both the state tangent and direct parameter
/// dependence, preserving selected physical coordinates across scheduled reinit.
#[cfg(feature = "idas")]
#[test]
fn forward_quadrature_selected_physical_axes_offsets_and_admission_are_exact() {
    #[derive(Debug)]
    struct Flux(Coupled, Arc<std::sync::atomic::AtomicU64>);
    impl Oracle for Flux {
        fn contract(&self) -> &Contract {
            self.0.contract()
        }
        fn snapshot_owner(
            &self,
            bytes: usize,
        ) -> Result<Arc<dyn pse_math::AllocationOwner>, ProblemError> {
            self.0.snapshot_owner(bytes)
        }
        fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
            if function == Function::QuadratureFlux {
                entries(vec![(0, 0), (0, 2)])
            } else {
                self.0.support(mode, function)
            }
        }
        fn evaluate(
            &mut self,
            mode: usize,
            function: Function,
            time: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            if function != Function::QuadratureFlux {
                return self
                    .0
                    .evaluate(mode, function, time, state, parameters, derivatives);
            }
            if derivatives {
                let _ = self.1.compare_exchange(u64::MAX, time.to_bits(), std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst);
            }
            let jacobian = derivatives.then(|| {
                faer::sparse::SparseColMat::try_new_from_triplets(
                    1,
                    6,
                    &[
                        faer::sparse::Triplet::new(0, 0, 1.),
                        faer::sparse::Triplet::new(0, 2, 3.),
                    ],
                )
                .unwrap()
            });
            Ok(Evaluation {
                values: vec![state[0] + 3. * parameters[0]],
                jacobian: jacobian.map(|j| Arc::new(j).into()),
            })
        }
    }
    let first_derivative = Arc::new(std::sync::atomic::AtomicU64::new(u64::MAX));
    let mut oracle = Flux(Coupled::new(), first_derivative.clone());
    oracle.0.0.quadratures = vec![SemanticId::from_bytes([44; 16])];
    let mut p = profile(Method::Idas, Some(vec![4, 0, 2]));
    p.rtol = 0.001;
    p.atol.fill(0.001);
    p.out_rtol = Some(0.001);
    p.out_atol = vec![0.001];
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    let progress = || Arc::new(crate::solve::Progress::new(256));
    let result = forward_quadratures_observed(
        &mut oracle,
        &p,
        &VALUES,
        Cancellation::default(),
        progress(),
        &snapshot,
        32 << 20,
    )
    .unwrap();
    assert_eq!(
        result.report.termination,
        Termination::Completed,
        "{:?}",
        result.report.error
    );
    assert_eq!(result.report.forward_axes, [4, 0, 2]);
    assert_eq!(first_derivative.load(std::sync::atomic::Ordering::SeqCst), p.start.to_bits(),
        "quadrature sensitivity history must be initialized by IDAInitialSetup before stepping");
    let expected = [1.625, VALUES[2] + 0.5, VALUES[0]];
    for (actual, expected) in result.derivatives.unwrap().iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.01 * expected.abs().max(1.),
            "{actual} vs {expected}"
        );
    }
    assert!(result.reserved_bytes > 0);
    assert!(matches!(
        forward_quadratures_observed(
            &mut oracle,
            &p,
            &VALUES,
            Cancellation::default(),
            progress(),
            &snapshot,
            1
        ),
        Err(ProblemError::Limit { .. })
    ));
    let stopped = forward_quadratures_observed(
        &mut oracle,
        &p,
        &VALUES,
        Cancellation::new(true),
        progress(),
        &snapshot,
        32 << 20,
    )
    .unwrap();
    assert_eq!(stopped.report.termination, Termination::Cancelled);
    assert!(stopped.derivatives.is_none());
    p.forward_axes = Some(vec![]);
    let empty = forward_quadratures_observed(
        &mut oracle,
        &p,
        &VALUES,
        Cancellation::default(),
        progress(),
        &snapshot,
        32 << 20,
    )
    .unwrap();
    assert_eq!(empty.report.termination, Termination::Completed);
    assert_eq!(empty.derivatives, Some(vec![]));
    p.forward_axes = Some(vec![4]);
    p.out_atol = vec![f64::MIN_POSITIVE];
    p.parameter_scales[0] = f64::MAX;
    assert!(
        forward_quadratures_observed(
            &mut oracle,
            &p,
            &VALUES,
            Cancellation::default(),
            progress(),
            &snapshot,
            32 << 20
        )
        .is_err(),
        "underflowed derivative tolerances must refuse before native work"
    );
}


fn adjoint_profile(method: Method, selection: AdjointParameters) -> Profile {
    let mut profile = profile(method, None);
    profile.sensitivity = DynamicSensitivity::Adjoint;
    profile.adjoint_parameters = selection;
    profile
}
fn selected_gradient(oracle: &mut dyn Oracle, p: &Profile, values: &[f64]) -> Gradient {
    let mut weights = |r: &Report| Ok(r.samples.iter().map(|s| match s.time {
        0. => 0.3, 0.5 => 0.7, 1. => 1.2, _ => 0.,
    }).collect());
    let result = gradient(oracle, p, values, &mut weights, Cancellation::default(), 64 << 20).unwrap();
    assert_eq!(result.report.termination, Termination::Completed, "{:?}: {:?}", p.method, result.report.error);
    result
}
fn selected_native_width(result: &Gradient, method: Method, width: usize) {
    let adjoint = result.report.statistics.iter().find_map(|s| s.get("adjoint")).unwrap();
    if method == Method::Diffsol {
        assert_eq!(adjoint["parameter_product_width"], width);
    } else {
        let problems = adjoint["backward"].as_array().unwrap();
        assert!(!problems.is_empty(), "empty parameter demand retains the state adjoint");
        for problem in problems {
            assert_eq!(problem["parameter_quadrature_width"], width);
            assert_eq!(problem["quadrature_allocated"], width != 0);
        }
    }
}
#[test]
fn selected_adjoint_full_selected_empty_preserve_values_initial_direct_and_schedule() {
    for method in methods() {
        let full = selected_gradient(&mut Coupled::new(), &adjoint_profile(method, AdjointParameters::All), &VALUES);
        assert_eq!(full.parameter_axes, [0, 1, 2, 3, 4]);
        selected_native_width(&full, method, 5);
        for axes in [vec![4, 2, 3], vec![0], vec![1, 0], vec![]] {
            let selected = selected_gradient(&mut Coupled::new(), &adjoint_profile(method, AdjointParameters::Selected(axes.clone())), &VALUES);
            assert_eq!(selected.parameter_axes, axes);
            selected_native_width(&selected, method, axes.len());
            assert!(selected.reserved_bytes < full.reserved_bytes);
            let derivatives = selected.gradient.as_ref().unwrap();
            assert_eq!(derivatives.len(), axes.len());
            for (column, axis) in axes.iter().enumerate() {
                let expected = 0.3 * analytic(0., *axis).2 + 0.7 * analytic(0.5, *axis).2 + 1.2 * analytic(1., *axis).2;
                // IDAS uses the existing checkpointed-adjoint agreement from
                // dynamics::tests::adjoint::tolerance; forward interpolation's
                // absolute assertion does not bound a backward quadrature.
                let agreement = if method == Method::Idas { 1e-5 * (1. + expected.abs()) } else { 2e-5 };
                let full_derivative = full.gradient.as_ref().unwrap()[*axis];
                assert!((full_derivative - expected).abs() < agreement, "{method:?}, full axis {axis}: {full_derivative} vs {expected}");
                assert!((derivatives[column] - expected).abs() < agreement, "{method:?}, axis {axis}: {} vs {expected}; full {full_derivative}", derivatives[column]);
                assert!((derivatives[column] - full_derivative).abs() < agreement);
            }
            for (sample, reference) in selected.report.samples.iter().zip(&full.report.samples) {
                assert!(sample.state_sensitivities.is_empty() && sample.output_sensitivities.is_empty());
                assert_eq!(sample.state.len(), 2);
                let t = sample.time;
                let later = (t - 0.5).max(0.);
                let x = VALUES[2] * VALUES[0] + (VALUES[3] + VALUES[0]) * t + (VALUES[4] - VALUES[3]) * later;
                let y = 1. + VALUES[1] * (VALUES[2] * VALUES[0] * t + (VALUES[3] + VALUES[0]) * t * t / 2. + (VALUES[4] - VALUES[3]) * later * later / 2.);
                let rate = if t < 0.5 { VALUES[3] } else { VALUES[4] };
                assert!((sample.state[0] - x).abs() < 2e-5 && (sample.state[1] - y).abs() < 2e-5);
                assert!((sample.outputs[0] - (x + y + 2. * rate + 3. * VALUES[0])).abs() < 2e-5);
                assert!((sample.outputs[0] - reference.outputs[0]).abs() < 2e-5);
            }
        }
    }
}
#[test]
fn selected_adjoint_anchor_and_later_interval_keep_physical_identity() {
    for method in methods() {
        let mut oracle = Anchored::new(Coupled::new(), false).unwrap();
        let mut profile = oracle.profile(&adjoint_profile(method, AdjointParameters::All));
        let values = [5., 2., 0.4, 2., 1., 1., 3.];
        profile.adjoint_parameters = AdjointParameters::Selected(vec![6, 3]);
        let result = selected_gradient(&mut oracle, &profile, &values);
        assert_eq!(result.parameter_axes, [6, 3]);
        selected_native_width(&result, method, 2);
        let expected = [0.7 * analytic(0.5, 4).2 + 1.2 * analytic(1., 4).2,
            0.3 + 0.7 * 2. + 1.2 * 3.];
        for (actual, exact) in result.gradient.unwrap().iter().zip(expected) {
            assert!((actual - exact).abs() < 2e-5, "{method:?}: {actual} vs {exact}");
        }
    }
}
#[test]
fn selected_adjoint_admission_identity_and_cancelled_product() {
    for method in methods() {
        let contract = Coupled::new().0;
        let full = adjoint_profile(method, AdjointParameters::All);
        let mut keys = BTreeSet::new();
        for selection in [AdjointParameters::All, AdjointParameters::Selected(vec![]),
            AdjointParameters::Selected(vec![4, 2]), AdjointParameters::Selected(vec![2, 4])] {
            let profile = adjoint_profile(method, selection);
            assert!(keys.insert(profile_json(&profile).to_string()));
            assert!(serde_json::to_value(&profile).unwrap().get("adjoint_parameters").is_none());
        }
        for axes in [vec![1, 1], vec![5], vec![usize::MAX]] {
            let p = adjoint_profile(method, AdjointParameters::Selected(axes));
            assert!(p.validate(&contract, &VALUES).is_err());
        }
        let mut p = adjoint_profile(method, AdjointParameters::Selected(vec![2]));
        p.sensitivity = DynamicSensitivity::None;
        assert!(p.validate(&contract, &VALUES).is_err());
        let mut cotangent = |r: &Report| Ok(vec![1.; r.samples.len()]);
        let cancelled = gradient(&mut Coupled::new(), &full, &VALUES, &mut cotangent, Cancellation::new(true), 64 << 20).unwrap();
        assert_eq!(cancelled.report.termination, Termination::Cancelled);
        assert!(cancelled.gradient.is_none());
        assert_eq!(cancelled.parameter_axes, [0, 1, 2, 3, 4]);
    }
}


#[test]
fn selected_adjoint_one_parameter_without_fixed_inputs_and_derivative_failure() {
    #[derive(Debug)]
    struct Single(Contract);
    impl Oracle for Single {
        fn contract(&self) -> &Contract { &self.0 }
        fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
            entries(match f { Function::Initial | Function::Rhs => vec![(0, 1)],
                Function::Output => vec![(0, 0), (0, 1)], _ => vec![] })
        }
        fn evaluate(&mut self, _: usize, f: Function, _: f64, x: &[f64], p: &[f64], derivative: bool) -> Result<Evaluation, ProblemError> {
            let values = match f { Function::Initial | Function::Rhs => vec![p[0]],
                Function::Output => vec![x[0] + 2. * p[0]], _ => vec![] };
            let jacobian = derivative.then(|| {
                let entries = match f { Function::Initial | Function::Rhs => vec![(0, 1, 1.)],
                    Function::Output => vec![(0, 0, 1.), (0, 1, 2.)], _ => vec![] };
                faer::sparse::SparseColMat::try_new_from_triplets(values.len(), 2,
                    &entries.into_iter().map(|(r,c,v)| faer::sparse::Triplet::new(r,c,v)).collect::<Vec<_>>()).unwrap()
            });
            Ok(Evaluation { values, jacobian: jacobian.map(|j| Arc::new(j).into()) })
        }
    }
    #[derive(Debug)]
    struct Fails(Coupled);
    impl Oracle for Fails {
        fn contract(&self) -> &Contract { self.0.contract() }
        fn support(&self, mode: usize, f: Function) -> Vec<SupportEntry> { self.0.support(mode, f) }
        fn evaluate(&mut self, mode: usize, f: Function, t: f64, x: &[f64], p: &[f64], derivative: bool) -> Result<Evaluation, ProblemError> {
            if derivative && f == Function::Output { return Err(ProblemError::numerical("selected derivative fixture failure")); }
            self.0.evaluate(mode, f, t, x, p, derivative)
        }
    }
    for method in methods() {
        let mut contract = Coupled::new().0;
        contract.states.truncate(1); contract.differential.truncate(1); contract.parameters.truncate(1);
        let mut p = adjoint_profile(method, AdjointParameters::Selected(vec![0]));
        p.schedule.clear(); p.atol.truncate(1); p.parameter_scales.truncate(1);
        let result = selected_gradient(&mut Single(contract), &p, &[2.]);
        let expected = 0.3 * 3. + 0.7 * 3.5 + 1.2 * 4.;
        assert!((result.gradient.unwrap()[0] - expected).abs() < 2e-5);
        for sample in result.report.samples { assert!((sample.outputs[0] - (6. + 2. * sample.time)).abs() < 2e-5); }
        let p = adjoint_profile(method, AdjointParameters::Selected(vec![4]));
        let mut weights = |r: &Report| Ok(vec![1.; r.samples.len()]);
        let failed = gradient(&mut Fails(Coupled::new()), &p, &VALUES, &mut weights, Cancellation::default(), 64 << 20).unwrap();
        assert_ne!(failed.report.termination, Termination::Completed);
        assert_eq!(failed.parameter_axes, [4]);
        assert!(failed.gradient.is_none());
        assert!(!failed.report.samples.is_empty(), "completed weaker forward samples survive derivative failure");
    }
}
