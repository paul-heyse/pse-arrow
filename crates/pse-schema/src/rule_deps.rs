// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `reference.rule_dependencies`, derived from the rule plans at assembly
//! (blueprint §6.11).
//!
//! The dependency facts are *derived*, never declared a second time: a rule that scans a
//! relation and a `rule_dependencies` row that says so are one fact, and a hand-maintained
//! second list would be the one that goes stale. `read` and `negate` are kept apart
//! because only the second constrains stratification (§14.2 rule 2), and both retain the
//! named input port because "which port" is what makes a read reproducible.
//!
//! Assembly resolves exact rule and relation identities. The row projection identifies
//! each dependency derivation from those identities, its port and its mode.

use crate::model::{DependencyMode, RuleDependency, RuleHead, RuleSpec};
use crate::{Registry, SchemaError};

/// The dependency facts of `rules`, sorted.
pub(crate) fn derive(
    rules: &[RuleSpec],
    registry: &Registry,
) -> Result<Vec<RuleDependency>, SchemaError> {
    let mut out: Vec<RuleDependency> = Vec::new();
    for rule in rules {
        for (relation, port, mode) in rule.plan.dependencies() {
            let relation_id = registry
                .relation(relation)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: rule.qualified_name(),
                    reference: relation.to_owned(),
                })?
                .id;
            out.push(RuleDependency {
                rule_id: rule.id,
                relation: relation.to_owned(),
                relation_id,
                input_port: Some(port),
                mode,
                stratum: rule.stratum,
            });
        }
        if let RuleHead::Relation(relation) = &rule.head {
            let relation_id = registry
                .relation(relation)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: rule.qualified_name(),
                    reference: relation.to_owned(),
                })?
                .id;
            out.push(RuleDependency {
                rule_id: rule.id,
                relation: relation.to_owned(),
                relation_id,
                input_port: None,
                mode: DependencyMode::Write,
                stratum: rule.stratum,
            });
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}
