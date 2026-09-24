// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated contracts with SQL checks execute under their actual native owner.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "native generated contract fixtures"
)]
use pse_ids::SemanticId;
use pse_relations::generated::reference::units;
use std::sync::Arc;

fn unit_row() -> units::Row {
    units::Row {
        unit_id: SemanticId::from_bytes([7; 16]),
        symbol: "m".to_owned(),
        name: "metre".to_owned(),
        dimension: std::array::from_fn(|index| {
            pse_relations::generated::extension_values::ExtensionDimensionVectorItem {
                num: i16::from(index == 0),
                den: 1,
            }
        }),
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
    pse_engine::validation::registry().unwrap();
    let mut invalid = unit_row();
    invalid.dimension[2].den = 0;
    let mut builder = units::Builder::new().expect("builder");
    builder.push(invalid).expect("representation append");
    assert!(builder.finish().is_err());
    let mut builder = units::Builder::new().expect("fresh builder");
    builder.push(unit_row()).expect("valid");
    let constructed = builder.finish().expect("batch");
    let batch = constructed.batch();
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
fn canonical_order_transfers_checked_values_and_retained_buffer_ownership() {
    let registry = pse_engine::validation::registry().expect("registry");
    let spec = units::spec(registry).expect("units declaration");
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(32 * 1024 * 1024));
    let mut first = unit_row();
    first.unit_id = SemanticId::from_bytes([1; 16]);
    first.offset_to_canonical = -0.0;
    let mut second = unit_row();
    second.unit_id = SemanticId::from_bytes([2; 16]);
    let mut builder = units::Builder::new().expect("builder");
    builder.push(second.clone()).expect("later key");
    builder.push(first.clone()).expect("earlier key");
    let input = builder.finish().expect("checked fields");
    let (ordered, identity) = input
        .canonicalize(
            registry,
            spec,
            &budget,
            pse_columnar::CanonicalizeOptions::default(),
        )
        .expect("actual canonical order");
    assert_eq!(identity.row_count, 2);
    assert!(identity.sorted.is_none());
    assert!(budget.reserved() > 0);
    ordered
        .check_declaration(registry, spec)
        .expect("same complete declaration");
    let view = units::View::from_checked(&ordered).expect("borrow without raw admission");
    assert_eq!(view.rows().expect("original values"), vec![first, second]);
    // Canonical identity normalizes floating zero in its hashing copy only.
    assert_eq!(
        view.offset_to_canonical_column().value(0).to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        units::View::from_checked(&input)
            .expect("original owner")
            .unit_id_column()
            .value(0),
        &[2; 16]
    );
    let retained = ordered.slice(0, 1).expect("retained sorted row");
    drop(ordered);
    assert!(budget.reserved() > 0);
    drop(retained);
    assert_eq!(budget.reserved(), 0);

    let duplicates =
        pse_relations::columnar::FieldCheckedBatch::concat(registry, spec, &[input.clone(), input])
            .expect("local fields permit duplicate keys");
    assert!(matches!(
        duplicates.canonicalize(
            registry,
            spec,
            &budget,
            pse_columnar::CanonicalizeOptions::default()
        ),
        Err(pse_relations::RelationError::Canon(
            pse_columnar::CanonError::DuplicateKey { .. }
        ))
    ));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn reserved_concat_refuses_before_allocation_and_keeps_its_claim_with_slices() {
    use pse_relations::columnar::FieldCheckedBatch;
    let registry = pse_engine::validation::registry().expect("registry");
    let spec = units::spec(registry).expect("units");
    let cancel = pse_columnar::CancellationToken::new();
    let mut builder = units::Builder::new().expect("builder");
    builder.push(unit_row()).expect("row");
    let input = builder.finish().expect("fields");
    let tiny: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    assert!(
        FieldCheckedBatch::concat_reserved(
            registry,
            spec,
            &[input.clone(), input.clone()],
            &tiny,
            &cancel
        )
        .is_err()
    );
    assert_eq!(tiny.reserved(), 0);
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(8 << 20));
    let output = FieldCheckedBatch::concat_reserved(
        registry,
        spec,
        &[input.clone(), input],
        &budget,
        &cancel,
    )
    .expect("accounted native concat");
    assert_eq!(output.batch().num_rows(), 2);
    assert!(budget.reserved() > 0);
    let slice = output.slice(1, 1).expect("view");
    drop(output);
    assert!(budget.reserved() > 0);
    drop(slice);
    assert_eq!(budget.reserved(), 0);
}

