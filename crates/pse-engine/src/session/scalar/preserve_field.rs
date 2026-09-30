// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked identity for the pinned engine's scalar nested-field expansion.
//! The input expression establishes the field; execution can restore missing
//! annotations only, and never convert units, domains, values or child layouts.

use datafusion::{
    arrow::datatypes::{DataType, Field, FieldRef},
    common::{DataFusionError, Result, tree_node::Transformed},
    logical_expr::{
        ColumnarValue, Expr, LogicalPlan, Projection, ReturnFieldArgs, ScalarFunctionArgs,
        ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
};
use pse_columnar::MemoryPool;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(super) fn function(pool: Arc<dyn MemoryPool>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(PreserveField {
        signature: Signature::any(1, Volatility::Immutable),
        pool,
    }))
}

#[derive(Debug)]
struct PreserveField {
    signature: Signature,
    pool: Arc<dyn MemoryPool>,
}
impl PartialEq for PreserveField {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.pool, &other.pool)
    }
}
impl Eq for PreserveField {}
impl Hash for PreserveField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.pool), state);
    }
}
impl ScalarUDFImpl for PreserveField {
    fn name(&self) -> &'static str {
        "pse_preserve_field"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        let [field] = args else {
            return Err(invalid("requires one established input field"));
        };
        Ok(field.clone())
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [field] = args.arg_fields else {
            return Err(invalid("requires one established input field"));
        };
        Ok(Arc::clone(field))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [value] = args.args.as_slice() else {
            return Err(invalid("requires one input value"));
        };
        let [input] = args.arg_fields.as_slice() else {
            return Err(invalid("requires one input field"));
        };
        if input.data_type() != args.return_field.data_type()
            || input.metadata() != args.return_field.metadata()
            || input.is_nullable() != args.return_field.is_nullable()
            || input.dict_is_ordered() != args.return_field.dict_is_ordered()
        {
            return Err(invalid(
                "return field differs from its actual input expression",
            ));
        }
        let expected = args.return_field.data_type();
        if !missing_metadata_only(&value.data_type(), expected) {
            return Err(DataFusionError::Execution(
                "pse_preserve_field: execution changed the established nested field layout or meaning".to_owned(),
            ));
        }
        if let ColumnarValue::Array(array) = value
            && array.data_type() == expected
        {
            return Ok(ColumnarValue::Array(Arc::clone(array)));
        }
        // A scalar with the right type may lose child annotations during broadcast.
        // Produce the checked Array now, before ProjectionExec's scalar shortcut.
        super::list_field::cast_field(args, &self.pool, "query:preserve-field")
    }
}

