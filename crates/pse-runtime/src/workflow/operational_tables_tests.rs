// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The query surface over the operational store (Plan 22 O9): predicate recognition, the
//! providers against isolated PostgreSQL 18 databases, and a durable run's operational
//! rows joined with its result relations in an engine session.
use super::super::durable_tests::{LINEAR, durable_runtime, package_on, record};
use super::super::tests::runtime;
use super::super::{ProgressStream, StreamRecord};
use super::*;
use datafusion::{
    arrow::array::{Array, AsArray},
    logical_expr::{cast, col, lit},
    physical_plan::collect,
    prelude::{SessionConfig, SessionContext},
};
use pse_operations::{
    attempts::{AttemptId, AttemptKind, NewAttempt, TransitionNote},
    lifecycle::AttemptState,
    testing::TestDatabase,
};
use pse_schema::Registry;

fn registry() -> Arc<Registry> {
    pse_schema::shared_registry().unwrap()
}

fn binary(id: SemanticId) -> ScalarValue {
    ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec()))
}

fn instant(micros: i64) -> ScalarValue {
    ScalarValue::TimestampMicrosecond(Some(micros), Some("UTC".into()))
}

fn columns_match<S: Pushdown>(
    registry: &Registry,
    validation: &pse_relations::validate::ValidationContext,
) {
    let relation = <S::Row as RelationRow>::relation(registry).unwrap();
    assert_eq!(relation.key.name, format!("operational_{}", S::TABLE));
    let schema = schema_of::<S>(registry, validation).unwrap();
    for (name, kind) in S::COLUMNS {
        let field = schema.field_with_name(name).unwrap();
        let expected = match kind {
            Kind::Id => DataType::FixedSizeBinary(16),
            Kind::Member => DataType::Utf8,
            Kind::Time => DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
        };
        assert_eq!(field.data_type(), &expected, "{}.{name}", S::TABLE);
    }
}

#[test]
fn pushdown_columns_are_the_registry_columns() {
    let runtime = runtime();
    let registry = &runtime.registry;
    let validation = runtime.validation_context().unwrap();
    columns_match::<AttemptScan>(registry, &validation);
    columns_match::<TransitionScan>(registry, &validation);
    columns_match::<JobScan>(registry, &validation);
    columns_match::<ProgressEventScan>(registry, &validation);
    columns_match::<ProgressValueScan>(registry, &validation);
    columns_match::<IncumbentScan>(registry, &validation);
    columns_match::<SolutionScan>(registry, &validation);
    columns_match::<StudyScan>(registry, &validation);
    columns_match::<StudyPointScan>(registry, &validation);
    columns_match::<WorkspaceScan>(registry, &validation);
    columns_match::<PublicationScan>(registry, &validation);
    columns_match::<PublicationMemberScan>(registry, &validation);
    columns_match::<SettlementScan>(registry, &validation);
}

