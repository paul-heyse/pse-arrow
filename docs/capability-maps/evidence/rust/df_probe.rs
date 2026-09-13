// Measurements against DataFusion 55.1.0 / Arrow 59.3.0.
// Each probe tests a specific blueprint assertion rather than trusting prose.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arrow::array::{Array, ArrayRef, FixedSizeBinaryArray, Float64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use datafusion::datasource::MemTable;
use datafusion::execution::context::SessionContext;
use datafusion::execution::session_state::SessionStateBuilder;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::prelude::SessionConfig;
use datafusion_catalog::{Session, TableProvider};
use datafusion_common::Result;
use datafusion_expr::{Expr, TableType, TableProviderFilterPushDown};

fn meta(p: &[(&str, &str)]) -> HashMap<String, String> {
    p.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn key_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::FixedSizeBinary(16), false).with_metadata(meta(&[
            ("ARROW:extension:name", "pse.semantic_id"),
            ("pse.semantic.role", "key"),
        ])),
        Field::new("kind", DataType::Utf8, false),
        Field::new("v", DataType::Float64, true),
    ]))
}

fn key_batch(n: usize) -> RecordBatch {
    let ids: Vec<[u8; 16]> = (0..n).map(|i| { let mut b = [0u8; 16]; b[15] = i as u8; b }).collect();
    let idarr = FixedSizeBinaryArray::try_from_iter(ids.into_iter()).unwrap();
    let kinds: Vec<&str> = (0..n).map(|i| if i % 2 == 0 { "a" } else { "b" }).collect();
    let vs: Vec<Option<f64>> = (0..n).map(|i| Some(i as f64)).collect();
    RecordBatch::try_new(
        key_schema(),
        vec![
            Arc::new(idarr) as ArrayRef,
            Arc::new(StringArray::from(kinds)) as ArrayRef,
            Arc::new(Float64Array::from(vs)) as ArrayRef,
        ],
    ).unwrap()
}

