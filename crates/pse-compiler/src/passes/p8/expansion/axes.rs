// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

mod fixed;
use super::super::outputs::Output;
use super::{Inputs, invalid, unique};
use crate::{CompilerError, PassContext};
use pse_mathir::{DomainRef, Payload, ValueRef, index::DomainFacts, relations::LoadedMath};
use pse_quantity::{
    BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet, QuantityRegistry, QuantityTypeId,
    UnitId,
};
use pse_relations::generated::compiled;
use pse_templates::{BindingValue, GroupBinding, InstantiationEnvironment};
use std::collections::BTreeMap;

pub(super) fn domains(
    inputs: &Inputs,
    _ctx: &PassContext<'_>,
) -> Result<BTreeMap<DomainId, DomainFacts>, CompilerError> {
    let domains = &inputs.domains;
    let members = &inputs.domain_members;
    domains
        .iter()
        .map(|domain| {
            let id = domain.domain_id;
            let mut actual = members
                .iter()
                .filter(|member| member.domain_id == id)
                .collect::<Vec<_>>();
            actual.sort_by_key(|row| row.ordinal);
            if actual
                .windows(2)
                .any(|pair| pair[0].ordinal == pair[1].ordinal)
            {
                return Err(invalid("domain member ordinal repeats"));
            }
            Ok((
                DomainId::from_id(id),
                DomainFacts {
                    kind: DomainKind::parse(domain.kind.as_str())
                        .ok_or_else(|| invalid("domain kind is undeclared"))?,
                    continuous: domain.continuous,
                    unit: domain.unit_id.map(UnitId::from_id),
                    members: actual.iter().map(|row| row.member_id).collect(),
                },
            ))
        })
        .collect()
}

pub(super) struct TermAxes {
    pub(super) environment: InstantiationEnvironment,
    pub(super) reductions: Vec<BoundIndexRef>,
    pub(super) fixed_positions: Vec<usize>,
    pub(super) broadcasts: Vec<BoundIndexRef>,
}

