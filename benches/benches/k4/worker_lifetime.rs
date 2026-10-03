// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Benchmark-local gates witness public worker abandonment through actual TLS teardown.
use super::*;
use pse_runtime::math::{MathRuntimeError, MathService};
use std::{cell::RefCell, sync::mpsc};

struct Teardown {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    release: mpsc::Receiver<()>,
}
impl Drop for Teardown {
    fn drop(&mut self) {
        let _ = self.entered.take().unwrap().send(());
        self.release.recv_timeout(Duration::from_secs(30)).unwrap();
    }
}
thread_local! {
    static TEARDOWN: RefCell<Option<Teardown>> = const { RefCell::new(None) };
}
fn occupied_job_slots(service: &MathService) -> usize {
    service
        .execution_report()
        .iter()
        .find(|(name, _)| *name == "math_native_active")
        .unwrap()
        .1
        .unwrap()
}

pub(super) async fn run(owner: &WorkflowRuntime, phases: &phases::Phases) -> (Duration, Value) {
    let (workspace, revision, root, context) = demand::definition(owner);
    let service = owner.runtime.math();
    let cancel = CancelSource::new();
    let instance = pse_modeling::specialize::root_instance(root);
    let model = service
        .prepare_modeling_revision(
            workspace.clone(),
            revision.clone(),
            root,
            instance,
            pse_modeling::Bindings {
                demand: vec!["x".into()],
                ..Default::default()
            },
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let row = model.compiled().model.equations.first().unwrap().id;
    let x = model.compiled().model.paths["x"];
    let executable = service
        .prepare_modeling_functions(
            workspace.clone(),
            model.clone(),
            vec![row],
            vec![x],
            pse_kernels::DerivativeOrder::Value,
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let pool = owner.runtime.pool();
    let baseline = pool.reserved();
    phases.reset();
    let started = Instant::now();
    let (entered_tx, mut entered_rx) = tokio::sync::oneshot::channel();
    let (closure_release_tx, closure_release_rx) = mpsc::channel();
    let (teardown_tx, teardown_rx) = tokio::sync::oneshot::channel();
    let (teardown_release_tx, teardown_release_rx) = mpsc::channel();
    let mut abandoned = Box::pin(service.with_worker(
        executable.clone(),
        BTreeMap::new(),
        &cancel,
        move |_worker| {
            TEARDOWN.with(|owner| {
                *owner.borrow_mut() = Some(Teardown {
                    entered: Some(teardown_tx),
                    release: teardown_release_rx,
                })
            });
            let _ = entered_tx.send(());
            closure_release_rx
                .recv_timeout(Duration::from_secs(30))
                .unwrap();
            Ok(())
        },
    ));
    tokio::select! {
        result = &mut abandoned => panic!("worker completed before its entry gate: {result:?}"),
        result = &mut entered_rx => result.unwrap(),
    }
    let entered_pool = pool.reserved();
    assert!(entered_pool > baseline);
    assert_eq!(occupied_job_slots(service), 1);
    drop(abandoned);
    let dropped_pool = pool.reserved();
    assert_eq!(
        dropped_pool, entered_pool,
        "abandonment does not release running ownership"
    );
    let refusal = service
        .with_worker(
            executable.clone(),
            BTreeMap::new(),
            &CancelSource::new(),
            |_| -> Result<(), MathRuntimeError> {
                panic!("single-worker admission entered before first closure returned")
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(refusal, MathRuntimeError::Limit("native jobs")));
    closure_release_tx.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(30), teardown_rx)
        .await
        .unwrap()
        .unwrap();
    let teardown_pool = pool.reserved();
    assert_eq!(
        teardown_pool, entered_pool,
        "native TLS teardown retains the entire reservation"
    );
    assert_eq!(occupied_job_slots(service), 1);
    let teardown_refusal = service
        .with_worker(
            executable.clone(),
            BTreeMap::new(),
            &CancelSource::new(),
            |_| -> Result<(), MathRuntimeError> {
                panic!("single-worker admission entered during TLS teardown")
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(
        teardown_refusal,
        MathRuntimeError::Limit("native jobs")
    ));
    teardown_release_tx.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(30), async {
        while occupied_job_slots(service) != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let drained_pool = pool.reserved();
    assert_eq!(
        drained_pool, baseline,
        "join releases first attempt reservations"
    );
    // This independent public call reacquires after drain. It is not an internal queue.
    let (second_tx, mut second_rx) = tokio::sync::oneshot::channel();
    let (second_release_tx, second_release_rx) = mpsc::channel();
    let second_cancel = CancelSource::new();
    let mut second = Box::pin(service.with_worker(
        executable.clone(),
        BTreeMap::new(),
        &second_cancel,
        move |_| {
            let _ = second_tx.send(());
            second_release_rx
                .recv_timeout(Duration::from_secs(30))
                .unwrap();
            Ok(())
        },
    ));
    tokio::select! {
        result = &mut second => panic!("second worker completed before its independent gate: {result:?}"),
        result = &mut second_rx => result.unwrap(),
    }
    assert_eq!(occupied_job_slots(service), 1);
    let second_pool = pool.reserved();
    assert!(second_pool > baseline);
    second_release_tx.send(()).unwrap();
    second.await.unwrap();
    assert_eq!(occupied_job_slots(service), 0);
    let final_pool = pool.reserved();
    assert_eq!(final_pool, baseline);
    let elapsed = started.elapsed();
    service.clear_program_cache();
    drop(executable);
    drop(model);
    drop(revision);
    drop(workspace);
    (
        elapsed,
        json!({"source_content_hash":pse_ids::encoding_checksum(demand::SOURCE.as_bytes()).content_hash(),"physical_context_hash":pse_compiler::workspace::physical_identity(&context.quantities,&context.preconditions),"stages":[{"stage":"worker-permit-lifetime","outcome":"prepared","seconds":{"total":elapsed.as_secs_f64()}}],"admission":{"cores":1,"worker_job_slots":1,"job_slot_observation":"math_native_active counts occupied admission slots, including waiters","concurrent_call_outcome":"typed-refusal","refusal":refusal.to_string(),"during_teardown_refusal":teardown_refusal.to_string(),"after_drain":"reacquired","internal_queue":false},"gates":["first-closure-entered","await-future-dropped","concurrent-call-refused","native-tls-teardown-entered","teardown-call-refused","native-thread-joined-and-admission-drained","second-worker-entered","second-worker-drained"],"pool_bytes":{"baseline":baseline,"first_entry":entered_pool,"after_future_drop":dropped_pool,"during_tls_teardown":teardown_pool,"after_first_drain":drained_pool,"second_entry":second_pool,"after_second_drain":final_pool},"after_products_release_bytes":pool.reserved(),"compiler_phases":phases.report(1),"constructions":phases.constructions()}),
    )
}
