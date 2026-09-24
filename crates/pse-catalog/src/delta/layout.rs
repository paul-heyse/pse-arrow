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
        ColumnarValue, Expr, LogicalPlan, Projection, ReturnFieldArgs, ScalarFunctionArgs,
        ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
};
use std::sync::Arc;

mod visibility;

#[cfg(test)]
mod tests;

/// A typed relation boundary restores schema annotations after native rewrites.
/// Exact names, types and metadata must already agree. A native optimizer may
/// establish a stronger nonnull result than the nullable relation declaration;
/// the native identity cast below widens only that outer nullable promise.
/// This cannot certify an arbitrary projection or retype its values.
pub(super) fn declared_output(
    input: Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    schema: &SchemaRef,
) -> Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>> {
    let actual = input.schema();
    if actual == *schema {
        return Ok(input);
    }
    if actual.fields().len() != schema.fields().len()
        || actual
            .fields()
            .iter()
            .zip(schema.fields())
            .any(|(source, target)| {
                (source.is_nullable() && !target.is_nullable())
                    || source.as_ref().clone().with_nullable(target.is_nullable()) != **target
            })
    {
        return Err(DataFusionError::Plan(
            "physical relation fields differ from the declaration".into(),
        ));
    }
    let expressions: Vec<_> = schema
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let mut expression: Arc<dyn datafusion::physical_expr::PhysicalExpr> = Arc::new(
                datafusion::physical_expr::expressions::Column::new(field.name(), index),
            );
            if field.is_nullable() && !actual.field(index).is_nullable() {
                // Types are exactly equal, so this is Arrow's identity path:
                // it cannot replace failed conversions with null values.
                expression = Arc::new(
                    datafusion::physical_expr::expressions::TryCastExpr::new_with_target_field(
                        expression,
                        Arc::clone(field),
                    ),
                );
            }
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
    decode_roundtrip: Arc<[bool]>,
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
        let decode_roundtrip = storage
            .fields()
            .iter()
            .zip(execution.fields())
            .map(|(from, to)| needs_roundtrip(from.data_type(), to.data_type()))
            .collect();
        Ok(Self {
            execution,
            storage,
            decode_roundtrip,
        })
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
        project(
            input,
            &self.execution,
            &self.storage,
            "pse_delta_encode",
            &[],
        )
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
        project(
            input,
            &self.storage,
            &self.execution,
            "pse_delta_decode",
            &self.decode_roundtrip,
        )
    }
    /// Decode stored relation fields while retaining native CDF metadata columns.
    /// The latter are supplied by Delta, not persisted execution descriptors.
    pub(super) fn decode_with_native_tail(&self, input: LogicalPlan) -> Result<LogicalPlan> {
        use datafusion::arrow::datatypes::Schema;
        let width = self.storage.fields().len();
        let source = input.schema().as_arrow();
        if source.fields().len() < width {
            return Err(DataFusionError::Plan(
                "CDF lacks the declared relation fields".into(),
            ));
        }
        let prefix =
            Schema::new_with_metadata(source.fields()[..width].to_vec(), source.metadata().clone());
        let declared = pse_schema::delta::execution_schema(&prefix)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        pse_schema::field_contract::execution_schema(&declared, &self.execution)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let tail = &source.fields()[width..];
        let combined = |schema: &SchemaRef| {
            Arc::new(Schema::new_with_metadata(
                schema
                    .fields()
                    .iter()
                    .chain(tail)
                    .cloned()
                    .collect::<Vec<_>>(),
                schema.metadata().clone(),
            ))
        };
        let stored = combined(&self.storage);
        let execution = combined(&self.execution);
        project(
            input,
            &stored,
            &execution,
            "pse_delta_decode",
            &self.decode_roundtrip,
        )
    }
    /// Restore a generated durable field after native struct/aggregate expressions
    /// infer conservative nested nullability. Arrow casts check the actual values.
    pub(super) fn storage_expression(&self, index: usize, value: Expr, actual: DataType) -> Expr {
        let target = Arc::clone(&self.storage.fields()[index]);
        ScalarUDF::from(DurableCast {
            name: "pse_delta_encode_field",
            roundtrip: false,
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
    roundtrip: &[bool],
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
            roundtrip: roundtrip.get(index).copied().unwrap_or(false),
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
    // These columns are already resolved from the admitted input fields. Native
    // Projection validates their types without the SQL builder's whole-graph
    // USING-name discovery, which would expand shared producer inputs repeatedly.
    let mut projection = Projection::try_new(expressions, Arc::new(input))?;
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
    roundtrip: bool,
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
        Ok(ColumnarValue::Array(restore_array_classified(
            &array,
            &self.intermediate,
            &self.target,
            self.name == "pse_delta_decode",
            self.roundtrip,
        )?))
    }
}

/// One mechanical conversion for scans, writes and CHECK input restoration.
pub(super) fn restore_array(
    array: &datafusion::arrow::array::ArrayRef,
    intermediate: &DataType,
    target: &FieldRef,
    decode: bool,
) -> Result<datafusion::arrow::array::ArrayRef> {
    restore_array_classified(
        array,
        intermediate,
        target,
        decode,
        needs_roundtrip(intermediate, target.data_type()),
    )
}

fn needs_roundtrip(from: &DataType, to: &DataType) -> bool {
    if from == to {
        return false;
    }
    // Representation widening preserves every value. Other mappings retain the
    // inverse check: Arrow cast support alone is not a losslessness proof.
    !matches!(
        (from, to),
        (DataType::Utf8, DataType::LargeUtf8 | DataType::Utf8View)
            | (
                DataType::Binary,
                DataType::LargeBinary | DataType::BinaryView
            )
            | (DataType::Int64, DataType::Timestamp(_, _))
    )
}

fn restore_array_classified(
    array: &datafusion::arrow::array::ArrayRef,
    intermediate: &DataType,
    target: &FieldRef,
    decode: bool,
    roundtrip: bool,
) -> Result<datafusion::arrow::array::ArrayRef> {
    // Delta scans use view arrays. Arrow does not directly cast BinaryView to
    // FixedSizeBinary; normalize through the declared durable type first.
    let options = CastOptions {
        safe: false,
        ..CastOptions::default()
    };
    let array = cast_with_options(array, intermediate, &options)?;
    let array = if decode {
        visibility::storage(array)?
    } else {
        array
    };
    let converted = cast_with_options(
        &array,
        target.data_type(),
        &CastOptions {
            safe: false,
            ..CastOptions::default()
        },
    )?;
    if !target.is_nullable() && converted.null_count() != 0 {
        return Err(DataFusionError::Execution(format!(
            "required durable field {} contains nulls",
            target.name()
        )));
    }
    if decode && roundtrip {
        // CastOptions::safe=false rejects integer overflow, but float narrowing
        // may still round. Native Arrow logical array equality checks the inverse
        // conversion recursively, respecting null parent masks and dictionaries.
        // The inverse of a null fixed-size container introduces masked child
        // slots into variable-size storage. Comparison-only nullable children
        // allow those placeholders; the returned declaration stays unchanged.
        let comparison = visibility::comparison_type(intermediate);
        let original = cast_with_options(&array, &comparison, &options)?;
        let restored = cast_with_options(&converted, &comparison, &options)?;
        if original.to_data() != restored.to_data() {
            return Err(DataFusionError::Execution(format!(
                "durable field {} cannot be decoded without changing values",
                target.name()
            )));
        }
    }
    Ok(converted)
}
