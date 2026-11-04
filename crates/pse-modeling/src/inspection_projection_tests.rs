// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Controls for actual selected model inspection owners.

use crate::{
    Type,
    resolved::{Carrier, CarrierKind},
    specialize::{
        Value,
        finite::{DomainKey, FamilyKey},
    },
};
use pse_contract::boundary_projection::{
    Direction, ProjectionAlternative, ProjectionGraph, ProjectionRecord, Shape, TypeRef,
    ValidateProjection,
};
use pse_model::generated::identities::{DeclarationId, InstanceId};
use pse_quantity::{boundary::BoundaryDocument, scheme::Scheme};

fn declaration(byte: u8) -> DeclarationId {
    pse_ids::SemanticId::from_bytes([byte; 16]).into()
}

#[test]
fn actual_function_type_retains_recursive_pair_arguments_order_and_native_bytes() {
    let native = Type::Function {
        arguments: vec![
            ("b".into(), Type::Text),
            ("a".into(), Type::Optional(Box::new(Type::Boolean))),
        ],
        result: Box::new(Type::Tuple(vec![Type::Integer, Type::Boolean])),
    };
    let expected = br#"{"Function":{"arguments":[["b","Text"],["a",{"Optional":"Boolean"}]],"result":{"Tuple":["Integer","Boolean"]}}}"#;
    assert_eq!(serde_json::to_vec(&native).unwrap(), expected);
    let (arguments, result) =
        <Type as ProjectionAlternative<22>>::into_projection_alternative(native.clone()).unwrap();
    assert_eq!(arguments[0].0, "b");
    assert_eq!(arguments[1].0, "a");
    assert_eq!(*result, Type::Tuple(vec![Type::Integer, Type::Boolean]));
    let restored =
        <Type as ProjectionAlternative<22>>::from_projection_alternative((arguments, result));
    assert_eq!(restored, native);
    assert_eq!(serde_json::from_slice::<Type>(expected).unwrap(), native);
    native.validate_projection().unwrap();

    // This root must be registered first: pre-registering the pair or Type would
    // hide the recursive generic-path failure encountered by real generation.
    let graph = ProjectionGraph::of::<Type>().unwrap();
    let ty = &graph.definitions()["ModelingType"];
    assert_eq!(ty.public_rust_path.as_deref(), Some("pse_modeling::Type"));
    assert_eq!(ty.direction, Direction::Output);
    let Shape::Alternatives(alternatives) = &ty.shape else {
        panic!("actual Type alternatives")
    };
    assert_eq!(alternatives.len(), 24);
    let function = alternatives
        .iter()
        .find(|alternative| alternative.tag == 22)
        .unwrap();
    assert_eq!(function.fields[0].name, "arguments");
    assert_eq!(
        function.fields[0].ty,
        TypeRef::Repeated(Box::new(TypeRef::Named(
            "Tuple2OfStringAndModelingType".into()
        )))
    );
    assert_eq!(
        graph.definitions()["Tuple2OfStringAndModelingType"]
            .public_rust_path
            .as_deref(),
        Some("(String,pse_modeling::Type)")
    );
    for name in [
        "QuantityScheme",
        "PhysicalSchemeRecord",
        "PhysicalSchemeShape",
        "PhysicalContractRecord",
        "PhysicalContractFacts",
        "PhysicalFormulaFacts",
        "BoundIndexReference",
        "BoundIndexId",
        "DomainId",
        "ModelingPhysicalRefinement",
        "ModelingBoundaryRef",
        "StructuralValue",
    ] {
        assert!(
            graph.definitions().contains_key(name),
            "actual Type closure omits {name}"
        );
    }
}

