// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-local bounded graph traversal, independent of native handles and transport.
use super::{Direction, Relationship};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Whether an answer must be reachable from any seed or every seed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeedMatch {
    /// At least one seed establishes the answer.
    #[default]
    Any,
    /// Every seed establishes the answer within the selected depth.
    All,
}

/// A canonical relationship; incoming traversal never reverses its meaning.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct Edge<R = Relationship> {
    /// Canonical source declaration or use.
    pub source: usize,
    /// Canonical destination declaration.
    pub target: usize,
    /// Provider-domain relationship meaning; the default is a native relationship.
    pub relation: R,
    /// Index of the original occurrence witness, distinct for parallel occurrences.
    pub witness: usize,
}

/// One demand-selected adjacency observation.
#[derive(Clone, Debug, Default)]
pub struct Adjacency {
    /// All observed canonical edges adjacent in the requested direction.
    pub edges: Vec<Edge>,
    /// False when the provider could not establish exhaustive adjacency.
    pub complete: bool,
}

/// Fixed bounded traversal intent supplied to one native snapshot.
#[derive(Clone, Debug)]
pub struct Traversal {
    /// Request seed ordinals are preserved even when declarations overlap.
    pub seeds: Vec<usize>,
    /// Selected relation families.
    pub relations: Vec<Relationship>,
    /// Which endpoint supplies adjacency.
    pub direction: Direction,
    /// Intentional maximum edge distance.
    pub depth: usize,
    /// Maximum distinct admitted nodes, including seeds.
    pub max_nodes: usize,
    /// Maximum edge observations examined, independent of rendered row allowance.
    pub max_edges: usize,
    /// Selected seed attribution rule.
    pub seed_match: SeedMatch,
    /// Stop once a positive selected answer is established, without claiming complete absence.
    pub exists: bool,
}

/// Minimum distances preserve attribution without enumerating every possible path.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Reached {
    /// Native request-local node index.
    pub node: usize,
    /// One minimum distance per original seed, absent when not established.
    pub distances: Vec<Option<usize>>,
}

/// Actual traversal stop, separate from the intentional depth boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    /// Exhausted the requested within-depth domain.
    #[default]
    Complete,
    /// A distinct-node allowance prevented further admission.
    Nodes,
    /// An edge-examination allowance prevented further examination.
    Edges,
    /// Native adjacency was incomplete or unsupported.
    Provider,
    /// One sufficient positive answer satisfied existential demand.
    Demand,
    /// Cooperative cancellation was observed.
    Cancelled,
}

/// Compact shared graph and coverage returned by the actual traversal owner.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct TraversalResult {
    /// All admitted supporting nodes, with per-seed attribution.
    pub reached: Vec<Reached>,
    /// Answer node indexes after output and seed selection.
    pub selected: Vec<usize>,
    /// Examined original edge witnesses; equal endpoints do not collapse roles.
    pub edges: Vec<Edge>,
    /// Admitted nodes at the intentional depth boundary or unexamined stop frontier.
    pub frontier: Vec<usize>,
    /// Complete within the declared depth and traversal domain.
    pub complete: bool,
    /// Actual reason examination stopped.
    pub stop: Stop,
    /// Number of edge observations examined, before any display projection.
    pub examined_edges: usize,
}

