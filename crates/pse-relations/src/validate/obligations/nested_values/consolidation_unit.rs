// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::too_many_lines,
    reason = "native nested occurrence assertions"
)]

use super::*;
use crate::native::{
    arrow::{
        array::{
            Array, ArrayRef, Int64Array, LargeListViewArray, ListViewArray, RecordBatch,
            StructArray,
            builder::{Int64Builder, MapBuilder},
        },
        buffer::NullBuffer,
        datatypes::Schema,
    },
    execution::context::SessionContext,
    physical_plan::collect,
};
use std::{collections::HashMap, sync::Arc};

fn member() -> Field {
    Field::new("item", DataType::Int64, false)
        .with_metadata(HashMap::from([("test.reference".into(), "true".into())]))
}
async fn values(array: ArrayRef) -> Vec<i64> {
    values_named("nested", array).await
}
async fn values_named(name: &str, array: ArrayRef) -> Vec<i64> {
    let field = Field::new(name, array.data_type().clone(), true);
    let context = SessionContext::new();
    let input = context
        .read_batch(
            RecordBatch::try_new(Arc::new(Schema::new(vec![field.clone()])), vec![array]).unwrap(),
        )
        .unwrap()
        .into_unoptimized_plan();
    let occurrences = occurrences(&input, [&field], |field| {
        field.metadata().contains_key("test.reference")
    })
    .unwrap();
    assert_eq!(occurrences.len(), 1);
    let state = context.state();
    collect(
        state
            .create_physical_plan(&occurrences[0].input)
            .await
            .unwrap(),
        state.task_ctx(),
    )
    .await
    .unwrap()
    .into_iter()
    .flat_map(|batch| {
        batch
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .values()
            .to_vec()
    })
    .collect()
}

#[tokio::test]
async fn nested_value_columns_do_not_collide_with_leaf_projection_aliases() {
    for name in ["value", "__pse_nested_value_0", "__pse_nested_value_1"] {
        let child = StructArray::new(
            vec![Arc::new(member().with_name("value"))].into(),
            vec![Arc::new(Int64Array::from(vec![7, 9, 7]))],
            None,
        );
        let parent = StructArray::new(
            vec![Arc::new(Field::new(
                "quantity",
                child.data_type().clone(),
                true,
            ))]
            .into(),
            vec![Arc::new(child)],
            Some(NullBuffer::from(vec![true, false, true])),
        );
        assert_eq!(values_named(name, Arc::new(parent)).await, [7, 7]);
    }
}

#[tokio::test]
async fn list_views_respect_visible_slices_and_container_masks() {
    let child = Arc::new(member());
    let storage: ArrayRef = Arc::new(Int64Array::from(vec![7, 9, 8]));
    let small: ArrayRef = Arc::new(
        ListViewArray::try_new(
            Arc::clone(&child),
            vec![2, 0, 1, 1].into(),
            vec![1, 1, 1, 0].into(),
            Arc::clone(&storage),
            Some(NullBuffer::from(vec![true, true, false, true])),
        )
        .unwrap(),
    );
    let large: ArrayRef = Arc::new(
        LargeListViewArray::try_new(
            child,
            vec![2_i64, 0, 1, 1].into(),
            vec![1_i64, 1, 1, 0].into(),
            storage,
            Some(NullBuffer::from(vec![true, true, false, true])),
        )
        .unwrap(),
    );
    for array in [small, large] {
        let mut result = values(array).await;
        result.sort_unstable();
        assert_eq!(result, [7, 8]);
    }
}

#[tokio::test]
async fn map_values_and_null_structs_do_not_create_hidden_reference_claims() {
    let mut map = MapBuilder::new(None, Int64Builder::new(), Int64Builder::new())
        .with_values_field(member().with_name("values"));
    for (value, visible) in [(7, true), (9, false), (8, true)] {
        map.keys().append_value(value);
        map.values().append_value(value);
        map.append(visible).unwrap();
    }
    let mut result = values(Arc::new(map.finish())).await;
    result.sort_unstable();
    assert_eq!(result, [7, 8]);
    let array = StructArray::new(
        vec![Arc::new(member())].into(),
        vec![Arc::new(Int64Array::from(vec![7, 9]))],
        Some(NullBuffer::from(vec![true, false])),
    );
    assert_eq!(values(Arc::new(array)).await, [7]);
}

#[tokio::test]
async fn encoded_union_and_run_values_preserve_active_occurrences() {
    use crate::native::arrow::{
        array::{Int32Array, RunArray, UnionArray, types::Int32Type},
        datatypes::{UnionFields, UnionMode},
    };
    let union: ArrayRef = Arc::new(
        UnionArray::try_new(
            UnionFields::try_new(
                [1, 3],
                vec![
                    Arc::new(member().with_name("active")),
                    Arc::new(Field::new("other", DataType::Int64, false)),
                ],
            )
            .unwrap(),
            vec![1_i8, 3, 1].into(),
            Some(vec![0_i32, 0, 1].into()),
            vec![
                Arc::new(Int64Array::from(vec![7, 8])),
                Arc::new(Int64Array::from(vec![99])),
            ],
        )
        .unwrap(),
    );
    assert!(matches!(
        union.data_type(),
        DataType::Union(_, UnionMode::Dense)
    ));
    assert_eq!(values(union).await, [7, 8]);
    let run = RunArray::<Int32Type>::try_new(
        &Int32Array::from(vec![2, 4]),
        &Int64Array::from(vec![7, 9]),
    )
    .unwrap();
    let DataType::RunEndEncoded(runs, _) = run.data_type() else {
        unreachable!()
    };
    let run = arrow_array::make_array(
        run.to_data()
            .into_builder()
            .data_type(DataType::RunEndEncoded(
                Arc::clone(runs),
                Arc::new(member().with_name("values")),
            ))
            .build()
            .unwrap(),
    );
    assert_eq!(values(run.slice(1, 2)).await, [7, 9]);
}
