//! Public consumer assertions. Captures retain IDs, parentage, order and field values.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use datafusion::arrow::array::{Int64Array, RecordBatch};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::common::config::ConfigOptions;
use datafusion::datasource::MemTable;
use datafusion::error::Result;
use datafusion::execution::SessionStateBuilder;
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::limit::GlobalLimitExec;
use datafusion::physical_plan::{ExecutionPlan, ExecutionPlanProperties};
use datafusion::prelude::*;
use datafusion_tracing::{
    InstrumentationOptions, RuleInstrumentationOptions, instrument_rules_with_info_spans,
    instrument_with_info_spans,
};
use futures::TryStreamExt;
use object_store::{ObjectStore, ObjectStoreExt, memory::InMemory, path::Path};
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{InMemorySpanExporter, Sampler, SdkTracerProvider};
use serde_json::{Value, json};
use tracing::instrument::{Instrument, WithSubscriber};
use tracing::{Id, Subscriber, field};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Value>>>);

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl field::Visit for Fields {
    fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }
    fn record_str(&mut self, field: &field::Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);
        let span = ctx.span(id).unwrap();
        let parent = span.parent().map(|p| p.id().into_u64());
        self.0.lock().unwrap().push(json!({
            "kind":"open", "id":id.into_u64(), "parent":parent,
            "name":attrs.metadata().name(), "target":attrs.metadata().target(),
            "fields":fields.0
        }));
        span.extensions_mut().insert(fields);
    }
    fn on_record(&self, id: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        values.record(extensions.get_mut::<Fields>().unwrap());
    }
    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        let extensions = span.extensions();
        self.0.lock().unwrap().push(json!({
            "kind":"close", "id":id.into_u64(), "name":span.metadata().name(),
            "target":span.metadata().target(), "fields":extensions.get::<Fields>().unwrap().0
        }));
    }
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        self.0
            .lock()
            .unwrap()
            .push(json!({"kind":"event","fields":fields.0}));
    }
}

