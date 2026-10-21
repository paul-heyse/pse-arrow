//! Throwaway design-review probe: DataFusion float semantics, statistics pruning over
//! custom containers, FixedSizeBinary key pruning. Not repository code.

use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanArray, FixedSizeBinaryArray, Float64Array, Int64Array, UInt8Array,
};
use arrow::compute::kernels::cmp::{eq, gt_eq, lt_eq};
use arrow::compute::{and_kleene, filter, is_not_null, is_null};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use datafusion::datasource::MemTable;
use datafusion::prelude::*;
use datafusion_common::pruning::PrunableStatistics;
use datafusion_common::stats::Precision;
use datafusion_common::{ColumnStatistics, DFSchema, ScalarValue, Statistics};
use datafusion_expr::execution_props::ExecutionProps;
use datafusion_expr::Expr;
use datafusion_physical_expr::{create_physical_expr, PhysicalExpr};
use datafusion_pruning::PruningPredicateBuilder;

fn pruner(p: Arc<dyn PhysicalExpr>, s: &SchemaRef) -> datafusion_pruning::PruningPredicate {
    PruningPredicateBuilder::new().with_file_schema(s.clone()).try_build(p).unwrap()
}

// ---------------------------------------------------------------- oracle
/// Faithful reimplementation of pse-runtime `scalar_predicate` (result_projection.rs),
/// minus the relation field list check.
fn oracle(
    values: &Float64Array,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
) -> Result<BooleanArray, String> {
    if minimum.is_some_and(|v| !v.is_finite())
        || maximum.is_some_and(|v| !v.is_finite())
        || minimum.zip(maximum).is_some_and(|(l, u)| l > u)
        || (missing == Some(true) && (minimum.is_some() || maximum.is_some()))
    {
        return Err("invalid scalar access predicate".into());
    }
    let k = |e: arrow::error::ArrowError| e.to_string();
    if missing == Some(true) {
        return is_null(values).map_err(k);
    }
    let mut mask = if missing == Some(false) || minimum.is_some() || maximum.is_some() {
        let finite: UInt8Array = values.unary(|v| u8::from(v.is_finite()));
        let finite = eq(&finite, &UInt8Array::new_scalar(1)).map_err(k)?;
        and_kleene(&is_not_null(values).map_err(k)?, &finite).map_err(k)?
    } else {
        BooleanArray::from(vec![true; values.len()])
    };
    if minimum.is_some() || maximum.is_some() {
        let comparison: Float64Array = values.unary(|v| if v == 0.0 { 0.0 } else { v });
        if let Some(lower) = minimum {
            mask = and_kleene(
                &mask,
                &gt_eq(
                    &comparison,
                    &Float64Array::new_scalar(if lower == 0.0 { 0.0 } else { lower }),
                )
                .map_err(k)?,
            )
            .map_err(k)?;
        }
        if let Some(upper) = maximum {
            mask = and_kleene(
                &mask,
                &lt_eq(
                    &comparison,
                    &Float64Array::new_scalar(if upper == 0.0 { 0.0 } else { upper }),
                )
                .map_err(k)?,
            )
            .map_err(k)?;
        }
    }
    Ok(mask)
}

/// C1 lowering: plain DataFusion Expr, no copy. None = no predicate.
fn lower(minimum: Option<f64>, maximum: Option<f64>, missing: Option<bool>) -> Option<Expr> {
    if missing == Some(true) {
        return Some(col("x").is_null());
    }
    let mut parts = Vec::new();
    if missing == Some(false) || minimum.is_some() || maximum.is_some() {
        parts.push(col("x").gt(lit(f64::NEG_INFINITY)));
        parts.push(col("x").lt(lit(f64::INFINITY)));
    }
    if let Some(m) = minimum {
        parts.push(col("x").gt_eq(lit(if m == 0.0 { -0.0f64 } else { m })));
    }
    if let Some(m) = maximum {
        parts.push(col("x").lt_eq(lit(if m == 0.0 { 0.0f64 } else { m })));
    }
    parts.into_iter().reduce(Expr::and)
}

// ---------------------------------------------------------------- helpers
fn schema_x() -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new("x", DataType::Float64, true)]))
}

