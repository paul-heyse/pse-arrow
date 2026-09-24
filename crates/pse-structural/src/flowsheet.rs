// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete physical flowsheet projections with connection-occurrence tear decisions.
use crate::projection::{GraphLimits, ProjectionError};
use petgraph::{Directed, Graph};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use std::collections::{BTreeMap, BTreeSet};
/// One process unit and its complete declared physical port inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// Stable unit identity, including isolated units.
    pub id: SemanticId,
    /// Declared scalar port contracts.
    pub ports: Vec<pse_kernels::Port>,
}
/// Explicit connection occurrence; parallel occurrences are never merged by endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    /// Stable connection occurrence identity.
    pub id: SemanticId,
    /// Upstream unit.
    pub from: SemanticId,
    /// Downstream unit.
    pub to: SemanticId,
    /// Explicit joint tear-decision identity.
    pub decision: SemanticId,
    /// Source-port/destination-port physical bindings.
    pub bindings: Vec<(SemanticId, SemanticId)>,
}
/// User constraint on one explicit decision group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Native optimizer may select it.
    Free,
    /// Every occurrence in this group is removed.
    Mandatory,
    /// This group cannot be selected.
    Forbidden,
}
/// One decision may group several occurrences, but grouping is always declared.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    /// Stable decision identity.
    pub id: SemanticId,
    /// Finite nonnegative authored weight.
    pub cost: f64,
    /// Mandatory/forbidden/free selection.
    pub policy: Policy,
}
/// Complete source declaration, safe to store in the semantic compiler.
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    /// Every process node, including isolates.
    pub nodes: Vec<Node>,
    /// Every connection occurrence, including parallel edges.
    pub connections: Vec<Connection>,
    /// Explicit group decisions.
    pub decisions: Vec<Decision>,
}
/// One physically checked scalar connection assignment.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    /// Source coordinate.
    pub source: SemanticId,
    /// Destination coordinate.
    pub target: SemanticId,
    /// Resolved representation conversion; applied only at the connection boundary.
    pub conversion: pse_quantity::UnitConvertSpec,
    /// Complete physical contract retained for semantic equality.
    pub quantity: pse_quantity::QuantityType,
}
/// Physically admitted canonical flow graph; petgraph owns graph algorithms.
#[derive(Clone, Debug)]
pub struct FlowGraph {
    declaration: Declaration,
    graph: Graph<SemanticId, usize, Directed>,
    bindings: BTreeMap<SemanticId, Vec<Binding>>,
    key: ContentHash,
}
impl PartialEq for FlowGraph {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
            && self.bindings == other.bindings
            && self.key == other.key
    }
}
impl FlowGraph {
    /// Canonicalize complete inventories and check physical bindings before projection.
    pub fn admit(
        mut d: Declaration,
        registry: &pse_quantity::QuantityRegistry,
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        limits.check(d.nodes.len(), d.connections.len())?;
        d.nodes.sort_by_key(|n| n.id);
        d.connections.sort_by_key(|e| e.id);
        d.decisions.sort_by_key(|g| g.id);
        if d.nodes.windows(2).any(|w| w[0].id == w[1].id)
            || d.connections.windows(2).any(|w| w[0].id == w[1].id)
            || d.decisions.windows(2).any(|w| w[0].id == w[1].id)
            || d.decisions
                .iter()
                .any(|g| !g.cost.is_finite() || g.cost < 0.0)
        {
            return Err(ProjectionError::Invalid(
                "duplicate flow identities or invalid tear weights".into(),
            ));
        }
        let mut ports = BTreeMap::new();
        let mut graph = Graph::new();
        let mut nodes = BTreeMap::new();
        let mut h = FramedHasher::new("pse.flow.projection.v1");
        for n in &mut d.nodes {
            n.ports.sort_by_key(|p| p.id);
            nodes.insert(n.id, graph.add_node(n.id));
            h.id(&n.id);
            for p in &n.ports {
                if ports.insert(p.id, (n.id, p)).is_some() {
                    return Err(ProjectionError::Invalid("duplicate flow port".into()));
                }
                h.id(&p.id).id(&p.quantity.as_id()).id(&p.unit.as_id());
            }
        }
        let groups: BTreeSet<_> = d.decisions.iter().map(|g| g.id).collect();
        let mut used = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        let mut bindings = BTreeMap::new();
        for (i, e) in d.connections.iter_mut().enumerate() {
            let from = *nodes.get(&e.from).ok_or(ProjectionError::Missing(e.from))?;
            let to = *nodes.get(&e.to).ok_or(ProjectionError::Missing(e.to))?;
            if !groups.contains(&e.decision) {
                return Err(ProjectionError::Missing(e.decision));
            }
            used.insert(e.decision);
            e.bindings.sort();
            if e.bindings.is_empty() {
                return Err(ProjectionError::Invalid(
                    "connection has no physical bindings".into(),
                ));
            }
            h.id(&e.id).id(&e.from).id(&e.to).id(&e.decision);
            for (source, target) in &e.bindings {
                let (source_node, source_port) =
                    ports.get(source).ok_or(ProjectionError::Missing(*source))?;
                let (target_node, target_port) =
                    ports.get(target).ok_or(ProjectionError::Missing(*target))?;
                if *source_node != e.from || *target_node != e.to || !destinations.insert(*target) {
                    return Err(ProjectionError::Invalid(
                        "port ownership or multiple connection assignments".into(),
                    ));
                }
                let check = || -> Result<Binding, pse_quantity::QuantityError> {
                    pse_quantity::admission::require_same_contract(
                        target_port.quantity,
                        source_port.quantity,
                        registry,
                    )?;
                    let ty = registry.quantity_type(source_port.quantity)?;
                    let conversion = pse_quantity::convert_spec_for_type(
                        registry.unit(source_port.unit)?,
                        registry.unit(target_port.unit)?,
                        &ty.key,
                    )?;
                    Ok(Binding {
                        source: *source,
                        target: *target,
                        conversion,
                        quantity: ty.clone(),
                    })
                };
                let binding = check().map_err(|e| {
                    ProjectionError::Invalid(format!("flow binding {source}->{target}: {e}"))
                })?;
                h.id(source)
                    .id(target)
                    .u64(binding.conversion.scale.to_bits())
                    .u64(binding.conversion.offset.to_bits());
                bindings.entry(e.id).or_insert_with(Vec::new).push(binding);
            }
            graph.add_edge(from, to, i);
        }
        if used != groups {
            return Err(ProjectionError::Invalid("unused tear decision".into()));
        }
        for g in &d.decisions {
            h.id(&g.id).u64(g.cost.to_bits()).u64(g.policy as u64);
        }
        Ok(Self {
            declaration: d,
            graph,
            bindings,
            key: h.finish_hash(),
        })
    }
    /// Complete admitted source declarations in canonical order.
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    /// Canonical physical assignments for each connection occurrence.
    pub fn bindings(&self) -> &BTreeMap<SemanticId, Vec<Binding>> {
        &self.bindings
    }
    /// Semantic projection identity, independent of native solver-local indices.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Independent residual-DAG witness for a proposed decision set.
    pub fn witness(
        &self,
        selected: &BTreeSet<SemanticId>,
    ) -> Result<Vec<SemanticId>, ProjectionError> {
        if selected.iter().any(|id| {
            self.declaration
                .decisions
                .binary_search_by_key(id, |g| g.id)
                .is_err()
        }) || self.declaration.decisions.iter().any(|g| {
            g.policy == Policy::Mandatory && !selected.contains(&g.id)
                || g.policy == Policy::Forbidden && selected.contains(&g.id)
        }) {
            return Err(ProjectionError::Invalid(
                "tear decision violates declared policy".into(),
            ));
        }
        let mut graph = self.graph.clone();
        graph.retain_edges(|g, e| !selected.contains(&self.declaration.connections[g[e]].decision));
        let order = petgraph::algo::toposort(&graph, None)
            .map_err(|_| ProjectionError::Invalid("selected tears leave a cycle".into()))?;
        Ok(order.into_iter().map(|n| graph[n]).collect())
    }
    /// Native library SCC decomposition, with complete original unit identities.
    pub fn components(&self) -> Vec<Vec<SemanticId>> {
        let mut groups: Vec<_> = petgraph::algo::kosaraju_scc(&self.graph)
            .into_iter()
            .map(|g| {
                let mut g: Vec<_> = g.into_iter().map(|n| self.graph[n]).collect();
                g.sort();
                g
            })
            .collect();
        groups.sort();
        groups
    }
    /// Explicit unweighted petgraph heuristic, followed by policy and DAG validation.
    pub fn unweighted_heuristic(&self) -> Result<BTreeSet<SemanticId>, ProjectionError> {
        let mut selected: BTreeSet<_> = self
            .declaration
            .decisions
            .iter()
            .filter(|g| g.policy == Policy::Mandatory)
            .map(|g| g.id)
            .collect();
        let mut graph = self.graph.clone();
        graph.retain_edges(|g, e| !selected.contains(&self.declaration.connections[g[e]].decision));
        for edge in petgraph::algo::greedy_feedback_arc_set(&graph) {
            selected.insert(self.declaration.connections[*edge.weight()].decision);
        }
        self.witness(&selected)?;
        Ok(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn physical_connections_retain_affine_conversion_and_original_occurrences() {
        let r = pse_quantity::standard::standard_registry().unwrap();
        let q = pse_quantity::standard::ids::quantity("temperature.point");
        let source = pse_kernels::Port {
            id: id(10),
            quantity: q,
            unit: pse_quantity::standard::ids::unit("degC"),
        };
        let target = pse_kernels::Port {
            id: id(11),
            quantity: q,
            unit: pse_quantity::standard::ids::unit("K"),
        };
        let mut d = Declaration {
            nodes: vec![
                Node {
                    id: id(1),
                    ports: vec![source],
                },
                Node {
                    id: id(2),
                    ports: vec![target],
                },
                Node {
                    id: id(3),
                    ports: vec![],
                },
            ],
            connections: vec![Connection {
                id: id(20),
                from: id(1),
                to: id(2),
                decision: id(30),
                bindings: vec![(id(10), id(11))],
            }],
            decisions: vec![Decision {
                id: id(30),
                cost: 2.0,
                policy: Policy::Free,
            }],
        };
        let g = FlowGraph::admit(
            d.clone(),
            &r,
            GraphLimits {
                nodes: 10,
                edges: 10,
            },
        )
        .unwrap();
        let b = &g.bindings()[&id(20)][0];
        assert!((pse_quantity::convert_value(&b.conversion, 25.0) - 298.15).abs() < 1e-12);
        assert_eq!(g.witness(&BTreeSet::new()).unwrap().len(), 3);
        d.nodes.reverse();
        assert_eq!(
            g,
            FlowGraph::admit(
                d.clone(),
                &r,
                GraphLimits {
                    nodes: 10,
                    edges: 10
                }
            )
            .unwrap()
        );
        d.connections.push(d.connections[0].clone());
        d.connections[1].id = id(21);
        assert!(
            FlowGraph::admit(
                d,
                &r,
                GraphLimits {
                    nodes: 10,
                    edges: 10
                }
            )
            .is_err()
        );
    }
}
