// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive restoration records retain their original scientific permission boundary.

use super::*;
use pse_contract::boundary_projection::{
    ConstructionRole, ProjectionGraph, ProjectionRecord, Scalar, Shape, TypeRef,
};

#[test]
fn passive_type_record_projection_preserves_recursive_source_shape_without_live_adoption() {
    let ty = Type::Function {
        arguments: vec![("x".into(), Type::Optional(Box::new(Type::Integer)))],
        result: Box::new(Type::Quantity(pse_quantity::scheme::Scheme::Variable(
            "T".into(),
        ))),
    };
    let record = TypeRecord::capture(&ty);
    let expected = br#"{"value":{"Function":{"arguments":[["x",{"Optional":"Integer"}]],"result":{"Quantity":{"Variable":"T"}}}}}"#;
    assert_eq!(serde_json::to_vec(&record).unwrap(), expected);
    let restored = TypeRecord::from_projection_fields(record.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), expected);
    assert!(
        restored
            .restore(DeclarationId::from(SemanticId::from_bytes([61; 16])))
            .is_err()
    );
    let graph = ProjectionGraph::of::<TypeRecord>().unwrap();
    for name in [
        "ModelingTypeRecord",
        "ModelingTypeFacts",
        "ModelingBoundaryFacts",
        "ModelingRefinementFacts",
        "ModelingStaticValueFacts",
        "PhysicalSchemeRecord",
    ] {
        assert_eq!(
            graph.definitions()[name].construction,
            ConstructionRole::Data
        );
    }
    assert!(!graph.definitions().contains_key("ModelingType"));
    assert!(!graph.definitions().contains_key("QuantityScheme"));
    let Shape::Alternatives(variants) = &graph.definitions()["ModelingTypeFacts"].shape else {
        panic!("actual passive type facts")
    };
    assert_eq!(variants.len(), 24);
    assert_eq!(variants[21].name, "Function");
    let pair = &graph.definitions()["Tuple2OfStringAndModelingTypeFacts"];
    assert_eq!(
        pair.public_rust_path.as_deref(),
        Some("(String,pse_modeling::portable::TypeWire)")
    );
}

#[test]
fn passive_value_record_projection_preserves_map_order_and_exact_raw_bits_without_grants() {
    let declaration = DeclarationId::from(SemanticId::from_bytes([62; 16]));
    let quantity = pse_quantity::QuantityTypeId::from(SemanticId::from_bytes([63; 16]));
    let value = crate::specialize::Value::Definition {
        id: declaration,
        bindings: BTreeMap::from([
            ("z".into(), crate::specialize::Value::Integer(-7)),
            (
                "a".into(),
                crate::specialize::Value::Number {
                    bits: 1u64 << 63,
                    quantity,
                },
            ),
        ]),
    };
    let record = ValueRecord::capture(&value);
    let original = serde_json::to_vec(&record).unwrap();
    let restored = ValueRecord::from_projection_fields(record.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), original);
    let ValueWire::Definition { bindings, .. } = &restored.value else {
        panic!("actual bound value facts")
    };
    assert_eq!(
        bindings.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert!(matches!(bindings["a"], ValueWire::Number { bits, .. } if bits == 1u64 << 63));
    assert!(restored.restore(declaration).is_err());

    // Passive exact bits remain representable; the original scientific owner refuses them.
    let bits = 0x7ff8_0000_0000_0042;
    let raw = ValueRecord::from_projection_fields((ValueWire::Number { bits, quantity },)).unwrap();
    assert!(serde_json::to_vec(&raw).is_ok());
    assert!(raw.value.restore(declaration).is_err());
    let graph = ProjectionGraph::of::<ValueRecord>().unwrap();
    let Shape::Alternatives(variants) = &graph.definitions()["ModelingStaticValueFacts"].shape
    else {
        panic!("actual passive static value facts")
    };
    assert_eq!(variants.len(), 16);
    assert_eq!(variants[2].fields[0].ty, TypeRef::Scalar(Scalar::F64Bits));
    assert!(!graph.definitions().contains_key("ModelingValue"));
}
