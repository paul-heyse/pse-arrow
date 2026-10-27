// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
thread_local! { static CONTEXT_CONSTRUCTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn record_context_construction() {
    CONTEXT_CONSTRUCTIONS.with(|value| value.set(value.get() + 1));
}
fn context_constructions() -> usize {
    CONTEXT_CONSTRUCTIONS.with(std::cell::Cell::get)
}

#[derive(Debug)]
struct Lease(Arc<AtomicUsize>);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
#[derive(Debug)]
struct Coverage(Arc<Lease>);
impl crate::solve::WorkAdmission for Coverage {
    fn prepaid_storage(
        &self,
        _: &Arc<()>,
        required: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        Ok((required == 1 << 28).then(|| {
            let owner: Arc<dyn std::any::Any + Send + Sync> = self.0.clone();
            owner
        }))
    }
    fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn observe(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
}
#[derive(Debug)]
struct Source {
    contract: Contract,
    layout: Arc<PreparedLayout>,
    fail: bool,
    stop_initial: Option<Cancellation>,
    initial_calls: usize,
    expected_execution: Option<Execution>,
}
impl Source {
    fn new() -> Self {
        Self::with_quadratures(true)
    }
    fn sampled() -> Self {
        Self::with_quadratures(false)
    }
    fn with_quadratures(quadratures: bool) -> Self {
        let id = |n| SemanticId::from_bytes([n; 16]);
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([42; 32]),
                states: vec![id(1)],
                differential: vec![true],
                parameters: vec![id(2)],
                outputs: vec![id(3)],
                quadratures: if quadratures { vec![id(4)] } else { vec![] },
                balances: vec![],
                events: vec![vec![]],
                signs: vec![StateSign::Free],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            layout: Arc::new(
                PreparedLayout::new(
                    1,
                    [SupportEntry::new(OriginalRow::new(0), OriginalCol::new(0))].into_iter(),
                    1 << 28,
                )
                .unwrap(),
            ),
            fail: false,
            stop_initial: None,
            initial_calls: 0,
            expected_execution: None,
        }
    }
}
impl Oracle for Source {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        let column = match function {
            Function::Initial => 1,
            Function::Rhs | Function::Output | Function::QuadratureFlux => 0,
            Function::Roots => return vec![],
            _ => return vec![],
        };
        vec![SupportEntry::new(
            OriginalRow::new(0),
            OriginalCol::new(column),
        )]
    }
    fn prepared_layout(&self, _: usize) -> Result<Arc<PreparedLayout>, ProblemError> {
        Ok(self.layout.clone())
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
        if let Some(expected) = &self.expected_execution {
            let actual = crate::execution::current_execution(&expected.cancel)
                .expect("current original execution");
            assert_eq!(
                actual.scope()?.deadline(),
                expected.scope()?.deadline(),
                "the same absolute caller deadline reaches each actual source callback"
            );
            assert!(
                Arc::ptr_eq(
                    actual.work_admission.as_ref().unwrap(),
                    expected.work_admission.as_ref().unwrap()
                ),
                "native callbacks must delegate to the actual current work owner"
            );
        }
        if self.fail {
            return Err(ProblemError::numerical("current refused source"));
        }
        if function == Function::Initial {
            self.initial_calls += 1;
            if let Some(cancel) = &self.stop_initial {
                cancel.cancel();
            }
        }
        let (value, state_partial, parameter_partial) = match function {
            Function::Initial => (parameters[0], 0., 1.),
            Function::Rhs => (-state[0], -1., 0.),
            Function::Output | Function::QuadratureFlux => (state[0], 1., 0.),
            Function::Roots => {
                return Ok(Evaluation {
                    values: if self.contract.events[mode].is_empty() {
                        vec![]
                    } else {
                        vec![if parameters[0] == 0. {
                            0.
                        } else {
                            time - parameters[0]
                        }]
                    },
                    jacobian: None,
                });
            }
            _ => return Err(ProblemError::unsupported("private workspace fixture role")),
        };
        let jacobian = derivatives.then(|| {
            let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                2,
                &[
                    faer::sparse::Triplet::new(0, 0, state_partial),
                    faer::sparse::Triplet::new(0, 1, parameter_partial),
                ],
            )
            .unwrap();
            Arc::new(matrix).into()
        });
        Ok(Evaluation {
            values: vec![value],
            jacobian,
        })
    }
    fn weighted_hessian(
        &mut self,
        _: usize,
        _: Function,
        _: f64,
        _: &[f64],
        _: &[f64],
        _: &[f64],
    ) -> Result<faer::sparse::SparseColMat<usize, f64>, ProblemError> {
        // Every role in this declared affine fixture has zero exact curvature.
        Ok(faer::sparse::SparseColMat::try_new_from_triplets(2, 2, &[]).unwrap())
    }
}
fn profile(sensitivity: DynamicSensitivity) -> Profile {
    Profile {
        method: Method::Idas,
        end: 0.2,
        samples: vec![0., 0.1, 0.2],
        sensitivity,
        parameter_scales: vec![1.],
        rtol: 0.001,
        atol: vec![0.001],
        out_rtol: Some(0.001),
        out_atol: vec![0.001],
        ..Default::default()
    }
}
fn execution(drops: Arc<AtomicUsize>) -> Execution {
    let mut execution = Execution::new(Default::default(), &crate::solve::Controls::default());
    execution.memory = Some(1 << 28);
    execution.work_admission = Some(Arc::new(Coverage(Arc::new(Lease(drops)))));
    execution
}
#[test]
fn retained_idas_native_resources_rebind_current_value_forward_and_discard_failures() {
    let mut source = Source::new();
    let drops = Arc::new(AtomicUsize::new(0));
    let execution = execution(drops.clone());
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    let mut workspace = IdasWorkspace::default();
    for sensitivity in [DynamicSensitivity::None, DynamicSensitivity::Forward] {
        let p = profile(sensitivity);
        let previous_reuses = workspace.reuses;
        let mut pointer = None;
        for parameter in [1., 2., 0.5] {
            let report = integrate_with_workspace_observed(
                &mut source,
                &p,
                &[parameter],
                &execution,
                &snapshot,
                &mut workspace,
            )
            .unwrap();
            assert_eq!(
                report.termination,
                Termination::Completed,
                "{:?}",
                report.error
            );
            for sample in &report.samples {
                let expected = parameter * (-sample.time).exp();
                assert!((sample.state[0] - expected).abs() <= 0.001 * parameter.max(1.));
                assert!(
                    (sample.integrals[0] - parameter * (1. - (-sample.time).exp())).abs()
                        <= 0.001 * parameter.max(1.)
                );
                if sensitivity == DynamicSensitivity::Forward {
                    assert!((sample.state_sensitivities[0] - (-sample.time).exp()).abs() <= 0.001);
                    assert!((sample.output_sensitivities[0] - (-sample.time).exp()).abs() <= 0.001);
                }
            }
            let held = workspace.held.as_ref().unwrap();
            let current = (
                held.1.mem as usize,
                held.1.y as usize,
                held.1.linear as usize,
            );
            if let Some(pointer) = pointer {
                assert_eq!(current, pointer);
            } else {
                pointer = Some(current);
            }
            let mut data = std::ptr::null_mut();
            assert_eq!(
                // SAFETY: held retains the idle initialized IDAS memory exclusively;
                // data is a writable local pointer slot for the getter's result.
                unsafe { ffi::IDAGetUserData(held.1.mem, &raw mut data) },
                ffi::IDA_SUCCESS
            );
            assert!(
                data.is_null(),
                "idle storage must detach the completed callback context"
            );
            assert!(held.1.user_data.as_ref().unwrap().current.is_null());
        }
        assert_eq!(workspace.reuses - previous_reuses, 2);
    }
    source.fail = true;
    let failed = integrate_with_workspace_observed(
        &mut source,
        &profile(DynamicSensitivity::Forward),
        &[1.],
        &execution,
        &snapshot,
        &mut workspace,
    )
    .unwrap();
    assert_ne!(failed.termination, Termination::Completed);
    assert!(workspace.held.is_none());
    source.fail = false;
    assert_eq!(
        integrate_with_workspace_observed(
            &mut source,
            &profile(DynamicSensitivity::None),
            &[1.],
            &execution,
            &snapshot,
            &mut workspace
        )
        .unwrap()
        .termination,
        Termination::Completed
    );
    drop(execution);
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "native storage owns its detached lease after execution drops"
    );
    drop(workspace);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn sundials_790_reinit_clears_attached_linear_counts_before_new_steps() {
    let mut source = Source::new();
    let execution = execution(Arc::new(AtomicUsize::new(0)));
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    let mut workspace = IdasWorkspace::default();
    let selected = profile(DynamicSensitivity::None);
    let report = integrate_with_workspace_observed(
        &mut source,
        &selected,
        &[1.],
        &execution,
        &snapshot,
        &mut workspace,
    )
    .unwrap();
    assert_eq!(report.termination, Termination::Completed);
    let held = &mut workspace.held.as_mut().unwrap().1;
    let mut prior = 0;
    // SAFETY: idle session retains initialized IDAS and its attached KLU interface.
    assert_eq!(unsafe { ffi::IDAGetNumJacEvals(held.mem, &mut prior) }, 0);
    assert!(
        prior > 0,
        "the completed solve must establish nonzero old linear work"
    );
    // SAFETY: live exclusively owned vectors of the original state extent; ReInit
    // copies them and resets counters without invoking the detached callback.
    let reinitialized = unsafe { ffi::IDAReInit(held.mem, selected.start, held.y, held.dy) };
    assert_eq!(reinitialized, 0);
    let mut current = -1;
    // SAFETY: the same idle memory and scalar out pointer remain live.
    assert_eq!(unsafe { ffi::IDAGetNumJacEvals(held.mem, &mut current) }, 0);
    assert_eq!(
        current, 0,
        "7.9 resets the attached linear counter at ReInit"
    );
    // SAFETY: read-only query of this reinitialized memory into the scalar local.
    let queried = unsafe { ffi::IDAGetNumConstraintFails(held.mem, &mut current) };
    assert_eq!(queried, 0);
    assert_eq!(current, 0);
}
#[test]
fn retained_forward_userdata_slot_does_not_retain_cancelled_prior_context() {
    let mut source = Source::new();
    let old_drops = Arc::new(AtomicUsize::new(0));
    let current_drops = Arc::new(AtomicUsize::new(0));
    let old = execution(old_drops.clone());
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    let p = profile(DynamicSensitivity::Forward);
    let mut workspace = IdasWorkspace::default();
    assert_eq!(
        integrate_with_workspace_observed(&mut source, &p, &[1.], &old, &snapshot, &mut workspace)
            .unwrap()
            .termination,
        Termination::Completed
    );
    old.cancellation_control().cancel();
    drop(old);
    let current = execution(current_drops.clone());
    let report = integrate_with_workspace_observed(
        &mut source,
        &p,
        &[2.],
        &current,
        &snapshot,
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        report.termination,
        Termination::Completed,
        "{:?}",
        report.error
    );
    assert_eq!(workspace.reuses, 1);
    assert_eq!(
        old_drops.load(Ordering::SeqCst),
        1,
        "replacement coverage releases only the old detached lease"
    );
    assert!(
        (report.samples.last().unwrap().state_sensitivities[0] - (-p.end).exp()).abs() <= 0.001
    );
    current.cancellation_control().cancel();
    assert!(
        integrate_with_workspace_observed(
            &mut source,
            &p,
            &[2.],
            &current,
            &snapshot,
            &mut workspace
        )
        .is_err()
    );
    assert!(
        workspace.held.is_none(),
        "cancelled current callers discard retained native state"
    );
    drop(current);
    assert_eq!(current_drops.load(Ordering::SeqCst), 1);
}
#[test]
fn same_count_mode_roots_replace_inactive_population_and_zero_directions() {
    let mut source = Source::new();
    let event = |id, direction| Event {
        id: SemanticId::from_bytes([id; 16]),
        terminal: true,
        next_mode: 0,
        tolerance: 0.001,
        direction,
    };
    source.contract.events = vec![
        vec![event(11, EventDirection::Falling)],
        vec![event(12, EventDirection::Either)],
    ];
    let p = profile(DynamicSensitivity::None);
    let execution = execution(Arc::new(AtomicUsize::new(0)));
    let mut session = Session::new(&mut source, &[0.], &p, &[], None, execution, false).unwrap();
    let mut time = 0.;
    assert_eq!(
        // SAFETY: this exclusive initialized session owns live one-state y/dy and
        // the current callback slot for the complete solve; time is a writable local.
        unsafe {
            ffi::IDASolve(
                session.mem,
                p.end,
                &raw mut time,
                session.y,
                session.dy,
                ffi::IDA_NORMAL,
            )
        },
        ffi::IDA_SUCCESS
    );
    session.callback.mode = 1;
    session.callback.parameters[0] = 0.1;
    // SAFETY: session.y is a live exclusive one-entry serial vector; IDAS is idle.
    unsafe {
        write(session.y, &[0.1]);
    }
    // SAFETY: session.dy is a separate live exclusive one-entry serial vector.
    unsafe {
        write(session.dy, &[-0.1]);
    }
    assert_eq!(
        // SAFETY: initialized IDAS memory and both one-state vectors remain live
        // under this idle session; the current callback slot is retained across reset.
        unsafe { ffi::IDAReInit(session.mem, 0., session.y, session.dy) },
        ffi::IDA_SUCCESS
    );
    session.initialize_roots().unwrap();
    session.consistent(0., &p, ffi::IDA_YA_YDP_INIT).unwrap();
    assert_eq!(
        // SAFETY: reset, roots and IC completed on the same exclusive session;
        // y/dy and the current callback slot stay live, and time is writable.
        unsafe {
            ffi::IDASolve(
                session.mem,
                p.end,
                &raw mut time,
                session.y,
                session.dy,
                ffi::IDA_NORMAL,
            )
        },
        ffi::IDA_ROOT_RETURN
    );
    assert!(
        (time - 0.1).abs() <= 1e-12,
        "the current either-direction increasing root must be active: {time}"
    );
}

