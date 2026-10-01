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
    fn component_responses_keep_distinct_meanings_and_species_context() {
        use crate::{PhysicalName, ResolvedPhysicalContract, resolved::infer_operation};
        let registry = standard_registry().unwrap();
        let quantity = |name| match registry.physical_name(name).unwrap() {
            PhysicalName::QuantityType(id) => id,
            PhysicalName::ReferenceState(_) => panic!("expected a quantity"),
        };
        let resolved = |name| {
            ResolvedPhysicalContract::named(quantity(name), IndexSet::new(), &registry).unwrap()
        };
        for name in [
            "LogFugacityCoefficient",
            "LogActivityCoefficient",
            "GibbsDuhemResidual",
        ] {
            let value = resolved(name);
            assert!(
                registry
                    .quantity_type(quantity(name))
                    .unwrap()
                    .key
                    .subject_kind
                    .is_some()
            );
            assert!(value.at_boundary(quantity("Scalar"), &registry).is_err());
            for other in [
                "LogFugacityCoefficient",
                "LogActivityCoefficient",
                "GibbsDuhemResidual",
            ] {
                assert_eq!(
                    value.at_boundary(quantity(other), &registry).is_ok(),
                    name == other
                );
                assert_eq!(
                    infer_operation(
                        &OpRequest::Add,
                        &[value.clone(), resolved(other)],
                        None,
                        &registry,
                        &StandardInvariantChecker
                    )
                    .is_ok(),
                    name == other
                );
            }
        }
        for (logarithm, coefficient) in [
            ("LogFugacityCoefficient", "FugacityCoefficient"),
            ("LogActivityCoefficient", "ActivityCoefficient"),
        ] {
            let result = infer_operation(
                &OpRequest::Transcendental(crate::Opcode::Exp),
                &[resolved(logarithm)],
                None,
                &registry,
                &StandardInvariantChecker,
            )
            .unwrap();
            assert_eq!(
                result.result.require_named().unwrap(),
                quantity(coefficient)
            );
            assert_eq!(
                result.result.qualified_factors()[0].key.subject_kind,
                registry
                    .quantity_type(quantity(logarithm))
                    .unwrap()
                    .key
                    .subject_kind
            );
        }
        let total = infer_operation(
            &OpRequest::FiniteReduce {
                kind: crate::ReductionKind::Sum,
                domain: registry
                    .quantity_type(quantity("Amount"))
                    .unwrap()
                    .key
                    .subject_kind,
            },
            &[resolved("Amount")],
            None,
            &registry,
            &StandardInvariantChecker,
        )
        .unwrap();
        assert_eq!(
            total.result.require_named().unwrap(),
            quantity("TotalAmount")
        );
    }

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

    #[test]
    fn discrete_scaling_keeps_the_scaled_contract() {
        use crate::infer::{BuiltInRule, OperationSelection};
        let registry = standard_registry().expect("source package projection");
        let quantity = |hex| QuantityTypeId::from_id(pse_ids::SemanticId::parse_hex(hex).unwrap());
        let power = quantity("e1f2106da9eb4fe0aa2749fa5469fa1a");
        let count = quantity("3a8f6d2c9b1e4f7a8c5d0e3b6a9f2c18");
        let indicator = quantity("b5d9e1c4a7f2483e9d6c1b0a5e8f3d27");
        let neutral = registry.neutral_dimensionless().unwrap();
        let indices = IndexSet::new();
        let apply = |request: OpRequest<'_>, left, right| {
            infer_with_evidence(
                &request,
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
        let discrete = OperationSelection::BuiltIn(BuiltInRule::DiscreteScaling);
        for (request, left, right, result) in [
            (OpRequest::Mul, power, indicator, power),
            (OpRequest::Mul, indicator, power, power),
            (OpRequest::Mul, count, power, power),
            (OpRequest::Div, power, count, power),
            // A neutral quantity switched by an indicator stays neutral.
            (OpRequest::Mul, neutral, indicator, neutral),
            (OpRequest::Mul, indicator, neutral, neutral),
            (OpRequest::Mul, indicator, indicator, indicator),
        ] {
            let inferred = apply(request, left, right).unwrap();
            assert_eq!((inferred.result, &inferred.selected), (result, &discrete));
        }
        // A count is not a divisor's reciprocal: dividing by a quantity stays registered-only.
        assert!(apply(OpRequest::Div, count, power).is_err());
        assert!(registry.discrete_category(power).unwrap().is_none());
        assert_eq!(
            registry.discrete_category(indicator).unwrap(),
            Some(crate::QuantityKindCategory::Indicator)
        );
    }
}
