// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Small process fixtures for the ADR-0110 routes: a draining vessel for the Diffsol
//! schemes and linear solvers (Y2), and a PI-controlled tank with piecewise inlet
//! pressure for the IDAS scheduled-input route (Y1, S09). The tank is comparable to the
//! IDAES PETSc PID example, not a reimplementation of it: an ideal-gas holdup replaces the
//! IAPWS-95 steam tank, and the inlet ramp is three scheduled input segments.
use super::*;

fn ids(base: u8, n: usize) -> Vec<SemanticId> {
    (0..n)
        .map(|i| {
            let mut bytes = [base; 16];
            bytes[0] = i as u8;
            SemanticId::from_bytes(bytes)
        })
        .collect()
}
fn jacobian(
    rows: usize,
    columns: usize,
    entries: &[(usize, usize, f64)],
) -> faer::sparse::SparseColMat<usize, f64> {
    let triplets: Vec<_> = entries
        .iter()
        .map(|&(r, c, v)| faer::sparse::Triplet::new(r, c, v))
        .collect();
    faer::sparse::SparseColMat::try_new_from_triplets(rows, columns, &triplets).unwrap()
}
fn domain(requirement: &'static str) -> ProblemError {
    pse_math::MathError::Domain {
        source_id: SemanticId::from_bytes([77; 16]),
        requirement,
    }
    .into()
}

