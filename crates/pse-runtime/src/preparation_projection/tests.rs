// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::host_operations::engineering::generated::pb::pse::engineering::v1 as wire;
use buffa::Message;
use pse_contract::boundary_projection::ProjectionRecord;
use pse_modeling::specialize::Value;

fn request() -> crate::workflow::PreparationDiscoveryRequest {
    let root = pse_modeling::DeclarationId::from_bytes([0x31; 16]);
    let mut bindings = pse_modeling::Bindings::default();
    bindings.arguments.insert(
        "values".into(),
        Value::Tuple(vec![
            Value::Missing,
            Value::Set(vec![Value::Integer(7)]),
            Value::Row {
                table: root,
                names: vec!["z".into(), "a".into()].into(),
                fields: vec![Value::Text("first".into()), Value::Text("second".into())].into(),
            },
        ]),
    );
    bindings.facts.insert(
        pse_modeling::analysis::Fact::Stage("warm".into()),
        Value::Boolean(true),
    );
    crate::workflow::PreparationDiscoveryRequest {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings,
        limits: pse_modeling::Limits::default(),
        durable_after: None,
    }
}

#[test]
fn structural_number_wire_matches_independent_exact_bit_and_identity_bytes() {
    let native = Value::Number {
        bits: 0x8000_0000_0000_0000,
        quantity: pse_quantity::QuantityTypeId::from_bytes([0x32; 16]),
    };
    let projected = encode_structural_value(native.clone()).unwrap();
    // Value.Number = field 3; Number.bits = fixed64 field 1; quantity = field 2;
    // its explicit 16-byte identity is field 1 of the nominal QuantityTypeId.
    let mut expected = vec![
        0x1a, 0x1d, 0x09, 0, 0, 0, 0, 0, 0, 0, 0x80, 0x12, 0x12, 0x0a, 0x10,
    ];
    expected.extend([0x32; 16]);
    assert_eq!(projected.encode_to_vec(), expected);
    assert_eq!(
        decode_structural_value(wire::StructuralValue::decode_from_slice(&expected).unwrap())
            .unwrap(),
        native
    );
    assert_eq!(
        encode_structural_value(Value::Missing)
            .unwrap()
            .encode_to_vec(),
        [0x82, 0x01, 0]
    );
    assert_ne!(
        encode_structural_value(Value::Integer(0))
            .unwrap()
            .encode_to_vec(),
        [0x82, 0x01, 0]
    );
}

#[test]
fn actual_preparation_bindings_cross_real_binary_without_losing_distinct_structure() {
    let native = request();
    let expected = native.bindings.clone();
    let bytes = encode_preparation_discovery_request(native)
        .unwrap()
        .encode_to_vec();
    let decoded = decode_preparation_discovery_request(
        wire::PreparationDiscoveryRequest::decode_from_slice(&bytes).unwrap(),
    )
    .unwrap();
    assert_eq!(decoded.bindings, expected);
    let Value::Tuple(values) = &decoded.bindings.arguments["values"] else {
        panic!("tuple stays distinct")
    };
    assert!(matches!(values[0], Value::Missing));
    assert!(matches!(&values[1], Value::Set(items) if items == &[Value::Integer(7)]));
    let Value::Row { names, fields, .. } = &values[2] else {
        panic!("ordered row")
    };
    assert_eq!(names.as_ref(), ["z", "a"]);
    assert_eq!(
        fields.as_ref(),
        [Value::Text("first".into()), Value::Text("second".into())]
    );
}

#[test]
fn duplicate_typed_fact_keys_refuse_instead_of_becoming_last_wins() {
    let mut projected = encode_preparation_discovery_request(request()).unwrap();
    let bindings = projected.bindings.as_option_mut().unwrap();
    bindings.facts.push(bindings.facts[0].clone());
    assert!(matches!(decode_preparation_discovery_request(projected),
        Err(BoundaryProjectionError::Adoption(AdapterError::Refused { path, kind: AdapterFailure::DuplicateKey, .. })) if path.ends_with("bindings.facts")));
}

