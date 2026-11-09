// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated passive inspection crossings preserve metadata without qualified adoption.

use super::*;
use crate::{
    host_operations::engineering::inspection::{self as codec, types as pb},
    workflow::OperationSource,
};
use buffa::Message as _;

#[tokio::test]
async fn generated_inspection_original_fixture_inputs_preserve_presence_bits_and_passive_custody() {
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
    let runtime = crate::workflow::tests::runtime();
    let (package, mut execution, preparation) = inspection_tests::admitted(
        &runtime,
        "package p {def Root {var x:Scalar; var y:Scalar; eq total:x+y==4;} test view fixture {dof 0; route steady; procedure solve; fix root.x=2; lower root.y=0;} {child root:Root=Root();}}",
    ).await;
    execution
        .analysis
        .case
        .values
        .insert("root.x".into(), 999.0);
    let document = execution
        .inspection(
            &runtime.preparation_services(),
            OperationSource::of(&package),
            preparation,
            0,
            128,
        )
        .unwrap();
    let original = serde_json::to_value(&document).unwrap();
    let wire = codec::encode_modeling_inspection(document).unwrap();
    let bytes = wire.encode_to_vec();
    let world = std::sync::Arc::new(DescriptorPool::decode(codec::DESCRIPTORS).unwrap());
    let message = world
        .message_index("pse.engineering.v1.ModelingInspection")
        .unwrap();
    let independent = DynamicMessage::decode(world, message, &bytes).unwrap();
    assert!(matches!(
        independent.field_by_number(12),
        Some(Value::Message(_))
    ));
    assert!(matches!(
        independent.field_by_number(13),
        Some(Value::Message(_))
    ));
    let Some(Value::Message(fixture)) = independent.field_by_number(14) else {
        panic!("original fixture must be independently readable");
    };
    assert_eq!(
        fixture.field_by_number(3),
        None,
        "steady fixture has no integration"
    );
    let raw = pb::ModelingInspection::decode_from_slice(&bytes).unwrap();
    let fixture = raw.fixture.as_option().unwrap();
    let x = fixture
        .specifications
        .iter()
        .find(|row| row.path == "root.x")
        .unwrap();
    let y = fixture
        .specifications
        .iter()
        .find(|row| row.path == "root.y")
        .unwrap();
    assert_eq!(
        (x.value, x.fixed, x.lower, x.upper),
        (Some(2.0f64.to_bits()), Some(true), None, None)
    );
    assert_eq!(
        (y.value, y.fixed, y.lower, y.upper),
        (None, None, Some(0.0f64.to_bits()), None)
    );
    assert!(x.quantity_id.as_option().is_some() && x.unit_id.as_option().is_some());
    let native = codec::decode_modeling_inspection(raw).unwrap();
    assert_eq!(serde_json::to_value(&native).unwrap(), original);
    assert!(native.payload_owner().is_none());
}

