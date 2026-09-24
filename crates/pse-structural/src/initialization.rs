// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conditional initialization blocks derived from complete library DM/BTF results.
use crate::{
    incidence::{Part, StructuralAnalysis},
    projection::{ProjectionError, Scope},
};
use pse_ids::SemanticId;
use std::collections::{BTreeMap, BTreeSet};
/// A conditional equality block, not an independent optimization problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// Original rows and variables solved together.
    pub members: Part,
    /// Explicit predecessor variable inputs.
    pub inputs: Vec<SemanticId>,
}
/// Predecessor-first complete square-system initialization plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Complete selected scope inherited from incidence admission.
    pub scope: Scope,
    /// Library-topologically ordered conditional blocks.
    pub blocks: Vec<Block>,
}
impl Plan {
    /// Structural deficiency is refused before attempting numerical factorization.
    pub fn from_analysis(a: &StructuralAnalysis) -> Result<Self, ProjectionError> {
        if matches!(a.scope, Scope::Partial(_))
            || !a.over.rows.is_empty()
            || !a.over.columns.is_empty()
            || !a.under.rows.is_empty()
            || !a.under.columns.is_empty()
        {
            return Err(ProjectionError::Invalid(format!(
                "initialization is structurally deficient: over={:?}, under={:?}",
                a.over, a.under
            )));
        }
        let mut columns = BTreeMap::new();
        let mut rows = BTreeMap::new();
        let mut graph = petgraph::Graph::<usize, ()>::new();
        let mut nodes = Vec::new();
        for (b, block) in a.blocks.iter().enumerate() {
            if block.members.rows.len() != block.members.columns.len()
                || block.members.rows.is_empty()
            {
                return Err(ProjectionError::Invalid(
                    "non-square initialization block".into(),
                ));
            }
            nodes.push(graph.add_node(b));
            for c in &block.members.columns {
                if columns.insert(*c, b).is_some() {
                    return Err(ProjectionError::Invalid("duplicate block variable".into()));
                }
            }
            for r in &block.members.rows {
                if rows.insert(*r, b).is_some() {
                    return Err(ProjectionError::Invalid("duplicate block row".into()));
                }
            }
        }
        if columns.keys().copied().collect::<BTreeSet<_>>()
            != a.square.columns.iter().copied().collect()
            || rows.keys().copied().collect::<BTreeSet<_>>()
                != a.square.rows.iter().copied().collect()
        {
            return Err(ProjectionError::Invalid(
                "incomplete initialization blocks".into(),
            ));
        }
        let mut inputs = vec![BTreeSet::new(); a.blocks.len()];
        let mut edges = BTreeSet::new();
        for edge in &a.contributions {
            let Some(&consumer) = rows.get(&edge.row) else {
                continue;
            };
            let &producer = columns
                .get(&edge.column)
                .ok_or(ProjectionError::Missing(edge.column))?;
            if consumer != producer {
                inputs[consumer].insert(edge.column);
                if edges.insert((producer, consumer)) {
                    graph.add_edge(nodes[producer], nodes[consumer], ());
                }
            }
        }
        let order = petgraph::algo::toposort(&graph, None).map_err(|_| {
            ProjectionError::Invalid("cyclic dependency between purported BTF blocks".into())
        })?;
        Ok(Self {
            scope: a.scope.clone(),
            blocks: order
                .into_iter()
                .map(|node| {
                    let b = graph[node];
                    Block {
                        members: a.blocks[b].members.clone(),
                        inputs: inputs[b].iter().copied().collect(),
                    }
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incidence::{CaseIncidence, Constraint, Incidence};
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn conditional_schedule_keeps_predecessors_and_refuses_deficiency() {
        let rows = (1..=2)
            .map(|n| Constraint {
                id: id(n),
                lower: Some(0.0),
                upper: Some(0.0),
            })
            .collect();
        let edges = [(1, 11), (2, 11), (2, 12)]
            .into_iter()
            .map(|(r, c)| Incidence {
                row: id(r),
                column: id(c),
                instance: id(30),
                output: r as usize,
            })
            .collect();
        let a = CaseIncidence::new(
            Scope::Whole(id(40)),
            rows,
            vec![id(11), id(12)],
            edges,
            BTreeSet::new(),
            crate::projection::GraphLimits {
                nodes: 10,
                edges: 10,
            },
        )
        .unwrap()
        .analyze(&std::sync::atomic::AtomicBool::new(false))
        .unwrap();
        let plan = Plan::from_analysis(&a).unwrap();
        assert_eq!(plan.blocks.len(), 2);
        assert_eq!(plan.blocks[0].members.columns, vec![id(11)]);
        assert_eq!(plan.blocks[1].inputs, vec![id(11)]);
        let mut bad = a.clone();
        bad.under.columns.push(id(13));
        assert!(Plan::from_analysis(&bad).is_err());
        let mut partial = a;
        partial.scope = Scope::Partial(id(40));
        assert!(Plan::from_analysis(&partial).is_err());
    }
}
