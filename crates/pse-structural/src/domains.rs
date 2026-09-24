// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Domain-specific projection boundaries. These are algorithm views of generated declarations.
use crate::projection::{Dependency, GraphLimits, Projection, ProjectionError, Scope};
use pse_ids::{FramedHasher, SemanticId};
use pse_model::generated::normalized;
use std::collections::{BTreeMap, BTreeSet};

fn edge_id(domain: &'static str, from: SemanticId, to: SemanticId, label: &str) -> SemanticId {
    let mut hash = FramedHasher::new(domain);
    hash.id(&from).id(&to).str(label);
    hash.finish_id()
}
macro_rules! graph_view {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(Projection);
        impl $name {
            /// Admitted direct graph; source scope remains explicit.
            pub fn graph(&self) -> &Projection {
                &self.0
            }
        }
    };
}
graph_view!(
    PackageDependencies,
    "Exact selected packages, prerequisite to dependent, including isolated packages."
);
impl PackageDependencies {
    /// Project direct dependency membership; depth and annotation are not graph identity.
    /// # Errors
    /// Duplicate packages/dependencies, missing selected dependency or exhausted bounds.
    pub fn admit(
        scope: Scope,
        rows: &[normalized::package_graph::Row],
        limits: GraphLimits,
    ) -> Result<Self, ProjectionError> {
        let nodes = rows.iter().map(|row| row.package_id).collect();
        let edges = rows
            .iter()
            .flat_map(|row| {
                row.dependency_package_ids
                    .iter()
                    .map(move |dependency| Dependency {
                        id: edge_id("pse:package-edge:v1", *dependency, row.package_id, ""),
                        from: *dependency,
                        to: row.package_id,
                    })
            })
            .collect();
        Ok(Self(Projection::admit(scope, nodes, edges, limits)?))
    }
}
/// A rule edge's effect on fixed-point/stratum semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RulePolarity {
    /// Monotone positive dependence.
    Positive,
    /// Absence-sensitive dependence.
    Negative,
    /// Nonmonotone aggregate or policy.
    Nonmonotone,
    /// Conflict or undecided evidence must first settle.
    ConflictSensitive,
}
/// Interpreted rule dependence, with original source identity retained in the edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDependency {
    /// Prerequisite-to-dependent direct edge.
    pub edge: Dependency,
    /// Complete effect classification from the owning rule contract.
    pub polarity: RulePolarity,
}
/// Rule SCC analysis preserves all effect kinds; evaluation belongs to the relational owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleDependencies {
    graph: Projection,
    edges: Vec<RuleDependency>,
}
impl RuleDependencies {
    /// Derive the minimum legal strata from the complete checked effect graph.
    /// Positive recursion is one component; settlement-sensitive recursion is refused.
    /// # Errors
    /// Partial coverage, duplicate/dangling identities, a strict cycle or exhausted bounds.
    pub fn derive(
        scope: Scope,
        rules: Vec<SemanticId>,
        mut edges: Vec<RuleDependency>,
        limits: GraphLimits,
    ) -> Result<RuleSchedule, ProjectionError> {
        edges.sort_by_key(|edge| edge.edge.id);
        let graph = Projection::admit(
            scope,
            rules,
            edges.iter().map(|edge| edge.edge.clone()).collect(),
            limits,
        )?;
        let quotient = graph.condensation(limits)?;
        let membership: BTreeMap<_, _> = quotient
            .components
            .iter()
            .flat_map(|members| members.iter().map(|id| (*id, members[0])))
            .collect();
        let mut incoming = BTreeMap::<SemanticId, Vec<(SemanticId, bool)>>::new();
        let mut links = BTreeSet::new();
        for dependency in &edges {
            let from = membership[&dependency.edge.from];
            let to = membership[&dependency.edge.to];
            let strict = dependency.polarity != RulePolarity::Positive;
            if from == to {
                if strict {
                    return Err(ProjectionError::Cycle(
                        edges
                            .iter()
                            .filter(|edge| {
                                membership[&edge.edge.from] == from
                                    && membership[&edge.edge.to] == from
                            })
                            .map(|edge| edge.edge.id)
                            .collect(),
                    ));
                }
            } else {
                incoming.entry(to).or_default().push((from, strict));
                links.insert((from, to));
            }
        }
        let order = Projection::admit(
            Scope::Whole(SemanticId::NIL),
            quotient
                .components
                .iter()
                .map(|members| members[0])
                .collect(),
            links
                .into_iter()
                .map(|(from, to)| Dependency {
                    id: edge_id("pse:rule-component:v1", from, to, ""),
                    from,
                    to,
                })
                .collect(),
            limits,
        )?
        .order()?;
        let mut levels = BTreeMap::<SemanticId, u16>::new();
        let mut components = BTreeMap::new();
        for (ordinal, id) in order.iter().enumerate() {
            let mut level = 0;
            for (predecessor, strict) in incoming.get(id).into_iter().flatten() {
                level = level.max(
                    levels[predecessor]
                        .checked_add(u16::from(*strict))
                        .ok_or(ProjectionError::Limit)?,
                );
            }
            levels.insert(*id, level);
            components.insert(
                *id,
                u32::try_from(ordinal).map_err(|_| ProjectionError::Limit)?,
            );
        }
        Ok(RuleSchedule {
            entries: membership
                .into_iter()
                .map(|(rule, component)| {
                    (
                        rule,
                        RulePosition {
                            component: components[&component],
                            stratum: levels[&component],
                        },
                    )
                })
                .collect(),
            dependencies: Self { graph, edges },
        })
    }
    /// Complete graph for SCC/quotient interpretation.
    pub fn graph(&self) -> &Projection {
        &self.graph
    }
    /// Complete positive/negative/nonmonotone/conflict evidence.
    pub fn dependencies(&self) -> &[RuleDependency] {
        &self.edges
    }
}
/// A deterministic executable position derived from checked dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RulePosition {
    /// Prerequisite-first SCC ordinal, with semantic-ID tie breaking.
    pub component: u32,
    /// Minimum settlement stratum; positive edges may share a stratum.
    pub stratum: u16,
}
/// Pure executable scheduling result; reference declarations carry no authority over order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSchedule {
    /// Every selected rule, including rules without dependencies.
    pub entries: BTreeMap<SemanticId, RulePosition>,
    /// Complete dependency/provenance graph used to derive the schedule.
    pub dependencies: RuleDependencies,
}
#[cfg(test)]
mod foundation_unit {
    #![allow(clippy::unwrap_used, reason = "isolated typed graph fixtures")]
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    const LIMITS: GraphLimits = GraphLimits {
        nodes: 100,
        edges: 100,
    };
    fn scope() -> Scope {
        Scope::Whole(id(0))
    }
    #[test]
    fn typed_package_boundary() {
        let package = |n, dependencies| normalized::package_graph::Row {
            package_id: id(n),
            version: "1".into(),
            content_hash: pse_ids::ContentHash::from_bytes([0; 32]),
            depth: 0,
            dependency_package_ids: dependencies,
            derivation_id: id(20),
        };
        let graph = PackageDependencies::admit(
            scope(),
            &[
                package(2, vec![id(1)]),
                package(1, vec![]),
                package(3, vec![]),
            ],
            LIMITS,
        )
        .unwrap();
        assert_eq!(graph.graph().order().unwrap(), vec![id(1), id(2), id(3)]);
        assert!(PackageDependencies::admit(scope(), &[package(2, vec![id(1)])], LIMITS).is_err());
    }
    #[test]
    fn rule_polarities_require_complete_strata() {
        let rules = vec![id(1), id(2)];
        let edge = |n, from, to, polarity| RuleDependency {
            edge: Dependency {
                id: id(n),
                from: id(from),
                to: id(to),
            },
            polarity,
        };
        assert!(
            RuleDependencies::derive(
                scope(),
                rules.clone(),
                vec![
                    edge(10, 1, 2, RulePolarity::Positive),
                    edge(11, 2, 1, RulePolarity::Positive)
                ],
                LIMITS
            )
            .is_ok()
        );
        for polarity in [
            RulePolarity::Negative,
            RulePolarity::Nonmonotone,
            RulePolarity::ConflictSensitive,
        ] {
            assert!(
                RuleDependencies::derive(
                    scope(),
                    rules.clone(),
                    vec![
                        edge(10, 1, 2, polarity),
                        edge(11, 2, 1, RulePolarity::Positive)
                    ],
                    LIMITS
                )
                .is_err()
            );
        }
    }
}
