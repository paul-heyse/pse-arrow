// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated equation bounds must carry the actual free binders into P10.
use super::typed_zero;
use pse_ids::SemanticId;
use pse_mathir::{
    ExprGraph,
    canonicalize::{CanonicalizeInput, Policy, RootEnvironment, canonicalize_with_environments},
    index::DomainFacts,
    infer::SymbolTypeSource,
};
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet, QuantityTypeId};
use std::collections::BTreeMap;

struct Domains(BTreeMap<DomainId, DomainFacts>);
impl SymbolTypeSource for Domains {
    fn symbol_type(&self, _: SemanticId) -> Option<QuantityTypeId> {
        None
    }
    fn domain(&self, domain: DomainId) -> Option<&DomainFacts> {
        self.0.get(&domain)
    }
}

#[test]
fn equation_zero_bounds_infer_scalar_and_multiple_declared_axes() {
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let scalar = registry.neutral_dimensionless().unwrap();
    for kinds in [
        vec![],
        vec![DomainKind::Species],
        vec![DomainKind::PortSet, DomainKind::Species],
    ] {
        let mut domains = Domains(BTreeMap::new());
        let mut indices = IndexSet::new();
        for (ordinal, kind) in kinds.iter().copied().enumerate() {
            let ordinal = u8::try_from(ordinal).unwrap();
            let domain = DomainId::from_id(SemanticId::from_bytes([ordinal + 1; 16]));
            indices
                .insert(BoundIndexRef::new(
                    BoundIndexId::from_id(SemanticId::from_bytes([ordinal + 10; 16])),
                    domain,
                    kind,
                ))
                .unwrap();
            domains.0.insert(
                domain,
                DomainFacts {
                    kind,
                    continuous: false,
                    unit: None,
                    members: vec![SemanticId::from_bytes([ordinal + 20; 16])],
                },
            );
        }
        let mut key = registry.quantity_type(scalar).unwrap().key.clone();
        key.shape = kinds;
        let expected = registry.resolve_key(&key).unwrap();
        let mut graph = ExprGraph::new();
        let bound = typed_zero(&mut graph, expected, &indices, &registry).unwrap();
        let roots = [bound];
        let environments = [RootEnvironment {
            indices: indices.clone(),
            expected: Some(expected),
        }];
        let result = canonicalize_with_environments(
            CanonicalizeInput::new(&graph, &roots, &domains, &registry),
            &environments,
            Policy::Strict,
        )
        .unwrap();
        let bound = result.node(result.roots()[0]).unwrap();
        assert_eq!(bound.quantity_type, expected);
        assert_eq!(bound.free_indices, indices);

        if !indices.is_empty() {
            let scalar_bound =
                typed_zero(&mut graph, expected, &IndexSet::new(), &registry).unwrap();
            assert!(
                canonicalize_with_environments(
                    CanonicalizeInput::new(&graph, &[scalar_bound], &domains, &registry),
                    &environments,
                    Policy::Strict,
                )
                .is_err(),
                "a scalar bound cannot silently acquire the equation axes"
            );
        }
    }
}