fn physical(expr: &Expr, schema: &SchemaRef) -> Arc<dyn PhysicalExpr> {
    let df = DFSchema::try_from(schema.as_ref().clone()).unwrap();
    create_physical_expr(
        expr,
        &df,
        &ExecutionProps::new(),
        &datafusion_expr::physical_planning_context::PhysicalPlanningContext::default(),
    )
    .unwrap()
}

fn eval_mask(p: &Arc<dyn PhysicalExpr>, batch: &RecordBatch) -> BooleanArray {
    let a = p.evaluate(batch).unwrap().into_array(batch.num_rows()).unwrap();
    a.as_any().downcast_ref::<BooleanArray>().unwrap().clone()
}

fn float_stats(batch: &RecordBatch) -> Arc<Statistics> {
    let x = batch.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
    let mut c = ColumnStatistics::new_unknown();
    c.null_count = Precision::Exact(x.null_count());
    c.min_value = Precision::Exact(ScalarValue::Float64(arrow::compute::min(x)));
    c.max_value = Precision::Exact(ScalarValue::Float64(arrow::compute::max(x)));
    Arc::new(Statistics {
        num_rows: Precision::Exact(batch.num_rows()),
        total_byte_size: Precision::Absent,
        column_statistics: vec![c],
    })
}

fn true_count(m: &BooleanArray) -> usize {
    (0..m.len()).filter(|&i| m.is_valid(i) && m.value(i)).count()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn grid() -> Vec<(&'static str, Option<f64>)> {
    let b = f64::from_bits;
    vec![
        ("+NaN", Some(f64::NAN)),
        ("-NaN", Some(-f64::NAN)),
        ("+sNaN(1)", Some(b(0x7ff0_0000_0000_0001))),
        ("+qNaN(payload)", Some(b(0x7ff8_dead_beef_0001))),
        ("+NaN(max payload)", Some(b(0x7fff_ffff_ffff_ffff))),
        ("-sNaN(1)", Some(b(0xfff0_0000_0000_0001))),
        ("-NaN(max payload)", Some(b(0xffff_ffff_ffff_ffff))),
        ("+0", Some(0.0)),
        ("-0", Some(-0.0)),
        ("+inf", Some(f64::INFINITY)),
        ("-inf", Some(f64::NEG_INFINITY)),
        ("+subnormal", Some(b(1))),
        ("-subnormal", Some(b(0x8000_0000_0000_0001))),
        ("MIN_POSITIVE", Some(f64::MIN_POSITIVE)),
        ("MAX", Some(f64::MAX)),
        ("-MAX", Some(-f64::MAX)),
        ("-5", Some(-5.0)),
        ("-1", Some(-1.0)),
        ("-0.5", Some(-0.5)),
        ("0.5", Some(0.5)),
        ("1", Some(1.0)),
        ("5", Some(5.0)),
        ("7", Some(7.0)),
        ("1e300", Some(1e300)),
        ("-1e300", Some(-1e300)),
        ("null", None),
    ]
}

fn batch_of(schema: &SchemaRef, vals: &[Option<f64>]) -> RecordBatch {
    RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(Float64Array::from(vals.to_vec())) as ArrayRef],
    )
    .unwrap()
}

fn containers(schema: &SchemaRef) -> Vec<RecordBatch> {
    let g: Vec<Option<f64>> = grid().into_iter().map(|(_, v)| v).collect();
    let mut out = Vec::new();
    // singletons
    for v in &g {
        out.push(batch_of(schema, &[*v]));
    }
    // every unordered pair (incl. self-pairs)
    for i in 0..g.len() {
        for j in i..g.len() {
            out.push(batch_of(schema, &[g[i], g[j]]));
        }
    }
    // named homogeneous / mixed containers
    let b = f64::from_bits;
    let named: Vec<Vec<Option<f64>>> = vec![
        vec![Some(f64::NAN); 8],
        vec![Some(-f64::NAN), Some(b(0xfff0_0000_0000_0001)), Some(b(0xffff_ffff_ffff_ffff))],
        vec![Some(f64::NAN), Some(-f64::NAN), Some(b(0x7ff0_0000_0000_0001))],
        vec![Some(-0.0); 8],
        vec![Some(0.0), Some(-0.0), Some(0.0)],
        vec![None; 8],
        vec![None, Some(f64::NAN), Some(-f64::NAN), Some(f64::INFINITY), Some(f64::NEG_INFINITY)],
        vec![Some(1.0), Some(2.0), Some(3.0), Some(f64::NAN)],
        vec![Some(-f64::NAN), Some(1.0), Some(2.0)],
        vec![Some(6.0), Some(7.0), Some(f64::NAN)],
        vec![Some(-0.0), Some(1.0), None],
        vec![Some(-3.0), Some(-0.0)],
        vec![Some(0.0), Some(4.0)],
        vec![Some(1e301), Some(f64::MAX), Some(f64::INFINITY)],
    ];
    for v in named {
        out.push(batch_of(schema, &v));
    }
    // random mixed containers of size 1..=12
    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..3000 {
        let n = 1 + r.below(12);
        let vals: Vec<Option<f64>> = (0..n).map(|_| g[r.below(g.len())]).collect();
        out.push(batch_of(schema, &vals));
    }
    out
}

