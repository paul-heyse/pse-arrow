// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Closed producer inventories, finite algebra and located input admission.
use super::{LocatedRuleInput, RuleBindings, RuleInputLocation};
use crate::{RuleError, errmap::internal};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::{
    Registry,
    model::{
        ConflictPolicy, DependencyMode, NegationPolicy, RelationKey, RuleHead, RulePlan, RuleSpec,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    rules: &[RuleSpec],
    bindings: &BTreeMap<SemanticId, RuleBindings>,
    session: &SnapshotSession,
    registry: &Registry,
    outputs: &BTreeSet<RelationKey>,
    cancel: &CancellationToken,
) -> Result<(), RuleError> {
    if !std::ptr::eq(registry, session.registry().as_ref()) {
        return Err(internal("rule program belongs to a different registry"));
    }
    let ids = rules.iter().map(|rule| rule.id).collect::<BTreeSet<_>>();
    if ids.len() != rules.len() || ids.iter().ne(bindings.keys()) {
        return Err(internal("rule and binding inventories differ or repeat"));
    }
    let mut expected_outputs = BTreeSet::new();
    for name in ["inferred.rule_outcomes", "provenance.rule_support_edges"] {
        expected_outputs.insert(
            registry
                .relation(name)
                .ok_or_else(|| internal("rule evidence relation is undeclared"))?
                .key,
        );
    }
    for rule in rules {
        if registry.rule(&rule.qualified_name()) != Some(rule) {
            return Err(internal(
                "rule differs from its actual registry declaration",
            ));
        }
        let RuleHead::Relation(head) = &rule.head else {
            return Err(internal("invariant violations use their own executor"));
        };
        let target = registry
            .relation(head)
            .ok_or_else(|| internal("head is undeclared"))?;
        let assertion =
            registry
                .relation(rule.assertion_relation.as_deref().ok_or_else(|| {
                    internal("relation rule requires an explicit assertion relation")
                })?)
                .ok_or_else(|| internal("assertion is undeclared"))?;
        expected_outputs.extend([target.key, assertion.key]);
        if target
            .columns
            .iter()
            .any(|column| ["assertion_id", "rule_id", "truth"].contains(&column.name()))
        {
            return Err(internal(
                "head payload collides with assertion identity columns",
            ));
        }
        for producer in registry
            .rules()
            .iter()
            .filter(|producer| producer.head == rule.head)
        {
            if !ids.contains(&producer.id) {
                return Err(internal(
                    "rule program omits a competing producer of an active head",
                ));
            }
        }
        let input = bindings
            .get(&rule.id)
            .ok_or_else(|| internal("rule has no bindings"))?;
        validate_rule_inputs(rule, input, rules, session, registry, cancel)?;
    }
    if &expected_outputs != outputs {
        return Err(internal(
            "pass output inventory differs from complete heads/assertions/evidence",
        ));
    }
    Ok(())
}
fn validate_rule_inputs(
    rule: &RuleSpec,
    input: &RuleBindings,
    rules: &[RuleSpec],
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<(), RuleError> {
    let deps = rule.plan.dependencies();
    let ports = deps
        .iter()
        .map(|(_, port, _)| *port)
        .collect::<BTreeSet<_>>();
    if ports
        .iter()
        .copied()
        .ne(input.ports.keys().map(String::as_str))
    {
        return Err(internal(
            "rule input-port inventory differs from declared scans",
        ));
    }
    for (name, port, mode) in deps {
        let bound = input
            .ports
            .get(port)
            .ok_or_else(|| internal("unbound scan"))?;
        let source = registry
            .relation(name)
            .ok_or_else(|| internal("undeclared scan"))?;
        if bound.relation != source.key {
            return Err(internal(
                "scan binding differs from declared relation/version",
            ));
        }
        match &bound.location {
            RuleInputLocation::Facts(_) => {
                validate_immutable(bound, session, registry)?;
            }
            RuleInputLocation::Completed(input) => {
                input.validate(source.key, session, cancel)?;
            }
            RuleInputLocation::Native(input) => input.validate(source.key, session, cancel)?,
            RuleInputLocation::Workspace => {
                if !rules.iter().any(|writer| writer.head.relation() == name) {
                    return Err(internal(
                        "workspace input has no declared producer in this program",
                    ));
                }
            }
        }
        if mode == DependencyMode::Negate && rule.negation != NegationPolicy::Stratified {
            return Err(internal("negation is not declared stratified"));
        }
        for writer in rules.iter().filter(|writer| writer.head.relation() == name) {
            if writer.stratum > rule.stratum
                || (mode == DependencyMode::Negate && writer.stratum >= rule.stratum)
            {
                return Err(internal("rule reads an unsettled stratum"));
            }
            if writer.stratum == rule.stratum
                && (writer.conflict_policy == ConflictPolicy::Undecided
                    || !rule.monotonic
                    || !monotone(&rule.plan, name))
            {
                return Err(internal(
                    "same-stratum consumer is non-monotone or reads an undecided-policy head",
                ));
            }
            if !matches!(bound.location, RuleInputLocation::Workspace) {
                return Err(internal(
                    "an active producer must bind its finite workspace",
                ));
            }
        }
    }
    Ok(())
}

fn monotone(plan: &RulePlan, relation: &str) -> bool {
    match plan {
        RulePlan::Aggregate { input, .. } => !input
            .dependencies()
            .iter()
            .any(|(name, _, _)| *name == relation),
        RulePlan::AntiJoin { left, right, .. } => {
            !right
                .dependencies()
                .iter()
                .any(|(name, _, _)| *name == relation)
                && monotone(left, relation)
        }
        other => other
            .children()
            .into_iter()
            .all(|child| monotone(child, relation)),
    }
}

pub(super) fn validate_immutable(
    bound: &LocatedRuleInput,
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<(), RuleError> {
    validate_immutable_owner(bound, registry)?;
    let RuleInputLocation::Facts(facts) = &bound.location else {
        return Err(internal("immutable source owner absent"));
    };
    session.validate_input_values(&bound.relation, facts.checked().batch())?;
    Ok(())
}

// Native witnesses scan retained owners under distinct roles. Their membership
// does not depend on the relation-only binding used by ordinary rule scans.
pub(super) fn validate_immutable_owner(
    bound: &LocatedRuleInput,
    registry: &Registry,
) -> Result<(), RuleError> {
    let source = registry
        .relation_by_key(bound.relation)
        .ok_or_else(|| internal("immutable source schema/version absent"))?;
    let RuleInputLocation::Facts(facts) = &bound.location else {
        return Err(internal("source must own actual immutable facts"));
    };
    facts.checked().check_declaration(registry, source)?;
    Ok(())
}
