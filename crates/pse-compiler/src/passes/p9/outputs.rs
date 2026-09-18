// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual provision and parameter correspondences, retaining complete typed row support.
use super::{Inputs, invalid, parameters::Bindings, selections::Selection};
use crate::{
    AlgorithmContext, CompilerError,
    passes::{
        native_outputs::{self, OutputRows, SourceKey, Sources},
        native_rows::{AlgorithmInputs, Located, located_input, workspace},
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{IndexTuple, SemanticId};
use pse_quantity::{QuantityTypeId, UnitId};
use pse_relations::{
    RecordBatch,
    generated::{compiled, enums::AliasKind, inferred, normalized, reference},
};
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::collections::{BTreeMap, BTreeSet};

type Support = BTreeSet<SourceKey>;
pub(super) async fn bind_parameters(
    inputs: &mut Inputs,
    sources: &mut Sources,
    bindings: Bindings,
    ctx: &AlgorithmContext<'_>,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
) -> Result<(), CompilerError> {
    let mut output = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    output.ensure::<compiled::method_parameter_bindings::Row>()?;
    for binding in &bindings.rows {
        output.push(binding.row.clone(), &binding.support)?;
    }
    materialize(output, inputs, sources, ctx, pass, session).await?;
    Ok(())
}
#[expect(
    clippy::too_many_lines,
    reason = "bind_methods keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn bind_methods(
    inputs: &mut Inputs,
    sources: &mut Sources,
    selection: &Selection,
    session: &SnapshotSession,
    ctx: &AlgorithmContext<'_>,
    pass: &AlgorithmSpec,
) -> Result<(), CompilerError> {
    let mut arguments = AlgorithmInputs::new(ctx.reserver, "P9:method-output-arguments");
    let declarations: Vec<Located<normalized::template_symbols::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let bindings: Vec<Located<normalized::instance_domain_bindings::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let products: Vec<Located<normalized::domain_products::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let tuples: Vec<Located<inferred::valid_index_tuples::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let requirements: Vec<Located<inferred::property_requirements::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let methods: Vec<Located<reference::method_specs::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let provisions: Vec<Located<reference::method_provisions::Row>> =
        located_input(&mut arguments, session, sources, ctx.registry, ctx.cancel).await?;
    let mut output = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    output.ensure::<compiled::method_realizations::Row>()?;
    let physical = ctx.physical()?.quantities();
    for request in &selection.requests {
        ctx.cancel.checkpoint()?;
        let method = selection
            .methods
            .get(&request.instance)
            .ok_or_else(|| invalid("selected method instance missing"))?;
        let reference::method_specs::ReferenceMethodSpecsFieldRealizationSelected::EquationTemplate(
            producer,
        ) = method.specification.realization.selected()?
        else {
            continue;
        };
        let reference::method_provisions::ReferenceMethodProvisionsFieldOutputSelected::TemplateSymbol(provision) = request.provision.output.selected()? else {
            return Err(invalid("template method requires a template symbol provision"));
        };
        let declaration = unique(
            declarations.iter().filter(|row| {
                row.symbol_decl_id == provision.symbol_decl_id
                    && row.template_id == producer.template_id
            }),
            "selected output symbol declaration",
        )?;
        let mut support = Support::from([declaration.source.clone()]);
        support.insert(
            unique(
                methods
                    .iter()
                    .filter(|row| row.method_id == method.specification.method_id),
                "actual method specification",
            )?
            .source
            .clone(),
        );
        support.insert(
            unique(
                provisions.iter().filter(|row| {
                    row.method_id == method.specification.method_id
                        && row.property_kind_id == request.requirement.property_kind_id
                }),
                "actual method provision",
            )?
            .source
            .clone(),
        );
        let mut domains = Vec::new();
        for name in &declaration.indexed_by {
            let binding = unique(
                bindings
                    .iter()
                    .filter(|row| row.instance_id == request.instance && row.domain_name == *name),
                "selected output domain",
            )?;
            domains.push(binding.domain_id);
            support.insert(binding.source.clone());
        }
        let product = pse_templates::identity::domain_product_id(&domains);
        let actual = unique(
            products
                .iter()
                .filter(|row| row.product_id == product || row.domain_ids == domains),
            "selected output product",
        )?;
        if actual.product_id != product
            || actual.domain_ids != domains
            || !tuples
                .iter()
                .any(|row| row.product_id == product && row.tuple == request.requirement.index)
        {
            return Err(invalid(
                "selected provision requirement coordinate is outside its actual output product",
            ));
        }
        support.insert(actual.source.clone());
        support.insert(
            unique(
                tuples.iter().filter(|row| {
                    row.product_id == product && row.tuple == request.requirement.index
                }),
                "method output tuple",
            )?
            .source
            .clone(),
        );
        let output_type =
            physical.quantity_type(QuantityTypeId::from_id(declaration.quantity_type_id))?;
        let provision_type =
            physical.quantity_type(QuantityTypeId::from_id(request.provision.quantity_type_id))?;
        if output_type.key != provision_type.key
            || request
                .provision
                .indexed_by
                .iter()
                .map(|kind| kind.as_str())
                .ne(output_type.key.shape.iter().map(|kind| kind.as_str()))
        {
            return Err(invalid(
                "method advertised output differs from actual declaration full physical contract",
            ));
        }
        pse_quantity::convert_spec_for_type(
            physical.unit(UnitId::from_id(request.provision.natural_unit_id))?,
            physical.unit(output_type.canonical_unit)?,
            &output_type.key,
        )?;
        let output_symbol = pse_ids::symbol_instance_id(
            request.instance,
            declaration.symbol_decl_id,
            IndexTuple(&request.requirement.index),
        );
        let requirement = requirements
            .iter()
            .find(|row| row.requirement_id == request.requirement.requirement_id)
            .ok_or_else(|| invalid("selected requirement disappeared"))?;
        support.insert(requirement.source.clone());
        output.push(
            compiled::method_realizations::Row {
                requirement_id: request.requirement.requirement_id,
                method_id: method.specification.method_id,
                output_symbol_id: output_symbol,
                realization: compiled::method_realizations::CompiledMethodRealizationsFieldRealization::from_template_symbol(
                    compiled::method_realizations::CompiledMethodRealizationsFieldRealizationTemplateSymbol {
                        template_instance_id: request.instance,
                    }
                ),
                derivation_id: SemanticId::NIL,
            },
            &support,
        )?;
    }
    materialize(output, inputs, sources, ctx, pass, session).await?;
    Ok(())
}

pub(super) fn without_relations(
    batches: Vec<RecordBatch>,
    replaced: &BTreeSet<RelationKey>,
    ctx: &AlgorithmContext<'_>,
) -> Result<Vec<RecordBatch>, CompilerError> {
    use datafusion::arrow::{
        array::{Array, BooleanArray, FixedSizeBinaryArray},
        compute::filter_record_batch,
    };
    let excluded = replaced
        .iter()
        .map(|key| {
            ctx.registry
                .relation(&key.qualified_name())
                .map(|spec| spec.id)
                .ok_or_else(|| invalid("replaced derivation declaration absent"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut result = Vec::new();
    for batch in batches {
        ctx.cancel.checkpoint()?;
        let mut lease = ctx.reserver.open("P9:final-derivations");
        lease
            .try_grow(pse_ids::validation_extent(&batch)?)
            .map_err(pse_ids::CanonError::from)?;
        let ids = batch
            .column_by_name("relation_id")
            .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
            .ok_or_else(|| invalid("derivation relation identity storage differs"))?;
        let mask = (0..batch.num_rows())
            .map(|row| {
                if ids.is_null(row) {
                    return Err(invalid("derivation relation identity is null"));
                }
                let id = SemanticId::try_from_slice(ids.value(row))
                    .map_err(|_| invalid("derivation identity width differs"))?;
                Ok(!excluded.contains(&id))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        if mask.iter().all(|keep| *keep) {
            result.push(batch);
        } else if mask.iter().any(|keep| *keep) {
            let filtered = filter_record_batch(&batch, &BooleanArray::from(mask))
                .map_err(pse_ids::CanonError::from)?;
            result.push(pse_ids::owned_buffer::attach_reservation(
                filtered,
                pse_ids::ReservationLease::new(lease),
            )?);
        }
    }
    Ok(result)
}

pub(super) async fn provisions(
    selection: &Selection,
    inputs: &Inputs,
    pass: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    output: &mut super::super::p7::RealizationOutput,
    session: &SnapshotSession,
) -> Result<(), CompilerError> {
    let mut actual = inputs.clone();
    actual.extend(output.rows.iter().map(|(key, batch)| (*key, batch.clone())));
    let session = workspace(session, &actual, ctx.cancel)?;
    let mut arguments = AlgorithmInputs::new(ctx.reserver, "P9:provision-arguments");
    let sources = &output.sources;
    let symbols: Vec<Located<compiled::symbols::Row>> =
        located_input(&mut arguments, &session, sources, ctx.registry, ctx.cancel).await?;
    let methods: Vec<Located<compiled::method_realizations::Row>> =
        located_input(&mut arguments, &session, sources, ctx.registry, ctx.cancel).await?;
    let properties: Vec<Located<normalized::template_symbol_properties::Row>> =
        located_input(&mut arguments, &session, sources, ctx.registry, ctx.cancel).await?;
    let scopes: Vec<Located<inferred::resolved_scopes::Row>> =
        located_input(&mut arguments, &session, sources, ctx.registry, ctx.cancel).await?;
    let members: Vec<Located<inferred::scope_members::Row>> =
        located_input(&mut arguments, &session, sources, ctx.registry, ctx.cancel).await?;
    let mut rows = Vec::new();
    for request in &selection.requests {
        let method = selection
            .methods
            .get(&request.instance)
            .ok_or_else(|| invalid("selected method disappeared"))?;
        let binding = unique(
            methods
                .iter()
                .filter(|row| row.requirement_id == request.requirement.requirement_id),
            "realized method output",
        )?;
        let target = binding.output_symbol_id;
        let target_symbol = unique(
            symbols.iter().filter(|row| row.symbol_id == target),
            "actual method symbol",
        )?;
        for property in properties
            .iter()
            .filter(|row| row.property_kind_id == request.requirement.property_kind_id)
        {
            for symbol in symbols.iter().filter(|row| {
                row.symbol_decl_id == property.symbol_decl_id
                    && row.owner_instance_id == method.state
                    && row.index == request.requirement.index
            }) {
                let scope = unique(
                    scopes.iter().filter(|row| {
                        row.scope_decl_id == property.scope_selector_id
                            && row.owner_instance_id == Some(symbol.owner_instance_id)
                    }),
                    "property placeholder scope",
                )?;
                let member = unique(
                    members.iter().filter(|row| {
                        row.scope_id == scope.scope_id && row.entity_id == method.state
                    }),
                    "actual selected singleton state membership",
                )?;
                if members
                    .iter()
                    .filter(|row| row.scope_id == scope.scope_id)
                    .count()
                    != 1
                {
                    return Err(invalid(
                        "method provision placeholder scope is not the selected singleton state",
                    ));
                }
                if symbol.symbol_id == target {
                    continue;
                }
                if symbol.quantity_type_id != target_symbol.quantity_type_id
                    || symbol.unit_id != target_symbol.unit_id
                {
                    return Err(invalid(
                        "method provider alias differs from placeholder complete scalar contract",
                    ));
                }
                rows.push((
                    compiled::symbol_references::Row {
                        alias_symbol_id: symbol.symbol_id,
                        target_symbol_id: target,
                        kind: AliasKind::Reference,
                    },
                    Support::from([
                        binding.source.clone(),
                        symbol.source.clone(),
                        target_symbol.source.clone(),
                        property.source.clone(),
                        scope.source.clone(),
                        member.source.clone(),
                    ]),
                ));
            }
        }
    }
    append_aliases(rows, &actual, pass, ctx, output, &session).await
}
async fn append_aliases(
    rows: Vec<(compiled::symbol_references::Row, Support)>,
    inputs: &Inputs,
    pass: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    output: &mut super::super::p7::RealizationOutput,
    session: &SnapshotSession,
) -> Result<(), CompilerError> {
    let mut arguments = AlgorithmInputs::new(ctx.reserver, "P9:alias-arguments");
    let existing: Vec<Located<compiled::symbol_references::Row>> = located_input(
        &mut arguments,
        session,
        &output.sources,
        ctx.registry,
        ctx.cancel,
    )
    .await?;
    let mut columns = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    columns.ensure::<compiled::symbol_references::Row>()?;
    let mut seen = BTreeMap::new();
    for original in existing {
        if seen
            .insert(original.alias_symbol_id, original.target_symbol_id)
            .is_some()
        {
            return Err(invalid("alias identity repeats"));
        }
        columns.push(original.row, &Support::from([original.source]))?;
    }
    for (row, support) in rows {
        if let Some(previous) = seen.insert(row.alias_symbol_id, row.target_symbol_id) {
            if previous != row.target_symbol_id {
                return Err(invalid(
                    "property placeholder has conflicting selected providers",
                ));
            }
            continue;
        }
        columns.push(row, &support)?;
    }
    for alias in seen.keys() {
        let mut active = BTreeSet::new();
        let mut current = *alias;
        while let Some(next) = seen.get(&current) {
            ctx.cancel.checkpoint()?;
            if !active.insert(current) {
                return Err(invalid("selected method aliases form a cycle"));
            }
            current = *next;
        }
    }
    let _ = inputs;
    let evidence = materialize(
        columns,
        &mut output.rows,
        &mut output.sources,
        ctx,
        pass,
        session,
    )
    .await?;
    output.derivations = without_relations(
        std::mem::take(&mut output.derivations),
        &BTreeSet::from([compiled::symbol_references::RELATION_KEY]),
        ctx,
    )?;
    output.derivations.extend(evidence);
    Ok(())
}
async fn materialize(
    columns: OutputRows<'_>,
    inputs: &mut Inputs,
    sources: &mut Sources,
    ctx: &AlgorithmContext<'_>,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
) -> Result<Vec<RecordBatch>, CompilerError> {
    let mut evidence = Vec::new();
    for (key, input) in
        native_outputs::materialize(columns.finish()?, sources, pass, session, ctx.cancel).await?
    {
        evidence.push(input.derivations().clone().into_batch());
        inputs.insert(key, input.checked().clone());
        sources.replace_native(key, input)?;
    }
    Ok(evidence)
}
fn unique<T>(mut values: impl Iterator<Item = T>, label: &str) -> Result<T, CompilerError> {
    let result = values
        .next()
        .ok_or_else(|| invalid(format!("{label} absent")))?;
    if values.next().is_some() {
        return Err(invalid(format!("{label} ambiguous")));
    }
    Ok(result)
}
