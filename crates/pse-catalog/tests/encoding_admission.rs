// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct registry contract and physical framing admission, independent of hash claims.
#![allow(clippy::unwrap_used, reason = "test fixtures fail directly")]

use datafusion::parquet;
use pse_catalog::{EncodingPolicy, RelationContract};
use pse_ids::{CancellationToken, Envelope, FixedBudget, SemanticId};
use pse_schema::Registry;
use pse_schema::builder::RegistryBuilder;
use pse_schema::model::{
    Authority, Cell, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
};

fn fixture() -> Registry {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("one", "one"), EnumMember::new("two", "two")],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "items",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "identity"),
            FieldContract::payload("choice", FieldContract::enumeration("Choice"), "choice"),
            FieldContract::payload(
                "text",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Utf8),
                "text",
            )
            .optional(),
        ]),
    );
    builder.build().unwrap()
}

#[test]
fn every_registered_contract_is_projected_from_its_actual_recursive_declaration() {
    let reg = pse_schema::registry().unwrap();
    for spec in reg.relations() {
        let contract = RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap();
        contract.validate_against_registry(reg, spec).unwrap();
        let mut forged = contract.clone();
        forged.name.push_str("_forged");
        assert!(forged.validate_against_registry(reg, spec).is_err());
        let mut forged = contract;
        forged.canonical.primary_key.clear();
        assert!(forged.validate_against_registry(reg, spec).is_err());
    }
}

