// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact pinned operator-contract controls, not a physical process qualification.
use super::*;
use std::sync::atomic::AtomicBool;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
#[derive(Debug)]
struct Toy {
    c: Contract,
    fail: Option<f64>,
}
impl Toy {
    fn new(dae: bool, event: bool) -> Self {
        let mut c = Contract {
            balances: vec![],
            identity: ContentHash::from_bytes([2; 32]),
            states: vec![id(1)],
            differential: vec![true],
            parameters: vec![id(3)],
            outputs: vec![id(4)],
            events: vec![vec![]],
        };
        if dae {
            c.states.push(id(2));
            c.differential.push(false);
        }
        if event {
            c.events = vec![
                vec![Event {
                    id: id(5),
                    terminal: false,
                    next_mode: 1,
                    tolerance: 1e-8,
                }],
                vec![],
            ];
        }
        Self { c, fail: None }
    }
}
impl Oracle for Toy {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<(usize, usize)> {
        let n = self.c.states.len();
        match f {
            Function::Initial => vec![(0, n)],
            Function::Rhs => {
                let mut s = vec![(0, 0), (0, n)];
                if n == 2 {
                    s.extend([(1, 0), (1, 1)]);
                }
                s
            }
            Function::BalanceFlux => vec![(0, 0), (0, n)],
            Function::Output => vec![(0, 0), (0, n)],
            Function::Roots => vec![],
            Function::Reset(_) => (0..n).map(|i| (i, i)).collect(),
        }
    }
    fn evaluate(
        &mut self,
        mode: usize,
        f: Function,
        t: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        if self.fail.is_some_and(|limit| t > limit) {
            return Err(contract("intentional late domain failure"));
        }
        let n = self.c.states.len();
        let (values, entries) = match f {
            Function::Initial => {
                let mut v = vec![p[0]];
                if n == 2 {
                    v.push(0.0);
                }
                (v, vec![(0, n, 1.0)])
            }
            Function::Rhs => {
                let mut v = vec![-p[0] * x[0]];
                let mut j = vec![(0, 0, -p[0]), (0, n, -x[0])];
                if n == 2 {
                    v.push(2.0 * x[0] - x[1]);
                    j.extend([(1, 0, 2.0), (1, 1, -1.0)]);
                }
                (v, j)
            }
            Function::BalanceFlux => (vec![-p[0] * x[0]], vec![(0, 0, -p[0]), (0, n, -x[0])]),
            Function::Output => (vec![x[0] + p[0]], vec![(0, 0, 1.0), (0, n, 1.0)]),
            Function::Roots => (
                if self.c.events[mode].is_empty() {
                    vec![]
                } else {
                    vec![t - 0.25]
                },
                vec![],
            ),
            Function::Reset(_) => (
                x.iter().map(|v| 2.0 * v).collect(),
                (0..n).map(|i| (i, i, 2.0)).collect(),
            ),
        };
        let jacobian = derivatives.then(|| {
            faer::sparse::SparseColMat::try_new_from_triplets(
                values.len(),
                n + 1,
                &entries
                    .into_iter()
                    .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        });
        Ok(Evaluation { values, jacobian })
    }
}
fn profile(dae: bool) -> Profile {
    Profile {
        samples: vec![0.0, 0.25, 0.5, 1.0],
        atol: vec![1e-10; if dae { 2 } else { 1 }],
        rtol: 1e-8,
        parameter_scales: vec![1.0],
        ..Default::default()
    }
}
fn run(t: &mut Toy, p: &Profile) -> Report {
    integrate(t, p, &[2.0], Arc::new(AtomicBool::new(false))).unwrap()
}
#[test]
fn smooth_forward_includes_initial_and_direct_output_parameter_terms() {
    let mut t = Toy::new(false, false);
    let mut p = profile(false);
    p.sensitivities = true;
    let r = run(&mut t, &p);
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.samples.len(), 4);
    for s in &r.samples {
        let expected = 2.0 * (-2.0 * s.time).exp();
        assert!((s.state[0] - expected).abs() < 1e-6);
        assert!(
            (s.output_sensitivities[0] - ((1.0 - 2.0 * s.time) * (-2.0 * s.time).exp() + 1.0))
                .abs()
                < 1e-5
        );
    }
    assert!(!r.statistics.is_empty());
}
#[test]
fn library_consistent_initialization_changes_only_algebraic_guesses() {
    let r = run(&mut Toy::new(true, false), &profile(true));
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.requested_initial, vec![2.0, 0.0]);
    assert!((r.consistent_initial[0] - 2.0).abs() < 1e-10);
    assert!((r.consistent_initial[1] - 4.0).abs() < 1e-8);
}
#[test]
fn coincident_root_samples_observe_reset_and_final_time_is_supported() {
    let mut p = profile(false);
    p.end = 0.25;
    p.samples = vec![0.0, 0.25];
    let r = run(&mut Toy::new(false, true), &p);
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.events.len(), 1);
    assert_eq!(r.samples.len(), 2);
    assert!((r.samples[1].state[0] - 4.0 * (-0.5f64).exp()).abs() < 1e-5);
    assert_eq!(r.events[0].after.as_ref().unwrap(), &r.samples[1].state);
}
#[test]
fn final_scheduled_change_is_reinitialized_and_observed() {
    let mut p = profile(false);
    p.changes = vec![InputChange {
        time: 1.0,
        parameters: vec![3.0],
    }];
    let r = run(&mut Toy::new(false, false), &p);
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.samples.len(), 4);
    assert!((r.samples[3].outputs[0] - r.samples[3].state[0] - 3.0).abs() < 1e-10);
    assert!(r.events[0].after.is_some());
}
#[test]
fn typed_callback_failure_retains_only_completed_prefix() {
    let mut t = Toy::new(false, false);
    t.fail = Some(0.4);
    let r = run(&mut t, &profile(false));
    assert_eq!(r.termination, Termination::Failed);
    assert!(r.error.unwrap().to_string().contains("intentional"));
    assert!(!r.samples.is_empty());
    assert!(r.samples.iter().all(|s| s.time <= 0.4));
    assert!(!r.statistics.is_empty());
}
#[test]
fn cancellation_and_step_budget_are_distinct() {
    let mut t = Toy::new(false, false);
    let p = profile(false);
    let r = integrate(&mut t, &p, &[2.0], Arc::new(AtomicBool::new(true))).unwrap();
    assert_eq!(r.termination, Termination::Cancelled);
    assert!(r.samples.is_empty());
    let r = run(&mut t, &Profile { max_steps: 1, ..p });
    assert_eq!(r.termination, Termination::StepLimit);
}
#[test]
fn unsupported_recovery_and_invalid_native_controls_are_refused() {
    let t = Toy::new(false, true);
    let mut p = profile(false);
    p.sensitivities = true;
    p.method = Method::Diffsol;
    p.trial_failures = TrialPolicy::Recoverable;
    assert!(p.validate(&t.c, &[2.0]).is_err());
    p.trial_failures = TrialPolicy::Terminal;
    p.sensitivities = false;
    Arc::get_mut(&mut p.native).unwrap().min_timestep = f64::NAN;
    assert!(p.validate(&t.c, &[2.0]).is_err());
}
#[test]
fn unsupported_hybrid_sensitivity_profiles_fail_before_native_entry() {
    let mut t = Toy::new(false, true);
    let mut p = profile(false);
    p.sensitivities = true;
    t.c.events[0][0].terminal = true;
    assert!(
        p.validate(&t.c, &[2.0])
            .unwrap_err()
            .to_string()
            .contains("terminal-event")
    );
    t.c.events[0][0].terminal = false;
    p.method = Method::Idas;
    assert!(p.validate(&t.c, &[2.0]).is_err());
    t.c.events = vec![vec![]];
    Arc::get_mut(&mut p.native).unwrap().min_timestep *= 2.0;
    assert!(
        p.validate(&t.c, &[2.0])
            .unwrap_err()
            .to_string()
            .contains("Diffsol-specific")
    );
}
#[test]
fn complete_native_options_round_trip_and_affect_identity() {
    let p = profile(false);
    let text = serde_json::to_string(&p).unwrap();
    let mut round: Profile = serde_json::from_str(&text).unwrap();
    assert_eq!(settings_identity(&p), settings_identity(&round));
    Arc::get_mut(&mut round.native)
        .unwrap()
        .pi_control_proportional = 0.4;
    assert_ne!(settings_identity(&p), settings_identity(&round));
}
#[test]
fn algebraic_initial_sensitivities_follow_native_consistency() {
    let mut p = profile(true);
    p.sensitivities = true;
    let r = run(&mut Toy::new(true, false), &p);
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        assert!((s.state_sensitivities[1] - 2.0 * s.state_sensitivities[0]).abs() < 1e-6);
    }
    assert!(!r.progress.is_empty());
}