#[derive(Debug)]
struct StopOnCoverage {
    coverage: Coverage,
    cancel: Cancellation,
}
impl crate::solve::WorkAdmission for StopOnCoverage {
    fn prepaid_storage(
        &self,
        identity: &Arc<()>,
        required: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        let coverage = self.coverage.prepaid_storage(identity, required);
        self.cancel.cancel();
        coverage
    }
    fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn observe(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
}
#[test]
fn current_stop_before_or_during_initial_source_prevents_native_construction() {
    for during_source in [false, true] {
        let mut source = Source::new();
        let mut execution = Execution::new(Default::default(), &crate::solve::Controls::default());
        execution.memory = Some(1 << 28);
        let drops = Arc::new(AtomicUsize::new(0));
        let coverage = Coverage(Arc::new(Lease(drops.clone())));
        if during_source {
            source.stop_initial = Some(execution.cancellation_control().clone());
            execution.work_admission = Some(Arc::new(coverage));
        } else {
            execution.work_admission = Some(Arc::new(StopOnCoverage {
                coverage,
                cancel: execution.cancellation_control().clone(),
            }));
        }
        let constructed = context_constructions();
        let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
        let mut workspace = IdasWorkspace::default();
        let report = integrate_with_workspace_observed(
            &mut source,
            &profile(DynamicSensitivity::None),
            &[1.],
            &execution,
            &snapshot,
            &mut workspace,
        )
        .unwrap();
        assert_eq!(report.termination, Termination::Cancelled);
        assert!(matches!(report.error, Some(ProblemError::Cancelled)));
        assert_eq!(source.initial_calls, usize::from(during_source));
        assert!(
            report.requested_initial.is_empty(),
            "a stopped initial result is never published"
        );
        assert_eq!(
            context_constructions(),
            constructed,
            "no native context may be constructed after the stop"
        );
        assert!(workspace.held.is_none());
        drop(execution);
        drop(source);
        assert_eq!(
            drops.load(Ordering::SeqCst),
            1,
            "refused native coverage is dropped"
        );
    }
}

#[derive(Debug)]
struct ScheduledDae {
    contract: Contract,
    layout: Arc<PreparedLayout>,
}
impl ScheduledDae {
    fn new() -> Self {
        let id = |n| SemanticId::from_bytes([n; 16]);
        Self {
            contract: Contract {
                identity: ContentHash::from_bytes([43; 32]),
                states: vec![id(1), id(2)],
                differential: vec![true, false],
                parameters: vec![id(3), id(4)],
                outputs: vec![id(5)],
                quadratures: vec![id(6)],
                balances: vec![],
                events: vec![vec![]],
                signs: vec![StateSign::Free; 2],
                derivatives: pse_kernels::DerivativeOrder::First,
            },
            layout: Arc::new(
                PreparedLayout::new(
                    2,
                    [(0, 0), (1, 0), (1, 1)]
                        .into_iter()
                        .map(|(r, c)| SupportEntry::new(OriginalRow::new(r), OriginalCol::new(c))),
                    1 << 28,
                )
                .unwrap(),
            ),
        }
    }
}
impl Oracle for ScheduledDae {
    fn contract(&self) -> &Contract {
        &self.contract
    }
    fn prepared_layout(&self, _: usize) -> Result<Arc<PreparedLayout>, ProblemError> {
        Ok(self.layout.clone())
    }
    fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
        let entries: &[(usize, usize)] = match function {
            Function::Initial => &[(0, 2), (1, 2)],
            Function::Rhs => &[(0, 0), (0, 3), (1, 0), (1, 1)],
            Function::Output | Function::QuadratureFlux => &[(0, 1)],
            _ => &[],
        };
        entries
            .iter()
            .map(|&(r, c)| SupportEntry::new(OriginalRow::new(r), OriginalCol::new(c)))
            .collect()
    }
    fn evaluate(
        &mut self,
        _: usize,
        function: Function,
        _: f64,
        x: &[f64],
        p: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError> {
        let (values, entries) = match function {
            Function::Initial => (vec![p[0]; 2], vec![(0, 2, 1.), (1, 2, 1.)]),
            Function::Rhs => (
                vec![-p[1] * x[0], x[1] - x[0]],
                vec![(0, 0, -p[1]), (0, 3, -x[0]), (1, 0, -1.), (1, 1, 1.)],
            ),
            Function::Output | Function::QuadratureFlux => (vec![x[1]], vec![(0, 1, 1.)]),
            Function::Roots => (vec![], vec![]),
            _ => return Err(ProblemError::unsupported("scheduled DAE workspace role")),
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
#[test]
fn retained_scheduled_dae_and_forward_quadratures_match_fresh_current_products() {
    let mut source = ScheduledDae::new();
    let mut p = profile(DynamicSensitivity::Forward);
    p.atol = vec![0.001; 2];
    p.parameter_scales = vec![1.; 2];
    p.schedule = vec![ScheduledInput {
        parameter: 1,
        times: vec![0.1],
    }];
    let drops = Arc::new(AtomicUsize::new(0));
    let execution = execution(drops.clone());
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    let mut workspace = IdasWorkspace::default();
    for quadrature_derivatives in [false, true] {
        let mut pointer = None;
        let before = workspace.reuses;
        for values in [[1., 1., 2.], [2., 0.5, 1.5], [0.5, 2., 0.75]] {
            let (report, derivatives) = if quadrature_derivatives {
                let layout = source.layout.clone();
                let result = forward_quadratures(
                    &mut source,
                    &p,
                    &values,
                    execution.cancellation_control().clone(),
                    execution.progress.clone(),
                    (layout, 1 << 28),
                    1 << 28,
                    Some((&mut workspace, execution.clone())),
                )
                .unwrap();
                (result.report, result.derivatives)
            } else {
                (
                    integrate_with_workspace_observed(
                        &mut source,
                        &p,
                        &values,
                        &execution,
                        &snapshot,
                        &mut workspace,
                    )
                    .unwrap(),
                    None,
                )
            };
            assert_eq!(
                report.termination,
                Termination::Completed,
                "{:?}",
                report.error
            );
            let mut fresh = ScheduledDae::new();
            let current = if quadrature_derivatives {
                let layout = fresh.layout.clone();
                let result = forward_quadratures(
                    &mut fresh,
                    &p,
                    &values,
                    execution.cancellation_control().clone(),
                    execution.progress.clone(),
                    (layout, 1 << 28),
                    1 << 28,
                    None,
                )
                .unwrap();
                (result.report, result.derivatives)
            } else {
                (
                    integrate_with_workspace_observed(
                        &mut fresh,
                        &p,
                        &values,
                        &execution,
                        &snapshot,
                        &mut IdasWorkspace::default(),
                    )
                    .unwrap(),
                    None,
                )
            };
            assert_eq!(
                current.0.termination,
                Termination::Completed,
                "{:?}",
                current.0.error
            );
            for (actual, expected) in report.samples.iter().zip(&current.0.samples) {
                let exponent =
                    -values[1] * actual.time.min(0.1) - values[2] * (actual.time - 0.1).max(0.);
                let analytic = values[0] * exponent.exp();
                assert!((actual.state[0] - analytic).abs() <= 0.001 * (1. + analytic.abs()));
                assert!((actual.state[1] - actual.state[0]).abs() <= 0.001);
                for (a, b) in actual
                    .state
                    .iter()
                    .chain(&actual.output_sensitivities)
                    .chain(&actual.integrals)
                    .zip(
                        expected
                            .state
                            .iter()
                            .chain(&expected.output_sensitivities)
                            .chain(&expected.integrals),
                    )
                {
                    assert!((a - b).abs() <= 0.001 * (1. + b.abs()));
                }
            }
            if quadrature_derivatives {
                let actual = derivatives.unwrap();
                let expected = current.1.unwrap();
                assert_eq!(actual.len(), 3);
                for (a, b) in actual.iter().zip(expected) {
                    assert!((a - b).abs() <= 0.001 * (1. + b.abs()));
                }
            }
            let held = &workspace.held.as_ref().unwrap().1;
            let current = (held.mem as usize, held.y as usize, held.linear as usize);
            if let Some(pointer) = pointer {
                assert_eq!(current, pointer);
            } else {
                pointer = Some(current);
            }
        }
        assert_eq!(workspace.reuses - before, 2);
    }
    drop(execution);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(workspace);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[derive(Debug)]
struct CurrentWork {
    occupied: bool,
    admitted: Arc<AtomicUsize>,
    observed: Arc<AtomicUsize>,
}
impl crate::solve::WorkAdmission for CurrentWork {
    fn prepaid_storage(
        &self,
        _: &Arc<()>,
        _: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        if self.occupied {
            Err(ProblemError::memory(
                "another native owner occupies current coverage",
            ))
        } else {
            Ok(None)
        }
    }
    fn admit(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.admitted
            .fetch_add(work.evaluations.unwrap_or(0) as usize, Ordering::SeqCst);
        Ok(())
    }
    fn observe(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.observed
            .fetch_add(work.evaluations.unwrap_or(0) as usize, Ordering::SeqCst);
        Ok(())
    }
}
/// Both fresh backward populations receive the actual work/storage owner and
/// unchanged original deadline. Unsupported retention does not remove admission.
#[test]
fn fresh_adjoint_and_second_order_preserve_current_work_deadline_and_occupied_coverage() {
    let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
    for second in [false, true] {
        for boundary in ["completed", "occupied", "expired", "initial_cancel"] {
            let mut source = Source::sampled();
            source.contract.derivatives = pse_kernels::DerivativeOrder::Second;
            let admitted = Arc::new(AtomicUsize::new(0));
            let observed = Arc::new(AtomicUsize::new(0));
            let mut current =
                Execution::new(Default::default(), &crate::solve::Controls::default());
            current.time_limit = Duration::from_secs(5);
            current.memory = Some(1 << 28);
            current.work_admission = Some(Arc::new(CurrentWork {
                occupied: boundary == "occupied",
                admitted: admitted.clone(),
                observed: observed.clone(),
            }));
            // Parent records must not leak into this independent report's child history.
            current.progress.push(crate::solve::Event {
                source: None,
                phase: "earlier.parent".into(),
                elapsed: Duration::ZERO,
                values: Default::default(),
                incumbent: None,
            });
            source.expected_execution = Some(current.clone());
            if boundary == "initial_cancel" {
                source.stop_initial = Some(current.cancellation_control().clone());
            }
            if boundary == "expired" {
                current.started -= Duration::from_secs(10);
            }
            let p = Profile {
                out_rtol: None,
                out_atol: vec![],
                ..profile(DynamicSensitivity::Adjoint)
            };
            let mut cotangent = |report: &Report| {
                let mut weights = vec![0.; report.samples.len()];
                *weights.last_mut().unwrap() = 1.;
                Ok(weights)
            };
            let before = context_constructions();
            let result = if second {
                hessian_current_observed(
                    &mut source,
                    &p,
                    &[1.],
                    &[0],
                    &mut cotangent,
                    &current,
                    &snapshot,
                )
            } else {
                gradient_current_observed(
                    &mut source,
                    &p,
                    &[1.],
                    &mut cotangent,
                    &current,
                    &snapshot,
                )
            };
            if boundary == "expired" {
                assert!(result.is_err());
                assert_eq!(source.initial_calls, 0);
                assert_eq!(context_constructions(), before);
                continue;
            }
            let result = result.unwrap();
            if boundary == "completed" {
                assert_eq!(
                    result.report.termination,
                    Termination::Completed,
                    "{:?}",
                    result.report.error
                );
                assert!((result.gradient.unwrap()[0] - (-p.end).exp()).abs() <= 0.001);
                if second {
                    assert!(result.hessian.unwrap()[(0, 0)].abs() <= 0.001);
                }
                assert!(admitted.load(Ordering::SeqCst) > 0);
                assert_eq!(
                    admitted.load(Ordering::SeqCst),
                    observed.load(Ordering::SeqCst)
                );
                assert!(
                    !result
                        .report
                        .progress
                        .iter()
                        .any(|event| event.phase == "earlier.parent"),
                    "earlier parent native history must remain outside this child report"
                );
            } else {
                assert_ne!(result.report.termination, Termination::Completed);
                assert!(result.report.error.is_some());
                assert_eq!(
                    source.initial_calls,
                    usize::from(boundary == "initial_cancel")
                );
                assert_eq!(
                    context_constructions(),
                    before,
                    "no native constructor follows occupied coverage or an Initial stop"
                );
            }
        }
    }
}