fn predicate_grid() -> Vec<(Option<f64>, Option<f64>, Option<bool>)> {
    let mins = [None, Some(0.0), Some(-0.0), Some(-1.0), Some(1.0), Some(5.0)];
    let maxs = [None, Some(0.0), Some(-0.0), Some(1.0), Some(5.0), Some(1e300)];
    let miss = [None, Some(true), Some(false)];
    let mut v = Vec::new();
    for &a in &mins {
        for &b in &maxs {
            for &c in &miss {
                v.push((a, b, c));
            }
        }
    }
    // non-finite bounds must be refused
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        v.push((Some(bad), None, None));
        v.push((None, Some(bad), None));
    }
    v
}

fn fmt_opt(v: Option<f64>) -> String {
    match v {
        None => "-".into(),
        Some(x) if x == 0.0 && x.is_sign_negative() => "-0".into(),
        Some(x) => format!("{x}"),
    }
}

fn sorted_bits(a: &Float64Array) -> Vec<Option<u64>> {
    let mut v: Vec<Option<u64>> = (0..a.len())
        .map(|i| a.is_valid(i).then(|| a.value(i).to_bits()))
        .collect();
    v.sort();
    v
}

// ---------------------------------------------------------------- probe 2
async fn probe2() {
    println!("\n## Probe 2: DataFusion float semantics\n");
    let ctx = SessionContext::new();
    let s = schema_x();
    let b = f64::from_bits;
    let tables: Vec<(&str, Vec<Option<f64>>)> = vec![
        ("t_nan", vec![Some(f64::NAN)]),
        ("t_max_nan", vec![Some(1.0), Some(f64::NAN), Some(5.0)]),
        ("t_max_negnan", vec![Some(1.0), Some(-f64::NAN), Some(5.0)]),
        ("t_zero_pm", vec![Some(0.0), Some(-0.0)]),
        ("t_zero_mp", vec![Some(-0.0), Some(0.0)]),
        ("t_zero_pm_one", vec![Some(1.0), Some(0.0), Some(-0.0), Some(2.0)]),
        ("t_payload", vec![Some(b(0x7ff8_dead_beef_0001)), Some(f64::NAN)]),
    ];
    for (name, vals) in &tables {
        let t = MemTable::try_new(s.clone(), vec![vec![batch_of(&s, vals)]]).unwrap();
        ctx.register_table(*name, Arc::new(t)).unwrap();
    }
    ctx.register_table(
        "t_ab",
        Arc::new(
            MemTable::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("a", DataType::Float64, true),
                    Field::new("b", DataType::Float64, true),
                ])),
                vec![vec![
                    RecordBatch::try_new(
                        Arc::new(Schema::new(vec![
                            Field::new("a", DataType::Float64, true),
                            Field::new("b", DataType::Float64, true),
                        ])),
                        vec![
                            Arc::new(Float64Array::from(vec![-0.0, f64::NAN])),
                            Arc::new(Float64Array::from(vec![0.0, 5.0])),
                        ],
                    )
                    .unwrap(),
                ]],
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let queries = [
        "SELECT CAST('NaN' AS DOUBLE) > 5 AS r",
        "SELECT CAST('NaN' AS DOUBLE) > CAST('Infinity' AS DOUBLE) AS r",
        "SELECT -0.0 = 0.0 AS r",
        "SELECT CAST(-0.0 AS DOUBLE) = CAST(0.0 AS DOUBLE) AS r",
        "SELECT CAST(-0.0 AS DOUBLE) < CAST(0.0 AS DOUBLE) AS r",
        "SELECT x > 5 AS r FROM t_nan",
        "SELECT x = x AS r FROM t_nan",
        "SELECT a = b AS r FROM t_ab",
        "SELECT a < b AS r FROM t_ab",
        "SELECT count(*) AS r FROM t_zero_pm WHERE x = 0.0",
        "SELECT count(*) AS r FROM t_zero_pm WHERE x = -0.0",
        "SELECT count(*) AS r FROM t_zero_pm WHERE x >= 0.0",
        "SELECT max(x) AS r FROM t_max_nan",
        "SELECT min(x) AS r FROM t_max_nan",
        "SELECT max(x) AS r FROM t_max_negnan",
        "SELECT min(x) AS r FROM t_max_negnan",
        "SELECT min(x) AS r FROM t_zero_pm",
        "SELECT min(x) AS r FROM t_zero_mp",
        "SELECT max(x) AS r FROM t_zero_pm",
        "SELECT max(x) AS r FROM t_zero_mp",
        "SELECT min(x) AS r FROM t_zero_pm_one",
        "SELECT max(x) AS r FROM t_payload",
        "SELECT count(DISTINCT x) AS r FROM t_zero_pm",
    ];
    println!("| query | result | f64 bits |");
    println!("|---|---|---|");
    for q in queries {
        let batches = ctx.sql(q).await.unwrap().collect().await.unwrap();
        let col = batches[0].column(0);
        let (shown, bits) = if let Some(f) = col.as_any().downcast_ref::<Float64Array>() {
            let v = f.value(0);
            (format!("{v:?}"), format!("{:#018x}", v.to_bits()))
        } else {
            (
                arrow::util::display::array_value_to_string(col, 0).unwrap(),
                String::new(),
            )
        };
        println!("| `{q}` | {shown} | {bits} |");
    }
    // Arrow aggregate kernels directly
    let arr = Float64Array::from(vec![1.0, f64::NAN, -f64::NAN, -0.0, 0.0]);
    let mn = arrow::compute::min(&arr).unwrap();
    let mx = arrow::compute::max(&arr).unwrap();
    println!(
        "\narrow::compute over [1, +NaN, -NaN, -0, +0]: min bits {:#018x}, max bits {:#018x}",
        mn.to_bits(),
        mx.to_bits()
    );
    let z = Float64Array::from(vec![0.0, -0.0]);
    println!(
        "arrow::compute over [+0, -0]: min bits {:#018x}, max bits {:#018x}",
        arrow::compute::min(&z).unwrap().to_bits(),
        arrow::compute::max(&z).unwrap().to_bits()
    );
}

