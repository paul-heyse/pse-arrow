// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test access to the admitted reference package projection (blueprint §8.2–§8.3).
//!
//! Source YAML is the declaration authority (ADR-0064). Currency years 2000/2001 use synthetic
//! CE indices 500/1000 solely to exercise scale ratios; they are not historical prices.
use crate::infer::{InvariantChecker, OpRequest, Operand};
use crate::{InvariantId, QuantityError, QuantityOperation, QuantityRegistry};

/// Named-policy fixture identities; the qualified name is the declaration's stable key.
pub mod ids {
    use crate::{
        BasisId, ConversionId, InvariantId, OperationId, QuantityKindId, QuantityTypeId,
        ReferenceStateId, UnitId, UnitSetId,
    };
    use pse_ids::{SemanticId, named_id};
    fn named(family: &str, name: &str) -> SemanticId {
        let package = named_id(SemanticId::NIL, "pse.test.standard");
        named_id(package, &format!("{family}.{name}"))
    }
    macro_rules! identity {
        ($function:ident,$ty:ty) => {
            #[doc=concat!("Named identity of a fixture `",stringify!($function),"` declaration.")]
            pub fn $function(name: &str) -> $ty {
                <$ty>::from_id(named(stringify!($function), name))
            }
        };
    }
    identity!(unit, UnitId);
    identity!(kind, QuantityKindId);
    identity!(quantity, QuantityTypeId);
    identity!(basis, BasisId);
    identity!(reference, ReferenceStateId);
    identity!(operation, OperationId);
    identity!(invariant, InvariantId);
    identity!(conversion, ConversionId);
    identity!(unit_set, UnitSetId);
}
/// Re-admit the generated projection of the explicitly selected source packages.
///
/// # Errors
/// Invalid generated declarations are reported by the ordinary quantity registry builder.
pub fn standard_registry() -> Result<QuantityRegistry, QuantityError> {
    crate::generated::standard_registry()
}

/// Fixture checker of the actual generated physical prerequisite declarations.
/// It cannot certify weighted means because no actual weight values were supplied.
#[derive(Debug)]
pub struct StandardInvariantChecker;
impl InvariantChecker for StandardInvariantChecker {
    fn immutable(&self) -> bool {
        true
    }
    fn check(
        &self,
        id: InvariantId,
        _request: &OpRequest<'_>,
        operation: Option<&QuantityOperation>,
        operands: &[Operand<'_>],
        registry: &QuantityRegistry,
    ) -> Result<(), QuantityError> {
        let operation = operation.ok_or_else(|| QuantityError::InferencePrecondition {
            rule: "fixture.invariant_scope",
            detail: "actual registered operation is required; no weight-value witness supplied"
                .to_owned(),
        })?;
        let declaration = crate::generated::standard_preconditions()
            .into_iter()
            .find(|declaration| declaration.id == id)
            .ok_or_else(|| QuantityError::InferencePrecondition {
                rule: "fixture.invariant_scope",
                detail: "physical prerequisite declaration is absent".to_owned(),
            })?;
        declaration.check(operation, operands, registry)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "explicit source-contract test fixtures"
)]
mod tests {
    use super::*;
    use crate::{IndexSet, QuantityTypeId, infer::infer_with_evidence};

    #[test]
    fn caloric_operations_require_the_declared_datum_and_difference_roles() {
        let registry = standard_registry().expect("source package projection");
        let quantity = |hex| QuantityTypeId::from_id(pse_ids::SemanticId::parse_hex(hex).unwrap());
        let delta_h = quantity("d5bb3d48b9804f2f8d5a6f0a7cadaee8");
        let delta_t = quantity("459a933fd00837bbc50372e31ac9801c");
        let cp = quantity("cd653ba98fa94d16b5d66b363f21c3d6");
        let indices = IndexSet::new();
        let divide = |left, right| {
            infer_with_evidence(
                &OpRequest::Div,
                &[
                    Operand {
                        quantity_type: left,
                        indices: &indices,
                    },
                    Operand {
                        quantity_type: right,
                        indices: &indices,
                    },
                ],
                &registry,
                &StandardInvariantChecker,
            )
        };
        assert_eq!(divide(delta_h, delta_t).unwrap().result, cp);
        assert_eq!(
            divide(cp, cp).unwrap().result,
            registry.neutral_dimensionless().unwrap()
        );
        for wrong_h in [
            quantity("1831d0d72dc74b299ba8ecb6d4da6f53"), // a point of the same datum
            quantity("94b88c5bead5458629c2afc11ffcff20"), // another datum's difference
        ] {
            assert!(divide(wrong_h, delta_t).is_err());
            assert!(divide(wrong_h, delta_h).is_err());
        }
        assert!(divide(delta_h, quantity("c64b96975a4a59755f8711d3bf628bc9")).is_err());
    }
}