#[tokio::test]
async fn generated_inspection_original_integration_retains_nonzero_clock_and_ordered_inputs() {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) = inspection_tests::admitted(
        &runtime,
        "package p {def Root {domain t:Time from 160{s} to 162{s}; discretize grid on t using integrated(elements=1,order=1); param u:Scalar=5; var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==u; eq initial:x[160{s}]==10{s}; let area:Time=integral(i in t | u);} test view fixture {dof 0; route integrated; procedure integrate; value root.x[160{s}]=9{s}; integrate samples(160{s},161{s},162{s}) relative(1e-6) normalized_absolute(1e-8) step(1e-4{s}) quadrature_relative(1e-6) quadrature_absolute(root.area=1e-7{s}); schedule root.u at(161{s}) values(2,4);} {child root:Root=Root();}}",
    ).await;
    let document = execution
        .inspection(
            &runtime.preparation_services(),
            OperationSource::of(&package),
            preparation,
            0,
            128,
        )
        .unwrap();
    let original = serde_json::to_value(&document).unwrap();
    let wire = codec::encode_modeling_inspection(document).unwrap();
    let raw = pb::ModelingInspection::decode_from_slice(&wire.encode_to_vec()).unwrap();
    let fixture = raw.fixture.as_option().unwrap();
    assert!(!fixture.initial_equations.is_empty());
    let integration = fixture.integration.as_option().unwrap();
    assert_eq!(
        (integration.lower, integration.upper),
        (160.0f64.to_bits(), 162.0f64.to_bits())
    );
    assert_eq!(integration.initial_step, 1e-4f64.to_bits());
    assert_eq!(
        integration.samples,
        [160.0f64, 161.0, 162.0].map(f64::to_bits)
    );
    assert_eq!(
        integration.quadratures[0].absolute_tolerance,
        1e-7f64.to_bits()
    );
    assert_eq!(
        integration
            .schedule_intervals
            .iter()
            .map(|row| (row.change_time, row.value))
            .collect::<Vec<_>>(),
        [
            (None, 2.0f64.to_bits()),
            (Some(161.0f64.to_bits()), 4.0f64.to_bits())
        ]
    );
    let native = codec::decode_modeling_inspection(raw).unwrap();
    assert_eq!(serde_json::to_value(&native).unwrap(), original);
    assert!(native.payload_owner().is_none());
}

#[tokio::test]
async fn generated_inspection_metadata_roundtrips_original_types_and_finite_domain_coordinates() {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) =
        inspection_tests::admitted(&runtime, inspection_tests::FINITE).await;
    let document = execution
        .inspection(
            &runtime.preparation_services(),
            OperationSource::of(&package),
            preparation,
            0,
            128,
        )
        .unwrap();
    assert!(document.payload_owner().is_some());
    let original = serde_json::to_value(&document).unwrap();
    let wire = codec::encode_modeling_inspection(document).unwrap();
    let encoded = wire.encode_to_vec();
    let raw = pb::ModelingInspection::decode_from_slice(&encoded).unwrap();
    // Protobuf maps preserve meaning, not their iteration/encoding order.
    assert_eq!(raw, wire);
    // Output direction describes the crossing. Its passive Data record can be
    // decoded without recreating the original native allocation or admission.
    let native = codec::decode_modeling_inspection(raw).unwrap();
    assert_eq!(serde_json::to_value(&native).unwrap(), original);
    assert!(
        native.payload_owner().is_none(),
        "metadata decoding cannot grant the original native allocation owner"
    );
}

#[tokio::test]
async fn generated_inspection_reads_passive_resolved_metadata_but_refuses_unqualified_native_adoption()
 {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) =
        inspection_tests::admitted(&runtime, inspection_tests::FINITE).await;
    let mut document = execution
        .inspection(
            &runtime.preparation_services(),
            OperationSource::of(&package),
            preparation,
            0,
            1,
        )
        .unwrap();
    let registry = &package.physical_context().quantities;
    let actual = pse_quantity::ResolvedPhysicalContract::named(
        registry.neutral_dimensionless().unwrap(),
        pse_quantity::IndexSet::new(),
        registry,
    )
    .unwrap();
    // This is a passive codec control. Only the actual physical owner produced
    // the contract; no record is promoted to a newly admitted contract by decoding.
    document.domains[0] = InspectionDomain {
        key: document.domains[0].key.clone(),
        carrier: pse_modeling::resolved::Carrier {
            kind: pse_modeling::resolved::CarrierKind::OrderedSet,
            element: pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Resolved(
                Box::new(actual),
            )),
            members: std::sync::Arc::from([]),
        },
    };
    let wire = codec::encode_modeling_inspection(document).unwrap();
    let encoded = wire.encode_to_vec();
    let raw = pb::ModelingInspection::decode_from_slice(&encoded).unwrap();
    assert_eq!(raw, wire);
    assert!(serde_json::to_value(&raw).unwrap().is_object());
    let error = codec::decode_modeling_inspection(raw).unwrap_err();
    assert!(
        error.to_string().contains("qualified"),
        "retain the original receipt-adoption refusal: {error}"
    );
}