// ---------------------------------------------------------------- probe 4
async fn probe4() {
    println!("\n## Probe 4: canonical predicate lowering and pruning grid\n");
    let s = schema_x();
    let cs = containers(&s);
    let stats = PrunableStatistics::new(cs.iter().map(float_stats).collect(), s.clone());
    let total_rows: usize = cs.iter().map(|b| b.num_rows()).sum();
    println!("containers: {}, rows: {}", cs.len(), total_rows);

    // full-optimizer path for the same expressions
    let ctx = SessionContext::new();
    ctx.register_table(
        "t",
        Arc::new(MemTable::try_new(s.clone(), vec![cs.clone()]).unwrap()),
    )
    .unwrap();
    let df_all = ctx.table("t").await.unwrap();

    println!(
        "\n| min | max | missing | oracle rows | strict mask eq | filter-equiv eq | DF null where oracle false | SQL filter rows / bits eq | pruned | containers w/ matches | soundness violations |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|");
    let mut refused = 0;
    let mut total_viol = 0;
    let mut all_eq = true;
    for (mn, mx, ms) in predicate_grid() {
        let o: Result<Vec<BooleanArray>, String> = cs
            .iter()
            .map(|b| {
                oracle(
                    b.column(0).as_any().downcast_ref::<Float64Array>().unwrap(),
                    mn,
                    mx,
                    ms,
                )
            })
            .collect();
        let Ok(o) = o else {
            refused += 1;
            continue;
        };
        let oracle_rows: usize = o.iter().map(true_count).sum();
        let Some(expr) = lower(mn, mx, ms) else {
            // no predicate: oracle must select everything
            let all = oracle_rows == total_rows;
            all_eq &= all;
            println!(
                "| {} | {} | {} | {} | n/a (no predicate; oracle all-true={}) | | | | 0 | | 0 |",
                fmt_opt(mn),
                fmt_opt(mx),
                ms.map_or("-".into(), |b| b.to_string()),
                oracle_rows,
                all
            );
            continue;
        };
        let p = physical(&expr, &s);
        let mut strict = true;
        let mut fe = true;
        let mut df_null = 0usize;
        for (b, om) in cs.iter().zip(&o) {
            let dm = eval_mask(&p, b);
            if dm != *om {
                strict = false;
            }
            for i in 0..b.num_rows() {
                let d = dm.is_valid(i) && dm.value(i);
                let ov = om.is_valid(i) && om.value(i);
                if d != ov {
                    fe = false;
                }
                if dm.is_null(i) && !ov {
                    df_null += 1;
                }
            }
        }
        all_eq &= fe;
        // optimizer path: filtered row bits vs oracle-selected bits
        let got = df_all.clone().filter(expr.clone()).unwrap().collect().await.unwrap();
        let mut got_bits: Vec<Option<u64>> = got
            .iter()
            .flat_map(|b| sorted_bits(b.column(0).as_any().downcast_ref::<Float64Array>().unwrap()))
            .collect();
        got_bits.sort();
        let mut want_bits: Vec<Option<u64>> = cs
            .iter()
            .zip(&o)
            .flat_map(|(b, m)| {
                let f = filter(b.column(0), m).unwrap();
                sorted_bits(f.as_any().downcast_ref::<Float64Array>().unwrap())
            })
            .collect();
        want_bits.sort();
        let sql_eq = got_bits == want_bits;
        all_eq &= sql_eq;
        // pruning
        let pp = pruner(p.clone(), &s);
        let keep = pp.prune(&stats).unwrap();
        let pruned = keep.iter().filter(|k| !**k).count();
        let with_matches = o.iter().filter(|m| true_count(m) > 0).count();
        let viol = keep
            .iter()
            .zip(&o)
            .filter(|(k, m)| !**k && true_count(m) > 0)
            .count();
        total_viol += viol;
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {} / {} | {} | {} | {} |",
            fmt_opt(mn),
            fmt_opt(mx),
            ms.map_or("-".into(), |b| b.to_string()),
            oracle_rows,
            strict,
            fe,
            df_null,
            got_bits.len(),
            sql_eq,
            pruned,
            with_matches,
            viol
        );
    }
    println!(
        "\nrefused combinations (oracle Err): {refused}; all filter-equivalent & bit-equal: {all_eq}; total soundness violations: {total_viol}"
    );

    // raw SQL-style predicates: prune vs row-wise evaluation, plus totalOrder reference
    println!("\n### Raw predicates (pruning soundness under totalOrder)\n");
    println!("| predicate | rows true (DF eval) | matches Rust total_cmp ref | pruned | violations |");
    println!("|---|---|---|---|---|");
    use std::cmp::Ordering::*;
    type Ref = fn(f64) -> bool;
    let raws: Vec<(&str, Expr, Ref)> = vec![
        ("x > 5", col("x").gt(lit(5.0)), |v| v.total_cmp(&5.0) == Greater),
        ("x = 0.0", col("x").eq(lit(0.0)), |v| v.total_cmp(&0.0) == Equal),
        ("x = -0.0", col("x").eq(lit(-0.0)), |v| v.total_cmp(&-0.0) == Equal),
        ("x < 0", col("x").lt(lit(0.0)), |v| v.total_cmp(&0.0) == Less),
        ("x >= 0", col("x").gt_eq(lit(0.0)), |v| v.total_cmp(&0.0) != Less),
        ("x > 1e308", col("x").gt(lit(1e308)), |v| v.total_cmp(&1e308) == Greater),
        ("x < -1e308", col("x").lt(lit(-1e308)), |v| v.total_cmp(&-1e308) == Less),
        ("x != 5", col("x").not_eq(lit(5.0)), |v| v.total_cmp(&5.0) != Equal),
        ("x = NaN", col("x").eq(lit(f64::NAN)), |v| v.total_cmp(&f64::NAN) == Equal),
        ("x > inf", col("x").gt(lit(f64::INFINITY)), |v| v.total_cmp(&f64::INFINITY) == Greater),
        ("x < -inf", col("x").lt(lit(f64::NEG_INFINITY)), |v| v.total_cmp(&f64::NEG_INFINITY) == Less),
        ("x IS NOT NULL", col("x").is_not_null(), |_| true),
        ("NOT (x <= 5)", Expr::Not(Box::new(col("x").lt_eq(lit(5.0)))), |v| v.total_cmp(&5.0) == Greater),
        ("x BETWEEN -1 AND 1", col("x").between(lit(-1.0), lit(1.0)), |v| {
            v.total_cmp(&-1.0) != Less && v.total_cmp(&1.0) != Greater
        }),
    ];
    for (name, expr, rf) in raws {
        let p = physical(&expr, &s);
        let pp = pruner(p.clone(), &s);
        let keep = pp.prune(&stats).unwrap();
        let mut rows = 0;
        let mut ref_ok = true;
        let mut viol = 0;
        for (b, k) in cs.iter().zip(&keep) {
            let m = eval_mask(&p, b);
            let x = b.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
            let t = true_count(&m);
            rows += t;
            for i in 0..x.len() {
                let d = m.is_valid(i) && m.value(i);
                let r = x.is_valid(i) && rf(x.value(i));
                if d != r {
                    ref_ok = false;
                }
            }
            if !*k && t > 0 {
                viol += 1;
            }
        }
        let pruned = keep.iter().filter(|k| !**k).count();
        println!("| `{name}` | {rows} | {ref_ok} | {pruned} | {viol} |");
    }

    // batch-size sweep
    println!("\n### Batch-size sweep (16384 rows; pruned fraction of containers)\n");
    let mut r = Rng(0x1234_5678_9abc_def1);
    let g: Vec<Option<f64>> = grid().into_iter().map(|(_, v)| v).collect();
    let n = 16384;
    let iid: Vec<Option<f64>> = (0..n)
        .map(|_| {
            if r.below(50) == 0 {
                g[r.below(g.len())]
            } else {
                Some((r.next() % 2_000_001) as f64 / 1000.0 - 1000.0)
            }
        })
        .collect();
    let mut sorted = iid.clone();
    sorted.sort_by(|a, b| match (a, b) {
        (None, None) => Equal,
        (None, _) => Greater,
        (_, None) => Less,
        (Some(a), Some(b)) => a.total_cmp(b),
    });
    let preds: Vec<(&str, Option<f64>, Option<f64>, Option<bool>)> = vec![
        ("min=5", Some(5.0), None, None),
        ("max=0", None, Some(0.0), None),
        ("[-1,1]", Some(-1.0), Some(1.0), None),
        ("[0,0]", Some(0.0), Some(0.0), None),
        ("missing=true", None, None, Some(true)),
        ("missing=false", None, None, Some(false)),
    ];
    print!("| layout | batch rows | containers |");
    for p in &preds {
        print!(" {} |", p.0);
    }
    println!(" violations |");
    print!("|---|---|---|");
    for _ in &preds {
        print!("---|");
    }
    println!("---|");
    for (layout, data) in [("iid (value-unordered)", &iid), ("value-sorted (best case)", &sorted)] {
        for bs in [1usize, 16, 64, 256, 1024, 4096] {
            let cs: Vec<RecordBatch> = data.chunks(bs).map(|c| batch_of(&s, c)).collect();
            let st = PrunableStatistics::new(cs.iter().map(float_stats).collect(), s.clone());
            print!("| {layout} | {bs} | {} |", cs.len());
            let mut viol = 0;
            for (_, mn, mx, ms) in &preds {
                let e = lower(*mn, *mx, *ms).unwrap();
                let p = physical(&e, &s);
                let keep = pruner(p, &s).prune(&st).unwrap();
                let pruned = keep.iter().filter(|k| !**k).count();
                for (b, k) in cs.iter().zip(&keep) {
                    let m = oracle(
                        b.column(0).as_any().downcast_ref::<Float64Array>().unwrap(),
                        *mn,
                        *mx,
                        *ms,
                    )
                    .unwrap();
                    if !*k && true_count(&m) > 0 {
                        viol += 1;
                    }
                }
                print!(" {:.1}% |", 100.0 * pruned as f64 / cs.len() as f64);
            }
            println!(" {viol} |");
        }
    }
}