pub(super) fn environment(
    inputs: &Inputs,
    loaded: &LoadedMath,
    domains: &BTreeMap<DomainId, DomainFacts>,
    law: &compiled::law_applications::Row,
    contribution: &compiled::contributions::Row,
    law_indices: &[BoundIndexRef],
    output: &mut Output<'_>,
) -> Result<TermAxes, CompilerError> {
    let owner = law.owner_instance_id;
    let root = *loaded
        .node_mapping
        .get(&pse_mathir::NodeId(contribution.expression_root))
        .ok_or_else(|| invalid("law source root absent"))?;
    let mut pending = vec![root];
    let mut used_nodes = std::collections::BTreeSet::new();
    let mut used_groups = std::collections::BTreeSet::new();
    let mut used_symbols = std::collections::BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !used_nodes.insert(id) {
            continue;
        }
        let node = loaded.graph.node(id)?;
        pending.extend(node.children.iter().copied());
        match node.payload {
            Payload::Gather { group, .. } => {
                used_groups.insert(group);
            }
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(id),
            } => {
                used_symbols.insert(id);
            }
            _ => {}
        }
    }
    for index in law_indices {
        support_domain(inputs, index.domain, output)?;
    }
    output.use_row(unique(
        &inputs.domain_products,
        |row| row.product_id == contribution.product_id,
        "contribution product",
    )?);
    let mut environment = InstantiationEnvironment::new(owner);
    environment.domain_facts = domains.clone();
    for domain in domains.keys() {
        environment
            .domains
            .insert(DomainRef::Actual(*domain), *domain);
    }
    let groups = &inputs.symbol_groups;
    let members = &inputs.symbol_group_members;
    let products = &inputs.domain_products;
    for group in groups
        .iter()
        .filter(|row| used_groups.contains(&row.group_id))
    {
        output.use_row(group);
        output.use_row(unique(
            products,
            |row| row.product_id == group.product_id,
            "group product",
        )?);
        let id = group.group_id;
        let axes = unique(
            products,
            |row| row.product_id == group.product_id,
            "group product",
        )?
        .domain_ids
        .clone()
        .into_iter()
        .map(DomainId::from_id)
        .collect::<Vec<_>>();
        let actual = members
            .iter()
            .filter(|member| member.group_id == id)
            .map(|member| {
                output.use_row(member);
                used_symbols.insert(member.symbol_id);
                (member.tuple.clone(), member.symbol_id)
            })
            .collect::<BTreeMap<_, _>>();
        environment.groups.insert(
            id,
            GroupBinding {
                group: id,
                axes,
                members: actual,
            },
        );
    }
    for symbol in inputs
        .symbols
        .iter()
        .filter(|row| used_symbols.contains(&row.symbol_id))
    {
        output.use_row(symbol);
        let id = symbol.symbol_id;
        environment
            .values
            .insert(ValueRef::ActualSymbol(id), BindingValue::Symbol(id));
    }
    for id in loaded.kernel_bindings.keys() {
        environment.kernel_bindings.insert(*id, *id);
    }
    // Bind operator-local reduction coordinates independently of outer occurrence axes.
    for (id, node) in loaded
        .graph
        .iter()
        .filter(|(id, _)| used_nodes.contains(id))
    {
        let _ = id;
        let (domain, binder) = match &node.payload {
            Payload::Reduction {
                domain: DomainRef::Actual(domain),
                bound_index,
                ..
            }
            | Payload::Integral {
                domain: DomainRef::Actual(domain),
                bound_index,
                ..
            }
            | Payload::Broadcast {
                domain: DomainRef::Actual(domain),
                bound_index,
            } => (*domain, *bound_index),
            Payload::ImplicitRef {
                implicit_system, ..
            } => {
                environment
                    .implicit_systems
                    .insert(*implicit_system, *implicit_system);
                continue;
            }
            _ => continue,
        };
        support_domain(inputs, domain, output)?;
        let kind = domains
            .get(&domain)
            .ok_or_else(|| invalid("local law operand domain absent"))?
            .kind;
        environment
            .binders
            .insert(binder, BoundIndexRef::new(binder, domain, kind));
    }
    let source_axes = &inputs.expression_root_indices;
    let id = contribution.contribution_id;
    let mut axes = source_axes
        .iter()
        .filter(|row| row.owner_id == id && row.role.as_str() == "contribution" && row.ordinal == 0)
        .collect::<Vec<_>>();
    axes.sort_by_key(|axis| axis.position);
    let factors = unique(
        products,
        |row| row.product_id == contribution.product_id,
        "contribution product",
    )?
    .domain_ids
    .clone();
    if axes.len() != factors.len() {
        return Err(invalid(
            "contribution root axes differ from its actual declared product",
        ));
    }
    let mut reductions = Vec::new();
    let mut fixed_positions = Vec::new();
    let mut free = Vec::new();
    for (position, axis) in axes.into_iter().enumerate() {
        if usize::from(axis.position) != position || axis.domain_id != factors[position] {
            return Err(invalid(
                "contribution axis order differs from its exact product",
            ));
        }
        output.use_row(axis);
        let domain = DomainId::from_id(axis.domain_id);
        support_domain(inputs, domain, output)?;
        let kind = domains
            .get(&domain)
            .ok_or_else(|| invalid("contribution axis domain absent"))?
            .kind;
        let matching = law_indices
            .iter()
            .filter(|bound| bound.domain == domain)
            .copied()
            .collect::<Vec<_>>();
        let fixed = fixed::member(inputs, law, contribution, position, domain, output)?;
        if fixed.is_some() && !matching.is_empty() {
            return Err(invalid("law axis is both fixed and free"));
        }
        let binding = match matching.as_slice() {
            [bound] => *bound,
            [] => {
                let bound = BoundIndexRef::new(
                    BoundIndexId::from_id(pse_ids::named_id(
                        law.application_id,
                        &format!("pse:law-term-axis:v1:{}:{position}", id.to_hex()),
                    )),
                    domain,
                    kind,
                );
                if fixed.is_none() {
                    reductions.push(bound);
                }
                bound
            }
            _ => {
                return Err(invalid(
                    "a repeated law domain requires an explicit positional mapping",
                ));
            }
        };
        environment
            .binders
            .insert(BoundIndexId::from_id(axis.bound_index_id), binding);
        if let Some(member) = fixed {
            environment
                .fixed_indices
                .insert(BoundIndexId::from_id(axis.bound_index_id), member);
            fixed_positions.push(position);
        } else {
            free.push(binding);
        }
    }
    let mut broadcasts = Vec::new();
    for (position, bound) in law_indices.iter().enumerate() {
        if !free.contains(bound) {
            if law.subject_projection.as_str() == "species_to_element"
                && bound.kind == DomainKind::Element
            {
                continue;
            }
            fixed::broadcast_allowed(law, contribution, position)?;
            broadcasts.push(*bound);
        }
    }
    environment.free_indices =
        IndexSet::try_from_iter(free).map_err(|_| invalid("law operand binder conflict"))?;
    reductions.reverse();
    Ok(TermAxes {
        environment,
        reductions,
        fixed_positions,
        broadcasts,
    })
}