#[test]
fn integrated_balances_carry_segments_and_refuse_undeclared_jumps() {
    let mut oracle = Toy::new(false, false);
    oracle.c.balances = vec![Balance {
        id: id(8),
        state: 0,
        scale: 1.0,
        tolerance: 1e-6,
        impulses: Default::default(),
    }];
    let mut profile = Profile {
        samples: vec![0.0, 0.2, 0.4, 0.7, 1.0],
        parameter_scales: vec![1.0],
        rtol: 1e-9,
        out_rtol: Some(1e-9),
        out_atol: vec![1e-10],
        changes: vec![InputChange {
            time: 0.4,
            parameters: vec![2.0],
        }],
        ..Default::default()
    };
    let r = integrate(
        &mut oracle,
        &profile,
        &[1.0],
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        assert!((s.state[0] - r.consistent_initial[0] - s.balance_integrals[0]).abs() < 1e-6);
    }
    let mut event = Toy::new(false, true);
    event.c.balances = oracle.c.balances.clone();
    profile.changes.clear();
    let r = integrate(
        &mut event,
        &profile,
        &[1.0],
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(r.termination, Termination::Failed);
    assert!(r.error.unwrap().to_string().contains("impulse"));
    let impulse = (-0.25f64).exp();
    event.c.balances[0].impulses.insert(id(5), impulse);
    let r = integrate(
        &mut event,
        &profile,
        &[1.0],
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        let jump = if s.time >= 0.25 { impulse } else { 0.0 };
        assert!(
            (s.state[0] - r.consistent_initial[0] - s.balance_integrals[0] - jump).abs() < 1e-6
        );
    }
    profile.out_rtol = None;
    assert!(profile.validate(&oracle.c, &[1.0]).is_err());
}

#[cfg(feature = "idas")]
#[test]
fn idas_consistent_dae_and_analytic_forward_sensitivities() {
    for dae in [false, true] {
        let mut toy = Toy::new(dae, false);
        let mut p = profile(dae);
        p.method = Method::Idas;
        p.sensitivities = true;
        let r = run(&mut toy, &p);
        assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
        assert_eq!(r.samples.len(), p.samples.len());
        for s in &r.samples {
            let y = 2.0 * (-2.0 * s.time).exp();
            let dy = (1.0 - 2.0 * s.time) * (-2.0 * s.time).exp();
            assert!((s.state[0] - y).abs() < 1e-6, "{s:?}");
            assert!((s.output_sensitivities[0] - dy - 1.0).abs() < 1e-5, "{s:?}");
            if dae {
                assert!((s.state[1] - 2.0 * y).abs() < 1e-6);
                assert!((s.state_sensitivities[1] - 2.0 * dy).abs() < 1e-5);
            }
        }
    }
}

#[test]
fn scheduled_changes_preserve_history_sensitivity_and_replace_direct_parameter_terms() {
    let mut p = profile(false);
    p.sensitivities = true;
    p.changes = vec![InputChange {
        time: 0.5,
        parameters: vec![3.0],
    }];
    let r = run(&mut Toy::new(false, false), &p);
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    for s in &r.samples {
        let (y, dy, direct) = if s.time < 0.5 {
            (
                2.0 * (-2.0 * s.time).exp(),
                (1.0 - 2.0 * s.time) * (-2.0 * s.time).exp(),
                1.0,
            )
        } else {
            (2.0 * (-1.0 - 3.0 * (s.time - 0.5)).exp(), 0.0, 0.0)
        };
        assert!((s.state[0] - y).abs() < 1e-6, "{s:?}");
        assert!(
            (s.output_sensitivities[0] - dy - direct).abs() < 1e-5,
            "{s:?}"
        );
    }
}
#[derive(Debug)]
struct ResetToy(Toy);
impl Oracle for ResetToy {
    fn contract(&self) -> &Contract {
        self.0.contract()
    }
    fn support(&self, m: usize, f: Function) -> Vec<(usize, usize)> {
        match f {
            Function::Roots => {
                if m == 0 {
                    vec![(0, 0)]
                } else {
                    vec![]
                }
            }
            Function::Reset(_) => vec![],
            _ => self.0.support(m, f),
        }
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
        if matches!(f, Function::Roots | Function::Reset(_)) {
            let values = if f == Function::Roots {
                if m == 0 { vec![x[0] - 1.0] } else { vec![] }
            } else {
                (0..x.len())
                    .map(|i| if i == 0 { 3.0 } else { 0.0 })
                    .collect()
            };
            let pairs = if f == Function::Roots && m == 0 {
                vec![faer::sparse::Triplet::new(0, 0, 1.0)]
            } else {
                vec![]
            };
            let jacobian = d.then(|| {
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    x.len() + p.len(),
                    &pairs,
                )
                .unwrap()
            });
            Ok(Evaluation { values, jacobian })
        } else {
            self.0.evaluate(m, f, t, x, p, d)
        }
    }
}
#[test]
fn state_triggered_reset_sensitivity_includes_moving_event_and_dae_consistency() {
    for dae in [false, true] {
        let mut p = profile(dae);
        p.sensitivities = true;
        p.samples = vec![0.0, 1.0];
        let mut toy = ResetToy(Toy::new(dae, true));
        let r = integrate(&mut toy, &p, &[2.0], Arc::default()).unwrap();
        assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
        let last = r.samples.last().unwrap();
        assert!(
            (last.state[0] - 6.0 * (-2.0f64).exp()).abs() < 1e-5,
            "{last:?}"
        );
        assert!(
            (last.output_sensitivities[0] - (1.0 - 3.0 * (-2.0f64).exp())).abs() < 1e-4,
            "{last:?}"
        );
        if dae {
            assert!(
                (last.state_sensitivities[1] + 6.0 * (-2.0f64).exp()).abs() < 1e-4,
                "{last:?}"
            );
        }
    }
}
#[cfg(feature = "idas")]
#[derive(Debug)]
struct TrialToy {
    toy: Toy,
    rejected: bool,
}
#[cfg(feature = "idas")]
impl Oracle for TrialToy {
    fn contract(&self) -> &Contract {
        self.toy.contract()
    }
    fn support(&self, m: usize, f: Function) -> Vec<(usize, usize)> {
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
        if f == Function::Rhs && t > 0.01 && !self.rejected {
            self.rejected = true;
            return Err(pse_math::MathError::Domain {
                source_id: id(9),
                requirement: "intentional recoverable trial",
            }
            .into());
        }
        self.toy.evaluate(m, f, t, x, p, d)
    }
}
#[cfg(feature = "idas")]
#[test]
fn idas_recovers_typed_trial_and_terminal_policy_stops() {
    for policy in [TrialPolicy::Recoverable, TrialPolicy::Terminal] {
        let mut p = profile(false);
        p.method = Method::Idas;
        p.trial_failures = policy;
        let mut toy = TrialToy {
            toy: Toy::new(false, false),
            rejected: false,
        };
        let r = integrate(&mut toy, &p, &[2.0], Arc::default()).unwrap();
        assert!(toy.rejected);
        assert_eq!(
            r.termination,
            if policy == TrialPolicy::Recoverable {
                Termination::Completed
            } else {
                Termination::Failed
            },
            "{:?}",
            r.error
        );
        assert!(
            r.progress
                .iter()
                .any(|e| e.phase == "idas.evaluation.failure")
        );
    }
}
