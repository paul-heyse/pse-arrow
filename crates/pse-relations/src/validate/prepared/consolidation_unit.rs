// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(clippy::unwrap_used, reason = "native local validation fixtures")]
use super::*;
use crate::native::arrow::{
    array::{
        ArrayRef, Float32Array, Int32Array, Int64Array, ListArray, RunArray, StructArray,
        UnionArray, types::Int32Type,
    },
    buffer::{NullBuffer, OffsetBuffer},
    datatypes::{DataType, UnionFields, UnionMode},
};
use datafusion::prelude::SessionContext;
use pse_schema::{RegistryBuilder, model::IntegerRange};
fn report(registry: &Registry, field: &Field, array: ArrayRef, limit: usize) -> ValidationReport {
    let context = ValidationContext::for_registry(registry).unwrap();
    let prepared = context.column(registry, field).unwrap();
    let batch = RecordBatch::try_new(Arc::clone(prepared.schema()), vec![array]).unwrap();
    prepared
        .evaluate(&batch, limit, &pse_columnar::CancellationToken::new())
        .unwrap()
}
#[test]
fn preparation_reuses_exact_owner_and_schema_and_rejects_foreign_registry() {
    let registry = RegistryBuilder::new().build().unwrap();
    let other = RegistryBuilder::new().build().unwrap();
    let context = ValidationContext::for_registry(&registry).unwrap();
    assert!(Arc::ptr_eq(
        &context,
        &ValidationContext::for_registry(&registry).unwrap()
    ));
    let field = Field::new("a.b", DataType::Float32, false);
    let first = context.column(&registry, &field).unwrap();
    assert!(Arc::ptr_eq(
        &first,
        &context.column(&registry, &field).unwrap()
    ));
    assert_eq!(context.prepared_count().unwrap(), 1);
    assert!(context.column(&other, &field).is_err());
    let changed = field.with_metadata(HashMap::from([(
        "unknown.semantic".into(),
        "changed".into(),
    )]));
    assert!(!Arc::ptr_eq(
        &first,
        &context.column(&registry, &changed).unwrap()
    ));
    assert_eq!(context.prepared_count().unwrap(), 2);
}
#[test]
fn nested_findings_retain_original_rows_exact_paths_and_total_when_sampled() {
    let registry = RegistryBuilder::new().build().unwrap();
    let child = Arc::new(IntegerRange::nonnegative(3).field("item"));
    let array = ListArray::new(
        Arc::clone(&child),
        OffsetBuffer::new(vec![0, 2, 3, 5].into()),
        Arc::new(Int64Array::from(vec![0, 9, 99, -1, 2])),
        Some(NullBuffer::from(vec![true, false, true])),
    );
    let field = Field::new("nested/key", array.data_type().clone(), true);
    let report = report(&registry, &field, Arc::new(array), 1);
    assert_eq!(report.violations, 2);
    assert!(report.truncated);
    assert!(!report.is_valid());
    let paths = report
        .findings
        .column_by_name("path")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(paths.value(0), "/nested~1key/1");
    let values = report
        .findings
        .column_by_name("observed_literal")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(values.value(0), r#"["i64",9]"#);
}
#[test]
fn hidden_struct_values_and_inactive_union_arms_are_not_evaluated() {
    let registry = RegistryBuilder::new().build().unwrap();
    let child = Arc::new(Field::new("number", DataType::Float32, false));
    let values = StructArray::new(
        vec![child].into(),
        vec![Arc::new(Float32Array::from(vec![f32::NAN, 1.0]))],
        Some(NullBuffer::from(vec![false, true])),
    );
    assert!(
        report(
            &registry,
            &Field::new("root", values.data_type().clone(), true),
            Arc::new(values),
            8
        )
        .is_valid()
    );
    let fields = UnionFields::try_new(
        vec![1, 3],
        vec![
            Field::new("safe", DataType::Int32, false),
            Field::new("float", DataType::Float32, false),
        ],
    )
    .unwrap();
    let array = UnionArray::try_new(
        fields.clone(),
        vec![1_i8, 3].into(),
        None,
        vec![
            Arc::new(Int32Array::from(vec![1, 2])),
            Arc::new(Float32Array::from(vec![f32::NAN, f32::INFINITY])),
        ],
    )
    .unwrap();
    let result = report(
        &registry,
        &Field::new("union", DataType::Union(fields, UnionMode::Sparse), false),
        Arc::new(array),
        8,
    );
    assert_eq!(result.violations, 1);
    assert_eq!(
        result
            .findings
            .column_by_name("path")
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .value(0),
        "/union/float"
    );
}
#[test]
fn encoded_values_use_visible_logical_occurrences_and_cancel_never_certifies() {
    let registry = RegistryBuilder::new().build().unwrap();
    let values = Float32Array::from(vec![f32::INFINITY, 1.0]);
    let run = RunArray::<Int32Type>::try_new(&Int32Array::from(vec![3, 5]), &values)
        .unwrap()
        .slice(2, 3);
    let field = Field::new("runs", run.data_type().clone(), false);
    assert_eq!(
        report(&registry, &field, Arc::new(run.clone()), 8).violations,
        1
    );
    let context = ValidationContext::for_registry(&registry).unwrap();
    let prepared = context.column(&registry, &field).unwrap();
    let batch = RecordBatch::try_new(Arc::clone(prepared.schema()), vec![Arc::new(run)]).unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    cancel.cancel();
    assert!(prepared.evaluate(&batch, 8, &cancel).is_err());
}

#[test]
fn local_masks_and_delta_expression_lowering_agree_without_storage_execution() {
    let registry = RegistryBuilder::new().build().unwrap();
    let child = Arc::new(IntegerRange::nonnegative(3).field("item"));
    let array: ArrayRef = Arc::new(ListArray::new(
        Arc::clone(&child),
        OffsetBuffer::new(vec![0, 2, 3, 4].into()),
        Arc::new(Int64Array::from(vec![0, 9, 99, 2])),
        Some(NullBuffer::from(vec![true, false, true])),
    ));
    let field = Field::new("values", array.data_type().clone(), true);
    let context = ValidationContext::for_registry(&registry).unwrap();
    let prepared = context.column(&registry, &field).unwrap();
    let batch = RecordBatch::try_new(Arc::clone(prepared.schema()), vec![array]).unwrap();
    let report = prepared
        .evaluate(&batch, 0, &pse_columnar::CancellationToken::new())
        .unwrap();
    assert!(report.truncated);
    assert_eq!(
        report.valid_rows,
        BooleanArray::from(vec![false, true, true])
    );
    let schema = DFSchema::try_from(batch.schema().as_ref().clone()).unwrap();
    let expression =
        super::super::predicates::relation(&registry, batch.schema().as_ref()).unwrap();
    let physical = context.state.prepare(expression, &schema).unwrap();
    let actual = physical
        .evaluate(&batch)
        .unwrap()
        .into_array(batch.num_rows())
        .unwrap();
    assert_eq!(
        actual.as_any().downcast_ref::<BooleanArray>().unwrap(),
        &report.valid_rows
    );
}

#[test]
fn actual_session_function_bindings_are_prepared_in_each_owner() {
    use crate::native::logical_expr::{ColumnarValue, Volatility, create_udf};
    let registry = RegistryBuilder::new().build().unwrap();
    let schema = Arc::new(Schema::new_with_metadata(
        vec![Field::new("value", DataType::Int64, false)],
        HashMap::from([(
            pse_schema::arrow::KEY_CHECKS.into(),
            r#"{"actual":"actual_policy(value)"}"#.into(),
        )]),
    ));
    let input = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(Int64Array::from(vec![1]))],
    )
    .unwrap();
    let owner = |accepted| {
        let session = SessionContext::new();
        session.register_udf(create_udf(
            "actual_policy",
            vec![DataType::Int64],
            DataType::Boolean,
            Volatility::Immutable,
            Arc::new(move |_| {
                Ok(ColumnarValue::Scalar(
                    crate::native::common::ScalarValue::Boolean(Some(accepted)),
                ))
            }),
        ));
        ValidationContext::new(&registry, session.state())
    };
    let allowed = owner(true);
    let refused = owner(false);
    let allowed = allowed.prepare(&registry, Arc::clone(&schema)).unwrap();
    let refused = refused.prepare(&registry, schema).unwrap();
    assert!(!Arc::ptr_eq(&allowed, &refused));
    let cancel = pse_columnar::CancellationToken::new();
    assert!(allowed.evaluate(&input, 8, &cancel).unwrap().is_valid());
    assert!(!refused.evaluate(&input, 8, &cancel).unwrap().is_valid());
}

