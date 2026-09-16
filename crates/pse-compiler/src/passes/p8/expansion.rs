// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native IR construction consumes complete relational membership and ordered descriptors.
mod axes;
mod connections;
mod elements;
mod projections;
#[cfg(test)]
mod tests;
use super::{
    graph,
    inventory::{Inventory as Inputs, Located},
    outputs::Output,
};
use crate::{CompilerError, PassContext, passes::native_outputs::Sources};
use pse_ids::SemanticId;
use pse_mathir::{
    NodeId,
    equation::{EquationRecord, FreeIndex},
};
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, IndexSet, QuantityTypeId};
use pse_relations::{
    RecordBatch,
    generated::{compiled, inferred},
};
use pse_schema::math::Sense;
use pse_schema::model::PassSpec;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Expanded {
    pub(super) rows: graph::Batches,
    pub(super) derivations: Vec<RecordBatch>,
}

pub(super) async fn expand(
    batches: &graph::Batches,
    sources: &Sources,
    session: &pse_catalog::session::SnapshotSession,
    ctx: &PassContext<'_>,
    spec: &PassSpec,
) -> Result<Expanded, CompilerError> {
    ctx.cancel.checkpoint()?;
    let mut work = ctx.reserver.open("P8:expansion");
    let extent = batches.values().try_fold(0_usize, |sum, batch| {
        sum.checked_add(pse_ids::validation_extent(batch.batch())?)
            .ok_or_else(|| invalid("law input extent overflow"))
    })?;
    work.try_grow(
        extent
            .checked_mul(12)
            .ok_or_else(|| invalid("law decode extent overflow"))?,
    )
    .map_err(pse_catalog::CatalogError::from)?;
    let loaded = graph::load(batches, ctx)?;
    let (inventory, mut arguments) = Inputs::load(session, sources, ctx).await?;
    let mut output = Output::new(ctx)?;
    graph::inputs(
        batches,
        &loaded,
        &mut output,
        &mut arguments,
        sources,
        session,
        ctx,
        spec,
    )
    .await?;
    let inputs = &inventory;
    let mut destination = loaded.graph.clone();
    let mut equations = loaded.equations.clone();
    let mut kernels = loaded.kernel_bindings.clone();
    let mut selections = loaded.selections.clone();
    let mut conversions = BTreeMap::new();
    let physical = ctx.physical()?.quantities();
    let checker = ctx.physical()?.precondition_checker();
    let laws = &inputs.law_applications;
    let contexts = &inputs.law_contexts;
    if contexts.len() != laws.len()
        || contexts.iter().any(|context| {
            !laws
                .iter()
                .any(|law| context.application_id == law.application_id)
        })
    {
        return Err(invalid(
            "a requested balance has no unique supported declared law binding",
        ));
    }
    let contributions = &inputs.contributions;
    let decisions = &inputs.law_participation_decisions;
    let candidates = &inputs.law_candidates;
    partition(candidates, decisions)?;
    let ordered = &inputs.law_ordered_terms;
    let all_axes = &inputs.law_axes;
    let domains = axes::domains(inputs, ctx)?;
    for law in laws {
        ctx.cancel.checkpoint()?;
        output.active.clear();
        output.use_row(law);
        for context in contexts
            .iter()
            .filter(|row| row.application_id == law.application_id)
        {
            output.use_row(context);
        }
        for row in candidates
            .iter()
            .filter(|row| row.application_id == law.application_id)
        {
            output.use_row(row);
        }
        for row in decisions
            .iter()
            .filter(|row| row.application_id == law.application_id)
        {
            output.use_row(row);
        }
        let application = law.application_id;
        let owner = law.owner_instance_id;
        let mut law_axes = all_axes
            .iter()
            .filter(|axis| axis.application_id == application)
            .collect::<Vec<_>>();
        law_axes.sort_by_key(|axis| axis.position);
        let indices = law_axes
            .iter()
            .enumerate()
            .map(|(position, axis)| {
                if usize::from(axis.position) != position {
                    return Err(invalid("law axes have gaps or duplicate positions"));
                }
                axis.support(&mut output.active);
                let domain = DomainId::from_id(axis.domain_id);
                Ok(BoundIndexRef::new(
                    BoundIndexId::from_id(axis.bound_index_id),
                    domain,
                    domains
                        .get(&domain)
                        .ok_or_else(|| invalid("law axis lacks actual domain"))?
                        .kind,
                ))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        let free = IndexSet::try_from_iter(indices.iter().copied())
            .map_err(|_| invalid("law index identity conflict"))?;
        let descriptors = ordered
            .iter()
            .filter(|row| row.application_id == application)
            .collect::<Vec<_>>();
        let [descriptor] = descriptors.as_slice() else {
            return Err(invalid(
                "law has no complete nonempty ordered descriptor set",
            ));
        };
        output.use_row(descriptor);
        let ids = &descriptor.contribution_ids;
        let expected_ids = decisions
            .iter()
            .filter(|row| row.application_id == application && row.decision.as_str() == "included")
            .map(|row| row.contribution_id)
            .collect::<BTreeSet<_>>();
        if ids.iter().copied().collect::<BTreeSet<_>>() != expected_ids
            || ids.len() != expected_ids.len()
            || ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid(
                "ordered descriptors differ from the complete included candidate set",
            ));
        }
        let mut terms = Vec::new();
        let mut element_terms = Vec::new();
        for &id in ids {
            let contribution = unique(
                contributions,
                |row| row.contribution_id == id,
                "contribution",
            )?;
            let decision = decisions
                .iter()
                .find(|row| row.application_id == application && row.contribution_id == id)
                .ok_or_else(|| invalid("included contribution lacks decision"))?;
            let sign = decision.sign;
            if !matches!(sign, -1 | 1) {
                return Err(invalid("included orientation is not signed unity"));
            }
            output.use_row(contribution);
            output.use_row(decision);
            let source_root = *loaded
                .node_mapping
                .get(&NodeId(contribution.expression_root))
                .ok_or_else(|| invalid("contribution root absent"))?;
            output.source_graph(&loaded.graph, source_root)?;
            work.try_grow(
                loaded
                    .graph
                    .len()
                    .checked_mul(8192)
                    .ok_or_else(|| invalid("law node bound overflow"))?,
            )
            .map_err(pse_catalog::CatalogError::from)?;
            let term_axes = axes::environment(
                inputs,
                &loaded,
                &domains,
                law,
                contribution,
                &indices,
                &mut output,
            )?;
            if law.subject_projection.as_str() == "species_to_element" {
                element_terms.push(elements::Term {
                    contribution,
                    source_quantity: axes::projected_quantity(physical, contribution, &term_axes)?,
                    environment: term_axes.environment,
                    reductions: term_axes.reductions,
                    sign,
                });
                continue;
            }
            let conversion =
                axes::check_quantity(physical, law, contribution, &term_axes, checker)?;
            if let Some((conversion, _)) = conversion {
                conversions.insert((application, id), conversion);
            }
            terms.push(pse_templates::laws::ConservationTerm {
                contribution: id,
                root: *loaded
                    .node_mapping
                    .get(&NodeId(contribution.expression_root))
                    .ok_or_else(|| invalid("contribution root absent"))?,
                environment: term_axes.environment,
                reductions: term_axes.reductions,
                broadcasts: term_axes.broadcasts,
                sign,
                conversion,
            });
        }
        let body = if law.subject_projection.as_str() == "species_to_element" {
            if law.expansion.as_str() != "conservation" {
                return Err(invalid(
                    "element projection requires conservation expansion",
                ));
            }
            elements::conservation(
                &elements::Context {
                    inputs,
                    ctx,
                    source: &loaded,
                    physical,
                    checker,
                },
                law,
                &free,
                &element_terms,
                &mut destination,
                &mut output,
                &mut *work,
            )
            .await?
        } else if law.subject_projection.as_str() == "identity" {
            match law.expansion.as_str() {
                "conservation" => pse_templates::laws::conservation(
                    &loaded,
                    owner,
                    &free,
                    &terms,
                    &mut destination,
                    ctx.cancel,
                )?,
                "isothermal" | "pressure_total" => {
                    equality_law(&loaded, owner, &free, &terms, &mut destination, ctx.cancel)?
                }
                _ => return Err(invalid("law expansion kind is undeclared")),
            }
        } else {
            return Err(invalid("undeclared law subject projection"));
        };
        for projection in body.projected_groups.values() {
            projections::emit(inputs, &mut output, projection)?;
        }
        for (key, binding) in body.kernel_bindings {
            output
                .kernels
                .entry(key)
                .or_default()
                .extend(output.active.iter().cloned());
            if kernels
                .insert(key, binding.clone())
                .is_some_and(|old| old != binding)
            {
                return Err(invalid("law kernel binding identity conflict"));
            }
        }
        selections.extend(body.selections);
        let declared_quantity = QuantityTypeId::from_id(law.quantity_type_id);
        let residual_quantity = if matches!(law.expansion.as_str(), "isothermal" | "pressure_total")
        {
            let operand = pse_quantity::infer::Operand {
                quantity_type: declared_quantity,
                indices: &free,
            };
            pse_quantity::infer::infer_with_evidence(
                &pse_quantity::infer::OpRequest::Sub,
                &[operand, operand],
                physical,
                checker,
            )?
            .result
        } else {
            declared_quantity
        };
        let zero = typed_zero(&mut destination, residual_quantity, &free, physical)?;
        let equation_id = pse_ids::named_id(application, "pse:law-equation:v1");
        output.record_graph(&destination, [body.body, zero])?;
        output.equations.insert(equation_id, output.active.clone());
        equations.push(EquationRecord {
            indexed_equation_id: equation_id,
            owner_instance: owner,
            equation_decl: None,
            qualified_name: format!("law:{}", application.to_hex()),
            product: Some(law.product_id),
            filter: None,
            body: body.body,
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
                            .map_err(|_| invalid("law axis position exceeds u16"))?,
                    })
                })
                .collect::<Result<_, CompilerError>>()?,
            residual_quantity_type: None,
            law_instance: Some(application),
            derivation: pse_ids::named_id(application, "P8:equation"),
        });
    }
    for decision in decisions {
        output.active.clear();
        output.use_row(decision);
        output.push(compiled::law_participation::Row {
            application_id: decision.application_id,
            contribution_id: decision.contribution_id,
            decision: decision.decision,
            reason: decision.reason,
            sign: decision.sign,
            conversion_id: conversions
                .get(&(decision.application_id, decision.contribution_id))
                .map(|id| id.as_id()),
            derivation_id: decision.derivation_id,
        })?;
    }
    connections::expand(
        inputs,
        ctx,
        physical,
        &mut destination,
        &mut equations,
        &mut output,
        &mut *work,
    )?;
    let merged = pse_mathir::relations::LoadedMath {
        graph: destination,
        equations,
        roots: Vec::new(),
        selections,
        kernel_bindings: kernels,
        node_mapping: BTreeMap::new(),
    };
    let (rows, derivations) = output
        .finish(&merged, batches, ctx, spec, sources, session)
        .await?;
    Ok(Expanded { rows, derivations })
}

