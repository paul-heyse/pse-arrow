// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixtures fail with direct diagnostics"
)]
//! Direct relation admission, recursive cell round trips and invalid-value boundaries.

use arrow_schema::{DataType, Schema};
use pse_ids::{ContentHash, SemanticId};
use pse_relations::cells::{batch_from_cells, cells_from_batch};
use pse_relations::validate::{validate_batch, validate_bundle, validate_field, validate_schema};
use pse_schema::Registry;
use pse_schema::builder::RegistryBuilder;
use pse_schema::model::{
    Authority, Cell, ColumnSpec, EnumDecl, EnumMember, ExtensionUse, LogicalType, Namespace,
    RelationDecl, SnapshotClass,
};
use std::collections::BTreeMap;

fn id(value: u8) -> Cell {
    Cell::Id(SemanticId::from_bytes([value; 16]))
}
fn decl(name: &'static str, columns: Vec<ColumnSpec>) -> RelationDecl {
    let mut all = vec![ColumnSpec::key("id", LogicalType::id(), "identity")];
    all.extend(columns);
    RelationDecl::new(
        Namespace::Authored,
        name,
        1,
        Authority::Authored,
        SnapshotClass::Model,
        "fixture",
    )
    .pk(&["id"])
    .columns(all)
}
fn fixture() -> Registry {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("one", "one"), EnumMember::new("two", "two")],
    ));
    builder.declare_relation(decl("target", vec![]));
    let types = vec![
        ("i64", LogicalType::I64),
        ("i32", LogicalType::I32),
        ("u8", LogicalType::U8),
        ("u16", LogicalType::U16),
        ("u32", LogicalType::U32),
        ("u64", LogicalType::U64),
        ("bool", LogicalType::Bool),
        ("text", LogicalType::Text),
        ("time", LogicalType::Timestamp),
        ("float", LogicalType::F64),
        ("hash", LogicalType::Ext(ExtensionUse::ContentHash)),
        ("choice", LogicalType::Ext(ExtensionUse::Enum("Choice"))),
        ("bound", LogicalType::Ext(ExtensionUse::Bound)),
        ("dimension", LogicalType::Ext(ExtensionUse::DimensionVector)),
        ("quantity", LogicalType::Ext(ExtensionUse::QuantityValue)),
        ("index", LogicalType::Ext(ExtensionUse::IndexTuple)),
        ("span", LogicalType::Ext(ExtensionUse::SourceSpan)),
        ("dsl", LogicalType::Ext(ExtensionUse::ExprDsl)),
        ("path", LogicalType::Ext(ExtensionUse::TargetPath)),
        (
            "ordinal",
            LogicalType::Ext(ExtensionUse::OrdinalRef {
                target: "authored.target",
            }),
        ),
        (
            "list",
            LogicalType::list(LogicalType::Ext(ExtensionUse::Enum("Choice"))),
        ),
        ("fixed", LogicalType::fixed_list(LogicalType::id(), 2)),
        (
            "nested",
            LogicalType::Struct(vec![
                (
                    "choice",
                    LogicalType::Ext(ExtensionUse::Enum("Choice")),
                    false,
                ),
                ("list", LogicalType::list(LogicalType::id()), true),
            ]),
        ),
    ];
    builder.declare_relation(decl(
        "values",
        types
            .into_iter()
            .map(|(name, ty)| ColumnSpec::payload(name, ty, "value").optional())
            .collect(),
    ));
    builder.build().unwrap()
}
fn row() -> Vec<Cell> {
    vec![
        id(1),
        Cell::I64(-9),
        Cell::I64(i64::from(i32::MIN)),
        Cell::U64(255),
        Cell::U64(65535),
        Cell::U64(u64::from(u32::MAX)),
        Cell::U64(u64::MAX),
        Cell::Bool(true),
        Cell::text("λ\n"),
        Cell::I64(-1000),
        Cell::F64(-0.0),
        Cell::Hash(ContentHash::from_bytes([7; 32])),
        Cell::Enum("two"),
        Cell::Struct(vec![Cell::Enum("finite"), Cell::F64(0.0)]),
        Cell::List(
            (0..8)
                .map(|_| Cell::Struct(vec![Cell::I64(0), Cell::I64(1)]))
                .collect(),
        ),
        Cell::Struct(vec![Cell::F64(1.5), id(4), id(5)]),
        Cell::List(vec![id(8), id(9)]),
        Cell::Struct(vec![id(4), Cell::U64(2), Cell::U64(5)]),
        Cell::text("x + 1"),
        Cell::text("fs.a.*"),
        Cell::U64(0),
        Cell::List(vec![Cell::Enum("two"), Cell::Enum("one")]),
        Cell::List(vec![id(2), id(3)]),
        Cell::Struct(vec![Cell::Enum("one"), Cell::List(vec![id(3)])]),
    ]
}

