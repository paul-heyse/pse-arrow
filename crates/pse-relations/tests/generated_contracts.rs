// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "generated boundary regression fixtures"
)]

//! Generated views and typed row boundaries exercise actual values.

use std::sync::Arc;

use arrow_array::Array;
use pse_ids::ContentHash;
use pse_relations::generated::authored::packages::{self, AuthoredPackagesFieldDependenciesItem};
use pse_relations::generated::enums::{IdPolicy, PackageKind};
use pse_relations::generated::identities::PackageId;
use pse_relations::generated::reference::units;

fn package_row() -> packages::Row {
    packages::Row {
        package_id: PackageId::from_bytes([3; 16]),
        name: "nested".to_owned(),
        version: "1.0.0".to_owned(),
        kind: PackageKind::Model,
        id_policy: IdPolicy::Explicit,
        dependencies: vec![AuthoredPackagesFieldDependenciesItem {
            package_id: PackageId::from_bytes([4; 16]),
            version_req: "=1.0.0".to_owned(),
        }],
        content_hash: ContentHash::NIL,
        doc: "fixture".to_owned(),
    }
}

#[test]
fn raw_string_enums_reject_unknown_members_and_old_dictionary_storage() {
    use arrow_array::{DictionaryArray, Int32Array, StringArray, types::Int32Type};
    let mut builder = packages::Builder::new().unwrap();
    builder.push(package_row()).unwrap();
    let checked = builder.finish().unwrap();
    let batch = checked.batch();
    let position = batch.schema().index_of("kind").unwrap();
    assert_eq!(
        batch.schema().field(position).data_type(),
        &arrow_schema::DataType::Utf8
    );
    let mut columns = batch.columns().to_vec();
    columns[position] = Arc::new(StringArray::from(vec!["not-a-member"]));
    let invalid = pse_relations::RecordBatch::try_new(batch.schema(), columns).unwrap();
    assert!(packages::validate(&invalid).is_err());

    let mut fields = batch.schema().fields().to_vec();
    let dictionary = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![0]),
        Arc::new(StringArray::from(vec!["model"])),
    )
    .unwrap();
    fields[position] = Arc::new(
        fields[position]
            .as_ref()
            .clone()
            .with_data_type(dictionary.data_type().clone()),
    );
    let schema = Arc::new(arrow_schema::Schema::new_with_metadata(
        fields,
        batch.schema().metadata().clone(),
    ));
    let mut columns = batch.columns().to_vec();
    columns[position] = Arc::new(dictionary);
    let old = pse_relations::RecordBatch::try_new(schema, columns).unwrap();
    assert!(packages::validate(&old).is_err());
}

#[test]
fn generated_nested_rows_round_trip_through_serde_and_direct_arrow_views() {
    let row = package_row();
    let encoded = serde_json::to_value(&row).expect("encode");
    assert_eq!(
        encoded["dependencies"][0]["package_id"],
        "04040404040404040404040404040404"
    );
    assert_eq!(
        serde_json::from_value::<packages::Row>(encoded.clone()).expect("decode"),
        row
    );
    let mut extra = encoded;
    extra["renamed_column"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<packages::Row>(extra).is_err());
    let mut builder = packages::Builder::with_capacity(1).expect("builder");
    builder.push(row.clone()).expect("push");
    let batch = builder.finish().expect("finish");
    let view = packages::View::from_checked(&batch).expect("borrow construction");
    assert_eq!(view.rows().expect("rows"), vec![row]);
    assert!(std::ptr::eq(view.batch(), batch.batch()));
    assert!(std::ptr::eq(
        view.dependencies_column(),
        batch
            .batch()
            .column(5)
            .as_any()
            .downcast_ref()
            .expect("list"),
    ));
    assert_eq!(view.name_column().value(0), "nested");
    assert_eq!(packages::COLUMNS[5].name, "dependencies");
    assert_eq!(packages::COLUMNS[5].relation_id, packages::RELATION_ID);
    assert_eq!(
        view.dependencies_field(),
        &packages::schema().expect("schema").fields()[5]
    );
    packages::validate(batch.batch()).expect("independent raw admission agrees");
}

#[test]
fn generated_enum_round_trip_preserves_names_and_rejects_unknown_members() {
    for value in IdPolicy::ALL {
        assert_eq!(value.as_str().parse::<IdPolicy>().expect("member"), value);
    }
    assert!("MISCLASSIFIED".parse::<IdPolicy>().is_err());
}

