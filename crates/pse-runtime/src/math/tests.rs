// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use datafusion::execution::memory_pool::FairSpillPool;
use std::sync::atomic::{AtomicBool, Ordering};
pub(super) fn service() -> Arc<MathService> {
    service_and_cache().0
}
#[cfg(feature = "canonical-tests")]
pub(super) fn limited_service(bytes: usize) -> Arc<MathService> {
    service_in(bytes).0
}
fn service_and_cache() -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    service_in(512 << 20)
}
fn service_in(
    pool: usize,
) -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    service_with(pool, 1 << 20)
}
/// A service on a `pool`-byte pool whose native jobs admit `foreign` bytes of
/// library-owned memory.
fn service_with(
    pool: usize,
    foreign: usize,
) -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    service_with_policy(
        pool,
        MathPolicy {
            foreign_bytes: foreign,
            worker_bytes: 8 << 20,
            workspace_bytes: 16 << 20,
            ..MathPolicy::default()
        },
    )
}
pub(super) fn service_with_policy(
    pool: usize,
    policy: MathPolicy,
) -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_engine::resources::ReleaseNotifyingPool::new(
        Arc::new(FairSpillPool::new(pool)),
    ));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    let service = MathService::new(
        pool,
        Arc::new(tokio::sync::Semaphore::new(2)),
        2,
        policy,
        &native,
    );
    (service, native)
}
#[tokio::test]
async fn cancellation_before_entry_releases_admission_without_starting() {
    let s = service();
    let permit = s.cpu.clone().acquire_many_owned(2).await.unwrap();
    let control = FlightCancellation::default();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let baseline = s.pool.reserved();
    let mut job = Box::pin(s.job(2, 0, control.clone(), move |_| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    }));
    assert!(futures_util::poll!(job.as_mut()).is_pending());
    control.cancel();
    assert!(matches!(job.await, Err(MathRuntimeError::Cancelled)));
    assert!(!called.load(Ordering::SeqCst));
    assert_eq!(s.pool.reserved(), baseline);
    drop(permit);
}
#[tokio::test]
async fn scoped_preparation_deadline_includes_cpu_wait_and_late_join() {
    use pse_backend_native::{LimitKind, ProblemError};
    let s = service();
    let permit = s.cpu.clone().acquire_many_owned(2).await.unwrap();
    let control = FlightCancellation::default();
    let baseline = s.pool.reserved();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(20);
    let result = s
        .job_retained_scoped(2, 0, control.clone(), Some(deadline), move |_| {
            flag.store(true, Ordering::Release);
            Ok(((), 0))
        })
        .await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(ProblemError::Limit {
            kind: LimitKind::Time,
            ..
        }))
    ));
    assert!(!called.load(Ordering::Acquire));
    assert!(!control.flag().load(Ordering::Acquire));
    assert_eq!(s.pool.reserved(), baseline);
    drop(permit);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(20);
    let result = s
        .job_retained_scoped(1, 0, control.clone(), Some(deadline), move |_| {
            std::thread::sleep(std::time::Duration::from_millis(40));
            Ok((42, 1024))
        })
        .await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(ProblemError::Limit {
            kind: LimitKind::Time,
            ..
        }))
    ));
    // Postdispatch expiry signals native checkpoints and keeps the typed deadline
    // cause while the supervisor retains the native owners through actual join.
    assert!(control.flag().load(Ordering::Acquire));
    assert_eq!(s.pool.reserved(), baseline);
    assert_eq!(s.cpu.available_permits(), 2);
}
#[tokio::test]
async fn task_preparation_waiter_expires_without_renewing_or_cancelling_parent() {
    let service = service();
    let permit = service.cpu.clone().acquire_many_owned(2).await.unwrap();
    let driver = crate::CancelSource::new();
    let parent = Arc::new(AtomicBool::new(false));
    let scope = pse_kernels::ExecutionScope::new(
        parent.clone(),
        Some(std::time::Instant::now() + std::time::Duration::from_millis(25)),
    );
    let control = FlightCancellation::default();
    let called = Arc::new(AtomicBool::new(false));
    let observed = called.clone();
    let baseline = service.pool.reserved();
    let operation = service.job_retained(1, 0, control.clone(), move |_| {
        observed.store(true, Ordering::Release);
        Ok((42, 0))
    });
    let result = MathService::within_task(&scope, &driver, operation).await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
        ))
    ));
    assert!(!parent.load(Ordering::Acquire));
    assert!(!driver.token().is_cancelled());
    assert!(!called.load(Ordering::Acquire));
    // The detached owner releases admission after observing waiter departure.
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while service.jobs.available_permits() != service.policy.jobs {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(service.pool.reserved(), baseline);
    assert_eq!(service.cpu.available_permits(), 0);
    drop(permit);
}
#[derive(Debug)]
struct SlowDrop {
    entered: Arc<tokio::sync::Notify>,
    gate: Arc<(Mutex<bool>, std::sync::Condvar)>,
    thread: std::thread::ThreadId,
}
impl Drop for SlowDrop {
    fn drop(&mut self) {
        assert_eq!(self.thread, std::thread::current().id());
        self.entered.notify_one();
        let (lock, cv) = &*self.gate;
        let mut ready = lock.lock().unwrap();
        while !*ready {
            ready = cv.wait(ready).unwrap();
        }
    }
}
thread_local! {static TLS:std::cell::RefCell<Option<SlowDrop>>=const{std::cell::RefCell::new(None)};}
#[tokio::test]
async fn abandoned_caller_holds_pool_and_cpu_through_tls_destructor() {
    let s = service();
    let entered = Arc::new(tokio::sync::Notify::new());
    let gate = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
    let e = entered.clone();
    let g = gate.clone();
    let control = FlightCancellation::default();
    let baseline = s.pool.reserved();
    let mut job = Box::pin(s.job(2, 1024, control, move |_| {
        let non_send = std::rc::Rc::new(3);
        TLS.set(Some(SlowDrop {
            entered: e,
            gate: g,
            thread: std::thread::current().id(),
        }));
        Ok(*non_send)
    }));
    assert!(futures_util::poll!(job.as_mut()).is_pending());
    entered.notified().await;
    drop(job);
    assert_eq!(s.cpu.available_permits(), 0);
    assert!(s.pool.reserved() > baseline);
    assert_eq!(s.jobs.available_permits(), s.policy.jobs - 1);
    *gate.0.lock().unwrap() = true;
    gate.1.notify_one();
    while s.jobs.available_permits() != s.policy.jobs {
        tokio::task::yield_now().await;
    }
    assert_eq!(s.cpu.available_permits(), 2);
    assert_eq!(s.pool.reserved(), baseline);
}
#[tokio::test]
async fn retained_result_capacity_transfers_without_a_second_reservation() {
    let s = service();
    let pool = s.pool.clone();
    let bytes = 1 << 20;
    let active =
        bytes + s.policy.stack_bytes + s.policy.foreign_bytes + s.policy.inner_session_bytes;
    let (result, owner) = s
        .job_retained(1, bytes, FlightCancellation::default(), move |_| {
            assert_eq!(pool.reserved(), active);
            Ok((vec![3_u64; 16], 16 * size_of::<u64>()))
        })
        .await
        .unwrap();
    assert_eq!(result.len(), 16);
    assert_eq!(owner.size(), 128);
    assert_eq!(s.pool.reserved(), 128);
    drop(result);
    drop(owner);
    assert_eq!(s.pool.reserved(), 0);
}