#[test]
fn finished_ipc_roundtrip_preserves_semantics_and_retains_owned_aligned_backing() {
    let reg = fixture();
    let spec = reg.relation("authored.items").unwrap();
    let batch = pse_relations::cells::batch_from_cells(
        &reg,
        spec,
        &[
            vec![
                Cell::Id(SemanticId::from_bytes([1; 16])),
                Cell::Enum("two"),
                Cell::text("λ"),
            ],
            vec![
                Cell::Id(SemanticId::from_bytes([2; 16])),
                Cell::Enum("one"),
                Cell::Null,
            ],
        ],
    )
    .unwrap();
    let budget = FixedBudget::new(32 << 20);
    let cancel = CancellationToken::default();
    let encoding = pse_catalog::store::encode::ipc_file(&batch, budget.as_ref(), &cancel).unwrap();
    assert!(encoding.bytes.starts_with(b"ARROW1"));
    assert!(encoding.bytes.ends_with(b"ARROW1"));
    let decoded = pse_catalog::store::verify::ipc_file(
        &encoding.bytes,
        &reg,
        spec,
        budget.as_ref(),
        &cancel,
        Envelope::default(),
    )
    .unwrap();
    assert_eq!(
        pse_relations::cells::cells_from_batch(&reg, spec, &decoded).unwrap(),
        pse_relations::cells::cells_from_batch(&reg, spec, &batch).unwrap()
    );
    let buffer = decoded.column(0).to_data().buffers()[0].clone();
    drop(decoded);
    drop(encoding);
    assert!(budget.reserved() > 0);
    drop(buffer);
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn complete_registry_ipc_admission_counts_values_independently_of_shared_backing() {
    let reg = pse_schema::catalog::assemble().unwrap();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::default();
    for (key, rows) in reg.schema_rows() {
        let spec = reg.relation(&key.qualified_name()).unwrap();
        let batch = pse_relations::cells::batch_from_cells_owned(
            &reg,
            spec,
            &rows,
            budget.as_ref(),
            &cancel,
        )
        .unwrap();
        let encoded =
            pse_catalog::store::encode::ipc_file(&batch, budget.as_ref(), &cancel).unwrap();
        let decoded = pse_catalog::store::verify::ipc_file(
            &encoded.bytes,
            &reg,
            spec,
            budget.as_ref(),
            &cancel,
            Envelope::default(),
        )
        .unwrap();
        assert_eq!(
            pse_relations::cells::cells_from_batch(&reg, spec, &decoded).unwrap(),
            rows,
            "actual values of {key}"
        );
        drop(decoded);
        drop(encoded);
        drop(batch);
        assert_eq!(budget.reserved(), 0, "all owners of {key} were released");
    }
}

#[test]
fn truncated_and_schema_mislabeled_ipc_fail_without_hash_admission() {
    let reg = fixture();
    let spec = reg.relation("authored.items").unwrap();
    let batch = pse_relations::cells::batch_from_cells(&reg, spec, &[]).unwrap();
    let budget = FixedBudget::new(32 << 20);
    let cancel = CancellationToken::default();
    let encoded = pse_catalog::store::encode::ipc_file(&batch, budget.as_ref(), &cancel).unwrap();
    for size in [0, 5, 10, encoded.bytes.len() - 1] {
        assert!(
            pse_catalog::store::verify::ipc_file(
                &encoded.bytes[..size],
                &reg,
                spec,
                budget.as_ref(),
                &cancel,
                Envelope::default()
            )
            .is_err()
        );
    }
    let mut forged = spec.clone();
    forged.columns[1] = forged.columns[1].clone().optional();
    assert!(
        pse_catalog::store::verify::ipc_file(
            &encoded.bytes,
            &reg,
            &forged,
            budget.as_ref(),
            &cancel,
            Envelope::default()
        )
        .is_err()
    );
    let tiny = FixedBudget::new(1);
    assert!(
        pse_catalog::store::verify::ipc_file(
            &encoded.bytes,
            &reg,
            spec,
            tiny.as_ref(),
            &cancel,
            Envelope::default()
        )
        .is_err()
    );
    assert_eq!(tiny.reserved(), 0);
}

#[test]
fn uncompressed_plain_parquet_roundtrip_preserves_declared_dictionary_and_nulls() {
    let reg = fixture();
    let spec = reg.relation("authored.items").unwrap();
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::default();
    for rows in [
        Vec::new(),
        vec![
            vec![
                Cell::Id(SemanticId::from_bytes([1; 16])),
                Cell::Enum("two"),
                Cell::text("λ"),
            ],
            vec![
                Cell::Id(SemanticId::from_bytes([2; 16])),
                Cell::Enum("one"),
                Cell::Null,
            ],
        ],
    ] {
        let batch = pse_relations::cells::batch_from_cells(&reg, spec, &rows).unwrap();
        let encoded =
            pse_catalog::store::encode::parquet_file(&batch, budget.as_ref(), &cancel).unwrap();
        let decoded = pse_catalog::store::verify::parquet_file(
            &encoded.bytes,
            &reg,
            spec,
            budget.as_ref(),
            &cancel,
            Envelope::default(),
        )
        .unwrap();
        assert_eq!(
            pse_relations::cells::cells_from_batch(&reg, spec, &decoded).unwrap(),
            rows
        );
    }
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn parquet_rejects_huge_footer_counts_before_pinned_parser_allocation() {
    let reg = fixture();
    let spec = reg.relation("authored.items").unwrap();
    let budget = FixedBudget::new(1 << 20);
    for footer in [
        vec![0x29, 0xfc, 0xff, 0xff, 0xff, 0xff, 7, 0],
        vec![0x29, 0x1c, 0x55, 0xfe, 1, 0, 0],
    ] {
        let mut bytes = b"PAR1".to_vec();
        bytes.extend_from_slice(&footer);
        bytes.extend_from_slice(&u32::try_from(footer.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(b"PAR1");
        assert!(
            pse_catalog::store::verify::parquet_file(
                &bytes.into(),
                &reg,
                spec,
                budget.as_ref(),
                &CancellationToken::default(),
                Envelope::default()
            )
            .is_err()
        );
        assert_eq!(budget.reserved(), 0);
    }
}

#[test]
fn parquet_rejects_declared_compression_and_inconsistent_page_value_counts() {
    use datafusion::parquet::file::reader::FileReader;
    let reg = fixture();
    let spec = reg.relation("authored.items").unwrap();
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::default();
    let batch = pse_relations::cells::batch_from_cells(
        &reg,
        spec,
        &[vec![
            Cell::Id(SemanticId::from_bytes([1; 16])),
            Cell::Enum("two"),
            Cell::Null,
        ]],
    )
    .unwrap();
    let encoded =
        pse_catalog::store::encode::parquet_file(&batch, budget.as_ref(), &cancel).unwrap();
    let reader =
        parquet::file::serialized_reader::SerializedFileReader::new(encoded.bytes.clone()).unwrap();
    let metadata = reader.metadata();
    let end = encoded.bytes.len() - 8;
    let footer_len = u32::from_le_bytes(encoded.bytes[end..end + 4].try_into().unwrap());
    let data_end = end - usize::try_from(footer_len).unwrap();
    let baseline = budget.reserved();
    for compression in [true, false] {
        let mut groups = metadata.row_groups().to_vec();
        let mut group = groups.remove(0).into_builder();
        let mut columns = group.take_columns();
        let column = columns.remove(0).into_builder();
        let column = if compression {
            column.set_compression(parquet::basic::Compression::SNAPPY)
        } else {
            column.set_num_values(i64::MAX)
        }
        .build()
        .unwrap();
        columns.insert(0, column);
        groups.insert(0, group.set_column_metadata(columns).build().unwrap());
        let changed = metadata
            .clone()
            .into_builder()
            .set_row_groups(groups)
            .build();
        let mut bytes = encoded.bytes[..data_end].to_vec();
        parquet::file::metadata::ParquetMetaDataWriter::new(&mut bytes, &changed)
            .finish()
            .unwrap();
        assert!(
            pse_catalog::store::verify::parquet_file(
                &bytes.into(),
                &reg,
                spec,
                budget.as_ref(),
                &cancel,
                Envelope::default()
            )
            .is_err()
        );
        assert_eq!(budget.reserved(), baseline);
    }
}
