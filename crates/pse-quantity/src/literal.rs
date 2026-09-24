// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Per-occurrence literal resolution (blueprint §8.3).
use crate::{QuantityError, QuantityRegistry, QuantityTypeId, UnitId};
/// Context belongs to a literal occurrence, never its shared untyped node identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralContext {
    /// No expected contract; ambiguity must fail.
    Free,
    /// Expected contract of the adjacent additive operand.
    Additive {
        /// Sibling's complete type.
        sibling: QuantityTypeId,
    },
    /// Expected contract supplied by the opposite equation side.
    EquationSide {
        /// Opposite side's complete type.
        side: QuantityTypeId,
    },
    /// An explicitly declared physical type, including point/difference semantics.
    Explicit {
        /// Declared complete type.
        quantity_type: QuantityTypeId,
    },
}
/// Resolve a literal using its expected complete type or a unique registered candidate.
///
/// # Errors
/// Rejects missing units/types, incompatible expected units, or ambiguous free literals.
pub fn resolve_literal(
    unit: UnitId,
    context: LiteralContext,
    registry: &QuantityRegistry,
) -> Result<QuantityTypeId, QuantityError> {
    let source = registry.unit(unit)?;
    let expected = match context {
        LiteralContext::Free => None,
        LiteralContext::Additive { sibling } => Some(sibling),
        LiteralContext::EquationSide { side } => Some(side),
        LiteralContext::Explicit { quantity_type } => Some(quantity_type),
    };
    if let Some(id) = expected {
        let ty = registry.quantity_type(id)?;
        if !ty.key.shape.is_empty() {
            return Err(QuantityError::InferencePrecondition {
                rule: "literal.scalar",
                detail: "a literal requires explicit broadcast to acquire free indices".to_owned(),
            });
        }
        crate::convert_spec_for_type(source, registry.unit(ty.canonical_unit)?, &ty.key)?;
        return Ok(id);
    }
    let mut candidates = Vec::new();
    for ty in registry.quantity_types() {
        if ty.key.shape.is_empty()
            && registry.kind(ty.key.kind)?.dimension == source.dimension
            && (source.reference_state.is_none()
                || source.reference_state == ty.key.reference_state)
        {
            candidates.push(ty.id);
        }
    }
    if let [only] = candidates.as_slice() {
        return Ok(*only);
    }
    Err(QuantityError::AmbiguousLiteral { unit, candidates })
}
