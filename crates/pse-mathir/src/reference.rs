// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit normalized declaration and predicate references (blueprint §7.1, ADR-0054).
use crate::{DomainRef, MathIrError, NodeId};
use pse_ids::SemanticId;
use pse_quantity::BoundIndexId;

/// Which declared template member supplies an unresolved value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemplateValueKind {
    /// A declared parameter.
    Parameter,
    /// A declared feature.
    Feature,
    /// A declared port.
    Port,
}
impl TemplateValueKind {
    /// Closed declaration vocabulary, used by schema projections.
    pub const ALL: [Self; 3] = [Self::Parameter, Self::Feature, Self::Port];
    /// Stable declaration spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parameter => "parameter",
            Self::Feature => "feature",
            Self::Port => "port",
        }
    }
}
/// A bound declaration reference, preserving its actual key until instantiation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueRef {
    /// A real symbol or template-symbol declaration identity.
    ActualSymbol(SemanticId),
    /// Exact template member key; never a fabricated symbol ID.
    Template {
        /// Owning template.
        template_id: SemanticId,
        /// Declared member category.
        kind: TemplateValueKind,
        /// Exact member name.
        name: String,
    },
    /// Actual or template-local domain declaration.
    Domain(DomainRef),
    /// Lexically declared bound index.
    Index(BoundIndexId),
}
impl ValueRef {
    /// Sole normalized declaration discriminator roster for registry projection.
    pub const KINDS: [&'static str; 6] = [
        "symbol",
        TemplateValueKind::Parameter.as_str(),
        TemplateValueKind::Feature.as_str(),
        TemplateValueKind::Port.as_str(),
        "domain",
        "index",
    ];

    /// Require an actual symbol before physical inference looks up its complete contract.
    ///
    /// # Errors
    /// Template members, domain values and bound indices require prior lowering.
    pub fn require_symbol(&self, node: NodeId) -> Result<SemanticId, MathIrError> {
        match self {
            Self::ActualSymbol(symbol) => Ok(*symbol),
            _ => Err(MathIrError::UnresolvedValue {
                node,
                kind: self.kind(),
            }),
        }
    }
    /// Normalized relation discriminator.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ActualSymbol(_) => Self::KINDS[0],
            Self::Template { kind, .. } => kind.as_str(),
            Self::Domain(_) => Self::KINDS[4],
            Self::Index(_) => Self::KINDS[5],
        }
    }
    pub(crate) fn validate(&self) -> Result<(), MathIrError> {
        match self {
            Self::Template { name, .. } if name.is_empty() => Err(MathIrError::malformed(
                "template value name must not be empty",
            )),
            Self::Domain(domain) => domain.validate(),
            _ => Ok(()),
        }
    }
}
impl From<SemanticId> for ValueRef {
    fn from(value: SemanticId) -> Self {
        Self::ActualSymbol(value)
    }
}
/// An existing math node or an explicit normalized predicate graph key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GuardRef {
    /// A compiled boolean math expression.
    Math(NodeId),
    /// A node in the normalized predicate relation graph.
    Predicate {
        /// Source expression identity.
        source_id: SemanticId,
        /// Predicate ordinal local to that source.
        predicate_id: u64,
    },
}
impl GuardRef {
    /// Return an ordinary math dependency only for that explicit alternative.
    pub const fn math(self) -> Option<NodeId> {
        match self {
            Self::Math(node) => Some(node),
            Self::Predicate { .. } => None,
        }
    }
    /// Require prior predicate lowering before physical math inference.
    ///
    /// # Errors
    /// A normalized predicate is not interchangeable with a numeric node ordinal.
    pub fn require_math(self, node: NodeId) -> Result<NodeId, MathIrError> {
        self.math().ok_or(MathIrError::UnresolvedValue {
            node,
            kind: "predicate",
        })
    }
    /// Admit exact nullable relation alternatives.
    ///
    /// # Errors
    /// Rejects absent, incomplete or overlapping math/predicate alternatives.
    pub fn from_columns(
        math: Option<NodeId>,
        source: Option<SemanticId>,
        predicate: Option<u64>,
    ) -> Result<Self, MathIrError> {
        match (math, source, predicate) {
            (Some(node), None, None) => Ok(Self::Math(node)),
            (None, Some(source_id), Some(predicate_id)) => Ok(Self::Predicate {
                source_id,
                predicate_id,
            }),
            _ => Err(MathIrError::malformed(
                "guard reference requires exactly one complete alternative",
            )),
        }
    }
}
impl From<NodeId> for GuardRef {
    fn from(value: NodeId) -> Self {
        Self::Math(value)
    }
}