/// A vessel of area `A = 2` filled at `Fin = 1` and drained through `Fout = k·√(V/A)`,
/// as an ODE in the volume or as a semi-explicit DAE with the outflow algebraic.
#[derive(Debug)]
struct Vessel {
    c: Contract,
}
const AREA: f64 = 2.0;
const INFLOW: f64 = 1.0;
impl Vessel {
    fn new(dae: bool) -> Self {
        let n = if dae { 2 } else { 1 };
        Self {
            c: Contract {
                quadratures: vec![],
                balances: vec![],
                identity: ContentHash::from_bytes([40; 32]),
                states: ids(41, n),
                differential: (0..n).map(|i| i == 0).collect(),
                parameters: ids(42, 1),
                outputs: ids(43, 1),
                events: vec![vec![]],
            },
        }
    }
    fn dae(&self) -> bool {
        self.c.states.len() == 2
    }
}
impl Oracle for Vessel {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match (f, self.dae()) {
            (Function::Rhs, false) => vec![(0, 0), (0, 1)],
            (Function::Rhs, true) => vec![(0, 1), (1, 0), (1, 1), (1, 2)],
            (Function::Output, _) => vec![(0, 0)],
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
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        let n = self.c.states.len();
        let (values, entries) = match f {
            Function::Initial => {
                let mut v = vec![1.0];
                if self.dae() {
                    v.push(0.0);
                }
                (v, vec![])
            }
            Function::Rhs => {
                if x[0] <= 0.0 {
                    return Err(domain("positive vessel volume"));
                }
                let s = (x[0] / AREA).sqrt();
                let slope = -p[0] / (2.0 * AREA * s);
                if self.dae() {
                    (
                        vec![INFLOW - x[1], x[1] - p[0] * s],
                        vec![(0, 1, -1.0), (1, 1, 1.0), (1, 0, slope), (1, 2, -s)],
                    )
                } else {
                    (vec![INFLOW - p[0] * s], vec![(0, 0, slope), (0, 1, -s)])
                }
            }
            Function::Output => (vec![x[0] / AREA], vec![(0, 0, 1.0 / AREA)]),
            _ => (vec![], vec![]),
        };
        let jacobian = d.then(|| jacobian(values.len(), n + 1, &entries));
        Ok(Evaluation { values, jacobian })
    }
}
fn vessel_profile(dae: bool) -> Profile {
    Profile {
        method: Method::Diffsol,
        end: 20.0,
        samples: (0..=10).map(|i| f64::from(i) * 2.0).collect(),
        rtol: 1e-8,
        atol: vec![1e-10; if dae { 2 } else { 1 }],
        initial_step: 1e-4,
        parameter_scales: vec![1.0],
        sensitivity: DynamicSensitivity::Forward,
        ..Default::default()
    }
}
fn drain(dae: bool, p: &Profile) -> Report {
    let r = integrate(&mut Vessel::new(dae), p, &[0.5], Arc::default()).unwrap();
    assert_eq!(
        r.termination,
        Termination::Completed,
        "{:?} {:?}: {:?}",
        p.diffsol,
        dae,
        r.error
    );
    r
}
fn close(a: &Report, b: &Report, relative: f64) {
    assert_eq!(a.samples.len(), b.samples.len());
    for (x, y) in a.samples.iter().zip(&b.samples) {
        for (u, v) in x
            .state
            .iter()
            .chain(&x.outputs)
            .chain(&x.output_sensitivities)
            .zip(
                y.state
                    .iter()
                    .chain(&y.outputs)
                    .chain(&y.output_sensitivities),
            )
        {
            assert!(
                (u - v).abs() <= relative * (1.0 + v.abs()),
                "{u} vs {v} at t={}",
                x.time
            );
        }
    }
}
/// L-D5: the SDIRK schemes (TR-BDF2, ESDIRK34) and, for the mass-free ODE, the explicit
/// Tsit45 scheme reproduce the BDF trajectory and forward sensitivities of the vessel.
#[test]
fn sdirk_matches_bdf_on_vessel() {
    for dae in [false, true] {
        let bdf = drain(dae, &vessel_profile(dae));
        // The level rises from 0.5 toward its steady value (Fin/k)² = 4.
        let last = bdf.samples.last().unwrap();
        assert!(last.outputs[0] > 3.0 && last.outputs[0] < 4.0, "{last:?}");
        let mut methods = vec![DiffsolMethod::TrBdf2, DiffsolMethod::Esdirk34];
        if !dae {
            methods.push(DiffsolMethod::Tsit45);
        }
        for method in methods {
            let mut p = vessel_profile(dae);
            p.diffsol.method = method;
            let r = drain(dae, &p);
            close(&r, &bdf, 1e-5);
            // Each segment records the scheme that produced it.
            assert_eq!(
                r.statistics[0]["diffsol"]["method"],
                serde_json::to_value(method).unwrap()
            );
        }
    }
}
/// ADR-0110 item 2: explicit Tsit45 is refused with a mass matrix before native work.
#[test]
fn tsit45_refuses_mass_matrix() {
    let mut p = vessel_profile(true);
    p.diffsol.method = DiffsolMethod::Tsit45;
    let vessel = Vessel::new(true);
    let Err(ProblemError::Unsupported(reason)) = p.validate(&vessel.c, &[0.5]) else {
        panic!("tsit45 admitted a mass matrix");
    };
    assert!(reason.contains("mass-free"), "{reason}");
    assert!(integrate(&mut Vessel::new(true), &p, &[0.5], Arc::default()).is_err());
    // The mass-free ODE is admitted; an explicit scheme has no Newton linear solver.
    let mut p = vessel_profile(false);
    p.diffsol.method = DiffsolMethod::Tsit45;
    assert!(p.validate(&Vessel::new(false).c, &[0.5]).is_ok());
    p.diffsol.linear = DiffsolLinear::Klu;
    assert!(p.validate(&Vessel::new(false).c, &[0.5]).is_err());
}
/// L-D5: SuiteSparse KLU (Diffsol `suitesparse`) reproduces the faer sparse LU
/// trajectories and sensitivities for the implicit schemes.
#[test]
fn diffsol_klu_matches_faer_lu() {
    for dae in [false, true] {
        for method in [DiffsolMethod::Bdf, DiffsolMethod::TrBdf2] {
            let mut p = vessel_profile(dae);
            p.diffsol.method = method;
            let lu = drain(dae, &p);
            p.diffsol.linear = DiffsolLinear::Klu;
            let klu = drain(dae, &p);
            close(&klu, &lu, 1e-8);
            assert_eq!(
                klu.statistics[0]["diffsol"]["linear"],
                serde_json::json!("klu")
            );
        }
    }
}

