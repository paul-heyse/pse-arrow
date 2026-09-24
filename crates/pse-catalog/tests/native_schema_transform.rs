// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit schema edits use native projection and checked nullability operations.

use pse_columnar::CancellationToken;
use pse_engine::session::{EngineSession, ExecutionSettings, ThreadBudget};
use pse_ids::SemanticId;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, FieldContract, MigrationSpec, MigrationStep, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

#[expect(
    clippy::unwrap_used,
    reason = "fixed declarations and source arrays must admit before exercising native execution"
)]
#[allow(
    clippy::too_many_lines,
    reason = "keep the complete independent nested fixture and its assertions together"
)]
fn fixture(values: &[serde_json::Value]) -> EngineSession {
    let declaration = |version, columns| {
        RelationDecl::new(
            Namespace::Authored,
            "samples",
            version,
            Authority::Authored,
            SnapshotClass::Model,
            "Explicit schema transformation source and destination.",
        )
        .pk(&["id"])
        .columns(columns)
    };
    let id = || {
        FieldContract::key(
            "id",
            FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
            "Row identity.",
        )
    };
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(declaration(
        1,
        vec![
            id(),
            FieldContract::label(
                "old_label",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "Label.",
            ),
            FieldContract::payload(
                "obsolete",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Removed value.",
            ),
            FieldContract::payload(
                "required",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Optional source value.",
            )
            .optional(),
        ],
    ));
    builder.declare_relation(declaration(
        2,
        vec![
            id(),
            FieldContract::label(
                "new_label",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "Label.",
            ),
            FieldContract::payload(
                "required",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Required destination value.",
            ),
            FieldContract::payload(
                "exact",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Exact default.",
            ),
            FieldContract::payload(
                "tags",
                FieldContract::list(FieldContract::id()),
                "Semantic nested default.",
            ),
        ],
    ));
    let default = |name: &str, value: serde_json::Value| {
        let field = builder
            .declared_relations()
            .iter()
            .find(|r| r.key.version == 2)
            .unwrap()
            .columns
            .iter()
            .find(|f| f.name() == name)
            .unwrap()
            .field()
            .clone();
        pse_schema::NativeLiteral::from_json(Arc::new(field), &value.to_string()).unwrap()
    };
    let exact = default("exact", serde_json::json!(["f64", "8000000000000000"]));
    let tags = default(
        "tags",
        serde_json::json!(["list", [["id", SemanticId::from_bytes([7; 16]).to_hex()]]]),
    );
    builder.declare_migration(MigrationSpec {
        relation: "authored.samples",
        from_version: 1,
        to_version: 2,
        steps: vec![
            MigrationStep::RenameColumn {
                from: "old_label",
                to: "new_label",
            },
            MigrationStep::DropColumn("obsolete"),
            MigrationStep::AddColumn {
                name: "exact",
                default: exact,
            },
            MigrationStep::AddColumn {
                name: "tags",
                default: tags,
            },
            MigrationStep::ChangeNullable {
                name: "required",
                nullable: false,
            },
        ],
        doc: "Rename, drop, add an exact default and require a present value.",
    });
    let registry = Arc::new(builder.build().unwrap());
    let source = registry
        .relations()
        .iter()
        .find(|relation| relation.key.version == 1)
        .unwrap();
    let rows = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            vec![
                serde_json::json!(["u64", u64::try_from(index).unwrap()]),
                serde_json::json!(["text", format!("row-{index}")]),
                serde_json::json!(["f64", format!("{:016x}", f64::to_bits(99.))]),
                value.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::testing::batch_from_literals(&registry, source, &rows).unwrap();
    let checked = FieldCheckedBatch::admit(&registry, source, batch).unwrap();
    pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
    )
    .unwrap()
    .candidate_checked_roles(
        BTreeMap::from([("source".to_owned(), checked)]),
        registry,
        &CancellationToken::new(),
    )
    .unwrap()
}

#[tokio::test]
async fn declared_schema_edits_preserve_rows_and_exact_defaults_in_the_native_plan() {
    let session = fixture(&[
        serde_json::json!(["f64", format!("{:016x}", f64::to_bits(7.))]),
        serde_json::json!(["f64", format!("{:016x}", f64::to_bits(-2.))]),
    ]);
    let cancel = CancellationToken::new();
    let prepared = session
        .prepare_schema_transform("authored.samples@1->2", "source", &cancel)
        .unwrap();
    let display = prepared.original_plan().display_indent().to_string();
    assert!(display.contains("Projection"));
    assert!(display.contains("pse_require_nonnull"));
    let complete = prepared.execute(&cancel).await.unwrap();
    let target = session.registry().relation("authored.samples").unwrap();
    let result = complete
        .checked_relation(session.registry(), target, &cancel)
        .unwrap();
    let mut rows =
        pse_relations::testing::literals_from_batch(session.registry(), target, result.batch())
            .unwrap();
    rows.sort_by_key(|row| row[0].to_string());
    assert_eq!(rows.len(), 2);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            row[0],
            serde_json::json!(["u64", u64::try_from(index).unwrap()])
        );
        assert_eq!(row[1], serde_json::json!(["text", format!("row-{index}")]));
        assert_eq!(
            row[2],
            serde_json::json!(["f64", format!("{:016x}", f64::to_bits([7., -2.][index]))])
        );
        assert_eq!(
            row[3].to_string(),
            serde_json::json!(["f64", format!("{:016x}", f64::to_bits(-0.0))]).to_string()
        );
        assert_eq!(
            row[4],
            serde_json::json!([
                "list",
                vec![serde_json::json!([
                    "id",
                    (SemanticId::from_bytes([7; 16])).to_hex()
                ])]
            ])
        );
    }
    assert!(
        session
            .prepare_schema_transform("authored.samples@2->3", "source", &cancel)
            .is_err()
    );
    assert!(
        session
            .prepare_schema_transform("authored.samples@1->2", "missing", &cancel)
            .is_err()
    );
    let with_destination = session
        .with_checked_role_inputs(
            BTreeMap::from([("destination".to_owned(), result.clone())]),
            &cancel,
        )
        .unwrap();
    assert!(
        with_destination
            .prepare_schema_transform("authored.samples@1->2", "destination", &cancel,)
            .is_err()
    );
    let source_key = session
        .input_roles()
        .find(|(role, _)| *role == "source")
        .unwrap()
        .1;
    let source = session.registry().relation_by_key(source_key).unwrap();
    let source_plan = pse_engine::session::output::declare_relation_output(
        session.scan_role("source").unwrap(),
        session.registry(),
        source,
    )
    .unwrap();
    let original = session
        .prepare(source_plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let original = original
        .checked_relation(session.registry(), source, &cancel)
        .unwrap();
    assert!(
        session
            .with_checked_workspace(
                BTreeMap::from([(source.key, original), (target.key, result)]),
                &cancel,
            )
            .is_err(),
        "two versions must not overwrite the same native table name"
    );
}

#[tokio::test]
async fn narrowing_nullability_fails_execution_on_an_actual_null() {
    let session = fixture(&[
        serde_json::json!(["f64", format!("{:016x}", f64::to_bits(7.))]),
        serde_json::json!(["null", null]),
    ]);
    let cancel = CancellationToken::new();
    let prepared = session
        .prepare_schema_transform("authored.samples@1->2", "source", &cancel)
        .unwrap();
    assert!(prepared.execute(&cancel).await.is_err());
}
