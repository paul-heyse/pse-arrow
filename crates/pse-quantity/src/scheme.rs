// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Quantity-polymorphic signatures use complete physical types and existing inference.
use crate::infer::{Chain, InvariantChecker, NoInvariantFacts, OpRequest, Operand};
use crate::{IndexSet, QuantityRegistry, QuantityTypeId, Ratio};
use std::collections::BTreeMap;

/// A physical type scheme; no dimension-only substitution is admitted.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scheme {
    /// A registered complete physical type.
    Concrete(QuantityTypeId),
    /// A universally quantified physical type.
    Variable(String),
    /// Difference type of a point or difference quantity.
    Delta(Box<Scheme>),
    /// Product resolved by registered physical operation rules.
    Product(Box<Scheme>, Box<Scheme>),
    /// Quotient resolved by registered physical operation rules.
    Quotient(Box<Scheme>, Box<Scheme>),
    /// Exact rational power.
    Power(Box<Scheme>, Ratio),
}
/// A malformed or incompatible physical signature.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SchemeError {
    /// Unresolved, ambiguous or incompatible complete physical contract.
    #[error("quantity scheme: {0}")]
    Contract(String),
}
pse_diagnostics::impl_diagnostic! {
    SchemeError, code(_this) { Some(pse_diagnostics::DiagnosticCode::ValidationInvariant) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
/// Concrete bindings carry IDs of fully admitted physical types.
pub type Substitution = BTreeMap<String, QuantityTypeId>;

impl Scheme {
    /// Resolve a scheme using the existing physical registry and operation inference.
    /// # Errors
    /// An unbound variable, missing difference type, ambiguous rule or physical mismatch.
    pub fn resolve(
        &self,
        registry: &QuantityRegistry,
        bindings: &Substitution,
    ) -> Result<QuantityTypeId, SchemeError> {
        self.resolve_with_evidence(registry, bindings, &NoInvariantFacts)
    }
    /// Resolve against explicit immutable physical prerequisites, checking actual operands.
    /// # Errors
    /// The same scheme errors, including absent or unsatisfied prerequisites.
    pub fn resolve_with_evidence(
        &self,
        registry: &QuantityRegistry,
        bindings: &Substitution,
        checker: &dyn InvariantChecker,
    ) -> Result<QuantityTypeId, SchemeError> {
        let bad = |reason: String| SchemeError::Contract(reason);
        match self {
            Self::Concrete(id) => {
                registry
                    .quantity_types()
                    .find(|t| t.id == *id)
                    .ok_or_else(|| bad(format!("unknown physical type {id}")))?;
                Ok(*id)
            }
            Self::Variable(name) => bindings
                .get(name)
                .copied()
                .ok_or_else(|| bad(format!("unbound {name}"))),
            Self::Delta(value) => {
                let id = value.resolve_with_evidence(registry, bindings, checker)?;
                binary(registry, OpRequest::Sub, id, id, checker)
            }
            // A product, quotient or power resolves as one maximal chain, by the
            // registered rules and by its canonical monomial (ADR-0124).
            Self::Product(..) | Self::Quotient(..) | Self::Power(..) => {
                let indices = IndexSet::new();
                let chain = self.chain(registry, bindings, checker, &indices)?;
                crate::infer::infer_chain(&chain, registry, checker)
                    .map(|t| t.result)
                    .map_err(|e| bad(e.to_string()))
            }
        }
    }
    fn chain<'a>(
        &self,
        registry: &QuantityRegistry,
        bindings: &Substitution,
        checker: &dyn InvariantChecker,
        indices: &'a IndexSet,
    ) -> Result<Chain<'a>, SchemeError> {
        let node = |value: &Self| {
            value
                .chain(registry, bindings, checker, indices)
                .map(Box::new)
        };
        Ok(match self {
            Self::Product(a, b) => Chain::Mul(node(a)?, node(b)?),
            Self::Quotient(a, b) => Chain::Div(node(a)?, node(b)?),
            Self::Power(a, exponent) => Chain::Pow {
                base: node(a)?,
                exponent: *exponent,
                power: Operand {
                    quantity_type: registry.neutral_dimensionless().ok_or_else(|| {
                        SchemeError::Contract("neutral exponent type absent".into())
                    })?,
                    indices,
                },
            },
            leaf => Chain::Leaf(Operand {
                quantity_type: leaf.resolve_with_evidence(registry, bindings, checker)?,
                indices,
            }),
        })
    }
    /// Substitute complete schemes when checking one polymorphic function inside another.
    /// # Errors
    /// A result variable was not determined by an explicit argument.
    pub fn substitute(&self, bindings: &BTreeMap<String, Scheme>) -> Result<Scheme, SchemeError> {
        Ok(match self {
            Self::Concrete(id) => Self::Concrete(*id),
            Self::Variable(name) => bindings
                .get(name)
                .cloned()
                .ok_or_else(|| SchemeError::Contract(format!("unbound type variable {name}")))?,
            Self::Delta(v) => Self::Delta(Box::new(v.substitute(bindings)?)),
            Self::Product(a, b) => Self::Product(
                Box::new(a.substitute(bindings)?),
                Box::new(b.substitute(bindings)?),
            ),
            Self::Quotient(a, b) => Self::Quotient(
                Box::new(a.substitute(bindings)?),
                Box::new(b.substitute(bindings)?),
            ),
            Self::Power(v, power) => Self::Power(Box::new(v.substitute(bindings)?), *power),
        })
    }
    /// Bind a formal argument without losing basis, reference, scale or shape.
    /// # Errors
    /// The concrete argument cannot satisfy the scheme or conflicts with an earlier argument.
    pub fn bind(
        &self,
        actual: QuantityTypeId,
        registry: &QuantityRegistry,
        bindings: &mut Substitution,
    ) -> Result<(), SchemeError> {
        self.bind_with_evidence(actual, registry, bindings, &NoInvariantFacts)
    }
    /// Bind a complete argument with the same prerequisites used to resolve its scheme.
    /// # Errors
    /// Incompatible types, unbound variables or unsatisfied physical prerequisites.
    pub fn bind_with_evidence(
        &self,
        actual: QuantityTypeId,
        registry: &QuantityRegistry,
        bindings: &mut Substitution,
        checker: &dyn InvariantChecker,
    ) -> Result<(), SchemeError> {
        if let Self::Variable(name) = self {
            if registry.quantity_types().all(|t| t.id != actual) {
                return Err(SchemeError::Contract("unknown actual type".into()));
            }
            if let Some(prior) = bindings.get(name) {
                if *prior != actual {
                    return Err(SchemeError::Contract(format!(
                        "conflicting complete types for {name}"
                    )));
                }
            } else {
                bindings.insert(name.clone(), actual);
            }
            return Ok(());
        }
        let expected = self.resolve_with_evidence(registry, bindings, checker)?;
        if expected != actual {
            return Err(SchemeError::Contract(format!(
                "expected {expected}, got {actual}"
            )));
        }
        Ok(())
    }
}
fn binary(
    registry: &QuantityRegistry,
    request: OpRequest<'_>,
    a: QuantityTypeId,
    b: QuantityTypeId,
    checker: &dyn InvariantChecker,
) -> Result<QuantityTypeId, SchemeError> {
    let indices = IndexSet::new();
    crate::infer::infer_with_evidence(
        &request,
        &[
            Operand {
                quantity_type: a,
                indices: &indices,
            },
            Operand {
                quantity_type: b,
                indices: &indices,
            },
        ],
        registry,
        checker,
    )
    .map(|t| t.result)
    .map_err(|e| SchemeError::Contract(e.to_string()))
}
