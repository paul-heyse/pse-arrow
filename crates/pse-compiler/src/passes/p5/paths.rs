// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed finite path resolution with actual source occurrences.
use super::super::p4::{
    invalid,
    predicates::inventory::{Inventory, Support, unique},
};
use crate::{
    AlgorithmContext, CompilerError,
    passes::{
        native_outputs::{OutputRows, Sources},
        native_rows::Keyed,
    },
};
use pse_ids::{Reservation, SemanticId};
use pse_relations::generated::{enums::PathTargetKind, inferred as i};
use pse_templates::paths::{PathInventory, ResolvedMember};
use std::collections::BTreeSet;

#[expect(
    clippy::too_many_lines,
    reason = "emit keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn emit(
    inventory: &mut Inventory<'_>,
    ctx: &AlgorithmContext<'_>,
    sources: &Sources,
    work: &mut dyn Reservation,
    output: &mut OutputRows<'_>,
) -> Result<(), CompilerError> {
    let instances = inventory.rows::<i::instances::Row>(ctx.cancel).await?;
    let tuples = inventory
        .rows::<i::valid_index_tuples::Row>(ctx.cancel)
        .await?;
    let instance_values = values(&instances);
    let prospective = values(&inventory.instances);
    let submodels = values(&inventory.submodels);
    let domains = values(&inventory.domain_bindings);
    let members = values(&inventory.members);
    let products = values(&inventory.products);
    let valid_tuples = values(&tuples);
    let symbols = values(&inventory.symbols);
    let parameters = values(&inventory.parameters);
    let features = values(&inventory.features);
    let ports = values(&inventory.ports);
    let configuration = values(&inventory.configuration);
    let paths = PathInventory {
        instances: &instance_values,
        semantic_id_type: ctx
            .registry
            .logical_type("semantic_id")
            .ok_or_else(|| invalid("SemanticId declaration absent"))?
            .id,
        prospective: &prospective,
        submodels: &submodels,
        domains: &domains,
        members: &members,
        products: &products,
        valid_tuples: &valid_tuples,
        symbols: &symbols,
        parameters: &parameters,
        features: &features,
        ports: &ports,
        configuration: &configuration,
    };
    output.ensure::<i::path_targets::Row>()?;
    for path_row in &inventory.paths {
        ctx.cancel.checkpoint()?;
        let path = &path_row.row;
        let source = inventory.source(path.source_id)?;
        let segments = path
            .segments
            .iter()
            .map(|segment| {
                let kind = pse_schema::model::ExpressionPathSegmentKind::ALL
                    .into_iter()
                    .find(|kind| kind.as_str() == segment.kind.as_str())
                    .ok_or_else(|| invalid("path segment declaration absent"))?;
                Ok((kind, segment.name.as_str(), segment.index_count))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        for requester in &instances {
            use pse_relations::generated::normalized::expression_sources::NormalizedExpressionSourcesFieldOwnerSelected as Owner;
            let matches = match source.row.owner.selected()? {
                Owner::Instance(value) => value.instance_id == requester.row.instance_id,
                Owner::Template(value) => value.template_id == requester.row.template_id,
            };
            if !matches {
                continue;
            }
            let targets = pse_templates::paths::enumerate(
                paths,
                requester.row.instance_id,
                path.root_instance_id,
                &segments,
                work,
                ctx.cancel,
            )?;
            for target in targets {
                ctx.cancel.checkpoint()?;
                work.try_grow(
                    target
                        .path_index
                        .len()
                        .checked_add(16)
                        .and_then(|size| size.checked_mul(4096))
                        .ok_or_else(|| invalid("path support extent overflow"))?,
                )
                .map_err(pse_ids::CanonError::from)?;
                let mut support =
                    BTreeSet::from([inventory.origin(path_row)?, inventory.origin(source)?]);
                for owner in [requester.row.instance_id, target.owner] {
                    support.insert(inventory.origin(unique(
                        &instances,
                        |row| row.instance_id == owner,
                        "actual path instance",
                    )?)?);
                    support.insert(inventory.origin(inventory.instance(owner)?)?);
                }
                let mut root = path.root_instance_id.unwrap_or(requester.row.instance_id);
                if let Some((
                    pse_schema::model::ExpressionPathSegmentKind::InstanceParameter,
                    name,
                    _,
                )) = segments.first()
                {
                    let configured = unique(
                        &inventory.configuration,
                        |row| {
                            row.owner_id == requester.row.instance_id
                                && row.category.as_str() == "parameter"
                                && row.name == *name
                        },
                        "instance path parameter",
                    )?;
                    root = configured
                        .row
                        .value
                        .semantic_id
                        .as_ref()
                        .map(|arm| arm.value)
                        .ok_or_else(|| invalid("instance path parameter has no actual identity"))?;
                    support.insert(inventory.origin(configured)?);
                    member_support(
                        inventory,
                        requester.row.instance_id,
                        PathTargetKind::Parameter,
                        name,
                        &mut support,
                    )?;
                }
                ancestry_support(
                    inventory,
                    &instances,
                    target.owner,
                    root,
                    ctx,
                    work,
                    &mut support,
                )?;
                for (domain, member) in target.path_domains.iter().zip(&target.path_index) {
                    support.insert(inventory.origin(unique(
                        &inventory.members,
                        |row| row.domain_id == *domain && row.member_id == *member,
                        "path member",
                    )?)?);
                }
                let product = pse_templates::identity::domain_product_id(&target.path_domains);
                let declared = unique(
                    &inventory.products,
                    |row| row.product_id == product,
                    "path domain product",
                )?;
                if declared.row.domain_ids != target.path_domains {
                    return Err(invalid(
                        "path product differs from its exact ordered domains",
                    ));
                }
                support.insert(inventory.origin(declared)?);
                let Some(valid) = tuples.iter().find(|row| {
                    row.row.product_id == product && row.row.tuple == target.path_index
                }) else {
                    continue;
                };
                support.insert(inventory.origin(valid)?);
                let (kind, symbol, name, index) = match &target.member {
                    ResolvedMember::Symbol { declaration, index } => {
                        let declaration = unique(
                            &inventory.symbols,
                            |row| row.symbol_decl_id == *declaration,
                            "path symbol declaration",
                        )?;
                        for name in &declaration.row.indexed_by {
                            domain_support(inventory, target.owner, name, &mut support)?;
                        }
                        support.insert(inventory.origin(declaration)?);
                        (
                            PathTargetKind::Symbol,
                            Some(declaration.row.symbol_decl_id),
                            None,
                            index.clone(),
                        )
                    }
                    ResolvedMember::Parameter(name) => (
                        PathTargetKind::Parameter,
                        None,
                        Some(name.clone()),
                        Vec::new(),
                    ),
                    ResolvedMember::Feature(name) => (
                        PathTargetKind::Feature,
                        None,
                        Some(name.clone()),
                        Vec::new(),
                    ),
                    ResolvedMember::Port(name) => {
                        (PathTargetKind::Port, None, Some(name.clone()), Vec::new())
                    }
                };
                if let Some(name) = &name {
                    member_support(inventory, target.owner, kind, name, &mut support)?;
                }
                output.push(
                    i::path_targets::Row {
                        requester_instance_id: requester.row.instance_id,
                        source_id: path.source_id,
                        path_id: path.path_id,
                        path_index: target.path_index,
                        path_domains: target.path_domains,
                        owner_instance_id: target.owner,
                        target_kind: kind,
                        symbol_decl_id: symbol,
                        name,
                        source_index: index,
                        derivation_id: SemanticId::NIL,
                    },
                    &sources.locate(support)?,
                )?;
            }
        }
    }
    Ok(())
}
fn ancestry_support(
    inventory: &Inventory<'_>,
    instances: &[Keyed<i::instances::Row>],
    mut owner: SemanticId,
    root: SemanticId,
    ctx: &AlgorithmContext<'_>,
    work: &mut dyn Reservation,
    support: &mut Support,
) -> Result<(), CompilerError> {
    let mut visited = BTreeSet::new();
    loop {
        ctx.cancel.checkpoint()?;
        if !visited.insert(owner) {
            return Err(invalid("resolved path ancestry is cyclic"));
        }
        work.try_grow(8192).map_err(pse_ids::CanonError::from)?;
        let actual = unique(
            instances,
            |row| row.instance_id == owner,
            "actual path ancestor",
        )?;
        let child = inventory.instance(owner)?;
        support.insert(inventory.origin(actual)?);
        support.insert(inventory.origin(child)?);
        if owner == root {
            return Ok(());
        }
        let parent = child
            .row
            .parent_instance_id
            .ok_or_else(|| invalid("resolved leaf is outside its actual path root"))?;
        let template = child
            .row
            .submodel_template_id
            .ok_or_else(|| invalid("resolved child has no composite declaration"))?;
        let name = child
            .row
            .submodel_name
            .as_deref()
            .ok_or_else(|| invalid("resolved child name absent"))?;
        let declaration = unique(
            &inventory.submodels,
            |row| row.template_id == template && row.name == name,
            "path submodel",
        )?;
        if let Some(name) = &declaration.row.multiplicity_domain {
            domain_support(inventory, parent, name, support)?;
        }
        support.insert(inventory.origin(declaration)?);
        owner = parent;
    }
}
fn domain_support(
    inventory: &Inventory<'_>,
    owner: SemanticId,
    name: &str,
    support: &mut Support,
) -> Result<(), CompilerError> {
    support.insert(inventory.origin(unique(
        &inventory.domain_bindings,
        |row| row.instance_id == owner && row.domain_name == name,
        "path domain binding",
    )?)?);
    Ok(())
}
fn member_support(
    inventory: &Inventory<'_>,
    owner: SemanticId,
    kind: PathTargetKind,
    name: &str,
    support: &mut Support,
) -> Result<(), CompilerError> {
    let template = inventory.instance(owner)?.row.template_id;
    let origin = match kind {
        PathTargetKind::Parameter => inventory.origin(unique(
            &inventory.parameters,
            |row| row.template_id == template && row.name == name,
            "path parameter",
        )?)?,
        PathTargetKind::Feature => inventory.origin(unique(
            &inventory.features,
            |row| row.template_id == template && row.name == name,
            "path feature",
        )?)?,
        PathTargetKind::Port => inventory.origin(unique(
            &inventory.ports,
            |row| row.template_id == template && row.name == name,
            "path port",
        )?)?,
        PathTargetKind::Symbol => {
            return Err(invalid("symbol paths require the exact symbol declaration"));
        }
    };
    support.insert(origin);
    Ok(())
}
fn values<T: Clone>(rows: &[Keyed<T>]) -> Vec<T> {
    rows.iter().map(|row| row.row.clone()).collect()
}