fn partition(
    candidates: &[Located<inferred::law_candidates::Row>],
    decisions: &[Located<inferred::law_participation_decisions::Row>],
) -> Result<(), CompilerError> {
    let candidates = candidates
        .iter()
        .map(|row| (row.application_id, row.contribution_id))
        .collect::<BTreeSet<_>>();
    let actual = decisions
        .iter()
        .map(|row| (row.application_id, row.contribution_id))
        .collect::<BTreeSet<_>>();
    if candidates != actual || actual.len() != decisions.len() {
        return Err(invalid(
            "law inclusion/exclusion partition is incomplete or overlaps",
        ));
    }
    Ok(())
}
pub(super) fn unique<'a, T>(
    rows: &'a [Located<T>],
    predicate: impl Fn(&T) -> bool,
    description: &str,
) -> Result<&'a Located<T>, CompilerError> {
    let mut selected = rows.iter().filter(|row| predicate(&row.row));
    let row = selected
        .next()
        .ok_or_else(|| invalid(format!("{description} actual row absent")))?;
    if selected.next().is_some() {
        return Err(invalid(format!("{description} actual row ambiguous")));
    }
    Ok(row)
}
pub(super) fn typed_zero(
    graph: &mut pse_mathir::ExprGraph,
    quantity: QuantityTypeId,
    indices: &IndexSet,
    physical: &pse_quantity::QuantityRegistry,
) -> Result<NodeId, CompilerError> {
    let mut key = physical.quantity_type(quantity)?.key.clone();
    key.shape.clear();
    let scalar = physical.quantity_type(physical.resolve_key(&key)?)?;
    let mut zero = graph.insert_typed(
        pse_mathir::Opcode::Const,
        pse_mathir::Payload::FloatConst {
            value: 0.0,
            unit: scalar.canonical_unit,
        },
        &[],
        scalar.id,
        None,
    )?;
    for index in indices {
        zero = graph.insert(
            pse_mathir::Opcode::Broadcast,
            pse_mathir::Payload::Broadcast {
                domain: index.domain.into(),
                bound_index: index.bound_index,
            },
            &[zero],
            None,
        )?;
    }
    Ok(zero)
}
fn equality_law(
    source: &pse_mathir::relations::LoadedMath,
    owner: SemanticId,
    free: &IndexSet,
    terms: &[pse_templates::laws::ConservationTerm],
    graph: &mut pse_mathir::ExprGraph,
    cancel: &pse_ids::CancellationToken,
) -> Result<pse_templates::laws::ConservationBody, CompilerError> {
    if terms.len() != 2
        || terms[0].sign == terms[1].sign
        || terms.iter().any(|term| !term.reductions.is_empty())
    {
        return Err(invalid(
            "intensive equality requires two opposing complete members without summation",
        ));
    }
    Ok(pse_templates::laws::equality(
        source, owner, free, terms, graph, cancel,
    )?)
}
pub(super) fn invalid(detail: impl Into<String>) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: SemanticId::NIL,
        detail: detail.into(),
    }
    .into()
}
