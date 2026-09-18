// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated projection of four native CDF images; no Delta storage operation.
use super::*;
use datafusion::arrow::datatypes::Schema;
use datafusion::{
    arrow::{
        array::{
            Int64Array, RecordBatch, StringArray, StructArray, TimestampMillisecondArray,
            TimestampNanosecondArray, UInt64Array,
        },
        datatypes::{DataType, Field},
    },
    datasource::MemTable,
    execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
};
use pse_ids::FixedBudget;
use pse_relations::generated::runtime::publications::{
    RuntimePublicationsFieldMembersItem as Member,
    RuntimePublicationsFieldMembersItemSelection as Selection,
};
use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

#[expect(
    clippy::too_many_lines,
    reason = "one four-image fixture and its independent Arrow assertions"
)]
#[tokio::test]
async fn cdf_images_keep_typed_keys_values_versions_and_timestamps() {
    let mut registry = pse_schema::RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut registry);
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "CDF unit input",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::Int64), "Key"),
            FieldContract::payload("value", FieldContract::native(DataType::Int64), "Value"),
        ]),
    );
    let registry = Arc::new(registry.build().unwrap());
    let spec = registry.relation("authored.values").unwrap();
    let member = Member {
        catalog_name: "test".into(),
        schema_name: "authored".into(),
        table_name: "values".into(),
        table_uri: "memory:///cdf-unit/".into(),
        delta_version: 3,
        relation_id: spec.id,
        relation_version: 1,
        contract_fingerprint: spec.fingerprint,
        selection: Selection::from_full(),
    };
    let mut fields = pse_schema::arrow::relation_schema(&registry, spec)
        .unwrap()
        .fields()
        .to_vec();
    fields.extend([
        Arc::new(Field::new("_commit_version", DataType::UInt64, false)),
        Arc::new(Field::new("_change_type", DataType::Utf8, false)),
        Arc::new(Field::new(
            "_commit_timestamp",
            DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Millisecond, None),
            false,
        )),
    ]);
    let schema = Arc::new(Schema::new(fields));
    let kinds = vec!["insert", "delete", "update_preimage", "update_postimage"];
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![7; 4])),
            Arc::new(Int64Array::from(vec![10, 10, 10, 11])),
            Arc::new(UInt64Array::from(vec![1, 2, 3, 3])),
            Arc::new(StringArray::from(kinds.clone())),
            Arc::new(TimestampMillisecondArray::from(vec![1000; 4])),
        ],
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let factory = crate::session::SessionFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(32 << 20),
        "cdf-unit",
        SessionStateBuilder::new_with_default_features(),
    );
    let session = factory
        .candidate_checked(std::collections::BTreeMap::new(), registry, &cancel)
        .unwrap();
    let source = TableReference::full("native", "cdf", "images");
    let provider: Arc<dyn TableProvider> =
        Arc::new(MemTable::try_new(schema, vec![vec![batch]]).unwrap());
    let session = session
        .with_provider(source.clone(), provider.clone(), &cancel)
        .unwrap();
    let input = LogicalPlanBuilder::scan(source, provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let plan = normalize(&session, &member, input).unwrap();
    let batches = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .into_batches();
    let change = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    assert_eq!(
        change.column_by_name("kind").unwrap().as_ref(),
        &StringArray::from(kinds)
    );
    assert_eq!(
        change.column_by_name("commit_version").unwrap().as_ref(),
        &Int64Array::from(vec![1, 2, 3, 3])
    );
    assert_eq!(
        change.column_by_name("committed_at").unwrap().as_ref(),
        &TimestampNanosecondArray::from(vec![1_000_000_000; 4]).with_timezone("UTC")
    );
    let keys = change.column_by_name("row_key").unwrap();
    assert_eq!(keys.slice(2, 1).as_ref(), keys.slice(3, 1).as_ref());
    let values = batches[0]
        .column(1)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    assert_eq!(
        values.column_by_name("value").unwrap().as_ref(),
        &Int64Array::from(vec![10, 10, 10, 11])
    );
}
