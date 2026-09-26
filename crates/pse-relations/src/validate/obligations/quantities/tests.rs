// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(clippy::unwrap_used, reason = "native quantity-admission assertions")]

use super::*;
use crate::generated::{
    enums::{DomainKind, ScaleKind},
    extension_values::{ExtensionDimensionVectorItem, QuantityValue},
    reference::{quantity_types, units},
};
use crate::native::{
    arrow::array::RecordBatch, execution::context::SessionContext, physical_plan::collect,
};
use pse_ids::SemanticId;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, ExtensionUse, FieldContract, Namespace, RelationDecl, SnapshotClass},
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

fn unit(identity: u8, length: bool, reference: Option<u8>) -> units::Row {
    units::Row {
        unit_id: id(identity),
        symbol: "test".into(),
        name: "test".into(),
        dimension: std::array::from_fn(|ordinal| ExtensionDimensionVectorItem {
            num: i16::from(length && ordinal == 0),
            den: 1,
        }),
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state_id: reference.map(id),
        system: "test".into(),
        doc: "fixture".into(),
    }
}
fn quantity(
    identity: u8,
    canonical: u8,
    reference: Option<u8>,
    indexed: bool,
) -> quantity_types::Row {
    quantity_types::Row {
        quantity_type_id: id(identity),
        quantity_kind_id: id(50),
        basis_id: None,
        reference_state_id: reference.map(id),
        scale_kind: ScaleKind::Point,
        shape: if indexed {
            vec![DomainKind::Time]
        } else {
            vec![]
        },
        subject_kind: None,
        canonical_unit_id: id(canonical),
        nominal_magnitude: None,
        doc: "fixture".into(),
    }
}
fn bind(
    context: &SessionContext,
    record: &mut super::super::RelationInputs,
    name: &str,
    batch: RecordBatch,
) {
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation(name).unwrap();
    record.insert(
        spec.id,
        context.read_batch(batch).unwrap().into_unoptimized_plan(),
    );
}
fn definitions(context: &SessionContext) -> super::super::RelationInputs {
    crate::validate::ValidationContext::install_default(
        pse_schema::registry().unwrap(),
        context.state(),
    )
    .unwrap();
    let mut record = super::super::RelationInputs::new();
    let mut units = units::Builder::new().unwrap();
    for row in [
        unit(10, false, None),
        unit(11, true, None),
        unit(12, false, Some(60)),
    ] {
        units.push(row).unwrap();
    }
    bind(
        context,
        &mut record,
        "reference.units",
        units.finish().unwrap().into_batch(),
    );
    let mut types = quantity_types::Builder::new().unwrap();
    for row in [
        quantity(20, 10, None, false),
        quantity(21, 10, Some(60), false),
        quantity(22, 10, Some(61), false),
        quantity(23, 99, None, false),
        quantity(24, 10, None, true),
        quantity(25, 12, Some(61), false),
    ] {
        types.push(row).unwrap();
    }
    bind(
        context,
        &mut record,
        "reference.quantity_types",
        types.finish().unwrap().into_batch(),
    );
    record
}
fn input(context: &SessionContext, value: Option<Vec<Option<QuantityValue>>>) -> LogicalPlan {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "quantities",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested typed quantity input.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "identity"),
            FieldContract::list(FieldContract::extended(ExtensionUse::QuantityValue).optional())
                .with_name("values")
                .optional(),
        ]),
    );
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.quantities").unwrap();
    let schema = std::sync::Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
    let batch = RecordBatch::try_new(
        std::sync::Arc::clone(&schema),
        vec![
            crate::columnar::array_from_values(schema.field(0), &[id(1)]).unwrap(),
            crate::columnar::array_from_values(schema.field(1), &[value]).unwrap(),
        ],
    )
    .unwrap();
    context.read_batch(batch).unwrap().into_unoptimized_plan()
}
async fn violations(
    context: &SessionContext,
    input: &LogicalPlan,
    record: &super::super::RelationInputs,
) -> usize {
    let checks = plans(input, record, pse_schema::registry().unwrap()).unwrap();
    assert_eq!(checks.len(), 1);
    let state = context.state();
    collect(
        state.create_physical_plan(&checks[0]).await.unwrap(),
        state.task_ctx(),
    )
    .await
    .unwrap()
    .iter()
    .map(RecordBatch::num_rows)
    .sum()
}

#[tokio::test]
async fn nested_quantities_resolve_exact_types_units_dimensions_and_reference_states() {
    let context = SessionContext::new();
    let record = definitions(&context);
    for (quantity_type, unit, valid) in [
        (20, 10, true),
        (20, 11, false),
        (20, 99, false),
        (99, 10, false),
        (20, 12, false),
        (21, 12, true),
        (22, 12, false),
        (23, 10, false),
        (24, 10, false),
        (25, 10, false),
    ] {
        let input = input(
            &context,
            Some(vec![Some(QuantityValue {
                value: 3.0,
                quantity_type_id: id(quantity_type),
                unit_id: id(unit),
            })]),
        );
        assert_eq!(
            violations(&context, &input, &record).await == 0,
            valid,
            "quantity {quantity_type} and unit {unit}"
        );
    }
}

#[tokio::test]
async fn missing_selected_definitions_reject_visible_values_but_not_absence() {
    let context = SessionContext::new();
    let mut record = definitions(&context);
    record.clear();
    for value in [None, Some(vec![]), Some(vec![None])] {
        assert_eq!(
            violations(&context, &input(&context, value), &record).await,
            0
        );
    }
    let input = input(
        &context,
        Some(vec![Some(QuantityValue {
            value: 3.0,
            quantity_type_id: id(20),
            unit_id: id(10),
        })]),
    );
    assert_eq!(
        violations(&context, &input, &record).await,
        1,
        "native tables exist but are outside the selected publication"
    );
}
