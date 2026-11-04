// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native resolved graph data, independent of admitted scientific reconstruction.

use super::*;
use pse_contract::boundary_projection::{
    ConstructionRole, ProjectionAlternative, ProjectionGraph, ProjectionRecord, Shape,
};

#[test]
fn resolved_graph_projection_preserves_original_native_forms_and_complete_grammar() {
    let binary = Node::Binary {
        op: BinaryOp::Add,
        lhs: NodeIx::from(0),
        rhs: NodeIx::from(1),
    };
    let expected = br#"{"Binary":{"op":"Add","lhs":0,"rhs":1}}"#;
    assert_eq!(serde_json::to_vec(&binary).unwrap(), expected);
    let fields = <Node as ProjectionAlternative<4>>::into_projection_alternative(binary).unwrap();
    let restored = <Node as ProjectionAlternative<4>>::from_projection_alternative(fields);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), expected);
    assert_eq!(
        serde_json::to_vec(&Binding::Accumulator).unwrap(),
        br#""Accumulator""#
    );
    let predicate = ResolvedPredicate::And(PredicateIx::from(0), PredicateIx::from(1));
    let fields =
        <ResolvedPredicate as ProjectionAlternative<3>>::into_projection_alternative(predicate)
            .unwrap();
    let restored =
        <ResolvedPredicate as ProjectionAlternative<3>>::from_projection_alternative(fields);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), br#"{"And":[0,1]}"#);
    assert_eq!(
        serde_json::to_vec(&dsl::Function::Exp).unwrap(),
        br#""Exp""#
    );
    assert_eq!(dsl::Function::Exp.as_str(), "exp");
    assert!(matches!(
        dsl::Function::Convert.implementation(),
        pse_quantity::functions::Implementation::Unavailable
    ));

    let graph = ProjectionGraph::of::<Node>().unwrap();
    let Shape::Alternatives(nodes) = &graph.definitions()["ModelingResolvedNode"].shape else {
        panic!("actual resolved node grammar")
    };
    assert_eq!(nodes.len(), 16);
    assert_eq!((nodes[0].name.as_str(), nodes[0].tag), ("Number", 1));
    assert_eq!((nodes[15].name.as_str(), nodes[15].tag), ("Let", 16));
    for name in [
        "ModelingResolvedNode",
        "ModelingResolvedReference",
        "ModelingProviderSelection",
        "AuthoredScientificNumber",
        "AuthoredBinaryOperator",
        "AuthoredReductionKind",
        "AuthoredBuiltinFunction",
    ] {
        assert_eq!(
            graph.definitions()[name].construction,
            ConstructionRole::Data
        );
    }
    let graph = ProjectionGraph::of::<ResolvedPredicate>().unwrap();
    let Shape::Alternatives(predicates) = &graph.definitions()["ModelingResolvedPredicate"].shape
    else {
        panic!("actual predicate grammar")
    };
    assert_eq!(predicates.len(), 8);
    assert!(
        graph
            .definitions()
            .contains_key("AuthoredComparisonOperator")
    );
}

#[test]
fn physical_evidence_record_projection_does_not_restore_scientific_or_live_context() {
    let record =
        PhysicalEvidenceRecord::from_projection_fields((ContentHash::from_bytes([53; 32]),))
            .unwrap();
    let original = serde_json::to_vec(&record).unwrap();
    let restored =
        PhysicalEvidenceRecord::from_projection_fields(record.into_projection_fields()).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), original);
    assert!(restored.restore().is_err());
    let graph = ProjectionGraph::of::<PhysicalEvidenceRecord>().unwrap();
    assert_eq!(
        graph.definitions()["ModelingPhysicalEvidenceRecord"].construction,
        ConstructionRole::Data
    );
}

#[test]
fn physical_operation_record_projection_keeps_original_transfer_orientation_and_scope_data() {
    use operation_records::OperationRecord;
    use pse_contract::boundary_projection::{Scalar, TypeRef};
    let declaration = DeclarationId::from(SemanticId::from_bytes([54; 16]));
    let value = OperationRecord::Transfer {
        source: None,
        result: crate::portable::RefinementWire::Coordinate {
            map: declaration,
            slot: declaration,
        },
        factor: -1,
        exchange: None,
    };
    let expected = format!(
        "{{\"Transfer\":{{\"source\":null,\"result\":{{\"Coordinate\":{{\"map\":\"{}\",\"slot\":\"{}\"}}}},\"factor\":-1,\"exchange\":null}}}}",
        declaration.as_id(),
        declaration.as_id()
    );
    assert_eq!(serde_json::to_vec(&value).unwrap(), expected.as_bytes());
    let fields =
        <OperationRecord as ProjectionAlternative<7>>::into_projection_alternative(value).unwrap();
    let restored =
        <OperationRecord as ProjectionAlternative<7>>::from_projection_alternative(fields);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), expected.as_bytes());
    let graph = ProjectionGraph::of::<OperationRecord>().unwrap();
    let Shape::Alternatives(variants) =
        &graph.definitions()["ModelingPhysicalOperationRecord"].shape
    else {
        panic!("actual physical-operation record variants")
    };
    assert_eq!(variants.len(), 8);
    assert_eq!(variants[6].fields[2].ty, TypeRef::Scalar(Scalar::I8));
    for name in [
        "ModelingPhysicalOperationRecord",
        "ModelingOperationTranslationRecord",
        "ModelingOperationAnchorRecord",
        "PhysicalCanonicalMagnitudeRecord",
    ] {
        assert_eq!(
            graph.definitions()[name].construction,
            ConstructionRole::Data
        );
    }
}
