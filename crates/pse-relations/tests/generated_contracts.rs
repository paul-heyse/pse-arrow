// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "generated boundary regression fixtures"
)]

//! Generated views and typed row boundaries exercise actual values.

use std::sync::Arc;

use pse_ids::{ContentHash, SemanticId};
use pse_relations::generated::authored::packages::{self, AuthoredPackagesFieldDependenciesItem};
use pse_relations::generated::enums::{IdPolicy, PackageKind};
use pse_relations::generated::reference::units;
use pse_relations::typed::CellCodec;
use pse_schema::model::Cell;

#[test]
fn generated_nested_rows_round_trip_through_serde_and_arrow_cells_and_typed_views() {
    let row = packages::Row {
        package_id: SemanticId::from_bytes([3; 16]),
        name: "nested".to_owned(),
        version: "1.0.0".to_owned(),
        kind: PackageKind::Model,
        id_policy: IdPolicy::Explicit,
        dependencies: vec![AuthoredPackagesFieldDependenciesItem {
            package_id: SemanticId::from_bytes([4; 16]),
            version_req: "=1.0.0".to_owned(),
        }],
        content_hash: ContentHash::NIL,
        doc: "fixture".to_owned(),
    };
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
    let mut builder = packages::Builder::with_capacity(1);
    builder.push(row.clone()).expect("push");
    let batch = builder.finish().expect("finish");
    let view = packages::View::try_from_batch(&batch).expect("admit");
    assert_eq!(view.rows().expect("rows"), vec![row]);
    assert!(std::ptr::eq(view.batch(), &raw const batch));
}

fn unit_row() -> units::Row {
    units::Row {
        unit_id: SemanticId::from_bytes([7; 16]),
        symbol: "m".to_owned(),
        name: "metre".to_owned(),
        dimension: CellCodec::from_cell(Cell::List(
            (0..8)
                .map(|index| Cell::Struct(vec![Cell::I64(i64::from(index == 0)), Cell::I64(1)]))
                .collect(),
        ))
        .expect("dimension"),
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state_id: None,
        system: "SI".to_owned(),
        doc: String::new(),
    }
}

#[test]
fn generated_builder_checks_actual_nested_values_and_view_checks_actual_fields() {
    let mut invalid = unit_row();
    invalid.dimension[2].den = 0;
    assert!(units::Builder::new().push(invalid).is_err());
    let mut builder = units::Builder::new();
    builder.push(unit_row()).expect("valid");
    let batch = builder.finish().expect("batch");
    let mut fields = batch
        .schema()
        .fields()
        .iter()
        .map(|field| field.as_ref().clone())
        .collect::<Vec<_>>();
    fields[1] = fields[1].clone().with_name("renamed_symbol");
    let forged = pse_relations::RecordBatch::try_new(
        Arc::new(pse_relations::Schema::new_with_metadata(
            fields,
            batch.schema().metadata().clone(),
        )),
        batch.columns().to_vec(),
    )
    .expect("Arrow accepts renamed field with copied fingerprint");
    assert!(units::View::try_from_batch(&forged).is_err());
}

#[test]
fn generated_enum_round_trip_preserves_names_and_rejects_unknown_members() {
    for value in pse_relations::generated::enums::EquationRole::ALL {
        assert_eq!(
            value
                .as_str()
                .parse::<pse_relations::generated::enums::EquationRole>()
                .expect("member"),
            value
        );
    }
    assert!(
        "MISCLASSIFIED"
            .parse::<pse_relations::generated::enums::EquationRole>()
            .is_err()
    );
}