#[tokio::test]
async fn retained_result_beyond_the_working_allowance_is_charged_at_its_extent() {
    let s = service();
    let (result, owner) = s
        .job_retained(1, 0, FlightCancellation::default(), move |_| {
            Ok((vec![3_u64; 1 << 20], 8 << 20))
        })
        .await
        .unwrap();
    assert_eq!(owner.size(), 8 << 20);
    assert_eq!(s.pool.reserved(), 8 << 20);
    drop((result, owner));
    assert_eq!(s.pool.reserved(), 0);
    // A product the pool cannot admit is refused after the work, as a pool limit.
    let refused = s
        .job_retained(1, 0, FlightCancellation::default(), move |_| {
            Ok(((), 1 << 30))
        })
        .await;
    assert!(matches!(refused, Err(MathRuntimeError::Pool(_))));
    assert_eq!(s.pool.reserved(), 0);
}

#[tokio::test]
async fn parallel_jobs_admit_library_team_stacks_and_release_them_after_join() {
    let service = service();
    let pool = service.pool.clone();
    let expected = 1024
        + 3 * service.policy.stack_bytes
        + service.policy.foreign_bytes
        + service.policy.inner_session_bytes;
    service
        .job(2, 1024, Default::default(), move |_| {
            assert_eq!(pool.reserved(), expected);
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(service.pool.reserved(), 0);
    assert_eq!(service.cpu.available_permits(), 2);
}

#[test]
fn worker_budget_releases_dropped_capacity() {
    // The budget refuses a worker beyond its capacity and releases a dropped one.
    let budget = WorkerBudget::new(100);
    let a = budget.charge(60).unwrap();
    assert!(matches!(
        budget.charge(60),
        Err(MathRuntimeError::Limit("worker storage"))
    ));
    drop(a);
    let b = budget.charge(60).unwrap();
    assert_eq!(budget.used(), 60);
    drop(b);
    assert_eq!(budget.used(), 0);
}

#[tokio::test]
async fn admission_pool_pressure_releases_cpu_and_uses_one_population_ticket() {
    let (s, _) = service_with_policy(
        256 << 20,
        MathPolicy {
            jobs: 1,
            ..Default::default()
        },
    );
    let held = MemoryConsumer::new("query-or-escaped-result").register(&s.pool);
    held.try_grow(256 << 20).unwrap();
    let called = Arc::new(AtomicBool::new(false));
    let entered = called.clone();
    let service = s.clone();
    let task = tokio::spawn(async move {
        service
            .job_retained(1, 0, FlightCancellation::default(), move |_| {
                entered.store(true, Ordering::Release);
                Ok((42, 0))
            })
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while s.jobs.available_permits() != 0 {
            tokio::task::yield_now().await;
        }
        // CPU acquisition/reservation runs in the detached supervisor.
        tokio::task::yield_now().await;
    })
    .await
    .unwrap();
    assert!(!called.load(Ordering::Acquire));
    assert_eq!(s.cpu.available_permits(), 2);
    let refused = s
        .job_retained(1, 0, FlightCancellation::default(), |_| Ok(((), 0)))
        .await;
    assert!(matches!(
        refused,
        Err(MathRuntimeError::Limit("native jobs"))
    ));
    held.free();
    let (answer, lease) = tokio::time::timeout(std::time::Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(answer, 42);
    drop(lease);
    assert_eq!(s.jobs.available_permits(), 1);
    assert_eq!(s.pool.reserved(), 0);
}
#[tokio::test]
async fn admission_only_deadline_expires_on_pressure_without_dispatch() {
    let (s, _) = service_with_policy(
        256 << 20,
        MathPolicy {
            admission_wait: std::time::Duration::from_millis(20),
            ..Default::default()
        },
    );
    let held = MemoryConsumer::new("retained-pressure").register(&s.pool);
    held.try_grow(256 << 20).unwrap();
    let called = Arc::new(AtomicBool::new(false));
    let entered = called.clone();
    let control = FlightCancellation::default();
    let result = s
        .job_retained(1, 0, control.clone(), move |_| {
            entered.store(true, Ordering::Release);
            Ok(((), 0))
        })
        .await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            }
        ))
    ));
    assert!(!called.load(Ordering::Acquire));
    assert!(!control.flag().load(Ordering::Acquire));
    assert_eq!(s.cpu.available_permits(), 2);
    assert_eq!(s.jobs.available_permits(), s.policy.jobs);
    assert_eq!(s.pool.reserved(), 256 << 20);
}
#[tokio::test]
async fn admission_only_clock_does_not_reclassify_completed_unscoped_work() {
    let (s, _) = service_with_policy(
        256 << 20,
        MathPolicy {
            admission_wait: std::time::Duration::from_millis(20),
            ..Default::default()
        },
    );
    let (answer, lease) = s
        .job_retained(1, 0, FlightCancellation::default(), |_| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            Ok((42, 0))
        })
        .await
        .unwrap();
    assert_eq!(answer, 42);
    drop(lease);
    assert_eq!(s.pool.reserved(), 0);
}
#[tokio::test]
async fn admission_cancellation_during_pressure_releases_ticket_without_dispatch() {
    let s = service();
    let held = MemoryConsumer::new("retained-pressure").register(&s.pool);
    held.try_grow(512 << 20).unwrap();
    let control = FlightCancellation::default();
    let called = Arc::new(AtomicBool::new(false));
    let entered = called.clone();
    let service = s.clone();
    let cancel = control.clone();
    let task = tokio::spawn(async move {
        service
            .job_retained(1, 0, cancel, move |_| {
                entered.store(true, Ordering::Release);
                Ok(((), 0))
            })
            .await
    });
    while s.jobs.available_permits() == s.policy.jobs {
        tokio::task::yield_now().await;
    }
    control.cancel();
    assert!(matches!(
        task.await.unwrap(),
        Err(MathRuntimeError::Cancelled)
    ));
    assert!(!called.load(Ordering::Acquire));
    assert_eq!(s.cpu.available_permits(), 2);
    assert_eq!(s.jobs.available_permits(), s.policy.jobs);
    assert_eq!(s.pool.reserved(), 512 << 20);
}
#[tokio::test]
async fn admission_oversized_extent_refuses_before_ticket_and_dispatch() {
    let s = service();
    let called = Arc::new(AtomicBool::new(false));
    let entered = called.clone();
    let result = s
        .job_retained(1, 512 << 20, FlightCancellation::default(), move |_| {
            entered.store(true, Ordering::Release);
            Ok(((), 0))
        })
        .await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Limit(
            "native entry exceeds deployment memory"
        ))
    ));
    assert!(!called.load(Ordering::Acquire));
    assert_eq!(s.jobs.available_permits(), s.policy.jobs);
    assert_eq!(s.pool.reserved(), 0);
}
#[tokio::test]
async fn term_diagnostics_admit_small_demand_under_generous_worker_capacity() {
    let (s, _) = service_with_policy(
        256 << 20,
        MathPolicy {
            worker_bytes: 16usize << 30,
            ..Default::default()
        },
    );
    let terms = BTreeMap::from([(pse_ids::SemanticId::NIL, vec![1.0, -1.0])]);
    let policy = pse_math::diagnostics::TermPolicy {
        zero: 1e-12,
        mismatch: 1e6,
        cancellation: 1e-8,
        maximum_terms: 32,
        combinations: 100_000,
        findings: 100_000,
    };
    let report = s
        .modeling_term_diagnostics(terms, policy, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(s.pool.reserved() < 1 << 20);
    drop(report);
    assert_eq!(s.pool.reserved(), 0);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn nested_provider_analysis_admits_known_source_below_default_workspace_capacity() {
    use pse_compiler::workspace::{CompilerContext, CompilerWorkspace, WorkspaceLimits};
    use pse_math::implicit::{Configuration, Options, Unknown};
    let (service, _cache) = service_with_policy(256 << 20, MathPolicy::default());
    assert_eq!(service.policy.workspace_bytes, 4usize << 30);
    assert_eq!(service.policy.worker_bytes, 8usize << 30);
    let rows = pse_authoring::language::parse(
        "package p {def Root {param p:Scalar=2;implicit a {var y:Scalar;eq ey:y==p+1;}realize ra on a using nested;}}",
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let context = CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),
        providers: BTreeMap::new(),
    };
    let mut workspace = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(rows, pse_modeling::PhysicalScope::default())
        .unwrap();
    let prepared = workspace
        .prepare_modeling_cancellable(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let source = prepared.implicit_order().unwrap().remove(0);
    let source_owner = service
        .reserve("test:nested-source", source.retained_bytes())
        .unwrap();
    let key = source.descriptor.spec().key();
    let configurations = BTreeMap::from([(
        source.residuals[0].id,
        Configuration::Fixed(
            vec![Unknown {
                id: source.unknowns[0],
                lower: 1.,
                upper: 8.,
            }],
            Options {
                start: vec![2.5],
                variable_nominals: vec![1.],
                variable_tolerance: vec![1e-9],
                residual_tolerance: vec![1e-9],
                iterations: 20,
                time_limit: std::time::Duration::from_secs(1),
                derivative_tolerance: 1e-9,
            },
        ),
    )]);
    let input = modeling::ModelingInner {
        admitted: source.with_owner(source_owner.clone()),
        configurations,
    };
    let accelerators = Arc::new(pse_math::implicit::accelerators::Accelerators::standard());
    let demands = BTreeMap::from([(key, pse_kernels::DerivativeOrder::First)]);
    let profile = pse_compiler::workspace::Profile {
        evaluation: pse_math::jets::EvaluationLimits {
            scratch_bytes: 4usize << 30,
            ..Default::default()
        },
        ..Default::default()
    };
    let baseline = service.pool.reserved();
    // The inherited operation clock can already have expired without cancelling
    // its driver. Refuse it before taking admission or publishing a factory.
    let expired = crate::CancelSource::new().with_deadline(Some(
        std::time::Instant::now() - std::time::Duration::from_secs(1),
    ));
    let result = service
        .modeling_inner_providers(
            vec![input.clone()],
            accelerators.clone(),
            BTreeMap::new(),
            demands.clone(),
            profile,
            &expired,
        )
        .await;
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            }
        ))
    ));
    assert!(!expired.token().is_cancelled());
    assert_eq!(service.pool.reserved(), baseline);
    assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    assert_eq!(service.cpu.available_permits(), service.cores);

    // Expiry while waiting for CPU capacity uses that same clock and drains its
    // admission ticket while preserving the source and the enclosing driver.
    let cpu = service
        .cpu
        .clone()
        .acquire_many_owned(service.cores as u32)
        .await
        .unwrap();
    let waiting = crate::CancelSource::new().with_deadline(Some(
        std::time::Instant::now() + std::time::Duration::from_millis(25),
    ));
    let mut pending = Box::pin(service.modeling_inner_providers(
        vec![input.clone()],
        accelerators.clone(),
        BTreeMap::new(),
        demands.clone(),
        profile,
        &waiting,
    ));
    assert!(futures_util::poll!(pending.as_mut()).is_pending());
    assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
    let result = tokio::time::timeout(std::time::Duration::from_secs(1), pending)
        .await
        .unwrap();
    assert!(matches!(
        result,
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            }
        ))
    ));
    assert!(!waiting.token().is_cancelled());
    assert_eq!(service.pool.reserved(), baseline);
    assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    assert_eq!(service.cpu.available_permits(), 0);
    drop(cpu);
    assert_eq!(service.cpu.available_permits(), service.cores);

    // A fresh operation can still construct and retain the same source after
    // both refusals; its returned registration owns only the factory extent.
    let registrations = service
        .modeling_inner_providers(
            vec![input.clone()],
            accelerators.clone(),
            BTreeMap::new(),
            demands.clone(),
            profile,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let factory = registrations[&key]
        .source::<pse_math::implicit::reconstruction::ReconstructionFactory>()
        .unwrap();
    assert_eq!(
        service.pool.reserved() - baseline,
        factory.retained_bytes().unwrap()
    );
    assert_eq!(
        registrations[&key].spec().derivatives,
        pse_kernels::DerivativeOrder::First
    );
    drop(registrations);
    assert_eq!(service.pool.reserved(), baseline);
    let cancelled = crate::CancelSource::new();
    cancelled.cancel();
    assert!(matches!(
        service
            .modeling_inner_providers(
                vec![input],
                accelerators,
                BTreeMap::new(),
                demands,
                profile,
                &cancelled
            )
            .await,
        Err(MathRuntimeError::Cancelled)
    ));
    assert_eq!(service.pool.reserved(), baseline);
    assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    assert_eq!(service.cpu.available_permits(), service.cores);
    drop(source_owner);
    assert_eq!(service.pool.reserved(), 0);
}

