// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated passive inspection crossings preserve metadata without qualified adoption.

use super::*;
use crate::{
    engineering_projection as codec,
    host_operations::engineering::generated::pb::pse::engineering::v1 as pb,
    workflow::OperationSource,
};
use buffa::Message as _;

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
