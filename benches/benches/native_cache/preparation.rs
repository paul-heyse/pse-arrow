// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordinary model preparation, distinct from resettable rule execution.
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    common::ScalarValue,
    functions_aggregate::expr_fn::sum,
    logical_expr::{
        ColumnarValue, LogicalPlan, LogicalPlanBuilder, ScalarUDF, Volatility, col, create_udf,
    },
};
use pse_columnar::CancellationToken;
use pse_engine::{EngineSession, session::PreparedComputation};
use pse_ids::SemanticId;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, FieldContract, Namespace, RelationDecl, RelationKey, SnapshotClass,
        provider::{ProviderPolicy, ProviderScope},
    },
};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

struct Fixture {
    native: pse_testkit::NativeFixture,
    registry: Arc<Registry>,
    base: EngineSession,
    key: RelationKey,
    unrelated: RelationKey,
    cancel: CancellationToken,
}
impl Fixture {
    fn new() -> Self {
        let native = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap()).unwrap();
        let mut builder = RegistryBuilder::new();
        for name in ["preparation_values", "unrelated_values"] {
            builder.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    name,
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "Model preparation measurement",
                )
                .pk(&["id"])
                .columns(vec![FieldContract::key(
                    "id",
                    FieldContract::native(DataType::Int64),
                    "identity",
                )]),
            );
        }
        let registry = Arc::new(builder.build().unwrap());
        let key = registry
            .relation("authored.preparation_values")
            .unwrap()
            .key;
        let unrelated = registry.relation("authored.unrelated_values").unwrap().key;
        let cancel = CancellationToken::new();
        let base = native
            .factory
            .clone()
            .with_observation(pse_engine::session::assurance::ObservationPolicy::Contract)
            .candidate_checked(BTreeMap::new(), registry.clone(), &cancel)
            .unwrap();
        Self {
            native,
            registry,
            base,
            key,
            unrelated,
            cancel,
        }
    }
    fn rows(&self, key: RelationKey, rows: i64, offset: i64) -> FieldCheckedBatch {
        let spec = self.registry.relation_by_key(key).unwrap();
        let schema = Arc::new(pse_schema::arrow::relation_schema(&self.registry, spec).unwrap());
        FieldCheckedBatch::admit(
            &self.registry,
            spec,
            RecordBatch::try_new(
                schema,
                vec![Arc::new(Int64Array::from_iter_values(
                    offset..offset + rows,
                ))],
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn counts(&self) -> BTreeMap<&'static str, Option<usize>> {
        self.native
            .resources
            .caches
            .execution_report()
            .into_iter()
            .collect()
    }
    fn graph(&self, session: &EngineSession, depth: usize, function: &ScalarUDF) -> LogicalPlan {
        let mut plan = LogicalPlanBuilder::scan(
            session.table_reference(&self.key).unwrap(),
            session.table_source(&self.key).unwrap(),
            None,
        )
        .unwrap()
        .project([(col("id") + function.call(vec![])).alias("value")])
        .unwrap()
        .build()
        .unwrap();
        // Every level has two edges to the same actual retained producer. The
        // independent oracle includes duplicate multiplicity, not set semantics.
        for _ in 0..depth {
            let shared = session.cache_plan(plan, &self.cancel).unwrap();
            plan = LogicalPlanBuilder::from(shared.clone())
                .union(shared)
                .unwrap()
                .build()
                .unwrap();
        }
        LogicalPlanBuilder::from(plan)
            .aggregate(
                Vec::<datafusion::logical_expr::Expr>::new(),
                [sum(col("value")).alias("total")],
            )
            .unwrap()
            .build()
            .unwrap()
    }
    async fn run(&self, label: &str, session: &EngineSession, plan: &LogicalPlan, expected: i64) {
        use tracing_subscriber::prelude::*;
        let visits = Visits::default();
        let dispatch = tracing::Dispatch::new(tracing_subscriber::registry().with(visits.clone()));
        let before = self.counts();
        let started = Instant::now();
        let mut prepared = None;
        let allocations = allocation_counter::measure(|| {
            prepared = Some(tracing::dispatcher::with_default(&dispatch, || {
                session.prepare(plan.clone(), &self.cancel).unwrap()
            }));
        });
        let prepared = prepared.unwrap();
        let preparation_seconds = started.elapsed().as_secs_f64();
        let prepared_counts = self.counts();
        let retained_after_prepare = self.native.resources.pool.reserved();
        let first = self.execute(prepared.clone(), expected).await;
        let second = self.execute(prepared, expected).await;
        println!(
            "{}",
            serde_json::json!({
                "experiment":"model_preparation", "variant":label,
                "preparation_seconds":preparation_seconds,
                "preparation_allocation_count":allocations.count_total,
                "preparation_allocated_bytes":allocations.bytes_total,
                "preparation_net_retained_heap_bytes":allocations.bytes_current,
                "preparation_heap_peak_bytes":allocations.bytes_max,
                "allocation_scope":"calling thread during synchronous preparation, including its native context and observation metadata; shared pre-existing plan allocations and other threads are excluded",
                "preparation_node_visits": visits.snapshot(),
                "preparation_node_visit_fields":["scoped_nodes","intrinsic_nodes","derived_nodes","traversals"],
                "preparation_observer":"bounded numeric counters included; repeated visits, not global distinct owners; no formatting or previews",
                "first_execution":first, "retained_preparation_execution":second,
                "counts_before":before, "counts_after_preparation":prepared_counts, "counts_after_execution":self.counts(),
                "pool_budget_bytes":256 << 20, "pool_retained_after_preparation_bytes":retained_after_prepare,
                "pool_retained_after_drop_bytes":self.native.resources.pool.reserved(),
                "pool_peak_cumulative_bytes":self.native.resources.peak.max_reserved(),
                "process_peak_rss_cumulative_bytes":super::cache_journey::process_peak_rss(),
                "accounting":"pool reservations, not an estimate of allocator or LogicalPlan heap bytes",
                "expected_sum":expected
            })
        );
    }
    async fn execute(&self, prepared: PreparedComputation, expected: i64) -> serde_json::Value {
        let started = Instant::now();
        let stream = prepared.execute_stream(&self.cancel).await.unwrap();
        let physical_preparation_and_stream_start_seconds = started.elapsed().as_secs_f64();
        let started = Instant::now();
        let result = stream.collect(&self.cancel).await.unwrap();
        let drain_and_owned_collection_seconds = started.elapsed().as_secs_f64();
        assert_eq!(
            result
                .batches()
                .iter()
                .map(|batch| batch.num_rows())
                .sum::<usize>(),
            1
        );
        assert_eq!(
            result.batches()[0]
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(0),
            expected
        );
        serde_json::json!({"physical_preparation_and_stream_start_seconds":physical_preparation_and_stream_start_seconds,
            "drain_and_owned_collection_seconds":drain_and_owned_collection_seconds})
    }
}

/// Fixed-size benchmark observation of the existing native traversal events.
#[derive(Clone, Default)]
struct Visits(Arc<[std::sync::atomic::AtomicU64; 4]>);
impl Visits {
    fn snapshot(&self) -> [u64; 4] {
        self.0
            .each_ref()
            .map(|value| value.load(std::sync::atomic::Ordering::Relaxed))
    }
}
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Visits {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        metadata.target() == "pse::admission"
    }
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if event.metadata().target() == "pse::admission" {
            event.record(&mut self.clone());
        }
    }
}
impl tracing::field::Visit for Visits {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        let index = match field.name() {
            "scoped_nodes" => {
                self.0[3].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                0
            }
            "intrinsic_nodes" => 1,
            "derived_nodes" => 2,
            _ => return,
        };
        self.0[index].fetch_add(value, std::sync::atomic::Ordering::Relaxed);
    }
    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}
}
fn function(value: i64) -> Arc<ScalarUDF> {
    Arc::new(create_udf(
        "measurement_offset",
        vec![],
        DataType::Int64,
        Volatility::Immutable,
        Arc::new(move |_| Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(value))))),
    ))
}