#[test]
fn recognized_predicates_become_the_typed_filter() {
    let target = SemanticId::from_bytes([7; 16]);
    let other = SemanticId::from_bytes([8; 16]);
    let filters = [
        col("attempt_id").eq(lit(binary(target))),
        col("state").in_list(vec![lit("running"), lit("not-a-state")], false),
        col("created_at").gt(lit(instant(10))),
        // A literal on the left, and nanoseconds rounded outward.
        lit(ScalarValue::TimestampNanosecond(
            Some(20_500),
            Some("UTC".into()),
        ))
        .gt_eq(col("created_at")),
        col("worker").eq(lit("w")),
        col("attempt_id")
            .eq(lit(binary(target)))
            .or(col("run_id").eq(lit(binary(other)))),
    ];
    let recognized: Vec<bool> = filters
        .iter()
        .map(|f| recognize(f, AttemptScan::COLUMNS).is_some())
        .collect();
    assert_eq!(recognized, [true, true, true, true, false, false]);
    assert_eq!(
        scan_of::<AttemptScan>(&filters),
        Some(AttemptScan {
            attempts: vec![target.into()],
            runs: Vec::new(),
            states: vec![AttemptState::Running],
            created: TimeRange {
                from: DateTime::from_timestamp_micros(10),
                to: DateTime::from_timestamp_micros(21),
            },
        })
    );
    // A short IN list rewritten as a disjunction on one column is one membership.
    let either = col("attempt_id")
        .eq(lit(binary(target)))
        .or(col("attempt_id").eq(lit(binary(other))));
    assert_eq!(
        scan_of::<AttemptScan>(&[either]).map(|scan| scan.attempts),
        Some(vec![target.into(), other.into()])
    );
    // A cast that keeps identity and order is seen through; one that coarsens is not.
    let widened = cast(col("attempt_id"), DataType::Binary)
        .eq(lit(ScalarValue::Binary(Some(target.as_bytes().to_vec()))));
    assert!(recognize(&widened, AttemptScan::COLUMNS).is_some());
    let coarse = cast(
        col("created_at"),
        DataType::Timestamp(TimeUnit::Second, Some("UTC".into())),
    )
    .lt_eq(lit(ScalarValue::TimestampSecond(
        Some(1),
        Some("UTC".into()),
    )));
    assert!(recognize(&coarse, AttemptScan::COLUMNS).is_none());
    // Contradictions select nothing, so no statement runs.
    for contradiction in [
        vec![
            col("attempt_id").eq(lit(binary(target))),
            col("attempt_id").eq(lit(binary(other))),
        ],
        vec![col("state").eq(lit("not-a-state"))],
        vec![
            col("created_at").gt(lit(instant(30))),
            col("created_at").lt(lit(instant(10))),
        ],
        vec![col("attempt_id").eq(lit(ScalarValue::Binary(Some(vec![1, 2]))))],
    ] {
        assert_eq!(
            scan_of::<AttemptScan>(&contradiction),
            None,
            "{contradiction:?}"
        );
    }
    assert_eq!(scan_of::<AttemptScan>(&[]), Some(AttemptScan::default()));
}

/// Register `attempts` in a plain DataFusion context over `store`.
fn context(store: &Store, batch_size: usize) -> SessionContext {
    let context = SessionContext::new_with_config(SessionConfig::new().with_batch_size(batch_size));
    let registry = registry();
    let validation = Arc::new(pse_relations::validate::ValidationContext::new(
        &registry,
        pse_engine::validation::NativeValidation(context.state()),
    ));
    context
        .register_table(
            "attempts",
            Arc::new(
                OperationalTable::<AttemptScan>::new(store.clone(), registry, validation).unwrap(),
            ),
        )
        .unwrap();
    context
}

/// What one scan of a query did: its typed filter, pushed limit and fetched rows.
#[derive(Debug, PartialEq)]
struct Scanned {
    filter: Option<AttemptScan>,
    limit: Option<usize>,
    fetched: usize,
}

fn scanned(plan: &Arc<dyn ExecutionPlan>) -> Option<Scanned> {
    if let Some(exec) = plan
        .as_ref()
        .downcast_ref::<OperationalScanExec<AttemptScan>>()
    {
        return Some(Scanned {
            filter: exec.scan.clone(),
            limit: exec.limit,
            fetched: exec
                .metrics()
                .and_then(|m| m.sum_by_name("fetched_rows"))
                .map_or(0, |v| v.as_usize()),
        });
    }
    plan.children().into_iter().find_map(scanned)
}

async fn run(context: &SessionContext, sql: &str) -> (Vec<RecordBatch>, Scanned) {
    let plan = context
        .sql(sql)
        .await
        .unwrap()
        .create_physical_plan()
        .await
        .unwrap();
    let batches = collect(Arc::clone(&plan), context.task_ctx())
        .await
        .unwrap();
    (
        batches,
        scanned(&plan).expect("the plan scans pse_ops.attempts"),
    )
}