#[test]
fn unknown_binary_data_roundtrips_in_buffa_but_native_adoption_explicitly_refuses() {
    let mut bytes = encode_structural_value(Value::Missing)
        .unwrap()
        .encode_to_vec();
    bytes.extend([0x98, 0x06, 7]); // Unknown field 99, varint 7.
    let projected = wire::StructuralValue::decode_from_slice(&bytes).unwrap();
    assert_eq!(projected.encode_to_vec(), bytes);
    assert!(matches!(
        decode_structural_value(projected),
        Err(BoundaryProjectionError::Adoption(AdapterError::Refused {
            kind: AdapterFailure::UnknownField,
            ..
        }))
    ));
    assert!(decode_structural_value(wire::StructuralValue::default()).is_err());
}

#[test]
fn private_durable_cursor_fields_and_original_report_scopes_survive_the_crossing() {
    let cursor =
        pse_operations::canonical_selection::ProductHeaderCursor::from_projection_fields((
            "issuing-store".into(),
            "protected-problem".into(),
            "original-revision".into(),
            "after-member".into(),
        ))
        .unwrap();
    let native = crate::workflow::PreparationDiscoveryReport {
        revision_id: "exact-local-parent".into(),
        physical: "exact-physical-context".into(),
        resident: vec![],
        residents_truncated: false,
        durable: Some(pse_operations::canonical_selection::ProductHeaderPage {
            rows: vec![],
            next: Some(cursor),
        }),
        durable_eligibility: crate::workflow::PreparationEligibility::Unchecked,
    };
    let projected = encode_preparation_discovery_report(native).unwrap();
    let decoded = decode_preparation_discovery_report(projected).unwrap();
    assert_eq!(decoded.revision_id, "exact-local-parent");
    assert_eq!(decoded.physical, "exact-physical-context");
    assert_eq!(
        decoded.durable_eligibility,
        crate::workflow::PreparationEligibility::Unchecked
    );
    assert_eq!(
        decoded
            .durable
            .unwrap()
            .next
            .unwrap()
            .into_projection_fields(),
        (
            "issuing-store".into(),
            "protected-problem".into(),
            "original-revision".into(),
            "after-member".into()
        )
    );
}

#[test]
fn outgoing_native_envelope_violation_is_an_internal_producer_failure() {
    let resident = crate::workflow::ResidentPreparation {
        kind: crate::workflow::PreparationProductKind::Body,
        eligibility: crate::workflow::PreparationEligibility::Unchecked,
        retained_bytes: 1,
        semantic_identity: None,
    };
    let native = crate::workflow::PreparationDiscoveryReport {
        revision_id: "exact-parent".into(),
        physical: "physical-context".into(),
        resident: vec![
            resident;
            crate::workflow::PreparationDiscoveryReport::MAX_RESIDENT_ROWS + 1
        ],
        residents_truncated: true,
        durable: None,
        durable_eligibility: crate::workflow::PreparationEligibility::Unchecked,
    };
    let error = encode_preparation_discovery_report(native).unwrap_err();
    assert!(matches!(&error, BoundaryProjectionError::OutgoingNative(
        pse_contract::boundary_projection::ValidationError::Rule { path, rule: "length", .. }) if path == "resident"));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::WorkflowInternal)
    );
}

#[test]
fn generated_ratio_validation_advertises_the_actual_signed_component_width() {
    use protovalidate_buffa::Validate as _;
    for num in [-32768, 32767] {
        wire::PhysicalRatio {
            num,
            den: 1,
            ..Default::default()
        }
        .validate()
        .unwrap();
    }
    for num in [-32769, 32768] {
        assert!(
            wire::PhysicalRatio {
                num,
                den: 1,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
    // Canonical reduction is a separate original-owner construction predicate, rather
    // than a second physics implementation in protobuf validation.
    assert!(<pse_quantity::Ratio as ProjectionRecord>::from_projection_fields((2, 4)).is_err());
}
