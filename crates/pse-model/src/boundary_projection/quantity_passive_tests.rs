// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused controls for the actual passive quantity bridge.

use super::*;
use pse_quantity::{
    resolved::receipts::{ContractRecord, SchemeRecord},
    scheme::Scheme,
};

#[test]
fn passive_scheme_projection_preserves_ordinary_native_document_bytes() {
    let native = Scheme::Product(
        Box::new(Scheme::Variable("left".into())),
        Box::new(Scheme::Delta(Box::new(Scheme::Variable("right".into())))),
    );
    let expected = br#"{"Product":[{"Variable":"left"},{"Delta":{"Variable":"right"}}]}"#;
    assert_eq!(serde_json::to_vec(&native).unwrap(), expected);
    let document = native.boundary_document();
    assert_eq!(serde_json::to_vec(&document).unwrap(), expected);
    assert!(
        document.restore().is_err(),
        "the original whole-record recipe requirement survives"
    );
    let native = Scheme::from_projection_fields(native.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&native).unwrap(), expected);
    native.validate_projection().unwrap();

    let graph = ProjectionGraph::of::<Scheme>().unwrap();
    let Shape::Alternatives(alternatives) = &graph.definitions()["PhysicalSchemeShape"].shape
    else {
        panic!("source-owned actual scheme alternatives")
    };
    assert_eq!(
        alternatives
            .iter()
            .map(|alternative| (alternative.name.as_str(), alternative.tag))
            .collect::<Vec<_>>(),
        vec![
            ("Concrete", 1),
            ("Resolved", 2),
            ("Variable", 3),
            ("Delta", 4),
            ("Product", 5),
            ("Quotient", 6),
            ("Power", 7)
        ]
    );
    let Shape::Record(fields) = &graph.definitions()["PhysicalContractFacts"].shape else {
        panic!("actual complete physical contract facts")
    };
    assert_eq!(fields.len(), 10);
    assert_eq!(fields[7].name, "scale_bits");
    assert_eq!(fields[7].tag, 8);
    assert_eq!(fields[7].ty, TypeRef::Scalar(Scalar::F64Bits));
    assert_eq!(
        graph.definitions()["QuantityScheme"].direction,
        Direction::Output
    );
}

#[test]
fn passive_resolved_output_facts_are_readable_but_native_restoration_requires_receipt() {
    // Construct a real contract through the actual native quantity registry. A
    // handmade receipt or a duplicate Type AST would not establish this case.
    let registry = pse_quantity::generated::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let contract = pse_quantity::ResolvedPhysicalContract::named(
        quantity,
        pse_quantity::IndexSet::new(),
        &registry,
    )
    .unwrap();
    let record = ContractRecord::capture(&contract);
    let original = serde_json::to_vec(&record).unwrap();
    let scheme = Scheme::Resolved(Box::new(contract.clone()));
    let document = scheme.boundary_document();
    scheme.validate_projection().unwrap();
    document.validate_projection().unwrap();

    // Borrow original passive payloads using the public associated types; no
    // private ContractWire member access or JSON parsing supplies their meaning.
    type SchemeShape = <SchemeRecord as BoundaryField<1>>::Type;
    let shape = <SchemeRecord as BoundaryField<1>>::boundary_field(&document);
    let (observed,) =
        <SchemeShape as BoundaryAlternative<2>>::boundary_alternative_fields(shape).unwrap();
    assert_eq!(serde_json::to_vec(observed).unwrap(), original);
    type Facts = <ContractRecord as BoundaryField<1>>::Type;
    let facts = <ContractRecord as BoundaryField<1>>::boundary_field(observed);
    assert_eq!(
        <Facts as BoundaryField<1>>::boundary_field(facts),
        &contract.named_id()
    );
    assert_eq!(
        *<Facts as BoundaryField<8>>::boundary_field(facts),
        1.0_f64.to_bits()
    );

    // Passive construction preserves the bytes without acquiring physical proof.
    let restored_record =
        ContractRecord::from_projection_fields(record.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&restored_record).unwrap(), original);
    let error = restored_record.restore().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("admitted record is absent from qualified reconstruction recipe")
    );
    assert!(Scheme::from_projection_fields((document.clone(),)).is_err());
    assert!(document.restore().is_err());
    assert!(serde_json::from_slice::<Scheme>(&serde_json::to_vec(&scheme).unwrap()).is_err());

    // A nested resolved leaf retains exactly the same original refusal.
    let nested = Scheme::Product(Box::new(Scheme::Variable("x".into())), Box::new(scheme));
    nested.validate_projection().unwrap();
    let error = Scheme::from_projection_fields(nested.into_projection_fields()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("admitted record is absent from qualified reconstruction recipe")
    );
}
