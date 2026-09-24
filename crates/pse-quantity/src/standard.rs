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
