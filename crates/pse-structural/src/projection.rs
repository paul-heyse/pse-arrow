// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete immutable projections; local indices never cross this boundary.
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use petgraph::{
    Directed, Graph,
    graph::NodeIndex,
    visit::{Dfs, EdgeRef, Reversed},
};
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
        let mut h = pse_ids::FramedHasher::new("pse.structural.scope.v1");
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
/// A direct semantic edge; identity preserves multiplicity and source correspondence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    /// Stable edge/source identity.
    pub id: SemanticId,
    /// Prerequisite (or declared parent).
    pub from: SemanticId,
    /// Dependent (or declared child).
    pub to: SemanticId,
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

/// Canonical semantic projection; graph indices and allocation are derived state.
#[derive(Clone, Debug)]
pub struct Projection {
    scope: Scope,
    graph: Graph<SemanticId, SemanticId, Directed, u32>,
    index: BTreeMap<SemanticId, NodeIndex<u32>>,
    edges: Vec<Dependency>,
}
impl PartialEq for Projection {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope
            && self.index.keys().eq(other.index.keys())
            && self.edges == other.edges
    }
}
impl Eq for Projection {}
impl Projection {
    /// Admit a complete direct-edge inventory without dropping isolates or parallel edges.
    /// # Errors
    /// Duplicate IDs, dangling edges or exceeded bounds.
    pub fn admit(
        scope: Scope,
        nodes: Vec<SemanticId>,
        edges: Vec<Dependency>,
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        if matches!(scope, Scope::Independent { .. }) {
            return Err(ProjectionError::Invalid(
                "independent scope requires parent proof".into(),
            ));
        }
        Self::build(scope, nodes, edges, limits)
    }
    fn build(
        scope: Scope,
        mut nodes: Vec<SemanticId>,
        mut edges: Vec<Dependency>,
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        limits.check(nodes.len(), edges.len())?;
        nodes.sort_unstable();
        if let Some(pair) = nodes.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(ProjectionError::Duplicate(pair[0]));
        }
        edges.sort_unstable_by_key(|edge| (edge.from, edge.to, edge.id));
        let mut identities = BTreeSet::new();
        let mut graph = Graph::with_capacity(nodes.len(), edges.len());
        let mut index = BTreeMap::new();
        for node in nodes {
            index.insert(
                node,
                graph
                    .try_add_node(node)
                    .map_err(|_| ProjectionError::Limit)?,
            );
        }
        for edge in &edges {
            if !identities.insert(edge.id) {
                return Err(ProjectionError::Duplicate(edge.id));
            }
            let from = *index
                .get(&edge.from)
                .ok_or(ProjectionError::Missing(edge.from))?;
            let to = *index
                .get(&edge.to)
                .ok_or(ProjectionError::Missing(edge.to))?;
            graph
                .try_add_edge(from, to, edge.id)
                .map_err(|_| ProjectionError::Limit)?;
        }
        if let Scope::Independent { members, .. } = &scope
            && !members.iter().eq(index.keys())
        {
            return Err(ProjectionError::Invalid(
                "independent scope membership differs".into(),
            ));
        }
        Ok(Self {
            scope,
            graph,
            index,
            edges,
        })
    }
    /// Exact admitted source coverage.
    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    /// Original edges, including all provenance and multiplicity.
    pub fn edges(&self) -> &[Dependency] {
        &self.edges
    }
    fn complete(&self) -> Result<(), ProjectionError> {
        if matches!(self.scope, Scope::Partial(_)) {
            Err(ProjectionError::Partial)
        } else {
            Ok(())
        }
    }
    /// Iterative strong connectivity, with canonical members and component order.
    /// # Errors
    /// The admitted source is only an inspection subset.
    pub fn components(&self) -> Result<Vec<Vec<SemanticId>>, ProjectionError> {
        self.complete()?;
        let mut groups = petgraph::algo::kosaraju_scc(&self.graph)
            .into_iter()
            .map(|group| {
                let mut members = group
                    .into_iter()
                    .map(|node| self.graph[node])
                    .collect::<Vec<_>>();
                members.sort_unstable();
                members
            })
            .collect::<Vec<_>>();
        groups.sort_unstable();
        Ok(groups)
    }
    /// Deterministic prerequisite order using a semantic-ID ready key.
    /// # Errors
    /// Partial coverage or a directed cycle (with actual edge provenance).
    pub fn order(&self) -> Result<Vec<SemanticId>, ProjectionError> {
        self.complete()?;
        let order = rustworkx_core::dag_algo::lexicographical_topological_sort(
            &self.graph,
            |node| Ok::<_, Infallible>(self.graph[node]),
            false,
            None,
        )
        .map_err(|_| ProjectionError::Invalid("semantic topological key failed".into()))?;
        // rustworkx 0.18.1 returns a partial order for cyclic input, not a cycle error.
        if order.len() != self.graph.node_count() {
            return Err(ProjectionError::Cycle(self.cycle_witness()?));
        }
        Ok(order.into_iter().map(|node| self.graph[node]).collect())
    }
    fn cycle_witness(&self) -> Result<Vec<SemanticId>, ProjectionError> {
        let mut components = petgraph::algo::kosaraju_scc(&self.graph);
        for group in &mut components {
            group.sort_unstable_by_key(|node| self.graph[*node]);
        }
        components.sort_unstable_by_key(|group| group.first().map(|node| self.graph[*node]));
        let source = components
            .iter()
            .find(|group| {
                group.len() > 1
                    || group
                        .first()
                        .is_some_and(|node| self.graph.contains_edge(*node, *node))
            })
            .and_then(|group| group.first())
            .copied();
        let Some(source) = source else {
            return Err(ProjectionError::Invalid(
                "incomplete topological order has no cyclic SCC".into(),
            ));
        };
        let pairs = rustworkx_core::connectivity::find_cycle(&self.graph, Some(source));
        let mut witness = pairs
            .into_iter()
            .map(|(from, to)| {
                self.graph
                    .edges_connecting(from, to)
                    .map(|edge| *edge.weight())
                    .min()
                    .ok_or_else(|| {
                        ProjectionError::Invalid("cycle witness names a missing edge".into())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some((position, _)) = witness.iter().enumerate().min_by_key(|(_, id)| **id) {
            witness.rotate_left(position);
        }
        if witness.is_empty() {
            return Err(ProjectionError::Invalid("empty cycle witness".into()));
        }
        Ok(witness)
    }
    /// Requested reachability using the library's iterative traversal.
    /// # Errors
    /// The start node is absent. Partial scope remains visible in `scope()`.
    pub fn reachable(
        &self,
        start: SemanticId,
        ancestors: bool,
    ) -> Result<Vec<SemanticId>, ProjectionError> {
        let start = *self
            .index
            .get(&start)
            .ok_or(ProjectionError::Missing(start))?;
        let mut values = Vec::new();
        if ancestors {
            let graph = Reversed(&self.graph);
            let mut dfs = Dfs::new(graph, start);
            while let Some(node) = dfs.next(graph) {
                values.push(self.graph[node]);
            }
        } else {
            let mut dfs = Dfs::new(&self.graph, start);
            while let Some(node) = dfs.next(&self.graph) {
                values.push(self.graph[node]);
            }
        }
        values.sort_unstable();
        Ok(values)
    }
    /// Prove independence against this complete graph before region extraction.
    /// # Errors
    /// Partial parent, absent member, or a crossing edge.
    pub fn independent_region(
        &self,
        members: BTreeSet<SemanticId>,
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        self.complete()?;
        if matches!(self.scope, Scope::Conditional { .. }) {
            return Err(ProjectionError::Partial);
        }
        for member in &members {
            if !self.index.contains_key(member) {
                return Err(ProjectionError::Missing(*member));
            }
        }
        for edge in &self.edges {
            if members.contains(&edge.from) != members.contains(&edge.to) {
                return Err(ProjectionError::Crossing(edge.id));
            }
        }
        let model = match &self.scope {
            Scope::Whole(model)
            | Scope::Independent { model, .. }
            | Scope::Conditional { model, .. }
            | Scope::Partial(model) => *model,
        };
        let edges = self
            .edges
            .iter()
            .filter(|edge| members.contains(&edge.from))
            .cloned()
            .collect();
        Self::build(
            Scope::Independent {
                model,
                members: members.clone(),
            },
            members.into_iter().collect(),
            edges,
            limits,
        )
    }
    /// Presence-only SCC quotient. Original typed/provenance edges remain in this projection.
    /// # Errors
    /// Partial coverage or insufficient additional workspace admission.
    pub fn condensation(&self, limits: GraphLimits) -> Result<Condensation, ProjectionError> {
        self.complete()?;
        limits.check(self.graph.node_count(), self.graph.edge_count())?;
        let lightweight = self.graph.map(|_, node| *node, |_, _| ());
        let condensed = petgraph::algo::condensation(lightweight, true);
        let mut edges = condensed
            .edge_references()
            .map(|edge| {
                let mut from = condensed[edge.source()].clone();
                let mut to = condensed[edge.target()].clone();
                from.sort_unstable();
                to.sort_unstable();
                (from, to)
            })
            .collect::<Vec<_>>();
        edges.sort_unstable();
        let mut components = condensed.node_weights().cloned().collect::<Vec<_>>();
        for component in &mut components {
            component.sort_unstable();
        }
        components.sort_unstable();
        Ok(Condensation { components, edges })
    }
}
/// Canonical SCC quotient, including isolated components. Original edge labels stay in the parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Condensation {
    /// Every component, including isolated vertices.
    pub components: Vec<Vec<SemanticId>>,
    /// Presence-only prerequisite links between components.
    pub edges: Vec<(Vec<SemanticId>, Vec<SemanticId>)>,
}

/// Conservative application admission extent, including the graph, SCC quotient, schedule
/// traversal and retained semantic maps.
/// This is an allocation allowance, not a measurement of allocator RSS.
/// # Errors
/// Arithmetic or library index bounds overflow.
pub fn workspace_extent(nodes: usize, edges: usize) -> Result<usize, ProjectionError> {
    GraphLimits {
        nodes: usize::MAX,
        edges: usize::MAX,
    }
    .check(nodes, edges)?;
    nodes
        .checked_mul(2048)
        .and_then(|bytes| {
            edges
                .checked_mul(512)
                .and_then(|edge_bytes| bytes.checked_add(edge_bytes))
        })
        .ok_or(ProjectionError::Limit)
}

#[cfg(test)]
mod foundation_unit {
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    const LIMITS: GraphLimits = GraphLimits {
        nodes: 100,
        edges: 100,
    };
    fn edge(n: u8, from: u8, to: u8) -> Dependency {
        Dependency {
            id: id(n),
            from: id(from),
            to: id(to),
        }
    }
    #[test]
    fn direction_ties_isolates_parallel_edges_and_cycles() {
        let graph = Projection::admit(
            Scope::Whole(id(0)),
            vec![id(3), id(2), id(1)],
            vec![edge(11, 1, 2), edge(10, 1, 2)],
            LIMITS,
        )
        .unwrap();
        assert_eq!(graph.order().unwrap(), vec![id(1), id(2), id(3)]);
        assert_eq!(graph.reachable(id(2), true).unwrap(), vec![id(1), id(2)]);
        assert_eq!(graph.edges().len(), 2);
        let cycle = Projection::admit(
            Scope::Whole(id(0)),
            vec![id(1), id(2)],
            vec![edge(11, 1, 2), edge(10, 2, 1)],
            LIMITS,
        )
        .unwrap();
        assert_eq!(cycle.components().unwrap(), vec![vec![id(1), id(2)]]);
        assert_eq!(
            cycle.order(),
            Err(ProjectionError::Cycle(vec![id(10), id(11)]))
        );
    }
    #[test]
    fn refuses_partial_crossing_dangling_and_index_overflow() {
        let graph = Projection::admit(Scope::Partial(id(0)), vec![id(1)], vec![], LIMITS).unwrap();
        assert_eq!(graph.order(), Err(ProjectionError::Partial));
        assert!(
            Projection::admit(
                Scope::Whole(id(0)),
                vec![id(1)],
                vec![edge(10, 1, 2)],
                LIMITS
            )
            .is_err()
        );
        assert!(
            GraphLimits {
                nodes: usize::MAX,
                edges: usize::MAX
            }
            .check(u32::MAX as usize, 0)
            .is_err()
        );
        let graph = Projection::admit(
            Scope::Whole(id(0)),
            vec![id(1), id(2)],
            vec![edge(10, 1, 2)],
            LIMITS,
        )
        .unwrap();
        assert!(matches!(
            graph.independent_region(BTreeSet::from([id(1)]), LIMITS),
            Err(ProjectionError::Crossing(_))
        ));
    }
    #[test]
    fn partial_topological_output_self_loops_and_diamond_regions_are_checked() {
        let graph = Projection::admit(
            Scope::Whole(id(0)),
            vec![id(1), id(2), id(3)],
            vec![edge(10, 2, 2)],
            LIMITS,
        )
        .unwrap();
        assert_eq!(graph.order(), Err(ProjectionError::Cycle(vec![id(10)])));
        assert_eq!(graph.condensation(LIMITS).unwrap().components.len(), 3);
        let diamond = Projection::admit(
            Scope::Whole(id(0)),
            vec![id(1), id(2), id(3), id(4), id(5)],
            vec![
                edge(10, 1, 2),
                edge(11, 1, 3),
                edge(12, 2, 4),
                edge(13, 3, 4),
            ],
            LIMITS,
        )
        .unwrap();
        assert_eq!(
            diamond.reachable(id(4), true).unwrap(),
            vec![id(1), id(2), id(3), id(4)]
        );
        assert_eq!(
            diamond
                .independent_region(BTreeSet::from([id(5)]), LIMITS)
                .unwrap()
                .order()
                .unwrap(),
            vec![id(5)]
        );
        assert!(
            Projection::admit(
                Scope::Independent {
                    model: id(0),
                    members: BTreeSet::from([id(1)])
                },
                vec![id(1)],
                vec![],
                LIMITS
            )
            .is_err()
        );
    }
}
