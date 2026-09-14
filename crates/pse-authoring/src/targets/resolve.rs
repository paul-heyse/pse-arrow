// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{IndexSelector, TargetBudget, TargetContext, TargetPath, TargetRow};
use crate::AuthoringError;
use pse_ids::SemanticId;
use pse_relations::generated::{authored, enums::TargetKind};
use std::collections::BTreeSet;

/// Resolve a target through actual declared rows, retaining identity across name changes.
/// # Errors
/// Missing/ambiguous instances or members, absent domain bindings, wrong template/domain
/// contracts and unknown indices are explicit refusals.
pub fn resolve(
    path: &TargetPath,
    context: &TargetContext,
    source_id: SemanticId,
) -> Result<Vec<TargetRow>, AuthoringError> {
    resolve_with_budget(path, context, source_id, TargetBudget::default(), &|| false)
}

/// Resolve with a checked Cartesian-product limit and caller-owned cancellation.
/// # Errors
/// The target contract errors from `resolve`, exhausted expansion budget or cancellation.
pub fn resolve_with_budget(
    path: &TargetPath,
    context: &TargetContext,
    source_id: SemanticId,
    budget: TargetBudget,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<TargetRow>, AuthoringError> {
    resolve_inner(path, context, source_id, budget, cancelled, &mut |_, _| {
        Ok(())
    })
}
pub(crate) fn resolve_accounted(
    path: &TargetPath,
    context: &TargetContext,
    source_id: SemanticId,
    work: &mut dyn pse_ids::Reservation,
    cancel: &pse_ids::CancellationToken,
) -> Result<Vec<TargetRow>, AuthoringError> {
    cancel.checkpoint()?;
    let result = resolve_inner(
        path,
        context,
        source_id,
        TargetBudget::default(),
        &|| cancel.is_cancelled(),
        &mut |count, arity| {
            cancel.checkpoint()?;
            // Concrete tuple, output Cell row, and simultaneous staging key/pre/post copies.
            let per_row = crate::work::add(4096, crate::work::mul(arity, 128)?)?;
            work.try_grow(crate::work::mul(count, per_row)?)?;
            Ok(())
        },
    );
    cancel.checkpoint()?;
    result
}
fn resolve_inner(
    path: &TargetPath,
    context: &TargetContext,
    source_id: SemanticId,
    budget: TargetBudget,
    cancelled: &dyn Fn() -> bool,
    account: &mut dyn FnMut(usize, usize) -> Result<(), AuthoringError>,
) -> Result<Vec<TargetRow>, AuthoringError> {
    if cancelled() {
        return Err(failure(path, "target resolution cancelled"));
    }
    if budget.max_targets == 0 {
        return Err(failure(path, "target expansion budget is zero"));
    }
    let instance_name = if path.instance_wildcard {
        path.names.join(".")
    } else {
        path.names[..path.names.len().saturating_sub(1)].join(".")
    };
    let instance = instance(context, &instance_name, path)?;
    let mut row = TargetRow {
        spec_id: source_id,
        ordinal: 0,
        instance_id: instance.instance_id,
        member_kind: TargetKind::InstanceWildcard,
        symbol_decl_id: None,
        equation_decl_id: None,
        port_template_id: None,
        port_name: None,
        index: None,
        wildcard: path.instance_wildcard,
    };
    if path.instance_wildcard {
        account(1, 0)?;
        return Ok(vec![row]);
    }
    let name = path
        .names
        .last()
        .ok_or_else(|| failure(path, "missing member"))?;
    let symbols = context
        .symbols
        .iter()
        .filter(|symbol| symbol.template_id == instance.template_id && symbol.name == *name)
        .collect::<Vec<_>>();
    let equations = context
        .equations
        .iter()
        .filter(|equation| equation.template_id == instance.template_id && equation.name == *name)
        .collect::<Vec<_>>();
    let ports = context
        .ports
        .iter()
        .filter(|port| port.template_id == instance.template_id && port.name == *name)
        .collect::<Vec<_>>();
    if symbols.len() + equations.len() + ports.len() != 1 {
        return Err(failure(
            path,
            "target member is missing or ambiguous in the instance template",
        ));
    }
    let indexed_by = if let Some(symbol) = symbols.first() {
        row.symbol_decl_id = Some(symbol.symbol_decl_id);
        row.member_kind = TargetKind::Symbol;
        Some(&symbol.indexed_by)
    } else if let Some(equation) = equations.first() {
        row.equation_decl_id = Some(equation.equation_decl_id);
        row.member_kind = TargetKind::Equation;
        Some(&equation.indexed_by)
    } else if let Some(port) = ports.first() {
        if !path.indices.is_empty() {
            return Err(failure(
                path,
                "port targets have no index tuple declaration",
            ));
        }
        row.member_kind = TargetKind::Port;
        row.port_template_id = Some(port.template_id);
        row.port_name = Some(port.name.clone());
        None
    } else {
        None
    };
    let tuples = if let Some(names) = indexed_by {
        indices(path, context, instance, names, budget, cancelled, account)?
    } else {
        account(1, 0)?;
        vec![None]
    };
    tuples
        .into_iter()
        .enumerate()
        .map(|(ordinal, index)| {
            if cancelled() {
                return Err(failure(path, "target resolution cancelled"));
            }
            let mut row = row.clone();
            row.ordinal = u16::try_from(ordinal)
                .map_err(|_| failure(path, "target ordinal exceeds UInt16"))?;
            row.index = index;
            row.wildcard = false;
            Ok(row)
        })
        .collect()
}

fn instance<'a>(
    context: &'a TargetContext,
    name: &str,
    path: &TargetPath,
) -> Result<&'a authored::instances::Row, AuthoringError> {
    if name.is_empty() {
        return Err(failure(path, "target must name its instance"));
    }
    let matches = context
        .entities
        .iter()
        .filter(|entity| {
            entity.qualified_name == name || entity.qualified_name.ends_with(&format!(".{name}"))
        })
        .flat_map(|entity| {
            context
                .instances
                .iter()
                .filter(move |instance| instance.instance_id == entity.entity_id)
        })
        .collect::<Vec<_>>();
    let [instance] = matches.as_slice() else {
        return Err(failure(path, "target instance is missing or ambiguous"));
    };
    Ok(*instance)
}

