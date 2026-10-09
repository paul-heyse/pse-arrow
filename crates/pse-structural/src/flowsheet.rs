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
/// User constraint on one decision group, declared by the canonical registry vocabulary.
pub use pse_model::generated::enums::TearPolicy as Policy;
/// One decision may group several occurrences, but grouping is always declared.
#[derive(Clone, Debug)]
pub struct Decision {
    /// Stable decision identity.
    pub id: SemanticId,
    /// Finite nonnegative authored weight.
    pub cost: f64,
    /// Mandatory/forbidden/free selection.
    pub policy: Policy,
}
impl PartialEq for Decision {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.policy == other.policy
            && pse_ids::canonical_f64_bits(self.cost) == pse_ids::canonical_f64_bits(other.cost)
    }
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
    /// Source-population bound before flow declaration and graph construction.
    /// Counts declaration growth, physical Binding clones, simultaneous graph copies,
    /// graph-index maps/sets and bounded traversal vectors. Registry-owned quantities
    /// remain shared; `quantity_payload` covers cloned names and shape vectors.
    pub fn construction_allocation_bound(
        nodes: usize,
        ports: usize,
        connections: usize,
        decisions: usize,
        bindings: usize,
        quantity_payload: usize,
    ) -> Result<usize, ProjectionError> {
        let add = |a: usize, b: usize| a.checked_add(b).ok_or(ProjectionError::Limit);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or(ProjectionError::Limit);
        let vector = |count: usize, width: usize| -> Result<usize, ProjectionError> {
            if count == 0 {
                return Ok(0);
            }
            mul(add(mul(count, 2)?, 4)?, width)
        };
        // A BTree node has at most eleven key/value slots and twelve child links.
        // Charging a full node per entry includes sparsely occupied roots/splits.
        let tree = |count: usize, width: usize| -> Result<usize, ProjectionError> {
            if count == 0 {
                return Ok(0);
            }
            mul(
                add(count, 1)?,
                add(mul(width, 11)?, 16 * size_of::<usize>())?,
            )
        };
        let mut bytes = 2 * size_of::<Self>() + size_of::<Declaration>();
        for (count, width) in [
            (nodes, size_of::<Node>()),
            (ports, size_of::<pse_kernels::Port>()),
            (connections, size_of::<Connection>()),
            (decisions, size_of::<Decision>()),
            (bindings, size_of::<(SemanticId, SemanticId)>()),
            (bindings, size_of::<Binding>()),
            // Index graphs coexist with a forbidden/tear witness clone.
            (nodes, 2 * size_of::<petgraph::graph::Node<SemanticId>>()),
            (connections, 2 * size_of::<petgraph::graph::Edge<usize>>()),
            // DFS/SCC/toposort stacks, visit/order maps and cycle result.
            (
                add(nodes, connections)?,
                size_of::<[usize; 8]>() + size_of::<SemanticId>(),
            ),
        ] {
            bytes = add(bytes, vector(count, width)?)?;
        }
        for (count, width) in [
            (nodes, size_of::<(SemanticId, petgraph::graph::NodeIndex)>()),
            (
                ports,
                size_of::<(SemanticId, (SemanticId, &pse_kernels::Port))>(),
            ),
            (connections, size_of::<(SemanticId, Vec<Binding>)>()),
            (
                decisions,
                size_of::<(SemanticId, Decision)>() + 2 * size_of::<SemanticId>(),
            ),
            (bindings, size_of::<SemanticId>()),
        ] {
            bytes = add(bytes, tree(count, width)?)?;
        }
        // Vec::push grows each separate node-port/edge-binding vector from four.
        bytes = add(bytes, mul(nodes, 4 * size_of::<pse_kernels::Port>())?)?;
        bytes = add(
            bytes,
            mul(
                connections,
                4 * (size_of::<Binding>() + size_of::<(SemanticId, SemanticId)>()),
            )?,
        )?;
        add(bytes, quantity_payload)
    }
    /// Physical names/shape arrays copied when Binding values are cloned.
    pub fn binding_payload_bytes(&self) -> Result<usize, ProjectionError> {
        self.bindings
            .values()
            .flatten()
            .try_fold(0usize, |total, binding| {
                let shape = binding
                    .quantity
                    .key
                    .shape
                    .len()
                    .checked_mul(size_of::<pse_quantity::EntityKindId>())
                    .ok_or(ProjectionError::Limit)?;
                total
                    .checked_add(shape)
                    .and_then(|n| {
                        n.checked_add(binding.quantity.name.as_ref().map_or(0, String::len))
                    })
                    .ok_or(ProjectionError::Limit)
            })
    }
    /// Retained declaration, graph and physical binding containers, including known
    /// vector capacities and sparsely occupied BTree nodes. Temporary traversal maps
    /// and forbidden/tear graph copies are excluded after construction completes.
    pub fn retained_allocation_bound(&self) -> Result<usize, ProjectionError> {
        let add = |a: usize, b: usize| a.checked_add(b).ok_or(ProjectionError::Limit);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or(ProjectionError::Limit);
        let d = self.declaration();
        let (nodes, edges) = self.graph.capacity();
        let mut bytes = size_of::<Self>();
        for (count, width) in [
            (d.nodes.capacity(), size_of::<Node>()),
            (d.connections.capacity(), size_of::<Connection>()),
            (d.decisions.capacity(), size_of::<Decision>()),
            (nodes, size_of::<petgraph::graph::Node<SemanticId>>()),
            (edges, size_of::<petgraph::graph::Edge<usize>>()),
        ] {
            bytes = add(bytes, mul(count, width)?)?;
        }
        for node in &d.nodes {
            bytes = add(
                bytes,
                mul(node.ports.capacity(), size_of::<pse_kernels::Port>())?,
            )?;
        }
        for edge in &d.connections {
            bytes = add(
                bytes,
                mul(
                    edge.bindings.capacity(),
                    size_of::<(SemanticId, SemanticId)>(),
                )?,
            )?;
        }
        if !self.bindings.is_empty() {
            bytes = add(
                bytes,
                mul(
                    add(self.bindings.len(), 1)?,
                    11 * size_of::<(SemanticId, Vec<Binding>)>() + 16 * size_of::<usize>(),
                )?,
            )?;
        }
        for bindings in self.bindings.values() {
            bytes = add(bytes, mul(bindings.capacity(), size_of::<Binding>())?)?;
            for binding in bindings {
                bytes = add(
                    bytes,
                    mul(
                        binding.quantity.key.shape.capacity(),
                        size_of::<pse_quantity::EntityKindId>(),
                    )?,
                )?;
                bytes = add(
                    bytes,
                    binding.quantity.name.as_ref().map_or(0, String::capacity),
                )?;
            }
        }
        Ok(bytes)
    }
    /// Tear witness construction uses the original admitted graph population.
    /// The greedy petgraph algorithm additionally has two adjacency copies,
    /// linked degree-bucket entries and positive/negative degree bucket vectors.
    pub fn tear_allocation_bound(&self) -> Result<usize, ProjectionError> {
        let d = self.declaration();
        let ports = d.nodes.iter().try_fold(0usize, |n, node| {
            n.checked_add(node.ports.len())
                .ok_or(ProjectionError::Limit)
        })?;
        let bindings = d.connections.iter().try_fold(0usize, |n, edge| {
            n.checked_add(edge.bindings.len())
                .ok_or(ProjectionError::Limit)
        })?;
        let base = Self::construction_allocation_bound(
            d.nodes.len(),
            ports,
            d.connections.len(),
            d.decisions.len(),
            bindings,
            self.binding_payload_bytes()?,
        )?;
        // Pinned petgraph 0.8.3 FasNode + optional prev/next links: at most
        // eight usize fields and two adjacency Vec descriptors per node. Each
        // edge occurs in both directions; degree buckets are bounded by edges.
        d.nodes
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(size_of::<[usize; 8]>() + 2 * size_of::<Vec<usize>>()))
            .and_then(|n| {
                d.connections
                    .len()
                    .checked_mul(size_of::<[usize; 8]>())
                    .and_then(|e| n.checked_add(e))
            })
            .and_then(|n| n.checked_add(base))
            .ok_or(ProjectionError::Limit)
    }
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
        let mut h = FramedHasher::new(pse_ids::Frame::FlowProjectionV2);
        h.str("nodes").u64(d.nodes.len() as u64);
        for n in &mut d.nodes {
            n.ports.sort_by_key(|p| p.id);
            nodes.insert(n.id, graph.add_node(n.id));
            h.str("node")
                .id(&n.id)
                .str("ports")
                .u64(n.ports.len() as u64);
            for p in &n.ports {
                if ports.insert(p.id, (n.id, p)).is_some() {
                    return Err(ProjectionError::Invalid("duplicate flow port".into()));
                }
                h.str("port")
                    .id(&p.id)
                    .id(&p.quantity.as_id())
                    .id(&p.unit.as_id());
            }
        }
        let groups: BTreeSet<_> = d.decisions.iter().map(|g| g.id).collect();
        let mut used = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        let mut bindings = BTreeMap::new();
        h.str("connections").u64(d.connections.len() as u64);
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
            h.str("connection")
                .id(&e.id)
                .id(&e.from)
                .id(&e.to)
                .id(&e.decision)
                .str("bindings")
                .u64(e.bindings.len() as u64);
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
                h.str("binding")
                    .id(source)
                    .id(target)
                    .u64(pse_ids::canonical_f64_bits(binding.conversion.scale))
                    .u64(pse_ids::canonical_f64_bits(binding.conversion.offset));
                bindings.entry(e.id).or_insert_with(Vec::new).push(binding);
            }
            graph.add_edge(from, to, i);
        }
        if used != groups {
            return Err(ProjectionError::Invalid("unused tear decision".into()));
        }
        h.str("decisions").u64(d.decisions.len() as u64);
        for g in &d.decisions {
            h.str("decision")
                .id(&g.id)
                .u64(pse_ids::canonical_f64_bits(g.cost))
                .u64(g.policy as u64);
        }
        let mut forbidden = graph.clone();
        forbidden.retain_edges(|g, e| {
            d.decisions
                .binary_search_by_key(&d.connections[g[e]].decision, |decision| decision.id)
                .is_ok_and(|index| d.decisions[index].policy == Policy::Forbidden)
        });
        let cycle = cycle_connections(&forbidden, &d.connections);
        if !cycle.is_empty() {
            let decisions = cycle
                .iter()
                .filter_map(|id| d.connections.iter().find(|e| e.id == *id))
                .map(|e| e.decision)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            return Err(ProjectionError::ForbiddenTearCycle {
                connections: cycle,
                decisions,
            });
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
        let mut conflicts: BTreeSet<_> = selected
            .iter()
            .filter(|id| {
                self.declaration
                    .decisions
                    .binary_search_by_key(*id, |g| g.id)
                    .is_err()
            })
            .copied()
            .collect();
        conflicts.extend(
            self.declaration
                .decisions
                .iter()
                .filter(|g| {
                    g.policy == Policy::Mandatory && !selected.contains(&g.id)
                        || g.policy == Policy::Forbidden && selected.contains(&g.id)
                })
                .map(|g| g.id),
        );
        if !conflicts.is_empty() {
            return Err(ProjectionError::TearPolicy(conflicts.into_iter().collect()));
        }
        let mut graph = self.graph.clone();
        graph.retain_edges(|g, e| !selected.contains(&self.declaration.connections[g[e]].decision));
        let order = petgraph::algo::toposort(&graph, None).map_err(|_| {
            ProjectionError::Cycle(cycle_connections(&graph, &self.declaration.connections))
        })?;
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
        if self
            .declaration
            .decisions
            .iter()
            .any(|d| d.policy == Policy::Forbidden)
        {
            let forbidden: BTreeSet<_> = self
                .declaration
                .decisions
                .iter()
                .filter(|d| d.policy == Policy::Forbidden)
                .map(|d| d.id)
                .collect();
            let mut backbone = graph.clone();
            backbone.retain_edges(|g, e| {
                forbidden.contains(&self.declaration.connections[g[e]].decision)
            });
            let order = petgraph::algo::toposort(&backbone, None).map_err(|_| {
                ProjectionError::Cycle(cycle_connections(&backbone, &self.declaration.connections))
            })?;
            let rank: BTreeMap<_, _> = order
                .into_iter()
                .enumerate()
                .map(|(i, n)| (backbone[n], i))
                .collect();
            for e in &self.declaration.connections {
                if !forbidden.contains(&e.decision) && rank[&e.from] >= rank[&e.to] {
                    selected.insert(e.decision);
                }
            }
        } else {
            for edge in petgraph::algo::greedy_feedback_arc_set(&graph) {
                selected.insert(self.declaration.connections[*edge.weight()].decision);
            }
        }
        self.witness(&selected)?;
        Ok(selected)
    }
}