#[cfg(all(
    feature = "canonical-tests",
    feature = "solver-kinsol",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn nested_preparation_admits_small_sources_with_generous_worker_capacity() {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime_on(
        256 << 20,
        MathPolicy {
            worker_bytes: 16usize << 30,
            workspace_bytes: 16 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let rows = pse_authoring::language::parse(
        "package p {def Root {param p:Scalar=2;implicit a {var y:Scalar;eq ey:y==p+1;annotation start y(2.5);annotation bounds y(1,8);}realize ra on a using nested;}}",
        pse_ids::SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut solver = fixture::profile();
    solver.presolve = pse_backend_native::presolve::Policy::Off;
    let mut compiler = fixture::compiler_profile();
    // Preserve the original reference compiler's generous evaluation allowances
    // (packages/reference/conformance.toml), including its 4 GiB scratch ceiling.
    compiler.evaluation = pse_math::jets::EvaluationLimits {
        derivative_components: 1_000_000,
        operations: 100_000_000,
        scratch_bytes: 4usize << 30,
        provider_calls: 1_000_000,
    };
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::First,
            compiler,
            solver,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(prepared.model.case.compiled().plan.columns().len(), 1);
}

#[cfg(all(feature = "canonical-tests", feature = "solver-kinsol"))]
#[tokio::test]
async fn general_rebind_admits_small_projection_with_generous_worker_capacity() {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime_on(
        512 << 20,
        MathPolicy {
            worker_bytes: 16usize << 30,
            workspace_bytes: 16 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let rows = pse_authoring::language::parse(
        "package p {def Root {param p:Scalar=1;var x:Scalar;eq a:x*p==1;annotation start x(1);}}",
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::First,
            fixture::compiler_profile(),
            fixture::profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let p = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path == "p" || symbol.lineage.path.ends_with(".p"))
        .unwrap()
        .id;
    let mut changed = prepared.model.values.clone();
    changed.scalars.insert(p, 2.0);
    assert!(!prepared.model.case.compiled().values_match(&changed));
    let bound = prepared
        .model
        .case
        .compiled()
        .rebind_allocation_bound(&changed)
        .unwrap()
        .unwrap();
    assert!(bound < 512 << 20);
    let (limited, _) = service_with_policy(
        512 << 20,
        MathPolicy {
            worker_bytes: bound - 1,
            foreign_bytes: 1 << 20,
            workspace_bytes: 16 << 20,
            ..Default::default()
        },
    );
    assert!(matches!(
        limited
            .rebind(
                &prepared.model.case,
                changed.clone(),
                &crate::CancelSource::new()
            )
            .await,
        Err(MathRuntimeError::Limit("rebind construction capacity"))
    ));
    assert_eq!(limited.pool.reserved(), 0);
    assert_eq!(limited.cpu.available_permits(), 2);
    let math = runtime.shared.math();
    // Leave less construction capacity than the old workspace-sized entry, while
    // preserving the full original stack/session/foreign allowance. Support is
    // prepared here; actual numerical programs remain deferred artifact flights.
    let demand = prepared
        .model
        .case
        .compiled()
        .support_upgrade_allocation_bound(pse_kernels::DerivativeOrder::Second)
        .unwrap()
        .unwrap();
    assert!(demand < 8 << 20);
    let baseline_before_upgrade = math.pool.reserved();
    let free = math.policy.stack_bytes
        + math.policy.inner_session_bytes
        + math.policy.foreign_bytes
        + (8 << 20);
    let pressure = MemoryConsumer::new("test:source-upgrade-pressure").register(&math.pool);
    pressure
        .try_grow((512 << 20) - baseline_before_upgrade - free)
        .unwrap();
    let stronger = math
        .prepare_order(
            prepared.model.case.clone(),
            pse_kernels::DerivativeOrder::Second,
            FlightCancellation::default(),
        )
        .await
        .unwrap();
    assert_eq!(
        stronger.compiled().plan.order(),
        pse_kernels::DerivativeOrder::Second
    );
    let directional = math.prepare_directional_actions(stronger).await.unwrap();
    assert!(directional.compiled().plan.has_directional_actions());
    assert_eq!(
        directional.compiled().plan.structure().key(),
        prepared.model.case.compiled().plan.structure().key()
    );
    drop(directional);
    drop(pressure);
    let baseline = math.pool.reserved();
    let hold = || {
        let reservation = MemoryConsumer::new("test:rebind-pressure").register(&math.pool);
        reservation.try_grow((512 << 20) - baseline).unwrap();
        reservation
    };
    // Even unchanged sharing needs bounded metadata comparison. Pressure retains
    // one population ticket and releases CPU, then a pool release wakes admission.
    let pressure = hold();
    let driver = crate::CancelSource::new();
    let sharing = math.rebind(&prepared.model.case, prepared.model.values.clone(), &driver);
    tokio::pin!(sharing);
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), &mut sharing)
            .await
            .is_err()
    );
    assert_eq!(math.jobs.available_permits(), math.policy.jobs - 1);
    assert_eq!(math.cpu.available_permits(), math.cores);
    drop(pressure);
    let shared = sharing.await.unwrap();
    assert!(Arc::ptr_eq(
        &shared.compiled().plan,
        &prepared.model.case.compiled().plan
    ));
    drop(shared);
    assert_eq!(math.pool.reserved(), baseline);

    let pressure = hold();
    let driver = crate::CancelSource::new();
    let cancelled = math.rebind(&prepared.model.case, changed.clone(), &driver);
    tokio::pin!(cancelled);
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), &mut cancelled)
            .await
            .is_err()
    );
    driver.cancel();
    assert!(matches!(cancelled.await, Err(MathRuntimeError::Cancelled)));
    assert_eq!(math.jobs.available_permits(), math.policy.jobs);
    assert_eq!(math.cpu.available_permits(), math.cores);
    drop(pressure);
    assert_eq!(math.pool.reserved(), baseline);

    let pressure = hold();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(20);
    let scope = pse_kernels::ExecutionScope::new(Arc::new(AtomicBool::new(false)), Some(deadline));
    let refused = math
        .rebind_within(
            &prepared.model.case,
            changed.clone(),
            &crate::CancelSource::new(),
            scope,
        )
        .await;
    assert!(matches!(
        &refused,
        Err(MathRuntimeError::Solve(
            pse_backend_native::ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            } | pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
        ))
    ));
    drop(refused);
    assert_eq!(math.jobs.available_permits(), math.policy.jobs);
    assert_eq!(math.cpu.available_permits(), math.cores);
    drop(pressure);
    assert_eq!(math.pool.reserved(), baseline);

    let pressure = hold();
    let driver = crate::CancelSource::new();
    let rebuilding = math.rebind(&prepared.model.case, changed, &driver);
    tokio::pin!(rebuilding);
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), &mut rebuilding)
            .await
            .is_err()
    );
    drop(pressure);
    let rebound = rebuilding.await.unwrap();
    assert!(Arc::ptr_eq(
        &rebound.compiled().plan,
        &prepared.model.case.compiled().plan
    ));
    assert!(!pse_math::SharedAllocation::ptr_eq(
        &rebound.compiled().presolve,
        &prepared.model.case.compiled().presolve
    ));
}

