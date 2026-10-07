// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent nonlinear inventory, original flux and state-dependent event controls.
use super::*;

/// The inventory is nonlinear and includes the consistently initialized algebraic state.
/// Its flux is the independently stated physical source, never the solved rate callback.
#[derive(Debug)]
struct Conserved {
    toy: Toy,
    source_factor: f64,
    transfer_factor: f64,
    changing_definition: bool,
    input_sensitive_inventory: bool,
}
impl Conserved {
    fn new(event: bool) -> Self {
        let mut toy = Toy::new(true, event);
        toy.c.quadratures = vec![id(80)];
        toy.c.balances = vec![Balance {
            id: id(81),
            inventory: id(82),
            flux: id(80),
            tolerance: 1e-6,
            transfers: BTreeSet::new(),
        }];
        Self {
            toy,
            source_factor: 1.0,
            transfer_factor: 1.0,
            changing_definition: false,
            input_sensitive_inventory: false,
        }
    }
    fn coefficients(&self, mode: usize) -> (f64, f64) {
        if mode == 1 && self.changing_definition {
            (0.25, 0.5)
        } else {
            (1.0, 1.0)
        }
    }
}
impl Oracle for Conserved {
    fn contract(&self) -> &Contract {
        self.toy.contract()
    }
    fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
        match function {
            Function::Inventory => entries([(0, 0), (0, 1), (0, 2)]),
            Function::QuadratureFlux => entries([(0, 0), (0, 2)]),
            Function::Transfer(_) => entries([(0, 0), (0, 1)]),
            _ => self.toy.support(mode, function),
        }
    }
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        let (square, linear) = self.coefficients(mode);
        let (values, partials) = match function {
            Function::Inventory => (
                vec![
                    square * x[0] * x[0]
                        + linear * x[1]
                        + if self.input_sensitive_inventory {
                            p[0]
                        } else {
                            0.0
                        },
                ],
                vec![
                    (0, 0, 2.0 * square * x[0]),
                    (0, 1, linear),
                    (0, 2, f64::from(self.input_sensitive_inventory)),
                ],
            ),
            Function::QuadratureFlux => {
                let factor = self.source_factor;
                (
                    vec![
                        factor * (-2.0 * square * p[0] * x[0] * x[0] - 2.0 * linear * p[0] * x[0]),
                    ],
                    vec![
                        (
                            0,
                            0,
                            factor * (-4.0 * square * p[0] * x[0] - 2.0 * linear * p[0]),
                        ),
                        (
                            0,
                            2,
                            factor * (-2.0 * square * x[0] * x[0] - 2.0 * linear * x[0]),
                        ),
                    ],
                )
            }
            Function::Transfer(_) => (
                vec![self.transfer_factor * (3.0 * x[0] * x[0] + x[1])],
                vec![
                    (0, 0, self.transfer_factor * 6.0 * x[0]),
                    (0, 1, self.transfer_factor),
                ],
            ),
            _ => return self.toy.evaluate(mode, function, time, x, p, derivatives),
        };
        let jacobian = derivatives.then(|| {
            faer::sparse::SparseColMat::try_new_from_triplets(
                values.len(),
                x.len() + p.len(),
                &partials
                    .into_iter()
                    .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        });
        Ok(Evaluation { values, jacobian })
    }
}
fn methods() -> Vec<Method> {
    let methods = vec![
        Method::Diffsol,
        #[cfg(feature = "idas")]
        Method::Idas,
    ];
    methods
}
fn policy(method: Method) -> Profile {
    Profile {
        method,
        samples: vec![0.0, 0.2, 0.25, 0.5, 1.0],
        atol: vec![1e-11; 2],
        rtol: 1e-9,
        out_rtol: Some(1e-9),
        out_atol: vec![1e-11],
        parameter_scales: vec![1.0],
        ..Default::default()
    }
}
fn execute(oracle: &mut Conserved, p: &Profile, parameters: &[f64]) -> Report {
    integrate(oracle, p, parameters, Arc::default()).unwrap()
}
fn closes(report: &Report) {
    assert!(!report.conservation.is_empty());
    for point in &report.conservation {
        assert!(point.defects[0].abs() < 1e-6, "{point:?}");
    }
}
#[test]
fn nonlinear_inventory_uses_consistent_baseline_and_original_flux_across_segments() {
    for method in methods() {
        let mut oracle = Conserved::new(false);
        let mut p = policy(method);
        p.schedule = vec![ScheduledInput {
            parameter: 0,
            times: vec![0.5],
        }];
        let report = execute(&mut oracle, &p, &[1.0, 2.0]);
        assert_eq!(
            report.termination,
            Termination::Completed,
            "{:?}",
            report.error
        );
        assert!(
            (report.conservation[0].inventories[0] - 3.0).abs() < 1e-8,
            "inventory uses consistent algebraic x1=2*x0 rather than initial guess zero"
        );
        closes(&report);
        assert_eq!(report.conservation.last().unwrap().time, 1.0);
        assert!(report.conservation.iter().all(|p| p.transfers == vec![0.0]));
    }
}
#[test]
fn zero_original_flux_exposes_continuous_inventory_drift_without_inventing_permission() {
    for method in methods() {
        let mut oracle = Conserved::new(false);
        oracle.source_factor = 0.0;
        let report = execute(&mut oracle, &policy(method), &[1.0]);
        assert_eq!(
            report.termination,
            Termination::Completed,
            "{:?}",
            report.error
        );
        let last = report.conservation.last().unwrap();
        assert_eq!(last.integrals, vec![0.0]);
        assert!(last.defects[0].abs() > 1.0, "{last:?}");
    }
}
#[test]
fn nonlinear_reset_requires_matching_evaluated_state_dependent_transfer() {
    for method in methods() {
        let mut unauthorized = Conserved::new(true);
        unauthorized.source_factor = 0.0;
        let report = execute(&mut unauthorized, &policy(method), &[1.0]);
        assert_eq!(report.termination, Termination::Failed);
        assert!(
            report
                .error
                .unwrap()
                .to_string()
                .contains("permitted event transfer")
        );
        assert!(report.conservation.last().unwrap().defects[0].abs() > 1.0);
        for initial in [1.0, 1.4] {
            let mut matching = Conserved::new(true);
            matching.toy.c.balances[0].transfers.insert(id(5));
            let report = execute(&mut matching, &policy(method), &[initial]);
            assert_eq!(
                report.termination,
                Termination::Completed,
                "{:?}",
                report.error
            );
            closes(&report);
            let before = &report.events[0].before;
            let expected = 3.0 * before[0] * before[0] + before[1];
            assert!((report.conservation.last().unwrap().transfers[0] - expected).abs() < 1e-8);
            matching.transfer_factor = 0.5;
            let report = execute(&mut matching, &policy(method), &[initial]);
            assert_eq!(report.termination, Termination::Failed);
            assert!(
                report
                    .error
                    .unwrap()
                    .to_string()
                    .contains("permitted event transfer")
            );
        }
    }
}
#[test]
fn mode_switch_preserves_subject_with_mode_specific_inventory_definition() {
    for method in methods() {
        let mut oracle = Conserved::new(true);
        oracle.changing_definition = true;
        let report = execute(&mut oracle, &policy(method), &[1.0]);
        assert_eq!(
            report.termination,
            Termination::Completed,
            "{:?}",
            report.error
        );
        closes(&report);
        let at = report
            .conservation
            .iter()
            .filter(|p| (p.time - 0.25).abs() < 1e-9)
            .collect::<Vec<_>>();
        let pre = at.iter().find(|p| p.mode == 0).unwrap();
        let post = at.iter().find(|p| p.mode == 1).unwrap();
        assert!((pre.inventories[0] - post.inventories[0]).abs() < 1e-7);
        assert_eq!(post.transfers, vec![0.0]);
    }
}
#[test]
fn terminal_endpoint_uses_terminating_mode_and_active_segment_without_sample_or_reset() {
    for method in methods() {
        let mut oracle = Conserved::new(true);
        oracle.toy.c.events[0][0].terminal = true;
        oracle.changing_definition = true;
        oracle.input_sensitive_inventory = true;
        let mut p = policy(method);
        p.samples = vec![0.0, 1.0];
        p.schedule = vec![ScheduledInput {
            parameter: 0,
            times: vec![0.25],
        }];
        let report = execute(&mut oracle, &p, &[1.0, 10.0]);
        assert_eq!(report.termination, Termination::Event, "{:?}", report.error);
        assert_eq!(report.events.len(), 1);
        assert!(report.events[0].after.is_none());
        assert_eq!(report.samples.len(), 1);
        let endpoint = report.conservation.last().unwrap();
        assert!((endpoint.time - 0.25).abs() < 1e-9);
        assert_eq!(endpoint.mode, 0);
        let before = &report.events[0].before;
        assert!((endpoint.inventories[0] - (before[0] * before[0] + before[1] + 1.0)).abs() < 1e-8);
        assert_eq!(endpoint.transfers, vec![0.0]);
        closes(&report);
    }
}
#[test]
fn coincident_reset_settles_before_scheduled_consistency_change() {
    for method in methods() {
        let mut oracle = Conserved::new(true);
        oracle.toy.c.balances[0].transfers.insert(id(5));
        let mut p = policy(method);
        p.schedule = vec![ScheduledInput {
            parameter: 0,
            times: vec![0.25],
        }];
        let report = execute(&mut oracle, &p, &[1.0, 2.0]);
        assert_eq!(
            report.termination,
            Termination::Completed,
            "{:?}",
            report.error
        );
        assert_eq!(report.events.len(), 2);
        assert!(report.events[0].event.is_some());
        assert!(report.events[1].event.is_none());
        assert_eq!(
            report.events[0].after.as_ref().unwrap(),
            &report.events[1].before
        );
        assert_eq!(
            report.samples[2].state,
            *report.events[1].after.as_ref().unwrap()
        );
        closes(&report);
    }
}

#[test]
fn coincident_schedule_cannot_hide_inventory_jump_inside_permitted_event_transfer() {
    for method in methods() {
        let mut oracle = Conserved::new(true);
        oracle.toy.c.balances[0].transfers.insert(id(5));
        oracle.input_sensitive_inventory = true;
        let mut p = policy(method);
        p.schedule = vec![ScheduledInput {
            parameter: 0,
            times: vec![0.25],
        }];
        let report = execute(&mut oracle, &p, &[1.0, 2.0]);
        assert_eq!(report.termination, Termination::Failed);
        assert_eq!(report.events.len(), 2);
        assert!(
            report.events[0].after.is_some(),
            "event's own permitted transfer settled"
        );
        assert!(
            report.events[1].after.is_none(),
            "scheduled segment transfers zero"
        );
        assert!((report.conservation.last().unwrap().defects[0] - 1.0).abs() < 1e-6);
    }
}