/// A provider that records exactly which filter expressions the planner offers it.
#[derive(Debug)]
struct RecordingProvider {
    inner: Arc<MemTable>,
    seen: Arc<Mutex<Vec<String>>>,
    scan_filters: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl TableProvider for RecordingProvider {
    fn schema(&self) -> SchemaRef { self.inner.schema() }
    fn table_type(&self) -> TableType { TableType::Base }
    async fn scan(
        &self, state: &dyn Session, projection: Option<&Vec<usize>>,
        filters: &[Expr], limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        {
            let mut g = self.scan_filters.lock().unwrap();
            for f in filters { g.push(format!("{f:?}")); }
        }
        self.inner.scan(state, projection, filters, limit).await
    }
    fn supports_filters_pushdown(&self, filters: &[&Expr]) -> Result<Vec<TableProviderFilterPushDown>> {
        let mut g = self.seen.lock().unwrap();
        for f in filters { g.push(format!("{f}")); }
        // Claim Inexact only: this probe measures the shapes offered, it does not
        // assert it can satisfy them. Claiming Exact would let the optimizer delete them.
        Ok(filters.iter().map(|_| TableProviderFilterPushDown::Inexact).collect())
    }
}

async fn probe_a() -> Result<()> {
    println!("== PROBE A: what filter shapes reach a provider on a FixedSizeBinary(16) key?");
    let mem = Arc::new(MemTable::try_new(key_schema(), vec![vec![key_batch(8)]])?);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let scan_filters = Arc::new(Mutex::new(Vec::new()));
    let p = Arc::new(RecordingProvider { inner: mem, seen: seen.clone(), scan_filters: scan_filters.clone() });
    let ctx = SessionContext::new();
    ctx.register_table("rel", p.clone())?;

    let queries = [
        "SELECT v FROM rel WHERE id = arrow_cast(X'00000000000000000000000000000003','FixedSizeBinary(16)')",
        "SELECT v FROM rel WHERE id IN (arrow_cast(X'00000000000000000000000000000001','FixedSizeBinary(16)'), arrow_cast(X'00000000000000000000000000000002','FixedSizeBinary(16)'))",
        "SELECT v FROM rel WHERE kind = 'a'",
        "SELECT v FROM rel WHERE kind = 'a' AND v > 2.0",
        "SELECT v FROM rel WHERE id IS NOT NULL",
    ];
    for q in queries {
        seen.lock().unwrap().clear();
        scan_filters.lock().unwrap().clear();
        match ctx.sql(q).await {
            Ok(df) => { let _ = df.collect().await; }
            Err(e) => { println!("   [query failed] {}", &format!("{e}")[..format!("{e}").len().min(120)]); continue; }
        }
        let offered = seen.lock().unwrap().clone();
        println!("   q: {}", &q[..q.len().min(84)]);
        println!("      offered to supports_filters_pushdown: {offered:?}");
        println!("      reached scan():                       {}", scan_filters.lock().unwrap().len());
    }
    Ok(())
}

/// Minimal codec: encodes a custom provider as its table name, which is what a
/// snapshot catalog can do (the artifact is content-addressed elsewhere).
#[derive(Debug)]
struct NameCodec { schema: SchemaRef, data: Vec<Vec<RecordBatch>> }

impl datafusion_proto::logical_plan::LogicalExtensionCodec for NameCodec {
    fn try_decode(&self, _buf: &[u8], _inputs: &[datafusion_expr::LogicalPlan],
                  _ctx: &datafusion_execution::TaskContext)
        -> Result<datafusion_expr::logical_plan::Extension> {
        datafusion_common::not_impl_err!("no extension nodes in this probe")
    }
    fn try_encode(&self, _node: &datafusion_expr::logical_plan::Extension, _buf: &mut Vec<u8>) -> Result<()> {
        datafusion_common::not_impl_err!("no extension nodes in this probe")
    }
    fn try_decode_table_provider(&self, buf: &[u8], _t: &datafusion_common::TableReference,
                                 _schema: SchemaRef, _ctx: &datafusion_execution::TaskContext)
        -> Result<Arc<dyn TableProvider>> {
        let _name = String::from_utf8_lossy(buf).to_string();
        Ok(Arc::new(MemTable::try_new(self.schema.clone(), self.data.clone())?))
    }
    fn try_encode_table_provider(&self, table_ref: &datafusion_common::TableReference,
                                 _node: Arc<dyn TableProvider>, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(table_ref.table().as_bytes());
        Ok(())
    }
}

async fn probe_c() -> Result<()> {
    use datafusion_proto::bytes::{
        logical_plan_to_bytes, logical_plan_to_bytes_with_extension_codec,
        logical_plan_from_bytes_with_extension_codec,
    };
    println!("== PROBE C: datafusion-proto encoding — usable? deterministic? stable?");
    let ctx = SessionContext::new();
    ctx.register_table("rel", Arc::new(MemTable::try_new(key_schema(), vec![vec![key_batch(4)]])?))?;
    let plan = ctx.sql("SELECT kind, sum(v) FROM rel WHERE v > 1.0 GROUP BY kind").await?
        .into_optimized_plan()?;

    match logical_plan_to_bytes(&plan) {
        Ok(b) => println!("   without a codec: encoded, len={}", b.len()),
        Err(e) => {
            let s = format!("{e}");
            println!("   without a codec: FAILS -> {}", &s[s.find("NotImplemented").unwrap_or(0)..s.len().min(s.find("NotImplemented").unwrap_or(0)+70)]);
        }
    }

    let codec = NameCodec { schema: key_schema(), data: vec![vec![key_batch(4)]] };
    let b1 = logical_plan_to_bytes_with_extension_codec(&plan, &codec)?;
    let b2 = logical_plan_to_bytes_with_extension_codec(&plan, &codec)?;
    println!("   with a codec: len={} same-plan-twice identical={}", b1.len(), b1 == b2);

    let ctx2 = SessionContext::new();
    ctx2.register_table("rel", Arc::new(MemTable::try_new(key_schema(), vec![vec![key_batch(4)]])?))?;
    let plan2 = ctx2.sql("SELECT kind, sum(v) FROM rel WHERE v > 1.0 GROUP BY kind").await?
        .into_optimized_plan()?;
    println!("   equivalent plan, fresh session:   identical={}",
        b1 == logical_plan_to_bytes_with_extension_codec(&plan2, &codec)?);

    let plan3 = ctx2.sql("SELECT   kind,   sum(v)   FROM rel   WHERE v>1.0   GROUP BY kind").await?
        .into_optimized_plan()?;
    println!("   same query, different whitespace: identical={}",
        b1 == logical_plan_to_bytes_with_extension_codec(&plan3, &codec)?);

    let plan4 = ctx2.sql("SELECT kind, sum(v) FROM rel WHERE v > 2.0 GROUP BY kind").await?
        .into_optimized_plan()?;
    println!("   different predicate:              differs={}",
        b1 != logical_plan_to_bytes_with_extension_codec(&plan4, &codec)?);

    // Does the encoding capture the *definition* of a UDF, or only its name?
    let tc = ctx.task_ctx();
    match logical_plan_from_bytes_with_extension_codec(&b1, &tc, &codec) {
        Ok(rt) => println!("   round-trip ok; re-encode identical={}",
            logical_plan_to_bytes_with_extension_codec(&rt, &codec)? == b1),
        Err(e) => { let s = format!("{e}"); println!("   round-trip FAILED: {}", &s[..s.len().min(150)]); }
    }
    Ok(())
}

async fn probe_d() -> Result<()> {
    println!("== PROBE D: null / NaN / -0.0 through the engine (sort, group by, join)");
    let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Float64, true)]));
    let b = RecordBatch::try_new(schema.clone(), vec![Arc::new(Float64Array::from(vec![
        Some(0.0), Some(-0.0), Some(f64::NAN), Some(-1.0), None,
    ])) as ArrayRef])?;
    let ctx = SessionContext::new();
    ctx.register_table("t", Arc::new(MemTable::try_new(schema, vec![vec![b]])?))?;

    let show = |batches: &Vec<RecordBatch>| -> Vec<String> {
        let mut out = Vec::new();
        for bt in batches {
            let c = bt.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
            for i in 0..c.len() {
                out.push(if c.is_null(i) { "null".to_string() } else {
                    let v = c.value(i);
                    format!("{}{}", if v.is_sign_negative() { "-" } else { "" },
                            if v.is_nan() { "NaN".to_string() } else { v.abs().to_string() })
                });
            }
        }
        out
    };
    let r = ctx.sql("SELECT x FROM t ORDER BY x").await?.collect().await?;
    println!("   ORDER BY x            : {:?}", show(&r));
    let r = ctx.sql("SELECT x FROM t GROUP BY x ORDER BY x").await?.collect().await?;
    println!("   GROUP BY x (distinct) : {:?}   <- does -0.0 group with 0.0?", show(&r));
    let r = ctx.sql("SELECT count(*) c FROM t a JOIN t b ON a.x = b.x").await?.collect().await?;
    println!("   self equi-join rows   : {:?}", r[0].column(0));
    Ok(())
}

