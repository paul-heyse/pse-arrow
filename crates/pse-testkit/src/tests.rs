// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use capture::{Assurance, Capture};
use pse_columnar::CancellationToken;
use pse_engine::session::assurance::{ObservationPolicy, OperationObservation, TerminalStatus};
use tracing::{Instrument, instrument::WithSubscriber};

fn fixture() -> NativeFixture {
    NativeFixture::new(NonZeroUsize::new(64 << 20).unwrap()).unwrap()
}

#[test]
fn native_unit_observation_coverage_distinguishes_disabled_sampled_and_truncated() {
    use capture::Coverage;
    for coverage in [Coverage::Disabled, Coverage::Sampled, Coverage::Interrupted] {
        let capture = Capture::new(8, 4096).with_coverage(coverage);
        assert_eq!(capture.coverage(), coverage);
        assert!(matches!(capture.assess(1, &[]), Assurance::Inconclusive(_)));
    }
    let capture = Capture::new(0, 0);
    tracing::dispatcher::with_default(&capture.dispatch(), || {
        let span = tracing::info_span!("refused");
        let _guard = span.enter();
    });
    assert_eq!(capture.coverage(), Coverage::Truncated);
}

#[tokio::test]
async fn native_unit_value_helper_defaults_to_contract_and_diagnostic_is_explicit() {
    let fixture = fixture();
    let state = fixture.factory.native_state();
    let context = datafusion::prelude::SessionContext::new_with_state(state.clone());
    let plan = context
        .sql("SELECT 42")
        .await
        .unwrap()
        .into_unoptimized_plan();
    let registry = pse_schema::shared_registry().unwrap();
    let prepared = execution::prepare(state, registry.clone(), &plan).unwrap();
    assert!(!prepared.observation().is_captured());
    let diagnostic =
        execution::prepare_with_observation(state, registry, &plan, ObservationPolicy::Diagnostic)
            .unwrap();
    assert!(diagnostic.observation().is_captured());
    let cancel = CancellationToken::new();
    assert_eq!(
        prepared.execute(&cancel).await.unwrap().batches(),
        diagnostic.execute(&cancel).await.unwrap().batches()
    );
}
fn operation(capture: &Capture) -> u64 {
    capture
        .snapshot()
        .0
        .iter()
        .find(|span| span.name == "pse.operation")
        .unwrap()
        .fields["operation_id"]
        .parse()
        .unwrap()
}

#[test]
fn native_unit_factory_shares_resources_and_observation_preserves_identity() {
    let fixture = fixture();
    let original = fixture
        .session(pse_schema::shared_registry().unwrap())
        .unwrap();
    let observed = fixture
        .factory
        .clone()
        .with_observation(ObservationPolicy::Contract)
        .candidate(
            std::collections::BTreeMap::default(),
            pse_schema::shared_registry().unwrap(),
            &CancellationToken::new(),
        )
        .unwrap();
    assert_eq!(
        original.implementation_generation(),
        observed.implementation_generation()
    );
    assert_eq!(original.settings_hash(), observed.settings_hash());
    assert!(Arc::ptr_eq(
        fixture.factory.native_state().runtime_env(),
        &fixture.resources.runtime
    ));
    assert!(Arc::ptr_eq(
        fixture.resources.caches.pool(),
        &fixture.resources.runtime.memory_pool
    ));
    let claim = pse_columnar::MemoryConsumer::new("fixture").register(&fixture.resources.pool);
    let before = fixture.resources.pool.reserved();
    claim.try_grow(1024).unwrap();
    assert_eq!(fixture.resources.pool.reserved(), before + 1024);
    drop(claim);
    assert_eq!(fixture.resources.pool.reserved(), before);
}

#[tokio::test]
async fn native_unit_execution_capture_has_terminal_and_negative_controls() {
    let fixture = fixture();
    let capture = Capture::new(4096, 1 << 20);
    async {
        let session = fixture
            .factory
            .clone()
            .with_observation(ObservationPolicy::Contract)
            .candidate(
                std::collections::BTreeMap::default(),
                pse_schema::shared_registry().unwrap(),
                &CancellationToken::new(),
            )
            .unwrap();
        let cancel = CancellationToken::new();
        let plan = session
            .prepare_sql("SELECT 42 AS value", &cancel)
            .await
            .unwrap();
        let prepared = plan;
        assert!(!prepared.observation().is_captured());
        let completed = prepared.execute(&cancel).await.unwrap();
        // Contract-only capture retains no live physical plan or metric snapshot.
        assert!(completed.observation().execution().is_none());
        drop(completed);
    }
    .with_subscriber(capture.dispatch())
    .await;
    let id = operation(&capture);
    assert_eq!(
        capture.assess(id, &[]),
        Assurance::Established,
        "{:?}",
        capture.snapshot()
    );
    assert_eq!(
        capture.assess(id, &["ImpossibleOperator"]),
        Assurance::Inconclusive("required native operator absent")
    );
    assert!(matches!(
        capture.assess(u64::MAX, &[]),
        Assurance::Inconclusive(_)
    ));
}

