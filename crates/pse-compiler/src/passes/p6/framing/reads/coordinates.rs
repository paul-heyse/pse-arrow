// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The shared evaluator consumes native finite assignments and returns exact sources.
use super::{
    Axis, BTreeMap, BTreeSet, CompilerError, Context, Coordinate, Extra, IndexEvaluator, Inventory,
    NodeId, Payload, Read, SemanticId, Support, Target, ValueRef, invalid, n, unique,
};
use crate::mathir_relations::domain::DomainValue;
use pse_ids::CancellationToken;
use pse_relations::generated::enums::PathTargetKind;

type Assignment = (BTreeMap<SemanticId, SemanticId>, Support);
pub(super) async fn assignments(
    evaluator: &mut IndexEvaluator<'_>,
    extra: &Extra,
    context: &Context<'_>,
    cancel: &CancellationToken,
) -> Result<Vec<Assignment>, CompilerError> {
    let read = context.read;
    let mut wanted = BTreeSet::new();
    if let Some(node) = read.node {
        wanted.extend(
            evaluator
                .free_indices(read.source, NodeId(node))?
                .into_iter()
                .map(|binder| (read.source, binder)),
        );
    }
    let mut support = Support::new();
    if let Some((source, predicate)) = read.guard {
        for axis in &extra.axes {
            if axis.row.source_id == source && axis.row.predicate_id == predicate {
                wanted.insert((source, axis.row.bound_index_id));
                support.insert(evaluator.inventory.origin(axis)?);
            }
        }
    }
    let mut binders = Vec::new();
    let mut domains = Vec::new();
    let mut fixed = BTreeMap::new();
    for (source, id) in wanted {
        let binder = evaluator.inventory.binder(source, id)?;
        support.insert(evaluator.inventory.origin(binder)?);
        let domain = evaluator.inventory.domain(
            &context.instance.row,
            &binder.row.domain.domain_ref()?,
            &mut support,
        )?;
        let members = evaluator.inventory.members(domain, &mut support)?;
        if members.is_empty() {
            return Err(invalid("read source binder has no finite actual members"));
        }
        if read.node.is_none() {
            let matching = read
                .index
                .iter()
                .flatten()
                .filter(|id| members.iter().any(|member| member.row.member_id == **id))
                .copied()
                .collect::<Vec<_>>();
            match matching.as_slice() {
                [member] => {
                    fixed.insert(domains.len(), *member);
                }
                [] => {}
                _ => {
                    return Err(invalid(
                        "opaque demand repeats a guard source domain ambiguously",
                    ));
                }
            }
        }
        if binders.contains(&id) {
            return Err(invalid(
                "read and guard share a binder identity across distinct sources",
            ));
        }
        binders.push(id);
        domains.push(domain);
    }
    let tuples = evaluator
        .inventory
        .tuples_filtered(&domains, &fixed, cancel)
        .await?;
    Ok(tuples
        .into_iter()
        .map(|(tuple, actual)| {
            let mut support = support.clone();
            support.extend(actual);
            (binders.iter().copied().zip(tuple).collect(), support)
        })
        .collect())
}