#[test]
fn every_declared_layout_round_trips_including_null_parents_and_slices() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let mut absent = vec![Cell::Null; spec.columns.len()];
    absent[0] = id(2);
    let rows = vec![row(), absent];
    let batch = batch_from_cells(&reg, spec, &rows).unwrap();
    assert_eq!(cells_from_batch(&reg, spec, &batch).unwrap(), rows);
    assert_eq!(
        cells_from_batch(&reg, spec, &batch.slice(1, 1)).unwrap(),
        rows[1..]
    );
    let empty = batch_from_cells(&reg, spec, &[]).unwrap();
    assert_eq!(empty.num_rows(), 0);
}

#[test]
fn all_extension_factories_reject_wrong_storage_versions_and_extra_metadata() {
    use pse_relations::ext::validate_extension;
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let mut count = 0;
    for field in schema.fields() {
        if !field
            .metadata()
            .contains_key(pse_schema::arrow::KEY_EXTENSION_NAME)
        {
            continue;
        }
        count += 1;
        assert!(validate_extension(field).is_ok());
        assert!(
            validate_extension(&field.as_ref().clone().with_data_type(DataType::Boolean)).is_err()
        );
        for corrupt in [
            field.metadata()[pse_schema::arrow::KEY_EXTENSION_METADATA]
                .replace("\"v\":1", "\"v\":2"),
            "{\"v\":1,\"extra\":false}".to_owned(),
            "{\"v\":1,\"v\":1}".to_owned(),
        ] {
            let mut metadata = field.metadata().clone();
            metadata.insert(
                pse_schema::arrow::KEY_EXTENSION_METADATA.to_owned(),
                corrupt,
            );
            assert!(validate_extension(&field.as_ref().clone().with_metadata(metadata)).is_err());
        }
    }
    assert_eq!(count, 11);
}

#[test]
fn matching_fingerprint_does_not_admit_forged_schema_or_registry_descriptor() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let mut fields = schema
        .fields()
        .iter()
        .map(|field| field.as_ref().clone())
        .collect::<Vec<_>>();
    fields[1] = fields[1].clone().with_data_type(DataType::UInt64);
    let forged = Schema::new_with_metadata(fields, schema.metadata().clone());
    assert!(validate_schema(&reg, spec, &forged).is_err());
    let mut descriptor = spec.clone();
    descriptor.primary_key = vec!["u64"];
    assert!(validate_schema(&reg, &descriptor, &schema).is_err());
    let mut metadata = schema.metadata().clone();
    metadata.insert("unknown".to_owned(), "x".to_owned());
    assert!(validate_schema(&reg, spec, &schema.clone().with_metadata(metadata)).is_err());
}

#[test]
fn invalid_visible_values_fail_with_unchanged_contract_hash() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let mutations = vec![
        ("choice", Cell::Enum("invalid")),
        ("u8", Cell::U64(256)),
        ("i32", Cell::I64(i64::MAX)),
        (
            "bound",
            Cell::Struct(vec![Cell::Enum("finite"), Cell::F64(f64::INFINITY)]),
        ),
        (
            "bound",
            Cell::Struct(vec![Cell::Enum("unbounded"), Cell::F64(0.0)]),
        ),
        (
            "span",
            Cell::Struct(vec![id(1), Cell::U64(8), Cell::U64(4)]),
        ),
        (
            "quantity",
            Cell::Struct(vec![Cell::F64(f64::NAN), id(1), id(2)]),
        ),
        (
            "dimension",
            Cell::List(
                (0..8)
                    .map(|_| Cell::Struct(vec![Cell::I64(0), Cell::I64(2)]))
                    .collect(),
            ),
        ),
        ("fixed", Cell::List(vec![id(1)])),
    ];
    for (name, invalid) in mutations {
        let mut values = row();
        let index = spec
            .columns
            .iter()
            .position(|column| column.name == name)
            .unwrap();
        values[index] = invalid;
        assert!(
            batch_from_cells(&reg, spec, &[values]).is_err(),
            "admitted invalid {name}"
        );
    }
}

