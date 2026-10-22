// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared ordinary query facts preserve real event and operation boundaries.
use super::*;
use pse_diagnostics::observation::{ObservationHub, Policy, RecordTap, Value};
use std::sync::{Mutex, atomic::AtomicUsize};
use std::time::Duration;

fn observe_query_event(event: &salsa::Event, cache: &Mutex<Option<QueryFacts>>) {
    super::observe_query_event(event, cache, &Mutex::new(QueryLabels::default()));
}

fn declarations(value: u64) -> Vec<pse_modeling::Declaration> {
    pse_authoring::language::parse(
        &format!(
            "package observation {{ def Root {{ var x: Scalar; eq balance: x == {value}; }} }}"
        ),
        SemanticId::from_bytes([91; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}

fn actual_key() -> salsa::DatabaseKeyIndex {
    let key = Arc::new(Mutex::new(None));
    let sink = key.clone();
    let mut workspace = CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        Some(Box::new(move |event| {
            if let salsa::EventKind::WillExecute { database_key } = event.kind {
                *sink.lock().unwrap() = Some(database_key);
            }
        })),
    )
    .unwrap();
    workspace
        .publish_modeling(declarations(1), pse_modeling::PhysicalScope::default())
        .unwrap();
    let captured = *key.lock().unwrap();
    captured.unwrap()
}

fn count(context: &pse_diagnostics::observation::ObservationContext, name: &str) -> u64 {
    context
        .reader()
        .snapshot(Duration::from_secs(60))
        .metrics
        .iter()
        .find(|metric| metric.name == name)
        .map_or(0, |metric| match metric.cumulative {
            Value::Count(value) => value,
            ref other => panic!("unexpected counter: {other:?}"),
        })
}

#[test]
fn query_observation_batches_exact_events_without_ordinary_key_formatting() {
    let key = actual_key();
    let cache = Mutex::new(None);
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("events");
    let context = operation.context();
    let _entered = context.enter();
    for _ in 0..2 {
        observe_query_event(
            &salsa::Event::new(salsa::EventKind::WillExecute { database_key: key }),
            &cache,
        );
        observe_query_event(
            &salsa::Event::new(salsa::EventKind::WillBlockOn {
                database_key: key,
                other_thread_id: std::thread::current().id(),
            }),
            &cache,
        );
    }
    observe_query_event(
        &salsa::Event::new(salsa::EventKind::DidValidateMemoizedValue { database_key: key }),
        &cache,
    );
    observe_query_event(
        &salsa::Event::new(salsa::EventKind::DidDiscard { key }),
        &cache,
    );
    assert_eq!(count(&context, "compiler.query_executions"), 2);
    assert_eq!(count(&context, "compiler.query_memo_validations"), 1);
    assert_eq!(count(&context, "compiler.query_shared_waits"), 2);
    assert_eq!(count(&context, "compiler.query_discards"), 1);
    let snapshot = operation.reader().snapshot(Duration::from_secs(60));
    assert!(
        !snapshot
            .sources
            .iter()
            .any(|source| source.name == "salsa_query")
    );
    assert!(
        snapshot
            .metrics
            .iter()
            .filter(|metric| metric.name.ends_with("reason") || metric.name.ends_with("duration"))
            .all(|metric| matches!(metric.latest, Value::Unavailable(_)))
    );
    assert_eq!(snapshot.coverage.dropped_updates, 0);
    let held = cache.lock().unwrap();
    observe_query_event(
        &salsa::Event::new(salsa::EventKind::WillExecute { database_key: key }),
        &cache,
    );
    drop(held);
    assert_eq!(count(&context, "compiler.query_executions"), 2);
    assert_eq!(
        operation
            .reader()
            .snapshot(Duration::from_secs(60))
            .coverage
            .dropped_updates,
        1
    );
}

#[test]
fn query_observation_selected_attribution_and_operation_switch_keep_original_populations() {
    #[derive(Debug)]
    struct Detail;
    impl RecordTap for Detail {
        fn selected(&self, family: &str) -> bool {
            family == "compiler_query.v1"
        }
        fn observe(&self, _: &pse_diagnostics::observation::CoherentRecord) {}
    }
    let key = actual_key();
    let event = salsa::Event::new(salsa::EventKind::WillExecute { database_key: key });
    let cache = Mutex::new(None);
    let hub = ObservationHub::new(Policy::default());
    let first = hub.start("first");
    let first_context = first.context().with_record_tap(Arc::new(Detail));
    {
        let _entered = first_context.enter();
        observe_query_event(&event, &cache);
    }
    let second = hub.start("second");
    let second_context = second.context();
    {
        let _entered = second_context.enter();
        observe_query_event(&event, &cache);
    }
    {
        let _entered = first_context.enter();
        observe_query_event(&event, &cache);
    }
    assert_eq!(count(&first_context, "compiler.query_executions"), 2);
    assert_eq!(count(&second_context, "compiler.query_executions"), 1);
    assert_eq!(
        first
            .reader()
            .snapshot(Duration::from_secs(60))
            .sources
            .iter()
            .find(|source| source.name == "salsa_query")
            .unwrap()
            .bytes,
        format!("{:?}:{:?}", key.ingredient_index(), key.key_index()).as_bytes()
    );
    let snapshot = first.reader().snapshot(Duration::from_secs(60));
    let detail = snapshot
        .producers
        .iter()
        .find(|producer| producer.descriptor.name == "compiler.query")
        .unwrap();
    assert_eq!(detail.current.as_ref().unwrap().family, "compiler_query.v1");
    assert_eq!(detail.current.as_ref().unwrap().phase, "execute");
    assert!(
        second
            .reader()
            .snapshot(Duration::from_secs(60))
            .sources
            .is_empty()
    );
}

#[test]
fn query_observation_actual_workspace_rebuild_keeps_event_hook_and_current_operation() {
    let executions = Arc::new(AtomicUsize::new(0));
    let sink = executions.clone();
    let mut workspace = CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        Some(Box::new(move |event| {
            if matches!(event.kind, salsa::EventKind::WillExecute { .. }) {
                sink.fetch_add(1, Ordering::Relaxed);
            }
        })),
    )
    .unwrap();
    let hub = ObservationHub::new(Policy::default());
    let first = hub.start("first");
    {
        let _entered = first.context().enter();
        workspace
            .publish_modeling(declarations(1), pse_modeling::PhysicalScope::default())
            .unwrap();
        workspace.rebuild(workspace.inputs.clone()).unwrap();
        workspace
            .publish_modeling(declarations(2), pse_modeling::PhysicalScope::default())
            .unwrap();
    }
    let first_count = count(&first.context(), "compiler.query_executions");
    assert_eq!(first_count, executions.load(Ordering::Relaxed) as u64);
    assert!(first_count > 0);
    let second = hub.start("second");
    {
        let _entered = second.context().enter();
        workspace
            .publish_modeling(declarations(1), pse_modeling::PhysicalScope::default())
            .unwrap();
    }
    assert_eq!(
        count(&first.context(), "compiler.query_executions"),
        first_count
    );
    assert_eq!(
        count(&second.context(), "compiler.query_executions"),
        executions.load(Ordering::Relaxed) as u64 - first_count
    );
}

#[test]
fn actual_preparation_owner_phases_close_on_success_and_cancellation() {
    fn closed(context: &pse_diagnostics::observation::ObservationContext, name: &str) {
        let snapshot = context.reader().snapshot(Duration::from_secs(60));
        assert!(
            snapshot.phases.is_empty(),
            "no current phase survives its actual owner"
        );
        let entries = snapshot
            .detail
            .iter()
            .filter(|event| event.name == name)
            .filter_map(|event| event.phase.as_ref())
            .filter(|phase| phase.finished_at.is_none())
            .collect::<Vec<_>>();
        assert!(!entries.is_empty(), "actual owner phase missing: {name}");
        for entry in entries {
            let exits = snapshot
                .detail
                .iter()
                .filter(|event| event.name == name)
                .filter_map(|event| event.phase.as_ref())
                .filter(|phase| phase.token == entry.token && phase.finished_at.is_some())
                .collect::<Vec<_>>();
            assert_eq!(exits.len(), 1, "each admitted owner retires exactly once");
            assert_eq!(exits[0].started_at, entry.started_at);
            assert!(exits[0].finished_at.unwrap() >= entry.started_at);
        }
    }
    let mut workspace = CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        None,
    )
    .unwrap();
    let revision = workspace
        .publish_modeling(declarations(7), pse_modeling::PhysicalScope::default())
        .unwrap();
    let root = revision.checked().entry("observation.Root").unwrap();
    let instance = pse_modeling::specialize::root_instance(root);
    let hub = ObservationHub::new(Policy::default());
    let success = hub.start("actual preparation owners");
    let context = success.context();
    {
        let _entered = context.enter();
        workspace
            .prepare_modeling_cancellable(
                root,
                instance,
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }
    for name in [
        "modeling.specialization",
        "compiler.body_materialization",
        "compiler.typed_body_lowering",
    ] {
        closed(&context, name);
    }
    let cancelled = hub.start("cancelled actual preparation owners");
    let context = cancelled.context();
    {
        let _entered = context.enter();
        assert!(
            pse_modeling::specialize::specialize_cancellable(
                revision.checked(),
                root,
                instance,
                &pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                &|| true
            )
            .is_err()
        );
        let fixture = crate::typed_math::direct_tests::fixture(
            crate::authored_transfer_tests::context(),
            "package direct {def Root {let out:Scalar=1;}}",
            &["out"],
        )
        .unwrap();
        let error = fixture.admit(&Arc::new(AtomicBool::new(true))).unwrap_err();
        assert!(matches!(error, MathError::Cancelled));
    }
    closed(&context, "modeling.specialization");
    closed(&context, "compiler.typed_body_lowering");
    assert!(
        !context
            .reader()
            .snapshot(Duration::from_secs(60))
            .detail
            .iter()
            .any(|event| event.name == "compiler.body_materialization"),
        "a cancelled direct typed admission does not fabricate a grouped materialization owner"
    );
}

#[test]
fn semantic_query_labels_are_exact_generation_owned_and_leave_unknown_events_unmapped() {
    #[derive(Debug, Default)]
    struct Detail(Mutex<Vec<pse_diagnostics::observation::CoherentRecord>>);
    impl RecordTap for Detail {
        fn selected(&self, family: &str) -> bool {
            family == "compiler_query.v1"
        }
        fn observe(&self, record: &pse_diagnostics::observation::CoherentRecord) {
            self.0.lock().unwrap().push(record.clone());
        }
    }
    let tap = Arc::new(Detail::default());
    let hub = ObservationHub::new(Policy::default());
    let owner = hub.start("semantic labels");
    let context = owner.context().with_record_tap(tap.clone());
    let _entered = context.enter();
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let original = declarations(1);
    let expected = original
        .iter()
        .map(|row| SemanticId::from(row.declaration_id).to_string())
        .collect::<Vec<_>>();
    workspace
        .publish_modeling(original, pse_modeling::PhysicalScope::default())
        .unwrap();
    workspace
        .publish_modeling(declarations(2), pse_modeling::PhysicalScope::default())
        .unwrap();
    let records = tap.0.lock().unwrap();
    let mapped = records
        .iter()
        .filter(|record| {
            record.values.iter().any(|field| {
                field.name == "query_family"
                    && field.value == Value::Text("declaration.parsed".into())
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !mapped.is_empty(),
        "real declaration query executions must be labeled"
    );
    assert!(mapped.iter().all(|record| {
        record.values.iter().any(|field| {
            field.name == "semantic_subject"
                && matches!(&field.value, Value::Text(subject) if expected.contains(subject))
        })
    }));
    assert!(
        records.iter().any(|record| record.phase == "validate"
            && record
                .values
                .iter()
                .any(|field| field.name == "semantic_subject"
                    && matches!(&field.value, Value::Text(subject) if expected.contains(subject)))),
        "an actual Salsa memo validation must retain an exact semantic label"
    );
    assert!(records.iter().any(|record| {
        record.values.iter().any(|field| {
            field.name == "semantic_subject" && matches!(field.value, Value::Unavailable(_))
        })
    }));
    drop(records);
    let old = workspace.db.query_labels.clone();
    workspace.rebuild(workspace.inputs.clone()).unwrap();
    assert!(!Arc::ptr_eq(&old, &workspace.db.query_labels));
    assert_eq!(workspace.db.query_labels.lock().unwrap().generation, 1);
    assert_eq!(old.lock().unwrap().generation, 0);
}
