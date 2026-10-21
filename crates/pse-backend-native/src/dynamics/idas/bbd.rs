// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native band approximation over the original checked residual; exact JVPs stay separate.
use super::*;

/// One consumed preparation authority, bound to its checked configuration and
/// original execution. Only admission can construct it; it cannot be cloned to
/// charge or authorize a second independent native owner.
pub(super) struct Admission {
    linear: IdasLinear,
    states: usize,
    execution: Execution,
}
impl Admission {
    pub(super) fn into_execution(
        self,
        linear: IdasLinear,
        states: usize,
    ) -> Result<Execution, ProblemError> {
        if self.linear != linear || self.states != states {
            return Err(ProblemError::internal(
                "IDAS BBD admission configuration changed",
            ));
        }
        self.execution.check()?;
        Ok(self.execution)
    }
}
/// Charge the complete native replacement population before initial source or
/// constructor effects. The same admitted scope covers subsequent segment resets.
pub(super) fn admit(
    linear: IdasLinear,
    states: usize,
    execution: Execution,
) -> Result<Admission, ProblemError> {
    execution.check()?;
    linear.validate_for_states(states)?;
    if matches!(linear, IdasLinear::SpgmrBbd { .. }) {
        let bytes = linear.bbd_bytes(states)?;
        if execution.memory.is_none_or(|allowance| bytes > allowance) {
            return Err(ProblemError::memory(
                "IDAS BBD exceeds admitted replacement storage",
            ));
        }
        if let Some(owner) = &execution.work_admission {
            owner.admit_storage(
                crate::solve::NativeStorageScope::Linear,
                "idas.bbd.native",
                bytes,
                false,
            )?;
        }
    }
    execution.check()?;
    Ok(Admission {
        linear,
        states,
        execution,
    })
}
/// Native BBD owns its buffers/factor until IDAS frees or replaces them. A new
/// population is allocated before the previous one is released; admission covers both.
pub(super) fn install(session: &mut Session<'_>, linear: IdasLinear) -> Result<(), ProblemError> {
    let IdasLinear::SpgmrBbd {
        difference,
        retained,
        ..
    } = linear
    else {
        return Ok(());
    };
    let n = session.callback.contract.states.len();
    linear.validate_for_states(n)?;
    let bytes = linear.bbd_bytes(n)?;
    let execution = session.callback.callback.execution.clone();
    execution.check()?;
    if execution.memory.is_none_or(|allowance| bytes > allowance) {
        return Err(ProblemError::memory(
            "IDAS BBD exceeds admitted replacement storage",
        ));
    }
    crate::observation::integer(
        "idas.bbd.replacement_workspace.bytes",
        "bytes",
        i64::try_from(bytes).map_err(|_| ProblemError::memory("IDAS BBD storage index"))?,
    );
    check(
        // SAFETY: the session owns live IDAS memory with attached left-SPGMR.
        // Checked extents fit its serial state. IDAS owns the resulting BBD state
        // and passes the same live current-context slot to local_residual.
        unsafe {
            ffi::IDABBDPrecInit(
                session.mem,
                index(n)?,
                index(difference.upper)?,
                index(difference.lower)?,
                index(retained.upper)?,
                index(retained.lower)?,
                0.,
                Some(local_residual),
                None,
            )
        },
        "BBD preconditioner",
    )?;
    execution.check()?;
    crate::observation::count("idas.bbd.installations", 1);
    let mut real = 0;
    let mut integer = 0;
    check(
        // SAFETY: this live IDAS owner just installed BBD; both exact ABI output
        // counters are private writable locals and no native call runs concurrently.
        unsafe { ffi::IDABBDPrecGetWorkSpace(session.mem, &mut real, &mut integer) },
        "BBD workspace observation",
    )?;
    crate::observation::integer("idas.bbd.native_workspace.real_cells", "cells", real);
    crate::observation::integer("idas.bbd.native_workspace.integer_cells", "cells", integer);
    Ok(())
}