// ---------------------------------------------------------------- probe 8
fn probe8() {
    println!("\n## Probe 8: FixedSizeBinary(16) symbol_id pruning\n");
    let s: SchemaRef = Arc::new(Schema::new(vec![
        Field::new("step", DataType::Int64, false),
        Field::new("symbol_id", DataType::FixedSizeBinary(16), false),
    ]));
    let mut r = Rng(0xdead_beef_cafe_f00d);
    let vars = 512usize;
    let steps = 32i64;
    let ids: Vec<[u8; 16]> = (0..vars)
        .map(|_| {
            let a = r.next().to_be_bytes();
            let b = r.next().to_be_bytes();
            let mut x = [0u8; 16];
            x[..8].copy_from_slice(&a);
            x[8..].copy_from_slice(&b);
            x
        })
        .collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    // rows sorted by (step, symbol_id) -> the C1 within-file key order
    let rows: Vec<(i64, [u8; 16])> = (0..steps)
        .flat_map(|st| sorted_ids.iter().map(move |id| (st, *id)))
        .collect();
    let stats_for = |b: &RecordBatch| -> Arc<Statistics> {
        let st = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        let id = b.column(1).as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
        let mut c0 = ColumnStatistics::new_unknown();
        c0.null_count = Precision::Exact(0);
        c0.min_value = Precision::Exact(ScalarValue::Int64(arrow::compute::min(st)));
        c0.max_value = Precision::Exact(ScalarValue::Int64(arrow::compute::max(st)));
        let mut c1 = ColumnStatistics::new_unknown();
        c1.null_count = Precision::Exact(0);
        c1.min_value = Precision::Exact(ScalarValue::FixedSizeBinary(
            16,
            arrow::compute::min_fixed_size_binary(id).map(|v| v.to_vec()),
        ));
        c1.max_value = Precision::Exact(ScalarValue::FixedSizeBinary(
            16,
            arrow::compute::max_fixed_size_binary(id).map(|v| v.to_vec()),
        ));
        Arc::new(Statistics {
            num_rows: Precision::Exact(b.num_rows()),
            total_byte_size: Precision::Absent,
            column_statistics: vec![c0, c1],
        })
    };
    let mk = |chunk: &[(i64, [u8; 16])]| {
        RecordBatch::try_new(
            s.clone(),
            vec![
                Arc::new(Int64Array::from(chunk.iter().map(|r| r.0).collect::<Vec<_>>())),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(chunk.iter().map(|r| r.1)).unwrap(),
                ),
            ],
        )
        .unwrap()
    };
    println!("{} variables x {} steps = {} rows, sorted by (step, symbol_id)\n", vars, steps, rows.len());
    println!("| batch rows | containers | query | avg kept | max kept | violations |");
    println!("|---|---|---|---|---|---|");
    for bs in [64usize, 256, 1024, 4096] {
        let cs: Vec<RecordBatch> = rows.chunks(bs).map(mk).collect();
        let st = PrunableStatistics::new(cs.iter().map(&stats_for).collect(), s.clone());
        for qkind in ["symbol_id = lit", "step = k AND symbol_id = lit", "symbol_id = absent lit"] {
            let mut kept_sum = 0usize;
            let mut kept_max = 0usize;
            let mut viol = 0;
            let trials = 64;
            for t in 0..trials {
                let id = if qkind.contains("absent") {
                    let mut x = ids[t % vars];
                    x[15] ^= 0x5a;
                    x
                } else {
                    ids[r.below(vars)]
                };
                let k = (t as i64) % steps;
                let lit_id = lit(ScalarValue::FixedSizeBinary(16, Some(id.to_vec())));
                let e = if qkind.starts_with("step") {
                    col("step").eq(lit(k)).and(col("symbol_id").eq(lit_id))
                } else {
                    col("symbol_id").eq(lit_id)
                };
                let p = physical(&e, &s);
                let keep = pruner(p.clone(), &s)
                    .prune(&st)
                    .unwrap();
                let kept = keep.iter().filter(|k| **k).count();
                kept_sum += kept;
                kept_max = kept_max.max(kept);
                for (b, kp) in cs.iter().zip(&keep) {
                    if !*kp && true_count(&eval_mask(&p, b)) > 0 {
                        viol += 1;
                    }
                }
            }
            println!(
                "| {bs} | {} | `{qkind}` | {:.2} | {kept_max} | {viol} |",
                cs.len(),
                kept_sum as f64 / trials as f64
            );
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    probe2().await;
    probe4().await;
    probe8();
}
