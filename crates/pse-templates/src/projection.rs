// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{GroupBinding, InstantiationEnvironment, TemplateError, identity};
use pse_ids::SemanticId;
use std::collections::BTreeMap;

/// Complete projected subgroup and its reversible source/member correspondence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupProjection {
    /// Original actual group identity.
    pub source_group: SemanticId,
    /// Ordered fixed source-axis positions and actual domain members.
    pub fixed_coordinates: Vec<(i64, SemanticId)>,
    /// Exact projected factors and tuple-to-provider inventory.
    pub group: GroupBinding,
}

/// Restrict an actual group and remove exactly the explicitly fixed axes.
/// # Errors
/// Unknown/unsorted/repeated axes, foreign members or conflicting projected providers.
pub fn project_group(
    source: &GroupBinding,
    fixed: &[(i64, SemanticId)],
    env: &InstantiationEnvironment,
) -> Result<GroupProjection, TemplateError> {
    if fixed.is_empty() || fixed.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(env.invalid("group projection requires sorted unique fixed axes"));
    }
    for (axis, member) in fixed {
        let domain = source
            .axes
            .get(usize::try_from(*axis).map_err(|_| env.invalid("negative projected axis"))?)
            .ok_or_else(|| env.invalid("projected axis absent"))?;
        let facts = env
            .domain_facts
            .get(domain)
            .ok_or_else(|| env.invalid("projected domain absent"))?;
        if !facts.members.contains(member) {
            return Err(env.invalid("fixed coordinate is not an actual domain member"));
        }
    }
    let remaining = source
        .axes
        .iter()
        .enumerate()
        .filter(|(axis, _)| {
            !fixed
                .iter()
                .any(|(fixed, _)| usize::try_from(*fixed).ok() == Some(*axis))
        })
        .collect::<Vec<_>>();
    let mut members = BTreeMap::new();
    for (tuple, symbol) in &source.members {
        if tuple.len() != source.axes.len() {
            return Err(env.invalid("source group tuple arity differs"));
        }
        for (domain, member) in source.axes.iter().zip(tuple) {
            if !env
                .domain_facts
                .get(domain)
                .is_some_and(|facts| facts.members.contains(member))
            {
                return Err(env.invalid("source projection contains a foreign actual member"));
            }
        }
        if fixed.iter().all(|(axis, member)| {
            usize::try_from(*axis).ok().and_then(|axis| tuple.get(axis)) == Some(member)
        }) {
            let projected = remaining
                .iter()
                .map(|(axis, _)| tuple[*axis])
                .collect::<Vec<_>>();
            if members
                .insert(projected, *symbol)
                .is_some_and(|old| old != *symbol)
            {
                return Err(env.invalid("projected tuple has conflicting actual providers"));
            }
        }
    }
    Ok(GroupProjection {
        source_group: source.group,
        fixed_coordinates: fixed.to_vec(),
        group: GroupBinding {
            group: identity::projected_group_id(source.group, fixed),
            axes: remaining.iter().map(|(_, domain)| **domain).collect(),
            members,
        },
    })
}
