// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit schema edits use native projection and checked nullability operations.

use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, Cell, FieldContract, MigrationSpec, MigrationStep, Namespace, RelationDecl,
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
fn fixture(values: &[Cell]) -> SnapshotSession {
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
                default: Cell::F64(-0.0),
            },
            MigrationStep::AddColumn {
                name: "tags",
                default: Cell::List(vec![Cell::Id(SemanticId::from_bytes([7; 16]))]),
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
                Cell::U64(u64::try_from(index).unwrap()),
                Cell::text(format!("row-{index}")),
                Cell::F64(99.),
                value.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::cells::batch_from_cells(&registry, source, &rows).unwrap();
    let checked = FieldCheckedBatch::admit(&registry, source, batch).unwrap();
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
        native_engine_profile(),
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
    let session = fixture(&[Cell::F64(7.), Cell::F64(-2.)]);
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
        pse_relations::cells::cells_from_batch(session.registry(), target, result.batch()).unwrap();
    rows.sort_by_key(|row| row[0].literal_spec());
    assert_eq!(rows.len(), 2);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row[0], Cell::U64(u64::try_from(index).unwrap()));
        assert_eq!(row[1], Cell::text(format!("row-{index}")));
        assert_eq!(row[2], Cell::F64([7., -2.][index]));
        assert_eq!(row[3].literal_spec(), Cell::F64(-0.0).literal_spec());
        assert_eq!(
            row[4],
            Cell::List(vec![Cell::Id(SemanticId::from_bytes([7; 16]))])
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
    let source_plan = pse_catalog::session::output::declare_relation_output(
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
    let session = fixture(&[Cell::F64(7.), Cell::Null]);
    let cancel = CancellationToken::new();
    let prepared = session
        .prepare_schema_transform("authored.samples@1->2", "source", &cancel)
        .unwrap();
    assert!(prepared.execute(&cancel).await.is_err());
}