/// Attempts under two runs: three of the first, one of the second.
async fn attempts(store: &Store) -> (Vec<AttemptId>, SemanticId) {
    let run: SemanticId = pse_operations::mint_id();
    let mut ids = Vec::new();
    for index in 0..4 {
        let attempt = NewAttempt {
            attempt_id: pse_operations::mint_id(),
            run_id: if index < 3 {
                run.into()
            } else {
                pse_operations::mint_id()
            },
            kind: AttemptKind::Modeling,
            operational_job_identity: pse_ids::roles::RecordedOperationalJobIdentity::current(
                pse_ids::roles::OperationalJobHash::from(pse_ids::ContentHash::from_bytes([1; 32])),
            ),
            preparation_identity: None,
            parent_attempt: None,
        };
        store
            .attempts()
            .create(&attempt, Some("test"))
            .await
            .unwrap();
        ids.push(attempt.attempt_id);
    }
    (ids, run)
}

#[tokio::test]
async fn provider_pushes_attempt_filter() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let (ids, _) = attempts(&store).await;
    let target = ids[1];
    let hex = SemanticId::from(target).to_hex();
    let never = SemanticId::from_bytes([0; 16]).to_hex();
    let context = context(&store, 8192);

    // The statement receives the typed filter and returns only the matching row.
    let (pushed, scan) = run(
        &context,
        &format!("SELECT attempt_id, run_id, state FROM attempts WHERE attempt_id = X'{hex}'"),
    )
    .await;
    assert_eq!(
        scan,
        Scanned {
            filter: Some(AttemptScan {
                attempts: vec![target],
                ..AttemptScan::default()
            }),
            limit: None,
            fetched: 1,
        }
    );
    // The same rows as the unfiltered scan filtered in DataFusion (a disjunction is not
    // pushed down).
    let (residual, scan) = run(
        &context,
        &format!(
            "SELECT attempt_id, run_id, state FROM attempts \
             WHERE attempt_id = X'{hex}' OR run_id = X'{never}'"
        ),
    )
    .await;
    assert_eq!(
        scan,
        Scanned {
            filter: Some(AttemptScan::default()),
            limit: None,
            fetched: 4,
        }
    );
    assert_eq!(pushed, residual);
    assert_eq!(pushed.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);

    // Several recognized predicates are conjoined in one statement.
    let (planned, scan) = run(
        &context,
        &format!(
            "SELECT attempt_id FROM attempts \
             WHERE state IN ('planned', 'queued') AND attempt_id IN (X'{hex}', X'{never}') \
             AND created_at <= now()"
        ),
    )
    .await;
    let filter = scan.filter.unwrap();
    assert_eq!(filter.attempts.len(), 2);
    assert_eq!(filter.states, [AttemptState::Planned, AttemptState::Queued]);
    assert!(filter.created.to.is_some());
    assert_eq!(scan.fetched, 1);
    assert_eq!(planned.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);

    // A contradiction reads nothing.
    let (none, scan) = run(
        &context,
        "SELECT * FROM attempts WHERE state = 'no-such-state'",
    )
    .await;
    assert_eq!((scan.filter, scan.fetched), (None, 0));
    assert_eq!(none.iter().map(RecordBatch::num_rows).sum::<usize>(), 0);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn provider_streams_bounded_pages_and_honours_limit_and_projection() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let (mut ids, _) = attempts(&store).await;
    ids.extend(attempts(&store).await.0);
    ids.sort();
    let context = context(&store, 3);

    let (batches, scan) = run(&context, "SELECT * FROM attempts").await;
    // Pages of at most the session's batch size, in primary-key order.
    assert!(batches.iter().all(|b| b.num_rows() <= 3), "{batches:?}");
    assert_eq!(batches.len(), 3);
    assert_eq!(scan.fetched, 8);
    let read: Vec<SemanticId> = batches
        .iter()
        .flat_map(|batch| {
            let column = batch.column(0).as_fixed_size_binary().clone();
            (0..column.len())
                .map(move |row| SemanticId::from_bytes(column.value(row).try_into().unwrap()))
        })
        .collect();
    assert_eq!(
        read,
        ids.iter()
            .copied()
            .map(SemanticId::from)
            .collect::<Vec<_>>()
    );

    // A limit without filters reaches the scan, which reads no further.
    let (limited, scan) = run(&context, "SELECT attempt_id FROM attempts LIMIT 4").await;
    assert_eq!(scan.limit, Some(4));
    assert_eq!(scan.fetched, 4);
    assert_eq!(limited.iter().map(RecordBatch::num_rows).sum::<usize>(), 4);

    // The projection is the scan's output schema.
    let (projected, _) = run(&context, "SELECT state FROM attempts").await;
    assert_eq!(projected[0].num_columns(), 1);
    assert_eq!(projected[0].schema().field(0).name(), "state");
    database.remove().await.unwrap();
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn operational_tables_join_results_in_datafusion() {
    let database = TestDatabase::create().await.unwrap();
    let durable_runtime = durable_runtime(&database, "query").await;
    let (package, analysis) = package_on(&durable_runtime, LINEAR).await;
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    assert!(result.usable());
    let durable = record(&result);
    let hex = SemanticId::from(durable.attempt_id).to_hex();
    let token = pse_columnar::CancellationToken::new();
    let session = durable_runtime
        .query_session(None, Some(&result), &token)
        .unwrap();

    // The durable attempt joined with its retained result relation.
    let joined = session
        .sql(
            &format!(
                "SELECT a.state, a.kind, s.step, s.state AS solve_state \
                 FROM {OPERATIONAL_SCHEMA}.attempts AS a \
                 JOIN workspace.runtime.solve_runs AS s ON a.run_id = s.run_id \
                 WHERE a.attempt_id = X'{hex}'"
            ),
            &token,
        )
        .await
        .unwrap();
    assert_eq!(joined.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    let row = &joined[0];
    assert_eq!(row.column(0).as_string::<i32>().value(0), "completed");
    assert_eq!(row.column(1).as_string::<i32>().value(0), "modeling");

    // Its stored progress, counted in SQL, is the stream the record holds.
    let counted = session
        .sql(
            &format!(
                "SELECT count(*) AS events FROM {OPERATIONAL_SCHEMA}.progress_events \
                 WHERE attempt_id = X'{hex}'"
            ),
            &token,
        )
        .await
        .unwrap();
    let events = counted[0]
        .column(0)
        .as_primitive::<datafusion::arrow::datatypes::Int64Type>()
        .value(0);
    assert_eq!(
        usize::try_from(events).unwrap(),
        durable.progress.as_ref().unwrap().len()
    );

    // An ephemeral runtime registers no operational tables.
    let ephemeral = runtime();
    assert!(ephemeral.operational_tables().unwrap().is_empty());
    let session = ephemeral.query_session(None, None, &token).unwrap();
    assert!(
        session
            .sql(
                &format!("SELECT * FROM {OPERATIONAL_SCHEMA}.attempts"),
                &token
            )
            .await
            .is_err()
    );
    database.remove().await.unwrap();
}

/// A running attempt of `runtime`'s store.
async fn running(runtime: &Runtime) -> (Store, AttemptId) {
    let Durability::Durable(operations) = runtime.durability() else {
        panic!("durable runtime");
    };
    let store = operations.store().clone();
    let attempt = NewAttempt {
        attempt_id: pse_operations::mint_id(),
        run_id: pse_operations::mint_id(),
        kind: AttemptKind::Modeling,
        operational_job_identity: pse_ids::roles::RecordedOperationalJobIdentity::current(
            pse_ids::roles::OperationalJobHash::from(pse_ids::ContentHash::from_bytes([2; 32])),
        ),
        preparation_identity: None,
        parent_attempt: None,
    };
    let attempts = store.attempts();
    attempts.create(&attempt, Some("test")).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("test"),
        )
        .await
        .unwrap();
    attempts
        .start(
            attempt.attempt_id,
            "worker",
            std::time::Duration::from_secs(30),
        )
        .await
        .unwrap();
    (store, attempt.attempt_id)
}

