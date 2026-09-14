// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Domain facts and lexical index admission (blueprint §6.3, §8.3).
use crate::{MathIrError, NodeId};
use pse_ids::SemanticId;
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet, UnitId};

/// Actual domain declaration plus ordered members, supplied by the predecessor bundle.
#[derive(Clone, Debug)]
pub struct DomainFacts {
    /// Declared domain meaning.
    pub kind: DomainKind,
    /// Authored continuous-domain flag, not inferred from its name.
    pub continuous: bool,
    /// Declared coordinate unit, mandatory for differentiation/integration.
    pub unit: Option<UnitId>,
    /// Members in declared domain-member order.
    pub members: Vec<SemanticId>,
}
/// Resolve an existing lexical binder to its complete actual domain identity.
///
/// # Errors
/// Rejects a binder absent from this expression occurrence's environment.
pub fn resolve_binder(
    node: NodeId,
    bound: BoundIndexId,
    environment: &IndexSet,
) -> Result<BoundIndexRef, MathIrError> {
    environment
        .get(bound)
        .copied()
        .ok_or(MathIrError::UnboundIndex {
            node,
            bound_index: bound,
        })
}
/// Extend a lexical environment without silently rebinding an existing index identity.
///
/// # Errors
/// Rejects a repeated binder even when its domain happens to agree.
pub fn bind(
    node: NodeId,
    environment: &IndexSet,
    bound: BoundIndexId,
    domain: DomainId,
    facts: &DomainFacts,
) -> Result<IndexSet, MathIrError> {
    if environment.contains_index(bound) {
        return Err(MathIrError::malformed_at(
            node,
            "binder identity is already in lexical scope",
        ));
    }
    let mut result = environment.clone();
    result
        .insert(BoundIndexRef::new(bound, domain, facts.kind))
        .map_err(|_| {
            MathIrError::malformed_at(node, "binder conflicts with lexical environment")
        })?;
    Ok(result)
}