#[test]
fn masked_struct_payload_does_not_leak_invalid_dictionary_values() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let mut values = vec![Cell::Null; spec.columns.len()];
    values[0] = id(1);
    // The builder physically stores empty enum strings and invalid dimension defaults
    // below absent parents. Admission must ignore those invisible bytes.
    let batch = batch_from_cells(&reg, spec, &[values.clone()]).unwrap();
    assert_eq!(cells_from_batch(&reg, spec, &batch).unwrap(), vec![values]);
}

#[test]
fn bundle_compares_keys_references_and_nested_ordinals_directly() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl("target", vec![]));
    builder.declare_relation(decl(
        "source",
        vec![
            ColumnSpec::reference("target_id", LogicalType::id(), "target")
                .with_fk("authored.target", "id"),
            ColumnSpec::payload(
                "ordinals",
                LogicalType::list(LogicalType::Ext(ExtensionUse::OrdinalRef {
                    target: "authored.target",
                })),
                "ordinals",
            ),
        ],
    ));
    let reg = builder.build().unwrap();
    let target = reg.relation("authored.target").unwrap();
    let source = reg.relation("authored.source").unwrap();
    let target_batch = batch_from_cells(&reg, target, &[vec![id(1)]]).unwrap();
    for (target_id, ordinal, duplicate) in [
        (id(1), 0, false),
        (id(9), 0, false),
        (id(1), 1, false),
        (id(1), 0, true),
    ] {
        let row = vec![id(2), target_id, Cell::List(vec![Cell::U64(ordinal)])];
        let rows = if duplicate {
            vec![row.clone(), row]
        } else {
            vec![row]
        };
        let source_batch = batch_from_cells(&reg, source, &rows).unwrap();
        let bundles = BTreeMap::from([
            (target.key, target_batch.clone()),
            (source.key, source_batch),
        ]);
        assert_eq!(
            validate_bundle(&reg, &bundles).is_ok(),
            !duplicate && ordinal == 0 && rows[0][1] == id(1)
        );
    }
}

#[test]
fn nested_metadata_is_checked_even_when_every_parent_value_is_null() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let list = schema.field_with_name("list").unwrap();
    let child = match list.data_type() {
        DataType::List(child) => Some(child),
        _ => None,
    }
    .expect("declared list child");
    let mut metadata = child.metadata().clone();
    metadata.insert("SERDE_ARROW:strategy".to_owned(), "unknown".to_owned());
    let bad = list
        .clone()
        .with_data_type(DataType::List(std::sync::Arc::new(
            child.as_ref().clone().with_metadata(metadata),
        )));
    assert!(validate_field(&reg, &bad).is_err());
    let mut metadata = child.metadata().clone();
    metadata.insert(
        pse_schema::arrow::KEY_EXTENSION_METADATA.to_owned(),
        "{\"v\":1,\"enum_id\":\"00000000000000000000000000000000\"}".to_owned(),
    );
    assert!(validate_field(&reg, &child.as_ref().clone().with_metadata(metadata)).is_err());
}