async fn probe_e() -> Result<()> {
    println!("== PROBE E: does an ARROW:extension:name field survive planning and execution?");
    let ctx = SessionContext::new();
    ctx.register_table("rel", Arc::new(MemTable::try_new(key_schema(), vec![vec![key_batch(4)]])?))?;
    let df = ctx.sql("SELECT id, v FROM rel").await?;
    let out_schema = df.schema().clone();
    println!("   planned output field 'id' metadata : {:?}",
        out_schema.field_with_unqualified_name("id").unwrap().metadata().get("ARROW:extension:name"));
    let batches = df.collect().await?;
    println!("   executed batch  field 'id' metadata: {:?}",
        batches[0].schema().field(0).metadata().get("ARROW:extension:name"));

    // Through a projection that renames
    let df2 = ctx.sql("SELECT id AS key_out FROM rel").await?;
    let b2 = df2.collect().await?;
    println!("   after `AS key_out`                 : {:?}",
        b2[0].schema().field(0).metadata().get("ARROW:extension:name"));

    // Through an aggregate grouping key
    let df3 = ctx.sql("SELECT id, count(*) FROM rel GROUP BY id").await?;
    let b3 = df3.collect().await?;
    println!("   as a GROUP BY key                  : {:?}",
        b3[0].schema().field(0).metadata().get("ARROW:extension:name"));
    Ok(())
}

async fn probe_h() -> Result<()> {
    use datafusion_execution::memory_pool::GreedyMemoryPool;
    use datafusion_execution::runtime_env::RuntimeEnvBuilder;
    println!("== PROBE H: bounded MemoryPool — typed failure or process death?");
    let rt = RuntimeEnvBuilder::new()
        .with_memory_pool(Arc::new(GreedyMemoryPool::new(64 * 1024)))
        .build_arc()?;
    let state = SessionStateBuilder::new()
        .with_default_features()
        .with_config(SessionConfig::new().with_target_partitions(2))
        .with_runtime_env(rt)
        .build();
    let ctx = SessionContext::new_with_state(state);
    let n = 200_000usize;
    let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Float64, false)]));
    let b = RecordBatch::try_new(schema.clone(),
        vec![Arc::new(Float64Array::from((0..n).map(|i| (i % 1000) as f64).collect::<Vec<_>>())) as ArrayRef])?;
    ctx.register_table("big", Arc::new(MemTable::try_new(schema, vec![vec![b]])?))?;
    match ctx.sql("SELECT x FROM big ORDER BY x").await?.collect().await {
        Ok(_) => println!("   sort completed within 64 KiB pool (no pressure)"),
        Err(e) => {
            let s = format!("{e}");
            println!("   error variant : {}", s.split(':').next().unwrap_or("?"));
            println!("   message       : {}", &s[..s.len().min(190)]);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    probe_a().await?;
    println!();
    probe_c().await?;
    println!();
    probe_d().await?;
    println!();
    probe_e().await?;
    println!();
    probe_h().await?;
    Ok(())
}