#[tokio::test]
async fn native_unit_abandonment_is_not_completion() {
    let fixture = fixture();
    let capture = Capture::new(4096, 1 << 20);
    async {
        let cancel = CancellationToken::new();
        let session = fixture
            .factory
            .clone()
            .with_observation(ObservationPolicy::Contract)
            .candidate(
                std::collections::BTreeMap::default(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let plan = session.prepare_sql("SELECT 1", &cancel).await.unwrap();
        let stream = plan.execute_stream(&cancel).await.unwrap();
        drop(stream);
    }
    .with_subscriber(capture.dispatch())
    .await;
    assert_eq!(
        capture.assess(operation(&capture), &[]),
        Assurance::Violated("operation did not complete")
    );
    assert!(
        capture
            .snapshot()
            .0
            .iter()
            .any(|s| s.fields.get("terminal").is_some_and(|v| v == "Abandoned"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_unit_async_and_blocking_tasks_keep_dispatch_and_parent() {
    let capture = Capture::new(128, 1 << 16);
    async {
        let mut operation = OperationObservation::start(ObservationPolicy::Contract, "tasks");
        async {
            let mut tasks = datafusion::common::runtime::JoinSet::new();
            tasks.spawn(async {
                let span = tracing::info_span!("async-child");
                let _entered = span.enter();
            });
            tasks.spawn_blocking(|| {
                let span = tracing::info_span!("blocking-child");
                let _entered = span.enter();
            });
            while let Some(result) = tasks.join_next().await {
                result.unwrap();
            }
        }
        .instrument(operation.span())
        .await;
        operation.finish(TerminalStatus::Completed);
    }
    .with_subscriber(capture.dispatch())
    .await;
    let spans = capture.snapshot().0;
    let root = spans
        .iter()
        .find(|span| span.name == "pse.operation")
        .unwrap();
    for name in ["async-child", "blocking-child"] {
        assert_eq!(
            spans.iter().find(|span| span.name == name).unwrap().parent,
            Some(root.id)
        );
    }
}

#[test]
fn native_unit_truncation_and_missing_execution_are_inconclusive() {
    for capture in [Capture::new(0, 0), Capture::new(4, 4096)] {
        tracing::dispatcher::with_default(&capture.dispatch(), || {
            let mut operation = OperationObservation::start(ObservationPolicy::Contract, "control");
            operation.finish(TerminalStatus::Completed);
        });
        let id = capture
            .snapshot()
            .0
            .first()
            .and_then(|s| s.fields.get("operation_id"))
            .map_or(0, |value| value.parse().unwrap());
        assert!(matches!(
            capture.assess(id, &[]),
            Assurance::Inconclusive(_)
        ));
    }
}

#[test]
fn native_unit_settings_resolve_actual_extensions_and_refuse_zero_capacity() {
    let fixture = fixture();
    let extension = Arc::new(731_u64);
    let factory = fixture.factory.clone().with_extension(extension.clone());
    let changes = std::collections::BTreeMap::from([(
        "datafusion.execution.batch_size".into(),
        "128".into(),
    )]);
    let (state, effective) = pse_engine::session::config::EffectiveSettings::resolve(
        factory.native_state().clone(),
        &changes,
    )
    .unwrap();
    assert!(Arc::ptr_eq(
        &state.config().get_extension::<u64>().unwrap(),
        &extension
    ));
    assert_eq!(
        effective.inspection["datafusion.execution.batch_size"].as_deref(),
        Some("128")
    );
    assert!(
        effective
            .operational
            .contains_key("datafusion.runtime.memory_limit")
    );
    let invalid =
        std::collections::BTreeMap::from([("datafusion.execution.batch_size".into(), "0".into())]);
    assert!(pse_engine::session::config::EffectiveSettings::resolve(state, &invalid).is_err());
}

#[test]
fn native_unit_registered_store_reads_keep_actual_identity() {
    let fixture = fixture();
    let root = datafusion::execution::object_store::ObjectStoreUrl::local_filesystem();
    let first = fixture.resources.runtime.object_store(&root).unwrap();
    let second = fixture.resources.runtime.object_store(&root).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    fixture
        .resources
        .runtime
        .register_object_store(root.as_ref(), first.clone());
    let rebound = fixture.resources.runtime.object_store(&root).unwrap();
    assert!(Arc::ptr_eq(&first, &rebound));
}

#[derive(Debug)]
struct CallerPlanner(Arc<std::sync::atomic::AtomicUsize>);
#[async_trait::async_trait]
impl datafusion::execution::context::QueryPlanner for CallerPlanner {
    async fn create_physical_plan(
        &self,
        plan: &datafusion::logical_expr::LogicalPlan,
        state: &dyn datafusion::catalog::Session,
    ) -> datafusion::common::Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        pse_engine::session::planner::UnifiedPlanner::default()
            .create_physical_plan(plan, state)
            .await
    }
}

#[tokio::test]
async fn native_unit_custom_assembly_and_freshness_survive_observation_modes() {
    use datafusion::{
        arrow::datatypes::DataType,
        common::ScalarValue,
        logical_expr::{ColumnarValue, Volatility, create_udf},
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    for policy in [
        ObservationPolicy::Off,
        ObservationPolicy::Contract,
        ObservationPolicy::Diagnostic,
    ] {
        let fixture = fixture();
        let calls = Arc::new(AtomicUsize::new(0));
        let udf_calls = Arc::new(AtomicUsize::new(0));
        let counter = udf_calls.clone();
        let udf = Arc::new(create_udf(
            "caller_value",
            vec![],
            DataType::Int64,
            Volatility::Volatile,
            Arc::new(move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(731))))
            }),
        ));
        let config = fixture
            .factory
            .native_state()
            .config()
            .clone()
            .with_extension(Arc::new(219_u64));
        let builder = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .with_config(config)
            .with_scalar_functions(vec![udf.clone()])
            .with_query_planner(Arc::new(CallerPlanner(calls.clone())));
        let factory = EngineFactory::from_builder(
            fixture.resources.runtime.clone(),
            fixture.resources.pool.clone(),
            "native-unit-custom",
            builder,
        )
        .with_observation(policy);
        assert!(Arc::ptr_eq(
            &udf,
            factory
                .native_state()
                .scalar_functions()
                .get("caller_value")
                .unwrap()
        ));
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                std::collections::BTreeMap::default(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let prepared = session
            .prepare_sql("SELECT caller_value()", &cancel)
            .await
            .unwrap();
        assert!(prepared.contains_volatile_expression());
        assert_eq!(udf_calls.load(Ordering::SeqCst), 0);
        let completed = prepared.execute(&cancel).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(udf_calls.load(Ordering::SeqCst), 1);
        assert!(session.requires_fresh_execution());
        assert_eq!(
            completed.observation().is_captured(),
            policy == ObservationPolicy::Diagnostic
        );
    }
}

#[tokio::test]
async fn native_unit_cancelled_preparation_records_explicit_terminal() {
    let fixture = fixture();
    let capture = Capture::new(4096, 1 << 20);
    async {
        let cancel = CancellationToken::new();
        let session = fixture
            .factory
            .clone()
            .with_observation(ObservationPolicy::Contract)
            .candidate(
                std::collections::BTreeMap::default(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let prepared = session.prepare_sql("SELECT 1", &cancel).await.unwrap();
        cancel.cancel();
        assert!(prepared.execute_stream(&cancel).await.is_err());
    }
    .with_subscriber(capture.dispatch())
    .await;
    assert!(capture.snapshot().0.iter().any(|s| {
        s.fields
            .get("terminal")
            .is_some_and(|value| value == "Cancelled")
    }));
    assert_eq!(
        capture.assess(operation(&capture), &[]),
        Assurance::Violated("operation did not complete")
    );
}

#[tokio::test]
async fn native_unit_poll_failure_is_not_successful_span_closure() {
    use datafusion::{
        arrow::datatypes::DataType,
        common::DataFusionError,
        logical_expr::{Volatility, create_udf},
    };
    let fixture = fixture();
    let capture = Capture::new(4096, 1 << 20);
    async {
        let udf = Arc::new(create_udf(
            "poll_error",
            vec![],
            DataType::Int64,
            Volatility::Volatile,
            Arc::new(|_| {
                Err(DataFusionError::Execution(
                    "deliberate execution refusal".into(),
                ))
            }),
        ));
        let builder = datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
            fixture.factory.native_state().clone(),
        )
        .with_scalar_functions(vec![udf]);
        let factory = EngineFactory::from_builder(
            fixture.resources.runtime.clone(),
            fixture.resources.pool.clone(),
            "poll-failure",
            builder,
        )
        .with_observation(ObservationPolicy::Contract);
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                std::collections::BTreeMap::default(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let prepared = session
            .prepare_sql("SELECT poll_error()", &cancel)
            .await
            .unwrap();
        let mut stream = prepared.execute_stream(&cancel).await.unwrap();
        assert!(stream.next_batch(&cancel).await.is_err());
        drop(stream);
    }
    .with_subscriber(capture.dispatch())
    .await;
    assert!(capture.snapshot().0.iter().any(|s| {
        s.fields
            .get("terminal")
            .is_some_and(|value| value == "Failed")
    }));
    assert_eq!(
        capture.assess(operation(&capture), &[]),
        Assurance::Violated("operation did not complete")
    );
}

#[tokio::test]
async fn native_unit_reusable_rounds_have_distinct_terminal_evidence() {
    let fixture = fixture();
    let capture = Capture::new(4096, 1 << 20);
    async {
        let cancel = CancellationToken::new();
        let session = fixture
            .factory
            .clone()
            .with_observation(ObservationPolicy::Contract)
            .candidate(
                std::collections::BTreeMap::new(),
                pse_schema::shared_registry().unwrap(),
                &cancel,
            )
            .unwrap();
        let mut reusable = session
            .prepare_sql("SELECT 42 AS value", &cancel)
            .await
            .unwrap()
            .prepare_reusable(&cancel)
            .await
            .unwrap();
        drop(reusable.execute(&cancel).await.unwrap());
        drop(reusable.execute(&cancel).await.unwrap());
        cancel.cancel();
        assert!(reusable.execute(&cancel).await.is_err());
        drop(reusable);
    }
    .with_subscriber(capture.dispatch())
    .await;
    let (spans, truncated) = capture.snapshot();
    assert!(!truncated);
    let rounds: Vec<_> = spans
        .iter()
        .filter(|span| {
            span.name == "pse.operation"
                && span
                    .fields
                    .get("operation_kind")
                    .is_some_and(|kind| kind == "round")
        })
        .collect();
    assert_eq!(rounds.len(), 3);
    let completed: Vec<_> = rounds
        .iter()
        .filter(|span| span.fields["terminal"] == "Completed")
        .collect();
    assert_eq!(completed.len(), 2);
    for span in completed {
        assert_eq!(
            capture.assess(
                span.fields["operation_id"].parse().unwrap(),
                &["ProjectionExec"]
            ),
            Assurance::Established
        );
    }
    assert_eq!(
        rounds
            .iter()
            .filter(|span| span.fields["terminal"] == "Cancelled")
            .count(),
        1
    );
}

#[tokio::test]
async fn native_unit_store_wrappers_preserve_conditions_ranges_and_consumption() {
    use fault_store::{Fault, FaultPlan, FaultStore};
    use object_store::{GetOptions, ObjectStore, ObjectStoreExt, PutMode, PutOptions, path::Path};
    let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
    let faults = FaultStore::new(inner.clone());
    let counts = counting_store::CountingStore::new(faults.clone());
    let path = Path::from("_delta_log/000.json");
    let create = PutOptions {
        mode: PutMode::Create,
        ..Default::default()
    };
    counts
        .put_opts(&path, Vec::from(b"abcdef").into(), create.clone())
        .await
        .unwrap();
    assert!(
        counts
            .put_opts(&path, Vec::from(b"wrong").into(), create)
            .await
            .is_err()
    );
    let read = counts
        .get_opts(
            &path,
            GetOptions {
                range: Some((1..4).into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(read.range, 1..4);
    assert_eq!(counts.report()["bytes_consumed"], 0);
    assert_eq!(read.bytes().await.unwrap().as_ref(), b"bcd");
    assert_eq!(counts.report()["bytes_consumed"], 3);
    assert_eq!(counts.report()["log_gets"], 1);
    faults.arm(FaultPlan {
        operation: "put",
        prefix: path.to_string(),
        call: 1,
        fault: Fault::LostResponse,
    });
    assert!(
        counts
            .put(&path, Vec::from(b"committed").into())
            .await
            .is_err()
    );
    assert_eq!(faults.fired(), 1);
    assert_eq!(
        inner
            .get(&path)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .as_ref(),
        b"committed"
    );
    // Injection is one-shot; a backend conditional refusal remains a native error.
    assert!(matches!(
        counts
            .put_opts(
                &path,
                Vec::from(b"duplicate").into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                }
            )
            .await,
        Err(object_store::Error::AlreadyExists { .. })
    ));
    counts.reset();
    assert_eq!(counts.report()["gets"], 0);
    assert_eq!(counts.report()["bytes_consumed"], 0);
}

#[test]
fn native_unit_caller_pool_is_the_native_runtime_pool() {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(1024));
    let factory = factory(
        pool.clone(),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: NonZeroUsize::MIN,
            target_partitions: NonZeroUsize::MIN,
        },
    )
    .unwrap();
    assert!(Arc::ptr_eq(
        &pool,
        &factory.native_state().runtime_env().memory_pool
    ));
}
