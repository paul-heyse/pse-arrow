// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Isolated construction characterization; does not exercise the PSE runtime.
use arrow::datatypes::{DataType, Field, FieldRef, Schema};
use datafusion::execution::context::SessionContext;
use datafusion_common::{DFSchema, DataFusionError, Result};
use datafusion_expr::logical_plan::{assert_expected_schema, table_scan};
use datafusion_expr::{
    ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
    Volatility, col,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static INVOCATIONS: AtomicUsize = AtomicUsize::new(0);
fn require(value: bool, name: &str) -> Result<()> {
    if !value {
        return Err(DataFusionError::Execution(format!("FAIL {name}")));
    }
    println!("PASS {name}");
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct DomainIdentity(Signature);
impl ScalarUDFImpl for DomainIdentity {
    fn name(&self) -> &str {
        "member_identity"
    }
    fn signature(&self) -> &Signature {
        &self.0
    }
    fn return_type(&self, types: &[DataType]) -> Result<DataType> {
        types
            .first()
            .cloned()
            .ok_or_else(|| DataFusionError::Plan("missing field".into()))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        let field = args
            .arg_fields
            .first()
            .ok_or_else(|| DataFusionError::Plan("missing field".into()))?;
        if field.metadata().get("review.domain").map(String::as_str) != Some("member") {
            return Err(DataFusionError::Plan(
                "expected member semantic domain".into(),
            ));
        }
        Ok(Arc::clone(field))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        INVOCATIONS.fetch_add(1, Ordering::SeqCst);
        args.args
            .into_iter()
            .next()
            .ok_or_else(|| DataFusionError::Execution("missing argument".into()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let field = |name: &str, domain: &str| {
        Field::new(name, DataType::Int64, false)
            .with_metadata(HashMap::from([("review.domain".into(), domain.into())]))
    };
    let schema = Schema::new(vec![field("member", "member"), field("unit", "unit")]);
    // Schema-only source: there are no data batches and no executable provider here.
    // It is not used as a production-provider optimizer surrogate.
    let source = table_scan(Some("source"), &schema, None)?;
    require(
        source.clone().project(vec![col("missing")]).is_err(),
        "unknown field rejected during construction without data",
    )?;
    let projected = source
        .clone()
        .project(vec![col("member").alias("selected")])?
        .build()?;
    require(
        projected
            .schema()
            .field(0)
            .metadata()
            .get("review.domain")
            .map(String::as_str)
            == Some("member")
            && !projected.schema().field(0).is_nullable(),
        "direct projection alias preserves domain metadata and nullability",
    )?;
    let udf = ScalarUDF::from(DomainIdentity(Signature::uniform(
        1,
        vec![DataType::Int64],
        Volatility::Immutable,
    )));
    let typed = source
        .clone()
        .project(vec![udf.call(vec![col("member")]).alias("selected")])?
        .build()?;
    require(
        typed
            .schema()
            .field(0)
            .metadata()
            .get("review.domain")
            .map(String::as_str)
            == Some("member"),
        "field-aware function derives semantic output during construction",
    )?;
    require(
        source.project(vec![udf.call(vec![col("unit")])]).is_err(),
        "same storage type wrong semantic domain rejected before execution",
    )?;
    require(
        INVOCATIONS.load(Ordering::SeqCst) == 0,
        "all construction checks invoked function body zero times",
    )?;
    let weak_expected = Arc::new(DFSchema::try_from(Schema::new(vec![Field::new(
        "selected",
        DataType::Int64,
        true,
    )]))?);
    require(
        assert_expected_schema(&weak_expected, &typed).is_ok()
            && weak_expected.fields() != typed.schema().fields(),
        "native expected-schema comparison accepts metadata and nullability mismatch",
    )?;
    let ctx = SessionContext::new();
    let null_failures = ctx
        .sql("SELECT x FROM (VALUES (1),(-1),(NULL)) t(x) WHERE (x > 0) IS NOT TRUE")
        .await?
        .collect()
        .await?;
    require(
        null_failures.iter().map(|b| b.num_rows()).sum::<usize>() == 2,
        "planned violation predicate includes false and unknown",
    )?;
    let violations = ctx.sql("WITH c AS (SELECT * FROM (VALUES (1,10),(2,99)) t(id,fk)), p AS (SELECT * FROM (VALUES (10)) t(id)) SELECT c.id FROM c LEFT ANTI JOIN p ON c.fk=p.id")
        .await?.collect().await?;
    require(
        violations.iter().map(|b| b.num_rows()).sum::<usize>() == 1,
        "native anti join expresses reference obligation",
    )?;
    println!(
        "SUMMARY checks=8 passed=8 failed=0 baseline=0; first six are construction/contract checks; no PSE acceptance or performance claim"
    );
    Ok(())
}
