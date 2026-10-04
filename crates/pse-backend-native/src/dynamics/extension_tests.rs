// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! ADR-0110 profile extensions: IDAS schedules, events, constraints, correctors, Krylov
//! solvers and steady starts; the sparse sensitivity products (F12).
use super::*;

/// A Toy whose first right-hand-side trial after `after` is a recoverable domain failure.
#[cfg(feature = "idas")]
#[derive(Debug)]
struct LateTrial {
    toy: Toy,
    after: f64,
    rejected: usize,
}
#[cfg(feature = "idas")]
impl Oracle for LateTrial {
    fn contract(&self) -> &Contract {
        self.toy.contract()
    }
    fn support(&self, m: usize, f: Function) -> Vec<SupportEntry> {
        self.toy.support(m, f)
    }
    fn evaluate(
        &mut self,
        m: usize,
        f: Function,
        t: f64,
        x: &[f64],
        p: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        if f == Function::Rhs && t > self.after && self.rejected == 0 {
            self.rejected += 1;
            return Err(pse_math::MathError::Domain {
                source_id: id(9),
                requirement: "intentional recoverable trial after a scheduled change",
            }
            .into());
        }
        self.toy.evaluate(m, f, t, x, p, d)
    }
}
/// L-D1/S09, I6: forward sensitivities cross a scheduled input change on IDAS, which
/// restarts with `IDAReInit`/`IDASensReInit`/`IDACalcIC` and recovers a rejected trial
/// after it. The input takes one integration parameter per interval.
#[cfg(feature = "idas")]
#[test]
fn idas_scheduled_inputs_with_recoverable_trials() {
    for dae in [false, true] {
        let mut p = profile(dae);
        p.samples = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        p.method = Method::Auto;
        p.trial_failures = TrialPolicy::Recoverable;
        p.sensitivity = DynamicSensitivity::Forward;
        p.schedule = vec![ScheduledInput {
            parameter: 0,
            times: vec![0.5],
        }];
        let mut oracle = LateTrial {
            toy: Toy::new(dae, false),
            after: 0.55,
            rejected: 0,
        };
        let selected = p
            .resolve_for(
                oracle.contract(),
                &[2.0, 3.0],
                &crate::execution::Snapshot::observe(&crate::execution::LINKED),
                DynamicDemand::Base,
                oracle.contract().derivatives,
                &[],
            )
            .unwrap();
        assert_eq!(selected.resolved_method().unwrap(), Method::Idas);
        let r = integrate(&mut oracle, &p, &[2.0, 3.0], Arc::default()).unwrap();
        assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
        assert_eq!(oracle.rejected, 1);
        assert_eq!(r.samples.len(), p.samples.len());
        // One recorded transition, settled with the consistent restarted state.
        assert_eq!(r.events.len(), 1);
        assert!(r.events[0].event.is_none() && r.events[0].time == 0.5);
        let after = r.events[0].after.as_ref().unwrap();
        assert!((after[0] - 2.0 * (-1.0f64).exp()).abs() < 1e-6);
        if dae {
            // IDACalcIC recomputed the algebraic state under the new parameters.
            assert!((after[1] - 2.0 * after[0]).abs() < 1e-6);
        }
        assert_eq!(r.statistics.len(), 2, "one statistics record per segment");
        for s in &r.samples {
            // Columns: the first interval's value p₀ = 2, then the second's p₁ = 3.
            let (y, dy, direct, later) = if s.time < 0.5 {
                (
                    2.0 * (-2.0 * s.time).exp(),
                    (1.0 - 2.0 * s.time) * (-2.0 * s.time).exp(),
                    1.0,
                    (0.0, 0.0),
                )
            } else {
                // The first interval's sensitivity is carried by the state and decays at
                // the new rate; the second interval's own column starts at the change.
                let carried = (1.0 - 2.0 * 0.5) * (-1.0f64).exp();
                let y = 2.0 * (-1.0 - 3.0 * (s.time - 0.5)).exp();
                (
                    y,
                    carried * (-3.0 * (s.time - 0.5)).exp(),
                    0.0,
                    (-(s.time - 0.5) * y, 1.0),
                )
            };
            let columns = 2;
            assert!((s.state[0] - y).abs() < 1e-6, "{s:?}");
            assert!((s.state_sensitivities[0] - dy).abs() < 1e-5, "{s:?}");
            assert!((s.state_sensitivities[1] - later.0).abs() < 1e-5, "{s:?}");
            assert!(
                (s.output_sensitivities[0] - dy - direct).abs() < 1e-5,
                "{s:?}"
            );
            assert!(
                (s.output_sensitivities[1] - later.0 - later.1).abs() < 1e-5,
                "{s:?}"
            );
            if dae {
                assert!((s.state[1] - 2.0 * y).abs() < 1e-6, "{s:?}");
                assert!(
                    (s.state_sensitivities[columns] - 2.0 * dy).abs() < 1e-5,
                    "{s:?}"
                );
            }
        }
        // The Diffsol route reproduces the same trajectory without the rejected trial.
        let mut diffsol = p.clone();
        diffsol.method = Method::Diffsol;
        diffsol.trial_failures = TrialPolicy::Terminal;
        let reference = integrate(
            &mut Toy::new(dae, false),
            &diffsol,
            &[2.0, 3.0],
            Arc::default(),
        )
        .unwrap();
        assert_eq!(reference.termination, Termination::Completed);
        for (a, b) in r.samples.iter().zip(&reference.samples) {
            assert!((a.state[0] - b.state[0]).abs() < 1e-6);
            for (u, v) in a.output_sensitivities.iter().zip(&b.output_sensitivities) {
                assert!((u - v).abs() < 1e-5);
            }
        }
    }
    // Output quadratures continue across the restart.
    let mut oracle = Toy::new(false, false);
    oracle.c.quadratures = vec![id(9)];
    let p = Profile {
        method: Method::Idas,
        samples: vec![0.0, 0.3, 0.5, 0.8, 1.0],
        parameter_scales: vec![1.0],
        rtol: 1e-9,
        out_rtol: Some(1e-9),
        out_atol: vec![1e-10],
        schedule: vec![ScheduledInput {
            parameter: 0,
            times: vec![0.5],
        }],
        ..Default::default()
    };
    let r = integrate(&mut oracle, &p, &[1.0, 2.0], Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        assert!((s.state[0] - 1.0 - s.integrals[0]).abs() < 1e-6, "{s:?}");
    }
}
/// A state-triggered reset (`x0 = 1` sets `x0 = 3`) with a declared crossing direction.
#[cfg(feature = "idas")]
fn directed(dae: bool, direction: EventDirection, terminal: bool) -> ResetToy {
    let mut toy = Toy::new(dae, true);
    toy.c.events[0][0].direction = direction;
    toy.c.events[0][0].terminal = terminal;
    if terminal {
        toy.c.events[0][0].next_mode = 0;
    }
    ResetToy(toy)
}
/// L-D5: IDAS roots without sensitivities — resets, mode changes, crossing directions,
/// terminal events and declared impulses — reproduce the Diffsol route.
#[cfg(feature = "idas")]
#[test]
fn idas_events_without_sensitivities() {
    let root = 2f64.ln() / 2.0;
    let go =
        |oracle: &mut ResetToy, p: &Profile| integrate(oracle, p, &[2.0], Arc::default()).unwrap();
    for dae in [false, true] {
        let mut p = profile(dae);
        p.method = Method::Idas;
        p.samples = vec![0.0, 0.25, 0.5, 1.0];
        let r = go(&mut directed(dae, EventDirection::Either, false), &p);
        assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
        assert_eq!(r.events.len(), 1);
        assert!((r.events[0].time - root).abs() < 1e-6);
        assert!((r.events[0].before[0] - 1.0).abs() < 1e-6);
        assert!((r.events[0].after.as_ref().unwrap()[0] - 3.0).abs() < 1e-12);
        let last = r.samples.last().unwrap();
        assert_eq!(last.mode, 1);
        assert!(
            (last.state[0] - 6.0 * (-2.0f64).exp()).abs() < 1e-5,
            "{last:?}"
        );
        if dae {
            assert!(
                (last.state[1] - 12.0 * (-2.0f64).exp()).abs() < 1e-5,
                "{last:?}"
            );
        }
        let mut diffsol = p.clone();
        diffsol.method = Method::Diffsol;
        let reference = go(&mut directed(dae, EventDirection::Either, false), &diffsol);
        assert_eq!(reference.samples.len(), r.samples.len());
        for (a, b) in r.samples.iter().zip(&reference.samples) {
            assert_eq!(a.mode, b.mode);
            assert!((a.state[0] - b.state[0]).abs() < 1e-6, "{a:?} {b:?}");
        }
        // The guard decreases through zero: a rising-only event never fires.
        let r = go(&mut directed(dae, EventDirection::Rising, false), &p);
        assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
        assert!(r.events.is_empty());
        assert!((r.samples.last().unwrap().state[0] - 2.0 * (-2.0f64).exp()).abs() < 1e-6);
        let r = go(&mut directed(dae, EventDirection::Falling, false), &p);
        assert_eq!(r.events.len(), 1);
        // A terminal event stops at the root, keeping only completed samples.
        let r = go(&mut directed(dae, EventDirection::Either, true), &p);
        assert_eq!(r.termination, Termination::Event, "{:?}", r.error);
        assert!((r.completed_time - root).abs() < 1e-6);
        assert!(r.events[0].after.is_none());
        assert_eq!(r.samples.len(), 2);
    }
    // Diffsol refuses a directional event before native work.
    let mut p = profile(false);
    p.method = Method::Diffsol;
    let toy = directed(false, EventDirection::Rising, false);
    assert!(matches!(
        p.validate(toy.contract(), &[2.0]),
        Err(ProblemError::Unsupported(_))
    ));
    // Declared impulses settle across the IDAS reset; undeclared jumps are refused.
    let mut event = Toy::new(false, true);
    event.c.quadratures = vec![id(8)];
    event.c.balances = vec![Balance {
        id: id(8),
        inventory: id(9),
        flux: id(8),
        tolerance: 1e-6,
        transfers: Default::default(),
    }];
    let mut p = Profile {
        method: Method::Idas,
        samples: vec![0.0, 0.2, 0.4, 0.7, 1.0],
        parameter_scales: vec![1.0],
        rtol: 1e-9,
        out_rtol: Some(1e-9),
        out_atol: vec![1e-10],
        ..Default::default()
    };
    let r = integrate(&mut event, &p, &[1.0], Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::Failed);
    assert!(r.error.unwrap().to_string().contains("transfer"));
    let impulse = (-0.25f64).exp();
    event.c.balances[0].transfers.insert(id(5));
    let r = integrate(&mut event, &p, &[1.0], Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        let jump = if s.time >= 0.25 { impulse } else { 0.0 };
        assert!(
            (s.state[0] - r.consistent_initial[0] - s.integrals[0] - jump).abs() < 1e-6,
            "{s:?}"
        );
    }
    // Every recorded transition counts against the event allowance.
    p.max_events = 1;
    p.schedule = vec![ScheduledInput {
        parameter: 0,
        times: vec![0.5],
    }];
    let r = integrate(&mut event, &p, &[1.0, 1.0], Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::EventLimit);
}
/// ADR-0110 item 1: events with sensitivities stay on Diffsol; IDAS refuses them with a
/// typed reason before any native work, including through `Auto` with recoverable trials.
#[test]
fn idas_events_with_sensitivities_refused() {
    let toy = Toy::new(false, true);
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    for (method, trials) in [
        (Method::Idas, TrialPolicy::Terminal),
        (Method::Auto, TrialPolicy::Recoverable),
    ] {
        let mut p = profile(false);
        p.method = method;
        p.trial_failures = trials;
        p.sensitivity = DynamicSensitivity::Forward;
        if !cfg!(feature = "idas") {
            assert!(
                p.resolve_for(
                    &toy.c,
                    &[2.0],
                    &snapshot,
                    DynamicDemand::Base,
                    toy.c.derivatives,
                    &[]
                )
                .is_err()
            );
            continue;
        }
        let Err(ProblemError::DynamicRouteRefused(decision)) = p.resolve_for(
            &toy.c,
            &[2.0],
            &snapshot,
            DynamicDemand::Base,
            toy.c.derivatives,
            &[],
        ) else {
            panic!("IDAS event sensitivities were admitted");
        };
        assert_eq!(decision.selected, None);
        let idas = decision
            .candidates
            .iter()
            .find(|candidate| candidate.method == Method::Idas)
            .unwrap();
        assert!(
            idas.causes.iter().any(|cause| matches!(cause.as_ref(), ProblemError::Unsupported(reason) if reason.contains("Diffsol owns reset sensitivities"))),
            "{decision:?}"
        );
        p.sensitivity = DynamicSensitivity::None;
        let admitted = p
            .resolve_for(
                &toy.c,
                &[2.0],
                &snapshot,
                DynamicDemand::Base,
                toy.c.derivatives,
                &[],
            )
            .unwrap();
        assert_eq!(admitted.method, Method::Idas);
        assert!(admitted.validate(&toy.c, &[2.0]).is_ok());
    }
    let mut p = profile(false);
    p.sensitivity = DynamicSensitivity::Forward;
    p.method = Method::Diffsol;
    assert!(p.validate(&toy.c, &[2.0]).is_ok());
}
/// A fast decay whose loose-tolerance BDF iterates overshoot zero without constraints.
#[cfg(feature = "idas")]
#[derive(Debug)]
struct Decay {
    c: Contract,
    rate: f64,
}
#[cfg(feature = "idas")]
impl Oracle for Decay {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        match f {
            Function::Rhs | Function::Output => entries([(0, 0)]),
            _ => vec![],
        }
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        _: f64,
        x: &[f64],
        _: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        let (values, slope) = match f {
            Function::Initial => (vec![1.0], 0.0),
            Function::Rhs => (vec![-self.rate * x[0]], -self.rate),
            Function::Output => (vec![x[0]], 1.0),
            _ => (vec![], 0.0),
        };
        let jacobian = d.then(|| {
            let entries: Vec<_> = if values.is_empty() || slope == 0.0 {
                vec![]
            } else {
                vec![faer::sparse::Triplet::new(0, 0, slope)]
            };
            faer::sparse::SparseColMat::try_new_from_triplets(values.len(), 1, &entries).unwrap()
        });
        Ok(Evaluation { values, jacobian })
    }
}
/// L-D5: `IDASetConstraints` keeps a non-negative state non-negative at every accepted
/// step, where the unconstrained loose-tolerance run dips below zero. The signs are the
/// contract's, derived from authored bounds (ADR-0119 Outcome 4).
#[cfg(feature = "idas")]
#[test]
fn idas_constraints_keep_positivity() {
    let mut decay = Decay {
        c: Contract {
            derivatives: pse_kernels::DerivativeOrder::First,
            quadratures: vec![],
            balances: vec![],
            identity: ContentHash::from_bytes([3; 32]),
            states: vec![id(1)],
            differential: vec![true],
            parameters: vec![],
            outputs: vec![id(4)],
            events: vec![vec![]],
            signs: vec![],
        },
        rate: 1.0e3,
    };
    let samples: Vec<f64> = (0..=400).map(|i| f64::from(i) * 0.025).collect();
    let mut p = Profile {
        method: Method::Idas,
        end: 10.0,
        samples,
        rtol: 1e-2,
        atol: vec![1e-2],
        initial_step: 1e-4,
        ..Default::default()
    };
    let free = integrate(&mut decay, &p, &[], Arc::default()).unwrap();
    assert_eq!(free.termination, Termination::Completed, "{:?}", free.error);
    let dips = free
        .samples
        .iter()
        .map(|s| s.state[0])
        .fold(f64::INFINITY, f64::min);
    decay.c.signs = vec![StateSign::NonNegative];
    let kept = integrate(&mut decay, &p, &[], Arc::default()).unwrap();
    assert_eq!(kept.termination, Termination::Completed, "{:?}", kept.error);
    let lowest = kept
        .samples
        .iter()
        .map(|s| s.state[0])
        .fold(f64::INFINITY, f64::min);
    assert!(lowest >= 0.0, "constrained minimum {lowest}");
    assert!(
        dips < 0.0,
        "the unconstrained control run stayed non-negative ({dips})"
    );
    // A sign vector is empty or has one sign per state.
    decay.c.signs = vec![StateSign::Positive, StateSign::Free];
    assert!(p.validate(&decay.c, &[]).is_err());
    // Diffsol has no sign control; the bounds' guard stays the validity authority there.
    decay.c.signs = vec![StateSign::Positive];
    p.method = Method::Diffsol;
    assert!(p.validate(&decay.c, &[]).is_ok());
}
/// L-D5: the staggered corrector reproduces the simultaneous one and the analytic
/// sensitivities.
#[cfg(feature = "idas")]
#[test]
fn idas_staggered_matches_simultaneous() {
    for dae in [false, true] {
        let mut p = profile(dae);
        p.method = Method::Idas;
        p.sensitivity = DynamicSensitivity::Forward;
        let simultaneous = run(&mut Toy::new(dae, false), &p);
        p.idas.sensitivity = SensitivityCorrector::Staggered;
        let staggered = run(&mut Toy::new(dae, false), &p);
        assert_eq!(
            staggered.termination,
            Termination::Completed,
            "{:?}",
            staggered.error
        );
        assert_eq!(staggered.samples.len(), simultaneous.samples.len());
        for (a, b) in simultaneous.samples.iter().zip(&staggered.samples) {
            let dy = (1.0 - 2.0 * a.time) * (-2.0 * a.time).exp();
            for (x, y) in a.state_sensitivities.iter().zip(&b.state_sensitivities) {
                assert!((x - y).abs() < 1e-6, "{a:?} {b:?}");
            }
            assert!((b.state_sensitivities[0] - dy).abs() < 1e-5, "{b:?}");
        }
    }
}
/// A banded chain `x_i' = -p0 (i+1) x_i + p1 x_{i-1}` with outputs over every third
/// state: sparse enough that the product path matters, with three parameters.
#[cfg(feature = "idas")]
#[derive(Debug)]
struct Chain {
    c: Contract,
    outside: bool,
}
#[cfg(feature = "idas")]
impl Chain {
    fn new(n: usize) -> Self {
        let ids = |base: u8, n: usize| -> Vec<SemanticId> {
            (0..n)
                .map(|i| {
                    let mut bytes = [base; 16];
                    bytes[..8].copy_from_slice(&(i as u64).to_le_bytes());
                    SemanticId::from_bytes(bytes)
                })
                .collect()
        };
        Self {
            c: Contract {
                derivatives: pse_kernels::DerivativeOrder::First,
                quadratures: vec![],
                balances: vec![],
                identity: ContentHash::from_bytes([5; 32]),
                states: ids(10, n),
                differential: vec![true; n],
                parameters: ids(20, 3),
                outputs: ids(30, n.div_ceil(3)),
                events: vec![vec![]],
                signs: vec![],
            },
            outside: false,
        }
    }
    fn n(&self) -> usize {
        self.c.states.len()
    }
}
#[cfg(feature = "idas")]
impl Oracle for Chain {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        let n = self.n();
        let pairs: Vec<(usize, usize)> = match f {
            Function::Rhs => (0..n)
                .flat_map(|i| {
                    let mut s = vec![(i, i), (i, n)];
                    if i > 0 {
                        s.extend([(i, i - 1), (i, n + 1)]);
                    }
                    s
                })
                .collect(),
            Function::Initial => (0..n).map(|i| (i, n + 2)).collect(),
            Function::Output => (0..n.div_ceil(3))
                .flat_map(|k| [(k, 3 * k), (k, n + 2)])
                .collect(),
            _ => vec![],
        };
        entries(pairs)
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        let n = self.n();
        let mut entries = Vec::new();
        let values = match f {
            Function::Initial => {
                entries.extend((0..n).map(|i| (i, n + 2, 1.0)));
                vec![p[2]; n]
            }
            Function::Rhs => (0..n)
                .map(|i| {
                    let k = (i + 1) as f64;
                    let mut v = -p[0] * k * x[i];
                    entries.extend([(i, i, -p[0] * k), (i, n, -k * x[i])]);
                    if i > 0 {
                        v += p[1] * x[i - 1];
                        entries.extend([(i, i - 1, p[1]), (i, n + 1, x[i - 1])]);
                    }
                    if self.outside && i == 0 {
                        // A partial the declared support does not contain.
                        entries.push((0, n - 1, 1e-3));
                    }
                    v
                })
                .collect(),
            Function::Output => (0..n.div_ceil(3))
                .map(|k| {
                    entries.extend([(k, 3 * k, 1.0), (k, n + 2, 1.0)]);
                    x[3 * k] + p[2]
                })
                .collect(),
            _ => vec![],
        };
        let jacobian = d.then(|| {
            let triplets: Vec<_> = entries
                .into_iter()
                .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                .collect();
            faer::sparse::SparseColMat::try_new_from_triplets(values.len(), n + 3, &triplets)
                .unwrap()
        });
        Ok(Evaluation { values, jacobian })
    }
}
/// F12: IDAS sensitivity residuals and output sensitivities come from faer sparse
/// products; they agree with Diffsol's forward sensitivities on a sparse chain, and a
/// partial outside the declared support is refused rather than dropped.
#[cfg(feature = "idas")]
#[test]
fn idas_sensitivity_sparse_products() {
    let n = 60;
    let p = Profile {
        samples: vec![0.0, 0.05, 0.2, 0.5],
        end: 0.5,
        rtol: 1e-9,
        atol: vec![1e-11; n],
        parameter_scales: vec![1.0; 3],
        sensitivity: DynamicSensitivity::Forward,
        ..Default::default()
    };
    let parameters = [0.4, 0.3, 1.5];
    let mut idas = p.clone();
    idas.method = Method::Idas;
    let a = integrate(&mut Chain::new(n), &idas, &parameters, Arc::default()).unwrap();
    let mut diffsol = p.clone();
    diffsol.method = Method::Diffsol;
    let b = integrate(&mut Chain::new(n), &diffsol, &parameters, Arc::default()).unwrap();
    assert_eq!(a.termination, Termination::Completed, "{:?}", a.error);
    assert_eq!(b.termination, Termination::Completed, "{:?}", b.error);
    for (x, y) in a.samples.iter().zip(&b.samples) {
        assert_eq!(x.state_sensitivities.len(), n * 3);
        assert_eq!(x.output_sensitivities.len(), n.div_ceil(3) * 3);
        for (u, v) in x.state_sensitivities.iter().zip(&y.state_sensitivities) {
            assert!(
                (u - v).abs() < 1e-6 * (1.0 + v.abs()),
                "{u} {v} at {}",
                x.time
            );
        }
        for (u, v) in x.output_sensitivities.iter().zip(&y.output_sensitivities) {
            assert!(
                (u - v).abs() < 1e-6 * (1.0 + v.abs()),
                "{u} {v} at {}",
                x.time
            );
        }
    }
    // First state: x0 = p2 exp(-p0 t), so dx0/dp0 = -t x0 and the output adds dp2 = 1.
    let last = a.samples.last().unwrap();
    let x0 = parameters[2] * (-parameters[0] * last.time).exp();
    assert!((last.state_sensitivities[0] + last.time * x0).abs() < 1e-7);
    assert!((last.output_sensitivities[2] - (x0 / parameters[2] + 1.0)).abs() < 1e-7);
    let mut outside = Chain::new(n);
    outside.outside = true;
    let mut direct = idas.clone();
    direct.sensitivity = DynamicSensitivity::None;
    let r = integrate(&mut outside, &direct, &parameters, Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::Failed);
    assert!(
        format!("{:?}", r.error).contains("outside its declared support"),
        "{:?}",
        r.error
    );
}
/// SPGMR and SPFGMR over analytic Jacobian products, with and without the Jacobi
/// preconditioner, reproduce the KLU trajectory and sensitivities.
#[cfg(feature = "idas")]
#[test]
fn idas_krylov_matches_klu() {
    use crate::solve::Preconditioner;
    for dae in [false, true] {
        let mut p = profile(dae);
        p.method = Method::Idas;
        p.sensitivity = DynamicSensitivity::Forward;
        let klu = run(&mut Toy::new(dae, false), &p);
        for preconditioner in [Preconditioner::None, Preconditioner::Jacobi] {
            for linear in [
                IdasLinear::Spgmr {
                    dimension: PositiveCount::try_new(4).unwrap(),
                    preconditioner,
                },
                IdasLinear::Spfgmr {
                    dimension: PositiveCount::try_new(4).unwrap(),
                    preconditioner,
                },
            ] {
                p.idas.linear = linear;
                let r = run(&mut Toy::new(dae, false), &p);
                assert_eq!(
                    r.termination,
                    Termination::Completed,
                    "{linear:?}: {:?}",
                    r.error
                );
                for (a, b) in r.samples.iter().zip(&klu.samples) {
                    for (x, y) in a.state.iter().zip(&b.state) {
                        assert!((x - y).abs() < 1e-6, "{linear:?}");
                    }
                    assert!((a.output_sensitivities[0] - b.output_sensitivities[0]).abs() < 1e-5);
                }
                let statistics = &r.statistics[0];
                assert!(
                    statistics["linear_iterations"].as_i64().unwrap() > 0,
                    "{statistics}"
                );
                assert!(
                    statistics["jacobian_products"].as_i64().unwrap() > 0,
                    "{statistics}"
                );
                if preconditioner == Preconditioner::Jacobi {
                    assert!(statistics["preconditioner_evaluations"].as_i64().unwrap() > 0);
                }
            }
        }
    }
    // A zero Krylov dimension cannot be decoded, so no profile carries one.
    assert!(
        serde_json::from_value::<IdasLinear>(serde_json::json!({"kind": "spgmr", "dimension": 0}))
            .is_err()
    );
}
/// `x0' = -p x0 + p` forces `x0 = 1` at steady state; the algebraic `x1 = 2 x0`.
#[cfg(feature = "idas")]
#[derive(Debug)]
struct Forced(Toy);
#[cfg(feature = "idas")]
impl Oracle for Forced {
    fn contract(&self) -> &Contract {
        self.0.contract()
    }
    fn support(&self, m: usize, f: Function) -> Vec<SupportEntry> {
        self.0.support(m, f)
    }
    fn evaluate(
        &mut self,
        m: usize,
        f: Function,
        t: f64,
        x: &[f64],
        p: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        let mut e = self.0.evaluate(m, f, t, x, p, d)?;
        if f == Function::Rhs {
            e.values[0] += p[0];
            if let Some(j) = &mut e.jacobian {
                let n = x.len();
                let triplets = [
                    faer::sparse::Triplet::new(0, 0, -p[0]),
                    faer::sparse::Triplet::new(0, n, 1.0 - x[0]),
                    faer::sparse::Triplet::new(1, 0, 2.0),
                    faer::sparse::Triplet::new(1, 1, -1.0),
                ];
                *j =
                    faer::sparse::SparseColMat::try_new_from_triplets(2, n + 1, &triplets).unwrap();
            }
        }
        Ok(e)
    }
}
/// `IDA_Y_INIT`: a steady start computes every state from zero rates; the requested
/// initial values are only the Newton guess.
#[cfg(feature = "idas")]
#[test]
fn idas_steady_start_computes_states() {
    let mut p = profile(true);
    p.method = Method::Idas;
    p.idas.initialization = IdasInitialization::SteadyStates;
    let r = integrate(
        &mut Forced(Toy::new(true, false)),
        &p,
        &[2.0],
        Arc::default(),
    )
    .unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.requested_initial, vec![2.0, 0.0]);
    assert!(
        (r.consistent_initial[0] - 1.0).abs() < 1e-8,
        "{:?}",
        r.consistent_initial
    );
    assert!((r.consistent_initial[1] - 2.0).abs() < 1e-8);
    for s in &r.samples {
        assert!((s.state[0] - 1.0).abs() < 1e-7, "{s:?}");
    }
}
