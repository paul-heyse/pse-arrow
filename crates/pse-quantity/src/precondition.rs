// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared physical prerequisites checked against actual operand contracts.

use std::collections::BTreeSet;

use crate::{
    BasisId, InvariantId, QuantityError, QuantityOperation, QuantityRegistry, QuantityTypeId,
    infer::Operand,
};

/// The closed physical predicate carried by a precondition declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PhysicalRequirement {
    /// All selected operands have the same non-absent admitted basis.
    EqualOperandBases {
        /// When present, that common basis must be this exact admitted basis.
        required: Option<BasisId>,
    },
    /// Every selected operand has the required complete physical key.
    OperandQuantityContract {
        /// Admitted quantity supplying every required physical axis.
        required: QuantityTypeId,
        /// Compare ordered shape as well; false still compares every other key axis.
        match_shape: bool,
    },
    /// Differences share one actual reference and the prototype's other axes.
    /// This explicitly permits normalization within another admitted datum;
    /// it never converts between datums or accepts affine points.
    SameReferenceDifferences {
        /// Admitted difference supplying kind, basis, subject and optional shape.
        required: QuantityTypeId,
        /// Compare the prototype's ordered shape as well.
        match_shape: bool,
    },
}

/// A predicate over explicitly selected positions in the operation's declared order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalPrecondition {
    /// Stable declaration identity, never evidence that the predicate holds.
    pub id: InvariantId,
    /// Distinct zero-based positions in declared operation order.
    pub operand_positions: Vec<u16>,
    /// Actual predicate to evaluate.
    pub requirement: PhysicalRequirement,
}

impl PhysicalPrecondition {
    /// Validate the predicate's own declaration against the admitted registry.
    ///
    /// # Errors
    /// Empty/duplicate positions, malformed equality or an undeclared physical type.
    pub fn validate(&self, registry: &QuantityRegistry) -> Result<(), QuantityError> {
        if self.operand_positions.is_empty()
            || self.operand_positions.iter().collect::<BTreeSet<_>>().len()
                != self.operand_positions.len()
        {
            return Err(refusal(
                "physical prerequisite needs distinct nonempty positions",
            ));
        }
        match self.requirement {
            PhysicalRequirement::EqualOperandBases { required } => {
                if self.operand_positions.len() < 2 {
                    return Err(refusal("basis equality needs at least two actual operands"));
                }
                if let Some(basis) = required {
                    registry.basis(basis)?;
                }
            }
            PhysicalRequirement::OperandQuantityContract { required, .. } => {
                registry.quantity_type(required)?;
            }
            PhysicalRequirement::SameReferenceDifferences { required, .. } => {
                if self.operand_positions.len() < 2
                    || registry.quantity_type(required)?.key.scale_kind
                        != crate::ScaleKind::Difference
                {
                    return Err(refusal(
                        "reference normalization needs a difference prototype and at least two operands",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Prove this prerequisite over the actual admitted operation and operand keys.
    /// Callers supply operands in declared order, after any admitted explicit input
    /// conversions. The request's output identity is irrelevant to this proof.
    ///
    /// # Errors
    /// Invalid declaration, foreign operation, absent position or unequal physical axes.
    pub fn check(
        &self,
        operation: &QuantityOperation,
        operands: &[Operand<'_>],
        registry: &QuantityRegistry,
    ) -> Result<(), QuantityError> {
        self.validate(registry)?;
        let admitted = registry.operation(operation.id)?;
        if !std::ptr::eq(admitted, operation)
            || !operation.precondition_invariants.contains(&self.id)
            || operands.len() != operation.input_kinds.len()
        {
            return Err(refusal(
                "prerequisite is not bound to the actual admitted operation",
            ));
        }
        let mut expected_basis = match self.requirement {
            PhysicalRequirement::EqualOperandBases { required } => required,
            PhysicalRequirement::OperandQuantityContract { .. }
            | PhysicalRequirement::SameReferenceDifferences { .. } => None,
        };
        let mut reference = None;
        for position in &self.operand_positions {
            let operand = operands
                .get(usize::from(*position))
                .ok_or_else(|| refusal("prerequisite operand position is absent"))?;
            let actual = &registry.quantity_type(operand.quantity_type)?.key;
            match self.requirement {
                PhysicalRequirement::EqualOperandBases { .. } => {
                    let basis = actual
                        .basis
                        .ok_or_else(|| refusal("prerequisite operand has no actual basis"))?;
                    registry.basis(basis)?;
                    if expected_basis.is_some_and(|expected| expected != basis) {
                        return Err(refusal("actual operand bases differ"));
                    }
                    expected_basis = Some(basis);
                }
                PhysicalRequirement::OperandQuantityContract {
                    required,
                    match_shape,
                }
                | PhysicalRequirement::SameReferenceDifferences {
                    required,
                    match_shape,
                } => {
                    let expected = &registry.quantity_type(required)?.key;
                    let family = matches!(
                        self.requirement,
                        PhysicalRequirement::SameReferenceDifferences { .. }
                    );
                    let reference_matches = if family {
                        let same = reference
                            .is_none_or(|previous| previous == actual.reference_state)
                            && actual.reference_state.is_some()
                                == expected.reference_state.is_some();
                        reference = Some(actual.reference_state);
                        same
                    } else {
                        actual.reference_state == expected.reference_state
                    };
                    if actual.kind != expected.kind
                        || actual.basis != expected.basis
                        || !reference_matches
                        || actual.scale_kind != expected.scale_kind
                        || actual.subject_kind != expected.subject_kind
                        || (match_shape && actual.shape != expected.shape)
                    {
                        return Err(refusal(&format!(
                            "operation {} prerequisite {} operand {position}: actual {actual:?} differs from required {expected:?} (match_shape={match_shape})",
                            operation.id, self.id,
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

fn refusal(detail: &str) -> QuantityError {
    QuantityError::InferencePrecondition {
        rule: "operation.physical_prerequisite",
        detail: detail.to_owned(),
    }
}