/// Walk each qualified adjacency once, propagating newly arriving seed attribution through it.
/// The adapter owns native identity and must return only edges from its selected snapshot/domain.
pub fn walk<E>(
    intent: &Traversal,
    mut adjacency: impl FnMut(usize) -> Result<Adjacency, E>,
    selected: impl Fn(usize) -> bool,
    cancelled: impl Fn() -> bool,
) -> Result<TraversalResult, E> {
    let mut distances = BTreeMap::<usize, Vec<Option<usize>>>::new();
    let mut queue = VecDeque::new();
    let mut cache = BTreeMap::<usize, Vec<Edge>>::new();
    let mut edges = BTreeSet::new();
    let mut positive = BTreeSet::new();
    let mut frontier = BTreeSet::new();
    let mut result = TraversalResult {
        complete: true,
        ..Default::default()
    };
    for (seed, &node) in intent.seeds.iter().enumerate() {
        if !distances.contains_key(&node) && distances.len() >= intent.max_nodes {
            result.complete = false;
            result.stop = Stop::Nodes;
            frontier.insert(node);
            continue;
        }
        distances
            .entry(node)
            .or_insert_with(|| vec![None; intent.seeds.len()])[seed] = Some(0);
        queue.push_back((node, seed, 0usize));
    }
    'walk: while let Some((node, seed, distance)) = queue.pop_front() {
        if cancelled() {
            result.complete = false;
            result.stop = Stop::Cancelled;
            frontier.insert(node);
            frontier.extend(queue.iter().map(|(node, _, _)| *node));
            break;
        }
        if distances[&node][seed] != Some(distance) {
            continue;
        }
        if distance >= intent.depth {
            frontier.insert(node);
            continue;
        }
        if !cache.contains_key(&node) {
            let observed = adjacency(node)?;
            if !observed.complete {
                result.complete = false;
                if result.stop == Stop::Complete {
                    result.stop = Stop::Provider;
                }
            }
            let mut admitted = Vec::new();
            for edge in observed.edges {
                if !intent.relations.contains(&edge.relation) {
                    continue;
                }
                let adjacent = match intent.direction {
                    Direction::Outgoing => edge.source == node,
                    Direction::Incoming => edge.target == node,
                };
                if !adjacent {
                    continue;
                }
                if result.examined_edges >= intent.max_edges {
                    result.complete = false;
                    result.stop = Stop::Edges;
                    frontier.insert(node);
                    break;
                }
                result.examined_edges += 1;
                edges.insert(edge.clone());
                admitted.push(edge);
            }
            cache.insert(node, admitted);
        }
        for edge in &cache[&node] {
            let next = match intent.direction {
                Direction::Outgoing => edge.target,
                Direction::Incoming => edge.source,
            };
            if !distances.contains_key(&next) && distances.len() >= intent.max_nodes {
                result.complete = false;
                if result.stop != Stop::Edges {
                    result.stop = Stop::Nodes;
                }
                frontier.insert(next);
                continue;
            }
            let entry = distances
                .entry(next)
                .or_insert_with(|| vec![None; intent.seeds.len()]);
            // A seed may itself have a positive cycle witness while its minimum distance
            // remains zero. Keep that fact separate from the shortest-distance state.
            positive.insert(next);
            if entry[seed].is_none_or(|old| distance + 1 < old) {
                entry[seed] = Some(distance + 1);
                queue.push_back((next, seed, distance + 1));
            }
            let matching = match intent.seed_match {
                SeedMatch::Any => entry.iter().any(Option::is_some),
                SeedMatch::All => entry.iter().all(Option::is_some),
            };
            if intent.exists && matching && selected(next) {
                result.complete = false;
                result.stop = Stop::Demand;
                frontier.insert(node);
                frontier.extend(queue.iter().map(|(node, _, _)| *node));
                break 'walk;
            }
        }
        // An edge stop means the cached partial adjacency must never be mistaken for complete.
        // Propagate already observed witnesses, then retain remaining queued work as frontier.
        if result.stop == Stop::Edges {
            frontier.extend(queue.iter().map(|(node, _, _)| *node));
            break;
        }
    }
    result.selected = distances
        .iter()
        .filter_map(|(&node, attribution)| {
            let matching = match intent.seed_match {
                SeedMatch::Any => attribution.iter().any(Option::is_some),
                SeedMatch::All => attribution.iter().all(Option::is_some),
            };
            (positive.contains(&node) && matching && selected(node)).then_some(node)
        })
        .collect();
    result.reached = distances
        .into_iter()
        .map(|(node, distances)| Reached { node, distances })
        .collect();
    result.edges = edges.into_iter().collect();
    result.frontier = frontier.into_iter().collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edge(source: usize, target: usize, witness: usize) -> Edge {
        Edge {
            source,
            target,
            witness,
            relation: Relationship::Uses,
        }
    }
    fn intent(seeds: Vec<usize>) -> Traversal {
        Traversal {
            seeds,
            relations: vec![Relationship::Uses],
            direction: Direction::Outgoing,
            depth: 3,
            max_nodes: 20,
            max_edges: 30,
            seed_match: SeedMatch::All,
            exists: false,
        }
    }
    #[test]
    fn shared_adjacency_propagates_later_seed_and_hidden_intermediates() {
        let mut calls = BTreeMap::new();
        let graph = [
            edge(0, 2, 0),
            edge(1, 3, 1),
            edge(3, 2, 2),
            edge(2, 4, 3),
            edge(4, 2, 4),
        ];
        let result = walk(
            &intent(vec![0, 1]),
            |node| {
                *calls.entry(node).or_insert(0) += 1;
                Ok::<_, ()>(Adjacency {
                    edges: graph
                        .iter()
                        .filter(|edge| edge.source == node)
                        .cloned()
                        .collect(),
                    complete: true,
                })
            },
            |node| node == 4,
            || false,
        )
        .unwrap();
        assert_eq!(result.selected, vec![4]);
        assert_eq!(
            result
                .reached
                .iter()
                .find(|node| node.node == 4)
                .unwrap()
                .distances,
            vec![Some(2), Some(3)]
        );
        assert!(calls.values().all(|count| *count == 1));
        assert!(result.complete);
    }
    #[test]
    fn canonical_edges_parallel_witnesses_and_bounded_absence() {
        let mut request = intent(vec![2]);
        request.direction = Direction::Incoming;
        request.depth = 1;
        let graph = vec![edge(0, 2, 0), edge(0, 2, 1)];
        let run = |request: &Traversal| {
            walk(
                request,
                |_| {
                    Ok::<_, ()>(Adjacency {
                        edges: graph.clone(),
                        complete: true,
                    })
                },
                |_| true,
                || false,
            )
            .unwrap()
        };
        let complete = run(&request);
        assert!(complete.complete);
        assert_eq!(complete.edges, graph);
        assert_eq!(complete.selected, vec![0]);
        assert_eq!(complete.frontier, vec![0]);
        request.max_edges = 1;
        let partial = run(&request);
        assert!(!partial.complete);
        assert_eq!(partial.stop, Stop::Edges);
        assert_eq!(partial.examined_edges, 1);
    }
    #[test]
    fn incomplete_provider_and_node_limit_never_establish_absence() {
        let request = Traversal {
            max_nodes: 1,
            ..intent(vec![0])
        };
        let result = walk(
            &request,
            |_| {
                Ok::<_, ()>(Adjacency {
                    edges: vec![edge(0, 1, 0)],
                    complete: false,
                })
            },
            |_| true,
            || false,
        )
        .unwrap();
        assert!(!result.complete);
        assert!(result.selected.is_empty());
        assert_eq!(result.stop, Stop::Nodes);
    }
    #[test]
    fn overlapping_seeds_share_work_and_cancelled_work_keeps_frontier() {
        let request = intent(vec![0, 0]);
        let mut examined = 0;
        let result = walk(
            &request,
            |node| {
                examined += 1;
                Ok::<_, ()>(Adjacency {
                    edges: if node == 0 {
                        vec![edge(0, 1, 0)]
                    } else {
                        Vec::new()
                    },
                    complete: true,
                })
            },
            |_| true,
            || false,
        )
        .unwrap();
        assert_eq!(result.selected, vec![1]);
        assert_eq!(result.reached[1].distances, vec![Some(1), Some(1)]);
        assert_eq!(examined, 2);
        let cancelled = walk(
            &request,
            |_| -> Result<Adjacency, ()> { panic!("cancelled work started adjacency") },
            |_| true,
            || true,
        )
        .unwrap();
        assert_eq!(cancelled.stop, Stop::Cancelled);
        assert!(!cancelled.complete);
        assert_eq!(cancelled.frontier, vec![0]);
    }
    #[test]
    fn existential_demand_waits_for_every_selected_seed() {
        let request = Traversal {
            exists: true,
            ..intent(vec![0, 1])
        };
        let graph = [edge(0, 2, 0), edge(1, 2, 1), edge(2, 3, 2)];
        let mut calls = Vec::new();
        let result = walk(
            &request,
            |node| {
                calls.push(node);
                Ok::<_, ()>(Adjacency {
                    edges: graph
                        .iter()
                        .filter(|edge| edge.source == node)
                        .cloned()
                        .collect(),
                    complete: true,
                })
            },
            |node| node == 2,
            || false,
        )
        .unwrap();
        assert_eq!(result.selected, vec![2]);
        assert_eq!(result.stop, Stop::Demand);
        assert!(!result.complete);
        assert_eq!(calls, vec![0, 1]);
    }
    #[test]
    fn a_positive_cycle_can_return_its_seed_without_changing_minimum_distance() {
        let request = intent(vec![0]);
        let graph = [edge(0, 1, 0), edge(1, 0, 1)];
        let result = walk(
            &request,
            |node| {
                Ok::<_, ()>(Adjacency {
                    edges: graph
                        .iter()
                        .filter(|edge| edge.source == node)
                        .cloned()
                        .collect(),
                    complete: true,
                })
            },
            |_| true,
            || false,
        )
        .unwrap();
        assert_eq!(result.selected, vec![0, 1]);
        assert_eq!(result.reached[0].distances, vec![Some(0)]);
        assert!(result.complete);
    }
}