/// Local compiler fixture: no canonical store or native solve participates.
pub(super) fn component_fixture(
    service: &MathService,
) -> (
    Preparation,
    pse_math::binding::CaseValues,
    pse_ids::SemanticId,
) {
    component_fixture_source(
        service,
        "package p {def Root {param p:Scalar=1;var x:Scalar;eq a:x*p==1;annotation start x(1);}}",
    )
}
pub(super) fn component_fixture_source(
    service: &MathService,
    source: &str,
) -> (
    Preparation,
    pse_math::binding::CaseValues,
    pse_ids::SemanticId,
) {
    pse_math::initialize().unwrap();
    let rows = pse_authoring::language::parse(
        source,
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let inputs = CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),
        providers: BTreeMap::new(),
    };
    let mut compiler = CompilerWorkspace::new(inputs, WorkspaceLimits::default()).unwrap();
    compiler
        .publish_modeling(rows, pse_modeling::PhysicalScope::default())
        .unwrap();
    let (_model, compiled, values, _) = compiler
        .prepare_modeling_case_cancellable(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            &Default::default(),
            pse_kernels::DerivativeOrder::First,
            Default::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let parameter = compiled.plan.structure().parameters()[0].id;
    let lease = service
        .reserve("test:component-fixture", compiled.retained_bytes())
        .unwrap();
    (
        service.own_preparation((compiled, lease)).unwrap(),
        values,
        parameter,
    )
}

