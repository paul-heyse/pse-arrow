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
    /// Stable identity inherited from the structural owner.
    pub id: crate::incidence::BlockId,
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
    /// Complete an unchanged numerical matching/BTF witness with execution and
    /// validity dependencies. Control cycles merge square numerical blocks; they
    /// never supply a matching edge or establish numerical regularity.
    pub fn with_execution_dependencies(
        a: &StructuralAnalysis,
        dependencies: &[(SemanticId, SemanticId)],
        cancel: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, ProjectionError> {
        let numerical = Self::from_analysis(a)?;
        let rows: BTreeMap<_, _> = numerical
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(index, block)| block.members.rows.iter().map(move |id| (*id, index)))
            .collect();
        let columns: BTreeMap<_, _> = numerical
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(index, block)| block.members.columns.iter().map(move |id| (*id, index)))
            .collect();
        let mut graph = petgraph::graph::DiGraph::<(), ()>::new();
        let nodes: Vec<_> = numerical
            .blocks
            .iter()
            .map(|_| graph.add_node(()))
            .collect();
        let edges: BTreeSet<_> = a
            .contributions
            .iter()
            .map(|e| (e.row, e.column))
            .chain(dependencies.iter().copied())
            .collect();
        for (row, column) in &edges {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(ProjectionError::Cancelled);
            }
            let Some(&consumer) = rows.get(row) else {
                continue;
            };
            if let Some(&producer) = columns.get(column)
                && producer != consumer
            {
                graph.update_edge(nodes[producer], nodes[consumer], ());
            }
        }
        let mut components = petgraph::algo::kosaraju_scc(&graph);
        components.reverse();
        let mut blocks = Vec::new();
        for component in components {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(ProjectionError::Cancelled);
            }
            let mut members = Part::default();
            for node in component {
                let block = &numerical.blocks[node.index()];
                members.rows.extend_from_slice(&block.members.rows);
                members.columns.extend_from_slice(&block.members.columns);
            }
            members.rows.sort_unstable();
            members.columns.sort_unstable();
            let inputs = edges
                .iter()
                .filter(|(row, _)| members.rows.binary_search(row).is_ok())
                .map(|(_, column)| *column)
                .filter(|column| members.columns.binary_search(column).is_err())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            blocks.push(Block {
                id: crate::incidence::BlockId::new(&a.scope, &members),
                members,
                inputs,
            });
        }
        Ok(Self {
            scope: a.scope.clone(),
            blocks,
        })
    }
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
        for (b, block) in a.blocks.iter().enumerate() {
            if block.members.rows.len() != block.members.columns.len()
                || block.members.rows.is_empty()
            {
                return Err(ProjectionError::Invalid(
                    "non-square initialization block".into(),
                ));
            }
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
        for edge in &a.contributions {
            let Some(&consumer) = rows.get(&edge.row) else {
                continue;
            };
            let &producer = columns
                .get(&edge.column)
                .ok_or(ProjectionError::Missing(edge.column))?;
            if consumer != producer {
                inputs[consumer].insert(edge.column);
                if producer >= consumer {
                    return Err(ProjectionError::Invalid(
                        "structural block order violates a dependency".into(),
                    ));
                }
            }
        }
        Ok(Self {
            scope: a.scope.clone(),
            blocks: a
                .blocks
                .iter()
                .enumerate()
                .map(|(b, block)| Block {
                    id: block.id,
                    members: block.members.clone(),
                    inputs: inputs[b].iter().copied().collect(),
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
    fn execution_cycles_merge_numerical_blocks_and_fixed_inputs_remain_explicit() {
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let analysis = CaseIncidence::new(
            Scope::Whole(id(40)),
            vec![
                Constraint {
                    id: id(1),
                    lower: Some(0.0),
                    upper: Some(0.0),
                },
                Constraint {
                    id: id(2),
                    lower: Some(0.0),
                    upper: Some(0.0),
                },
            ],
            vec![id(11), id(12)],
            vec![
                Incidence {
                    row: id(1),
                    column: id(11),
                    instance: id(30),
                    output: 0,
                },
                Incidence {
                    row: id(2),
                    column: id(12),
                    instance: id(30),
                    output: 1,
                },
            ],
            BTreeSet::new(),
            crate::projection::GraphLimits {
                nodes: 100,
                edges: 100,
            },
        )
        .unwrap()
        .analyze(&cancel)
        .unwrap();
        assert_eq!(analysis.blocks.len(), 2);
        let plan = Plan::with_execution_dependencies(
            &analysis,
            &[(id(1), id(12)), (id(2), id(11)), (id(1), id(90))],
            &cancel,
        )
        .unwrap();
        assert_eq!(plan.blocks.len(), 1);
        assert_eq!(plan.blocks[0].members.rows, [id(1), id(2)]);
        assert_eq!(plan.blocks[0].members.columns, [id(11), id(12)]);
        assert_eq!(plan.blocks[0].inputs, [id(90)]);
        assert_eq!(analysis.blocks.len(), 2); // No invented numerical matching/rank.
    }
    #[test]
    fn structural_identity_preserves_independent_block_order() {
        use crate::incidence::{Block as StructuralBlock, BlockId, StructuralAnalysis};
        let scope = Scope::Whole(id(40));
        let parts = [
            Part {
                rows: vec![id(2)],
                columns: vec![id(12)],
            },
            Part {
                rows: vec![id(1)],
                columns: vec![id(11)],
            },
        ];
        let blocks = parts
            .iter()
            .map(|members| StructuralBlock {
                id: BlockId::new(&scope, members),
                members: members.clone(),
                coupling: pounce_presolve::coupling::AuxiliaryCouplingClass::PureEquality,
            })
            .collect();
        let a = StructuralAnalysis {
            scope,
            provenance: crate::incidence::PROVENANCE,
            matching: vec![],
            over: Part::default(),
            under: Part::default(),
            square: Part {
                rows: vec![id(1), id(2)],
                columns: vec![id(11), id(12)],
            },
            blocks,
            contributions: vec![],
        };
        let first = Plan::from_analysis(&a).unwrap();
        assert_eq!(first.blocks[0].id, a.blocks[0].id);
        assert_eq!(first.blocks[0].members, parts[0]);
        let mut reordered = a.clone();
        reordered.blocks.reverse();
        let second = Plan::from_analysis(&reordered).unwrap();
        assert_eq!(second.blocks[1].id, first.blocks[0].id);
        reordered.contributions.push(Incidence {
            row: id(1),
            column: id(12),
            instance: id(30),
            output: 0,
        });
        assert!(Plan::from_analysis(&reordered).is_err());
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
