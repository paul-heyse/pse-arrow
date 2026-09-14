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
//! Packet C-3 extends this with the derivation identities the `derivation_id` column
//! carries once `provenance.derivations` exists.

use crate::model::{DependencyMode, RuleDependency, RuleSpec};

/// The dependency facts of `rules`, sorted.
pub(crate) fn derive(rules: &[RuleSpec]) -> Vec<RuleDependency> {
    let mut out: Vec<RuleDependency> = Vec::new();
    for rule in rules {
        let name: &'static str = rule.name;
        for (relation, port, mode) in rule.plan.dependencies() {
            out.push(RuleDependency {
                rule: name,
                relation,
                input_port: Some(port),
                mode,
                stratum: rule.stratum,
            });
        }
        out.push(RuleDependency {
            rule: name,
            relation: rule.head.relation(),
            input_port: None,
            mode: DependencyMode::Write,
            stratum: rule.stratum,
        });
    }
    out.sort_unstable();
    out.dedup();
    out
}
