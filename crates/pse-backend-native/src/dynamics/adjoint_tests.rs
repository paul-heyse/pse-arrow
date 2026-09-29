// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! ADR-0110 item 3: checkpointed adjoint gradients against forward sensitivities and
//! central finite differences, and the checkpoint allowance.
use super::*;
use std::cell::Cell;

/// A nonlinear reactor with a rate constant `k` and a feed `u`: as an ODE,
/// `x' = u − k·x²` from `x(0) = k`; as a semi-explicit DAE, `x' = u − z·x` with the
/// algebraic `0 = k·x − z`. The outputs are `x (+ z)` and `x·u`, so the second observes the
/// feed directly. `evaluations` counts every oracle call.
#[derive(Debug)]
struct Reactor {
    c: Contract,
    evaluations: Cell<usize>,
}
impl Reactor {
    fn new(dae: bool) -> Self {
        let n = if dae { 2 } else { 1 };
        Self {
            c: Contract {
                derivatives: pse_kernels::DerivativeOrder::Second,
                quadratures: vec![],
                balances: vec![],
                identity: ContentHash::from_bytes([60; 32]),
                states: (61..).take(n).map(id).collect(),
                differential: (0..n).map(|i| i == 0).collect(),
                parameters: vec![id(64), id(65)],
                outputs: vec![id(66), id(67)],
                events: vec![vec![]],
                signs: vec![],
            },
            evaluations: Cell::new(0),
        }
    }
    fn dae(&self) -> bool {
        self.c.states.len() == 2
    }
}
impl Oracle for Reactor {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        let n = self.c.states.len();
        let (k, u) = (n, n + 1);
        entries(match (f, self.dae()) {
            (Function::Initial, false) => vec![(0, k)],
            (Function::Initial, true) => vec![(0, k), (1, k)],
            (Function::Rhs, false) => vec![(0, 0), (0, k), (0, u)],
            (Function::Rhs, true) => vec![(0, 0), (0, 1), (0, u), (1, 0), (1, 1), (1, k)],
            (Function::Output, false) => vec![(0, 0), (1, 0), (1, u)],
            (Function::Output, true) => vec![(0, 0), (0, 1), (1, 0), (1, u)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        self.evaluations.set(self.evaluations.get() + 1);
        let n = self.c.states.len();
        let (k, u) = (p[0], p[1]);
        let (kc, uc) = (n, n + 1);
        let (values, entries) = match (f, self.dae()) {
            (Function::Initial, false) => (vec![k], vec![(0, kc, 1.0)]),
            (Function::Initial, true) => (vec![k, k * k], vec![(0, kc, 1.0), (1, kc, 2.0 * k)]),
            (Function::Rhs, false) => (
                vec![u - k * x[0] * x[0]],
                vec![(0, 0, -2.0 * k * x[0]), (0, kc, -x[0] * x[0]), (0, uc, 1.0)],
            ),
            (Function::Rhs, true) => (
                vec![u - x[1] * x[0], k * x[0] - x[1]],
                vec![
                    (0, 0, -x[1]),
                    (0, 1, -x[0]),
                    (0, uc, 1.0),
                    (1, 0, k),
                    (1, 1, -1.0),
                    (1, kc, x[0]),
                ],
            ),
            (Function::Output, false) => (
                vec![x[0], x[0] * u],
                vec![(0, 0, 1.0), (1, 0, u), (1, uc, x[0])],
            ),
            (Function::Output, true) => (
                vec![x[0] + x[1], x[0] * u],
                vec![(0, 0, 1.0), (0, 1, 1.0), (1, 0, u), (1, uc, x[0])],
            ),
            _ => (vec![], vec![]),
        };
        let jacobian = derivatives.then(|| {
            faer::sparse::SparseColMat::try_new_from_triplets(
                values.len(),
                n + 2,
                &entries
                    .into_iter()
                    .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        });
        Ok(Evaluation { values, jacobian })
    }
    /// The exact second derivatives: `−2k` in x·x and `−2x` in k·x of the ODE rate,
    /// `−1` in z·x of the DAE rate and `1` in k·x of its constraint, `2` in k·k of the DAE's
    /// initial `k²`, and `1` in u·x of the output `x·u`.
    fn weighted_hessian(
        &mut self,
        _: usize,
        f: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        w: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        self.evaluations.set(self.evaluations.get() + 1);
        let n = self.c.states.len();
        let (kc, uc) = (n, n + 1);
        let entries = match (f, self.dae()) {
            (Function::Initial, true) => vec![(kc, kc, 2.0 * w[1])],
            (Function::Rhs, false) => vec![(0, 0, -2.0 * p[0] * w[0]), (kc, 0, -2.0 * x[0] * w[0])],
            (Function::Rhs, true) => vec![(1, 0, -w[0]), (kc, 0, w[1])],
            (Function::Output, _) => vec![(uc, 0, w[1])],
            _ => vec![],
        };
        Ok(faer::sparse::SparseColMat::try_new_from_triplets(
            n + 2,
            n + 2,
            &entries
                .into_iter()
                .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                .collect::<Vec<_>>(),
        )
        .unwrap())
    }
}
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}

/// The relative integration tolerance of `method`, the declared relative agreement of the
/// adjoint with the forward sensitivities and with central differences, and the relative
/// difference step, which exceeds the integration noise. IDAS DAE forward sensitivities
/// fail their first error test at t = 0 at tighter tolerances (a known limitation
/// predating the adjoint), so IDAS runs at 1e-8.
fn tolerance(method: Method) -> (f64, f64, f64, f64) {
    if method == Method::Idas {
        (1e-8, 1e-5, 1e-4, 1e-3)
    } else {
        (1e-10, 1e-7, 1e-5, 1e-4)
    }
}
/// The reactor's profile on `method`; a scheduled feed changes at the samples at 1 s and
/// 1.5 s, which observe the later intervals.
fn reactor_profile(method: Method, scheme: DiffsolMethod, dae: bool, scheduled: bool) -> Profile {
    let mut p = Profile {
        method,
        end: 2.0,
        samples: vec![0.0, 0.3, 0.7, 1.0, 1.5, 2.0],
        rtol: tolerance(method).0,
        atol: vec![1e-12; if dae { 2 } else { 1 }],
        initial_step: 1e-5,
        parameter_scales: vec![1.0, 1.0],
        sensitivity: DynamicSensitivity::Adjoint,
        ..Default::default()
    };
    p.diffsol.method = scheme;
    if scheduled {
        p.schedule = vec![ScheduledInput {
            parameter: 1,
            times: vec![1.0, 1.5],
        }];
    }
    p
}
/// The integration vector: `k`, then the feed or its three intervals.
fn reactor_parameters(scheduled: bool) -> Vec<f64> {
    if scheduled {
        vec![0.8, 1.5, 0.5, 2.5]
    } else {
        vec![0.8, 1.5]
    }
}
/// The weights of J = Σ wᵢₒ·yₒ(tᵢ).
fn weights(samples: usize) -> Vec<f64> {
    (0..samples)
        .flat_map(|i| [1.0 + i as f64, -0.5 * (1.0 + i as f64)])
        .collect()
}
fn functional(report: &Report) -> f64 {
    let w = weights(report.samples.len());
    report
        .samples
        .iter()
        .flat_map(|s| s.outputs.iter())
        .zip(&w)
        .map(|(y, w)| y * w)
        .sum()
}
fn cases(method: Method) -> Vec<(DiffsolMethod, bool)> {
    let schemes = if method == Method::Idas {
        vec![DiffsolMethod::Bdf]
    } else {
        vec![
            DiffsolMethod::Bdf,
            DiffsolMethod::TrBdf2,
            DiffsolMethod::Esdirk34,
            DiffsolMethod::Tsit45,
        ]
    };
    schemes
        .into_iter()
        .flat_map(|scheme| [(scheme, false), (scheme, true)])
        // The explicit scheme integrates mass-free ODEs only.
        .filter(|(scheme, dae)| !(*scheme == DiffsolMethod::Tsit45 && *dae))
        .collect()
}
fn adjoint(oracle: &mut Reactor, p: &Profile, parameters: &[f64]) -> Gradient {
    gradient(
        oracle,
        p,
        parameters,
        &mut |report: &Report| Ok(weights(report.samples.len())),
        Arc::default(),
        256 << 20,
    )
    .unwrap()
}

/// The adjoint gradient of J = Σ wᵢₒ·yₒ(tᵢ) equals the forward sensitivities contracted
/// with the same weights, and central finite differences of J, for every scheme, the ODE
/// and the DAE, and a scheduled feed whose intervals cross two changes, with the declared
/// tolerances of [`tolerance`].
fn adjoint_equals_forward_and_differences(method: Method) {
    for (scheme, dae) in cases(method) {
        for scheduled in [false, true] {
            let case = format!("{method:?} {scheme:?} dae={dae} scheduled={scheduled}");
            let mut p = reactor_profile(method, scheme, dae, scheduled);
            let parameters = reactor_parameters(scheduled);
            let g = adjoint(&mut Reactor::new(dae), &p, &parameters);
            assert_eq!(g.report.termination, Termination::Completed, "{case}: {:?}", g.report.error);
            let adjoint = g.gradient.unwrap();
            assert_eq!(adjoint.len(), parameters.len());
            assert_eq!(g.report.samples.len(), p.samples.len(), "{case}");
            assert!(g.report.samples.iter().all(|s| s.output_sensitivities.is_empty()));
            // Diffsol holds every segment's checkpoints, IDAS one segment's at a time.
            let held = if method == Method::Idas { 1 } else { 2 * p.segments() };
            assert!(g.checkpoints >= held, "{case}: {}", g.checkpoints);
            assert_eq!(g.reserved_bytes, p.checkpoint_bytes(&Reactor::new(dae).c).unwrap());
            // Forward sensitivities of the same samples.
            p.sensitivity = DynamicSensitivity::Forward;
            let forward = integrate(&mut Reactor::new(dae), &p, &parameters, Arc::default()).unwrap();
            assert_eq!(forward.termination, Termination::Completed, "{case}: {:?}", forward.error);
            let w = weights(forward.samples.len());
            let np = parameters.len();
            let contracted = (0..np)
                .map(|j| {
                    forward
                        .samples
                        .iter()
                        .enumerate()
                        .flat_map(|(i, s)| (0..2).map(move |o| (i, o, s)))
                        .map(|(i, o, s)| w[i * 2 + o] * s.output_sensitivities[o * np + j])
                        .sum::<f64>()
                })
                .collect::<Vec<_>>();
            // Central differences of the functional.
            p.sensitivity = DynamicSensitivity::None;
            let differences = (0..np)
                .map(|j| {
                    let h = tolerance(method).3 * (1.0 + parameters[j].abs());
                    let at = |delta: f64| {
                        let mut q = parameters.clone();
                        q[j] += delta;
                        let r = integrate(&mut Reactor::new(dae), &p, &q, Arc::default()).unwrap();
                        assert_eq!(r.termination, Termination::Completed, "{case}");
                        functional(&r)
                    };
                    (at(h) - at(-h)) / (2.0 * h)
                })
                .collect::<Vec<_>>();
            for j in 0..np {
                assert!(
                    (adjoint[j] - contracted[j]).abs()
                        <= tolerance(method).1 * (1.0 + contracted[j].abs()),
                    "{case}: column {j}: adjoint {} forward {}",
                    adjoint[j],
                    contracted[j]
                );
                assert!(
                    (adjoint[j] - differences[j]).abs()
                        <= tolerance(method).2 * (1.0 + differences[j].abs()),
                    "{case}: column {j}: adjoint {} differences {}",
                    adjoint[j],
                    differences[j]
                );
            }
        }
    }
}
#[test]
fn diffsol_adjoint_gradient_equals_forward_and_differences() {
    adjoint_equals_forward_and_differences(Method::Diffsol);
}
#[cfg(feature = "idas")]
#[test]
fn idas_adjoint_gradient_equals_forward_and_differences() {
    adjoint_equals_forward_and_differences(Method::Idas);
}

/// A backward pass that starts where the forward state is nearly steady: Diffsol's own
/// adjoint start state would probe a step of 0.01·|y|/|y'| back from the stop, far outside
/// the segment's checkpoints (a Diffsol error found by shooting at an optimum). The feed
/// holds the reactor at its steady state `u = k³` and moves it by one part in a million in
/// the last interval; every scheme's adjoint gradient equals the forward sensitivities
/// (declared relative tolerance of `tolerance`).
#[test]
fn diffsol_adjoint_starts_at_a_nearly_steady_stop() {
    let steady = 0.8f64.powi(3);
    let parameters = [0.8, steady, steady, steady * (1.0 + 1e-6)];
    for (scheme, _) in cases(Method::Diffsol).into_iter().filter(|(_, dae)| !dae) {
        let mut p = reactor_profile(Method::Diffsol, scheme, false, true);
        let g = adjoint(&mut Reactor::new(false), &p, &parameters);
        assert_eq!(g.report.termination, Termination::Completed, "{scheme:?}: {:?}", g.report.error);
        let adjoint = g.gradient.unwrap();
        p.sensitivity = DynamicSensitivity::Forward;
        let forward = integrate(&mut Reactor::new(false), &p, &parameters, Arc::default()).unwrap();
        let w = weights(forward.samples.len());
        let np = parameters.len();
        for (j, value) in adjoint.iter().enumerate() {
            let contracted = forward
                .samples
                .iter()
                .enumerate()
                .flat_map(|(i, s)| (0..2).map(move |o| (i, o, s)))
                .map(|(i, o, s)| w[i * 2 + o] * s.output_sensitivities[o * np + j])
                .sum::<f64>();
            assert!(
                (value - contracted).abs() <= tolerance(Method::Diffsol).1 * (1.0 + contracted.abs()),
                "{scheme:?}: column {j}: adjoint {value} forward {contracted}"
            );
        }
    }
}

/// The checkpoints stay within `AdjointSettings` and are charged against the caller's
/// memory before any native work.
fn checkpoints_bounded(method: Method) {
    let dae = true;
    let mut p = reactor_profile(method, DiffsolMethod::Bdf, dae, true);
    let parameters = reactor_parameters(true);
    let contract = Reactor::new(dae).c;
    let segments = p.segments();
    assert_eq!(segments, 3);
    // Frequent checkpoints: more than the two ends per segment, within the maximum, and
    // the same gradient.
    let reference = adjoint(&mut Reactor::new(dae), &p, &parameters);
    p.adjoint.steps_between_checkpoints = PositiveCount::try_new(3).unwrap();
    let frequent = adjoint(&mut Reactor::new(dae), &p, &parameters);
    assert_eq!(frequent.report.termination, Termination::Completed, "{:?}", frequent.report.error);
    assert!(frequent.checkpoints > reference.checkpoints, "{} vs {}", frequent.checkpoints, reference.checkpoints);
    assert!(frequent.checkpoints <= p.adjoint.max_checkpoints.into_inner());
    for (a, b) in frequent.gradient.unwrap().iter().zip(&reference.gradient.unwrap()) {
        assert!((a - b).abs() <= tolerance(method).1 * (1.0 + b.abs()), "{a} vs {b}");
    }
    // The forward pass stops with a typed memory limit when it needs more checkpoints
    // than the maximum; it returns no gradient.
    p.adjoint.max_checkpoints = PositiveCount::try_new(2 * segments + 1).unwrap();
    let limited = adjoint(&mut Reactor::new(dae), &p, &parameters);
    assert_eq!(limited.report.termination, Termination::Failed);
    assert!(limited.gradient.is_none());
    assert!(
        matches!(
            limited.report.error,
            Some(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ),
        "{:?}",
        limited.report.error
    );
    // Fewer than two checkpoints per segment is refused at admission.
    p.adjoint.max_checkpoints = PositiveCount::try_new(2 * segments - 1).unwrap();
    assert!(matches!(
        p.validate(&contract, &parameters),
        Err(ProblemError::Contract(_))
    ));
    // The estimate grows with the checkpoints allowed and is charged before native work:
    // an allowance one byte short evaluates nothing.
    p.adjoint = AdjointSettings::default();
    let bytes = p.checkpoint_bytes(&contract).unwrap();
    let mut fewer = p.clone();
    fewer.adjoint.max_checkpoints = PositiveCount::try_new(2 * segments).unwrap();
    assert!(fewer.checkpoint_bytes(&contract).unwrap() < bytes);
    let mut oracle = Reactor::new(dae);
    let refused = gradient(
        &mut oracle,
        &p,
        &parameters,
        &mut |r: &Report| Ok(weights(r.samples.len())),
        Arc::default(),
        bytes - 1,
    )
    .unwrap_err();
    assert!(
        matches!(refused, ProblemError::Limit { kind: crate::LimitKind::Memory, .. }),
        "{refused:?}"
    );
    assert_eq!(oracle.evaluations.get(), 0);
    let admitted = gradient(
        &mut oracle,
        &p,
        &parameters,
        &mut |r: &Report| Ok(weights(r.samples.len())),
        Arc::default(),
        bytes,
    )
    .unwrap();
    assert!(admitted.gradient.is_some());
    assert_eq!(admitted.reserved_bytes, bytes);
}
#[test]
fn checkpoint_memory_bounded() {
    checkpoints_bounded(Method::Diffsol);
}
#[cfg(feature = "idas")]
#[test]
fn idas_checkpoint_memory_bounded() {
    checkpoints_bounded(Method::Idas);
}

/// Events, resets and quadratures are outside the adjoint profile and refused before
/// native work; a gradient needs the adjoint profile, and the cotangent covers every
/// output of every sample.
#[test]
fn adjoint_profile_limits_are_typed_refusals() {
    let p = reactor_profile(Method::Diffsol, DiffsolMethod::Bdf, false, false);
    let parameters = reactor_parameters(false);
    let mut evented = Reactor::new(false);
    evented.c.events = vec![
        vec![Event {
            id: id(70),
            terminal: false,
            next_mode: 0,
            tolerance: 1e-8,
            direction: EventDirection::Either,
        }],
    ];
    assert!(matches!(
        p.validate(&evented.c, &parameters),
        Err(ProblemError::Unsupported(_))
    ));
    let mut integrated = Reactor::new(false);
    integrated.c.quadratures = vec![id(71)];
    let mut q = p.clone();
    q.out_rtol = Some(1e-8);
    q.out_atol = vec![1e-10];
    assert!(matches!(
        q.validate(&integrated.c, &parameters),
        Err(ProblemError::Unsupported(_))
    ));
    // A change at the end leaves the final sample alone in its interval.
    let mut late = p.clone();
    late.schedule = vec![ScheduledInput {
        parameter: 1,
        times: vec![1.0, 2.0],
    }];
    assert!(matches!(
        late.validate(&Reactor::new(false).c, &reactor_parameters(true)),
        Err(ProblemError::Contract(_))
    ));
    let mut forward = p.clone();
    forward.sensitivity = DynamicSensitivity::Forward;
    let mut oracle = Reactor::new(false);
    assert!(matches!(
        gradient(&mut oracle, &forward, &parameters, &mut |_: &Report| Ok(vec![]), Arc::default(), 1 << 30),
        Err(ProblemError::Contract(_))
    ));
    let short = gradient(
        &mut oracle,
        &p,
        &parameters,
        &mut |_: &Report| Ok(vec![1.0]),
        Arc::default(),
        1 << 30,
    )
    .unwrap();
    assert_eq!(short.report.termination, Termination::Failed);
    assert!(short.gradient.is_none());
}

/// ADR-0110 item 4: the IDAS forward-over-adjoint Hessian of J = Σ wᵢₒ·yₒ(tᵢ) equals
/// central differences of J's forward-sensitivity gradient (Diffsol BDF at a relative
/// tolerance of 1e-10, steps of 1e-4 relative) within a declared relative tolerance of
/// 1e-4, for the ODE and the DAE, with and without a scheduled feed whose intervals cross
/// two changes. The Hessian over all directions is symmetric within integration error,
/// its gradient is the first-order adjoint's, a subset of directions gives the matching
/// block, and the backward problems' Newton counters are recorded.
#[cfg(feature = "idas")]
#[test]
fn second_order_adjoint_matches_finite_difference() {
    for dae in [false, true] {
        for scheduled in [false, true] {
            let case = format!("dae={dae} scheduled={scheduled}");
            let p = reactor_profile(Method::Idas, DiffsolMethod::Bdf, dae, scheduled);
            let parameters = reactor_parameters(scheduled);
            let np = parameters.len();
            let all = (0..np).collect::<Vec<_>>();
            let second = |directions: &[usize]| {
                hessian(
                    &mut Reactor::new(dae),
                    &p,
                    &parameters,
                    directions,
                    &mut |report: &Report| Ok(weights(report.samples.len())),
                    Arc::default(),
                    256 << 20,
                )
                .unwrap()
            };
            let result = second(&all);
            assert_eq!(result.report.termination, Termination::Completed, "{case}: {:?}", result.report.error);
            let h = result.hessian.unwrap();
            let g = result.gradient.unwrap();
            let adjoint = adjoint(&mut Reactor::new(dae), &p, &parameters).gradient.unwrap();
            for j in 0..np {
                assert!(
                    (g[j] - adjoint[j]).abs() <= 1e-6 * (1.0 + adjoint[j].abs()),
                    "{case}: gradient {j}: {} vs {}",
                    g[j],
                    adjoint[j]
                );
            }
            // The forward pass integrated the state sensitivities its tangents need.
            assert!(result.report.samples.iter().all(|s| s.state_sensitivities.len() == np * p.atol.len()));
            let statistics = result
                .report
                .statistics
                .iter()
                .find_map(|s| s.get("adjoint"))
                .unwrap();
            assert!(statistics["hessian_asymmetry"].as_f64().unwrap() < 1e-5, "{case}: {statistics}");
            let backward = statistics["backward"].as_array().unwrap();
            assert_eq!(backward.len(), np, "{case}");
            eprintln!("{case}: backward Newton counters {backward:?}");
            // Central differences of the forward-sensitivity gradient.
            let mut reference = reactor_profile(Method::Diffsol, DiffsolMethod::Bdf, dae, scheduled);
            reference.sensitivity = DynamicSensitivity::Forward;
            let contracted = |q: &[f64]| -> Vec<f64> {
                let r = integrate(&mut Reactor::new(dae), &reference, q, Arc::default()).unwrap();
                assert_eq!(r.termination, Termination::Completed, "{case}: {:?}", r.error);
                let w = weights(r.samples.len());
                (0..np)
                    .map(|j| {
                        r.samples
                            .iter()
                            .enumerate()
                            .flat_map(|(i, s)| (0..2).map(move |o| (i, o, s)))
                            .map(|(i, o, s)| w[i * 2 + o] * s.output_sensitivities[o * np + j])
                            .sum::<f64>()
                    })
                    .collect()
            };
            for j in 0..np {
                let step = 1e-4 * (1.0 + parameters[j].abs());
                let shifted = |delta: f64| {
                    let mut q = parameters.clone();
                    q[j] += delta;
                    contracted(&q)
                };
                let (plus, minus) = (shifted(step), shifted(-step));
                for i in 0..np {
                    let difference = (plus[i] - minus[i]) / (2.0 * step);
                    assert!(
                        (h[(i, j)] - difference).abs() <= 1e-4 * (1.0 + difference.abs()),
                        "{case}: H[{i},{j}] {} vs differences {difference}",
                        h[(i, j)]
                    );
                }
            }
            // A subset of directions gives the same block.
            let subset = [np - 1, 0];
            let block = second(&subset).hessian.unwrap();
            for (a, i) in subset.iter().enumerate() {
                for (b, j) in subset.iter().enumerate() {
                    assert!(
                        (block[(a, b)] - h[(*i, *j)]).abs() <= 1e-6 * (1.0 + h[(*i, *j)].abs()),
                        "{case}: block [{a},{b}]"
                    );
                }
            }
        }
    }
}

/// The second-order route is refused before native work outside its profile: on Diffsol,
/// without declared second derivatives, with directions that are not distinct
/// integration columns, and above the caller's allowance.
#[cfg(feature = "idas")]
#[test]
fn second_order_route_limits_are_typed_refusals() {
    let parameters = reactor_parameters(true);
    let run = |oracle: &mut Reactor, p: &Profile, directions: &[usize], memory: usize| {
        hessian(
            oracle,
            p,
            &parameters,
            directions,
            &mut |r: &Report| Ok(weights(r.samples.len())),
            Arc::default(),
            memory,
        )
    };
    let idas = reactor_profile(Method::Idas, DiffsolMethod::Bdf, true, true);
    let diffsol = reactor_profile(Method::Diffsol, DiffsolMethod::Bdf, true, true);
    let mut oracle = Reactor::new(true);
    assert!(matches!(run(&mut oracle, &diffsol, &[0], 1 << 30), Err(ProblemError::Unsupported(_))));
    let mut first = Reactor::new(true);
    first.c.derivatives = pse_kernels::DerivativeOrder::First;
    assert!(matches!(run(&mut first, &idas, &[0], 1 << 30), Err(ProblemError::Unsupported(_))));
    for directions in [&[][..], &[0, 0], &[4]] {
        assert!(matches!(run(&mut oracle, &idas, directions, 1 << 30), Err(ProblemError::Contract(_))));
    }
    let mut forward = idas.clone();
    forward.sensitivity = DynamicSensitivity::Forward;
    assert!(matches!(run(&mut oracle, &forward, &[0], 1 << 30), Err(ProblemError::Contract(_))));
    let bytes = idas.admit_second_order(&oracle.c, &[0, 1]).unwrap();
    assert!(bytes > idas.checkpoint_bytes(&oracle.c).unwrap());
    let refused = run(&mut oracle, &idas, &[0, 1], bytes - 1).unwrap_err();
    assert!(matches!(refused, ProblemError::Limit { kind: crate::LimitKind::Memory, .. }), "{refused:?}");
    assert_eq!(oracle.evaluations.get(), 0);
    let admitted = run(&mut oracle, &idas, &[0, 1], bytes).unwrap();
    assert!(admitted.hessian.is_some());
    assert_eq!(admitted.reserved_bytes, bytes);
}
