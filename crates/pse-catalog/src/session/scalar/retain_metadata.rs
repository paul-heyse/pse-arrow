// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Project existing field annotations without changing values or adding meaning.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{Column, DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, LogicalPlan, LogicalPlanBuilder, ReturnFieldArgs, ScalarFunctionArgs,
        ScalarUDF, ScalarUDFImpl, Signature, Union, Volatility, lit,
    },
};
use std::sync::{Arc, LazyLock};

static FUNCTION: LazyLock<Arc<ScalarUDF>> = LazyLock::new(|| {
    Arc::new(ScalarUDF::from(RetainMetadata {
        signature: Signature::variadic_any(Volatility::Immutable),
    }))
});
pub(super) fn function() -> Arc<ScalarUDF> {
    Arc::clone(&FUNCTION)
}
pub(crate) fn retain(value: Expr, keys: impl IntoIterator<Item = String>) -> Expr {
    FUNCTION.call(
        std::iter::once(value)
            .chain(keys.into_iter().map(lit))
            .collect(),
    )
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct RetainMetadata {
    signature: Signature,
}
impl ScalarUDFImpl for RetainMetadata {
    fn name(&self) -> &'static str {
        "pse_retain_metadata"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        args.first()
            .cloned()
            .ok_or_else(|| invalid("requires an input value"))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let input = args
            .arg_fields
            .first()
            .ok_or_else(|| invalid("requires an input field"))?;
        let mut metadata = std::collections::HashMap::new();
        for key in args.scalar_arguments.iter().skip(1) {
            let key = key
                .and_then(|value| value.try_as_str().flatten())
                .ok_or_else(|| invalid("metadata keys must be constant strings"))?;
            if let Some(value) = input.metadata().get(key) {
                metadata.insert(key.to_owned(), value.clone());
            }
        }
        Ok(Arc::new(input.as_ref().clone().with_metadata(metadata)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        args.args
            .first()
            .cloned()
            .ok_or_else(|| invalid("requires an input value"))
    }
}

/// Native UNION schema merging may retain annotations absent from another input,
/// and its physical operator uses the first input's metadata. Project the actual
/// intersection on each input so grouping and downstream planning see the same field contract.
pub(crate) fn union(inputs: Vec<Arc<LogicalPlan>>) -> Result<LogicalPlan> {
    // Like LogicalPlanBuilder::union, leave primitive coercion to the native
    // analyzer. Preparation also runs this derivation before analysis.
    let native = Union::try_new_with_loose_types(inputs)?;
    let common_metadata = native
        .schema
        .fields()
        .iter()
        .enumerate()
        .map(|(ordinal, _)| {
            let first = native.inputs[0].schema().field(ordinal);
            first
                .metadata()
                .iter()
                .filter(|(key, value)| {
                    native.inputs.iter().all(|input| {
                        input.schema().field(ordinal).metadata().get(*key) == Some(*value)
                    })
                })
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();
    let mut inputs = Vec::with_capacity(native.inputs.len());
    for input in native.inputs {
        let mut changed = false;
        let expressions = input
            .schema()
            .iter()
            .zip(&common_metadata)
            .map(|((qualifier, field), common)| {
                let value = Expr::Column(Column::new(qualifier.cloned(), field.name()));
                if field.metadata().len() == common.len()
                    && common
                        .iter()
                        .all(|(key, value)| field.metadata().get(key) == Some(value))
                {
                    return Ok(value);
                }
                changed = true;
                Ok(retain(value, common.keys().cloned()).alias(field.name()))
            })
            .collect::<Result<Vec<_>>>()?;
        inputs.push(if changed {
            Arc::new(
                LogicalPlanBuilder::from(input.as_ref().clone())
                    .project(expressions)?
                    .build()?,
            )
        } else {
            input
        });
    }
    Ok(LogicalPlan::Union(Union::try_new_with_loose_types(inputs)?))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("pse_retain_metadata: {reason}"))
}
