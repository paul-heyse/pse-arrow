// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Element conservation over actual composition coefficients and explicit free axes.
mod coefficients;
mod physical;
use super::super::outputs::Output;
use super::{Inputs, invalid};
use crate::{AlgorithmContext, CompilerError};
use pse_ids::{Reservation, SemanticId};
use pse_mathir::{ExprGraph, NodeId, Opcode, Payload, payload::AffineTerm, relations::LoadedMath};
use pse_quantity::{
    BoundIndexRef, DomainKind, IndexSet, QuantityRegistry, QuantityTypeId, infer::InvariantChecker,
};
use pse_relations::generated::compiled;
use pse_templates::{InstantiationEnvironment, laws::ConservationBody};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Context<'a, 'b> {
    pub(super) inputs: &'a Inputs,
    pub(super) ctx: &'a AlgorithmContext<'b>,
    pub(super) source: &'a LoadedMath,
    pub(super) physical: &'a QuantityRegistry,
    pub(super) checker: &'a (dyn InvariantChecker + Sync),
}
pub(super) struct Term<'a> {
    pub(super) contribution: &'a compiled::contributions::Row,
    pub(super) environment: InstantiationEnvironment,
    pub(super) reductions: Vec<BoundIndexRef>,
    pub(super) sign: i32,
    pub(super) source_quantity: QuantityTypeId,
}
/// Actual numerical assembly follows complete source-backed relational coefficient joins.
pub(super) async fn conservation(
    context: &Context<'_, '_>,
    law: &compiled::law_applications::Row,
    free: &IndexSet,
    terms: &[Term<'_>],
    destination: &mut ExprGraph,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<ConservationBody, CompilerError> {
    if terms.is_empty() || law.subject_projection.as_str() != "species_to_element" {
        return Err(invalid(
            "element projection requires declared nonempty species contributions",
        ));
    }
    let element_axes = free
        .iter()
        .filter(|axis| axis.kind == DomainKind::Element)
        .copied()
        .collect::<Vec<_>>();
    let [_] = element_axes.as_slice() else {
        return Err(invalid(
            "element law must retain exactly one actual Element axis",
        ));
    };
    let owner = law.owner_instance_id;
    let mut affine = Vec::new();
    let mut kernels = BTreeMap::new();
    let mut selections = Vec::new();
    let mut seen = BTreeSet::new();
    let mut projections = BTreeMap::new();
    for term in terms {
        context.ctx.cancel.checkpoint()?;
        let id = term.contribution.contribution_id;
        if !seen.insert(id) || !matches!(term.sign, -1 | 1) {
            return Err(invalid("element descriptor repeats or has a non-unit sign"));
        }
        let assembled = contribution(context, law, free, term, destination, output, work).await?;
        for (key, value) in assembled.kernel_bindings {
            if kernels
                .insert(key, value.clone())
                .is_some_and(|old| old != value)
            {
                return Err(invalid(
                    "element kernel occurrence has conflicting bindings",
                ));
            }
        }
        selections.extend(assembled.selections);
        for (id, projection) in assembled.projected_groups {
            if projections
                .insert(id, projection.clone())
                .is_some_and(|old| old != projection)
            {
                return Err(invalid("element source group projections conflict"));
            }
        }
        affine.push(AffineTerm {
            coefficient: f64::from(term.sign),
            child: assembled.body,
        });
    }
    let children = affine.iter().map(|term| term.child).collect::<Vec<_>>();
    let body = destination.insert(
        Opcode::Affine,
        Payload::Affine {
            constant: 0.0,
            constant_quantity_type: None,
            constant_unit: None,
            terms: affine,
        },
        &children,
        Some(owner),
    )?;
    Ok(ConservationBody {
        body,
        kernel_bindings: kernels,
        selections,
        projected_groups: projections,
    })
}

async fn contribution(
    context: &Context<'_, '_>,
    law: &compiled::law_applications::Row,
    free: &IndexSet,
    term: &Term<'_>,
    destination: &mut ExprGraph,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<ConservationBody, CompilerError> {
    let owner = law.owner_instance_id;
    let element = free
        .iter()
        .find(|axis| axis.kind == DomainKind::Element)
        .copied()
        .ok_or_else(|| invalid("element law axis absent"))?;
    let species = term
        .environment
        .free_indices
        .iter()
        .filter(|axis| axis.kind == DomainKind::Species)
        .copied()
        .collect::<Vec<_>>();
    let [species] = species.as_slice() else {
        return Err(invalid(
            "element term requires exactly one actual Species axis",
        ));
    };
    let group = coefficients::build(
        context,
        law,
        term,
        [element, *species],
        destination,
        output,
        work,
    )
    .await?;
    let (mut flow, instance) = instantiate_source(context, term, destination)?;
    let combined = flow
        .indices
        .union(&group.indices)
        .map_err(|_| invalid("element coefficient and flow binders conflict"))?;
    physical::broadcast(context, &mut flow, &combined, owner, destination)?;
    let mut coefficient = physical::Value {
        root: group.root,
        quantity: group.quantity,
        indices: group.indices,
    };
    physical::broadcast(context, &mut coefficient, &combined, owner, destination)?;
    let (mut value, selection) = physical::multiply(
        context,
        &flow,
        &coefficient,
        group.operation,
        owner,
        destination,
    )?;

    physical::reduce(context, &mut value, &term.reductions, owner, destination)?;
    if value.indices != *free
        || context.physical.quantity_type(value.quantity)?.key
            != context
                .physical
                .quantity_type(QuantityTypeId::from_id(law.quantity_type_id))?
                .key
    {
        return Err(invalid(
            "element projection does not leave the exact declared Element balance contract",
        ));
    }
    Ok(ConservationBody {
        body: value.root,
        kernel_bindings: instance.kernel_bindings,
        selections: vec![selection],
        projected_groups: instance.projected_groups,
    })
}
fn instantiate_source(
    context: &Context<'_, '_>,
    term: &Term<'_>,
    destination: &mut ExprGraph,
) -> Result<(physical::Value, pse_templates::Instantiation), CompilerError> {
    let mut environment = term.environment.clone();
    let source_indices = IndexSet::try_from_iter(
        environment
            .free_indices
            .iter()
            .filter(|axis| axis.kind != DomainKind::Element)
            .copied(),
    )
    .map_err(|_| invalid("element source binder conflict"))?;
    environment.free_indices = source_indices.clone();
    let source_root = *context
        .source
        .node_mapping
        .get(&NodeId(term.contribution.expression_root))
        .ok_or_else(|| invalid("element source root absent"))?;
    let instance = pse_templates::instantiate(
        context.source,
        &[source_root],
        &environment,
        destination,
        context.ctx.cancel,
    )?;
    let [root] = instance.roots.as_slice() else {
        return Err(invalid("element source root inventory differs"));
    };
    Ok((
        physical::Value {
            root: *root,
            quantity: term.source_quantity,
            indices: source_indices,
        },
        instance,
    ))
}
