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
use pse_relations::testing::{batch_from_literals, literals_from_batch};
use pse_relations::validate::{validate_batch, validate_field, validate_schema};
use pse_schema::Registry;
use pse_schema::builder::RegistryBuilder;
use pse_schema::model::{
    Authority, EnumDecl, EnumMember, ExtensionUse, FieldContract, Namespace, RelationDecl,
    SnapshotClass,
};
use std::collections::BTreeMap;

fn id(value: u8) -> serde_json::Value {
    serde_json::json!(["id", (SemanticId::from_bytes([value; 16])).to_hex()])
}
fn decl(name: &'static str, columns: Vec<FieldContract>) -> RelationDecl {
    let mut all = vec![FieldContract::key("id", FieldContract::id(), "identity")];
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

#[test]
fn dictionary_ordering_cannot_be_forged_with_an_unchanged_fingerprint() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl(
        "ordered",
        vec![FieldContract::payload(
            "value",
            FieldContract::native(DataType::Dictionary(
                Box::new(DataType::Int32),
                Box::new(DataType::Utf8),
            )),
            "native dictionary",
        )],
    ));
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.ordered").unwrap();
    let mut forged = spec.clone();
    forged.columns[1] =
        FieldContract::from_field(forged.columns[1].field().clone().with_dict_is_ordered(true));
    assert_eq!(forged.fingerprint, spec.fingerprint);
    assert!(pse_schema::arrow::relation_schema(&registry, &forged).is_err());
}
fn fixture() -> Registry {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "BoundKind",
        vec![
            EnumMember::new("finite", "finite"),
            EnumMember::new("unbounded", "unbounded"),
        ],
    ));
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("one", "one"), EnumMember::new("two", "two")],
    ));
    builder.declare_relation(decl("target", vec![]));
    let types = vec![
        ("i64", FieldContract::native(DataType::Int64)),
        ("i32", FieldContract::native(DataType::Int32)),
        ("u8", FieldContract::native(DataType::UInt8)),
        ("u16", FieldContract::native(DataType::UInt16)),
        ("u32", FieldContract::native(DataType::UInt32)),
        ("u64", FieldContract::native(DataType::UInt64)),
        ("bool", FieldContract::native(DataType::Boolean)),
        ("text", FieldContract::native(DataType::Utf8)),
        (
            "time",
            FieldContract::native(DataType::Timestamp(
                arrow_schema::TimeUnit::Nanosecond,
                Some("UTC".into()),
            )),
        ),
        ("float", FieldContract::native(DataType::Float64)),
        ("hash", FieldContract::extended(ExtensionUse::ContentHash)),
        (
            "choice",
            FieldContract::extended(ExtensionUse::Enum("Choice")),
        ),
        ("bound", FieldContract::extended(ExtensionUse::Bound)),
        (
            "dimension",
            FieldContract::extended(ExtensionUse::DimensionVector),
        ),
        (
            "quantity",
            FieldContract::extended(ExtensionUse::QuantityValue),
        ),
        ("index", FieldContract::extended(ExtensionUse::IndexTuple)),
        ("span", FieldContract::extended(ExtensionUse::SourceSpan)),
        ("dsl", FieldContract::extended(ExtensionUse::ExprDsl)),
        ("path", FieldContract::extended(ExtensionUse::TargetPath)),
        (
            "ordinal",
            FieldContract::extended(ExtensionUse::OrdinalRef {
                target: "authored.target",
            }),
        ),
        (
            "list",
            FieldContract::list(FieldContract::extended(ExtensionUse::Enum("Choice"))),
        ),
        ("fixed", FieldContract::fixed_list(FieldContract::id(), 2)),
        (
            "nested",
            FieldContract::structure(vec![
                FieldContract::extended(ExtensionUse::Enum("Choice"))
                    .with_name("choice")
                    .with_nullable(false),
                FieldContract::list(FieldContract::id())
                    .with_name("list")
                    .with_nullable(true),
            ]),
        ),
    ];
    builder.declare_relation(decl(
        "values",
        types
            .into_iter()
            .map(|(name, ty)| FieldContract::payload(name, ty, "value").optional())
            .collect(),
    ));
    builder.build().unwrap()
}
fn row() -> Vec<serde_json::Value> {
    vec![
        id(1),
        serde_json::json!(["i64", -9]),
        serde_json::json!(["i64", i64::from(i32::MIN)]),
        serde_json::json!(["u64", 255]),
        serde_json::json!(["u64", 65535]),
        serde_json::json!(["u64", u64::from(u32::MAX)]),
        serde_json::json!(["u64", u64::MAX]),
        serde_json::json!(["bool", true]),
        serde_json::json!(["text", "λ\n"]),
        serde_json::json!(["i64", -1000]),
        serde_json::json!(["f64", format!("{:016x}", f64::to_bits(-0.0))]),
        serde_json::json!(["hash", (ContentHash::from_bytes([7; 32])).to_hex()]),
        serde_json::json!(["enum", "two"]),
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["enum", "finite"]),
                serde_json::json!(["f64", format!("{:016x}", f64::to_bits(0.0))])
            ]
        ]),
        serde_json::json!([
            "list",
            (0..8)
                .map(|_| serde_json::json!([
                    "struct",
                    vec![serde_json::json!(["i64", 0]), serde_json::json!(["i64", 1])]
                ]))
                .collect::<Vec<serde_json::Value>>()
        ]),
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["f64", format!("{:016x}", f64::to_bits(1.5))]),
                id(4),
                id(5)
            ]
        ]),
        serde_json::json!(["list", vec![id(8), id(9)]]),
        serde_json::json!([
            "struct",
            vec![
                id(4),
                serde_json::json!(["i64", 2]),
                serde_json::json!(["i64", 5])
            ]
        ]),
        serde_json::json!(["text", "x + 1"]),
        serde_json::json!(["text", "fs.a.*"]),
        serde_json::json!(["i64", 0]),
        serde_json::json!([
            "list",
            vec![
                serde_json::json!(["enum", "two"]),
                serde_json::json!(["enum", "one"])
            ]
        ]),
        serde_json::json!(["list", vec![id(2), id(3)]]),
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["enum", "one"]),
                serde_json::json!(["list", vec![id(3)]])
            ]
        ]),
    ]
}

