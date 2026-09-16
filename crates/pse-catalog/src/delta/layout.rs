// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native, inspectable lossless projections between declared Arrow and Delta layouts.
use datafusion::{
    arrow::{
        compute::{CastOptions, cast_with_options},
        datatypes::{DataType, FieldRef, SchemaRef},
    },
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, LogicalPlan, LogicalPlanBuilder, ReturnFieldArgs, ScalarFunctionArgs,
        ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
};
use std::sync::Arc;

/// A typed relation boundary restores schema annotations after native rewrites.
/// Exact fields (including nested annotations and nullability) must already agree;
/// this cannot certify an arbitrary projection or retype its values.
pub(super) fn declared_output(
    input: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    schema: &SchemaRef,
) -> Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>> {
    if input.schema().fields() != schema.fields() {
        return Err(DataFusionError::Plan(
            "physical relation fields differ from the declaration".into(),
        ));
    }
    let expressions: Vec<_> = schema
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let expression: Arc<dyn datafusion::physical_expr::PhysicalExpr> = Arc::new(
                datafusion::physical_expr::expressions::Column::new(field.name(), index),
            );
            (expression, field.name().to_owned())
        })
        .collect();
    Ok(Arc::new(
        datafusion::physical_plan::projection::ProjectionExec::try_new_with_schema_metadata(
            expressions,
            input,
            schema,
        )?,
    ))
}

/// Explicit Arrow execution and Delta storage views of one declared relation.
#[derive(Debug, Clone)]
pub struct DurableLayout {
    execution: SchemaRef,
    storage: SchemaRef,
}
impl DurableLayout {
    /// Derive the durable shape from one execution declaration, never from input values.
    /// # Errors
    /// A logical type lacks an implemented lossless mapping.
    pub fn new(execution: SchemaRef) -> Result<Self> {
        pse_schema::field_contract::declaration(&execution)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let storage = Arc::new(
            pse_schema::delta::storage_schema(&execution)
                .map_err(|e| DataFusionError::External(Box::new(e)))?,
        );
        Ok(Self { execution, storage })
    }
    /// Declared durable fields for table creation and native provider binding.
    pub fn storage_schema(&self) -> &SchemaRef {
        &self.storage
    }
    /// Declared semantic execution fields.
    pub fn execution_schema(&self) -> &SchemaRef {
        &self.execution
    }
    /// Project a native input into its lossless durable representation.
    /// # Errors
    /// Input fields differ from the declared execution contract.
    pub fn encode(&self, input: LogicalPlan) -> Result<LogicalPlan> {
        project(input, &self.execution, &self.storage, "pse_delta_encode")
    }
    /// Project a stored relation back to its declared semantic representation.
    /// Runtime conversion rejects overflows, invalid widths and required null values.
    /// # Errors
    /// Input fields differ from the durable contract.
    pub fn decode(&self, input: LogicalPlan) -> Result<LogicalPlan> {
        let stored_declaration = pse_schema::delta::execution_schema(input.schema().as_arrow())
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        pse_schema::field_contract::execution_schema(&stored_declaration, &self.execution)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        project(input, &self.storage, &self.execution, "pse_delta_decode")
    }
    /// Restore a generated durable field after native struct/aggregate expressions
    /// infer conservative nested nullability. Arrow casts check the actual values.
    pub(super) fn storage_expression(&self, index: usize, value: Expr, actual: DataType) -> Expr {
        let target = Arc::clone(&self.storage.fields()[index]);
        ScalarUDF::from(DurableCast {
            name: "pse_delta_encode_field",
            signature: Signature::exact(vec![actual], Volatility::Immutable),
            intermediate: target.data_type().clone(),
            target,
        })
        .call(vec![value])
    }
}
fn project(
    input: LogicalPlan,
    from: &SchemaRef,
    to: &SchemaRef,
    name: &'static str,
) -> Result<LogicalPlan> {
    let admission = if name == "pse_delta_decode" {
        pse_schema::field_contract::delta_scan_schema(input.schema().as_arrow(), from)
    } else {
        pse_schema::field_contract::execution_schema(input.schema().as_arrow(), from)
    };
    admission.map_err(|e| DataFusionError::External(Box::new(e)))?;
    let mut expressions = Vec::new();
    for (index, target) in to.fields().iter().enumerate() {
        let actual = input.schema().field(index);
        let function = ScalarUDF::from(DurableCast {
            name,
            signature: Signature::exact(vec![actual.data_type().clone()], Volatility::Immutable),
            intermediate: if name == "pse_delta_decode" {
                from.field(index).data_type().clone()
            } else {
                target.data_type().clone()
            },
            target: Arc::clone(target),
        });
        let (qualifier, field) = input.schema().qualified_field(index);
        let column = datafusion::common::Column::new(qualifier.cloned(), field.name());
        expressions.push(
            function
                .call(vec![Expr::Column(column)])
                .alias(target.name()),
        );
    }
    let plan = LogicalPlanBuilder::from(input)
        .project(expressions)?
        .build()?;
    let LogicalPlan::Projection(mut projection) = plan else {
        return Err(DataFusionError::Internal(
            "durable projection construction changed node kind".into(),
        ));
    };
    // Record the intended schema metadata. Logical projection rewrites may inherit
    // source metadata again; the Delta physical-input bridge enforces the storage
    // boundary after planning using the native physical projection metadata API.
    let fields = projection
        .schema
        .iter()
        .map(|(qualifier, field)| (qualifier.cloned(), Arc::clone(field)))
        .collect();
    projection.schema = Arc::new(
        datafusion::common::DFSchema::new_with_metadata(fields, to.metadata().clone())?
            .with_functional_dependencies(projection.schema.functional_dependencies().clone())?,
    );
    Ok(LogicalPlan::Projection(projection))
}
#[derive(Debug, PartialEq, Eq, Hash)]
struct DurableCast {
    name: &'static str,
    signature: Signature,
    intermediate: DataType,
    target: FieldRef,
}
impl ScalarUDFImpl for DurableCast {
    fn name(&self) -> &str {
        self.name
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(self.target.data_type().clone())
    }
    fn return_field_from_args(&self, _: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        Ok(Arc::clone(&self.target))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(DataFusionError::Execution(
                "durable cast needs one input".into(),
            ));
        };
        let array = value.to_array(args.number_rows)?;
        // Delta scans use view arrays. Arrow does not directly cast BinaryView to
        // FixedSizeBinary; normalize through the declared durable type first.
        let options = CastOptions {
            safe: false,
            ..CastOptions::default()
        };
        let array = cast_with_options(&array, &self.intermediate, &options)?;
        let converted = cast_with_options(
            &array,
            self.target.data_type(),
            &CastOptions {
                safe: false,
                ..CastOptions::default()
            },
        )?;
        if !self.target.is_nullable() && converted.null_count() != 0 {
            return Err(DataFusionError::Execution(format!(
                "required durable field {} contains nulls",
                self.target.name()
            )));
        }
        if self.name == "pse_delta_decode" && converted.data_type() != &self.intermediate {
            // CastOptions::safe=false rejects integer overflow, but float narrowing
            // may still round. Native Arrow logical array equality checks the inverse
            // conversion recursively, respecting null parent masks and dictionaries.
            let restored = cast_with_options(&converted, &self.intermediate, &options)?;
            if array.to_data() != restored.to_data() {
                return Err(DataFusionError::Execution(format!(
                    "durable field {} cannot be decoded without changing values",
                    self.target.name()
                )));
            }
        }
        Ok(ColumnarValue::Array(converted))
    }
}
