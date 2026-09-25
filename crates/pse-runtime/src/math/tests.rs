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
use std::sync::atomic::Ordering;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn service() -> Arc<MathService> {
    let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(512 << 20));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    MathService::new(
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
    )
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
            domain: VariableDomain::Continuous,
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
    use pse_backend_native::{
        kinsol::Linear,
        solve::{Controls, Termination},
    };
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
                        domain: VariableDomain::Continuous,
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
        let workspace = s.workspace(input, WorkspaceLimits::default()).unwrap();
        let prepared = s
            .prepare_initialization(workspace, id(5), profile())
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
                    controls: Controls::default(),
                    linear: Linear::Klu,
                    variable_tolerances: BTreeMap::from([(id(1), 1e-7), (id(6), 1e-7)]),
                    row_tolerances: BTreeMap::from([(id(4), 1e-7), (id(7), 1e-7)]),
                    stages: vec![BTreeMap::new(), BTreeMap::new()],
                },
            )
            .unwrap()
            .finish()
            .await
            .unwrap();
        if singular {
            assert_eq!(result.completed_stages, 0);
            assert_eq!(
                result.values.scalars, initial.scalars,
                "failed trials must not commit"
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
    use pse_backend_native::{quality::Tolerances, solve::*};
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
                scaling: None,
                intent: SolveIntent::Root,
                selection: SolverSelection::Auto,
                controls: Controls::default(),
                backend: BackendSettings::Default,
                tolerances: Tolerances {
                    variables: vec![],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(p.route(), pse_backend_native::routing::Route::Constant);
    let report = s
        .solve(SolveSequence {
            steps: vec![p],
            continue_independent: false,
            result_limit: 1,
        })
        .unwrap()
        .finish()
        .await
        .unwrap();
    assert_eq!(report.unattempted, 0);
    assert!(matches!(&report.outcomes[0],Outcome::Constant(c)if c.quality.feasible()));
    s.invalidate();
    assert!(s.pool.reserved() > 0);
    let retained = report.outcomes[0].clone();
    drop(report);
    assert!(s.pool.reserved() > 0);
    drop(retained);
    assert_eq!(s.pool.reserved(), 0);
    assert_eq!(s.cpu.available_permits(), 2);
}
#[tokio::test]
async fn solver_profile_refuses_mismatched_backend_before_artifact_construction() {
    use super::solves::*;
    use pse_backend_native::{quality::Tolerances, solve::*};
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
                scaling: None,
                intent: SolveIntent::Root,
                selection: SolverSelection::Explicit(Backend::Clarabel),
                controls: Controls::default(),
                backend: BackendSettings::Default,
                tolerances: Tolerances {
                    variables: vec![1e-8],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
            },
            None,
        )
        .await;
    assert!(result.is_err());
    assert_eq!(s.entries.len(), before);
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn initialization_prepares_conditional_programs_and_rejects_invalid_schedules() {
    use super::initialization::*;
    use pse_backend_native::{kinsol, solve::*};
    let s = service();
    let w = s.workspace(inputs(), WorkspaceLimits::default()).unwrap();
    let p = s.prepare_initialization(w, id(5), profile()).await.unwrap();
    assert_eq!(p.boundaries().count(), 1);
    let profile = InitializationProfile {
        controls: Controls::default(),
        linear: kinsol::Linear::Klu,
        variable_tolerances: BTreeMap::from([(id(1), 1e-8)]),
        row_tolerances: BTreeMap::from([(id(4), 1e-8)]),
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
        .workspace(source, WorkspaceLimits::default())
        .unwrap();
    let flow = service.prepare_flow(workspace, id(70)).await.unwrap();
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
