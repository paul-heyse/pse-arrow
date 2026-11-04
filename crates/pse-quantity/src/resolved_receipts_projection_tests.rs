// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Controls for source-owned complete physical receipt data, without qualification.

use super::*;
use pse_contract::boundary_projection::{
    ConstructionRole, Direction, ProjectionGraph, ProjectionRecord, Scalar, Shape, TypeRef,
    ValidateProjection,
};

#[test]
fn physical_receipt_projection_preserves_actual_outcomes_without_replay_authority() {
    let registry = crate::generated::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let contract = ResolvedPhysicalContract::named(quantity, IndexSet::new(), &registry).unwrap();
    let ((inferred, boundary), stream) = capture(|| {
        let inferred = infer_operation(
            &OpRequest::Add,
            &[contract.clone(), contract.clone()],
            None,
            &registry,
            &crate::infer::NoInvariantFacts,
        )?;
        let boundary = AdmittedOutputBoundary::declared(
            pse_ids::SemanticId::from_bytes([47; 16]),
            contract.clone(),
            quantity,
            &registry,
        )?;
        Ok::<_, QuantityError>((inferred, boundary))
    })
    .unwrap();
    assert_eq!(stream.entries.len(), 1);
    assert_eq!(stream.boundaries.len(), 1);
    let original = serde_json::to_vec(&stream).unwrap();
    let reconstructed =
        ReceiptStream::from_projection_fields(stream.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&reconstructed).unwrap(), original);
    reconstructed.validate_projection().unwrap();
    assert!(require_record(&reconstructed).is_err());
    assert!(InferenceRecord::capture(&inferred).restore().is_err());
    assert!(OutputBoundaryRecord::capture(&boundary).restore().is_err());

    let graph = ProjectionGraph::of::<ReceiptStream>().unwrap();
    for name in [
        "PhysicalReceiptStream",
        "PhysicalInferenceReceipt",
        "PhysicalInferenceFacts",
        "PhysicalOutputBoundaryReceipt",
        "PhysicalOutputBoundaryFacts",
        "PhysicalOperationSelection",
        "PhysicalOperandConversion",
    ] {
        let owner = &graph.definitions()[name];
        assert_eq!(owner.construction, ConstructionRole::Data);
        assert_eq!(owner.direction, Direction::Both);
    }
    let Shape::Record(fields) = &graph.definitions()["PhysicalOutputBoundaryFacts"].shape else {
        panic!("actual complete output boundary data")
    };
    assert_eq!(fields[3].name, "scale_bits");
    assert_eq!(fields[3].ty, TypeRef::Scalar(Scalar::F64Bits));
    // Reconstructing passive stream fields never starts a scientific replay session.
    assert!(
        infer_operation(
            &OpRequest::Add,
            &[contract.clone(), contract],
            None,
            &registry,
            &crate::infer::NoInvariantFacts,
        )
        .is_ok()
    );
    assert!(
        ReceiptStream {
            version: 2,
            entries: vec![],
            boundaries: vec![]
        }
        .validate_projection()
        .is_err()
    );
}

#[test]
fn physical_selection_projection_preserves_registered_permutation_and_closed_rules() {
    let selected = OperationSelection::Registered {
        operation: pse_ids::SemanticId::from_bytes([48; 16]).into(),
        operand_permutation: vec![2, 0, 2, u16::MAX],
    };
    use pse_contract::boundary_projection::ProjectionAlternative;
    let bytes = serde_json::to_vec(&selected).unwrap();
    let fields =
        <OperationSelection as ProjectionAlternative<2>>::into_projection_alternative(selected)
            .unwrap();
    let reconstructed =
        <OperationSelection as ProjectionAlternative<2>>::from_projection_alternative(fields);
    assert_eq!(serde_json::to_vec(&reconstructed).unwrap(), bytes);
    let graph = ProjectionGraph::of::<OperationSelection>().unwrap();
    let Shape::StringEnum(values) = &graph.definitions()["PhysicalBuiltInRule"].shape else {
        panic!("original closed native vocabulary")
    };
    assert_eq!(
        values,
        &BuiltInRule::ALL
            .iter()
            .map(|rule| rule.as_str().to_owned())
            .collect::<Vec<_>>()
    );
}