unsafe extern "C" fn local_residual(
    n: ffi::sunindextype,
    t: f64,
    y: ffi::N_Vector,
    yp: ffi::N_Vector,
    output: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: the native slot resolves this active session's Context; the borrow ends before
    // delegating to the existing panic-contained residual callback.
    let Some(context) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if usize::try_from(n).ok() != Some(context.contract.states.len()) {
            context.callback.last_failure =
                Some(ProblemError::internal("IDAS BBD local state extent"));
            return -1;
        }
        crate::observation::count("idas.bbd.local_residuals", 1);
        0
    }));
    let flag = finish_callback(context, result);
    if flag != 0 {
        return flag;
    }
    // SAFETY: native BBD supplies serial vectors at the checked extent. The
    // ordinary callback checks current physical/source inputs, execution and
    // trial failures, catches panics, and publishes only a completed residual.
    unsafe { residual(t, y, yp, output, data) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Debug)]
    struct Coupled {
        contract: Contract,
        layout: Arc<PreparedLayout>,
        calls: Arc<AtomicUsize>,
        initial_calls: Arc<AtomicUsize>,
        actions: Arc<AtomicUsize>,
        fail: Arc<AtomicBool>,
        allow_full: bool,
    }
    impl Coupled {
        fn new() -> Self {
            let id = |i: u8| SemanticId::from_bytes([i; 16]);
            Self {
                contract: Contract {
                    identity: ContentHash::from_bytes([208; 32]),
                    states: vec![id(1), id(2), id(3)],
                    differential: vec![true, false, false],
                    parameters: vec![id(4)],
                    outputs: vec![id(5)],
                    events: vec![vec![], vec![]],
                    quadratures: vec![],
                    balances: vec![],
                    signs: vec![],
                    derivatives: pse_kernels::DerivativeOrder::First,
                },
                layout: Arc::new(
                    PreparedLayout::new(3, Self::rhs_support().into_iter(), 64 << 20).unwrap(),
                ),
                calls: Arc::default(),
                initial_calls: Arc::default(),
                actions: Arc::default(),
                fail: Arc::default(),
                allow_full: false,
            }
        }
        fn rhs_support() -> Vec<SupportEntry> {
            entries(vec![(0, 0), (0, 1), (0, 3), (1, 0), (1, 2), (2, 0), (2, 1)])
        }
    }
    impl Oracle for Coupled {
        fn contract(&self) -> &Contract {
            &self.contract
        }
        fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
            match function {
                Function::Rhs => Self::rhs_support(),
                Function::Output => entries(vec![(0, 0)]),
                _ => vec![],
            }
        }
        fn prepared_layout(&self, _: usize) -> Result<Arc<PreparedLayout>, ProblemError> {
            // Like prepared production sources, this layout already exists. A
            // small attempt allowance must reach BBD admission rather than be
            // refused earlier by the default support-construction fallback.
            Ok(self.layout.clone())
        }
        fn evaluate(
            &mut self,
            mode: usize,
            function: Function,
            _: f64,
            x: &[f64],
            p: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if function == Function::Initial {
                self.initial_calls.fetch_add(1, Ordering::Relaxed);
            }
            if self.fail.load(Ordering::Acquire) {
                return Err(ProblemError::numerical(
                    "BBD current original source refusal",
                ));
            }
            if derivatives && !self.allow_full {
                return Err(ProblemError::unsupported(
                    "BBD test forbids full numeric Jacobian",
                ));
            }
            let values = match function {
                Function::Initial => vec![1., 1., 2.],
                Function::Rhs => vec![
                    -p[0] * (mode + 1) as f64 * x[0] + 100. * (x[1] - x[0]),
                    x[2] - 2. * x[0],
                    x[1] - x[0],
                ],
                Function::Output => vec![x[0]],
                _ => vec![],
            };
            let jacobian = if derivatives {
                let triples = match function {
                    Function::Rhs => vec![
                        (0, 0, -p[0] * (mode + 1) as f64 - 100.),
                        (0, 1, 100.),
                        (0, 3, -((mode + 1) as f64) * x[0]),
                        (1, 0, -2.),
                        (1, 2, 1.),
                        (2, 0, -1.),
                        (2, 1, 1.),
                    ],
                    Function::Output => vec![(0, 0, 1.)],
                    _ => vec![],
                };
                Some(
                    Arc::new(
                        faer::sparse::SparseColMat::try_new_from_triplets(
                            values.len(),
                            4,
                            &triples
                                .into_iter()
                                .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
                                .collect::<Vec<_>>(),
                        )
                        .unwrap(),
                    )
                    .into(),
                )
            } else {
                None
            };
            Ok(Evaluation { values, jacobian })
        }
        fn jacobian_action(
            &mut self,
            mode: usize,
            function: Function,
            _: f64,
            state: &[f64],
            p: &[f64],
            direction: &[f64],
            output: &mut [f64],
        ) -> Result<(), ProblemError> {
            self.actions.fetch_add(1, Ordering::Relaxed);
            if self.fail.load(Ordering::Acquire)
                || state.len() != 3
                || p.len() != 1
                || direction.len() != 4
                || state
                    .iter()
                    .chain(p)
                    .chain(direction)
                    .any(|value| !value.is_finite())
            {
                return Err(ProblemError::numerical("BBD exact current action input"));
            }
            let scale = (mode + 1) as f64;
            let candidate = match function {
                Function::Rhs => vec![
                    -(p[0] * scale + 100.) * direction[0] + 100. * direction[1]
                        - scale * state[0] * direction[3],
                    direction[2] - 2. * direction[0],
                    direction[1] - direction[0],
                ],
                Function::Initial => vec![0.; 3],
                Function::Output => vec![direction[0]],
                _ => vec![],
            };
            if candidate.len() != output.len() || candidate.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("BBD exact action output"));
            }
            output.copy_from_slice(&candidate);
            Ok(())
        }
    }
    fn linear(retained: usize) -> IdasLinear {
        IdasLinear::SpgmrBbd {
            dimension: PositiveCount::try_new(8).unwrap(),
            difference: IdasBand { upper: 2, lower: 2 },
            retained: IdasBand {
                upper: retained,
                lower: retained,
            },
        }
    }
    fn profile(linear: IdasLinear) -> Profile {
        Profile {
            method: Method::Idas,
            end: 0.4,
            samples: vec![0.1, 0.2, 0.3, 0.4],
            parameter_scales: vec![1.],
            rtol: 0.001,
            atol: vec![0.001; 3],
            initial_step: 1e-5,
            idas: IdasSettings {
                linear,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn execution() -> Execution {
        let mut execution = Execution::new(Cancellation::default(), &Default::default());
        execution.memory = Some(64 << 20);
        execution
    }

    #[derive(Debug, Default)]
    struct StorageAdmission {
        charges: AtomicUsize,
        bytes: AtomicUsize,
        deny: bool,
        cancel: Option<Cancellation>,
    }
    impl crate::solve::WorkAdmission for StorageAdmission {
        fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            Ok(())
        }
        fn observe(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            Ok(())
        }
        fn admit_storage(
            &self,
            scope: crate::solve::NativeStorageScope,
            owner: &str,
            bytes: usize,
            opaque: bool,
        ) -> Result<(), ProblemError> {
            if owner == "idas.bbd.native" {
                assert_eq!(scope, crate::solve::NativeStorageScope::Linear);
                assert!(!opaque);
                self.charges.fetch_add(1, Ordering::Relaxed);
                self.bytes.store(bytes, Ordering::Relaxed);
                if let Some(cancel) = &self.cancel {
                    cancel.cancel();
                }
                if self.deny {
                    return Err(ProblemError::memory("denied BBD storage owner"));
                }
            }
            Ok(())
        }
    }
    struct CallerOutput(Vec<f64>);
    impl SampleObserver for CallerOutput {
        fn sample(&mut self, _: usize, sample: &Sample) -> Result<(), ProblemError> {
            self.0.push(sample.state[0]);
            Ok(())
        }
    }
    fn assert_memory_refusal(report: &Report, oracle: &Coupled) {
        assert!(
            matches!(
                report.error,
                Some(ProblemError::Limit {
                    kind: crate::LimitKind::Memory,
                    ..
                })
            ),
            "{:?}",
            report.error
        );
        assert_ne!(report.termination, Termination::Completed);
        assert!(report.requested_initial.is_empty());
        assert!(report.consistent_initial.is_empty());
        assert!(report.samples.is_empty());
        assert_eq!(oracle.initial_calls.load(Ordering::Relaxed), 0);
        assert_eq!(oracle.calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn bbd_public_entries_refuse_storage_before_initial_effects_and_sample_output() {
        let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
        let p = profile(linear(1));
        let parameters = [1.];
        let mut oracle = Coupled::new();
        let mut output = CallerOutput(vec![777.]);
        let report = integrate_with_samples_observed(
            &mut oracle,
            &p,
            &parameters,
            Cancellation::default(),
            Arc::new(Progress::new(256)),
            &snapshot,
            SampleRetention::Full,
            &mut output,
        )
        .unwrap();
        assert_memory_refusal(&report, &oracle);
        assert_eq!(output.0, [777.]);
        for workspace_entry in [false, true] {
            for refusal in 0..3 {
                let mut oracle = Coupled::new();
                let owner = Arc::new(StorageAdmission {
                    deny: refusal == 2,
                    ..Default::default()
                });
                let mut current = execution();
                current.memory = match refusal {
                    0 => None,
                    1 => Some(1),
                    _ => current.memory,
                };
                current.work_admission = Some(owner.clone());
                let mut workspace = IdasWorkspace::default();
                let report = if workspace_entry {
                    integrate_with_workspace_observed(
                        &mut oracle,
                        &p,
                        &parameters,
                        &current,
                        &snapshot,
                        &mut workspace,
                    )
                } else {
                    integrate_with_samples_current_observed(
                        &mut oracle,
                        &p,
                        &parameters,
                        &current,
                        &snapshot,
                        SampleRetention::Full,
                        &mut output,
                    )
                }
                .unwrap();
                assert_memory_refusal(&report, &oracle);
                assert_eq!(output.0, [777.]);
                assert_eq!(parameters, [1.]);
                assert!(workspace.held.is_none());
                assert_eq!(
                    owner.charges.load(Ordering::Relaxed),
                    usize::from(refusal == 2)
                );
            }
        }
    }

    #[test]
    fn bbd_public_finite_attempt_charges_once_and_drains_failed_or_cancelled_owners() {
        let snapshot = crate::execution::Snapshot::observe(&crate::execution::LINKED);
        let p = profile(linear(1));
        for failed_source in [false, true] {
            let mut oracle = Coupled::new();
            oracle.fail.store(failed_source, Ordering::Release);
            let owner = Arc::new(StorageAdmission::default());
            let mut current = execution();
            current.work_admission = Some(owner.clone());
            let mut workspace = IdasWorkspace::default();
            let report = integrate_with_workspace_observed(
                &mut oracle,
                &p,
                &[1.],
                &current,
                &snapshot,
                &mut workspace,
            )
            .unwrap();
            assert_eq!(owner.charges.load(Ordering::Relaxed), 1);
            assert_eq!(
                owner.bytes.load(Ordering::Relaxed),
                p.idas.linear.bbd_bytes(3).unwrap()
            );
            assert!(oracle.initial_calls.load(Ordering::Relaxed) > 0);
            assert!(workspace.held.is_none(), "BBD storage remains fresh-only");
            if failed_source {
                assert!(matches!(report.error, Some(ProblemError::Numerical { .. })));
                assert!(report.samples.is_empty());
            } else {
                assert_eq!(
                    report.termination,
                    Termination::Completed,
                    "{:?}",
                    report.error
                );
                assert_eq!(report.samples.len(), p.samples.len());
                for sample in &report.samples {
                    assert!((sample.state[1] - sample.state[0]).abs() <= 0.001);
                    assert!((sample.state[2] - 2. * sample.state[0]).abs() <= 0.001);
                }
                assert!(oracle.actions.load(Ordering::Relaxed) > 0);
            }
        }
        // Admission can itself trigger a current stop. No Initial effects or
        // output publication may escape that late cancellation checkpoint.
        let mut oracle = Coupled::new();
        let mut current = execution();
        let owner = Arc::new(StorageAdmission {
            cancel: Some(current.cancellation.clone()),
            ..Default::default()
        });
        current.work_admission = Some(owner.clone());
        let mut output = CallerOutput(vec![888.]);
        let report = integrate_with_samples_current_observed(
            &mut oracle,
            &p,
            &[1.],
            &current,
            &snapshot,
            SampleRetention::Full,
            &mut output,
        )
        .unwrap();
        assert_eq!(report.termination, Termination::Cancelled);
        assert!(matches!(report.error, Some(ProblemError::Cancelled)));
        assert_eq!(oracle.initial_calls.load(Ordering::Relaxed), 0);
        assert_eq!(owner.charges.load(Ordering::Relaxed), 1);
        assert_eq!(output.0, [888.]);
    }

    #[test]
    fn bbd_header_allowance_covers_actual_bound_native_abi() {
        let vector = size_of::<ffi::_generic_N_Vector>()
            + size_of::<ffi::_generic_N_Vector_Ops>()
            + size_of::<ffi::_N_VectorContent_Serial>();
        let matrix = size_of::<ffi::_generic_SUNMatrix>()
            + size_of::<ffi::_generic_SUNMatrix_Ops>()
            + size_of::<ffi::_SUNMatrixContent_Band>();
        let solver = size_of::<ffi::_generic_SUNLinearSolver>()
            + size_of::<ffi::_generic_SUNLinearSolver_Ops>()
            + size_of::<ffi::_SUNLinearSolverContent_Band>();
        // The private IBBDPrecDataRec has five indices, one real, three
        // long counters and eleven pointers/function pointers in the pinned
        // source; 256 bytes includes its native alignment padding.
        assert!(6 * vector + matrix + solver + 256 <= 6 * 512 + 4096);
    }

    #[test]
    fn bbd_coupled_native_route_keeps_exact_off_band_action_and_current_reset() {
        for retained in [1, 2] {
            let complete = Instant::now();
            let mut oracle = Coupled::new();
            let calls = oracle.calls.clone();
            let actions = oracle.actions.clone();
            // The exact action retains differential/algebraic couplings beyond
            // a retained width-one band. Independent perturbations check all rows.
            let state = [0.7, 0.8, 1.1];
            let direction = [0.2, -0.4, 0.9, 0.3];
            let mut action = [0.; 3];
            oracle
                .jacobian_action(1, Function::Rhs, 0., &state, &[2.], &direction, &mut action)
                .unwrap();
            let h = 1e-5;
            let perturbed = |sign: f64| {
                state
                    .iter()
                    .zip(direction)
                    .map(|(x, d)| x + sign * h * d)
                    .collect::<Vec<_>>()
            };
            let plus = oracle
                .evaluate(
                    1,
                    Function::Rhs,
                    0.,
                    &perturbed(1.),
                    &[2. + h * direction[3]],
                    false,
                )
                .unwrap();
            let minus = oracle
                .evaluate(
                    1,
                    Function::Rhs,
                    0.,
                    &perturbed(-1.),
                    &[2. - h * direction[3]],
                    false,
                )
                .unwrap();
            for i in 0..3 {
                assert!((action[i] - (plus.values[i] - minus.values[i]) / (2. * h)).abs() < 0.001);
            }
            let p = profile(linear(retained));
            p.validate(oracle.contract(), &[1.]).unwrap();
            let admission = Arc::new(StorageAdmission::default());
            let mut current_execution = execution();
            current_execution.work_admission = Some(admission.clone());
            let mut session =
                Session::new(&mut oracle, &[1.], &p, &[], None, current_execution, false).unwrap();
            let mut returned = 0.;
            assert!(
                session
                    .solve(0.2, &mut returned, "bbd initial segment")
                    .unwrap()
                    >= 0
            );
            // SAFETY: the native state remains owned by this live session.
            let current = unsafe { read(session.y, 3) }.to_vec();
            assert!((current[0] - (-returned).exp()).abs() <= 0.01);
            assert!((current[1] - current[0]).abs() <= 0.001);
            assert!((current[2] - 2. * current[0]).abs() <= 0.001);
            session.callback.parameters[0] = 2.;
            session.callback.mode = 1;
            session.restart(returned, &current, &p, true).unwrap();
            assert_eq!(admission.charges.load(Ordering::Relaxed), 1);
            assert_eq!(
                admission.bytes.load(Ordering::Relaxed),
                p.idas.linear.bbd_bytes(3).unwrap()
            );
            let origin = returned;
            assert!(
                session
                    .solve(0.4, &mut returned, "bbd changed mode and parameter")
                    .unwrap()
                    >= 0
            );
            // SAFETY: as above, after this same current native owner returned.
            let final_state = unsafe { read(session.y, 3) };
            let expected = current[0] * (-4. * (returned - origin)).exp();
            assert!((final_state[0] - expected).abs() <= 0.01);
            assert!((final_state[1] - final_state[0]).abs() <= 0.001);
            assert!((final_state[2] - 2. * final_state[0]).abs() <= 0.001);
            let mut local_calls = 0;
            // SAFETY: BBD is installed on the live current session after restart.
            check(
                unsafe { ffi::IDABBDPrecGetNumGfnEvals(session.mem, &mut local_calls) },
                "BBD current call count",
            )
            .unwrap();
            assert!(local_calls > 0);
            assert!(actions.load(Ordering::Relaxed) > 1);
            let statistics = session.statistics(0, &p, Some(0.4));
            drop(session);
            drop(oracle);
            eprintln!(
                "bbd-native retained={retained} complete_ns={} source_calls={} exact_actions={} current_local_residuals={local_calls} statistics={statistics}",
                complete.elapsed().as_nanos(),
                calls.load(Ordering::Relaxed),
                actions.load(Ordering::Relaxed)
            );
        }
    }

    #[test]
    fn bbd_singular_retained_band_refuses_an_invertible_full_index_one_operator() {
        // The algebraic block [[0,1],[1,0]] is invertible, but its retained
        // diagonal is identically singular. Exact equations are unchanged.
        for selected in [IdasLinear::Klu, linear(0)] {
            let mut oracle = Coupled::new();
            oracle.allow_full = selected == IdasLinear::Klu;
            let p = profile(selected);
            p.validate(oracle.contract(), &[1.]).unwrap();
            let result = Session::new(&mut oracle, &[1.], &p, &[], None, execution(), false);
            if selected == IdasLinear::Klu {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err(), "singular band must not silently retry KLU");
            }
        }
    }

    #[test]
    fn bbd_preallocation_and_current_failure_stop_preserve_caller_output() {
        let mut oracle = Coupled::new();
        let calls = oracle.calls.clone();
        let p = profile(linear(1));
        for memory in [None, Some(1)] {
            let mut denied = execution();
            denied.memory = memory;
            assert!(matches!(
                Session::new(&mut oracle, &[1.], &p, &[], None, denied, false),
                Err(ProblemError::Limit {
                    kind: crate::LimitKind::Memory,
                    ..
                })
            ));
        }
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        let mut policy_denied = execution();
        policy_denied.work_admission = Some(Arc::new(StorageAdmission {
            deny: true,
            ..Default::default()
        }));
        assert!(Session::new(&mut oracle, &[1.], &p, &[], None, policy_denied, false).is_err());
        assert_eq!(
            calls.load(Ordering::Relaxed),
            0,
            "storage policy refusal precedes original source effects"
        );
        for source_failure in [false, true] {
            let mut oracle = Coupled::new();
            let failure = oracle.fail.clone();
            let mut session =
                Session::new(&mut oracle, &[1.], &p, &[], None, execution(), false).unwrap();
            let y = session.vector(&[1., 1., 2.]).unwrap();
            let yp = session.vector(&[-1., 0., 0.]).unwrap();
            let out = session.vector(&[777., 888., 999.]).unwrap();
            if source_failure {
                failure.store(true, Ordering::Release);
            } else {
                session.callback.callback.execution.cancellation.cancel();
            }
            assert_ne!(
                // SAFETY: this session owns each three-entry serial vector and its
                // active user-data slot throughout the synchronous callback.
                unsafe { local_residual(3, 0., y, yp, out, session.user_data()) },
                0
            );
            // SAFETY: the output remains owned by this current native session.
            assert_eq!(unsafe { read(out, 3) }, [777., 888., 999.]);
        }
    }

    #[test]
    fn bbd_admission_cannot_transfer_to_a_changed_configuration_or_extent() {
        for (selected, states) in [(linear(2), 3), (linear(1), 2)] {
            let owner = Arc::new(StorageAdmission::default());
            let mut current = execution();
            current.work_admission = Some(owner.clone());
            let admission = admit(linear(1), 3, current).unwrap();
            assert!(matches!(
                admission.into_execution(selected, states),
                Err(ProblemError::Internal(_))
            ));
            assert_eq!(owner.charges.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn bbd_reinitialization_preserves_current_memory_source_and_cancellation_refusals() {
        let p = profile(linear(1));
        for refusal in 0..3 {
            let mut oracle = Coupled::new();
            let failure = oracle.fail.clone();
            let owner = Arc::new(StorageAdmission::default());
            let mut current = execution();
            current.work_admission = Some(owner.clone());
            let mut session =
                Session::new(&mut oracle, &[1.], &p, &[], None, current, false).unwrap();
            match refusal {
                0 => session.callback.callback.execution.memory = Some(1),
                1 => failure.store(true, Ordering::Release),
                _ => session.callback.callback.execution.cancellation.cancel(),
            }
            let state = [1., 1., 2.];
            let error = session.restart(0., &state, &p, false).unwrap_err();
            match refusal {
                0 => assert!(matches!(
                    error,
                    ProblemError::Limit {
                        kind: crate::LimitKind::Memory,
                        ..
                    }
                )),
                1 => assert!(matches!(error, ProblemError::Numerical { .. })),
                _ => assert!(matches!(error, ProblemError::Cancelled)),
            }
            assert_eq!(owner.charges.load(Ordering::Relaxed), 1);
            assert_eq!(state, [1., 1., 2.]);
            // The failed native owner is destroyed here while its current source
            // and callback context remain alive; nothing can enter idle storage.
            drop(session);
        }
    }
}