/// Final logical materialization after constant folding. The call is observable
/// and immutable; no volatile marker is used to suppress optimization or reuse.
pub(crate) fn materialize(plan: LogicalPlan, function: &Arc<ScalarUDF>) -> Result<LogicalPlan> {
    Ok(plan
        .transform_up_with_subqueries(|node| {
            let LogicalPlan::Projection(projection) = node else {
                return Ok(Transformed::no(node));
            };
            let mut changed = false;
            let expressions = projection
                .expr
                .iter()
                .zip(projection.schema.fields())
                .map(|(expr, field)| {
                    if !nested_metadata(field.data_type()) || wrapped(expr, function) {
                        return expr.clone();
                    }
                    changed = true;
                    function.call(vec![expr.clone()]).alias(field.name())
                })
                .collect();
            if !changed {
                return Ok(Transformed::no(LogicalPlan::Projection(projection)));
            }
            Ok(Transformed::yes(LogicalPlan::Projection(
                Projection::try_new_with_schema(expressions, projection.input, projection.schema)?,
            )))
        })?
        .data)
}
fn wrapped(expr: &Expr, function: &Arc<ScalarUDF>) -> bool {
    match expr {
        Expr::Alias(alias) => wrapped(&alias.expr, function),
        Expr::ScalarFunction(call) => Arc::ptr_eq(&call.func, function),
        _ => false,
    }
}
pub(crate) fn nested_metadata(data_type: &DataType) -> bool {
    match data_type {
        DataType::List(field)
        | DataType::LargeList(field)
        | DataType::ListView(field)
        | DataType::LargeListView(field)
        | DataType::FixedSizeList(field, _)
        | DataType::Map(field, _) => {
            !field.metadata().is_empty() || nested_metadata(field.data_type())
        }
        DataType::Struct(fields) => fields
            .iter()
            .any(|field| !field.metadata().is_empty() || nested_metadata(field.data_type())),
        DataType::Union(fields, _) => fields
            .iter()
            .any(|(_, field)| !field.metadata().is_empty() || nested_metadata(field.data_type())),
        DataType::RunEndEncoded(runs, values) => [runs, values]
            .iter()
            .any(|field| !field.metadata().is_empty() || nested_metadata(field.data_type())),
        DataType::Dictionary(_, value) => nested_metadata(value),
        _ => false,
    }
}
fn field_missing_metadata_only(actual: &Field, expected: &Field, refine: bool) -> bool {
    if std::ptr::eq(actual, expected) {
        return true;
    }
    actual.name() == expected.name()
        && (actual.is_nullable() == expected.is_nullable() || (refine && actual.is_nullable()))
        && actual.dict_is_ordered() == expected.dict_is_ordered()
        && pse_columnar::native_field::admits_metadata(
            actual.metadata(), expected.metadata(),
            pse_columnar::native_field::MetadataAdmission::RestoreEstablished,
        )
        && layout_compatible(actual.data_type(), expected.data_type(), refine)
}
fn layout_compatible(actual: &DataType, expected: &DataType, refine: bool) -> bool {
    // Arrow equality omits dictionary ordering. Check native children even when
    // the type compares equal; shared field pointers still short-circuit above.
    if std::ptr::eq(actual, expected) {
        return true;
    }
    match (actual, expected) {
        (DataType::List(a), DataType::List(b))
        | (DataType::LargeList(a), DataType::LargeList(b))
        | (DataType::ListView(a), DataType::ListView(b))
        | (DataType::LargeListView(a), DataType::LargeListView(b)) => {
            field_missing_metadata_only(a, b, refine)
        }
        (DataType::FixedSizeList(a, n), DataType::FixedSizeList(b, m)) => {
            n == m && field_missing_metadata_only(a, b, refine)
        }
        (DataType::Map(a, n), DataType::Map(b, m)) => {
            n == m && field_missing_metadata_only(a, b, refine)
        }
        (DataType::Struct(a), DataType::Struct(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(a, b)| field_missing_metadata_only(a, b, refine))
        }
        (DataType::Union(a, am), DataType::Union(b, bm)) => {
            am == bm
                && a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|((ai, a), (bi, b))| ai == bi && field_missing_metadata_only(a, b, refine))
        }
        (DataType::RunEndEncoded(ar, av), DataType::RunEndEncoded(br, bv)) => {
            field_missing_metadata_only(ar, br, refine)
                && field_missing_metadata_only(av, bv, refine)
        }
        (DataType::Dictionary(a, b), DataType::Dictionary(c, d)) => {
            a == c && layout_compatible(b, d, refine)
        }
        _ => actual == expected,
    }
}
pub(crate) fn missing_metadata_only(actual: &DataType, expected: &DataType) -> bool {
    layout_compatible(actual, expected, false)
}
/// A native function's proven input-to-output correspondence may strengthen
/// nullability that its default type-only return inference discarded.
pub(crate) fn proven_native_layout(actual: &DataType, expected: &DataType) -> bool {
    layout_compatible(actual, expected, true)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("pse_preserve_field: {reason}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn restoration_requires_established_annotations_and_exact_nested_layout() {
        let field = Field::new("member", DataType::Utf8, false);
        let expected = field.clone().with_metadata(HashMap::from([("custom.meaning".into(), "a".into())]));
        let list = |field: Field| DataType::List(Arc::new(field));
        assert!(missing_metadata_only(&list(field.clone()), &list(expected.clone())));
        assert!(!missing_metadata_only(&list(expected.clone()), &list(field.clone())));
        let conflict = field.clone().with_metadata(HashMap::from([("custom.meaning".into(), "b".into())]));
        assert!(!missing_metadata_only(&list(conflict), &list(expected.clone())));
        assert!(!missing_metadata_only(&list(field.clone().with_name("renamed")), &list(expected.clone())));
        let nullable = list(field.with_nullable(true));
        assert!(!missing_metadata_only(&nullable, &list(expected.clone())));
        assert!(proven_native_layout(&nullable, &list(expected)));
    }

    #[test]
    fn restoration_never_changes_dictionary_ordering_even_when_arrow_types_compare_equal() {
        let field = Field::new_dictionary("value", DataType::Int32, DataType::Utf8, false);
        let list = |field: Field| DataType::List(Arc::new(field));
        let expected = list(field.clone().with_dict_is_ordered(true));
        let actual = list(field);
        assert_eq!(actual, expected);
        assert!(!missing_metadata_only(&actual, &expected));
        assert!(!proven_native_layout(&actual, &expected));
    }
}