fn cycle_connections(
    graph: &Graph<SemanticId, usize, Directed>,
    connections: &[Connection],
) -> Vec<SemanticId> {
    rustworkx_core::connectivity::find_cycle(graph, None)
        .into_iter()
        .filter_map(|(from, to)| {
            graph
                .edges_connecting(from, to)
                .map(|e| connections[*e.weight()].id)
                .min()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn flow_source_construction_checks_populations_and_overflow_before_allocation() {
        let small = FlowGraph::construction_allocation_bound(2, 4, 2, 2, 2, 32).unwrap();
        assert!(small < 1 << 20);
        let larger = FlowGraph::construction_allocation_bound(2, 4, 2, 2, 4, 64).unwrap();
        assert!(larger > small);
        assert!(matches!(
            FlowGraph::construction_allocation_bound(usize::MAX, 1, 1, 1, 1, 1),
            Err(ProjectionError::Limit)
        ));
    }
    #[test]
    fn flow_projection_v2_frames_empty_collections_under_its_own_domain() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let graph = FlowGraph::admit(
            Declaration {
                nodes: vec![],
                connections: vec![],
                decisions: vec![],
            },
            &registry,
            GraphLimits { nodes: 8, edges: 8 },
        )
        .unwrap();
        let expected = pse_ids::derive_hash(
            pse_ids::Frame::FlowProjectionV2,
            &[
                b"nodes",
                &0u64.to_le_bytes(),
                b"connections",
                &0u64.to_le_bytes(),
                b"decisions",
                &0u64.to_le_bytes(),
            ],
        );
        assert_eq!(graph.key(), expected);
        assert_ne!(
            graph.key(),
            pse_ids::derive_hash(pse_ids::Frame::FlowProjectionV1, &[])
        );
    }
    #[test]
    fn flow_projection_v2_distinguishes_raw_id_parent_boundaries() {
        // A raw-ID encoding control, not an authored/compiler-derived physical
        // collision claim. Unconnected ports do not require registry resolution.
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let port = |p, q, u| pse_kernels::Port {
            id: id(p),
            quantity: id(q).into(),
            unit: id(u).into(),
        };
        let a = Declaration {
            nodes: vec![
                Node {
                    id: id(1),
                    ports: vec![port(2, 3, 4)],
                },
                Node {
                    id: id(5),
                    ports: vec![port(6, 7, 8)],
                },
            ],
            connections: vec![],
            decisions: vec![],
        };
        let b = Declaration {
            nodes: vec![
                Node {
                    id: id(1),
                    ports: vec![],
                },
                Node {
                    id: id(2),
                    ports: vec![port(3, 4, 5), port(6, 7, 8)],
                },
            ],
            connections: vec![],
            decisions: vec![],
        };
        let untagged_identifiers = |d: &Declaration| {
            d.nodes
                .iter()
                .flat_map(|n| {
                    std::iter::once(n.id).chain(
                        n.ports
                            .iter()
                            .flat_map(|p| [p.id, p.quantity.as_id(), p.unit.as_id()]),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(a.nodes.len(), b.nodes.len());
        assert_eq!(untagged_identifiers(&a), untagged_identifiers(&b));
        let a = FlowGraph::admit(a, &registry, GraphLimits { nodes: 8, edges: 8 }).unwrap();
        let b = FlowGraph::admit(b, &registry, GraphLimits { nodes: 8, edges: 8 }).unwrap();
        assert_eq!(
            a.key(),
            pse_ids::derive_hash(
                pse_ids::Frame::FlowProjectionV2,
                &[
                    b"nodes",
                    &2u64.to_le_bytes(),
                    b"node",
                    id(1).as_bytes(),
                    b"ports",
                    &1u64.to_le_bytes(),
                    b"port",
                    id(2).as_bytes(),
                    id(3).as_bytes(),
                    id(4).as_bytes(),
                    b"node",
                    id(5).as_bytes(),
                    b"ports",
                    &1u64.to_le_bytes(),
                    b"port",
                    id(6).as_bytes(),
                    id(7).as_bytes(),
                    id(8).as_bytes(),
                    b"connections",
                    &0u64.to_le_bytes(),
                    b"decisions",
                    &0u64.to_le_bytes(),
                ],
            )
        );
        assert_ne!(a.key(), b.key());
    }
    #[test]
    fn flow_projection_v2_canonicalizes_inventories_and_keeps_full_equality() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit,
        };
        let declaration = Declaration {
            nodes: vec![
                Node {
                    id: id(1),
                    ports: vec![port(10), port(12), port(14)],
                },
                Node {
                    id: id(2),
                    ports: vec![port(11), port(13), port(15)],
                },
                Node {
                    id: id(3),
                    ports: vec![],
                },
            ],
            connections: vec![
                Connection {
                    id: id(20),
                    from: id(1),
                    to: id(2),
                    decision: id(30),
                    bindings: vec![(id(10), id(11)), (id(12), id(13))],
                },
                Connection {
                    id: id(21),
                    from: id(2),
                    to: id(1),
                    decision: id(31),
                    bindings: vec![(id(15), id(14))],
                },
            ],
            decisions: vec![
                Decision {
                    id: id(30),
                    cost: 0.0,
                    policy: Policy::Forbidden,
                },
                Decision {
                    id: id(31),
                    cost: 2.0,
                    policy: Policy::Free,
                },
            ],
        };
        let admit = |d| FlowGraph::admit(d, &registry, GraphLimits { nodes: 8, edges: 8 }).unwrap();
        let graph = admit(declaration.clone());
        let mut reordered = declaration.clone();
        reordered.nodes.reverse();
        for node in &mut reordered.nodes {
            node.ports.reverse();
        }
        reordered.connections.reverse();
        for connection in &mut reordered.connections {
            connection.bindings.reverse();
        }
        reordered.decisions.reverse();
        assert_eq!(graph, admit(reordered));
        assert_eq!(
            graph.unweighted_heuristic().unwrap(),
            BTreeSet::from([id(31)])
        );
        assert_eq!(graph.witness(&BTreeSet::from([id(31)])).unwrap().len(), 3);
        let mut signed_zero = declaration.clone();
        signed_zero.decisions[0].cost = -0.0;
        assert_ne!(graph.key(), admit(signed_zero).key());
        let mut nonfinite = declaration;
        nonfinite.decisions[0].cost = f64::NAN;
        assert!(
            FlowGraph::admit(nonfinite, &registry, GraphLimits { nodes: 8, edges: 8 }).is_err()
        );
        // The scoped fingerprint does not replace full physical-context equality.
        let mut different_context = graph.clone();
        different_context.bindings.get_mut(&id(20)).unwrap()[0]
            .quantity
            .name = Some("different-context-name".into());
        assert_eq!(graph.key(), different_context.key());
        assert_ne!(graph, different_context);
    }
    #[test]
    fn forbidden_cycles_retain_actual_connection_and_decision_identities() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit,
        };
        let declaration = Declaration {
            nodes: vec![
                Node {
                    id: id(1),
                    ports: vec![port(10)],
                },
                Node {
                    id: id(2),
                    ports: vec![port(11)],
                },
            ],
            connections: vec![
                Connection {
                    id: id(20),
                    from: id(1),
                    to: id(2),
                    decision: id(30),
                    bindings: vec![(id(10), id(11))],
                },
                Connection {
                    id: id(21),
                    from: id(2),
                    to: id(1),
                    decision: id(31),
                    bindings: vec![(id(11), id(10))],
                },
            ],
            decisions: vec![
                Decision {
                    id: id(30),
                    cost: 1.0,
                    policy: Policy::Forbidden,
                },
                Decision {
                    id: id(31),
                    cost: 1.0,
                    policy: Policy::Forbidden,
                },
            ],
        };
        let mut mixed = declaration.clone();
        mixed.decisions[1].policy = Policy::Free;
        let canonical =
            FlowGraph::admit(mixed.clone(), &registry, GraphLimits { nodes: 8, edges: 8 }).unwrap();
        mixed.decisions.reverse();
        let admitted =
            FlowGraph::admit(mixed, &registry, GraphLimits { nodes: 8, edges: 8 }).unwrap();
        assert_eq!(admitted, canonical);
        let selected = admitted.unweighted_heuristic().unwrap();
        assert_eq!(selected, BTreeSet::from([id(31)]));
        assert_eq!(admitted.witness(&selected).unwrap(), vec![id(1), id(2)]);
        let Err(ProjectionError::ForbiddenTearCycle {
            connections,
            decisions,
        }) = FlowGraph::admit(declaration, &registry, GraphLimits { nodes: 8, edges: 8 })
        else {
            panic!("missing forbidden cycle witness")
        };
        assert_eq!(
            connections.into_iter().collect::<BTreeSet<_>>(),
            BTreeSet::from([id(20), id(21)])
        );
        assert_eq!(
            decisions.into_iter().collect::<BTreeSet<_>>(),
            BTreeSet::from([id(30), id(31)])
        );
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
        let expected = pse_ids::derive_hash(
            pse_ids::Frame::FlowProjectionV2,
            &[
                b"nodes",
                &3u64.to_le_bytes(),
                b"node",
                id(1).as_bytes(),
                b"ports",
                &1u64.to_le_bytes(),
                b"port",
                id(10).as_bytes(),
                q.as_id().as_bytes(),
                pse_quantity::standard::ids::unit("degC").as_id().as_bytes(),
                b"node",
                id(2).as_bytes(),
                b"ports",
                &1u64.to_le_bytes(),
                b"port",
                id(11).as_bytes(),
                q.as_id().as_bytes(),
                pse_quantity::standard::ids::unit("K").as_id().as_bytes(),
                b"node",
                id(3).as_bytes(),
                b"ports",
                &0u64.to_le_bytes(),
                b"connections",
                &1u64.to_le_bytes(),
                b"connection",
                id(20).as_bytes(),
                id(1).as_bytes(),
                id(2).as_bytes(),
                id(30).as_bytes(),
                b"bindings",
                &1u64.to_le_bytes(),
                b"binding",
                id(10).as_bytes(),
                id(11).as_bytes(),
                &1.0f64.to_bits().to_le_bytes(),
                &273.15f64.to_bits().to_le_bytes(),
                b"decisions",
                &1u64.to_le_bytes(),
                b"decision",
                id(30).as_bytes(),
                &2.0f64.to_bits().to_le_bytes(),
                &(Policy::Free as u64).to_le_bytes(),
            ],
        );
        assert_eq!(g.key(), expected);
        let conversion =
            pse_quantity::CanonicalConversionPlan::registered(&r, b.quantity.id, b.conversion.from)
                .unwrap();
        assert!((conversion.apply(25.0).unwrap().value() - 298.15).abs() < 1e-12);
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
