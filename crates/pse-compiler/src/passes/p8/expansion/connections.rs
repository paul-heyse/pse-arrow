// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::super::{inventory::Located, outputs::Output};
use super::{Inputs, axes, invalid, typed_zero, unique};
use crate::{CompilerError, PassContext};
use pse_ids::{Reservation, SemanticId};
use pse_mathir::{
    ExprGraph, Opcode, Payload,
    equation::{EquationRecord, FreeIndex, Sense},
};
use pse_quantity::{
    BoundIndexId, BoundIndexRef, DomainId, IndexSet, QuantityRegistry, QuantityTypeId,
};
use pse_relations::generated::{compiled, inferred};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn expand(
    inputs: &Inputs,
    ctx: &PassContext<'_>,
    physical: &QuantityRegistry,
    graph: &mut ExprGraph,
    equations: &mut Vec<EquationRecord>,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<(), CompilerError> {
    let connections = &inputs.connections;
    let bindings = &inputs.connection_bindings;
    let ports = &inputs.ports;
    let members = &inputs.port_members;
    let member_domains = &inputs.port_member_domains;
    let domains = axes::domains(inputs, ctx)?;
    let mut groups: BTreeMap<_, (SemanticId, super::super::outputs::Support)> = BTreeMap::new();
    for connection in connections {
        ctx.cancel.checkpoint()?;
        let connection_id = connection.connection_id;
        output.active.clear();
        output.use_row(connection);
        let binding = unique(
            bindings,
            |row| row.rule_template_id == connection.rule_template_id,
            "connection binding",
        )?;
        output.use_row(binding);
        if binding.expansion.as_str() != "equality" {
            return Err(invalid(
                "connection expansion has no declared implementation",
            ));
        }
        let from = unique(
            ports,
            |row| row.port_id == connection.from_port_id,
            "connection from port",
        )?;
        let to = unique(
            ports,
            |row| row.port_id == connection.to_port_id,
            "connection to port",
        )?;
        output.use_row(from);
        output.use_row(to);
        let connection_support = output.active.clone();
        if from.kind != to.kind
            || !matches!(from.direction.as_str(), "outlet" | "bidirectional")
            || !matches!(to.direction.as_str(), "inlet" | "bidirectional")
        {
            return Err(invalid(
                "connection endpoint kinds or actual directions disagree",
            ));
        }
        let mut left = members
            .iter()
            .filter(|row| row.port_id == from.port_id)
            .collect::<Vec<_>>();
        let right = members
            .iter()
            .filter(|row| row.port_id == to.port_id)
            .collect::<Vec<_>>();
        left.sort_by_key(|member| member.ordinal);
        if left.len() != right.len() {
            return Err(invalid("connection member inventories differ"));
        }
        for (position, member) in left.into_iter().enumerate() {
            let ordinal = member.ordinal;
            output.active.clone_from(&connection_support);
            output.use_row(member);
            if usize::from(ordinal) != position {
                return Err(invalid("port member ordinal has a gap"));
            }
            let other = right
                .iter()
                .filter(|row| row.ordinal == member.ordinal)
                .copied()
                .collect::<Vec<_>>();
            let [other] = other.as_slice() else {
                return Err(invalid("connection member ordinal absent or repeated"));
            };
            output.use_row(other);
            let a = member_domain(member_domains, from.port_id, ordinal)?;
            let b = member_domain(member_domains, to.port_id, ordinal)?;
            output.use_row(a);
            output.use_row(b);
            if a.domain_ids != b.domain_ids || a.product_id != b.product_id {
                return Err(invalid("connection uses different actual ordered domains"));
            }
            let left_type =
                physical.quantity_type(QuantityTypeId::from_id(member.quantity_type_id))?;
            let right_type =
                physical.quantity_type(QuantityTypeId::from_id(other.quantity_type_id))?;
            if left_type.key != right_type.key {
                return Err(invalid(
                    "connection members differ in complete physical type",
                ));
            }
            let equation_id = pse_ids::named_id(
                connection_id,
                &format!("pse:connection-member-equation:v1:{ordinal}"),
            );
            let factors = a.domain_ids.clone();
            let indices = factors
                .iter()
                .enumerate()
                .map(|(position, domain)| {
                    let domain = DomainId::from_id(*domain);
                    axes::support_domain(inputs, domain, output)?;
                    Ok(BoundIndexRef::new(
                        BoundIndexId::from_id(pse_ids::named_id(
                            equation_id,
                            &format!("pse:connection-axis:v1:{position}"),
                        )),
                        domain,
                        domains
                            .get(&domain)
                            .ok_or_else(|| invalid("connection domain absent"))?
                            .kind,
                    ))
                })
                .collect::<Result<Vec<_>, CompilerError>>()?;
            let mut quantity_key = left_type.key.clone();
            quantity_key.shape = indices.iter().map(|index| index.kind).collect();
            let quantity = physical.resolve_key(&quantity_key)?;
            let mut roots = Vec::new();
            for (port, member) in [(from, member), (to, *other)] {
                let key = (port.port_id, ordinal);
                let group = if let Some((group, support)) = groups.get(&key) {
                    output.active.extend(support.iter().cloned());
                    *group
                } else {
                    let group = group(
                        inputs, ctx, physical, port, member, a, quantity, output, work,
                    )?;
                    groups.insert(key, (group, output.active.clone()));
                    group
                };
                let coordinate_map = indices
                    .iter()
                    .enumerate()
                    .map(|(position, index)| {
                        Ok((
                            index.bound_index,
                            u16::try_from(position)
                                .map_err(|_| invalid("connection axis exceeds u16"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, CompilerError>>()?;
                roots.push(graph.insert(
                    Opcode::Gather,
                    Payload::Gather {
                        group,
                        coordinate_map,
                    },
                    &[],
                    Some(from.instance_id),
                )?);
            }
            let [left, right] = roots.as_slice() else {
                return Err(invalid("connection endpoint count differs"));
            };
            let free = IndexSet::try_from_iter(indices.iter().copied())
                .map_err(|_| invalid("connection binder conflict"))?;
            let operand = pse_quantity::infer::Operand {
                quantity_type: quantity,
                indices: &free,
            };
            let residual = pse_quantity::infer::infer(
                &pse_quantity::infer::OpRequest::Sub,
                &[operand, operand],
                physical,
            )?
            .result;
            let body = graph.sub(*left, *right)?;
            let zero = typed_zero(graph, residual, &free, physical)?;
            output.record_graph(graph, [body, zero])?;
            output.equations.insert(equation_id, output.active.clone());
            equations.push(EquationRecord {
                indexed_equation_id: equation_id,
                owner_instance: from.instance_id,
                equation_decl: None,
                qualified_name: format!("connection:{}:{ordinal}", connection_id.to_hex()),
                product: Some(a.product_id),
                filter: None,
                body,
                sense: Sense::Eq,
                lower: Some(zero),
                upper: Some(zero),
                free_indices: indices
                    .iter()
                    .enumerate()
                    .map(|(position, index)| {
                        Ok(FreeIndex {
                            bound_index: index.bound_index,
                            domain: index.domain,
                            position: u16::try_from(position)
                                .map_err(|_| invalid("connection axis exceeds u16"))?,
                        })
                    })
                    .collect::<Result<_, CompilerError>>()?,
                residual_quantity_type: None,
                law_instance: None,
                derivation: pse_ids::named_id(equation_id, "P8:connection"),
            });
            output.push(inferred::connection_equations::Row {
                connection_id,
                member_ordinal: ordinal,
                product_id: a.product_id,
                equation_id,
            })?;
        }
    }
    Ok(())
}

fn member_domain(
    rows: &[Located<inferred::port_member_domains::Row>],
    port: SemanticId,
    ordinal: u16,
) -> Result<&Located<inferred::port_member_domains::Row>, CompilerError> {
    unique(
        rows,
        |row| row.port_id == port && row.ordinal == ordinal,
        "port member domains",
    )
}
fn group(
    inputs: &Inputs,
    _ctx: &PassContext<'_>,
    physical: &QuantityRegistry,
    port: &inferred::ports::Row,
    member: &inferred::port_members::Row,
    domains: &inferred::port_member_domains::Row,
    quantity: QuantityTypeId,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<SemanticId, CompilerError> {
    let port_id = port.port_id;
    let ordinal = member.ordinal;
    let group = pse_ids::named_id(port_id, &format!("pse:port-member-group:v1:{ordinal}"));
    let targets = &inputs.port_state_targets;
    let symbols = &inputs.symbols;
    let mut mapped = BTreeMap::new();
    let mut scalar_key = physical.quantity_type(quantity)?.key.clone();
    scalar_key.shape.clear();
    for target in targets.iter().filter(|target| target.port_id == port_id) {
        output.use_row(target);
        let state = target.state_instance_id;
        for symbol in symbols.iter().filter(|symbol| {
            symbol.owner_instance_id == state && symbol.symbol_decl_id == member.symbol_decl_id
        }) {
            output.use_row(symbol);
            if physical
                .quantity_type(QuantityTypeId::from_id(symbol.quantity_type_id))?
                .key
                != scalar_key
            {
                return Err(invalid(
                    "actual connection provider differs from promised complete member type",
                ));
            }
            let tuple = target
                .state_index
                .clone()
                .into_iter()
                .chain(symbol.index.clone())
                .collect::<Vec<_>>();
            if mapped.insert(tuple, symbol.symbol_id).is_some() {
                return Err(invalid("port tuple has multiple actual providers"));
            }
        }
    }
    let tuples = &inputs.valid_index_tuples;
    let expected = tuples
        .iter()
        .filter(|tuple| tuple.product_id == domains.product_id)
        .map(|tuple| {
            output.use_row(tuple);
            tuple.tuple.clone()
        })
        .collect::<BTreeSet<_>>();
    if mapped.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(invalid(
            "port group differs from complete actual valid tuple/provider inventory",
        ));
    }
    work.try_grow(
        mapped
            .len()
            .checked_mul(8192)
            .ok_or_else(|| invalid("port group extent overflow"))?,
    )
    .map_err(pse_catalog::CatalogError::from)?;
    output.push(compiled::port_member_groups::Row {
        group_id: group,
        port_id,
        ordinal,
        quantity_type_id: quantity.as_id(),
        product_id: domains.product_id,
        derivation_id: pse_ids::named_id(group, "P8:port-group"),
    })?;
    output.push(compiled::symbol_groups::Row {
        group_id: group,
        owner_instance_id: port.instance_id,
        name: format!("port:{}:{ordinal}", port_id.to_hex()),
        product_id: domains.product_id,
    })?;
    for (tuple, symbol_id) in mapped {
        output.push(compiled::symbol_group_members::Row {
            group_id: group,
            tuple,
            symbol_id,
        })?;
    }

    Ok(group)
}
