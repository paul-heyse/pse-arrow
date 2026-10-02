// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Optimizer nullability refinement does not authorize other field changes.
#![allow(clippy::unwrap_used, reason = "boundary assertions")]

use datafusion::{
    arrow::datatypes::{DataType, Field, Schema},
    physical_plan::{ExecutionPlan, empty::EmptyExec},
};
use std::{collections::HashMap, sync::Arc};

fn input(field: Field) -> Arc<dyn ExecutionPlan> {
    Arc::new(EmptyExec::new(Arc::new(Schema::new(vec![field]))))
}

#[test]
fn null_fixed_size_struct_lists_roundtrip_with_required_elements() {
    use datafusion::arrow::{
        array::{Array, ArrayRef, FixedSizeListArray, Int16Array, StructArray},
        buffer::NullBuffer,
    };
    let fields = vec![
        Arc::new(Field::new("num", DataType::Int16, false)),
        Arc::new(Field::new("den", DataType::Int16, false)),
    ]
    .into();
    let child = Arc::new(StructArray::new(
        fields,
        vec![
            Arc::new(Int16Array::from(vec![0, 0, 1, 2, 0, 0])),
            Arc::new(Int16Array::from(vec![0, 0, 1, 1, 0, 0])),
        ],
        Some(NullBuffer::from(vec![
            false, false, true, true, false, false,
        ])),
    ));
    let element = Arc::new(Field::new("element", child.data_type().clone(), false));
    let source = FixedSizeListArray::new(
        element,
        2,
        child,
        Some(NullBuffer::from(vec![false, true, false])),
    );
    let execution = Arc::new(Field::new("dimension", source.data_type().clone(), true));
    let layout = super::DurableLayout::new(Arc::new(Schema::new(vec![execution.clone()]))).unwrap();
    let storage = layout.storage_schema().fields()[0].clone();
    for (offset, length) in [(0, 3), (1, 2), (1, 1)] {
        let original: ArrayRef = Arc::new(source.slice(offset, length));
        original.to_data().validate_full().unwrap();
        let encoded =
            super::restore_array(&original, execution.data_type(), &storage, false).unwrap();
        encoded.to_data().validate_full().unwrap();
        assert_eq!(encoded.data_type(), storage.data_type());
        let decoded =
            super::restore_array(&encoded, storage.data_type(), &execution, true).unwrap();
        decoded.to_data().validate_full().unwrap();
        assert_eq!(decoded.to_data(), original.to_data());
    }
}

