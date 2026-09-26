// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::too_many_lines,
    reason = "isolated native obligation fixtures"
)]
use super::*;
use crate::native::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    execution::context::SessionContext,
};
use pse_schema::{RegistryBuilder, model::*};
use std::sync::Arc;

#[test]
fn selective_binding_preserves_relational_checks_and_still_validates_schema() {
    let registry = pse_schema::catalog::assemble().unwrap();
    let spec = registry.relation("reference.elements").unwrap();
    let context = SessionContext::new();
    let batch = RecordBatch::new_empty(Arc::new(
        pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
    ));
    let inputs = BTreeMap::from([(
        spec.id,
        context.read_batch(batch).unwrap().into_unoptimized_plan(),
    )]);
    let templates = ObligationTemplates::new(&registry);
    let all = templates
        .bind_classified(&inputs, &context.state())
        .unwrap();
    let relational = templates
        .bind_required(&inputs, &context.state(), |_, kind| {
            kind == ObligationKind::Relational
        })
        .unwrap();
    assert!(!relational.is_empty());
    assert_eq!(relational.len() + 1, all.len());
    assert!(
        relational
            .iter()
            .all(|check| check.kind == ObligationKind::Relational)
    );
    let invalid = BTreeMap::from([(spec.id, LogicalPlanBuilder::empty(false).build().unwrap())]);
    assert!(
        templates
            .bind_required(&invalid, &context.state(), |_, _| false)
            .is_err()
    );
}

#[tokio::test]
async fn token_collisions_compare_actual_key_tuples() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "keys",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "fixture",
        )
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::native(DataType::Int64),
            "key",
        )])
        .pk(&["id"]),
    );
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.keys").unwrap();
    let context = SessionContext::new();
    for (ids, expected) in [(vec![1, 1], 0), (vec![1, 2], 1)] {
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)])),
            vec![Arc::new(Int64Array::from(ids))],
        )
        .unwrap();
        let input = context.read_batch(batch).unwrap().into_unoptimized_plan();
        let plan = key_collisions(
            input,
            spec,
            lit(crate::native::common::ScalarValue::FixedSizeBinary(
                32,
                Some(vec![0; 32]),
            )),
        )
        .unwrap();
        let batches = context
            .execute_logical_plan(plan)
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        assert_eq!(
            batches.iter().map(RecordBatch::num_rows).sum::<usize>(),
            expected
        );
    }
}

#[tokio::test]
async fn selected_native_relations_check_keys_references_and_nested_ordinals() {
    use crate::native::arrow::{
        array::{ArrayRef, FixedSizeBinaryArray, ListArray},
        buffer::OffsetBuffer,
    };
    let mut builder = RegistryBuilder::new();
    let declaration = |name| {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "native obligation fixture",
        )
        .pk(&["id"])
    };
    builder.declare_relation(declaration("target").columns(vec![FieldContract::key(
        "id",
        FieldContract::id(),
        "key",
    )]));
    builder.declare_relation(declaration("source").columns(vec![
            FieldContract::key("id", FieldContract::id(), "key"),
            FieldContract::reference("target_id", FieldContract::id(), "target")
                .with_fk("authored.target", "id"),
            FieldContract::list(FieldContract::extended(ExtensionUse::OrdinalRef {
                target: "authored.target",
            }))
            .with_name("ordinals"),
        ]));
    let registry = builder.build().unwrap();
    let target = registry.relation("authored.target").unwrap();
    let source = registry.relation("authored.source").unwrap();
    let identities = |values: Vec<u8>| -> ArrayRef {
        Arc::new(
            FixedSizeBinaryArray::try_from_iter(values.into_iter().map(|id| [id; 16])).unwrap(),
        )
    };
    let context = SessionContext::new();
    for (target_id, ordinal, duplicate, include_target) in [
        (1, 0, false, true),
        (9, 0, false, true),
        (1, 1, false, true),
        (1, 0, true, true),
        (1, 0, false, false),
    ] {
        let target_batch = RecordBatch::try_new(
            Arc::new(pse_schema::arrow::relation_schema(&registry, target).unwrap()),
            vec![identities(vec![1])],
        )
        .unwrap();
        let count = if duplicate { 2 } else { 1 };
        let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, source).unwrap());
        let DataType::List(child) = schema.field(2).data_type() else {
            unreachable!()
        };
        let indices = ListArray::new(
            Arc::clone(child),
            OffsetBuffer::new((0..=count).collect::<Vec<i32>>().into()),
            Arc::new(Int64Array::from(vec![
                ordinal;
                usize::try_from(count).unwrap()
            ])),
            None,
        );
        let batch = RecordBatch::try_new(
            schema,
            vec![
                identities(vec![2; usize::try_from(count).unwrap()]),
                identities(vec![target_id; usize::try_from(count).unwrap()]),
                Arc::new(indices),
            ],
        )
        .unwrap();
        let mut inputs = RelationInputs::from([(
            source.id,
            context.read_batch(batch).unwrap().into_unoptimized_plan(),
        )]);
        if include_target {
            inputs.insert(
                target.id,
                context
                    .read_batch(target_batch)
                    .unwrap()
                    .into_unoptimized_plan(),
            );
        }
        let plans = ObligationTemplates::new(&registry)
            .bind(&inputs, &context.state())
            .unwrap();
        let mut violations = 0;
        for plan in plans {
            violations += context
                .execute_logical_plan(plan)
                .await
                .unwrap()
                .collect()
                .await
                .unwrap()
                .iter()
                .map(RecordBatch::num_rows)
                .sum::<usize>();
        }
        assert_eq!(
            violations == 0,
            target_id == 1 && ordinal == 0 && !duplicate && include_target
        );
    }
}