#[test]
fn every_declared_layout_round_trips_including_null_parents_and_slices() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let mut absent = vec![serde_json::json!(["null", null]); spec.columns.len()];
    absent[0] = id(2);
    let rows = vec![row(), absent];
    let batch = batch_from_literals(&reg, spec, &rows).unwrap();
    assert_eq!(literals_from_batch(&reg, spec, &batch).unwrap(), rows);
    assert_eq!(
        literals_from_batch(&reg, spec, &batch.slice(1, 1)).unwrap(),
        rows[1..]
    );
    let empty = batch_from_literals(&reg, spec, &[]).unwrap();
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
        ("choice", serde_json::json!(["enum", "invalid"])),
        ("u8", serde_json::json!(["u64", 256])),
        ("i32", serde_json::json!(["i64", i64::MAX])),
        (
            "bound",
            serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["enum", "finite"]),
                    serde_json::json!(["f64", format!("{:016x}", (f64::INFINITY).to_bits())])
                ]
            ]),
        ),
        (
            "bound",
            serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["enum", "unbounded"]),
                    serde_json::json!(["f64", format!("{:016x}", f64::to_bits(0.0))])
                ]
            ]),
        ),
        (
            "span",
            serde_json::json!([
                "struct",
                vec![
                    id(1),
                    serde_json::json!(["i64", 8]),
                    serde_json::json!(["i64", 4])
                ]
            ]),
        ),
        (
            "quantity",
            serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["f64", format!("{:016x}", (f64::NAN).to_bits())]),
                    id(1),
                    id(2)
                ]
            ]),
        ),
        (
            "dimension",
            serde_json::json!([
                "list",
                (0..8)
                    .map(|_| serde_json::json!([
                        "struct",
                        vec![serde_json::json!(["i64", 0]), serde_json::json!(["i64", 2])]
                    ]))
                    .collect::<Vec<serde_json::Value>>()
            ]),
        ),
        ("fixed", serde_json::json!(["list", vec![id(1)]])),
    ];
    for (name, invalid) in mutations {
        let mut values = row();
        let index = spec
            .columns
            .iter()
            .position(|column| column.name() == name)
            .unwrap();
        values[index] = invalid;
        assert!(
            batch_from_literals(&reg, spec, &[values]).is_err(),
            "admitted invalid {name}"
        );
    }
}

