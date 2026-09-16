// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::passes::native_test::{Inputs, put, session};
use pse_ids::FixedBudget;
use pse_relations::generated::{
    enums::{DomainKind, ParameterSourceCoordinate},
    normalized, reference,
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn inputs(registry: &Registry, kind: DomainKind, mapping: ParameterSourceCoordinate) -> Inputs {
    let mut rows = BTreeMap::new();
    put(
        &mut rows,
        registry,
        vec![reference::method_parameters::Row {
            method_id: id(1),
            name: "coefficient".into(),
            quantity_type_id: id(2),
            natural_unit_id: id(3),
            indexed_by: vec![kind],
            required: true,
        }],
    );
    put(
        &mut rows,
        registry,
        vec![reference::method_parameter_axes::Row {
            method_id: id(1),
            parameter_kind: "coefficient".into(),
            position: 0,
            source_coordinate: mapping,
        }],
    );
    put(
        &mut rows,
        registry,
        vec![normalized::domains::Row {
            domain_id: id(4),
            owner_entity_id: id(5),
            kind,
            continuous: false,
            unit_id: None,
            parent_domain_id: None,
            doc: String::new(),
        }],
    );
    put(
        &mut rows,
        registry,
        vec![normalized::domain_members::Row {
            domain_id: id(4),
            member_id: id(6),
            ordinal: 20,
            label: "unrelated-label".into(),
            coordinate: None,
            ref_entity_id: Some(id(7)),
        }],
    );
    put(
        &mut rows,
        registry,
        vec![normalized::material_domain_members::Row {
            domain_id: id(4),
            member_id: id(6),
            material_system_id: id(8),
            phase_id: Some(id(9)),
            species_id: Some(id(10)),
            element_id: None,
            derivation_id: id(11),
        }],
    );
    rows
}
#[tokio::test]
async fn declared_member_reference_and_material_pair_are_distinct_projections() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let budget = FixedBudget::new(2usize << 30);
    let cancel = CancellationToken::new();
    for (kind, mapping, expected) in [
        (
            DomainKind::Species,
            ParameterSourceCoordinate::Member,
            vec![id(6)],
        ),
        (
            DomainKind::Species,
            ParameterSourceCoordinate::RefEntity,
            vec![id(7)],
        ),
        (
            DomainKind::PhaseSpecies,
            ParameterSourceCoordinate::PhaseSpeciesPair,
            vec![id(9), id(10)],
        ),
    ] {
        let rows = inputs(&registry, kind, mapping);
        let (native, sources) = session(&registry, rows, &budget, &cancel).unwrap();
        let projector = ParameterIndexProjector::new(&registry, &native, &sources, budget.as_ref());
        let result = projector
            .project(id(1), "coefficient", &[id(4)], &[id(6)], &cancel)
            .await
            .unwrap();
        assert_eq!(result.values, expected);
        assert!(
            result
                .sources
                .iter()
                .any(|source| source.relation == reference::method_parameter_axes::RELATION_KEY)
        );
        assert!(
            projector
                .project(id(1), "coefficient", &[id(4)], &[id(20)], &cancel)
                .await
                .is_err()
        );
    }
    assert_eq!(budget.reserved(), 0);
}
#[tokio::test]
async fn missing_mapping_and_missing_reference_never_fall_back_to_member() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let budget = FixedBudget::new(2usize << 30);
    let cancel = CancellationToken::new();
    let mut rows = inputs(
        &registry,
        DomainKind::Species,
        ParameterSourceCoordinate::RefEntity,
    );
    put(
        &mut rows,
        &registry,
        Vec::<reference::method_parameter_axes::Row>::new(),
    );
    let (native, sources) = session(&registry, rows, &budget, &cancel).unwrap();
    let projector = ParameterIndexProjector::new(&registry, &native, &sources, budget.as_ref());
    assert!(
        projector
            .project(id(1), "coefficient", &[id(4)], &[id(6)], &cancel)
            .await
            .is_err()
    );
    drop(projector);
    drop(native);
    drop(sources);
    rows = inputs(
        &registry,
        DomainKind::Species,
        ParameterSourceCoordinate::RefEntity,
    );
    put(
        &mut rows,
        &registry,
        vec![normalized::domain_members::Row {
            domain_id: id(4),
            member_id: id(6),
            ordinal: 0,
            label: String::new(),
            coordinate: None,
            ref_entity_id: None,
        }],
    );
    let (native, sources) = session(&registry, rows, &budget, &cancel).unwrap();
    let projector = ParameterIndexProjector::new(&registry, &native, &sources, budget.as_ref());
    assert!(
        projector
            .project(id(1), "coefficient", &[id(4)], &[id(6)], &cancel)
            .await
            .is_err()
    );
    drop(projector);
    drop(native);
    drop(sources);
    assert_eq!(budget.reserved(), 0);
}
#[tokio::test]
async fn projection_checks_exact_domain_and_releases_on_cancellation() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let budget = FixedBudget::new(2usize << 30);
    let cancel = CancellationToken::new();
    let rows = inputs(
        &registry,
        DomainKind::Species,
        ParameterSourceCoordinate::PhaseSpeciesPair,
    );
    let (native, sources) = session(&registry, rows, &budget, &cancel).unwrap();
    let projector = ParameterIndexProjector::new(&registry, &native, &sources, budget.as_ref());
    assert!(
        projector
            .project(id(1), "coefficient", &[id(4)], &[id(6)], &cancel)
            .await
            .is_err()
    );
    cancel.cancel();
    assert!(
        projector
            .project(id(1), "coefficient", &[id(4)], &[id(6)], &cancel)
            .await
            .is_err()
    );
    drop(projector);
    drop(native);
    drop(sources);
    assert_eq!(budget.reserved(), 0);
}
