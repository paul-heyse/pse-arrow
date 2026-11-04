// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical bulk values preserve exact representation and identity without admission.
use super::{ProjectionGraph, ProjectionRecord, ValidateProjection};
use crate::generated::{extension_values::QuantityValue, structures::ScientificCoordinateValue};
use pse_ids::SemanticId;

#[test]
fn physical_coordinate_projection_preserves_actual_extension_keys_bits_and_native_json() {
    let coordinate = SemanticId::from_bytes([21; 16]);
    let quantity = SemanticId::from_bytes([22; 16]);
    let unit = SemanticId::from_bytes([23; 16]);
    for bits in [
        0,
        (-0.0_f64).to_bits(),
        2.5_f64.to_bits(),
        f64::INFINITY.to_bits(),
        0x7ff8_0000_0000_1234,
    ] {
        let original = ScientificCoordinateValue {
            coordinate_id: coordinate,
            value: QuantityValue {
                value: f64::from_bits(bits),
                quantity_type_id: quantity,
                unit_id: unit,
            },
        };
        let json = f64::from_bits(bits)
            .is_finite()
            .then(|| serde_json::to_vec(&original).unwrap());
        original.validate_projection().unwrap();
        let restored =
            ScientificCoordinateValue::from_projection_fields(original.into_projection_fields())
                .unwrap();
        assert_eq!(restored.coordinate_id, coordinate);
        assert_eq!(restored.value.quantity_type_id, quantity);
        assert_eq!(restored.value.unit_id, unit);
        assert_eq!(restored.value.value.to_bits(), bits);
        if let Some(json) = json {
            assert_eq!(serde_json::to_vec(&restored).unwrap(), json);
        }
    }
    let graph = ProjectionGraph::of::<ScientificCoordinateValue>().unwrap();
    assert!(graph.definitions().contains_key("ExtensionQuantityValue"));
    assert_eq!(
        QuantityValue::FIELD_LAYOUT,
        &[(1, "value"), (2, "quantity_type_id"), (3, "unit_id")]
    );
    // A pure projection validates representation without resolving these keys;
    // scientific admission remains contextual.
}