fn indices(
    path: &TargetPath,
    context: &TargetContext,
    instance: &authored::instances::Row,
    names: &[String],
    budget: TargetBudget,
    cancelled: &dyn Fn() -> bool,
    account: &mut dyn FnMut(usize, usize) -> Result<(), AuthoringError>,
) -> Result<Vec<Option<Vec<SemanticId>>>, AuthoringError> {
    if names.is_empty() {
        account(1, 0)?;
        return if path.indices.is_empty() {
            Ok(vec![None])
        } else {
            Err(failure(path, "scalar member cannot have indices"))
        };
    }
    if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
        return Err(failure(path, "duplicate indexed domain dimensions"));
    }
    if !path.indices.is_empty() && path.indices.len() != names.len() {
        return Err(failure(path, "index tuple arity differs from indexed_by"));
    }
    let domains = names
        .iter()
        .map(|name| bound_domain(path, context, instance, name))
        .collect::<Result<Vec<_>, _>>()?;
    let mut selections = Vec::new();
    let mut count = 1_u64;
    let allowed = u64::from(budget.max_targets.min(65_536));
    for (axis, domain) in domains.iter().enumerate() {
        if cancelled() {
            return Err(failure(path, "target resolution cancelled"));
        }
        let choices = match path.indices.get(axis) {
            None | Some(IndexSelector::Wildcard) => {
                let mut members = context
                    .members
                    .iter()
                    .filter(|member| member.domain_id == domain.domain_id)
                    .collect::<Vec<_>>();
                members.sort_by_key(|member| (member.ordinal, member.member_id));
                members
                    .into_iter()
                    .map(|member| member.member_id)
                    .collect::<Vec<_>>()
            }
            Some(selector) => vec![member(path, context, selector, domain.domain_id)?],
        };
        if choices.is_empty() || choices.iter().collect::<BTreeSet<_>>().len() != choices.len() {
            return Err(failure(
                path,
                "bound domain has no members or duplicate member identities",
            ));
        }
        count = count.saturating_mul(u64::try_from(choices.len()).unwrap_or(u64::MAX));
        if count > allowed {
            return Err(AuthoringError::Budget {
                limit: "target expansion",
                allowed,
                needed: count,
            });
        }
        selections.push(choices);
    }
    account(
        usize::try_from(count).map_err(|_| failure(path, "target extent exceeds usize"))?,
        names.len(),
    )?;
    let mut tuples = vec![Vec::new()];
    for choices in selections {
        let mut next = Vec::new();
        for tuple in tuples {
            for choice in &choices {
                if cancelled() {
                    return Err(failure(path, "target resolution cancelled"));
                }
                let mut complete = tuple.clone();
                complete.push(*choice);
                next.push(complete);
            }
        }
        tuples = next;
    }
    Ok(tuples.into_iter().map(Some).collect())
}

