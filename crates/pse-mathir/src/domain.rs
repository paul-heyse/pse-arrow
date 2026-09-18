// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual and template-local domain references (blueprint §7.1, ADR-0054).
use crate::{MathIrError, NodeId};
use pse_ids::SemanticId;
use pse_quantity::DomainId;

/// A domain identity, or the explicit composite key awaiting instance binding.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainRef {
    /// An instantiated domain declaration.
    Actual(DomainId),
    /// An authored template-domain key; no synthetic domain ID is manufactured.
    Template {
        /// Owning template declaration.
        template_id: SemanticId,
        /// Exact local domain name declared by the template.
        domain_name: String,
    },
}
impl DomainRef {
    /// Return the actual declaration before physical or index inference.
    ///
    /// # Errors
    /// Refuses an unresolved template key; a digest cannot resolve the binding.
    pub fn require_actual(&self, node: NodeId) -> Result<DomainId, MathIrError> {
        match self {
            Self::Actual(domain) => Ok(*domain),
            Self::Template {
                template_id,
                domain_name,
            } => Err(MathIrError::UnresolvedDomain {
                node,
                template_id: *template_id,
                domain_name: domain_name.clone(),
            }),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), MathIrError> {
        if matches!(self, Self::Template { domain_name, .. } if domain_name.is_empty()) {
            return Err(MathIrError::malformed(
                "template domain name must not be empty",
            ));
        }
        Ok(())
    }
}
impl From<DomainId> for DomainRef {
    fn from(value: DomainId) -> Self {
        Self::Actual(value)
    }
}
