// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned process exclusion and probe allocation controls, independent of solver size.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "focused controls assert ownership transitions"
)]
use super::*;
use crate::execution::{ComputeAdmission, ComputeAvailability, compute_scoped};
use crate::solve::{WorkAdmission, WorkEvidence};
use std::{
    any::Any,
    sync::{atomic::AtomicUsize, mpsc},
};

fn defaults() -> crate::mkl::NestedDefaults {
    crate::mkl::NestedDefaults {
        omp: 1,
        mkl: 1,
        all: 1,
        blas: 1,
        lapack: 1,
        pardiso: 1,
        cancellation: true,
        proc_bind: 1,
    }
}
fn execution() -> Execution {
    let mut execution = Execution::new(
        pse_diagnostics::Cancellation::default(),
        &Controls {
            time_limit: Duration::from_secs(5),
            ..Default::default()
        },
    );
    execution.memory = Some(256 << 20);
    execution
}

#[test]
fn copied_watch_context_attributes_progress_and_restores_foreign_worker_context() {
    use pse_diagnostics::observation::{ObservationHub, Policy, Value, current};
    let hub = ObservationHub::new(Policy::default());
    let source = hub.start("scip-captured-attempt");
    let unrelated = hub.start("unrelated-worker-task");
    let captured = source.context();
    let other = unrelated.context();
    std::thread::spawn(move || {
        assert!(current().is_none());
        let watch = Watch {
            observation_context: Some(captured.clone()),
            cancel: Arc::default(),
            progress: Arc::new(Progress::new(0)),
            started: Instant::now(),
            events: AtomicU64::new(0),
            interrupted: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            objective: None,
        };
        for unrelated in [false, true] {
            let _other = unrelated.then(|| other.enter());
            let previous = current().map(|context| context.id());
            // This is the scope entered by both native event callbacks, including
            // a copied handler on a library-created thread without caller TLS.
            let code = contained(|| {
                let _observation = watch.observation_scope();
                assert_eq!(current().unwrap().id(), captured.id());
                watch.progress.push(Event {
                    source: Some(crate::solve::EventSource::new(
                        "branch_and_bound",
                        Some("original_source"),
                        None,
                    )),
                    phase: "scip.incumbent".into(),
                    elapsed: Duration::ZERO,
                    values: BTreeMap::from([("worker_probe".into(), Metric::Integer(7))]),
                    incumbent: None,
                });
                ffi::SCIP_Retcode_SCIP_OKAY
            });
            assert_eq!(code, ffi::SCIP_Retcode_SCIP_OKAY);
            assert_eq!(current().map(|context| context.id()), previous);
        }
        assert!(current().is_none());
    })
    .join()
    .unwrap();
    let snapshot = source.reader().snapshot(Duration::from_secs(60));
    let producer = snapshot
        .producers
        .iter()
        .find(|producer| {
            producer
                .current
                .as_ref()
                .is_some_and(|record| record.family == "branch_and_bound.v1")
        })
        .unwrap();
    assert_eq!(producer.coverage.accepted, 2);
    let record = producer.current.as_ref().unwrap();
    assert_eq!(record.sequence, 2);
    assert_eq!(
        record
            .values
            .iter()
            .find(|field| field.name == "worker_probe")
            .unwrap()
            .value,
        Value::Integer(7)
    );
    assert!(
        unrelated
            .reader()
            .snapshot(Duration::from_secs(60))
            .metrics
            .is_empty()
    );
}

#[test]
fn selected_plugin_statistics_respect_stage_presence_and_optional_timing() {
    let execution = execution();
    let instance = Instance::new(&execution, Modes::default()).unwrap();
    let mut metrics = BTreeMap::new();
    selected_plugin_statistics(&instance, &mut metrics);
    assert_eq!(
        metrics["scip.plugins.selected"],
        Metric::Unavailable(UnavailableReason::NotApplicable)
    );
    assert!(!metrics.contains_key("scip.plugin.presolve.dualinfer.calls"));
    native!("SCIPsolve", ffi::SCIPsolve(instance.ptr())).unwrap();
    metrics.clear();
    selected_plugin_statistics(&instance, &mut metrics);
    assert!(matches!(metrics["scip.plugins.selected"], Metric::Text(_)));
    assert!(matches!(
        metrics["scip.plugins.reset_on_initialization"],
        Metric::Bool(_)
    ));
    for prefix in [
        "scip.plugin.presolve.dualinfer",
        "scip.plugin.propagation.pseudoobj",
        "scip.plugin.separation.gomory",
        "scip.plugin.constraint.nonlinear",
    ] {
        assert_eq!(metrics[&format!("{prefix}.present")], Metric::Bool(true));
    }
    for name in [
        "scip.plugin.presolve.dualinfer.calls",
        "scip.plugin.propagation.pseudoobj.calls",
        "scip.plugin.separation.gomory.calls",
        "scip.plugin.constraint.nonlinear.presolve_calls",
    ] {
        assert!(matches!(metrics[name], Metric::Integer(value) if value >= 0));
    }
    if metrics["scip.plugin.nlpi.ipopt.present"] == Metric::Bool(true) {
        assert_eq!(
            metrics["scip.plugin.nlpi.ipopt.evaluation_timing_enabled"],
            Metric::Bool(false)
        );
        assert_eq!(
            metrics["scip.plugin.nlpi.ipopt.evaluation_seconds"],
            Metric::Unavailable(UnavailableReason::NotRequested)
        );
        assert!(
            matches!(metrics["scip.plugin.nlpi.ipopt.solves"], Metric::Integer(value) if value >= 0)
        );
    } else {
        assert_eq!(
            metrics["scip.plugin.nlpi.ipopt"],
            Metric::Unavailable(UnavailableReason::NotApplicable)
        );
        assert!(!metrics.contains_key("scip.plugin.nlpi.ipopt.solves"));
    }
}