#[test]
fn actual_domain_key_keeps_nominal_owners_parent_context_and_ragged_prefix() {
    let native = DomainKey {
        family: FamilyKey {
            instance: InstanceId::from_bytes([7; 16]),
            declaration: declaration(11),
            role: "operating-points".into(),
            parents: vec![Value::Text("outer".into())],
        },
        position: 1,
        prefix: vec![Value::Integer(i64::MAX), Value::Boolean(false)],
    };
    let bytes = serde_json::to_vec(&native).unwrap();
    let (family, position, prefix) = native.clone().into_projection_fields();
    assert_eq!(family.instance, InstanceId::from_bytes([7; 16]));
    assert_eq!(family.declaration, declaration(11));
    assert_eq!(family.parents, vec![Value::Text("outer".into())]);
    assert_eq!(
        prefix,
        vec![Value::Integer(i64::MAX), Value::Boolean(false)]
    );
    let restored = DomainKey::from_projection_fields((family, position, prefix)).unwrap();
    assert_eq!(restored, native);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert_eq!(serde_json::from_slice::<DomainKey>(&bytes).unwrap(), native);
    let graph = ProjectionGraph::of::<DomainKey>().unwrap();
    assert_eq!(
        graph.definitions()["ModelingDomainKey"]
            .public_rust_path
            .as_deref(),
        Some("pse_modeling::specialize::finite::DomainKey")
    );
    assert_eq!(
        DomainKey::FIELD_LAYOUT,
        &[(1, "family"), (2, "position"), (3, "prefix")]
    );
}

#[test]
fn actual_carrier_keeps_empty_element_meaning_kind_and_semantic_member_order() {
    let empty = Carrier {
        kind: CarrierKind::OrderedSet,
        element: Type::Entity(declaration(2)),
        members: Vec::<Value>::new().into(),
    };
    let bytes = serde_json::to_vec(&empty).unwrap();
    let restored = Carrier::from_projection_fields(empty.clone().into_projection_fields()).unwrap();
    assert_eq!(restored, empty);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    assert_eq!(serde_json::from_slice::<Carrier>(&bytes).unwrap(), empty);
    assert_eq!(restored.element, Type::Entity(declaration(2)));
    assert!(restored.members.is_empty());

    let native = Carrier {
        kind: CarrierKind::OrderedSet,
        element: Type::Integer,
        members: vec![
            Value::Integer(17),
            Value::Integer(-5),
            Value::Integer(i64::MAX),
        ]
        .into(),
    };
    let restored =
        Carrier::from_projection_fields(native.clone().into_projection_fields()).unwrap();
    assert_eq!(
        restored.members.as_ref(),
        &[
            Value::Integer(17),
            Value::Integer(-5),
            Value::Integer(i64::MAX)
        ]
    );
    assert_eq!(restored, native);
    assert_ne!(serde_json::to_vec(&native).unwrap(), bytes);
    native.validate_projection().unwrap();
    let graph = ProjectionGraph::of::<Carrier>().unwrap();
    assert_eq!(
        graph.definitions()["ModelingCarrier"]
            .public_rust_path
            .as_deref(),
        Some("pse_modeling::resolved::Carrier")
    );
    assert_eq!(
        Carrier::FIELD_LAYOUT,
        &[(1, "kind"), (2, "element"), (3, "members")]
    );
}

#[test]
fn actual_type_publishes_resolved_contract_metadata_without_reconstructing_authority() {
    let registry = pse_quantity::generated::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let contract = pse_quantity::ResolvedPhysicalContract::named(
        quantity,
        pse_quantity::IndexSet::new(),
        &registry,
    )
    .unwrap();
    let scheme = Scheme::Resolved(Box::new(contract));
    let document = scheme.boundary_document();
    let native = Type::Quantity(scheme);
    native.validate_projection().unwrap();
    let bytes = serde_json::to_vec(&native).unwrap();
    let (observed,) =
        <Type as ProjectionAlternative<10>>::into_projection_alternative(native).unwrap();
    assert_eq!(observed.boundary_document(), document);
    assert!(document.restore().is_err());
    assert!(Scheme::from_projection_fields((document,)).is_err());
    assert!(
        serde_json::from_slice::<Type>(&bytes).is_err(),
        "native receipt restoration remains qualified"
    );
}