#[test]
fn per_row_quantity_presence_is_a_direct_cross_field_check() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl(
        "quantities",
        vec![
            ColumnSpec::new(
                "value",
                LogicalType::F64,
                true,
                pse_schema::model::ColumnRole::Measure,
                "value",
            )
            .with_per_row_quantity(),
            ColumnSpec::reference("value_quantity_type_id", LogicalType::id(), "type").optional(),
        ],
    ));
    let reg = builder.build().unwrap();
    let spec = reg.relation("authored.quantities").unwrap();
    assert!(batch_from_cells(&reg, spec, &[vec![id(1), Cell::F64(5.0), Cell::Null]]).is_err());
    let batch = batch_from_cells(
        &reg,
        spec,
        &[
            vec![id(1), Cell::F64(5.0), id(2)],
            vec![id(2), Cell::Null, Cell::Null],
        ],
    )
    .unwrap();
    assert!(validate_batch(&reg, spec, &batch).is_ok());
}

#[test]
fn explicit_migration_admits_both_versions_and_preserves_values() {
    use pse_schema::model::{MigrationSpec, MigrationStep};
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl(
        "versioned",
        vec![ColumnSpec::label("old_name", LogicalType::Text, "label")],
    ));
    let mut target = decl(
        "versioned",
        vec![
            ColumnSpec::label("new_name", LogicalType::Text, "label"),
            ColumnSpec::payload("number", LogicalType::U64, "number"),
        ],
    );
    target.key.version = 2;
    builder.declare_relation(target);
    builder.declare_migration(MigrationSpec {
        relation: "authored.versioned",
        from_version: 1,
        to_version: 2,
        steps: vec![
            MigrationStep::RenameColumn {
                from: "old_name",
                to: "new_name",
            },
            MigrationStep::AddColumn {
                name: "number",
                default: Cell::U64(7),
            },
        ],
        doc: "add value",
    });
    let reg = builder.build().unwrap();
    let source = reg
        .relations()
        .iter()
        .find(|spec| spec.key.version == 1)
        .unwrap();
    let target = reg.relation("authored.versioned").unwrap();
    let batch = batch_from_cells(&reg, source, &[vec![id(1), Cell::text("kept")]]).unwrap();
    let migrated = pse_relations::migrate::migrate(&reg, &reg.migrations()[0], &batch).unwrap();
    let current = pse_relations::migrate::migrate_to_current(&reg, &batch).unwrap();
    assert_eq!(
        cells_from_batch(&reg, target, &current).unwrap(),
        cells_from_batch(&reg, target, &migrated).unwrap()
    );
    assert_eq!(
        cells_from_batch(&reg, target, &migrated).unwrap(),
        vec![vec![id(1), Cell::text("kept"), Cell::U64(7)]]
    );
    let mut undeclared = reg.migrations()[0].clone();
    undeclared.steps.clear();
    assert!(pse_relations::migrate::migrate(&reg, &undeclared, &batch).is_err());
    assert!(pse_relations::migrate::migrate(&reg, &reg.migrations()[0], &migrated).is_err());
}

#[test]
fn contextual_snapshot_metadata_requires_a_hash_and_producer_requires_an_identity() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let mut metadata = schema.metadata().clone();
    metadata.insert(
        "pse.snapshot_id".to_owned(),
        ContentHash::from_bytes([1; 32]).to_prefixed(),
    );
    metadata.insert(
        "pse.producer_pass_id".to_owned(),
        SemanticId::from_bytes([2; 16]).to_hex(),
    );
    assert!(validate_schema(&reg, spec, &schema.clone().with_metadata(metadata.clone())).is_ok());
    metadata.insert(
        "pse.snapshot_id".to_owned(),
        SemanticId::from_bytes([2; 16]).to_hex(),
    );
    assert!(validate_schema(&reg, spec, &schema.with_metadata(metadata)).is_err());
}

#[test]
fn formatter_renders_all_extensions_without_raw_storage_bytes() {
    use arrow::util::display::{ArrayFormatterFactory, FormatOptions};
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let batch = batch_from_cells(&reg, spec, &[row()]).unwrap();
    let schema = batch.schema();
    let factory = pse_relations::ext::PseFormatterFactory;
    let mut rendered = BTreeMap::new();
    for (field, array) in schema.fields().iter().zip(batch.columns()) {
        if let Some(formatter) = factory
            .create_array_formatter(array.as_ref(), &FormatOptions::default(), Some(field))
            .unwrap()
        {
            rendered.insert(
                field.name().clone(),
                formatter.value(0).try_to_string().unwrap(),
            );
        }
    }
    assert_eq!(rendered.len(), 11);
    assert_eq!(rendered["bound"], "finite(0.0)");
    assert_eq!(rendered["dimension"], "dimensionless");
    assert!(rendered["quantity"].contains("unit:id:"));
    assert!(rendered["id"].starts_with("id:"));
    assert!(rendered["hash"].starts_with("blake3:"));
    assert!(rendered["span"].ends_with(":2..5"));
}

