// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native output declarations and exact Arrow diagnostic codecs.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "explicit native-plan fixtures"
)]

use datafusion::arrow::array::{
    FixedSizeBinaryArray, Float64Array, RecordBatch, StringArray, UInt64Array,
};
use datafusion::common::ScalarValue;
use datafusion::execution::runtime_env::RuntimeEnv;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
    output::{checked_literal, declare_relation_output, declare_relation_projection},
    scalar,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one semantic-field scenario follows actual values through preparation, execution and output admission"
)]
async fn native_codecs_preserve_exact_values_and_output_schema_inside_the_plan() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "source",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Values.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Key.",
            ),
            FieldContract::payload("namespace", FieldContract::id(), "Label namespace."),
            FieldContract::payload(
                "label",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "Label.",
            ),
            FieldContract::payload(
                "value",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Exact floating bits.",
            ),
        ]),
    );
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "labels",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Diagnostic values.",
        )
        .pk(&["label_id"])
        .columns(vec![
            FieldContract::key("label_id", FieldContract::id(), "Identity only."),
            FieldContract::payload("key", FieldContract::row_key(), "Exact key."),
            FieldContract::payload(
                "literal",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "Exact bits.",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let source = registry.relation("authored.source").unwrap();
    let namespace = SemanticId::from_bytes([7; 16]);
    let batch = RecordBatch::try_new(
        Arc::new(pse_schema::arrow::relation_schema(&registry, source).unwrap()),
        vec![
            Arc::new(UInt64Array::from(vec![u64::MAX, 2])),
            Arc::new(
                FixedSizeBinaryArray::try_from_iter(
                    [namespace.as_bytes(), namespace.as_bytes()].into_iter(),
                )
                .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["quoted\"\n", "second"])),
            Arc::new(Float64Array::from(vec![
                -0.0,
                f64::from_bits(0x7ff8_0000_0000_002a),
            ])),
        ],
    )
    .unwrap();
    let key = source.key;
    let threads = NonZeroUsize::new(1).unwrap();
    let budget = FixedBudget::new(256 << 20);
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        budget.clone(),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: threads,
            target_partitions: threads,
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::default();
    let session = factory
        .candidate(
            BTreeMap::from([(key, batch)]),
            Arc::clone(&registry),
            &cancel,
        )
        .unwrap();
    let scan = LogicalPlanBuilder::scan(
        session.table_reference(&key).unwrap(),
        session.table_source(&key).unwrap(),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    let source_values = session
        .checked_input(&key)
        .unwrap()
        .batch()
        .column(0)
        .to_data()
        .buffers()[0]
        .as_ptr();
    let passthrough = declare_relation_output(scan.clone(), &registry, source).unwrap();
    let passthrough = session.prepare(passthrough, &cancel).unwrap();
    let passthrough = passthrough.execute(&cancel).await.unwrap();
    assert_eq!(
        passthrough.batches()[0].column(0).to_data().buffers()[0].as_ptr(),
        source_values,
        "metadata transport retains the original values allocation"
    );
    let encoded = scalar::key(source.id, vec![("id", col("id")), ("label", col("label"))]);
    let plan = LogicalPlanBuilder::from(scan.clone())
        .project(vec![
            scalar::named_id(col("namespace"), encoded.clone()).alias("label_id"),
            encoded.alias("key"),
            scalar::literal(col("value")).alias("literal"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let target = registry.relation("authored.labels").unwrap();
    let plan = declare_relation_output(plan, &registry, target).unwrap();
    assert!(plan.display_indent().to_string().contains("with_metadata"));
    assert_eq!(
        plan.schema().as_arrow(),
        &pse_schema::arrow::relation_schema(&registry, target).unwrap()
    );
    let prepared = session.prepare(plan, &cancel).unwrap();
    assert!(
        prepared
            .optimized_plan()
            .display_indent()
            .to_string()
            .contains("with_metadata")
    );
    let completed = prepared.execute(&cancel).await.unwrap();
    let before_materialization = budget.reserved();
    let output = completed
        .checked_relation(&registry, target, &cancel)
        .unwrap();
    assert_eq!(
        budget.reserved(),
        before_materialization,
        "single-batch materialization transfers its existing claim"
    );
    let second = completed
        .checked_relation(&registry, target, &cancel)
        .unwrap();
    assert_eq!(budget.reserved(), before_materialization);
    assert!(Arc::ptr_eq(
        output.batch().column(0),
        second.batch().column(0)
    ));
    let batch = output.batch();
    let keys = batch
        .column(1)
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .unwrap();
    let literals = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(keys.value(0).len(), 32);
    assert_ne!(keys.value(0), keys.value(1));
    assert_eq!(literals.value(0), "[\"f64\",\"8000000000000000\"]");
    assert_eq!(literals.value(1), "[\"f64\",\"7ff800000000002a\"]");
    let ids = batch
        .column(0)
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .unwrap();
    assert_eq!(
        ids.value(0),
        pse_ids::derive_id(
            pse_ids::derive::context::NAMED,
            &[namespace.as_bytes(), keys.value(0)]
        )
        .as_bytes()
    );

    let repeated = LogicalPlanBuilder::from(completed.prepared().original_plan().clone())
        .union(completed.prepared().original_plan().clone())
        .unwrap()
        .build()
        .unwrap();
    let repeated = declare_relation_output(repeated, &registry, target).unwrap();
    let repeated = session
        .prepare(repeated, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    assert!(
        repeated.batches().len() > 1,
        "exercise actual multi-batch materialization"
    );
    let first = repeated
        .checked_relation(&registry, target, &cancel)
        .unwrap();
    let reserved = budget.reserved();
    let again = repeated
        .checked_relation(&registry, target, &cancel)
        .unwrap();
    assert_eq!(first.batch().num_rows(), 4);
    assert_eq!(
        budget.reserved(),
        reserved,
        "repeat access shares the concatenated result allocation"
    );
    assert!(Arc::ptr_eq(
        first.batch().column(0),
        again.batch().column(0)
    ));

    // The ancillary key and selected value travel with the declared row through
    // a real native sort. No second query reconstructs their correspondence.
    let scan = LogicalPlanBuilder::from(scan)
        .sort([col("id").sort(true, false)])
        .unwrap()
        .build()
        .unwrap();
    let projected = declare_relation_projection(
        scan,
        &registry,
        source,
        vec![
            scalar::key(source.id, vec![("id", col("id"))]).alias("actual_key"),
            col("label").alias("selected_override"),
        ],
    )
    .unwrap();
    let complete = session
        .prepare(projected, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let before = budget.reserved();
    for batch in complete.batches() {
        let checked = pse_relations::columnar::FieldCheckedBatch::admit_owned_projection(
            &registry,
            source,
            batch,
            &[0, 1, 2, 3],
        )
        .unwrap();
        let actual_ids = checked
            .batch()
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap();
        let labels = checked
            .batch()
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let keys = batch
            .column(4)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap();
        let overrides = batch
            .column(5)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        for index in 0..batch.num_rows() {
            assert_eq!(keys.value(index).len(), 32);
            assert!(matches!(actual_ids.value(index), 2 | u64::MAX));
            assert_eq!(labels.value(index), overrides.value(index));
        }
        let retained = checked.retained(budget.as_ref(), &cancel).unwrap();
        assert_eq!(retained.batch().num_rows(), batch.num_rows());
        assert_eq!(
            budget.reserved(),
            before,
            "ancillary projection does not charge its native buffers again"
        );
    }
}

#[test]
fn explicit_literal_and_output_declarations_refuse_unproved_semantic_meaning() {
    let registry = pse_schema::registry().unwrap();
    let units = registry.relation("reference.units").unwrap();
    let id = units.column("unit_id").unwrap();
    assert!(checked_literal(registry, id, ScalarValue::UInt64(Some(9))).is_err());
    let plan = LogicalPlanBuilder::empty(true)
        .project(vec![
            Expr::Literal(ScalarValue::FixedSizeBinary(16, Some(vec![1; 16])), None)
                .alias("unit_id"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let mut target = units.clone();
    target.columns.truncate(1);
    // A copied ID/fingerprint with a different declaration cannot establish output meaning.
    assert!(declare_relation_output(plan, registry, &target).is_err());

    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "one_id",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "One semantic identity.",
        )
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::id(),
            "Identity.",
        )]),
    );
    let registry = builder.build().unwrap();
    let plan = LogicalPlanBuilder::empty(true)
        .project(vec![
            Expr::Literal(ScalarValue::FixedSizeBinary(16, Some(vec![1; 16])), None).alias("id"),
        ])
        .unwrap()
        .build()
        .unwrap();
    // Exact storage and a real complete target still cannot invent semantic-ID meaning.
    assert!(
        declare_relation_output(
            plan,
            &registry,
            registry.relation("authored.one_id").unwrap()
        )
        .is_err()
    );
}

#[test]
fn conditional_identity_keeps_meaning_and_only_common_relation_annotations() {
    use datafusion::logical_expr::{ExprSchemable, lit};
    use pse_catalog::session::output::same_field_case;
    let registry = pse_schema::registry().unwrap();
    let schema = datafusion::common::DFSchema::empty();
    let key = checked_literal(
        registry,
        &FieldContract::key("key", FieldContract::id(), "Identity"),
        ScalarValue::FixedSizeBinary(16, Some(vec![1; 16])),
    )
    .unwrap();
    let absent = checked_literal(
        registry,
        &FieldContract::payload("optional", FieldContract::id(), "Optional identity").optional(),
        ScalarValue::FixedSizeBinary(16, None),
    )
    .unwrap();
    let expression = same_field_case(&schema, lit(true), key.clone(), absent).unwrap();
    let (_, field) = expression.to_field(&schema).unwrap();
    assert_eq!(
        field
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
            .map(String::as_str),
        Some("pse.semantic_id")
    );
    assert!(!field.metadata().contains_key(pse_schema::arrow::KEY_ROLE));
    assert!(
        same_field_case(
            &schema,
            lit(true),
            key,
            lit(ScalarValue::FixedSizeBinary(16, None))
        )
        .is_err()
    );
}

#[tokio::test]
async fn conditional_enum_literals_keep_the_declared_string_domain_and_values() {
    use datafusion::logical_expr::lit;
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let truth = FieldContract::payload("truth", FieldContract::enumeration("TruthValue"), "Guard");
    let value = |text: &str| {
        checked_literal(&registry, &truth, ScalarValue::Utf8(Some(text.to_owned()))).unwrap()
    };
    assert!(
        checked_literal(
            &registry,
            &truth,
            ScalarValue::Utf8(Some("not-a-member".to_owned()))
        )
        .is_err()
    );
    let input = LogicalPlanBuilder::values(vec![vec![lit(true)], vec![lit(false)]])
        .unwrap()
        .build()
        .unwrap();
    let condition = Expr::Column(input.schema().columns()[0].clone());
    let expression = pse_catalog::session::output::same_field_cases(
        input.schema(),
        vec![
            (lit(false), value("unknown")),
            (condition, value("true")),
            (lit(true), value("false")),
        ],
        value("unknown"),
    )
    .unwrap();
    let plan = LogicalPlanBuilder::from(input)
        .project([expression.alias("truth")])
        .unwrap()
        .build()
        .unwrap();
    let one = NonZeroUsize::new(1).unwrap();
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(128 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(BTreeMap::new(), Arc::clone(&registry), &cancel)
        .unwrap();
    let result = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut actual = Vec::new();
    for owned in result.batches() {
        let batch = owned.batch();
        let field = batch.schema().field(0).clone();
        assert_eq!(field.data_type(), &truth.data_type());
        assert_eq!(
            field
                .metadata()
                .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                .map(String::as_str),
            Some("enum:TruthValue")
        );
        assert!(!field.is_nullable());
        let values = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        actual.extend(values.iter().map(|value| value.unwrap().to_owned()));
    }
    assert_eq!(actual, ["true", "false"]);
}
