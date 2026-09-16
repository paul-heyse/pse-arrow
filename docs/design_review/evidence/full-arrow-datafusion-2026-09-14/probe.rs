// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library characterization only; no pse compiler path is exercised.
use arrow::array::{
    Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray, UInt32Array,
};
use arrow::compute::{concat_batches, filter_record_batch, take_record_batch};
use arrow::datatypes::{DataType, Field, FieldRef, Schema};
use arrow::record_batch::RecordBatch;
use arrow::row::{RowConverter, SortField};
use datafusion::execution::config::SessionConfig;
use datafusion::execution::context::SessionContext;
use datafusion_common::{DataFusionError, Result, ScalarValue};
use datafusion_expr::{
    ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
    Volatility,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
static VALUE_FAILURES: AtomicUsize = AtomicUsize::new(0);

fn require(condition: bool, detail: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(DataFusionError::Execution(detail.into()))
    }
}
async fn check(ctx: &SessionContext, label: &str, sql: &str, expected: &[&[&str]]) -> Result<()> {
    let batches = ctx.sql(sql).await?.collect().await?;
    let mut actual = Vec::new();
    for batch in &batches {
        for row in 0..batch.num_rows() {
            actual.push(
                batch
                    .columns()
                    .iter()
                    .map(|array| {
                        let value = ScalarValue::try_from_array(array, row)?;
                        Ok(if value.is_null() {
                            "NULL".to_owned()
                        } else {
                            value.to_string()
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        }
    }
    actual.sort();
    let mut expected = expected
        .iter()
        .map(|row| {
            row.iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        VALUE_FAILURES.fetch_add(1, Ordering::SeqCst);
        println!("FAIL {label}: actual={actual:?}; expected={expected:?}");
    } else {
        println!("PASS {label}: {actual:?}");
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct FieldIdentity {
    signature: Signature,
}
impl ScalarUDFImpl for FieldIdentity {
    fn name(&self) -> &str {
        "review_identity"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, types: &[DataType]) -> Result<DataType> {
        types
            .first()
            .cloned()
            .ok_or_else(|| DataFusionError::Plan("missing argument".into()))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        args.arg_fields
            .first()
            .cloned()
            .ok_or_else(|| DataFusionError::Plan("missing argument field".into()))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        args.args
            .into_iter()
            .next()
            .ok_or_else(|| DataFusionError::Execution("missing argument".into()))
    }
    fn is_strict(&self) -> bool {
        true
    }
}

async fn relational(ctx: &SessionContext) -> Result<()> {
    check(ctx,"union_all", "SELECT x FROM (VALUES (1),(1),(NULL)) a(x) UNION ALL SELECT x FROM (VALUES (1),(2),(NULL)) b(x)", &[&["1"],&["1"],&["1"],&["2"],&["NULL"],&["NULL"]]).await?;
    check(ctx,"union_distinct", "SELECT x FROM (VALUES (1),(1),(NULL)) a(x) UNION SELECT x FROM (VALUES (1),(2),(NULL)) b(x)", &[&["1"],&["2"],&["NULL"]]).await?;
    check(ctx,"intersect_distinct", "SELECT x FROM (VALUES (1),(1),(2),(NULL)) a(x) INTERSECT SELECT x FROM (VALUES (1),(NULL)) b(x)", &[&["1"],&["NULL"]]).await?;
    check(ctx,"except_distinct", "SELECT x FROM (VALUES (1),(1),(2),(NULL)) a(x) EXCEPT SELECT x FROM (VALUES (1),(NULL)) b(x)", &[&["2"]]).await?;
    check(ctx,"intersect_all", "SELECT x FROM (VALUES (1),(1),(2),(NULL)) a(x) INTERSECT ALL SELECT x FROM (VALUES (1),(NULL)) b(x)", &[&["1"],&["NULL"]]).await?;
    check(ctx,"except_all", "SELECT x FROM (VALUES (1),(1),(2),(NULL)) a(x) EXCEPT ALL SELECT x FROM (VALUES (1),(NULL)) b(x)", &[&["1"],&["2"]]).await?;
    check(ctx,"intersect_all_occurrence_lowering", "WITH a AS (SELECT x,row_number() OVER (PARTITION BY x) AS occurrence FROM (VALUES (1),(1),(2),(NULL)) a(x)), b AS (SELECT x,row_number() OVER (PARTITION BY x) AS occurrence FROM (VALUES (1),(NULL)) b(x)) SELECT a.x FROM a JOIN b ON (a.x IS NOT DISTINCT FROM b.x) AND a.occurrence=b.occurrence", &[&["1"],&["NULL"]]).await?;
    check(ctx,"except_all_occurrence_lowering", "WITH a AS (SELECT x,row_number() OVER (PARTITION BY x) AS occurrence FROM (VALUES (1),(1),(2),(NULL)) a(x)), b AS (SELECT x,row_number() OVER (PARTITION BY x) AS occurrence FROM (VALUES (1),(NULL)) b(x)) SELECT a.x FROM a LEFT ANTI JOIN b ON (a.x IS NOT DISTINCT FROM b.x) AND a.occurrence=b.occurrence", &[&["1"],&["2"]]).await?;
    check(
        ctx,
        "full_outer_join",
        "SELECT a.x,b.x FROM (VALUES (1),(2)) a(x) FULL JOIN (VALUES (2),(3)) b(x) ON a.x=b.x",
        &[&["1", "NULL"], &["2", "2"], &["NULL", "3"]],
    )
    .await?;
    check(
        ctx,
        "non_equi_join",
        "SELECT a.x,b.x FROM (VALUES (1),(3)) a(x) JOIN (VALUES (2),(4)) b(x) ON a.x < b.x",
        &[&["1", "2"], &["1", "4"], &["3", "4"]],
    )
    .await?;
    check(ctx,"correlated_exists", "SELECT a.x FROM (VALUES (1),(3)) a(x) WHERE EXISTS (SELECT 1 FROM (VALUES (2),(3)) b(x) WHERE b.x=a.x)", &[&["3"]]).await?;
    check(ctx,"window_preserves_ties", "SELECT id,dense_rank() OVER (PARTITION BY k ORDER BY priority) FROM (VALUES (7,1,2),(8,1,2),(9,1,3)) a(id,k,priority)", &[&["7","1"],&["8","1"],&["9","2"]]).await?;
    check(ctx,"grouping_sets", "SELECT coalesce(g,-1),count(*) FROM (VALUES (1,2),(1,3),(2,4)) a(g,v) GROUP BY GROUPING SETS ((g),())", &[&["1","2"],&["2","1"],&["-1","3"]]).await?;
    check(ctx,"aggregate_filter_and_order", "SELECT sum(v) FILTER (WHERE v>1),array_element(array_agg(v ORDER BY v DESC NULLS LAST),1) FROM (VALUES (1),(3),(2),(NULL)) a(v)", &[&["5","3"]]).await?;
    check(ctx,"nested_list_transform", "SELECT array_length(array_distinct(make_array(1,2,2))),array_element(array_sort(make_array(3,1,2)),1)", &[&["2","1"]]).await?;
    check(ctx,"finite_recursive_distinct_cycle", "WITH RECURSIVE r(n) AS (SELECT 1 UNION SELECT CASE WHEN n=1 THEN 2 ELSE 1 END FROM r) SELECT n FROM r", &[&["1"],&["2"]]).await?;
    check(
        ctx,
        "bounded_recursive_all",
        "WITH RECURSIVE r(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM r WHERE n<4) SELECT n FROM r",
        &[&["1"], &["2"], &["3"], &["4"]],
    )
    .await?;
    Ok(())
}

async fn metadata_and_udf(ctx: &SessionContext) -> Result<()> {
    let field = Field::new("key", DataType::Utf8, true).with_metadata(HashMap::from([(
        "review.semantic-kind".into(),
        "entity-label".into(),
    )]));
    let schema = Arc::new(Schema::new(vec![field]));
    let array: ArrayRef = Arc::new(StringArray::from(vec![Some("a"), None, Some("b")]));
    let batch = RecordBatch::try_new(schema, vec![array])?;
    ctx.register_batch("labels", batch)?;
    ctx.register_udf(ScalarUDF::from(FieldIdentity {
        signature: Signature::exact(vec![DataType::Utf8], Volatility::Immutable),
    }));
    let out = ctx
        .sql("SELECT review_identity(key) AS key FROM labels")
        .await?
        .collect()
        .await?;
    require(
        out.iter().all(|batch| {
            batch
                .schema()
                .field(0)
                .metadata()
                .get("review.semantic-kind")
                .is_some_and(|kind| kind == "entity-label")
        }),
        "UDF field metadata lost",
    )?;
    require(
        out.iter().map(RecordBatch::num_rows).sum::<usize>() == 3,
        "UDF rows differ",
    )?;
    let mut values = out
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .iter()
                .map(|value| value.map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    values.sort();
    require(
        values == [None, Some("a".into()), Some("b".into())],
        "UDF values/nulls differ",
    )?;
    println!("PASS field_aware_scalar_udf: metadata, nulls, and row count preserved");
    Ok(())
}

fn arrow_kernels() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false).with_metadata(HashMap::from([(
            "review.semantic-kind".into(),
            "member-ordinal".into(),
        )])),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(Int64Array::from(vec![3, 1, 2]))],
    )?;
    let filtered = filter_record_batch(&batch, &BooleanArray::from(vec![true, false, true]))?;
    let taken = take_record_batch(&filtered, &UInt32Array::from(vec![1, 0]))?;
    let merged = concat_batches(&schema, [&taken, &taken])?;
    require(
        merged.schema() == schema && merged.num_rows() == 4,
        "Arrow kernels changed contract or row count",
    )?;
    let actual = merged
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();
    require(
        actual.values().as_ref() == [2, 3, 2, 3],
        "Arrow kernel values differ",
    )?;
    let converter = RowConverter::new(vec![SortField::new(DataType::Float64)])?;
    let encoded = converter.convert_columns(&[Arc::new(Float64Array::from(vec![-0.0, 0.0]))])?;
    require(
        encoded.row(0) != encoded.row(1),
        "Arrow sort rows merged signed zero",
    )?;
    println!(
        "PASS Arrow filter/take/concat fields and values; RowConverter distinguishes signed zero"
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut outcomes = BTreeMap::new();
    for partitions in [1, 4] {
        let ctx = SessionContext::new_with_config(
            SessionConfig::new()
                .with_target_partitions(partitions)
                .with_batch_size(2),
        );
        println!(
            "CONDITIONS target_partitions={partitions}; batch_size=2; finite in-memory inputs"
        );
        for (name, result) in [
            ("relational", relational(&ctx).await),
            ("metadata_udf", metadata_and_udf(&ctx).await),
        ] {
            if let Err(error) = &result {
                println!("FAIL {name} partitions={partitions}: {error}");
            }
            outcomes.insert((name, partitions), result.is_ok());
        }
    }
    let arrow = arrow_kernels();
    if let Err(error) = &arrow {
        println!("FAIL Arrow kernels: {error}");
    }
    let failures = outcomes.values().filter(|passed| !**passed).count()
        + usize::from(arrow.is_err())
        + VALUE_FAILURES.load(Ordering::SeqCst);
    println!(
        "SUMMARY groups=5 failures={failures} baseline=0; library behavior only; no platform acceptance or performance claim"
    );
    require(failures == 0, "library characterization had failures")
}