#[test]
fn nested_defaults_refuse_domain_overrides_and_nonserial_openmp_lists() {
    let serial = defaults();
    for list in [None, Some("1"), Some("1, 1")] {
        serial.admit(list, true).unwrap();
    }
    for list in ["2", "1,2", "", "off", "0", "1,"] {
        assert!(matches!(
            serial.admit(Some(list), false),
            Err(ProblemError::Unsupported(_))
        ));
    }
    for field in 0..6 {
        let mut changed = serial;
        match field {
            0 => changed.omp = 2,
            1 => changed.mkl = 2,
            2 => changed.all = 2,
            3 => changed.blas = 2,
            4 => changed.lapack = 2,
            _ => changed.pardiso = 2,
        }
        assert!(changed.admit(Some("1"), false).is_err(), "{changed:?}");
    }
    let mut spral = serial;
    spral.cancellation = false;
    assert!(spral.admit(None, true).is_err());
    spral.admit(None, false).unwrap();
    spral.cancellation = true;
    spral.proc_bind = 0;
    assert!(spral.admit(None, true).is_err());
}

#[derive(Debug)]
struct Compute {
    available: Arc<AtomicUsize>,
    attempts: Arc<AtomicUsize>,
    paused: Option<mpsc::Sender<()>>,
    held: bool,
}
impl Drop for Compute {
    fn drop(&mut self) {
        if self.held {
            self.available.fetch_add(1, Ordering::AcqRel);
        }
    }
}
impl ComputeAdmission for Compute {
    fn pause(&mut self) {
        if self.held {
            self.held = false;
            self.available.fetch_add(1, Ordering::AcqRel);
        }
        if let Some(paused) = self.paused.take() {
            paused.send(()).unwrap();
        }
    }
    fn try_resume(&mut self, execution: &Execution) -> Result<ComputeAvailability, ProblemError> {
        execution.check()?;
        self.attempts.fetch_add(1, Ordering::AcqRel);
        if self.held
            || self
                .available
                .compare_exchange(1, 0, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            self.held = true;
            Ok(ComputeAvailability::Acquired)
        } else {
            Ok(ComputeAvailability::Unavailable)
        }
    }
    fn resume(&mut self, execution: &Execution) -> Result<(), ProblemError> {
        loop {
            if self.try_resume(execution)? == ComputeAvailability::Acquired {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[test]
fn concurrent_gate_saturation_releases_tentative_gate_before_capacity_wait() {
    let lock = Arc::new(Mutex::new(()));
    let holder = lock.lock().unwrap();
    let available = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::new(AtomicUsize::new(0));
    let (paused, pause) = mpsc::channel();
    let (entered, entry) = mpsc::channel();
    let (release, drain) = mpsc::channel();
    let worker_lock = lock.clone();
    let worker_available = available.clone();
    let worker_attempts = attempts.clone();
    let worker = std::thread::spawn(move || {
        let execution = execution();
        compute_scoped(
            Box::new(Compute {
                available: worker_available,
                attempts: worker_attempts,
                paused: Some(paused),
                held: true,
            }),
            || {
                let failed = AtomicBool::new(false);
                let owner =
                    ConcurrentAdmission::acquire(&worker_lock, &failed, &execution).unwrap();
                entered.send(()).unwrap();
                drain.recv_timeout(Duration::from_secs(2)).unwrap();
                drop(owner);
            },
        );
    });
    pause.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(
        available.compare_exchange(1, 0, Ordering::AcqRel, Ordering::Acquire),
        Ok(1)
    );
    drop(holder);
    let deadline = Instant::now() + Duration::from_secs(2);
    while attempts.load(Ordering::Acquire) == 0 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    // Another active team can enter the native gate while this waiter has no CPU.
    drop(lock.lock().unwrap());
    assert!(entry.try_recv().is_err());
    available.store(1, Ordering::Release);
    entry.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(lock.try_lock(), Err(TryLockError::WouldBlock)));
    release.send(()).unwrap();
    worker.join().unwrap();
    drop(lock.lock().unwrap());
    assert_eq!(available.load(Ordering::Acquire), 1);
}

#[test]
fn concurrent_gate_cancel_preserves_parent_charge_and_never_claims_compute() {
    let lock = Arc::new(Mutex::new(()));
    let holder = lock.lock().unwrap();
    let available = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::new(AtomicUsize::new(0));
    let (paused, pause) = mpsc::channel();
    let mut execution = execution();
    let cancel = Arc::new(AtomicBool::new(false));
    execution.cancel = cancel.clone();
    let worker_lock = lock.clone();
    let worker_available = available.clone();
    let worker_attempts = attempts.clone();
    let worker = std::thread::spawn(move || {
        compute_scoped(
            Box::new(Compute {
                available: worker_available,
                attempts: worker_attempts,
                paused: Some(paused),
                held: true,
            }),
            || {
                assert!(matches!(
                    ConcurrentAdmission::acquire(&worker_lock, &AtomicBool::new(false), &execution),
                    Err(ProblemError::Cancelled)
                ))
            },
        );
    });
    pause.recv_timeout(Duration::from_secs(2)).unwrap();
    cancel.cancel();
    worker.join().unwrap();
    assert_eq!(available.load(Ordering::Acquire), 1);
    assert_eq!(attempts.load(Ordering::Acquire), 0);
    drop(holder);
}

#[derive(Debug, Default)]
struct Storage {
    live: AtomicUsize,
    requested: AtomicUsize,
    denied: AtomicBool,
}
#[derive(Debug, Default)]
struct Admission(Arc<Storage>);
struct Grant(Arc<Storage>);
impl Drop for Grant {
    fn drop(&mut self) {
        self.0.live.fetch_sub(DEFAULT_PROBE_STACK, Ordering::AcqRel);
    }
}
impl WorkAdmission for Admission {
    fn admit(&self, _: WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn observe(&self, _: WorkEvidence) -> Result<(), ProblemError> {
        Ok(())
    }
    fn reserve_retained_storage(
        &self,
        owner: &str,
        bytes: usize,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, ProblemError> {
        assert_eq!(owner, "scip-nested-default-probe");
        assert_eq!(bytes, DEFAULT_PROBE_STACK);
        self.0.requested.fetch_add(bytes, Ordering::AcqRel);
        if self.0.denied.load(Ordering::Acquire) {
            return Ok(None);
        }
        self.0.live.fetch_add(bytes, Ordering::AcqRel);
        Ok(Some(Arc::new(Grant(self.0.clone()))))
    }
}

#[test]
fn probe_cancellation_joins_actual_thread_before_releasing_separate_stack_charge() {
    let mut execution = execution();
    let admission = Arc::new(Admission::default());
    execution.work_admission = Some(admission.clone());
    let cancel = execution.cancellation.clone();
    let (started, start) = mpsc::channel();
    let (finish, finished) = mpsc::channel();
    let state = admission.0.clone();
    let worker = std::thread::spawn(move || {
        probe_defaults_with(&execution, move |_| {
            assert_eq!(state.live.load(Ordering::Acquire), DEFAULT_PROBE_STACK);
            started.send(()).unwrap();
            finished.recv_timeout(Duration::from_secs(2)).unwrap();
            Ok(defaults())
        })
    });
    start.recv_timeout(Duration::from_secs(2)).unwrap();
    cancel.cancel();
    assert_eq!(
        admission.0.live.load(Ordering::Acquire),
        DEFAULT_PROBE_STACK
    );
    assert!(
        !worker.is_finished(),
        "caller cancellation must not detach the probe"
    );
    finish.send(()).unwrap();
    assert!(matches!(
        worker.join().unwrap(),
        Err(ProblemError::Cancelled)
    ));
    assert_eq!(admission.0.live.load(Ordering::Acquire), 0);
    assert_eq!(
        admission.0.requested.load(Ordering::Acquire),
        DEFAULT_PROBE_STACK
    );
}

#[test]
fn probe_refuses_denied_storage_before_creating_a_thread() {
    let mut execution = execution();
    let admission = Arc::new(Admission::default());
    admission.0.denied.store(true, Ordering::Release);
    execution.work_admission = Some(admission.clone());
    let called = AtomicBool::new(false);
    assert!(
        probe_defaults_with(&execution, |_| {
            called.store(true, Ordering::Release);
            Ok(defaults())
        })
        .is_err()
    );
    assert!(!called.load(Ordering::Acquire));
    assert_eq!(admission.0.live.load(Ordering::Acquire), 0);
}

#[test]
fn probe_observes_fresh_thread_defaults_without_parent_local_settings() {
    let execution = execution();
    let baseline = probe_nested_defaults(&execution).unwrap();
    let _local = crate::mkl::Threads::enter(3).unwrap();
    let observed = probe_nested_defaults(&execution).unwrap();
    assert_eq!(observed.defaults, baseline.defaults);
    assert_eq!(observed.omp_list, baseline.omp_list);
}

#[test]
fn concurrent_gate_survives_instance_native_destruction() {
    static INSTANCE_GATE: Mutex<()> = Mutex::new(());
    static INSTANCE_FAILED: AtomicBool = AtomicBool::new(false);
    let execution = execution();
    let mut instance = Instance::new(&execution, Modes::default()).unwrap();
    instance._concurrent =
        Some(ConcurrentAdmission::acquire(&INSTANCE_GATE, &INSTANCE_FAILED, &execution).unwrap());
    assert!(matches!(
        INSTANCE_GATE.try_lock(),
        Err(TryLockError::WouldBlock)
    ));
    drop(instance);
    assert!(INSTANCE_GATE.try_lock().is_ok());
    assert!(!CONCURRENT_ACTIVE.get());
}

#[test]
fn injected_free_failure_exposes_typed_cause_and_quarantines_only_uncertain_compute() {
    static IDLE_GATE: Mutex<()> = Mutex::new(());
    static IDLE_FAILED: AtomicBool = AtomicBool::new(false);
    static UNCERTAIN_GATE: Mutex<()> = Mutex::new(());
    static UNCERTAIN_FAILED: AtomicBool = AtomicBool::new(false);
    for (uncertain, gate, failed) in [
        (false, &IDLE_GATE, &IDLE_FAILED),
        (true, &UNCERTAIN_GATE, &UNCERTAIN_FAILED),
    ] {
        let execution = execution();
        let admission = Admission::default();
        let mut instance = Instance::new(&execution, Modes::default()).unwrap();
        instance._storage_owner = admission
            .reserve_retained_storage("scip-nested-default-probe", DEFAULT_PROBE_STACK)
            .unwrap();
        let available = Arc::new(AtomicUsize::new(0));
        let mut quarantined = None;
        compute_scoped(
            Box::new(Compute {
                available: available.clone(),
                attempts: Arc::default(),
                paused: None,
                held: true,
            }),
            || {
                instance._concurrent =
                    Some(ConcurrentAdmission::acquire(gate, failed, &execution).unwrap());
                instance.concurrent_may_be_active = uncertain;
                let result = instance.shutdown_with(
                    |raw| {
                        // Destroy a real idle fixture, then inject the returned failure. No
                        // uncertain native jobs are run; this exercises the ownership branch.
                        // SAFETY: raw points to this fixture's live, idle SCIP instance;
                        // shutdown invokes this closure once and relinquishes that instance.
                        let status = unsafe { ffi::SCIPfree(raw) };
                        assert_eq!(status, ffi::SCIP_Retcode_SCIP_OKAY);
                        ffi::SCIP_Retcode_SCIP_ERROR
                    },
                    |owner| quarantined = Some(owner),
                );
                assert!(
                    matches!(result, Err(ProblemError::Native { kind: crate::NativeFailureKind::Infrastructure, ref status, .. }) if status.code == i64::from(ffi::SCIP_Retcode_SCIP_ERROR))
                );
            },
        );
        let owner = quarantined.take().unwrap();
        assert_eq!(owner._compute.is_some(), uncertain);
        assert_eq!(available.load(Ordering::Acquire), usize::from(!uncertain));
        assert_eq!(
            admission.0.live.load(Ordering::Acquire),
            DEFAULT_PROBE_STACK
        );
        assert!(matches!(gate.try_lock(), Err(TryLockError::WouldBlock)));
        assert!(matches!(
            check_concurrent_health(failed),
            Err(ProblemError::Internal(_))
        ));
        assert!(
            matches!(
                instance.shutdown(),
                Err(ProblemError::Native {
                    kind: crate::NativeFailureKind::Infrastructure,
                    ..
                })
            ),
            "failed shutdown remains typed and never retries partial native free"
        );
        // The injected fixture was actually freed, so this test can release its
        // quarantined owners. Production uncertainty deliberately never does so.
        // SAFETY: the fixture's successful SCIPfree above ended all uses of the
        // uniquely allocated watch. Quarantine retained its pointer without freeing it.
        drop(unsafe { Box::from_raw(owner._watch.as_ptr()) });
        drop(owner);
        assert_eq!(available.load(Ordering::Acquire), 1);
        assert_eq!(admission.0.live.load(Ordering::Acquire), 0);
        assert!(gate.try_lock().is_ok());
    }
}
