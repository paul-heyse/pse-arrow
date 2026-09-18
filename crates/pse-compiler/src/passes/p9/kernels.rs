// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete descriptor binding remains distinct from numeric execution availability.
#[cfg(test)]
mod tests;
use super::{
    invalid,
    selections::{Method, Request, Selection},
    unbound_kernel,
};
use crate::{
    AlgorithmContext, CompilerError,
    passes::{
        native_outputs::{SourceKey, Sources},
        native_rows::{AlgorithmInputs, Located, located_input},
        p7::KernelMethod,
        parameter_indices::ParameterIndexProjector,
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{IndexTuple, SemanticId};
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::generated::{inferred, normalized, reference};
use std::collections::{BTreeMap, BTreeSet};

pub(super) async fn prepare(
    selection: &Selection,
    ctx: &AlgorithmContext<'_>,
    session: &SnapshotSession,
    sources: &Sources,
) -> Result<(Vec<KernelMethod>, AlgorithmInputs), CompilerError> {
    let mut arguments = AlgorithmInputs::new(ctx.reserver, "P9:kernel-arguments");
    let inventory = Inventory::new(&mut arguments, session, sources, ctx).await?;
    let physical = ctx.physical()?.quantities();
    let projector = ParameterIndexProjector::new(ctx.registry, session, sources, ctx.reserver);
    let mut pending = BTreeMap::new();
    let mut completed = BTreeSet::new();
    for request in &selection.requests {
        let method = selection
            .methods
            .get(&request.instance)
            .ok_or_else(|| invalid("kernel selected method absent"))?;
        if method.specification.realization.equation_template.is_some() {
            completed.insert(request.requirement.requirement_id);
            continue;
        }
        let prepared = inventory
            .prepare(method, request, selection, physical, &projector, ctx)
            .await?;
        if pending
            .insert(request.requirement.requirement_id, prepared)
            .is_some()
        {
            return Err(invalid("kernel requirement repeated"));
        }
    }
    let mut result = Vec::new();
    while !pending.is_empty() {
        ctx.cancel.checkpoint()?;
        let next = pending
            .iter()
            .find(|(_, (_, dependencies))| dependencies.is_subset(&completed))
            .map(|(id, _)| *id)
            .ok_or_else(|| invalid("selected kernel dependency order is cyclic or incomplete"))?;
        let (method, _) = pending
            .remove(&next)
            .ok_or_else(|| invalid("selected kernel disappeared"))?;
        result.push(method);
        completed.insert(next);
    }
    Ok((result, arguments))
}
type KernelInputs = (Vec<(String, SemanticId)>, BTreeSet<SemanticId>);

struct Inventory {
    descriptors: Vec<Located<reference::kernel_specs::Row>>,
    parameters: Vec<Located<reference::method_parameters::Row>>,
    mappings: Vec<Located<reference::method_kernel_inputs::Row>>,
    dependencies: Vec<Located<reference::method_dependencies::Row>>,
    property_keys: Vec<Located<inferred::dependency_key_maps::Row>>,
    state_keys: Vec<Located<inferred::state_dependency_keys::Row>>,
    values: Vec<Located<normalized::parameter_values::Row>>,
    resolutions: Vec<Located<inferred::method_resolutions::Row>>,
}
impl Inventory {
    async fn new(
        arguments: &mut AlgorithmInputs,
        session: &SnapshotSession,
        sources: &Sources,
        ctx: &AlgorithmContext<'_>,
    ) -> Result<Self, CompilerError> {
        Ok(Self {
            descriptors: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            parameters: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            mappings: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            dependencies: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            property_keys: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            state_keys: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            values: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            resolutions: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
        })
    }
    async fn prepare(
        &self,
        method: &Method,
        request: &Request,
        selection: &Selection,
        physical: &QuantityRegistry,
        projector: &ParameterIndexProjector<'_>,
        ctx: &AlgorithmContext<'_>,
    ) -> Result<(KernelMethod, BTreeSet<SemanticId>), CompilerError> {
        let reference::method_specs::ReferenceMethodSpecsFieldRealizationSelected::Kernel(producer) =
            method.specification.realization.selected()?
        else {
            return Err(invalid("kernel binding requires a kernel producer"));
        };
        let reference::method_provisions::ReferenceMethodProvisionsFieldOutputSelected::KernelOutput(provision) = request.provision.output.selected()? else {
            return Err(invalid("kernel binding requires a kernel output provision"));
        };
        let descriptor = one(
            self.descriptors
                .iter()
                .filter(|row| row.kernel_id == producer.kernel_id),
            "selected kernel descriptor",
        )?;
        crate::quantity_relations::kernel_contract::check_descriptor(descriptor, physical)?;
        let ordinal = provision.ordinal;
        let output = descriptor
            .outputs
            .get(usize::try_from(ordinal).map_err(|_| invalid("negative kernel output ordinal"))?)
            .ok_or_else(|| invalid("kernel provision output ordinal outside actual descriptor"))?;
        let output_type =
            physical.quantity_type(QuantityTypeId::from_id(output.quantity_type_id))?;
        pse_quantity::admission::require_same_contract(
            output_type.id,
            QuantityTypeId::from_id(request.provision.quantity_type_id),
            physical,
        )?;
        if output.shape != request.provision.indexed_by
            || output.natural_unit_id != request.provision.natural_unit_id
        {
            return Err(invalid(
                "kernel provision shape/natural unit differs from descriptor",
            ));
        }
        if !output.shape.is_empty() || !request.requirement.index.is_empty() {
            return Err(unbound_kernel(
                method.specification.method_id,
                Some(descriptor.kernel_id),
                "indexed kernel output needs an explicit non-scalar expression carrier",
            ));
        }
        let resolution = self
            .resolutions
            .iter()
            .find(|row| row.requirement_id == request.requirement.requirement_id)
            .ok_or_else(|| invalid("kernel resolution source absent"))?;
        let mut support = BTreeSet::from([descriptor.source.clone(), resolution.source.clone()]);
        let (inputs, dependencies) =
            self.input_bindings(method, request, selection, descriptor, &mut support, ctx)?;
        let parameters = self
            .parameter_bindings(method, descriptor, physical, projector, &mut support, ctx)
            .await?;
        Ok((
            KernelMethod {
                descriptor: descriptor.row.clone(),
                method: method.specification.method_id,
                requirement: request.requirement.requirement_id,
                state_scope: method.scope,
                state: method.state,
                index: request.requirement.index.clone(),
                output_ordinal: ordinal,
                inputs,
                parameters,
                support,
            },
            dependencies,
        ))
    }
    fn input_bindings(
        &self,
        method: &Method,
        request: &Request,
        selection: &Selection,
        descriptor: &reference::kernel_specs::Row,
        support: &mut BTreeSet<SourceKey>,
        ctx: &AlgorithmContext<'_>,
    ) -> Result<KernelInputs, CompilerError> {
        let mut inputs = Vec::new();
        let mut required = BTreeSet::new();
        if self
            .mappings
            .iter()
            .filter(|row| row.method_id == method.specification.method_id)
            .count()
            != descriptor.inputs.len()
        {
            return Err(invalid(
                "kernel input mapping is incomplete or contains unknown names",
            ));
        }
        for input in &descriptor.inputs {
            ctx.cancel.checkpoint()?;
            let mapping = one(
                self.mappings.iter().filter(|row| {
                    row.method_id == method.specification.method_id && row.input_name == input.name
                }),
                "kernel input dependency mapping",
            )?;
            let dependency = one(
                self.dependencies.iter().filter(|row| {
                    row.method_id == method.specification.method_id
                        && row.ordinal == mapping.dependency_ordinal
                }),
                "kernel input method dependency",
            )?;
            support.insert(mapping.source.clone());
            support.insert(dependency.source.clone());
            let symbol = if dependency.target_kind.as_str() == "property" {
                let key = one(
                    self.property_keys.iter().filter(|row| {
                        row.requirement_id == request.requirement.requirement_id
                            && row.method_id == method.specification.method_id
                            && row.dependency_ordinal == mapping.dependency_ordinal
                    }),
                    "scalar kernel property dependency key",
                )?;
                let target = one(
                    selection
                        .requests
                        .iter()
                        .filter(|row| row.requirement.requirement_id == key.target_requirement_id),
                    "kernel input selected property provider",
                )?;
                let selected = selection
                    .methods
                    .get(&target.instance)
                    .ok_or_else(|| invalid("kernel dependency selected method absent"))?;
                required.insert(target.requirement.requirement_id);
                support.insert(key.source.clone());
                match target.provision.output.selected()? {
                        reference::method_provisions::ReferenceMethodProvisionsFieldOutputSelected::TemplateSymbol(value) => pse_ids::symbol_instance_id(
                            target.instance, value.symbol_decl_id, IndexTuple(&target.requirement.index),
                        ),
                        reference::method_provisions::ReferenceMethodProvisionsFieldOutputSelected::KernelOutput(value) => super::kernel_output_id(
                            selected.scope, selected.specification.method_id, value.ordinal, &target.requirement.index,
                        ),
                    }
            } else {
                let key = one(
                    self.state_keys.iter().filter(|row| {
                        row.requirement_id == request.requirement.requirement_id
                            && row.method_id == method.specification.method_id
                            && row.dependency_ordinal == mapping.dependency_ordinal
                    }),
                    "scalar kernel state dependency key",
                )?;
                if key.symbol_decl_id != dependency.target_id {
                    return Err(invalid(
                        "kernel state dependency differs from its declaration",
                    ));
                }
                support.insert(key.source.clone());
                pse_ids::symbol_instance_id(
                    method.state,
                    key.symbol_decl_id,
                    IndexTuple(&key.index),
                )
            };
            inputs.push((input.name.clone(), symbol));
        }
        Ok((inputs, required))
    }
    async fn parameter_bindings(
        &self,
        method: &Method,
        descriptor: &reference::kernel_specs::Row,
        physical: &QuantityRegistry,
        projector: &ParameterIndexProjector<'_>,
        support: &mut BTreeSet<SourceKey>,
        ctx: &AlgorithmContext<'_>,
    ) -> Result<Vec<pse_mathir::relations::ParameterBinding>, CompilerError> {
        if self
            .parameters
            .iter()
            .filter(|row| row.method_id == method.specification.method_id)
            .count()
            != descriptor.parameters.len()
        {
            return Err(invalid(
                "kernel method parameter roster differs from descriptor",
            ));
        }
        let mut result = Vec::new();
        for port in &descriptor.parameters {
            ctx.cancel.checkpoint()?;
            let parameter = one(
                self.parameters.iter().filter(|row| {
                    row.method_id == method.specification.method_id && row.name == port.name
                }),
                "kernel method parameter declaration",
            )?;
            pse_quantity::admission::require_same_contract(
                QuantityTypeId::from_id(parameter.quantity_type_id),
                QuantityTypeId::from_id(port.quantity_type_id),
                physical,
            )?;
            if parameter.indexed_by != port.indexed_by {
                return Err(invalid("kernel parameter axes differ from method contract"));
            }
            if !port.indexed_by.is_empty() {
                return Err(unbound_kernel(
                    method.specification.method_id,
                    Some(descriptor.kernel_id),
                    "indexed kernel parameter requires an array/group carrier; scalar ParameterBinding cannot encode it",
                ));
            }
            let projected = projector
                .project(
                    method.specification.method_id,
                    &parameter.name,
                    &[],
                    &[],
                    ctx.cancel,
                )
                .await?;
            let value = one(
                self.values.iter().filter(|row| {
                    row.owner_entity_id == method.package
                        && row.parameter_kind == parameter.name
                        && row.index == projected.values
                }),
                "actual kernel parameter source value",
            )?;
            let number = crate::quantity_relations::kernel_contract::scalar_parameter(
                parameter, port, value, physical,
            )?;
            support.insert(parameter.source.clone());
            support.insert(value.source.clone());
            support.extend(projected.sources.iter().cloned());
            result.push((
                port.name.clone(),
                None,
                Some(number),
                Some(UnitId::from_id(port.natural_unit_id)),
            ));
        }
        Ok(result)
    }
}

fn one<T>(mut rows: impl Iterator<Item = T>, label: &str) -> Result<T, CompilerError> {
    let value = rows
        .next()
        .ok_or_else(|| invalid(format!("{label} absent")))?;
    if rows.next().is_some() {
        return Err(invalid(format!("{label} ambiguous")));
    }
    Ok(value)
}