#[test]
fn actual_registry_rows_materialize_under_their_own_contracts() {
    let reg = pse_schema::registry().unwrap();
    let batches = pse_relations::registry_relations::materialize(reg).unwrap();
    let expected = reg.schema_rows();
    assert_eq!(batches.len(), expected.len());
    for (key, rows) in expected {
        let spec = reg.relations().iter().find(|spec| spec.key == key).unwrap();
        assert_eq!(cells_from_batch(reg, spec, &batches[&key]).unwrap(), rows);
    }
}

#[test]
fn checked_rule_columns_validate_values_without_fabricating_relations() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let choice = schema.field_with_name("choice").unwrap();
    let array =
        pse_relations::cells::array_from_cells(&reg, choice, &[Cell::Enum("two"), Cell::Null])
            .unwrap();
    pse_relations::validate::validate_column(&reg, choice, array.as_ref()).unwrap();
    assert!(
        pse_relations::cells::array_from_cells(&reg, choice, &[Cell::Enum("invented")]).is_err()
    );
    let bound = schema.field_with_name("bound").unwrap();
    assert!(
        pse_relations::cells::array_from_cells(
            &reg,
            bound,
            &[Cell::Struct(vec![
                Cell::Enum("finite"),
                Cell::F64(f64::INFINITY)
            ])]
        )
        .is_err()
    );
    let identity = schema.field_with_name("id").unwrap();
    let array = pse_relations::cells::array_from_cells(&reg, identity, &[id(1)]).unwrap();
    let formatter = pse_relations::ext::create_owned_formatter(
        array.as_ref(),
        &arrow::util::display::FormatOptions::default(),
        identity.clone(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        formatter.value(0).try_to_string().unwrap(),
        format!("id:{}", "01".repeat(16))
    );
}

#[test]
fn owned_cell_construction_reserves_first_and_detached_children_keep_the_lease() {
    use pse_ids::{CancellationToken, FixedBudget};
    use pse_relations::cells::batch_from_cells_owned;
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let cancel = CancellationToken::default();
    let budget = FixedBudget::new(16 << 20);
    let rows = vec![row()];
    let batch = batch_from_cells_owned(&reg, spec, &rows, budget.as_ref(), &cancel).unwrap();
    assert_eq!(cells_from_batch(&reg, spec, &batch).unwrap(), rows);
    let size = budget.reserved();
    assert!(size > 0);
    let expected = batch_from_cells(&reg, spec, &rows).unwrap();
    assert_eq!(
        size,
        pse_ids::owned_buffer::retained_buffer_bytes(&expected).unwrap()
    );
    drop(expected);
    let detached = batch
        .column_by_name("fixed")
        .unwrap()
        .to_data()
        .child_data()[0]
        .buffers()[0]
        .clone();
    let copy = batch.clone();
    assert_eq!(budget.reserved(), size);
    drop(batch);
    drop(copy);
    assert_eq!(budget.reserved(), size);
    drop(detached);
    assert_eq!(budget.reserved(), 0);
    let tiny = FixedBudget::new(1);
    assert!(matches!(
        batch_from_cells_owned(&reg, spec, &rows, tiny.as_ref(), &cancel),
        Err(pse_relations::RelationError::Canon(
            pse_ids::CanonError::Reservation(_)
        ))
    ));
    assert_eq!(tiny.reserved(), 0);
    cancel.cancel();
    assert!(matches!(
        batch_from_cells_owned(&reg, spec, &rows, budget.as_ref(), &cancel),
        Err(pse_relations::RelationError::Canon(
            pse_ids::CanonError::Cancelled
        ))
    ));
    assert_eq!(budget.reserved(), 0);
}
