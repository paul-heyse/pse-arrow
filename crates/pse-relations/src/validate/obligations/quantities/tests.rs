// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(clippy::unwrap_used, reason = "physical closure admission assertions")]
use super::super::{RelationInputs, quantity_requests};
use super::*;
use crate::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::{QuantityAdditionKind, ReferenceStateKind, ScaleKind},
        extension_values::{ExtensionDimensionVectorItem, QuantityValue},
        reference::{
            quantity_kinds, quantity_types, reference_states,
            units::{self, ReferenceUnitsFieldDefinitionItem},
        },
    },
    native::{
        arrow::array::RecordBatch, execution::context::SessionContext, physical_plan::collect,
    },
};
use pse_ids::SemanticId;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, ExtensionUse, FieldContract, Namespace, RelationDecl, RelationKey, SnapshotClass,
    },
};
use std::collections::BTreeMap;

fn id(value: u8) -> SemanticId {
    if value == 13 || value == 14 {
        return pse_quantity::unit_product_id(&[pse_quantity::UnitFactor {
            unit: pse_quantity::UnitId::from_id(id(if value == 13 { 11 } else { 15 })),
            exponent: pse_quantity::Ratio::from_parts(2, 1).unwrap(),
        }])
        .as_id();
    }
    SemanticId::from_bytes([value; 16])
}
fn dimension(power: i16) -> [ExtensionDimensionVectorItem; 8] {
    std::array::from_fn(|index| ExtensionDimensionVectorItem {
        num: if index == 0 { power } else { 0 },
        den: 1,
    })
}
fn unit(identity: u8, power: i16, reference: Option<u8>) -> units::Row {
    units::Row {
        unit_id: id(identity),
        symbol: format!("u{identity}"),
        name: format!("unit{identity}"),
        dimension: Some(dimension(power)),
        scale_to_canonical: Some(if identity == 15 { 0.01 } else { 1.0 }),
        offset_to_canonical: Some(0.0),
        is_affine: Some(false),
        reference_state_id: reference.map(id),
        definition: None,
        system: "test".into(),
        doc: "fixture".into(),
    }
}
fn defined(identity: u8, factor: u8) -> units::Row {
    units::Row {
        dimension: None,
        scale_to_canonical: None,
        offset_to_canonical: None,
        is_affine: None,
        definition: Some(vec![ReferenceUnitsFieldDefinitionItem {
            unit_id: id(factor),
            num: 2,
            den: 1,
        }]),
        ..unit(identity, 0, None)
    }
}
fn quantity(identity: u8, canonical: u8, reference: Option<u8>) -> quantity_types::Row {
    quantity_types::Row {
        name: None,
        quantity_type_id: id(identity),
        quantity_kind_id: id(if identity == 26 { 51 } else { 50 }),
        basis_id: None,
        reference_state_id: reference.map(id),
        scale_kind: ScaleKind::Point,
        shape: vec![],
        subject_kind: None,
        canonical_unit_id: id(canonical),
        nominal_magnitude: None,
        doc: "fixture".into(),
    }
}
#[derive(Default)]
struct Definitions {
    inputs: RelationInputs,
    batches: BTreeMap<RelationKey, FieldCheckedBatch>,
}
fn bind(context: &SessionContext, defs: &mut Definitions, name: &str, batch: RecordBatch) {
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation(name).unwrap();
    let checked = FieldCheckedBatch::admit(
        registry,
        spec,
        batch.clone(),
        &crate::validate::ValidationContext::new(registry, SessionContext::new().state()),
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    defs.batches.insert(spec.key, checked);
    defs.inputs.insert(
        spec.id,
        context.read_batch(batch).unwrap().into_unoptimized_plan(),
    );
}
fn definitions(context: &SessionContext) -> Definitions {
    let mut defs = Definitions::default();
    let mut rows = units::Builder::new(&crate::validate::ValidationContext::new(
        pse_schema::registry().unwrap(),
        SessionContext::new().state(),
    ))
    .unwrap();
    for row in [
        unit(10, 0, None),
        unit(11, 1, None),
        unit(12, 0, Some(60)),
        unit(15, 1, None),
        defined(13, 11),
        defined(14, 15),
    ] {
        rows.push(row).unwrap();
    }
    bind(
        context,
        &mut defs,
        "reference.units",
        rows.finish().unwrap().into_batch(),
    );
    let mut rows = quantity_kinds::Builder::new(&crate::validate::ValidationContext::new(
        pse_schema::registry().unwrap(),
        SessionContext::new().state(),
    ))
    .unwrap();
    for (identity, power) in [(50, 0), (51, 2)] {
        rows.push(quantity_kinds::Row {
            quantity_kind_id: id(identity),
            name: format!("kind{identity}"),
            dimension: Some(dimension(power)),
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            category: None,
            definition: None,
            doc: "fixture".into(),
        })
        .unwrap();
    }
    bind(
        context,
        &mut defs,
        "reference.quantity_kinds",
        rows.finish().unwrap().into_batch(),
    );
    let mut rows = reference_states::Builder::new(&crate::validate::ValidationContext::new(
        pse_schema::registry().unwrap(),
        SessionContext::new().state(),
    ))
    .unwrap();
    for identity in [60, 61] {
        rows.push(reference_states::Row {
            reference_state_id: id(identity),
            name: format!("reference{identity}"),
            kind: ReferenceStateKind::Custom,
            temperature: None,
            pressure: None,
            include_enthalpy_of_formation: false,
            subject_id: None,
            doc: "fixture".into(),
        })
        .unwrap();
    }
    bind(
        context,
        &mut defs,
        "reference.reference_states",
        rows.finish().unwrap().into_batch(),
    );
    let mut rows = quantity_types::Builder::new(&crate::validate::ValidationContext::new(
        pse_schema::registry().unwrap(),
        SessionContext::new().state(),
    ))
    .unwrap();
    for row in [
        quantity(20, 10, None),
        quantity(21, 10, Some(60)),
        quantity(22, 10, Some(61)),
        quantity(26, 13, None),
    ] {
        rows.push(row).unwrap();
    }
    bind(
        context,
        &mut defs,
        "reference.quantity_types",
        rows.finish().unwrap().into_batch(),
    );
    defs
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
        schema.clone(),
        vec![
            crate::columnar::array_from_values(schema.field(0), &[id(1)]).unwrap(),
            crate::columnar::array_from_values(schema.field(1), &[value]).unwrap(),
        ],
    )
    .unwrap();
    context.read_batch(batch).unwrap().into_unoptimized_plan()
}
async fn compatibility(
    context: &SessionContext,
    input: &LogicalPlan,
    defs: &Definitions,
) -> Result<QuantityCompatibility> {
    let mut inputs = defs.inputs.clone();
    inputs.insert(id(100), input.clone());
    let request = quantity_requests(&inputs)?.unwrap();
    let state = context.state();
    let requests = collect(
        state.create_physical_plan(&request).await?,
        state.task_ctx(),
    )
    .await?;
    QuantityCompatibility::admit(
        pse_schema::registry().unwrap(),
        &inputs,
        &defs.batches,
        &requests,
        &pse_columnar::CancellationToken::new(),
    )
}
async fn violations(context: &SessionContext, input: &LogicalPlan, defs: &Definitions) -> usize {
    let admitted = compatibility(context, input, defs).await.unwrap();
    let checks = plans(input, Some(&admitted)).unwrap();
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
fn value(quantity: u8, unit: u8) -> Option<Vec<Option<QuantityValue>>> {
    Some(vec![Some(QuantityValue {
        value: 3.0,
        quantity_type_id: id(quantity),
        unit_id: id(unit),
    })])
}
#[tokio::test]
async fn admitted_quantity_pairs_use_defined_unit_algebra_and_retain_datum() {
    let context = SessionContext::new();
    let defs = definitions(&context);
    for (quantity, unit, valid) in [
        (20, 10, true),
        (20, 11, false),
        (20, 99, false),
        (99, 10, false),
        (20, 12, false),
        (21, 12, true),
        (22, 12, false),
        (26, 13, true),
        (26, 14, true),
        (26, 10, false),
        (20, 13, false),
    ] {
        assert_eq!(
            violations(&context, &input(&context, value(quantity, unit)), &defs).await == 0,
            valid,
            "quantity {quantity}, representation {unit}"
        );
    }
}
#[tokio::test]
async fn selected_quantity_absence_cannot_be_filled_from_other_native_tables() {
    let context = SessionContext::new();
    let _unselected = definitions(&context);
    let defs = Definitions::default();
    for absent in [None, Some(vec![]), Some(vec![None])] {
        assert_eq!(
            violations(&context, &input(&context, absent), &defs).await,
            0
        );
    }
    assert_eq!(
        violations(&context, &input(&context, value(20, 10)), &defs).await,
        1
    );
}
#[tokio::test]
async fn physical_projection_requires_complete_selected_definitions() {
    let context = SessionContext::new();
    let mut defs = definitions(&context);
    let spec = pse_schema::registry()
        .unwrap()
        .relation("reference.quantity_kinds")
        .unwrap();
    defs.inputs.remove(&spec.id);
    defs.batches.remove(&spec.key);
    assert!(
        compatibility(&context, &input(&context, value(26, 14)), &defs)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn physical_projection_refuses_bad_composite_identity_missing_factor_and_cycle() {
    let context = SessionContext::new();
    for variant in 0..3 {
        let mut defs = definitions(&context);
        let mut rows = units::Builder::new(&crate::validate::ValidationContext::new(
            pse_schema::registry().unwrap(),
            SessionContext::new().state(),
        ))
        .unwrap();
        for row in [
            unit(10, 0, None),
            unit(11, 1, None),
            unit(12, 0, Some(60)),
            unit(15, 1, None),
            defined(13, 11),
        ] {
            rows.push(row).unwrap();
        }
        let mut bad = defined(14, 15);
        match variant {
            0 => bad.unit_id = id(80),
            1 => bad.definition.as_mut().unwrap()[0].unit_id = id(81),
            _ => bad.definition.as_mut().unwrap()[0].unit_id = id(14),
        }
        rows.push(bad).unwrap();
        bind(
            &context,
            &mut defs,
            "reference.units",
            rows.finish().unwrap().into_batch(),
        );
        assert!(
            compatibility(&context, &input(&context, value(26, 14)), &defs)
                .await
                .is_err()
        );
    }
}
#[tokio::test]
async fn physical_compatibility_cannot_move_to_another_selection() {
    let context = SessionContext::new();
    let defs = definitions(&context);
    let admitted = compatibility(&context, &input(&context, value(26, 14)), &defs)
        .await
        .unwrap();
    assert!(admitted.require_selection(&RelationInputs::new()).is_err());
}