fn event(seq: i64) -> pse_operations::streams::ProgressEvent {
    pse_operations::streams::ProgressEvent {
        seq,
        step: 0,
        at: DateTime::from_timestamp_micros(1_000_000 + seq * 10).unwrap(),
        elapsed_seconds: 0.1,
        phase: "iterate".into(),
        values: BTreeMap::from([(
            "iteration".to_owned(),
            pse_operations::streams::ProgressValue::Integer(seq),
        )]),
    }
}

fn incumbent(
    attempt: AttemptId,
    seq: i64,
    at: i64,
) -> pse_operations::streams::RuntimeOperationalIncumbentsRow {
    pse_operations::streams::RuntimeOperationalIncumbentsRow {
        attempt_id: attempt,
        seq,
        step: 0,
        at,
        elapsed_seconds: 0.2,
        phase: "scip.incumbent".into(),
        objective: 3.0,
        dual_bound: Some(1.0),
        gap: Some(2.0),
        nodes: Some(5),
        seconds: Some(0.2),
        solution_id: None,
    }
}

async fn drain(stream: &mut ProgressStream) -> Vec<StreamRecord> {
    let mut records = Vec::new();
    while let Some(page) =
        tokio::time::timeout(std::time::Duration::from_secs(10), stream.next_page())
            .await
            .expect("the stream wakes on new rows and on the attempt's end")
            .unwrap()
    {
        assert!(!page.is_empty());
        records.extend(page);
    }
    records
}