#[test]
fn direct_predicates_simplify_coalesce_before_physical_evaluation() {
    let state = SessionContext::new().state();
    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Int64,
        true,
    )]));
    let expression = crate::native::functions::core::expr_fn::coalesce(vec![
        crate::native::logical_expr::col("value"),
        crate::native::logical_expr::lit(7_i64),
    ]);
    let physical = prepare_expression(
        &state,
        expression,
        &DFSchema::try_from(schema.as_ref().clone()).unwrap(),
    )
    .unwrap();
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Int64Array::from(vec![None, Some(3)]))],
    )
    .unwrap();
    let values = physical.evaluate(&batch).unwrap().into_array(2).unwrap();
    assert_eq!(
        values.as_any().downcast_ref::<Int64Array>().unwrap(),
        &Int64Array::from(vec![7, 3])
    );
}

#[test]
fn sql_checks_require_an_explicit_native_binding() {
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "checked",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "fixture",
        )
        .pk(&["n"])
        .columns(vec![FieldContract::key(
            "n",
            FieldContract::native(DataType::Int64),
            "key",
        )])
        .checks(BTreeMap::from([("positive".into(), "n > 0".into())])),
    );
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.checked").unwrap();
    let error = ValidationContext::for_registry(&registry)
        .unwrap()
        .relation(&registry, spec)
        .unwrap_err();
    assert!(error.to_string().contains("engine-bound ValidationContext"));
    ValidationContext::install_default(&registry, SessionContext::new().state()).unwrap();
    let prepared = ValidationContext::for_registry(&registry)
        .unwrap()
        .relation(&registry, spec)
        .unwrap();
    let batch = RecordBatch::try_new(
        prepared.schema().clone(),
        vec![Arc::new(Int64Array::from(vec![-1]))],
    )
    .unwrap();
    assert_eq!(
        prepared
            .evaluate(&batch, 1, &pse_columnar::CancellationToken::new())
            .unwrap()
            .violations,
        1
    );
}
