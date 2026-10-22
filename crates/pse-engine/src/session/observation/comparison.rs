// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive operator-family correspondence; no execution or semantic equivalence inference.
use super::*;

/// Cross-capture operator correspondence with unresolved populations retained.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalGraphComparison {
    /// Unique operator/schema/adjacent-family matches, not scientific equivalence.
    pub matched: Vec<PhysicalNodeMatch>,
    /// Baseline nodes with multiple plausible operator correspondences.
    pub ambiguous_before: Vec<usize>,
    /// Current nodes with multiple plausible operator correspondences.
    pub ambiguous_after: Vec<usize>,
    /// Baseline nodes without a counterpart.
    pub unmatched_before: Vec<usize>,
    /// Current nodes without a counterpart.
    pub unmatched_after: Vec<usize>,
    /// Either original topology is incomplete; correspondences are withheld.
    pub incomplete: bool,
}
/// One unique structural operator correspondence across independent ordinals.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalNodeMatch {
    /// Baseline metric ordinal.
    pub before: usize,
    /// Current metric ordinal, never compared by numerical equality.
    pub after: usize,
    /// Actual original partition count.
    pub partitions_before: usize,
    /// Actual current partition count.
    pub partitions_after: usize,
}
type Context = (String, Vec<(String, String)>, Vec<(usize, String, bool)>, Vec<(usize, String, bool)>);
fn valid_coordinates(graph: &ObservedPhysicalGraph) -> bool {
    let nodes: std::collections::BTreeSet<_> = graph.nodes.iter().map(|node| node.node).collect();
    if nodes.len() != graph.nodes.len() || match graph.root {
        Some(root) => !nodes.contains(&root),
        None => !nodes.is_empty(),
    } { return false; }
    let mut slots = std::collections::BTreeSet::new();
    graph.edges.iter().all(|edge| nodes.contains(&edge.parent) && nodes.contains(&edge.child)
        && slots.insert((edge.parent,edge.ordinal)))
}
fn contexts(graph: &ObservedPhysicalGraph) -> std::collections::BTreeMap<Context, Vec<&ObservedPhysicalNode>> {
    let nodes: std::collections::BTreeMap<_,_> = graph.nodes.iter().map(|node| (node.node,node)).collect();
    let mut adjacency: std::collections::BTreeMap<_, (Vec<_>,Vec<_>)> = std::collections::BTreeMap::new();
    for edge in &graph.edges {
        if let (Some(parent),Some(child)) = (nodes.get(&edge.parent),nodes.get(&edge.child)) {
            adjacency.entry(edge.parent).or_default().1.push((edge.ordinal,child.operator.clone(),edge.cache_producer));
            adjacency.entry(edge.child).or_default().0.push((edge.ordinal,parent.operator.clone(),edge.cache_producer));
        }
    }
    let mut groups: std::collections::BTreeMap<Context,Vec<&ObservedPhysicalNode>> = std::collections::BTreeMap::new();
    for node in &graph.nodes {
        let (mut parents,mut children) = adjacency.remove(&node.node).unwrap_or_default();
        parents.sort(); children.sort();
        groups.entry((node.operator.clone(),node.output_fields.clone(),parents,children)).or_default().push(node);
    }
    groups
}
impl ObservedPhysicalGraph {
    /// Compare copied facts only. Equal local keys, native display strings and counts
    /// supply no identity. Unique family/schema/adjacency context supplies operator
    /// correspondence only; changed expressions/providers can still change meaning.
    pub fn compare(&self, after: &Self) -> PhysicalGraphComparison {
        let incomplete = self.truncated || after.truncated || !valid_coordinates(self) || !valid_coordinates(after);
        let mut result = PhysicalGraphComparison { matched: vec![], ambiguous_before: vec![], ambiguous_after: vec![], unmatched_before: vec![], unmatched_after: vec![], incomplete };
        if incomplete {
            result.unmatched_before = self.nodes.iter().map(|node| node.node).collect();
            result.unmatched_after = after.nodes.iter().map(|node| node.node).collect();
            return result;
        }
        let before_groups = contexts(self);
        let after_groups = contexts(after);
        for (key, before) in &before_groups {
            match after_groups.get(key) {
                Some(current) if before.len() == 1 && current.len() == 1 => result.matched.push(PhysicalNodeMatch { before: before[0].node, after: current[0].node, partitions_before: before[0].partitions, partitions_after: current[0].partitions }),
                Some(current) => { result.ambiguous_before.extend(before.iter().map(|node| node.node)); result.ambiguous_after.extend(current.iter().map(|node| node.node)); }
                None => result.unmatched_before.extend(before.iter().map(|node| node.node)),
            }
        }
        for (key, nodes) in after_groups { if !before_groups.contains_key(&key) { result.unmatched_after.extend(nodes.iter().map(|node| node.node)); } }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn graph(ids: &[usize]) -> ObservedPhysicalGraph {
        ObservedPhysicalGraph { root: ids.first().copied(), nodes: ids.iter().map(|node| ObservedPhysicalNode { node: *node, operator: "EmptyExec".into(), output_fields: vec![], partitions: 1 }).collect(), edges: vec![], truncated: false }
    }
    #[test]
    fn correspondence_uses_unique_context_and_preserves_ambiguity_and_missing_capture() {
        let before = graph(&[4]);
        let mut after = graph(&[9]); after.nodes[0].partitions = 3;
        let comparison = before.compare(&after);
        assert_eq!((comparison.matched[0].before, comparison.matched[0].after), (4, 9));
        assert_eq!((comparison.matched[0].partitions_before, comparison.matched[0].partitions_after), (1, 3));
        let repeated = graph(&[1, 2]);
        let ambiguous = repeated.compare(&graph(&[7, 8]));
        assert!(ambiguous.matched.is_empty()); assert_eq!(ambiguous.ambiguous_before, [1, 2]); assert_eq!(ambiguous.ambiguous_after, [7, 8]);
        let mut changed = graph(&[4]); changed.nodes[0].operator = "FilterExec".into();
        assert_eq!(before.compare(&changed).unmatched_before, [4]);
        after.truncated = true;
        assert!(before.compare(&after).incomplete); assert!(before.compare(&after).matched.is_empty());
    }
    #[test]
    fn malformed_copied_coordinates_withhold_matches_without_losing_shared_children() {
        let before = graph(&[1,2,3]);
        let mut shared = before.clone();
        shared.edges = vec![
            ObservedPhysicalEdge { parent: 1, ordinal: 0, child: 3, cache_producer: true },
            ObservedPhysicalEdge { parent: 2, ordinal: 0, child: 3, cache_producer: true },
        ];
        assert!(!shared.compare(&shared).incomplete,"multiple parents of the same cache producer are valid");
        let mut duplicate = before.clone(); duplicate.nodes.push(duplicate.nodes[0].clone());
        let mut root = before.clone(); root.root = Some(99);
        let mut missing_root = before.clone(); missing_root.root = None;
        let mut dangling = shared.clone(); dangling.edges[0].child = 99;
        let mut slots = shared.clone(); slots.edges.push(ObservedPhysicalEdge { parent: 1, ordinal: 0, child: 2, cache_producer: false });
        for malformed in [duplicate,root,missing_root,dangling,slots] {
            // Public frozen inputs can be deserialized independently of the producer.
            let decoded: ObservedPhysicalGraph = serde_json::from_slice(&serde_json::to_vec(&malformed).unwrap()).unwrap();
            let comparison = decoded.compare(&before);
            assert!(comparison.incomplete); assert!(comparison.matched.is_empty());
        }
    }
}
