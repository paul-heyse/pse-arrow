// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Quantity-polymorphic signatures use complete physical types and existing inference.
use crate::infer::{Exponent, InvariantChecker, NoInvariantFacts, OpRequest};
use crate::{IndexSet, QuantityRegistry, QuantityTypeId, Ratio};
use std::collections::BTreeMap;

/// A physical type scheme; no dimension-only substitution is admitted.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scheme {
    /// A registered complete physical type.
    Concrete(QuantityTypeId),
    /// A closed admitted anonymous or named expression contract.
    Resolved(Box<crate::ResolvedPhysicalContract>),
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
/// Concrete bindings retain complete admitted contracts, including anonymous intermediates.
pub type Substitution = BTreeMap<String, crate::ResolvedPhysicalContract>;

impl Scheme {
    /// Structural physical signature identity, including retained intermediate contracts.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        match self {
            Self::Concrete(id) => {
                hash.str("named").id(&id.as_id());
            }
            Self::Resolved(contract) => {
                hash.str("resolved");
                contract.frame(hash);
            }
            Self::Variable(name) => {
                hash.str("variable").str(name);
            }
            Self::Delta(value) => {
                hash.str("difference");
                value.frame(hash);
            }
            Self::Product(left, right) | Self::Quotient(left, right) => {
                hash.str(if matches!(self, Self::Product(..)) {
                    "product"
                } else {
                    "quotient"
                });
                left.frame(hash);
                right.frame(hash);
            }
            Self::Power(value, power) => {
                hash.str("power")
                    .part(&power.num().to_le_bytes())
                    .part(&power.den().to_le_bytes());
                value.frame(hash);
            }
        }
    }
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
        if let Self::Concrete(id) = self {
            registry
                .quantity_type(*id)
                .map_err(|error| SchemeError::Contract(error.to_string()))?;
            return Ok(*id);
        }
        self.resolve_contract_with_evidence(registry, bindings, checker)?
            .require_named()
            .map_err(|error| SchemeError::Contract(error.to_string()))
    }
    /// Whether this scheme contains no unbound physical variable.
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Concrete(_) | Self::Resolved(_) => true,
            Self::Variable(_) => false,
            Self::Delta(value) | Self::Power(value, _) => value.is_closed(),
            Self::Product(left, right) | Self::Quotient(left, right) => {
                left.is_closed() && right.is_closed()
            }
        }
    }
    /// Whether every formal variable has an admitted binding in this invocation.
    pub fn is_bound(&self, bindings: &Substitution) -> bool {
        match self {
            Self::Concrete(_) | Self::Resolved(_) => true,
            Self::Variable(name) => bindings.contains_key(name),
            Self::Delta(value) | Self::Power(value, _) => value.is_bound(bindings),
            Self::Product(left, right) | Self::Quotient(left, right) => {
                left.is_bound(bindings) && right.is_bound(bindings)
            }
        }
    }
    /// Resolve without inventing a named declaration for a physical intermediate.
    pub fn resolve_contract_with_evidence(
        &self,
        registry: &QuantityRegistry,
        bindings: &Substitution,
        checker: &dyn InvariantChecker,
    ) -> Result<crate::ResolvedPhysicalContract, SchemeError> {
        let bad = |error: crate::QuantityError| SchemeError::Contract(error.to_string());
        let resolve =
            |scheme: &Self| scheme.resolve_contract_with_evidence(registry, bindings, checker);
        let (request, values) = match self {
            Self::Concrete(id) => {
                return crate::ResolvedPhysicalContract::named(*id, IndexSet::new(), registry)
                    .map_err(bad);
            }
            Self::Resolved(contract) => return Ok((**contract).clone()),
            Self::Variable(name) => {
                return bindings
                    .get(name)
                    .cloned()
                    .ok_or_else(|| SchemeError::Contract(format!("unbound {name}")));
            }
            Self::Delta(value) => {
                let value = resolve(value)?;
                (OpRequest::Sub, vec![value.clone(), value])
            }
            Self::Product(left, right) => (OpRequest::Mul, vec![resolve(left)?, resolve(right)?]),
            Self::Quotient(left, right) => (OpRequest::Div, vec![resolve(left)?, resolve(right)?]),
            Self::Power(base, power) => {
                let neutral = registry
                    .neutral_dimensionless()
                    .ok_or_else(|| SchemeError::Contract("neutral exponent type absent".into()))?;
                (
                    OpRequest::Pow {
                        exponent: Exponent::Rational(*power),
                    },
                    vec![resolve(base)?, resolve(&Self::Concrete(neutral))?],
                )
            }
        };
        crate::resolved::infer_operation(&request, &values, None, registry, checker)
            .map(|admitted| admitted.result)
            .map_err(bad)
    }
    /// Preserve named syntax where possible while retaining an anonymous closed contract.
    pub fn from_contract(contract: crate::ResolvedPhysicalContract) -> Self {
        match contract.named_id() {
            Some(id) if contract.indices().is_empty() => Self::Concrete(id),
            _ => Self::Resolved(Box::new(contract)),
        }
    }
    /// Substitute complete schemes when checking one polymorphic function inside another.
    /// # Errors
    /// A result variable was not determined by an explicit argument.
    pub fn substitute(&self, bindings: &BTreeMap<String, Scheme>) -> Result<Scheme, SchemeError> {
        Ok(match self {
            Self::Concrete(id) => Self::Concrete(*id),
            Self::Resolved(contract) => Self::Resolved(contract.clone()),
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
        let actual = crate::ResolvedPhysicalContract::named(actual, IndexSet::new(), registry)
            .map_err(|error| SchemeError::Contract(error.to_string()))?;
        self.bind_contract_with_evidence(&actual, registry, bindings, checker)
    }
    /// Bind an admitted argument without requiring a synthetic named declaration.
    pub fn bind_contract_with_evidence(
        &self,
        actual: &crate::ResolvedPhysicalContract,
        registry: &QuantityRegistry,
        bindings: &mut Substitution,
        checker: &dyn InvariantChecker,
    ) -> Result<(), SchemeError> {
        if let Self::Variable(name) = self {
            if let Some(prior) = bindings.get(name) {
                if !prior.same_meaning(actual) {
                    return Err(SchemeError::Contract(format!(
                        "conflicting complete types for {name}"
                    )));
                }
            } else {
                bindings.insert(name.clone(), actual.clone());
            }
            return Ok(());
        }
        let expected = self.resolve_contract_with_evidence(registry, bindings, checker)?;
        actual
            .at_contract_boundary(&expected, registry)
            .map(|_| ())
            .map_err(|error| SchemeError::Contract(error.to_string()))
    }
}