#[tokio::test]
async fn progress_stream_follows_progress_and_incumbents_until_the_attempt_ends() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = durable_runtime(&database, "stream").await;
    let (store, attempt) = running(&runtime).await;
    let token = pse_columnar::CancellationToken::new();
    let mut followed = runtime.progress(attempt, true, 2, token).await.unwrap();
    let producer = tokio::spawn({
        let store = store.clone();
        async move {
            for batch in [0..3, 3..5] {
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                let events: Vec<_> = batch.map(event).collect();
                store
                    .streams()
                    .append_progress(attempt, &events)
                    .await
                    .unwrap();
            }
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            // Observed between the second and third event.
            store
                .streams()
                .record_incumbents(&[incumbent(attempt, 0, 1_000_015)], &[])
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            store
                .attempts()
                .transition(
                    attempt,
                    AttemptState::Completed,
                    &TransitionNote::by("worker"),
                )
                .await
                .unwrap();
        }
    });
    let records = drain(&mut followed).await;
    producer.await.unwrap();
    let kinds: Vec<(char, i64)> = records
        .iter()
        .map(|record| match record {
            StreamRecord::Progress(event) => ('p', event.seq),
            StreamRecord::Incumbent(incumbent) => ('i', incumbent.seq),
        })
        .collect();
    assert_eq!(kinds.len(), 6);
    assert_eq!(
        kinds.iter().filter(|(kind, _)| *kind == 'p').count(),
        5,
        "{kinds:?}"
    );
    // Within a page, records are in observation order.
    assert!(
        records
            .iter()
            .any(|r| matches!(r, StreamRecord::Incumbent(i) if i.nodes == Some(5)))
    );

    // Without following, the stream reads what is stored and ends.
    let token = pse_columnar::CancellationToken::new();
    let mut stored = runtime.progress(attempt, false, 10, token).await.unwrap();
    let again = drain(&mut stored).await;
    assert_eq!(again.len(), 6);
    let order: Vec<i64> = again.iter().map(StreamRecord::at).collect();
    assert!(order.windows(2).all(|w| w[0] <= w[1]), "{order:?}");
    assert!(stored.next_page().await.unwrap().is_none());

    // Closing a followed stream ends a waiting read.
    let (_, waiting) = running(&runtime).await;
    let token = pse_columnar::CancellationToken::new();
    let mut followed = runtime
        .progress(waiting, true, 4, token.clone())
        .await
        .unwrap();
    let closer = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        token.cancel();
    });
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(10), followed.next_page())
            .await
            .expect("cancellation ends the wait")
            .unwrap()
            .is_none()
    );
    closer.await.unwrap();

    // An unknown attempt is refused.
    assert!(
        runtime
            .progress(
                pse_operations::mint_id(),
                false,
                4,
                pse_columnar::CancellationToken::new()
            )
            .await
            .is_err()
    );
    // An ephemeral runtime has no stored streams.
    assert!(
        super::super::tests::runtime()
            .progress(attempt, false, 4, pse_columnar::CancellationToken::new())
            .await
            .is_err()
    );
    database.remove().await.unwrap();
}