pub(super) fn guard_index(
    extra: &Extra,
    read: &Read,
    bindings: &BTreeMap<SemanticId, SemanticId>,
) -> Result<Vec<SemanticId>, CompilerError> {
    let Some((source, predicate)) = read.guard else {
        return Ok(Vec::new());
    };
    let mut axes = extra
        .axes
        .iter()
        .filter(|row| row.row.source_id == source && row.row.predicate_id == predicate)
        .collect::<Vec<_>>();
    axes.sort_by_key(|axis| axis.row.position);
    axes.into_iter()
        .enumerate()
        .map(|(position, axis)| {
            if usize::try_from(axis.row.position).ok() != Some(position) {
                return Err(invalid("guard axis positions are incomplete"));
            }
            bindings
                .get(&axis.row.bound_index_id)
                .copied()
                .ok_or_else(|| invalid("guard coordinate has no exact binder assignment"))
        })
        .collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "targets keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) fn targets(
    evaluator: &IndexEvaluator<'_>,
    extra: &Extra,
    context: &Context<'_>,
    bindings: &BTreeMap<SemanticId, SemanticId>,
) -> Result<Option<Vec<Target>>, CompilerError> {
    let inventory = &evaluator.inventory;
    let read = context.read;
    let Some(node) = read.node else {
        let declaration = unique(
            &extra.opaque,
            |row| row.requirement_id == read.source,
            "opaque requirement",
        )?;
        let index = read
            .index
            .as_ref()
            .ok_or_else(|| invalid("opaque demand has no explicit index"))?;
        if index.len() != declaration.row.index_domain_bindings.len() {
            return Err(invalid(
                "opaque index differs from its explicit domain binding",
            ));
        }
        let axes = index
            .iter()
            .zip(&declaration.row.index_domain_bindings)
            .map(|(member, binding)| axis(inventory, binding.domain_id, *member))
            .collect::<Result<_, _>>()?;
        return Ok(Some(vec![Target {
            owner: context.instance.row.instance_id,
            symbol: None,
            axes,
            support: BTreeSet::from([inventory.origin(declaration)?]),
        }]));
    };
    let mut support = Support::new();
    inventory.node_support(read.source, NodeId(node), &mut support)?;
    let (symbol, coordinates) = match evaluator.payload(read.source, NodeId(node))? {
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(symbol),
        } => (symbol, Vec::new()),
        Payload::PendingGather { group, indices } => (
            group,
            evaluate(evaluator, context, &indices, bindings, &mut support)?,
        ),
        Payload::Gather {
            group,
            coordinate_map,
        } => {
            let mut ordered = coordinate_map;
            ordered.sort_by_key(|(_, position)| *position);
            let coordinates = ordered
                .into_iter()
                .enumerate()
                .map(|(position, (binder, actual))| {
                    if usize::from(actual) != position {
                        return Err(invalid("gather coordinate map is incomplete"));
                    }
                    bindings
                        .get(&binder.as_id())
                        .copied()
                        .map(Coordinate::Member)
                        .ok_or_else(|| invalid("read gather binder absent"))
                })
                .collect::<Result<_, _>>()?;
            (group, coordinates)
        }
        Payload::PendingPath {
            source_id,
            path_id,
            indices,
        } if source_id == read.source && read.path == Some(path_id) => {
            let coordinates = evaluate(evaluator, context, &indices, bindings, &mut support)?;
            return path_targets(inventory, extra, context, &coordinates, &support);
        }
        _ => return Err(invalid("demand node is not its claimed exact source read")),
    };
    if read.symbol != Some(symbol) {
        return Err(invalid("read node names another symbol declaration"));
    }
    let declaration = unique(
        &inventory.symbols,
        |row| row.symbol_decl_id == symbol,
        "read symbol",
    )?;
    if declaration.row.template_id != context.instance.row.template_id {
        return Err(invalid("direct read symbol belongs to another template"));
    }
    if declaration.row.indexed_by.len() != coordinates.len() {
        return Err(invalid("read does not bind every declaration axis"));
    }
    support.insert(inventory.origin(declaration)?);
    let mut axes = Vec::new();
    for (name, coordinate) in declaration.row.indexed_by.iter().zip(coordinates) {
        let binding = unique(
            &inventory.domain_bindings,
            |row| row.instance_id == context.instance.row.instance_id && row.domain_name == *name,
            "read symbol axis",
        )?;
        support.insert(inventory.origin(binding)?);
        let Some(resolved) = resolve(inventory, binding.row.domain_id, &coordinate)? else {
            return Ok(None);
        };
        axes.push(resolved);
    }
    Ok(Some(vec![Target {
        owner: context.instance.row.instance_id,
        symbol: Some(symbol),
        axes,
        support,
    }]))
}