impl Capture {
    fn rows(&self, kind: &str, name: &str) -> Vec<Value> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|v| v["kind"] == kind && v["name"] == name)
            .cloned()
            .collect()
    }
    fn save(&self, scenario: &str) {
        if let Ok(folder) = std::env::var("TRACE_PROBE_OUTPUT") {
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(
                std::path::Path::new(&folder).join(format!("{scenario}.json")),
                serde_json::to_vec_pretty(&*self.0.lock().unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
}

fn state(options: InstrumentationOptions) -> SessionStateBuilder {
    SessionStateBuilder::new()
        .with_default_features()
        .with_physical_optimizer_rule(
            instrument_with_info_spans!(options: options, query_id = field::Empty),
        )
}

async fn run_query(ctx: &SessionContext) -> Result<Vec<RecordBatch>> {
    ctx.sql("SELECT a FROM (VALUES (1), (2), (3)) AS t(a)")
        .await?
        .collect()
        .await
}

async fn run_preview(options: InstrumentationOptions, label: &str) -> Result<Capture> {
    let capture = Capture::default();
    let guard = tracing::subscriber::set_default(Registry::default().with(capture.clone()));
    let ctx = SessionContext::new_with_state(state(options).build());
    let batches = run_query(&ctx).await?;
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 3);
    drop(ctx);
    drop(guard);
    capture.save(label);
    Ok(capture)
}

#[tokio::test]
async fn preview_default_custom_disabled_and_error() -> Result<()> {
    let default = run_preview(
        InstrumentationOptions::builder().preview_limit(2).build(),
        "preview-default",
    )
    .await?;
    assert!(default.rows("close", "InstrumentedExec").iter().any(|s| {
        let text = s["fields"]["datafusion.preview"].as_str().unwrap_or("");
        text.contains("| 1 |") && text.contains("| 2 |") && !text.contains("| 3 |")
    }));
    let custom = run_preview(
        InstrumentationOptions::builder()
            .preview_limit(2)
            .preview_fn(Arc::new(|b| Ok(format!("CUSTOM:{}", b.num_rows()))))
            .build(),
        "preview-custom",
    )
    .await?;
    assert!(
        custom
            .rows("close", "InstrumentedExec")
            .iter()
            .any(|s| s["fields"]["datafusion.preview"] == "CUSTOM:2")
    );
    let disabled = run_preview(InstrumentationOptions::default(), "preview-disabled").await?;
    assert!(
        disabled
            .rows("close", "InstrumentedExec")
            .iter()
            .all(|s| s["fields"].get("datafusion.preview").is_none())
    );
    let error = run_preview(
        InstrumentationOptions::builder()
            .preview_limit(2)
            .preview_fn(Arc::new(|_| {
                Err(datafusion::arrow::error::ArrowError::ComputeError(
                    "formatter-control".into(),
                ))
            }))
            .build(),
        "preview-error",
    )
    .await?;
    assert!(
        error
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|v| v["kind"] == "event" && v.to_string().contains("formatter-control"))
    );
    assert!(
        error
            .rows("close", "InstrumentedExec")
            .iter()
            .all(|s| s["fields"].get("datafusion.preview").is_none())
    );
    Ok(())
}

async fn partitioned_plan(
    options: InstrumentationOptions,
) -> Result<(SessionContext, Arc<dyn ExecutionPlan>)> {
    let ctx = SessionContext::new_with_state(state(options).build());
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int64, false)]));
    let batch = |values: Vec<i64>| {
        RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(values))]).unwrap()
    };
    ctx.register_table(
        "t",
        Arc::new(MemTable::try_new(
            schema.clone(),
            vec![
                vec![batch(vec![1, 2]), batch(vec![3, 4])],
                vec![batch(vec![5, 6])],
            ],
        )?),
    )?;
    let plan = ctx.table("t").await?.create_physical_plan().await?;
    Ok((ctx, plan))
}

#[tokio::test]
async fn partition_preview_cap_and_live_stream_lifetime() -> Result<()> {
    let c = Capture::default();
    let _guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
    let (ctx, plan) = partitioned_plan(
        InstrumentationOptions::builder()
            .preview_limit(3)
            .preview_fn(Arc::new(|b| Ok(format!("ROWS:{}", b.num_rows()))))
            .build(),
    )
    .await?;
    assert_eq!(plan.output_partitioning().partition_count(), 2);
    let task = ctx.task_ctx();
    let mut streams = (0..2)
        .map(|p| plan.execute(p, task.clone()))
        .collect::<Result<Vec<_>>>()?;
    assert!(c.rows("close", "InstrumentedExec").is_empty());
    let retained_plan = plan.clone();
    let mut count = 0;
    for stream in &mut streams {
        while let Some(batch) = stream.try_next().await? {
            count += batch.num_rows();
        }
    }
    assert_eq!(count, 6);
    // EOF alone does not release ExecutionRecordingStream's reservation.
    assert!(c.rows("close", "InstrumentedExec").is_empty());
    c.save("partition-preview-at-eof");
    drop(streams);
    let closed = c.rows("close", "InstrumentedExec");
    assert!(!closed.is_empty());
    assert!(
        closed
            .iter()
            .any(|s| s["fields"]["datafusion.preview"] == "ROWS:3")
    );
    let before = closed.len();
    drop(retained_plan);
    drop(plan);
    assert_eq!(c.rows("close", "InstrumentedExec").len(), before);
    c.save("partition-preview-lifetime");
    Ok(())
}

