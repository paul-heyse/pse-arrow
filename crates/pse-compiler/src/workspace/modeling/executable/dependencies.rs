// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current-identity dependency access over model-owned resolved read sets.
use super::*;
use petgraph::{
    graph::NodeIndex,
    visit::{Bfs, VisitMap, Visitable},
};

#[derive(Clone, Copy)]
enum EdgeRole {
    Numerical,
    Guard,
}

/// Edges point from an input/member to its consumers. Free inputs and rates are
/// nodes even when they have no defining expression. Validity edges are separate
/// from the numerical expression graph and never create an arithmetic cycle.
pub(super) struct Dependencies {
    graph: DiGraph<SemanticId, EdgeRole>,
    nodes: BTreeMap<SemanticId, NodeIndex>,
}
impl Dependencies {
    pub(super) fn new(ids: impl IntoIterator<Item = SemanticId>) -> Self {
        let mut graph = DiGraph::new();
        let nodes = ids.into_iter().map(|id| (id, graph.add_node(id))).collect();
        Self { graph, nodes }
    }
    fn node(&mut self, id: SemanticId) -> NodeIndex {
        *self
            .nodes
            .entry(id)
            .or_insert_with(|| self.graph.add_node(id))
    }
    pub(super) fn numerical(
        &mut self,
        consumer: SemanticId,
        reads: impl IntoIterator<Item = SemanticId>,
    ) {
        let consumer = self.node(consumer);
        for id in reads {
            let dependency = self.node(id);
            self.graph
                .add_edge(dependency, consumer, EdgeRole::Numerical);
        }
    }
    pub(super) fn guard(
        &mut self,
        consumer: SemanticId,
        reads: impl IntoIterator<Item = SemanticId>,
    ) {
        let consumer = self.node(consumer);
        for id in reads {
            let dependency = self.node(id);
            self.graph.add_edge(dependency, consumer, EdgeRole::Guard);
        }
    }
    /// Source/control expression references are included in topological order;
    /// externally declared validity guards belong only to selected read closure.
    pub(super) fn order(&self, members: &BTreeSet<SemanticId>) -> Result<Vec<SemanticId>> {
        let graph = self.graph.filter_map(
            |_, id| members.contains(id).then_some(*id),
            |_, role| matches!(role, EdgeRole::Numerical).then_some(()),
        );
        toposort(&graph, None)
            .map(|order| order.into_iter().map(|node| graph[node]).collect())
            .map_err(|_| CompileError::Missing("cyclic expression members".into()))
    }
    pub(super) fn consumers(
        &self,
        roots: impl IntoIterator<Item = SemanticId>,
        db: &dyn CompilerDb,
    ) -> BTreeSet<SemanticId> {
        let mut reached = BTreeSet::new();
        let mut bfs = Bfs {
            stack: Default::default(),
            discovered: self.graph.visit_map(),
        };
        for root in roots
            .into_iter()
            .filter_map(|id| self.nodes.get(&id).copied())
        {
            if bfs.discovered.visit(root) {
                bfs.stack.push_back(root);
            }
        }
        while let Some(node) = bfs.next(&self.graph) {
            checkpoint(db);
            reached.insert(self.graph[node]);
        }
        reached
    }
}