/// A semi-explicit DAE whose algebraic row degenerates at `t = 1`: `x' = −z` and
/// `0 = w(t)·(z − x)` with `w = max(0, 1 − t)`. Past `t = 1` the algebraic row and its
/// partials vanish, so every Newton matrix factored at an accepted state is singular.
#[derive(Debug)]
struct Degenerate {
    c: Contract,
}
impl Degenerate {
    fn new() -> Self {
        Self {
            c: Contract {
                quadratures: vec![],
                balances: vec![],
                identity: ContentHash::from_bytes([44; 32]),
                states: ids(45, 2),
                differential: vec![true, false],
                parameters: vec![],
                outputs: ids(46, 1),
                events: vec![vec![]],
            },
        }
    }
}
impl Oracle for Degenerate {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match f {
            Function::Rhs => vec![(0, 1), (1, 0), (1, 1)],
            Function::Output => vec![(0, 0)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        t: f64,
        x: &[f64],
        _: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        let w = (1.0 - t).max(0.0);
        let (values, entries) = match f {
            Function::Initial => (vec![1.0, 1.0], vec![]),
            Function::Rhs => (
                vec![-x[1], w * (x[1] - x[0])],
                vec![(0, 1, -1.0), (1, 0, -w), (1, 1, w)],
            ),
            Function::Output => (vec![x[0]], vec![(0, 0, 1.0)]),
            _ => (vec![], vec![]),
        };
        let jacobian = d.then(|| jacobian(values.len(), 2, &entries));
        Ok(Evaluation { values, jacobian })
    }
}
/// I9: a singular Newton factorization is an error of the pse-owned linear solver, which
/// Diffsol answers by reducing the step; the final failure is a typed numerical
/// termination with the completed samples kept, for faer LU and KLU alike.
#[test]
fn diffsol_singular_factorization_is_typed_numerical() {
    for linear in [DiffsolLinear::FaerLu, DiffsolLinear::Klu] {
        let p = Profile {
            method: Method::Diffsol,
            end: 10.0,
            samples: vec![0.0, 0.5, 10.0],
            rtol: 1e-6,
            atol: vec![1e-8; 2],
            diffsol: DiffsolSettings {
                linear,
                ..Default::default()
            },
            // Past the degeneration the stale Newton matrix still converges. Refresh the
            // Jacobian at every step-size change, and change the step at every order
            // check, so an accepted state past `t = 1` is factored.
            native: Arc::new(diffsol::OdeSolverOptions {
                update_jacobian_after_steps: 1,
                update_rhs_jacobian_after_steps: 1,
                min_timestep_growth: Some(1.0),
                max_timestep_shrink: Some(1.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let r = integrate(&mut Degenerate::new(), &p, &[], Arc::default()).unwrap();
        assert_eq!(r.termination, Termination::Failed, "{linear:?}: {:?}", r.error);
        // Diffsol's own step recovery ends it, not a nonfinite value reaching the oracle.
        assert!(
            matches!(&r.error, Some(ProblemError::Numerical { detail, .. })
                if detail.contains("nonlinear solver failures") || detail.contains("Step size is too small")),
            "{linear:?}: {:?}",
            r.error
        );
        // The samples before the degeneration are kept: x = exp(−t).
        assert_eq!(r.samples.len(), 2, "{linear:?}");
        assert!((r.samples[1].outputs[0] - (-0.5_f64).exp()).abs() < 1e-4);
        assert!(r.completed_time >= 1.0, "{linear:?}: {}", r.completed_time);
        let failures = r.statistics[0]["number_of_nonlinear_solver_fails"]
            .as_u64()
            .unwrap();
        assert!(failures > 0, "{linear:?}: {:?}", r.statistics);
    }
}

/// PI control of an ideal-gas tank between two valves, `F = Cv·u·√(Pᵢ² − Pₒ²)`, with a
/// smoothly bounded valve opening. States are normalized: holdup/100 mol, integral
/// error/1e5 Pa·s, then the algebraic pressure/1e5 Pa, flows/100 mol/s and opening.
/// Parameters: inlet pressure offset and slope (the piecewise input), setpoint, gains
/// and the opening bias.
#[cfg(feature = "idas")]
#[derive(Debug)]
struct Tank {
    c: Contract,
}
#[cfg(feature = "idas")]
mod tank {
    pub(super) const GAS: f64 = 8.314 * 500.0 / 2.0;
    pub(super) const HOLDUP: f64 = 100.0 * GAS / 1e5;
    pub(super) const VALVE: f64 = 0.001 / 100.0;
    pub(super) const OUTLET: f64 = 101_325.0;
    pub(super) const SMOOTH: f64 = 1e-3;
    /// `min(1, max(0, v))`, smoothed, and its derivative.
    pub(super) fn clip(v: f64) -> (f64, f64) {
        let r = (v * v + SMOOTH * SMOOTH).sqrt();
        let upper = 0.5 * (v + r);
        let d_upper = 0.5 * (1.0 + v / r);
        let q = ((1.0 - upper).powi(2) + SMOOTH * SMOOTH).sqrt();
        (
            0.5 * (1.0 + upper - q),
            0.5 * (1.0 + (1.0 - upper) / q) * d_upper,
        )
    }
}
#[cfg(feature = "idas")]
impl Tank {
    fn new() -> Self {
        Self {
            c: Contract {
                quadratures: vec![],
                balances: vec![],
                identity: ContentHash::from_bytes([50; 32]),
                states: ids(51, 6),
                differential: vec![true, true, false, false, false, false],
                parameters: ids(52, 6),
                outputs: ids(53, 4),
                events: vec![vec![]],
            },
        }
    }
}
#[cfg(feature = "idas")]
impl Oracle for Tank {
    fn contract(&self) -> &Contract {
        &self.c
    }
    fn support(&self, _: usize, f: Function) -> Vec<SupportEntry> {
        entries(match f {
            Function::Rhs => vec![
                (0, 3),
                (0, 4),
                (1, 2),
                (1, 8),
                (2, 2),
                (2, 0),
                (3, 3),
                (3, 5),
                (3, 2),
                (3, 6),
                (3, 7),
                (4, 4),
                (4, 2),
                (5, 5),
                (5, 2),
                (5, 1),
                (5, 11),
                (5, 9),
                (5, 8),
                (5, 10),
            ],
            Function::Output => vec![(0, 2), (1, 6), (1, 7), (2, 5), (3, 3)],
            _ => vec![],
        })
    }
    fn evaluate(
        &mut self,
        _: usize,
        f: Function,
        t: f64,
        z: &[f64],
        p: &[f64],
        d: bool,
    ) -> Result<Evaluation, ProblemError> {
        use tank::*;
        let [base, slope, setpoint, kp, ki, bias] = [p[0], p[1], p[2], p[3], p[4], p[5]];
        let inlet = base + slope * t;
        let (values, entries) = match f {
            Function::Initial => (vec![3.0 / HOLDUP, 0.7, 3.0, 2.8, 2.8, 0.7], vec![]),
            Function::Rhs => {
                let pressure = 1e5 * z[2];
                let s1 = inlet * inlet - pressure * pressure;
                let s2 = pressure * pressure - OUTLET * OUTLET;
                if s1 <= 0.0 || s2 <= 0.0 {
                    return Err(domain("valve pressure drop"));
                }
                let (s1, s2) = (s1.sqrt(), s2.sqrt());
                let (opening, g) = clip(bias + kp * (setpoint - pressure) + ki * 1e5 * z[1]);
                (
                    vec![
                        z[3] - z[4],
                        setpoint / 1e5 - z[2],
                        z[2] - HOLDUP * z[0],
                        z[3] - VALVE * z[5] * s1,
                        z[4] - VALVE * s2,
                        z[5] - opening,
                    ],
                    vec![
                        (0, 3, 1.0),
                        (0, 4, -1.0),
                        (1, 2, -1.0),
                        (1, 8, 1e-5),
                        (2, 2, 1.0),
                        (2, 0, -HOLDUP),
                        (3, 3, 1.0),
                        (3, 5, -VALVE * s1),
                        (3, 2, VALVE * z[5] * pressure * 1e5 / s1),
                        (3, 6, -VALVE * z[5] * inlet / s1),
                        (3, 7, -VALVE * z[5] * inlet * t / s1),
                        (4, 4, 1.0),
                        (4, 2, -VALVE * pressure * 1e5 / s2),
                        (5, 5, 1.0),
                        (5, 2, g * kp * 1e5),
                        (5, 1, -g * ki * 1e5),
                        (5, 11, -g),
                        (5, 9, -g * (setpoint - pressure)),
                        (5, 8, -g * kp),
                        (5, 10, -g * 1e5 * z[1]),
                    ],
                )
            }
            Function::Output => (
                vec![1e5 * z[2], inlet, z[5], 100.0 * z[3]],
                vec![
                    (0, 2, 1e5),
                    (1, 6, 1.0),
                    (1, 7, t),
                    (2, 5, 1.0),
                    (3, 3, 100.0),
                ],
            ),
            _ => (vec![], vec![]),
        };
        let jacobian = d.then(|| jacobian(values.len(), 12, &entries));
        Ok(Evaluation { values, jacobian })
    }
}
/// S09 (L-D1), comparable to the IDAES PETSc PID example: a steady start (`IDA_Y_INIT`),
/// an inlet pressure ramp from 5e5 Pa to 6e5 Pa between 10 s and 12 s as three scheduled
/// input segments, recoverable valve-domain trials, and the PI controller returning the
/// tank to its 3e5 Pa setpoint. IDAES asserts the inlet pressure at 5 s and 20 s and the
/// tank pressure at 9 s and 22 s; this fixture asserts the same points, plus the
/// analytic steady valve openings before and after the ramp.
#[cfg(feature = "idas")]
#[test]
fn idas_pid_piecewise_inputs_match_petsc_example() {
    // The inlet pressure offset and slope are scheduled inputs with one value per
    // interval; setpoint, gains and bias are static (I6).
    let mut values = vec![3e5, 1e-6, 1e-5, 0.0];
    values.extend([5e5, 0.0, 6e5]);
    values.extend([0.0, 5e4, 0.0]);
    let p = Profile {
        method: Method::Auto,
        trial_failures: TrialPolicy::Recoverable,
        start: 0.0,
        end: 24.0,
        samples: (0..=24).map(f64::from).collect(),
        rtol: 1e-8,
        atol: vec![1e-10; 6],
        initial_step: 1e-3,
        parameter_scales: vec![1e5, 1e4, 1e5, 1e-6, 1e-5, 1.0],
        schedule: [0, 1]
            .map(|parameter| ScheduledInput {
                parameter,
                times: vec![10.0, 12.0],
            })
            .to_vec(),
        idas: IdasSettings {
            initialization: IdasInitialization::SteadyStates,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(p.resolved_method().unwrap(), Method::Idas);
    let r = integrate(&mut Tank::new(), &p, &values, Arc::default()).unwrap();
    assert_eq!(r.termination, Termination::Completed, "{:?}", r.error);
    assert_eq!(r.samples.len(), 25);
    let at = |t: usize| &r.samples[t].outputs;
    // Steady valve opening that passes the outlet flow at the setpoint.
    let steady = |inlet: f64| {
        let outlet = 0.001 * (3e5f64.powi(2) - tank::OUTLET.powi(2)).sqrt();
        outlet / (0.001 * (inlet * inlet - 9e10).sqrt())
    };
    // The steady start holds the setpoint exactly until the ramp.
    assert!((r.consistent_initial[2] - 3.0).abs() < 1e-9);
    assert!((at(5)[1] - 5e5).abs() < 1e-6);
    assert!((at(9)[0] - 3e5).abs() < 1e-6 * 3e5, "{:?}", at(9));
    assert!((at(9)[2] - steady(5e5)).abs() < 1e-6, "{:?}", at(9));
    // The ramp is exact at its midpoint and complete afterwards.
    assert!((at(11)[1] - 5.5e5).abs() < 1e-6);
    assert!((at(20)[1] - 6e5).abs() < 1e-6);
    // The disturbance lifts the tank pressure; the controller removes it.
    assert!(r.samples[10..=14].iter().any(|s| s.outputs[0] > 3e5 + 1e3));
    assert!((at(22)[0] - 3e5).abs() < 1e-6 * 3e5, "{:?}", at(22));
    assert!((at(24)[2] - steady(6e5)).abs() < 1e-6, "{:?}", at(24));
    // Both scheduled changes restarted IDAS with consistent algebraic states.
    assert_eq!(r.events.len(), 2);
    for (event, time) in r.events.iter().zip([10.0, 12.0]) {
        assert_eq!(event.time, time);
        let after = event.after.as_ref().unwrap();
        assert_eq!(after[..2], event.before[..2]);
        assert!((after[2] - tank::HOLDUP * after[0]).abs() < 1e-9);
    }
    assert_eq!(r.statistics.len(), 3);
}