fn bound_domain<'a>(
    path: &TargetPath,
    context: &'a TargetContext,
    instance: &authored::instances::Row,
    name: &str,
) -> Result<&'a authored::domains::Row, AuthoringError> {
    let declarations = context
        .template_domains
        .iter()
        .filter(|domain| domain.template_id == instance.template_id && domain.name == name)
        .collect::<Vec<_>>();
    let [declaration] = declarations.as_slice() else {
        return Err(failure(
            path,
            "indexed_by does not identify one template domain declaration",
        ));
    };
    let bindings = context
        .domain_bindings
        .iter()
        .filter(|binding| {
            binding.instance_id == instance.instance_id && binding.domain_name == name
        })
        .collect::<Vec<_>>();
    let [binding] = bindings.as_slice() else {
        return Err(failure(
            path,
            "missing or duplicate explicit instance domain binding",
        ));
    };
    let domains = context
        .domains
        .iter()
        .filter(|domain| domain.domain_id == binding.domain_id)
        .collect::<Vec<_>>();
    let [domain] = domains.as_slice() else {
        return Err(failure(
            path,
            "bound domain identity is missing or duplicate",
        ));
    };
    if declaration.kind != domain.kind
        || declaration.continuous != domain.continuous
        || !owner_in_context(domain.owner_entity_id, instance, context)
    {
        return Err(failure(
            path,
            "bound domain kind, continuity or owner differs from the instance context",
        ));
    }
    Ok(*domain)
}
fn owner_in_context(
    owner: SemanticId,
    instance: &authored::instances::Row,
    context: &TargetContext,
) -> bool {
    let mut current = Some(instance.instance_id);
    let mut seen = BTreeSet::new();
    while let Some(id) = current {
        if !seen.insert(id) {
            return false;
        }
        if id == owner {
            return true;
        }
        current = context
            .instances
            .iter()
            .find(|instance| instance.instance_id == id)
            .and_then(|instance| instance.parent_instance_id);
    }
    false
}
fn member(
    path: &TargetPath,
    context: &TargetContext,
    selector: &IndexSelector,
    domain: SemanticId,
) -> Result<SemanticId, AuthoringError> {
    let (value, quoted) = match selector {
        IndexSelector::Value(value) => (value, false),
        IndexSelector::Label(value) => (value, true),
        IndexSelector::Wildcard => return Err(failure(path, "concrete selector required")),
    };
    let id = (!quoted)
        .then(|| SemanticId::parse_hex(value).ok())
        .flatten();
    let coordinate = (!quoted)
        .then(|| value.parse::<f64>().ok().filter(|value| value.is_finite()))
        .flatten();
    let matches = context
        .members
        .iter()
        .filter(|member| {
            member.domain_id == domain
                && (id == Some(member.member_id)
                    || member.label == *value
                    || coordinate
                        .zip(member.coordinate)
                        .is_some_and(|(expected, actual)| expected.to_bits() == actual.to_bits()))
        })
        .collect::<Vec<_>>();
    let [member] = matches.as_slice() else {
        return Err(failure(
            path,
            "selector must match exactly one actual member of the bound domain",
        ));
    };
    Ok(member.member_id)
}
fn failure(path: &TargetPath, reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: Some(path.at),
        reason: format!("target `{}`: {reason}", path.text),
    }
}
