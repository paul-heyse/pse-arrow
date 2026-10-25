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
        unit_id: SemanticId::from_bytes([7; 16]).into(),
        symbol: "m".to_owned(),
        name: "metre".to_owned(),
        dimension: Some(std::array::from_fn(|index| {
            pse_relations::generated::extension_values::ExtensionDimensionVectorItem {
                num: i16::from(index == 0),
                den: 1,
            }
        })),
        scale_to_canonical: Some(1.0),
        offset_to_canonical: Some(0.0),
        is_affine: Some(false),
        reference_state_id: None,
        definition: None,
        system: "SI".to_owned(),
        doc: String::new(),
    }
}

#[test]
fn generated_builder_checks_actual_nested_values_and_view_checks_actual_fields() {
    let registry = pse_schema::registry().unwrap();
    let validation = fixture_validation(registry);
    let mut invalid = unit_row();
    invalid.dimension.as_mut().expect("atomic unit")[2].den = 0;
    let mut builder = units::Builder::new(&validation).expect("builder");
    builder.push(invalid).expect("representation append");
    assert!(builder.finish().is_err());
    let mut builder = units::Builder::new(&validation).expect("fresh builder");
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
    assert!(units::View::try_from_batch(&forged, &validation).is_err());
}

#[test]
fn reserved_concat_refuses_before_allocation_and_keeps_its_claim_with_slices() {
    use pse_relations::columnar::FieldCheckedBatch;
    let registry = pse_schema::registry().expect("registry");
    let validation = fixture_validation(registry);
    let spec = units::spec(registry).expect("units");
    let cancel = pse_columnar::CancellationToken::new();
    let mut builder = units::Builder::new(&validation).expect("builder");
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

// Deliberate native context for these isolated generated-contract fixtures.
fn fixture_validation(
    registry: &pse_schema::Registry,
) -> Arc<pse_relations::validate::ValidationContext> {
    Arc::new(pse_relations::validate::ValidationContext::new(
        registry,
        pse_engine::validation::NativeValidation(
            datafusion::prelude::SessionContext::new().state(),
        ),
    ))
}