#[tokio::test]
async fn early_drop_partial_partition_and_repeated_execution() -> Result<()> {
    let c = Capture::default();
    let _guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
    let (ctx, plan) = partitioned_plan(
        InstrumentationOptions::builder()
            .preview_limit(10)
            .preview_fn(Arc::new(|b| Ok(format!("ROWS:{}", b.num_rows()))))
            .build(),
    )
    .await?;
    let mut stream = plan.execute(0, ctx.task_ctx())?;
    assert_eq!(stream.try_next().await?.unwrap().num_rows(), 2);
    assert!(c.rows("close", "InstrumentedExec").is_empty());
    drop(stream);
    let first = c.rows("close", "InstrumentedExec");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0]["fields"]["datafusion.preview"], "ROWS:2");
    let batches = plan
        .execute(0, ctx.task_ctx())?
        .try_collect::<Vec<_>>()
        .await?;
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 4);
    let closed = c.rows("close", "InstrumentedExec");
    assert_eq!(closed.len(), 2);
    assert_ne!(closed[0]["id"], closed[1]["id"]);
    assert_eq!(closed[1]["fields"]["datafusion.preview"], "ROWS:4");
    c.save("early-drop-repeat");
    Ok(())
}

#[tokio::test]
async fn independent_execution_groups_do_not_mix_previews() -> Result<()> {
    let c = Capture::default();
    let _guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
    let (ctx, plan) = partitioned_plan(
        InstrumentationOptions::builder()
            .preview_limit(10)
            .preview_fn(Arc::new(|b| Ok(format!("ROWS:{}", b.num_rows()))))
            .build(),
    )
    .await?;
    let one = plan.execute(0, ctx.task_ctx())?;
    let two = plan.execute(1, ctx.task_ctx())?;
    let _ = one.try_collect::<Vec<_>>().await?;
    let _ = two.try_collect::<Vec<_>>().await?;
    let values: BTreeSet<_> = c
        .rows("close", "InstrumentedExec")
        .iter()
        .map(|v| {
            v["fields"]["datafusion.preview"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(values, BTreeSet::from(["ROWS:4".into(), "ROWS:2".into()]));
    c.save("independent-executions");
    Ok(())
}

#[tokio::test]
async fn metrics_and_custom_fields_have_separate_controls() -> Result<()> {
    let enabled = run_preview(
        InstrumentationOptions::builder()
            .record_metrics(true)
            .add_custom_field("query_id", "abc")
            .add_custom_field("undeclared", "lost")
            .build(),
        "metrics-fields-on",
    )
    .await?;
    let closed = enabled.rows("close", "InstrumentedExec");
    assert!(
        closed
            .iter()
            .any(|s| s["fields"].get("datafusion.metrics.output_rows").is_some())
    );
    assert!(
        closed
            .iter()
            .all(|s| s["fields"]["query_id"] == "abc" && s["fields"].get("undeclared").is_none())
    );
    let disabled = run_preview(InstrumentationOptions::default(), "metrics-fields-off").await?;
    assert!(disabled.rows("close", "InstrumentedExec").iter().all(|s| {
        !s["fields"]
            .as_object()
            .unwrap()
            .keys()
            .any(|k| k.starts_with("datafusion.metrics."))
    }));
    Ok(())
}

#[tokio::test]
async fn phase_selection_and_rule_parentage() -> Result<()> {
    let mut results = Vec::new();
    for (label, options) in [
        ("phase-only", RuleInstrumentationOptions::phase_only()),
        ("rules-full", RuleInstrumentationOptions::full()),
        (
            "physical-only",
            RuleInstrumentationOptions::builder()
                .physical_optimizer()
                .build(),
        ),
    ] {
        let c = Capture::default();
        let guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
        let state = SessionStateBuilder::new().with_default_features().build();
        let state = instrument_rules_with_info_spans!(options: options,state: state);
        let ctx = SessionContext::new_with_state(state);
        assert_eq!(
            run_query(&ctx)
                .await?
                .iter()
                .map(RecordBatch::num_rows)
                .sum::<usize>(),
            3
        );
        drop(ctx);
        drop(guard);
        c.save(label);
        results.push(c);
    }
    let phases = |c: &Capture| {
        c.rows("open", "Phase")
            .iter()
            .map(|v| v["fields"]["otel.name"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>()
    };
    assert!(results[0].rows("open", "Rule").is_empty());
    assert!(!results[1].rows("open", "Rule").is_empty());
    assert_eq!(phases(&results[0]), phases(&results[1]));
    assert_eq!(
        phases(&results[2]),
        BTreeSet::from(["optimize_physical_plan".into()])
    );
    let parent_ids: BTreeSet<_> = results[1]
        .rows("open", "Phase")
        .iter()
        .map(|v| v["id"].as_u64().unwrap())
        .collect();
    assert!(results[1].rows("open", "Rule").iter().all(|v| {
        v["parent"]
            .as_u64()
            .is_some_and(|id| parent_ids.contains(&id))
    }));
    Ok(())
}

#[derive(Debug)]
struct AddOuterNode;
impl PhysicalOptimizerRule for AddOuterNode {
    fn optimize(
        &self,
        plan: Arc<dyn ExecutionPlan>,
        _: &ConfigOptions,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(GlobalLimitExec::new(plan, 0, None)))
    }
    fn name(&self) -> &str {
        "contract_add_outer"
    }
    fn schema_check(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn later_rewrite_changes_observed_node_coverage() -> Result<()> {
    let mut observed = Vec::new();
    for instrument_last in [false, true] {
        let c = Capture::default();
        let guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
        let base = SessionStateBuilder::new().with_default_features();
        let rule = instrument_with_info_spans!(options: InstrumentationOptions::default());
        let builder = if instrument_last {
            base.with_physical_optimizer_rule(Arc::new(AddOuterNode))
                .with_physical_optimizer_rule(rule)
        } else {
            base.with_physical_optimizer_rule(rule)
                .with_physical_optimizer_rule(Arc::new(AddOuterNode))
        };
        let ctx = SessionContext::new_with_state(builder.build());
        assert_eq!(
            run_query(&ctx)
                .await?
                .iter()
                .map(RecordBatch::num_rows)
                .sum::<usize>(),
            3
        );
        drop(ctx);
        drop(guard);
        let saw = c
            .rows("close", "InstrumentedExec")
            .iter()
            .any(|v| v["fields"]["otel.name"] == "GlobalLimitExec");
        observed.push(saw);
        c.save(if instrument_last {
            "instrument-last"
        } else {
            "rewrite-last"
        });
    }
    assert_eq!(observed, vec![false, true]);
    Ok(())
}

#[tokio::test]
async fn explicit_target_and_filter_control() -> Result<()> {
    let mut counts = Vec::new();
    for (explicit, filter) in [
        (true, "contract_explicit=info"),
        (true, "contract_missing=info"),
        (false, "contracts=info"),
    ] {
        let c = Capture::default();
        let guard = tracing::subscriber::set_default(
            Registry::default().with(
                c.clone()
                    .with_filter(tracing_subscriber::EnvFilter::new(filter)),
            ),
        );
        let rule = if explicit {
            instrument_with_info_spans!(target:"contract_explicit", options: InstrumentationOptions::default())
        } else {
            instrument_with_info_spans!(options: InstrumentationOptions::default())
        };
        let ctx = SessionContext::new_with_state(
            SessionStateBuilder::new()
                .with_default_features()
                .with_physical_optimizer_rule(rule)
                .build(),
        );
        run_query(&ctx).await?;
        drop(ctx);
        drop(guard);
        counts.push(c.rows("open", "InstrumentedExec").len());
        c.save(&format!("target-{}", counts.len()));
    }
    assert!(counts[0] > 0 && counts[1] == 0 && counts[2] > 0);
    Ok(())
}

#[tokio::test]
async fn spawned_future_propagates_parent_and_dispatch() {
    let c = Capture::default();
    let dispatch = tracing::Dispatch::new(Registry::default().with(c.clone()));
    let parent = tracing::dispatcher::with_default(&dispatch, || tracing::info_span!("parent"));
    let parent_id = parent.id().unwrap().into_u64();
    tokio::spawn(
        async {
            let _child = tracing::info_span!("child");
        }
        .instrument(parent.clone())
        .with_subscriber(dispatch.clone()),
    )
    .await
    .unwrap();
    tokio::spawn(
        async {
            let _child = tracing::info_span!("unparented");
        }
        .with_subscriber(dispatch),
    )
    .await
    .unwrap();
    drop(parent);
    assert_eq!(c.rows("open", "child")[0]["parent"], parent_id);
    assert!(c.rows("open", "unparented")[0]["parent"].is_null());
    c.save("async-parent-dispatch");
}

#[tokio::test]
async fn storage_wrapper_result_error_and_lazy_lifetime() -> object_store::Result<()> {
    let store: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
    let path = Path::from("sample");
    store.put(&path, b"abcdef".to_vec().into()).await?;
    let c = Capture::default();
    let _guard = tracing::subscriber::set_default(Registry::default().with(c.clone()));
    assert_eq!(store.get(&path).await?.bytes().await?.as_ref(), b"abcdef");
    assert!(c.rows("open", "get_opts").is_empty());
    let wrapped = instrumented_object_store::instrument_object_store(store, "fixture");
    let get = wrapped.get(&path).await?;
    assert_eq!(c.rows("close", "get_opts").len(), 1);
    assert_eq!(get.bytes().await?.as_ref(), b"abcdef");
    assert_eq!(c.rows("close", "get_opts").len(), 1);
    let ranges = wrapped.get_ranges(&path, &[0..2, 4..6]).await?;
    assert_eq!(
        ranges.iter().map(|b| b.as_ref()).collect::<Vec<_>>(),
        vec![b"ab".as_slice(), b"ef".as_slice()]
    );
    assert!(wrapped.get(&Path::from("missing")).await.is_err());
    assert!(
        c.rows("close", "get_opts")
            .iter()
            .any(|v| v["fields"].get("object_store.result.err").is_some())
    );
    let listing = wrapped.list(None);
    assert!(c.rows("close", "list").is_empty());
    assert_eq!(listing.try_collect::<Vec<_>>().await?.len(), 1);
    assert_eq!(c.rows("close", "list").len(), 1);
    c.save("store-lifetimes");
    Ok(())
}

#[test]
fn local_layer_and_otel_sampling_are_independent() {
    for (label, sampler, expected) in [
        ("sdk-on", Sampler::AlwaysOn, 1),
        ("sdk-off", Sampler::AlwaysOff, 0),
    ] {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_sampler(sampler)
            .with_simple_exporter(exporter.clone())
            .build();
        let c = Capture::default();
        let subscriber = Registry::default()
            .with(c.clone())
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("contracts")));
        tracing::subscriber::with_default(subscriber, || {
            let _span = tracing::info_span!("local-name", otel.name = "exported-name", value = 7);
        });
        provider.force_flush().unwrap();
        let spans = exporter.get_finished_spans().unwrap();
        assert_eq!(spans.len(), expected);
        assert_eq!(c.rows("close", "local-name").len(), 1);
        if expected == 1 {
            assert_eq!(spans[0].name, "exported-name");
        }
        c.save(label);
        provider.shutdown().unwrap();
    }
}

#[test]
fn batch_flush_exports_completed_spans_and_shutdown_is_observed() {
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter.clone())
        .build();
    let c = Capture::default();
    let subscriber = Registry::default()
        .with(c.clone())
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("contracts")));
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("batch-control");
        provider.force_flush().unwrap();
        assert!(exporter.get_finished_spans().unwrap().is_empty());
        drop(span);
    });
    provider.force_flush().unwrap();
    let spans = exporter.get_finished_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "batch-control");
    provider.shutdown().unwrap();
    assert!(exporter.get_finished_spans().unwrap().is_empty());
    c.save("batch-flush-shutdown");
}
