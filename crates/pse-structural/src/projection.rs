// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared scope, limits and failures for consumed incidence and flowsheet analyses.
use std::collections::BTreeSet;

use pse_ids::SemanticId;

/// Source coverage, independent of the operation's requested output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Every admitted node and edge for the named model.
    Whole(SemanticId),
    /// A region extracted from a complete projection after cross-edge checks.
    Independent {
        /// Parent complete model identity.
        model: SemanticId,
        /// Members proved to have no crossing edge.
        members: BTreeSet<SemanticId>,
    },
    /// Complete algebraic selection conditional on explicitly held inputs.
    Conditional {
        /// Parent case identity.
        model: SemanticId,
        /// Selected equality rows.
        rows: BTreeSet<SemanticId>,
        /// Selected free coordinates.
        columns: BTreeSet<SemanticId>,
        /// External coordinates held fixed for this analysis.
        inputs: BTreeSet<SemanticId>,
    },
    /// An inspection-only subset, ineligible for complete analysis.
    Partial(SemanticId),
}
impl Scope {
    /// Versioned semantic scope identity, independent of traversal order.
    pub fn key(&self) -> pse_ids::ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::StructuralScopeV1);
        match self {
            Self::Whole(id) => {
                h.u64(0).id(id);
            }
            Self::Partial(id) => {
                h.u64(1).id(id);
            }
            Self::Independent { model, members } => {
                h.u64(2).id(model).u64(members.len() as u64);
                for id in members {
                    h.id(id);
                }
            }
            Self::Conditional {
                model,
                rows,
                columns,
                inputs,
            } => {
                h.u64(3).id(model);
                for group in [rows, columns, inputs] {
                    h.u64(group.len() as u64);
                    for id in group {
                        h.id(id);
                    }
                }
            }
        }
        h.finish_hash()
    }
}
/// Finite admission bounds, checked before graph allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    /// Maximum nodes, including isolated vertices.
    pub nodes: usize,
    /// Maximum edges, including parallel edges.
    pub edges: usize,
}
impl GraphLimits {
    /// Check arithmetic and the index sentinel without allocating a graph.
    /// # Errors
    /// A configured bound or the u32 index space would be exceeded.
    pub fn check(self, nodes: usize, edges: usize) -> Result<(), ProjectionError> {
        if nodes > self.nodes
            || edges > self.edges
            || nodes >= u32::MAX as usize
            || edges >= u32::MAX as usize
        {
            return Err(ProjectionError::Limit);
        }
        Ok(())
    }
}
/// A graph admission or complete-analysis failure.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProjectionError {
    /// Cooperative structural cancellation, distinct from incomplete input.
    #[error("structural analysis cancelled")]
    Cancelled,
    /// A configured or library cardinality limit.
    #[error("graph cardinality exceeds its configured or u32 index limit")]
    Limit,
    /// Duplicate semantic identity.
    #[error("duplicate graph identity {0}")]
    Duplicate(SemanticId),
    /// An edge or request names an absent vertex.
    #[error("graph vertex {0} is absent")]
    Missing(SemanticId),
    /// Complete analysis cannot follow incomplete source admission.
    #[error("partial graph cannot establish a whole-region result")]
    Partial,
    /// The requested region has a crossing dependency.
    #[error("region is not independent; crossing edge {0}")]
    Crossing(SemanticId),
    /// Actual semantic edge IDs form a cycle witness.
    #[error("graph contains a directed cycle")]
    Cycle(Vec<SemanticId>),
    /// A directed cycle whose every connection is forbidden to tear.
    #[error("forbidden tear cycle: connections={connections:?}, decisions={decisions:?}")]
    ForbiddenTearCycle {
        /// Actual cycle connection occurrences.
        connections: Vec<SemanticId>,
        /// Source policy decisions preventing removal.
        decisions: Vec<SemanticId>,
    },
    /// A requested decision set conflicts with source policy.
    #[error("tear selection violates decisions {0:?}")]
    TearPolicy(Vec<SemanticId>),
    /// A domain projection violated its declared meaning.
    #[error("invalid graph projection: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    ProjectionError,
    code(this) { Some(match this { Self::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled, Self::Limit => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit, _ => pse_diagnostics::DiagnosticCode::ValidationInvariant }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
