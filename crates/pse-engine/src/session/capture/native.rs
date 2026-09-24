// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native source scan with field assertions and window-based key checks.
use datafusion::{
    arrow::{
        array::{Array, BooleanArray, RecordBatch},
        datatypes::{DataType, SchemaRef},
    },
    common::{Column, Constraint, Constraints, DataFusionError, Result},
    functions_aggregate::count::count_udaf,
    logical_expr::{
        ColumnarValue, Expr, ExprFunctionExt, LogicalPlan, LogicalPlanBuilder, ScalarFunctionArgs,
        ScalarUDF, ScalarUDFImpl, Signature, Volatility, expr::WindowFunction, lit,
    },
};
use pse_columnar::CancellationToken;
use pse_schema::Registry;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(super) fn plan(
    input: LogicalPlan,
    registry: &Registry,
    constraints: &Constraints,
    cancel: CancellationToken,
    state: datafusion::execution::session_state::SessionState,
) -> Result<LogicalPlan> {
    let schema = Arc::new(input.schema().as_arrow().clone());
    let fields = schema
        .fields()
        .iter()
        .map(|f| column(f.name()))
        .collect::<Vec<_>>();
    let context = pse_relations::validate::ValidationContext::new(
        registry,
        crate::validation::NativeValidation(state),
    );
    let prepared = context
        .prepare(registry, Arc::clone(&schema))
        .map_err(pse_columnar::external)?;
    let check = ScalarUDF::from(Values(Arc::new(Fields {
        prepared,
        schema: Arc::clone(&schema),
        cancel,
        signature: Signature::exact(
            schema
                .fields()
                .iter()
                .map(|f| f.data_type().clone())
                .collect(),
            Volatility::Immutable,
        ),
    })));
    let mut plan = LogicalPlanBuilder::from(input)
        .filter(check.call(fields.clone()))?
        .build()?;
    for (ordinal, constraint) in constraints.iter().enumerate() {
        let indices = match constraint {
            Constraint::PrimaryKey(indices) | Constraint::Unique(indices) => indices,
        };
        let keys = indices
            .iter()
            .map(|index| column(schema.field(*index).name()))
            .collect::<Vec<_>>();
        let null = keys
            .iter()
            .cloned()
            .map(Expr::is_null)
            .reduce(Expr::or)
            .ok_or_else(|| external(super::invalid("empty key declaration")))?;
        let mut name = format!("pse_capture_count_{ordinal}");
        while schema.fields().iter().any(|field| field.name() == &name) {
            name.push('_');
        }
        let count =
            Expr::WindowFunction(Box::new(WindowFunction::new(count_udaf(), vec![lit(1i64)])))
                .partition_by(keys)
                .build()?
                .alias(&name);
        let unique = column(&name).eq(lit(1i64));
        let condition = match constraint {
            Constraint::PrimaryKey(_) => (!null).and(unique),
            Constraint::Unique(_) => null.or(unique),
        };
        let assertion = ScalarUDF::from(KeyCheck {
            signature: Signature::exact(vec![DataType::Boolean], Volatility::Immutable),
        });
        plan = LogicalPlanBuilder::from(plan)
            .window(vec![count])?
            .filter(assertion.call(vec![condition]))?
            .project(fields.clone())?
            .build()?;
    }
    Ok(plan)
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn external(
    error: impl pse_diagnostics::TypedDiagnostic + Send + Sync + 'static,
) -> DataFusionError {
    pse_columnar::external(error)
}

#[derive(Debug)]
struct Fields {
    prepared: Arc<pse_relations::validate::PreparedLocalContract>,
    schema: SchemaRef,
    cancel: CancellationToken,
    signature: Signature,
}
#[derive(Debug, Clone)]
struct Values(Arc<Fields>);
impl PartialEq for Values {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Values {}
impl Hash for Values {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl ScalarUDFImpl for Values {
    fn name(&self) -> &'static str {
        "pse_capture_values"
    }
    fn signature(&self) -> &Signature {
        &self.0.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let columns = args
            .args
            .iter()
            .map(|value| value.to_array(args.number_rows))
            .collect::<Result<Vec<_>>>()?;
        let batch = RecordBatch::try_new(Arc::clone(&self.0.schema), columns)?;
        self.0
            .prepared
            .evaluate(&batch, 256, &self.0.cancel)
            .and_then(pse_relations::validate::ValidationReport::require_valid)
            .map_err(pse_columnar::external)?;
        Ok(ColumnarValue::Array(Arc::new(BooleanArray::from(
            vec![true; args.number_rows],
        ))))
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
struct KeyCheck {
    signature: Signature,
}
impl ScalarUDFImpl for KeyCheck {
    fn name(&self) -> &'static str {
        "pse_capture_key"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Boolean)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(external(super::invalid("key check requires one predicate")));
        };
        let value = value.to_array(args.number_rows)?;
        let values = value
            .as_any()
            .downcast_ref::<BooleanArray>()
            .ok_or_else(|| external(super::invalid("key predicate is not Boolean")))?;
        if values.true_count() != values.len() {
            return Err(external(super::invalid(
                "actual captured rows violate an advertised key",
            )));
        }
        Ok(ColumnarValue::Array(value))
    }
}