fn evaluate(
    evaluator: &IndexEvaluator<'_>,
    context: &Context<'_>,
    nodes: &[NodeId],
    bindings: &BTreeMap<SemanticId, SemanticId>,
    support: &mut Support,
) -> Result<Vec<Coordinate>, CompilerError> {
    nodes
        .iter()
        .map(|node| {
            let (value, actual) = evaluator.evaluate_supported(
                context.instance.row.instance_id,
                context.read.source,
                *node,
                bindings,
            )?;
            support.extend(actual);
            Ok(value)
        })
        .collect()
}
fn axis(
    inventory: &Inventory<'_>,
    domain: SemanticId,
    member: SemanticId,
) -> Result<Axis, CompilerError> {
    let member = unique(
        &inventory.members,
        |row| row.domain_id == domain && row.member_id == member,
        "read coordinate member",
    )?;
    let domain = unique(
        &inventory.domains,
        |row| row.domain_id == domain,
        "read coordinate domain",
    )?;
    Ok(Axis {
        domain: domain.row.domain_id,
        member: member.row.member_id,
        kind: domain.row.kind,
        subject: member.row.ref_entity_id,
        support: BTreeSet::from([inventory.origin(member)?, inventory.origin(domain)?]),
    })
}
fn matches(member: &n::domain_members::Row, coordinate: &Coordinate) -> bool {
    match coordinate {
        Coordinate::Member(id) => member.member_id == *id,
        Coordinate::Integer(value) => {
            member
                .coordinate
                .and_then(pse_quantity::numeric::exact_i64_from_f64)
                == Some(*value)
        }
    }
}
fn resolve(
    inventory: &Inventory<'_>,
    domain: SemanticId,
    coordinate: &Coordinate,
) -> Result<Option<Axis>, CompilerError> {
    let found = inventory
        .members
        .iter()
        .filter(|row| row.row.domain_id == domain && matches(&row.row, coordinate))
        .collect::<Vec<_>>();
    match found.as_slice() {
        [] => Ok(None),
        [member] => axis(inventory, domain, member.row.member_id).map(Some),
        _ => Err(invalid(
            "read coordinate has no unique actual domain member",
        )),
    }
}
fn path_targets(
    inventory: &Inventory<'_>,
    extra: &Extra,
    context: &Context<'_>,
    coordinates: &[Coordinate],
    support: &Support,
) -> Result<Option<Vec<Target>>, CompilerError> {
    let mut targets = Vec::new();
    let mut non_property = false;
    for path in &extra.paths {
        let row = &path.row;
        if row.requester_instance_id != context.instance.row.instance_id
            || row.source_id != context.read.source
            || Some(row.path_id) != context.read.path
        {
            continue;
        }
        if row.path_domains.len() != row.path_index.len()
            || row.path_index.len() != coordinates.len()
        {
            return Err(invalid(
                "path domain/member/coordinate partition is incomplete",
            ));
        }
        let mut exact = true;
        let mut actual = support.clone();
        actual.insert(inventory.origin(path)?);
        for ((member, domain), coordinate) in row
            .path_index
            .iter()
            .zip(&row.path_domains)
            .zip(coordinates)
        {
            let member = unique(
                &inventory.members,
                |row| row.domain_id == *domain && row.member_id == *member,
                "path coordinate member",
            )?;
            exact &= matches(&member.row, coordinate);
            actual.insert(inventory.origin(member)?);
        }
        if !exact {
            continue;
        }
        if row.target_kind != PathTargetKind::Symbol {
            non_property = true;
            continue;
        }
        let offset = row
            .path_domains
            .len()
            .checked_sub(row.source_index.len())
            .ok_or_else(|| invalid("path leaf has more axes than its full path"))?;
        let axes = row
            .source_index
            .iter()
            .zip(&row.path_domains[offset..])
            .map(|(member, domain)| axis(inventory, *domain, *member))
            .collect::<Result<_, _>>()?;
        targets.push(Target {
            owner: row.owner_instance_id,
            symbol: row.symbol_decl_id,
            axes,
            support: actual,
        });
    }
    Ok((non_property || !targets.is_empty()).then_some(targets))
}
