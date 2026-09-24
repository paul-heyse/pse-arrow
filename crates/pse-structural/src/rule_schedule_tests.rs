// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{
    domains::{RuleDependencies, RuleDependency, RulePolarity},
    projection::{Dependency, GraphLimits, Scope},
};
use pse_ids::SemanticId;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn edge(n: u8, from: u8, to: u8, polarity: RulePolarity) -> RuleDependency {
    RuleDependency {
        edge: Dependency {
            id: id(n),
            from: id(from),
            to: id(to),
        },
        polarity,
    }
}
const LIMITS: GraphLimits = GraphLimits {
    nodes: 100,
    edges: 100,
};
#[test]
fn derived_strata_positive_scc_settlement_edges_and_isolates() {
    let nodes = (1..=6).map(id).collect::<Vec<_>>();
    let edges = vec![
        edge(10, 1, 2, RulePolarity::Positive),
        edge(11, 2, 1, RulePolarity::Positive),
        edge(12, 2, 3, RulePolarity::Negative),
        edge(13, 3, 4, RulePolarity::Positive),
        edge(14, 4, 5, RulePolarity::ConflictSensitive),
    ];
    let schedule =
        RuleDependencies::derive(Scope::Whole(id(0)), nodes.clone(), edges.clone(), LIMITS)
            .unwrap();
    assert_eq!(
        (1..=6)
            .map(|i| schedule.entries[&id(i)].stratum)
            .collect::<Vec<_>>(),
        [0, 0, 1, 1, 2, 0]
    );
    assert_eq!(
        schedule.entries[&id(1)].component,
        schedule.entries[&id(2)].component
    );
    for dependency in &edges {
        let (from, to) = (
            schedule.entries[&dependency.edge.from],
            schedule.entries[&dependency.edge.to],
        );
        assert!(from.component <= to.component);
    }
    let mut reversed = nodes.clone();
    reversed.reverse();
    let mut reordered = edges.clone();
    reordered.reverse();
    assert_eq!(
        RuleDependencies::derive(Scope::Whole(id(0)), reversed, reordered, LIMITS).unwrap(),
        schedule
    );
    for polarity in [
        RulePolarity::Negative,
        RulePolarity::Nonmonotone,
        RulePolarity::ConflictSensitive,
    ] {
        assert!(
            RuleDependencies::derive(
                Scope::Whole(id(0)),
                vec![id(1), id(2)],
                vec![
                    edge(10, 1, 2, polarity),
                    edge(11, 2, 1, RulePolarity::Positive)
                ],
                LIMITS
            )
            .is_err()
        );
    }
    assert!(RuleDependencies::derive(Scope::Partial(id(0)), nodes, edges, LIMITS).is_err());
}