#[tokio::test]
async fn compiled_nested_composite_references_agree_with_invariant_sql() {
    use crate::native::arrow::{
        array::{ArrayRef, ListArray, MapArray, StructArray},
        buffer::OffsetBuffer,
    };
    use datafusion::catalog::MemorySchemaProvider;
    for map in [false, true] {
        let mut builder = RegistryBuilder::new();
        let declaration = |name| {
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "composite fixture",
            )
        };
        builder.declare_relation(
            declaration("target").pk(&["tenant", "id"]).columns(
                ["tenant", "id"]
                    .map(|name| FieldContract::native(DataType::Int64).with_name(name))
                    .to_vec(),
            ),
        );
        let key = FieldContract::structure(
            ["tenant", "id"]
                .map(|name| {
                    FieldContract::native(DataType::Int64)
                        .with_name(name)
                        .with_nullable(!map || name != "tenant")
                })
                .to_vec(),
        )
        .with_reference(&ReferenceContract {
            relation: "authored.target".into(),
            columns: ["tenant", "id"]
                .map(|name| ReferenceColumn {
                    source: vec![name.into()],
                    target: name.into(),
                })
                .to_vec(),
            null_policy: ReferenceNullPolicy::AllOrNone,
        })
        .unwrap();
        builder.declare_relation(declaration("source").pk(&["id"]).columns(vec![
            FieldContract::native(DataType::Int64).with_name("id"),
            if map {
                FieldContract::native(DataType::Map(
                    Arc::new(key.with_name("entries").into_field()),
                    false,
                ))
                .with_name("keys")
            } else {
                FieldContract::list(key).with_name("keys")
            },
        ]));
        let registry = builder.build().unwrap();
        let source = registry.relation("authored.source").unwrap();
        let target = registry.relation("authored.target").unwrap();
        let product = registry.obligations(source.key).unwrap();
        assert_eq!(product.references.len(), 1);
        assert_eq!(product.references[0].path, ["keys", "[]"]);
        let invariant = registry
            .invariants()
            .iter()
            .find(|i| i.relation == "authored.source" && i.kind == InvariantKind::ForeignKey)
            .unwrap();
        for (tenant, id, expected) in [
            (Some(10), Some(1), 0),
            (Some(10), Some(2), 1),
            (None, None, 0),
        ] {
            if map && tenant.is_none() {
                continue;
            }
            let context = SessionContext::new();
            context
                .catalog("datafusion")
                .unwrap()
                .register_schema("authored", Arc::new(MemorySchemaProvider::new()))
                .unwrap();
            let target_schema =
                Arc::new(pse_schema::arrow::relation_schema(&registry, target).unwrap());
            let target_batch = RecordBatch::try_new(
                target_schema,
                vec![
                    Arc::new(Int64Array::from(vec![10, 20])),
                    Arc::new(Int64Array::from(vec![1, 2])),
                ],
            )
            .unwrap();
            let source_schema =
                Arc::new(pse_schema::arrow::relation_schema(&registry, source).unwrap());
            let (DataType::List(item) | DataType::Map(item, _)) =
                source_schema.field(1).data_type()
            else {
                unreachable!()
            };
            let DataType::Struct(fields) = item.data_type() else {
                unreachable!()
            };
            let values = StructArray::new(
                fields.clone(),
                vec![
                    Arc::new(Int64Array::from(if map {
                        vec![tenant, Some(20)]
                    } else {
                        vec![tenant]
                    })),
                    Arc::new(Int64Array::from(if map {
                        vec![id, Some(2)]
                    } else {
                        vec![id]
                    })),
                ],
                None,
            );
            let keys: ArrayRef = if map {
                Arc::new(MapArray::new(
                    item.clone(),
                    OffsetBuffer::new(vec![0, 2].into()),
                    values,
                    None,
                    false,
                ))
            } else {
                Arc::new(ListArray::new(
                    item.clone(),
                    OffsetBuffer::new(vec![0, 1].into()),
                    Arc::new(values),
                    None,
                ))
            };
            let source_batch = RecordBatch::try_new(
                source_schema.clone(),
                vec![Arc::new(Int64Array::from(vec![7])), keys],
            )
            .unwrap();
            context
                .register_batch("authored.source", source_batch.clone())
                .unwrap();
            context
                .register_batch("authored.target", target_batch.clone())
                .unwrap();
            let input = context
                .read_batch(source_batch)
                .unwrap()
                .into_unoptimized_plan();
            let inputs = BTreeMap::from([(
                target.id,
                context
                    .read_batch(target_batch)
                    .unwrap()
                    .into_unoptimized_plan(),
            )]);
            let mut violations = 0;
            for plan in references::plans(&input, source, &inputs, &registry).unwrap() {
                violations += context
                    .execute_logical_plan(plan)
                    .await
                    .unwrap()
                    .collect()
                    .await
                    .unwrap()
                    .iter()
                    .map(RecordBatch::num_rows)
                    .sum::<usize>();
            }
            let queried = context
                .sql(&invariant.query)
                .await
                .unwrap()
                .collect()
                .await
                .unwrap()
                .iter()
                .map(RecordBatch::num_rows)
                .sum::<usize>();
            assert_eq!((violations, queried), (expected, expected));
        }
    }
}
