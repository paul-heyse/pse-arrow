// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact physical-contract comparison (blueprint §8.1, §8.3).
use crate::{ContractComponent, QuantityError, QuantityRegistry, QuantityTypeId};
/// Compare all semantic components and the canonical unit after actual registry lookup.
///
/// # Errors
/// Rejects unknown identities or names the first differing contract component.
pub fn require_same_contract(
    expected: QuantityTypeId,
    actual: QuantityTypeId,
    registry: &QuantityRegistry,
) -> Result<(), QuantityError> {
    let left = registry.quantity_type(expected)?;
    let right = registry.quantity_type(actual)?;
    for (equal, component) in [
        (left.key.kind == right.key.kind, ContractComponent::Kind),
        (left.key.basis == right.key.basis, ContractComponent::Basis),
        (
            left.key.reference_state == right.key.reference_state,
            ContractComponent::ReferenceState,
        ),
        (
            left.key.scale_kind == right.key.scale_kind,
            ContractComponent::ScaleKind,
        ),
        (left.key.shape == right.key.shape, ContractComponent::Shape),
        (
            left.key.subject_kind == right.key.subject_kind,
            ContractComponent::SubjectKind,
        ),
        (
            left.canonical_unit == right.canonical_unit,
            ContractComponent::Unit,
        ),
    ] {
        if !equal {
            return Err(QuantityError::ContractMismatch {
                component,
                expected,
                actual,
            });
        }
    }
    Ok(())
}
