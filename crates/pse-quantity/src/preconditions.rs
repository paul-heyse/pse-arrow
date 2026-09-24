// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One retained physical prerequisite inventory; inference checks actual operands.

use crate::{
    InvariantId, PhysicalPrecondition, QuantityError, QuantityOperation, QuantityRegistry,
    infer::{InvariantChecker, OpRequest, Operand},
};

/// Immutable physical prerequisite values; no execution or native owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalPreconditions {
    declarations: Vec<PhysicalPrecondition>,
}
impl PhysicalPreconditions {
    /// Admit a strictly ordered complete prerequisite inventory.
    /// # Errors
    /// Duplicate or reversed prerequisite identities.
    pub fn new(declarations: Vec<PhysicalPrecondition>) -> Result<Self, QuantityError> {
        // Input is natively ordered by the declared ID. This algorithm inventory retains
        // each actual declaration exactly once, never a caller-supplied status flag.
        if declarations.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(refusal(
                "physical prerequisite identity order is duplicated or reversed",
            ));
        }
        Ok(Self { declarations })
    }
    /// Exact declared prerequisite values.
    pub fn declarations(&self) -> &[PhysicalPrecondition] {
        &self.declarations
    }
}
impl InvariantChecker for PhysicalPreconditions {
    fn immutable(&self) -> bool {
        true
    }
    fn check(
        &self,
        invariant: InvariantId,
        _request: &OpRequest<'_>,
        operation: Option<&QuantityOperation>,
        operands: &[Operand<'_>],
        registry: &QuantityRegistry,
    ) -> Result<(), QuantityError> {
        let position = self
            .declarations
            .binary_search_by_key(&invariant, |declaration| declaration.id)
            .map_err(|_| refusal("no actual quantity prerequisite declaration"))?;
        let operation =
            operation.ok_or_else(|| refusal("prerequisite is not bound to an actual operation"))?;
        self.declarations[position].check(operation, operands, registry)
    }
}
fn refusal(detail: &str) -> QuantityError {
    QuantityError::InferencePrecondition {
        rule: "operation.physical_prerequisite",
        detail: detail.to_owned(),
    }
}
