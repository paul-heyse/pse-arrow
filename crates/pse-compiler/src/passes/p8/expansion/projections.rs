// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact subgroup correspondence produced by indexed law substitution.
use super::super::outputs::Output;
use super::{Inputs, invalid, unique};
use crate::CompilerError;
use pse_relations::generated::compiled;

pub(super) fn emit(
    inputs: &Inputs,
    output: &mut Output<'_>,
    projection: &pse_templates::GroupProjection,
) -> Result<(), CompilerError> {
    let factors = projection
        .group
        .axes
        .iter()
        .map(|id| id.as_id())
        .collect::<Vec<_>>();
    let product = pse_templates::identity::domain_product_id(&factors);
    let declared = unique(
        &inputs.domain_products,
        |row| row.product_id == product && row.domain_ids == factors,
        "projection product",
    )?;
    output.use_row(declared);
    let groups = output.rows::<compiled::symbol_groups::Row>()?;
    let owner_instance_id = if let Some(source) = inputs
        .symbol_groups
        .iter()
        .find(|row| row.group_id == projection.source_group)
    {
        output.use_row(source);
        source.owner_instance_id
    } else {
        groups
            .iter()
            .find(|row| row.group_id == projection.source_group)
            .ok_or_else(|| invalid("law projection source group absent"))?
            .owner_instance_id
    };
    let group_id = projection.group.group;
    let expected = compiled::group_projections::Row {
        group_id,
        source_group_id: projection.source_group,
        fixed_coordinates: projection
            .fixed_coordinates
            .iter()
            .map(|(axis, member)| {
                compiled::group_projections::CompiledGroupProjectionsFieldFixedCoordinatesItem {
                    axis: *axis,
                    member_id: *member,
                }
            })
            .collect(),
        product_id: product,
        derivation_id: pse_ids::named_id(group_id, "P8:projection"),
    };
    if let Some(mut existing) = output
        .rows::<compiled::group_projections::Row>()?
        .into_iter()
        .find(|row| row.group_id == group_id)
    {
        existing.derivation_id = expected.derivation_id;
        if existing != expected {
            return Err(invalid(
                "law projection reuses an identity for different actual members",
            ));
        }
        return Ok(());
    }
    output.push(expected)?;
    output.push(compiled::symbol_groups::Row {
        group_id,
        owner_instance_id,
        name: format!(
            "projection:{}:{:?}",
            projection.source_group.to_hex(),
            projection.fixed_coordinates
        ),
        product_id: product,
    })?;
    for (tuple, symbol) in &projection.group.members {
        output.push(compiled::symbol_group_members::Row {
            group_id,
            tuple: tuple.clone(),
            symbol_id: *symbol,
        })?;
    }
    Ok(())
}