#[tokio::test]
async fn latest_only_changed_rebinds_plateau_and_escaped_fields_keep_only_their_components() {
    let service = service();
    let (mut latest, mut values, parameter) = component_fixture(&service);
    let source = latest.clone();
    let mut plateau = None;
    for index in 0..24 {
        values.scalars.insert(parameter, 2.0 + index as f64);
        let previous = Arc::downgrade(&latest._owner);
        latest = service
            .rebind(&latest, values.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        if index > 0 {
            assert!(
                previous.upgrade().is_none(),
                "preceding complete binding remains reachable"
            );
        }
        // Compare the scientific projection with a fresh rebind from the original.
        let fresh = source
            .compiled()
            .rebind(&values, &Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert_eq!(
            latest.compiled().coefficient_values,
            fresh.coefficient_values
        );
        assert_eq!(latest.compiled().presolve.key, fresh.presolve.key);
        assert_eq!(
            latest.compiled().complete(&values).scalars,
            fresh.complete(&values).scalars
        );
        drop(fresh);
        if index == 1 {
            plateau = Some(service.pool.reserved());
        }
        if index > 1 {
            assert_eq!(service.pool.reserved(), plateau.unwrap());
        }
    }
    let stable = latest.structural_witness();
    drop(source);
    drop(latest);
    assert!(service.pool.reserved() > 0);
    drop(stable);
    assert_eq!(service.pool.reserved(), 0);
}

#[tokio::test]
async fn changed_unused_coefficients_do_not_pin_obsolete_bindings_through_reused_fields() {
    let (service, _) = service_with_policy(
        512 << 20,
        MathPolicy {
            worker_bytes: 16usize << 30,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    );
    let (mut latest, mut values, parameter) = component_fixture_source(
        &service,
        "package p {def Root {param p:Scalar=1;var x:Scalar;eq a:x==1;annotation start x(1);}}",
    );
    // The memoized executable survives every edit, but must retain only the plan
    // and programs rather than the first complete coefficient binding.
    drop(service.assemble(latest.clone()).await.unwrap());
    service.clear_program_cache();
    let mut held = Vec::new();
    for index in 0..12 {
        let old_snapshot = latest.compiled().coefficient_values.clone();
        let old_presolve = latest.compiled().presolve.clone();
        let old_derived = latest.compiled().derived.clone();
        let previous = Arc::downgrade(&latest._owner);
        values.scalars.insert(parameter, 2.0 + index as f64);
        latest = service
            .rebind(&latest, values.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(
            previous.upgrade().is_none(),
            "a reused value field retained the preceding binding"
        );
        assert!(pse_math::SharedAllocation::ptr_eq(
            &old_presolve,
            &latest.compiled().presolve
        ));
        assert!(pse_math::SharedAllocation::ptr_eq(
            &old_derived,
            &latest.compiled().derived
        ));
        assert!(!pse_math::SharedAllocation::ptr_eq(
            &old_snapshot,
            &latest.compiled().coefficient_values
        ));
        assert!(
            latest
                .compiled()
                .coefficient_values
                .iter()
                .any(|(id, bits)| *id == parameter && *bits == (2.0 + index as f64).to_bits())
        );
        held.push(old_snapshot);
    }
    let with_aliases = service.pool.reserved();
    drop(held);
    assert!(service.pool.reserved() < with_aliases);
    let plateau = service.pool.reserved();
    for index in 0..12 {
        values.scalars.insert(parameter, 20.0 + index as f64);
        latest = service
            .rebind(&latest, values.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(service.pool.reserved(), plateau);
    }
    drop(latest);
    assert_eq!(service.pool.reserved(), 0);
}