#[test]
fn bulk_append_preserves_nested_slices_and_declared_enum_strings() {
    let mut first = packages::Builder::new().expect("first");
    let first_row = package_row();
    first.push(first_row.clone()).expect("push");
    let first = first.finish().expect("finish");

    let mut second = packages::Builder::new().expect("second");
    let mut skipped = package_row();
    skipped.name = "skipped".to_owned();
    second.push(skipped).expect("skip row");
    let mut second_row = package_row();
    second_row.name = "second".to_owned();
    second_row.package_id = PackageId::from_bytes([9; 16]);
    second_row.kind = PackageKind::Library;
    second_row.dependencies[0].version_req = "^2.0".to_owned();
    second.push(second_row.clone()).expect("second row");
    let second = second.finish().expect("finish");
    let slice = second.batch().slice(1, 1);
    // The canonical string enum column retains its slice alongside nested children.
    let second_view = packages::View::try_from_batch(&slice).expect("candidate admission");
    assert_eq!(second_view.dependencies_column().value_offsets()[0], 1);

    let mut combined = packages::Builder::new().expect("combined");
    combined
        .append_view(&packages::View::from_checked(&first).expect("first view"))
        .expect("bulk first");
    combined.append_view(&second_view).expect("bulk slice");
    combined.push(first_row.clone()).expect("append after bulk");
    let combined = combined.finish().expect("combined");
    let view = packages::View::from_checked(&combined).expect("combined view");
    assert_eq!(
        view.rows().expect("direct rows"),
        vec![first_row.clone(), second_row, first_row]
    );
    assert_eq!(
        view.dependencies_column().data_type(),
        packages::schema().expect("schema").field(5).data_type()
    );
    packages::validate(combined.batch()).expect("complete raw field validation");
}

#[test]
fn sole_bulk_chunk_keeps_the_original_arrow_buffer_owners() {
    let mut source = packages::Builder::new().expect("source");
    source.push(package_row()).expect("push");
    let source = source.finish().expect("finish");
    let mut target = packages::Builder::new().expect("target");
    target
        .append_view(&packages::View::from_checked(&source).expect("view"))
        .expect("bulk");
    let target = target.finish().expect("finish");
    for (left, right) in source
        .batch()
        .columns()
        .iter()
        .zip(target.batch().columns())
    {
        assert!(Arc::ptr_eq(left, right));
    }
}

#[test]
fn constructed_scope_checks_relation_identity_and_does_not_claim_key_validity() {
    let mut builder = packages::Builder::new().expect("builder");
    builder.push(package_row()).expect("first candidate");
    builder
        .push(package_row())
        .expect("duplicate remains a relational obligation");
    let constructed = builder.finish().expect("local construction");
    assert_eq!(
        packages::View::from_checked(&constructed)
            .expect("matching local contract")
            .len(),
        2
    );
    assert!(units::View::from_checked(&constructed).is_err());
}

#[test]
fn raw_admission_concat_and_slices_share_one_private_field_capability() {
    let registry = pse_schema::registry().expect("registry");
    let spec = packages::spec(registry).expect("declaration");
    let mut builder = packages::Builder::new().expect("builder");
    builder.push(package_row()).expect("push");
    let raw = builder.finish().expect("finish").into_batch();
    let admitted = pse_relations::columnar::FieldCheckedBatch::admit(registry, spec, raw)
        .expect("raw admission");
    let slice = admitted.slice(0, 1).expect("checked slice");
    assert!(admitted.slice(1, 1).is_err());
    let combined =
        pse_relations::columnar::FieldCheckedBatch::concat(registry, spec, &[admitted, slice])
            .expect("typed concat");
    assert_eq!(
        packages::View::from_checked(&combined)
            .expect("borrow checked")
            .rows()
            .expect("rows"),
        vec![package_row(), package_row()]
    );
    let empty = pse_relations::columnar::FieldCheckedBatch::concat(registry, spec, &[])
        .expect("empty relation");
    assert!(
        packages::View::from_checked(&empty)
            .expect("typed empty")
            .is_empty()
    );

    let mut impostor = spec.clone();
    impostor.columns[1] = impostor.columns[1]
        .clone()
        .with_nullable(!impostor.columns[1].nullable());
    assert_eq!(impostor.fingerprint, spec.fingerprint);
    combined
        .check_declaration(registry, spec)
        .expect("actual declaration");
    assert!(combined.check_declaration(registry, &impostor).is_err());
    assert!(
        pse_relations::columnar::FieldCheckedBatch::admit(
            registry,
            &impostor,
            combined.batch().clone()
        )
        .is_err()
    );
    assert!(
        pse_relations::columnar::FieldCheckedBatch::concat(
            registry,
            units::spec(registry).expect("units"),
            &[combined]
        )
        .is_err()
    );
}
