// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use datafusion::execution::{cache::Cache, memory_pool::FairSpillPool};
use pse_compiler::{
    typed_math::Formal,
    workspace::{Case, Definition},
};
use pse_ids::ContentHash;
use pse_kernels::Port;
use pse_math::{binding::*, typed::BodyLimits};
use pse_model::generated::enums::ModelingVariableDomain;
use std::sync::atomic::Ordering;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn service() -> Arc<MathService> {
    service_and_cache().0
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
    let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(pool));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    let service = MathService::new(
        pool,
        Arc::new(tokio::sync::Semaphore::new(2)),
        2,
        MathPolicy {
            foreign_bytes: 1 << 20,
            worker_bytes: 8 << 20,
            workspace_bytes: 16 << 20,
            ..MathPolicy::default()
        },
        &native,
    );
    (service, native)
}
fn inputs() -> Inputs {
    let quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let q = pse_quantity::standard::ids::quantity("neutral");
    let unit = quantities.quantity_type(q).unwrap().canonical_unit;
    let port = Port {
        id: id(1),
        quantity: q,
        unit,
    };
    let d = Definition {
        sources: vec!["x*x".into()],
        formals: vec![Formal {
            path: "x".into(),
            quantity: q,
        }],
        domains: vec![],
        groups: vec![],
        providers: vec![],
        units: BTreeMap::from([("1".into(), unit)]),
        literals: BTreeMap::new(),
        limits: BodyLimits::default(),
    };
    let s = CaseStructure::new(
        vec![Variable {
            port: port.clone(),
            fixed: false,
            domain: ModelingVariableDomain::Continuous,
            lower: None,
            upper: None,
        }],
        vec![],
        vec![InstanceBinding {
            instance: id(3),
            body: ContentHash::from_bytes([0; 32]),
            slots: vec![SlotBinding::new(&port, &port, &quantities).unwrap()],
            contributions: vec![Contribution {
                output: 0,
                target: Target::Row(id(4)),
                scale: 1.,
            }],
        }],
        vec![Row {
            id: id(4),
            quantity: q,
            lower: 0.,
            upper: 0.,
        }],
        None,
        CaseLimits::default(),
    )
    .unwrap();
    Inputs {
        flows: BTreeMap::new(),
        quantities,
        preconditions: Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap()),
        definitions: BTreeMap::from([(id(2), d)]),
        domains: BTreeMap::new(),
        groups: BTreeMap::new(),
        providers: BTreeMap::new(),
        cases: BTreeMap::from([(
            id(5),
            Case {
                structure: Arc::new(s),
                definitions: BTreeMap::from([(id(3), id(2))]),
            },
        )]),
        values: BTreeMap::new(),
    }
}
fn profile() -> Profile {
    Profile {
        evaluation: pse_math::jets::EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        },
        ..Default::default()
    }
}
async fn prepared(s: &Arc<MathService>) -> Preparation {
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    s.prepare(
        w,
        id(5),
        DerivativeOrder::Second,
        profile(),
        false,
        &crate::CancelSource::new(),
    )
    .await
    .unwrap()
}
#[tokio::test]
async fn preparation_runs_inside_its_workspace_reservation() {
    // A pool holding one workspace, one native thread and the product admits the
    // preparation: the job does not charge the workspace allowance a second time.
    let policy = service().policy.clone();
    let pool = policy.workspace_bytes
        + policy.stack_bytes
        + policy.foreign_bytes
        + policy.inner_session_bytes
        + (policy.workspace_bytes / 2);
    let s = service_in(pool).0;
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let prepared = s
        .prepare(
            w,
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(s.pool.reserved() >= prepared.prepared.retained_bytes());
    drop(prepared);
    assert_eq!(s.pool.reserved(), 0);
}
#[tokio::test]
async fn prepared_products_share_capacity_and_survive_workspace_rotation() {
    let s = service();
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let a = s
        .prepare(
            w.clone(),
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let first = s.pool.reserved();
    let b = s
        .prepare(
            w.clone(),
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(Arc::ptr_eq(&a.owner, &b.owner));
    assert_eq!(s.pool.reserved(), first);
    assert!(first < 2 * s.policy.workspace_bytes);
    // A structural edit has a distinct immutable allocation and retained owner.
    let mut changed = inputs();
    changed.definitions.get_mut(&id(2)).unwrap().sources[0] = "x*x+x".into();
    s.publish(&w, changed).unwrap();
    let c = s
        .prepare(
            w.clone(),
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(!Arc::ptr_eq(&a.owner, &c.owner));
    let escaped = a.compiled().plan.clone();
    drop((w, a, b, c));
    assert!(s.pool.reserved() > 0);
    assert!(s.pool.reserved() < s.policy.workspace_bytes);
    drop(escaped);
    assert_eq!(s.pool.reserved(), 0);
}
#[tokio::test]
async fn storage_invalidation_preserves_programs_and_owner_attachment_shares_payload() {
    let (s, storage) = service_and_cache();
    let prepared = prepared(&s).await;
    let original = prepared.compiled().plan.bodies().values().next().unwrap();
    let attached_plan = prepared
        .compiled()
        .plan
        .as_ref()
        .clone()
        .with_owner(Arc::new(()));
    let attached = attached_plan.bodies().values().next().unwrap();
    assert!(std::ptr::eq(original.support(), attached.support()));
    assert_eq!(
        original.output_quantities().as_ptr(),
        attached.output_quantities().as_ptr()
    );
    let request = prepared.compiled().artifacts[0].clone();
    let program = s.artifact(request.clone()).await.unwrap();
    storage.invalidate();
    assert_eq!(s.entries.len(), 1);
    assert!(Arc::ptr_eq(&program, &s.artifact(request).await.unwrap()));
    s.clear_program_cache();
    assert_eq!(s.entries.len(), 0);
    assert!(s.live.load(Ordering::Acquire) > 0);
    drop(program);
    assert_eq!(s.live.load(Ordering::Acquire), 0);
}

#[tokio::test]
async fn actual_artifact_singleflight_profiles_epoch_and_retained_owners() {
    let s = service();
    let p = prepared(&s).await;
    let request = p.prepared.artifacts[0].clone();
    let baseline = s.pool.reserved();
    let (a, b) = tokio::join!(s.artifact(request.clone()), s.artifact(request.clone()));
    let a = a.unwrap();
    let b = b.unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(s.entries.len(), 1);
    assert!(a.lease.size() < s.policy.foreign_bytes + profile().evaluation.scratch_bytes);
    let retained = a.program.clone();
    drop(a);
    drop(b);
    s.invalidate();
    assert!(s.pool.reserved() > baseline);
    assert!(s.live.load(Ordering::Acquire) > 0);
    drop(retained);
    assert_eq!(s.live.load(Ordering::Acquire), 0);
    assert_eq!(s.pool.reserved(), baseline);
    // Fence an actual build while it waits for CPU, then invalidate before it can complete.
    let permit = s.cpu.clone().acquire_many_owned(2).await.unwrap();
    let mut load = Box::pin(s.artifact(request));
    assert!(futures_util::poll!(load.as_mut()).is_pending());
    while s.jobs.available_permits() == s.policy.jobs {
        tokio::task::yield_now().await;
    }
    s.invalidate();
    drop(permit);
    let value = load.await.unwrap();
    assert_eq!(s.entries.len(), 0);
    drop(value);
    let mut opts = profile();
    opts.optimization.cores = 2;
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let changed = s
        .prepare(
            w,
            id(5),
            DerivativeOrder::Second,
            opts,
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_ne!(
        changed.prepared.artifacts[0].key(),
        p.prepared.artifacts[0].key()
    );
    assert!(
        s.artifact(changed.prepared.artifacts[0].clone())
            .await
            .is_ok()
    );
}
#[tokio::test]
async fn independent_workspaces_use_one_pool_and_worker_thread() {
    let s = service();
    let (a, b) = tokio::join!(prepared(&s), prepared(&s));
    assert_eq!(a.prepared.artifacts, b.prepared.artifacts);
    let (a, b) = tokio::join!(s.assemble(a), s.assemble(b));
    let a = a.unwrap();
    let b = b.unwrap();
    assert!(Arc::ptr_eq(&a._artifacts[0], &b._artifacts[0]));
    let caller = std::thread::current().id();
    let (value, retained_body) = s
        .with_worker(
            a,
            BTreeMap::new(),
            &crate::CancelSource::new(),
            move |worker| {
                assert_ne!(std::thread::current().id(), caller);
                let local = std::rc::Rc::new(4.0);
                let values = worker
                    .constraints(&CaseValues {
                        scalars: BTreeMap::from([(id(1), *local)]),
                    })
                    .map_err(MathRuntimeError::Math)?;
                let body = worker.assembly().bodies().values().next().unwrap().clone();
                Ok((values, body))
            },
        )
        .await
        .unwrap();
    assert_eq!(value, vec![16.]);
    drop(b);
    s.invalidate();
    assert!(s.pool.reserved() > 0);
    drop(retained_body);
    assert_eq!(s.pool.reserved(), 0);
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
async fn actual_compile_failure_is_retryable_and_admission_is_finite() {
    let s = service();
    let mut i = inputs();
    i.definitions.get_mut(&id(2)).unwrap().sources[0] = "abs(x)".into();
    let w = s.workspace(i, WorkspaceLimits::default()).unwrap();
    let p = s
        .prepare(
            w,
            id(5),
            DerivativeOrder::Second,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(s.artifact(p.prepared.artifacts[0].clone()).await.is_err());
    assert!(s.artifact(p.prepared.artifacts[0].clone()).await.is_err());
    assert_eq!(s.flights.active(), 0);
    assert_eq!(s.entries.len(), 0);
    assert!(matches!(
        s.job(3, 0, FlightCancellation::default(), |_| Ok(())).await,
        Err(MathRuntimeError::Limit(_))
    ));
    let cancelled = crate::CancelSource::new();
    cancelled.cancel();
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    assert!(
        s.prepare(
            w,
            id(5),
            DerivativeOrder::Second,
            profile(),
            false,
            &cancelled
        )
        .await
        .is_err()
    );
}

#[cfg(feature = "native-solvers")]
#[tokio::test]
async fn native_staged_recycle_and_singular_block_preserve_original_values() {
    use super::initialization::InitializationProfile;
    use pse_backend_native::solve::{SolveIntent, Termination};
    for singular in [false, true] {
        let s = service();
        let mut input = inputs();
        let q = pse_quantity::standard::ids::quantity("neutral");
        let unit = input.quantities.quantity_type(q).unwrap().canonical_unit;
        let x = Port {
            id: id(1),
            quantity: q,
            unit,
        };
        let y = Port {
            id: id(6),
            quantity: q,
            unit,
        };
        let d = input.definitions.get_mut(&id(2)).unwrap();
        d.formals.push(Formal {
            path: "y".into(),
            quantity: q,
        });
        d.sources = if singular {
            vec!["x+y".into(), "2*x+2*y".into()]
        } else {
            vec!["x-0.5*y".into(), "y-0.25*x".into()]
        };
        input.cases.get_mut(&id(5)).unwrap().structure = Arc::new(
            CaseStructure::new(
                vec![x.clone(), y.clone()]
                    .into_iter()
                    .map(|port| Variable {
                        port,
                        fixed: false,
                        domain: ModelingVariableDomain::Continuous,
                        lower: None,
                        upper: None,
                    })
                    .collect(),
                vec![],
                vec![InstanceBinding {
                    instance: id(3),
                    body: ContentHash::from_bytes([0; 32]),
                    slots: vec![
                        SlotBinding::new(&x, &x, &input.quantities).unwrap(),
                        SlotBinding::new(&y, &y, &input.quantities).unwrap(),
                    ],
                    contributions: vec![
                        Contribution {
                            output: 0,
                            target: Target::Row(id(4)),
                            scale: 1.,
                        },
                        Contribution {
                            output: 1,
                            target: Target::Row(id(7)),
                            scale: 1.,
                        },
                    ],
                }],
                vec![
                    Row {
                        id: id(4),
                        quantity: q,
                        lower: 1.,
                        upper: 1.,
                    },
                    Row {
                        id: id(7),
                        quantity: q,
                        lower: if singular { 3. } else { 1. },
                        upper: if singular { 3. } else { 1. },
                    },
                ],
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let workspace = s
            .workspace(input.clone(), WorkspaceLimits::default())
            .unwrap();
        let prepared = s
            .prepare_initialization(workspace, input, id(5), profile(), DerivativeOrder::First)
            .await
            .unwrap();
        assert_eq!(
            prepared.boundaries().count(),
            1,
            "both recycle coordinates belong to the coupled block"
        );
        let initial = CaseValues {
            scalars: BTreeMap::from([(id(1), 1000.), (id(6), -100.)]),
        };
        let result = s
            .initialize(
                prepared,
                initial.clone(),
                BTreeMap::new(),
                InitializationProfile {
                    solver: solves::SolverProfile {
                        intent: SolveIntent::Initialize,
                        ..Default::default()
                    },
                    stages: vec![BTreeMap::new(), BTreeMap::new()],
                },
            )
            .unwrap()
            .finish()
            .await
            .unwrap();
        if singular {
            assert_eq!(result.completed_stages, 0);
            assert_eq!(result.original.scalars, initial.scalars);
            assert!(
                result.values.scalars.is_empty(),
                "failed trials must not commit solved unknowns"
            );
            assert!(!result.attempts[0].committed);
            if let Ok(report) = &result.attempts[0].result {
                assert!(!matches!(
                    report.termination.category,
                    Termination::Success | Termination::Acceptable
                ));
            }
        } else {
            assert_eq!(result.completed_stages, 2, "{result:?}");
            assert!(result.attempts.iter().all(|attempt| attempt.committed));
            assert!((result.values.scalars[&id(1)] - 12. / 7.).abs() < 1e-6);
            assert!((result.values.scalars[&id(6)] - 10. / 7.).abs() < 1e-6);
        }
    }
}

#[tokio::test]
async fn constant_sequence_uses_shared_lifecycle_and_retains_result_allowance() {
    use super::solves::*;
    use pse_backend_native::{execution::BackendSettings, solve::*};
    let s = service();
    let mut i = inputs();
    i.values.insert(id(1), 0.0);
    let c = i.cases.get_mut(&id(5)).unwrap();
    let old = &c.structure;
    let mut variables = old.variables().to_vec();
    variables[0].fixed = true;
    c.structure = Arc::new(
        CaseStructure::new(
            variables,
            old.parameters().to_vec(),
            old.instances().to_vec(),
            old.rows().to_vec(),
            old.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let w = s.workspace(i, WorkspaceLimits::default()).unwrap();
    let p = s
        .prepare(
            w,
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let p = s
        .prepare_solve(
            p,
            CaseValues {
                scalars: BTreeMap::from([(id(1), 0.0)]),
            },
            BTreeMap::new(),
            SolverProfile {
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                intent: SolveIntent::Root,
                selection: SolverSelection::Auto,
                controls: Controls::default(),
                backend: BackendSettings::Default,
                sensitivity: None,
            },
            NumericalInputs::default(),
        )
        .await
        .unwrap();
    assert_eq!(p.route(), pse_backend_native::routing::Route::Constant);
    let report = s.solve(p).unwrap().finish().await.unwrap();
    assert!(matches!(&report.outcome,Outcome::Constant(c)if c.quality.feasible()));
    s.invalidate();
    assert!(s.pool.reserved() > 0);
    let retained = report.outcome.clone();
    drop(report);
    assert!(s.pool.reserved() > 0);
    drop(retained);
    assert_eq!(s.pool.reserved(), 0);
    assert_eq!(s.cpu.available_permits(), 2);
}
#[tokio::test]
async fn solver_profile_refuses_mismatched_backend_before_artifact_construction() {
    use super::solves::*;
    use pse_backend_native::{execution::BackendSettings, solve::*};
    let s = service();
    let p = prepared(&s).await;
    let before = s.entries.len();
    let result = s
        .prepare_solve(
            p,
            CaseValues {
                scalars: BTreeMap::from([(id(1), 1.0)]),
            },
            BTreeMap::new(),
            SolverProfile {
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                intent: SolveIntent::Root,
                selection: SolverSelection::Explicit(Backend::Clarabel),
                controls: Controls::default(),
                backend: BackendSettings::Default,
                sensitivity: None,
            },
            NumericalInputs::default(),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(s.entries.len(), before);
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn initialization_prepares_conditional_programs_and_rejects_invalid_schedules() {
    use super::initialization::*;
    use pse_backend_native::solve::*;
    let s = service();
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let p = s
        .prepare_initialization(w, inputs(), id(5), profile(), DerivativeOrder::First)
        .await
        .unwrap();
    assert_eq!(p.boundaries().count(), 1);
    let profile = InitializationProfile {
        solver: solves::SolverProfile {
            intent: SolveIntent::Initialize,
            ..Default::default()
        },
        stages: vec![BTreeMap::from([(id(1), 2.0)])],
    };
    assert!(
        s.initialize(
            p,
            CaseValues {
                scalars: BTreeMap::from([(id(1), 1.0)])
            },
            BTreeMap::new(),
            profile
        )
        .is_err()
    );
}
/// F26: the initialization admission rules live in Rust `validate_profile`, which every
/// caller reaches before native work; typed backend settings are route-typed.
#[cfg(feature = "native-solvers")]
#[tokio::test]
async fn initialization_admission_in_rust() {
    use super::{initialization::*, solves::SolverProfile};
    use pse_backend_native::{ProblemError, execution::BackendSettings, presolve, solve::*};
    let s = service();
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let p = s
        .prepare_initialization(w, inputs(), id(5), profile(), DerivativeOrder::First)
        .await
        .unwrap();
    let values = CaseValues {
        scalars: BTreeMap::from([(id(1), 1.0)]),
    };
    let admit = |solver: SolverProfile| {
        p.validate_profile(
            &values,
            &InitializationProfile {
                solver,
                stages: vec![BTreeMap::new()],
            },
        )
        .map(|(routes, _)| routes)
    };
    let refused = |solver: SolverProfile| match admit(solver) {
        Err(MathRuntimeError::Solve(ProblemError::Contract(_))) => true,
        other => panic!("expected a typed contract refusal: {other:?}"),
    };
    let initialize = SolverProfile {
        intent: SolveIntent::Initialize,
        ..Default::default()
    };
    // The square free block routes to KINSOL under both root purposes.
    let kinsol_route = vec![pse_backend_native::routing::Route::Native(Backend::Kinsol)];
    assert_eq!(admit(initialize.clone()).unwrap(), kinsol_route);
    assert!(
        admit(SolverProfile {
            intent: SolveIntent::Root,
            ..initialize.clone()
        })
        .is_ok()
    );
    // The rules the Python adapter used to hold are refused here, before native work.
    assert!(refused(SolverProfile {
        intent: SolveIntent::Optimize,
        ..initialize.clone()
    }));
    let explicit = presolve::Policy::new(
        presolve::PolicyKind::Explicit,
        &Options::new(),
        Default::default(),
    )
    .unwrap();
    assert!(refused(SolverProfile {
        presolve: explicit,
        ..initialize.clone()
    }));
    assert!(refused(SolverProfile {
        convexity: pse_math::convexity::ConvexityPolicy::Numerical {
            absolute: 1e-9,
            relative: 1e-9,
        },
        ..initialize.clone()
    }));
    // Route-typed settings: KINSOL's method reaches the KINSOL block, and a KINSOL
    // method on another route (here SCIP, eligible for this first-order block), or another
    // backend's settings on the KINSOL route, is refused, never applied.
    let kinsol = serde_json::from_value::<BackendSettings>(
        serde_json::json!({"backend": "kinsol", "strategy": "newton"}),
    )
    .unwrap();
    assert_eq!(
        admit(SolverProfile {
            backend: kinsol.clone(),
            ..initialize.clone()
        })
        .unwrap(),
        kinsol_route
    );
    assert!(refused(SolverProfile {
        backend: kinsol,
        selection: SolverSelection::Explicit(Backend::Scip),
        ..initialize.clone()
    }));
    let highs =
        serde_json::from_value::<BackendSettings>(serde_json::json!({"backend": "highs"})).unwrap();
    assert!(refused(SolverProfile {
        backend: highs,
        ..initialize
    }));
}

#[tokio::test]
async fn flow_selection_uses_shared_lifecycle_and_owns_extracted_witness() {
    use super::flows::*;
    use pse_backend_native::solve::Controls;
    use pse_structural::flowsheet::{Declaration, Node};
    let service = service();
    let mut source = inputs();
    source.flows.insert(
        id(70),
        Declaration {
            nodes: vec![Node {
                id: id(71),
                ports: vec![],
            }],
            connections: vec![],
            decisions: vec![],
        },
    );
    let workspace = service
        .workspace(source.clone(), WorkspaceLimits::default())
        .unwrap();
    let mut other_revision = source.clone();
    other_revision.flows.get_mut(&id(70)).unwrap().nodes[0].id = id(72);
    service.publish(&workspace, other_revision).unwrap();
    let flow = service
        .prepare_flow(workspace, source, id(70))
        .await
        .unwrap();
    let result = service
        .select_tears(flow, TearMethod::UnweightedHeuristic, Controls::default())
        .unwrap()
        .finish()
        .await
        .unwrap();
    assert!(result.attempt.is_none());
    let witness = result.selected.as_ref().unwrap().clone();
    assert_eq!(witness.order, vec![id(71)]);
    service.invalidate();
    drop(result);
    assert!(service.pool.reserved() > 0);
    drop(witness);
    assert_eq!(service.pool.reserved(), 0);
    assert_eq!(service.cpu.available_permits(), 2);
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

/// A provider factory recording every cancellation handle it is constructed with.
#[derive(Debug)]
struct Recording {
    spec: pse_kernels::ProviderSpec,
    scoped: Mutex<Vec<Arc<AtomicBool>>>,
}
#[derive(Debug)]
struct Inert(pse_kernels::ProviderSpec);
impl Provider for Inert {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.0
    }
    fn evaluate(
        &mut self,
        _: &[f64],
        _: &pse_kernels::ProviderRequest,
        _: &pse_kernels::EvaluationContext<'_>,
    ) -> Result<pse_kernels::ProviderValues, pse_kernels::ProviderError> {
        Err(pse_kernels::ProviderError::Trial("not evaluated".into()))
    }
}
impl pse_kernels::ProviderFactory for Recording {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<Box<dyn Provider>, pse_kernels::ProviderError> {
        Ok(Box::new(Inert(self.spec.clone())))
    }
    fn create_scoped(
        &self,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Provider>, pse_kernels::ProviderError> {
        self.scoped.lock().unwrap().push(cancel);
        Ok(Box::new(Inert(self.spec.clone())))
    }
}
fn recording() -> (
    Arc<Recording>,
    BTreeMap<ProviderKey, pse_kernels::Registration>,
) {
    let i = inputs();
    let port = i.cases[&id(5)].structure.variables()[0].port.clone();
    let hash = ContentHash::from_bytes([0; 32]);
    let factory = Arc::new(Recording {
        spec: pse_kernels::ProviderSpec {
            shapes: pse_kernels::ProviderShapes::default(),
            derivative_source: pse_kernels::DerivativeSource::Analytic,
            id: id(80),
            revision: hash,
            data: hash,
            inputs: vec![port.clone()],
            outputs: vec![port],
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        },
        scoped: Default::default(),
    });
    let registration = pse_kernels::Registration::new(factory.clone(), &i.quantities).unwrap();
    let registrations = BTreeMap::from([(registration.spec().key(), registration)]);
    (factory, registrations)
}

#[tokio::test]
async fn nested_worker_observes_attempt_cancel() {
    let s = service();
    let case = s.assemble(prepared(&s).await).await.unwrap();
    let (factory, registrations) = recording();
    // Admission constructs one worker, on a private flag, to check the factory product.
    let admitted = factory.scoped.lock().unwrap().len();
    let flag = Arc::new(AtomicBool::new(false));
    let budget = WorkerBudget::new(s.policy.worker_bytes);
    let worker = s
        .worker(case.clone(), &registrations, flag.clone(), &budget)
        .unwrap();
    // The evaluator's providers are attached to the attempt's cancellation, never a
    // private flag of their own.
    let scoped = factory.scoped.lock().unwrap()[admitted..].to_vec();
    assert_eq!(scoped.len(), 1);
    assert!(Arc::ptr_eq(&scoped[0], &flag));
    flag.store(true, Ordering::Release);
    assert!(scoped[0].load(Ordering::Acquire));
    assert_eq!(budget.used(), case.assembly.numeric_worker_bytes());
    drop(worker);
    assert_eq!(budget.used(), 0);
}

#[tokio::test]
async fn session_step_reserves_per_worker() {
    use super::solves::*;
    use pse_backend_native::{execution::BackendSettings, solve::*};
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

    let s = service();
    let case = s.assemble(prepared(&s).await).await.unwrap();
    let (_, registrations) = recording();
    let mut i = inputs();
    i.values.insert(id(1), 0.0);
    let c = i.cases.get_mut(&id(5)).unwrap();
    let mut variables = c.structure.variables().to_vec();
    variables[0].fixed = true;
    c.structure = Arc::new(
        CaseStructure::new(
            variables,
            c.structure.parameters().to_vec(),
            c.structure.instances().to_vec(),
            c.structure.rows().to_vec(),
            c.structure.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let w = s.workspace(i, WorkspaceLimits::default()).unwrap();
    let p = s
        .prepare(
            w,
            id(5),
            DerivativeOrder::First,
            profile(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let step = s
        .prepare_solve(
            p,
            CaseValues {
                scalars: BTreeMap::from([(id(1), 0.0)]),
            },
            BTreeMap::new(),
            SolverProfile {
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                intent: SolveIntent::Root,
                selection: SolverSelection::Auto,
                controls: Controls::default(),
                backend: BackendSettings::Default,
                sensitivity: None,
            },
            NumericalInputs::default(),
        )
        .await
        .unwrap();
    let session = s.open_session().unwrap();
    let owner = s
        .reserve("test:results", step.result_bytes().unwrap())
        .unwrap();
    let assessor = s.clone();
    let (_, seen) = session
        .step(
            step,
            None,
            0,
            Arc::new(Progress::new(0)),
            owner,
            &crate::CancelSource::new(),
            move |_, cancel, budget| {
                let idle = assessor.pool.reserved();
                // The step's assessment builds two evaluators at once on the session worker.
                let first = assessor
                    .worker(case.clone(), &registrations, cancel.clone(), budget)
                    .unwrap();
                let second = assessor
                    .worker(case.clone(), &registrations, cancel.clone(), budget)
                    .unwrap();
                let both = budget.used();
                // No further evaluator fits once the session's worker share is spent.
                let overflow = budget.charge(budget.capacity() - both + 1).is_err();
                let reserved = assessor.pool.reserved();
                drop((first, second));
                let seen = (
                    budget.capacity(),
                    both,
                    case.assembly.numeric_worker_bytes(),
                    reserved,
                    reserved - idle,
                    reserved - assessor.pool.reserved(),
                    overflow,
                );
                (seen, true)
            },
        )
        .await
        .unwrap();
    session.close().await;
    let (capacity, both, worker, reserved, drawn, released, overflow) = seen;
    // Every evaluator the step builds is charged to the session's worker share and drawn
    // from the pool while it lives; the share is a ceiling, not a standing reservation.
    assert_eq!(capacity, s.policy.worker_bytes);
    assert_eq!(both, 2 * worker);
    assert!(reserved >= both + s.policy.stack_bytes + s.policy.foreign_bytes);
    assert_eq!(drawn, both);
    assert_eq!(released, both);
    assert!(overflow);
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn resolved_accuracy_not_user_input() {
    use super::solves::*;
    use pse_backend_native::{execution::BackendSettings, solve::*};
    // User controls carry no accuracy, so nothing a caller supplies can stand in for the
    // resolved value or be mistaken for an explicit default.
    let fields = serde_json::to_value(Controls::default()).unwrap();
    assert!(fields.get("accuracy").is_none(), "{fields}");
    let s = service();
    let prepare = |numerics: pse_model::numerics::NumericalPolicy| {
        let s = s.clone();
        async move {
            let p = prepared(&s).await;
            s.prepare_solve(
                p,
                CaseValues {
                    scalars: BTreeMap::from([(id(1), 1.0)]),
                },
                BTreeMap::new(),
                SolverProfile {
                    presolve: Default::default(),
                    numerics,
                    convexity: Default::default(),
                    intent: SolveIntent::Root,
                    selection: SolverSelection::Auto,
                    controls: Controls::default(),
                    backend: BackendSettings::Default,
                    sensitivity: None,
                },
                NumericalInputs::default(),
            )
            .await
            .unwrap()
        }
    };
    let default = prepare(Default::default()).await;
    let mut policy = pse_model::numerics::NumericalPolicy::default();
    policy.kkt.stationarity = 1e-6;
    policy.kkt.complementarity = 1e-6;
    let loose = prepare(policy.clone()).await;
    // The budgets are the numerical policy's, resolved at preparation.
    let nominal = pse_model::numerics::NumericalPolicy::default();
    assert_eq!(default.accuracy().stationarity, nominal.kkt.stationarity);
    assert_eq!(loose.accuracy().stationarity, policy.kkt.stationarity);
    assert_eq!(loose.accuracy().complementarity, policy.kkt.complementarity);
    assert_eq!(loose.accuracy().feasibility, default.accuracy().feasibility);
    let expected = ResolvedAccuracy::resolve(
        &nominal,
        default.tolerances(),
        &pse_math::normalization::Normalization::from_policy(
            default.numerics(),
            &[id(1)],
            &[id(4)],
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(default.accuracy(), &expected);
}
/// A solve's declared foreign allowance replaces the deployment's in its own reservation
/// only; a solve without one keeps the deployment's, and a zero allowance is refused.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn solve_reservation_charges_its_declared_foreign_allowance() {
    use super::solves::*;
    use pse_backend_native::{execution::BackendSettings, solve::*};
    let s = service();
    let deployment = s.policy.foreign_bytes;
    let declared = 32 << 20;
    let p = prepared(&s).await;
    let prepare = |foreign_bytes| {
        let (s, p) = (s.clone(), p.clone());
        async move {
            s.prepare_solve(
                p,
                CaseValues {
                    scalars: BTreeMap::from([(id(1), 1.0)]),
                },
                BTreeMap::new(),
                SolverProfile {
                    intent: SolveIntent::Root,
                    controls: Controls {
                        foreign_bytes,
                        ..Controls::default()
                    },
                    backend: BackendSettings::Default,
                    ..SolverProfile::default()
                },
                NumericalInputs::default(),
            )
            .await
        }
    };
    // Programs are assembled once per structure; later solves share them.
    drop(prepare(None).await.unwrap());
    let before = s.pool.reserved();
    let open = prepare(None).await.unwrap();
    let open_bytes = s.pool.reserved() - before;
    let scoped = prepare(Some(declared)).await.unwrap();
    let scoped_bytes = s.pool.reserved() - before - open_bytes;
    assert_eq!(scoped_bytes - open_bytes, declared - deployment);
    assert_eq!(s.policy.foreign_allowance(&Controls::default()), deployment);
    // The allowance is a per-attempt budget: the request names it, and the retained native
    // session a sequence may reuse does not.
    assert_ne!(
        open.preparation_identity().unwrap(),
        scoped.preparation_identity().unwrap()
    );
    assert_eq!(
        open.compatibility().map(|c| c.profile),
        scoped.compatibility().map(|c| c.profile)
    );
    drop((open, scoped));
    assert_eq!(s.pool.reserved(), before);
    assert!(prepare(Some(0)).await.is_err());
}
/// min y*y - 3y + x  s.t.  y - x*x = 0, x in `x_box`, y in [0, 4]: the quartic
/// x^4 - 3x^2 + x, whose global minimum is near x = -1.3008.
#[cfg(all(feature = "solver-scip", feature = "solver-ipopt"))]
fn quartic_inputs(x_box: (Option<f64>, Option<f64>)) -> Inputs {
    let mut i = inputs();
    let q = pse_quantity::standard::ids::quantity("neutral");
    let unit = i.quantities.quantity_type(q).unwrap().canonical_unit;
    let port = |n| Port {
        id: id(n),
        quantity: q,
        unit,
    };
    let d = i.definitions.get_mut(&id(2)).unwrap();
    d.sources = vec!["y - x*x".into(), "y*y - y - y - y + x".into()];
    d.formals.push(Formal {
        path: "y".into(),
        quantity: q,
    });
    let s = CaseStructure::new(
        vec![
            Variable {
                port: port(1),
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: x_box.0,
                upper: x_box.1,
            },
            Variable {
                port: port(6),
                fixed: false,
                domain: ModelingVariableDomain::Continuous,
                lower: Some(0.0),
                upper: Some(4.0),
            },
        ],
        vec![],
        vec![InstanceBinding {
            instance: id(3),
            body: ContentHash::from_bytes([0; 32]),
            slots: vec![
                SlotBinding::new(&port(1), &port(1), &i.quantities).unwrap(),
                SlotBinding::new(&port(6), &port(6), &i.quantities).unwrap(),
            ],
            contributions: vec![
                Contribution {
                    output: 0,
                    target: Target::Row(id(4)),
                    scale: 1.,
                },
                Contribution {
                    output: 1,
                    target: Target::PRIMARY,
                    scale: 1.,
                },
            ],
        }],
        vec![Row {
            id: id(4),
            quantity: q,
            lower: 0.,
            upper: 0.,
        }],
        Some(Objective {
            quantity: q,
            sense: ObjectiveSense::Minimize,
        }),
        CaseLimits::default(),
    )
    .unwrap();
    i.cases.get_mut(&id(5)).unwrap().structure = Arc::new(s);
    i
}
#[cfg(all(feature = "solver-scip", feature = "solver-ipopt"))]
#[tokio::test]
async fn certify_intent_projects_and_solves_through_the_factorable_route() {
    use super::solves::*;
    use pse_backend_native::{
        execution::BackendSettings,
        routing::Route,
        solve::{Assurance, Backend, Controls, Qualification, SolveIntent, SolverSelection},
    };
    use pse_model::generated::enums::CandidateUse;
    let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(1 << 30));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    // SCIP's memory limit is the job's foreign allowance.
    let s = MathService::new(
        pool,
        Arc::new(tokio::sync::Semaphore::new(2)),
        2,
        MathPolicy {
            foreign_bytes: 128 << 20,
            worker_bytes: 8 << 20,
            workspace_bytes: 16 << 20,
            ..MathPolicy::default()
        },
        &native,
    );
    let prepare = |x_box| {
        let s = s.clone();
        async move {
            let w = s
                .workspace(quartic_inputs(x_box), WorkspaceLimits::default())
                .unwrap();
            let p = s
                .prepare(
                    w,
                    id(5),
                    DerivativeOrder::Second,
                    profile(),
                    false,
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            s.prepare_solve(
                p,
                CaseValues {
                    scalars: BTreeMap::from([(id(1), 1.0), (id(6), 1.0)]),
                },
                BTreeMap::new(),
                SolverProfile {
                    presolve: Default::default(),
                    numerics: Default::default(),
                    convexity: Default::default(),
                    intent: SolveIntent::Certify,
                    selection: SolverSelection::Auto,
                    controls: Controls::default(),
                    backend: BackendSettings::Default,
                    sensitivity: None,
                },
                NumericalInputs::default(),
            )
            .await
        }
    };
    // An unbounded variable in a nonlinear term is refused before any worker exists.
    let refused = prepare((None, Some(2.0))).await.unwrap_err();
    assert!(
        refused.to_string().contains("spatial branching"),
        "{refused}"
    );
    // Certification selects the certifying record and projects the case lazily; the
    // single solve is a one-step native session (A6).
    let p = prepare((Some(-2.0), Some(2.0))).await.unwrap();
    assert_eq!(p.route(), Route::Native(Backend::Scip));
    let report = s.solve(p).unwrap().finish().await.unwrap();
    let outcome = &report.outcome;
    let Outcome::Native(r) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(r.backend, Backend::Scip);
    let x = r.candidate.as_ref().unwrap().primal[0];
    assert!((x + 1.300_839).abs() < 1e-3, "{x}");
    assert_eq!(r.qualification, Qualification::GapQualified);
    assert_eq!(r.termination.assurance, Assurance::GlobalBound);
    assert_eq!(outcome.candidate_use().usability, CandidateUse::Usable);
}