pub(super) async fn measure(smoke: bool) {
    for rows in if smoke { vec![16_i64] } else { vec![16, 1024] } {
        for depth in if smoke { vec![2] } else { vec![0, 4, 8] } {
            let fixture = Fixture::new();
            let original_function = function(1);
            let values = fixture
                .base
                .with_checked_workspace(
                    BTreeMap::from([(fixture.key, fixture.rows(fixture.key, rows, 0))]),
                    &fixture.cancel,
                )
                .unwrap();
            let original = values
                .with_scalar_functions(vec![original_function.clone()])
                .unwrap();
            let started = Instant::now();
            let plan = fixture.graph(&original, depth, &original_function);
            println!(
                "{}",
                serde_json::json!({"experiment":"model_preparation_input", "rows":rows,"diamond_depth":depth,
                "duplicate_multiplier":1_i64 << depth, "graph_construction_seconds":started.elapsed().as_secs_f64(),
                "conditions":"one worker and partition; Contract observation; exact native cache producers; no OS cold-cache claim"})
            );
            let expected = (rows * (rows - 1) / 2 + rows) * (1_i64 << depth);
            fixture.run("cold", &original, &plan, expected).await;
            fixture
                .run("unchanged_repeat", &original, &plan, expected)
                .await;
            let unrelated = original
                .with_checked_workspace(
                    BTreeMap::from([(
                        fixture.unrelated,
                        fixture.rows(fixture.unrelated, rows, rows),
                    )]),
                    &fixture.cancel,
                )
                .unwrap();
            fixture
                .run("unrelated_source_edit", &unrelated, &plan, expected)
                .await;
            let relevant = fixture
                .base
                .with_checked_workspace(
                    BTreeMap::from([(fixture.key, fixture.rows(fixture.key, rows, rows))]),
                    &fixture.cancel,
                )
                .unwrap()
                .with_scalar_functions(vec![original_function.clone()])
                .unwrap();
            assert!(
                relevant.prepare(plan.clone(), &fixture.cancel).is_err(),
                "old source cannot bind to a replacement merely by name"
            );
            let changed_plan = fixture.graph(&relevant, depth, &original_function);
            fixture
                .run(
                    "relevant_source_edit",
                    &relevant,
                    &changed_plan,
                    expected + rows * rows * (1_i64 << depth),
                )
                .await;
            let replacement = function(2);
            let changed = values
                .with_scalar_functions(vec![replacement.clone()])
                .unwrap();
            let changed_plan = fixture.graph(&changed, depth, &replacement);
            fixture
                .run(
                    "changed_udf_owner",
                    &changed,
                    &changed_plan,
                    expected + rows * (1_i64 << depth),
                )
                .await;
            let mut policy = ProviderPolicy::new(SemanticId::NIL, ProviderScope::Root);
            policy.max_bytes = Some(128 << 20);
            let changed = original.with_policy(policy).unwrap();
            fixture
                .run("changed_policy", &changed, &plan, expected)
                .await;
            // Retained old input and function environments still produce old values.
            fixture
                .run("retained_old_revision", &original, &plan, expected)
                .await;
        }
    }
}