#[tokio::test]
async fn nested_leaf_extraction_preserves_same_named_durable_conversion() {
    use datafusion::{
        arrow::array::{BinaryArray, RecordBatch, StructArray},
        functions::core::expr_fn::get_field,
        logical_expr::{LogicalPlanBuilder, col},
        prelude::SessionContext,
    };
    let declaration = Arc::new(Schema::new(vec![Field::new_struct(
        "value",
        vec![Field::new("id", DataType::FixedSizeBinary(16), false)],
        false,
    )]));
    let layout = super::DurableLayout::new(declaration).unwrap();
    let DataType::Struct(fields) = layout.storage_schema().field(0).data_type() else {
        panic!("struct storage");
    };
    let batch = RecordBatch::try_new(
        Arc::clone(layout.storage_schema()),
        vec![Arc::new(StructArray::new(
            fields.clone(),
            vec![Arc::new(BinaryArray::from(vec![&[7u8; 16][..]]))],
            None,
        ))],
    )
    .unwrap();
    let fixture =
        pse_testkit::NativeFixture::new(std::num::NonZeroUsize::new(16 << 20).unwrap()).unwrap();
    let context = SessionContext::new_with_state(fixture.into_factory().native_state().clone());
    let input = context.read_batch(batch).unwrap().into_unoptimized_plan();
    let plan = LogicalPlanBuilder::from(layout.decode(input).unwrap())
        .project(vec![
            col("value"),
            get_field(col("value"), "id").alias("id"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let expected = plan.schema().clone();
    let optimized = context.state().optimize(&plan).unwrap();
    assert_eq!(optimized.schema(), &expected);
    let batches = context
        .execute_logical_plan(plan)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(
        batches[0].column(1).data_type(),
        &DataType::FixedSizeBinary(16)
    );
    assert_eq!(
        batches[0].schema().field(0).data_type(),
        layout.execution_schema().field(0).data_type()
    );
}

#[tokio::test]
async fn source_span_obligations_preserve_decoded_view_fields() {
    use datafusion::{arrow::array::RecordBatch, datasource::ViewTable, prelude::SessionContext};
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation("authored.entities").unwrap();
    let layout = super::DurableLayout::new(Arc::new(
        pse_schema::arrow::relation_schema(registry, spec).unwrap(),
    ))
    .unwrap();
    let fixture =
        pse_testkit::NativeFixture::new(std::num::NonZeroUsize::new(16 << 20).unwrap()).unwrap();
    let state = fixture.into_factory().native_state().clone();
    let config = state
        .config()
        .clone()
        .with_create_default_catalog_and_schema(true);
    let context = SessionContext::new_with_state(
        datafusion::execution::session_state::SessionStateBuilder::new_from_existing(state)
            .with_config(config)
            .build(),
    );
    let input = context
        .read_batch(RecordBatch::new_empty(layout.storage_schema().clone()))
        .unwrap()
        .into_unoptimized_plan();
    context
        .register_table(
            "entities",
            Arc::new(ViewTable::new(layout.decode(input).unwrap(), None)),
        )
        .unwrap();
    let selected = context
        .table("entities")
        .await
        .unwrap()
        .into_unoptimized_plan();
    let inputs = std::collections::BTreeMap::from([(spec.id, selected)]);
    let checks = pse_relations::validate::obligations::ObligationTemplates::new(registry)
        .bind(
            &inputs,
            &pse_engine::validation::NativeValidation(context.state()),
            None,
        )
        .unwrap();
    for check in checks {
        let optimized = context.state().optimize(&check).unwrap();
        assert_eq!(
            optimized.schema().as_arrow().fields(),
            check.schema().as_arrow().fields()
        );
    }
}

#[test]
fn exact_boundary_widens_only_the_outer_nullable_promise() {
    let field = Field::new("value", DataType::Int64, false)
        .with_metadata(HashMap::from([("meaning".into(), "unchanged".into())]));
    let schema = Arc::new(Schema::new(vec![field.clone().with_nullable(true)]));
    let result = super::declared_output(input(field), &schema).unwrap();
    assert_eq!(result.schema(), schema);
}

#[test]
fn exact_boundary_refuses_new_meaning_or_stronger_value_claims() {
    let field = Field::new("value", DataType::Int64, true);
    for target in [
        field.clone().with_nullable(false),
        field.clone().with_name("other"),
        field.clone().with_data_type(DataType::UInt64),
        field
            .clone()
            .with_metadata(HashMap::from([("meaning".into(), "invented".into())])),
    ] {
        assert!(
            super::declared_output(input(field.clone()), &Arc::new(Schema::new(vec![target])))
                .is_err()
        );
    }
    let source = Field::new_list("values", Field::new("item", DataType::Int64, false), false);
    let target = Field::new_list("values", Field::new("item", DataType::Int64, true), true);
    assert!(super::declared_output(input(source), &Arc::new(Schema::new(vec![target]))).is_err());
}

#[tokio::test]
async fn durable_decode_checks_visible_binary_children_only() {
    use datafusion::{
        arrow::{
            array::{Array, BinaryArray, RecordBatch, StructArray},
            buffer::NullBuffer,
        },
        prelude::SessionContext,
    };

    let target = Arc::new(Field::new("id", DataType::FixedSizeBinary(16), false));
    let declaration = Arc::new(Schema::new(vec![Field::new_struct(
        "value",
        vec![target],
        true,
    )]));
    let layout = super::DurableLayout::new(declaration).unwrap();
    let DataType::Struct(fields) = layout.storage_schema().field(0).data_type() else {
        panic!("expected struct storage")
    };
    for visible in [false, true] {
        let array = StructArray::new(
            fields.clone(),
            vec![Arc::new(BinaryArray::from(vec![
                Some(&[][..]),
                Some(&[7u8; 16][..]),
            ]))],
            Some(NullBuffer::from(vec![visible, true])),
        );
        let context = SessionContext::new();
        let batch =
            RecordBatch::try_new(Arc::clone(layout.storage_schema()), vec![Arc::new(array)])
                .unwrap();
        let input = context.read_batch(batch).unwrap().into_unoptimized_plan();
        let decoded = context
            .execute_logical_plan(layout.decode(input).unwrap())
            .await
            .unwrap()
            .collect()
            .await;
        if visible {
            assert!(
                decoded.is_err(),
                "a visible malformed identity must still refuse"
            );
        } else {
            let batches = decoded.unwrap();
            let values = batches[0]
                .column(0)
                .as_any()
                .downcast_ref::<StructArray>()
                .unwrap();
            assert_eq!(values.len(), 2);
            assert!(values.is_null(0));
            assert!(values.is_valid(1));
        }
    }
}

#[test]
fn storage_visibility_preserves_sliced_boolean_values_and_parent_masks() {
    use datafusion::arrow::{
        array::{Array, BooleanArray, StructArray},
        buffer::NullBuffer,
    };
    let values = (0..24)
        .map(|i| (i != 9).then_some(i % 2 == 0))
        .collect::<Vec<_>>();
    let source = StructArray::new(
        vec![Arc::new(Field::new("flag", DataType::Boolean, true))].into(),
        vec![Arc::new(BooleanArray::from(values.clone()))],
        Some(NullBuffer::from(
            (0..24).map(|i| i != 8).collect::<Vec<_>>(),
        )),
    );
    // A non-byte-aligned child offset, a parent null, an independent child null,
    // and visible true/false values exercise the validity and value coordinates.
    for (offset, length) in [(0, 24), (7, 5), (8, 4), (9, 0)] {
        let result = super::visibility::storage(Arc::new(source.slice(offset, length))).unwrap();
        result.to_data().validate_full().unwrap();
        let actual = result.as_any().downcast_ref::<StructArray>().unwrap();
        let child = actual
            .column(0)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .unwrap();
        for i in 0..length {
            assert_eq!(actual.is_null(i), offset + i == 8);
            assert_eq!(
                child.iter().nth(i).unwrap(),
                if offset + i == 8 {
                    None
                } else {
                    values[offset + i]
                }
            );
        }
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "paired visible/hidden attacks across three native container representations"
)]
async fn durable_decode_discards_only_hidden_list_and_map_child_ranges() {
    use datafusion::{
        arrow::{
            array::{
                Array, ArrayRef, BinaryArray, FixedSizeBinaryArray, FixedSizeListArray, ListArray,
                MapArray, RecordBatch, StringArray, StructArray,
            },
            buffer::{NullBuffer, OffsetBuffer},
        },
        prelude::SessionContext,
    };

    let item = Arc::new(Field::new("item", DataType::FixedSizeBinary(16), false));
    let entries = Arc::new(Field::new_struct(
        "entries",
        vec![
            Arc::new(Field::new("key", DataType::Utf8, false)),
            Arc::new(Field::new("value", DataType::FixedSizeBinary(16), false)),
        ],
        false,
    ));
    for kind in [
        DataType::List(item.clone()),
        DataType::FixedSizeList(item, 1),
        DataType::Map(entries, false),
    ] {
        let layout =
            super::DurableLayout::new(Arc::new(Schema::new(vec![Field::new("value", kind, true)])))
                .unwrap();
        for visible in [false, true] {
            let bytes: ArrayRef =
                Arc::new(BinaryArray::from(vec![Some(&[][..]), Some(&[7u8; 16][..])]));
            let offsets = OffsetBuffer::new(vec![0, 1, 2].into());
            let nulls = Some(NullBuffer::from(vec![visible, true]));
            let array: ArrayRef = match layout.storage_schema().field(0).data_type() {
                DataType::List(field) => {
                    Arc::new(ListArray::try_new(field.clone(), offsets, bytes, nulls).unwrap())
                }
                DataType::Map(field, _) => {
                    let DataType::Struct(fields) = field.data_type() else {
                        panic!("expected entries")
                    };
                    let entries = StructArray::new(
                        fields.clone(),
                        vec![
                            Arc::new(StringArray::from(vec!["hidden", "visible"])),
                            bytes,
                        ],
                        None,
                    );
                    Arc::new(
                        MapArray::try_new(field.clone(), offsets, entries, nulls, false).unwrap(),
                    )
                }
                _ => panic!("expected container storage"),
            };
            let batch =
                RecordBatch::try_new(Arc::clone(layout.storage_schema()), vec![array]).unwrap();
            let context = SessionContext::new();
            let input = context.read_batch(batch).unwrap().into_unoptimized_plan();
            let result = context
                .execute_logical_plan(layout.decode(input).unwrap())
                .await
                .unwrap()
                .collect()
                .await;
            if visible {
                assert!(result.is_err(), "a visible malformed child must refuse");
                continue;
            }
            let batches = result.unwrap();
            let values = batches[0].column(0);
            assert_eq!(values.len(), 2);
            assert!(values.is_null(0));
            let child = match values.data_type() {
                DataType::List(_) => values
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .unwrap()
                    .value(1),
                DataType::FixedSizeList(_, _) => values
                    .as_any()
                    .downcast_ref::<FixedSizeListArray>()
                    .unwrap()
                    .value(1),
                DataType::Map(_, _) => values
                    .as_any()
                    .downcast_ref::<MapArray>()
                    .unwrap()
                    .value(1)
                    .column(1)
                    .clone(),
                _ => panic!("expected decoded container"),
            };
            assert_eq!(
                child
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .unwrap()
                    .value(0),
                &[7u8; 16]
            );
        }
    }
}

#[test]
fn only_proven_representation_widening_omits_inverse_checks() {
    assert!(!super::needs_roundtrip(
        &DataType::Utf8,
        &DataType::LargeUtf8
    ));
    assert!(!super::needs_roundtrip(
        &DataType::Binary,
        &DataType::BinaryView
    ));
    assert!(super::needs_roundtrip(
        &DataType::Float32,
        &DataType::Float16
    ));
    assert!(super::needs_roundtrip(&DataType::Int16, &DataType::UInt8));
    assert!(super::needs_roundtrip(
        &DataType::Binary,
        &DataType::FixedSizeBinary(16)
    ));
}