#[test]
fn masked_struct_payload_does_not_leak_invalid_enum_strings() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let mut values = vec![serde_json::json!(["null", null]); spec.columns.len()];
    values[0] = id(1);
    // The builder physically stores empty enum strings and invalid dimension defaults
    // below absent parents. Admission must ignore those invisible bytes.
    let batch = batch_from_literals(&reg, spec, &[values.clone()]).unwrap();
    assert_eq!(
        literals_from_batch(&reg, spec, &batch).unwrap(),
        vec![values]
    );
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
fn a_quantity_carries_its_measure_quantity_and_unit_in_one_nullable_value() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl(
        "quantities",
        vec![
            FieldContract::payload(
                "measure",
                FieldContract::extended(ExtensionUse::QuantityValue),
                "value with quantity and unit identities",
            )
            .optional(),
        ],
    ));
    let reg = builder.build().unwrap();
    let spec = reg.relation("authored.quantities").unwrap();
    for value in [
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["f64", format!("{:016x}", f64::to_bits(5.0))]),
                id(2),
                serde_json::json!(["null", null])
            ]
        ]),
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["f64", format!("{:016x}", f64::to_bits(5.0))]),
                serde_json::json!(["null", null]),
                id(3)
            ]
        ]),
        serde_json::json!([
            "struct",
            vec![
                serde_json::json!(["f64", format!("{:016x}", (f64::NAN).to_bits())]),
                id(2),
                id(3)
            ]
        ]),
    ] {
        assert!(batch_from_literals(&reg, spec, &[vec![id(1), value]]).is_err());
    }
    let batch = batch_from_literals(
        &reg,
        spec,
        &[
            vec![
                id(1),
                serde_json::json!([
                    "struct",
                    vec![
                        serde_json::json!(["f64", format!("{:016x}", f64::to_bits(5.0))]),
                        id(2),
                        id(3)
                    ]
                ]),
            ],
            vec![id(2), serde_json::json!(["null", null])],
        ],
    )
    .unwrap();
    assert!(validate_batch(&reg, spec, &batch).is_ok());
}

#[test]
fn predecessor_contextual_metadata_is_not_part_of_the_declared_contract() {
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
    assert!(validate_schema(&reg, spec, &schema.clone().with_metadata(metadata.clone())).is_err());
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
    let batch = batch_from_literals(&reg, spec, &[row()]).unwrap();
    let schema = batch.schema();
    let factory = pse_relations::ext::PseFormatterFactory;
    let mut rendered = BTreeMap::new();
    for (field, array) in schema.fields().iter().zip(batch.columns()) {
        if let Some(formatter) = factory
            .create_array_formatter(array.as_ref(), &FormatOptions::default(), Some(field))
            .unwrap()
        {
            rendered.insert(
                field.name().to_owned(),
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
    let expected = reg.schema_batches();
    assert_eq!(batches.len(), expected.len());
    for (key, batch) in expected {
        assert_eq!(&batches[key], batch);
        assert!(std::sync::Arc::ptr_eq(
            batches[key].column(0),
            batch.column(0)
        ));
    }
}

#[test]
fn checked_rule_columns_validate_values_without_fabricating_relations() {
    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let schema = pse_schema::arrow::relation_schema(&reg, spec).unwrap();
    let choice = schema.field_with_name("choice").unwrap();
    let array = pse_relations::testing::array_from_literals(
        &reg,
        choice,
        &[
            serde_json::json!(["enum", "two"]),
            serde_json::json!(["null", null]),
        ],
    )
    .unwrap();
    pse_relations::validate::validate_column(&reg, choice, array.as_ref()).unwrap();
    assert!(
        pse_relations::testing::array_from_literals(
            &reg,
            choice,
            &[serde_json::json!(["enum", "invented"])]
        )
        .is_err()
    );
    let bound = schema.field_with_name("bound").unwrap();
    assert!(
        pse_relations::testing::array_from_literals(
            &reg,
            bound,
            &[serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["enum", "finite"]),
                    serde_json::json!(["f64", format!("{:016x}", (f64::INFINITY).to_bits())])
                ]
            ])]
        )
        .is_err()
    );
    let identity = schema.field_with_name("id").unwrap();
    let array = pse_relations::testing::array_from_literals(&reg, identity, &[id(1)]).unwrap();
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
fn owned_native_admission_retains_detached_children_and_checks_limits() {
    use pse_columnar::CancellationToken;

    let reg = fixture();
    let spec = reg.relation("authored.values").unwrap();
    let cancel = CancellationToken::default();
    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
    let rows = vec![row()];
    let construct = |pool: &std::sync::Arc<dyn pse_columnar::MemoryPool>,
                     cancel: &CancellationToken| {
        let batch = batch_from_literals(&reg, spec, &rows)?;
        pse_relations::columnar::FieldCheckedBatch::admit_external(&reg, spec, &batch, pool, cancel)
            .map(pse_relations::columnar::FieldCheckedBatch::into_batch)
    };
    let batch = construct(&budget, &cancel).unwrap();
    assert_eq!(literals_from_batch(&reg, spec, &batch).unwrap(), rows);
    let size = budget.reserved();
    assert!(size > 0);
    // Wrapper-visible capacity is a lower bound: imported ownership includes
    // the detached allocations that those wrappers keep alive.
    assert!(size >= pse_columnar::owned_buffer::retained_buffer_bytes(&batch).unwrap());
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
    assert!(budget.reserved() >= detached.len() && budget.reserved() < size);
    drop(detached);
    assert_eq!(budget.reserved(), 0);
    let tiny: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let failure = construct(&tiny, &cancel).unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&failure),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    assert_eq!(tiny.reserved(), 0);
    cancel.cancel();
    let failure = construct(&budget, &cancel).unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&failure),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
    assert_eq!(budget.reserved(), 0);
}
