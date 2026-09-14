// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded target syntax resolves through actual instance, declaration and domain rows.

mod parse;
mod resolve;

use crate::SourceSpan;
use pse_ids::SemanticId;
use pse_relations::generated::authored;

pub use parse::parse;
pub(crate) use resolve::resolve_accounted;
pub use resolve::{resolve, resolve_with_budget};
/// Exact generated target row contract; its source identity is supplied to resolution.
pub type TargetRow = authored::case_spec_targets::Row;

/// Finite expansion bound; target ordinals are `UInt16` in the declared relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetBudget {
    /// Maximum emitted targets, at most 65,536.
    pub max_targets: u32,
}
impl Default for TargetBudget {
    fn default() -> Self {
        Self {
            max_targets: 65_536,
        }
    }
}

/// One concrete domain selector, before identity resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexSelector {
    /// The complete declared indexed shape.
    Wildcard,
    /// An exact semantic ID, member label or finite coordinate spelling.
    Value(String),
    /// A quoted exact member label, without numeric-coordinate interpretation.
    Label(String),
}
/// A retained target path with original source provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetPath {
    /// Instance/member names in path order.
    pub names: Vec<String>,
    /// Selectors on the final member, in declaration order.
    pub indices: Vec<IndexSelector>,
    /// Whether the final path segment selects every member of the instance.
    pub instance_wildcard: bool,
    /// Original source position.
    pub at: SourceSpan,
    /// Original text, used in diagnostic reports.
    pub text: String,
}

/// Actual generated facts available at the P1 target binding boundary.
#[derive(Clone, Debug, Default)]
pub struct TargetContext {
    /// Explicit entity names and containment.
    pub entities: Vec<authored::entities::Row>,
    /// Concrete authored instance declarations.
    pub instances: Vec<authored::instances::Row>,
    /// Symbol declarations of the selected templates.
    pub symbols: Vec<authored::template_symbols::Row>,
    /// Equation declarations of the selected templates.
    pub equations: Vec<authored::template_equations::Row>,
    /// Ports retain their actual composite declaration keys.
    pub ports: Vec<authored::template_ports::Row>,
    /// Template-local domain declarations.
    pub template_domains: Vec<authored::template_domains::Row>,
    /// Actual domain facts.
    pub domains: Vec<authored::domains::Row>,
    /// Actual members and their labels/coordinates.
    pub members: Vec<authored::domain_members::Row>,
    /// Explicit realized domain bindings; never guessed from names.
    pub domain_bindings: Vec<authored::instance_domain_bindings::Row>,
}

impl TargetContext {
    /// Decode actual generated rows from a complete candidate relation inventory.
    /// # Errors
    /// Any wrong row shape or logical value type.
    pub fn from_rows(rows: &crate::document::Rows) -> Result<Self, crate::AuthoringError> {
        fn decode<T>(
            rows: &crate::document::Rows,
            id: SemanticId,
            convert: fn(Vec<pse_schema::model::Cell>) -> Result<T, pse_relations::RelationError>,
        ) -> Result<Vec<T>, crate::AuthoringError> {
            rows.get(&id)
                .into_iter()
                .flatten()
                .cloned()
                .map(convert)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| crate::AuthoringError::Contract {
                    at: None,
                    reason: error.to_string(),
                })
        }
        Ok(Self {
            entities: decode(
                rows,
                authored::entities::RELATION_ID,
                authored::entities::Row::from_cells,
            )?,
            instances: decode(
                rows,
                authored::instances::RELATION_ID,
                authored::instances::Row::from_cells,
            )?,
            symbols: decode(
                rows,
                authored::template_symbols::RELATION_ID,
                authored::template_symbols::Row::from_cells,
            )?,
            equations: decode(
                rows,
                authored::template_equations::RELATION_ID,
                authored::template_equations::Row::from_cells,
            )?,
            ports: decode(
                rows,
                authored::template_ports::RELATION_ID,
                authored::template_ports::Row::from_cells,
            )?,
            template_domains: decode(
                rows,
                authored::template_domains::RELATION_ID,
                authored::template_domains::Row::from_cells,
            )?,
            domains: decode(
                rows,
                authored::domains::RELATION_ID,
                authored::domains::Row::from_cells,
            )?,
            members: decode(
                rows,
                authored::domain_members::RELATION_ID,
                authored::domain_members::Row::from_cells,
            )?,
            domain_bindings: decode(
                rows,
                authored::instance_domain_bindings::RELATION_ID,
                authored::instance_domain_bindings::Row::from_cells,
            )?,
        })
    }
}