pub(super) fn check_quantity(
    physical: &QuantityRegistry,
    law: &compiled::law_applications::Row,
    contribution: &compiled::contributions::Row,
    axes: &TermAxes,
    checker: &dyn pse_quantity::infer::InvariantChecker,
) -> Result<Option<(pse_quantity::ConversionId, pse_quantity::UnitConvertSpec)>, CompilerError> {
    let mut quantity = projected_quantity(physical, contribution, axes)?;
    let mut indices = axes.environment.free_indices.clone();
    for bound in &axes.reductions {
        let request = pse_quantity::infer::OpRequest::Reduce {
            kind: pse_quantity::ReductionKind::Sum,
            bound: *bound,
        };
        let result = pse_quantity::infer::infer_with_evidence(
            &request,
            &[pse_quantity::infer::Operand {
                quantity_type: quantity,
                indices: &indices,
            }],
            physical,
            checker,
        )?;
        quantity = result.result;
        indices = result.indices;
    }
    for bound in &axes.broadcasts {
        let result = pse_quantity::infer::infer_with_evidence(
            &pse_quantity::infer::OpRequest::Broadcast { index: *bound },
            &[pse_quantity::infer::Operand {
                quantity_type: quantity,
                indices: &indices,
            }],
            physical,
            checker,
        )?;
        quantity = result.result;
        indices = result.indices;
    }
    let expected = QuantityTypeId::from_id(law.quantity_type_id);
    if physical.quantity_type(quantity)?.key == physical.quantity_type(expected)?.key {
        return Ok(None);
    }
    let candidates = physical
        .conversions_between(quantity, expected)
        .collect::<Vec<_>>();
    let [conversion] = candidates.as_slice() else {
        return Err(invalid(
            "law contribution requires exactly one explicit directed conversion of its complete physical contract",
        ));
    };
    if conversion.kind == pse_quantity::ConversionKind::Kernel
        || !conversion.required_parameters.is_empty()
    {
        return Err(invalid(
            "parameterized law basis conversion requires complete actual kernel/parameter binding",
        ));
    }
    Ok(Some((
        conversion.id,
        pse_quantity::UnitConvertSpec {
            from: physical.quantity_type(quantity)?.canonical_unit,
            to: physical.quantity_type(expected)?.canonical_unit,
            scale: conversion
                .scale
                .ok_or_else(|| invalid("physical conversion scale absent"))?,
            offset: conversion.offset.unwrap_or(0.0),
        },
    )))
}

pub(super) fn projected_quantity(
    physical: &QuantityRegistry,
    contribution: &compiled::contributions::Row,
    axes: &TermAxes,
) -> Result<QuantityTypeId, CompilerError> {
    let actual = QuantityTypeId::from_id(contribution.quantity_type_id);
    let mut key = physical.quantity_type(actual)?.key.clone();
    if axes
        .fixed_positions
        .iter()
        .any(|position| *position >= key.shape.len())
    {
        return Err(invalid(
            "fixed contribution axis is absent from its full quantity shape",
        ));
    }
    for position in axes.fixed_positions.iter().rev() {
        key.shape.remove(*position);
    }
    Ok(physical.resolve_key(&key)?)
}

pub(super) fn support_domain(
    inputs: &Inputs,
    domain: DomainId,
    output: &mut Output<'_>,
) -> Result<(), CompilerError> {
    output.use_row(unique(
        &inputs.domains,
        |row| row.domain_id == domain.as_id(),
        "domain",
    )?);
    for member in inputs
        .domain_members
        .iter()
        .filter(|row| row.domain_id == domain.as_id())
    {
        output.use_row(member);
    }
    Ok(())
}
