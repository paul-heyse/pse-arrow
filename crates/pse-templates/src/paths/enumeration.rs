// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{
    Coordinate, PathInventory, PathRequest, PathTarget, parameter_instance, resolve, unique,
    validate_child,
};
use crate::TemplateError;
use pse_ids::{CancellationToken, Reservation, SemanticId};
use pse_schema::model::ExpressionPathSegmentKind as Segment;

struct Cursor {
    owner: SemanticId,
    coordinates: Vec<Coordinate>,
}

/// Enumerate every actual realized owner and complete local member tuple of a source path.
/// Index expressions do not select a subset here: this is the finite provider inventory.
/// The caller retains `work` while holding the returned values. Every output is checked
/// through [`resolve`]; missing or inconsistent facts are never swallowed as exclusions.
/// # Errors
/// Invalid paths, ambiguous declarations, inconsistent actual tuples, cancellation or
/// exhaustion of the caller's shared workspace reservation.
pub fn enumerate(
    inventory: PathInventory<'_>,
    requester: SemanticId,
    root_instance: Option<SemanticId>,
    segments: &[(Segment, &str, i64)],
    work: &mut dyn Reservation,
    cancel: &CancellationToken,
) -> Result<Vec<PathTarget>, TemplateError> {
    instance(inventory, requester, requester)?;
    reserve(work, 0, segments, requester)?;
    let mut cursors = vec![Cursor {
        owner: root_instance.unwrap_or(requester),
        coordinates: Vec::new(),
    }];
    let mut output = Vec::new();
    for (position, (kind, name, count)) in segments.iter().enumerate() {
        let last = position + 1 == segments.len();
        let mut next = Vec::new();
        for mut cursor in cursors {
            cancel.checkpoint()?;
            let instance = instance(inventory, cursor.owner, requester)?;
            match kind {
                Segment::InstanceParameter
                    if position == 0 && !last && *count == 0 && root_instance.is_none() =>
                {
                    cursor.owner = parameter_instance(inventory, instance, name, requester)?;
                    reserve(work, cursor.coordinates.len(), segments, requester)?;
                    next.push(cursor);
                }
                Segment::Child if !last => {
                    let declaration = unique(
                        inventory.submodels.iter().filter(|row| {
                            row.template_id == instance.template_id && row.name == *name
                        }),
                        requester,
                        "enumerated child key absent or ambiguous",
                    )?;
                    if usize::try_from(*count).ok()
                        != Some(declaration.multiplicity_domain.iter().count())
                    {
                        return Err(invalid(
                            requester,
                            "enumerated child coordinate partition differs",
                        ));
                    }
                    for child in inventory.prospective.iter().filter(|row| {
                        row.parent_instance_id == Some(cursor.owner)
                            && row.submodel_template_id == Some(instance.template_id)
                            && row.submodel_name.as_deref() == Some(*name)
                    }) {
                        if !inventory
                            .instances
                            .iter()
                            .any(|row| row.instance_id == child.instance_id)
                        {
                            continue;
                        }
                        validate_child(inventory, instance, declaration, child, requester)?;
                        reserve(
                            work,
                            cursor.coordinates.len() + child.index.len(),
                            segments,
                            requester,
                        )?;
                        let coordinates = cursor
                            .coordinates
                            .iter()
                            .copied()
                            .chain(child.index.iter().copied().map(Coordinate::Member))
                            .collect();
                        next.push(Cursor {
                            owner: child.instance_id,
                            coordinates,
                        });
                    }
                }
                Segment::Member if last => enumerate_member(
                    inventory,
                    PathRequest {
                        requester,
                        root_instance,
                        segments,
                        coordinates: &cursor.coordinates,
                    },
                    (name, *count),
                    cursor.owner,
                    &mut output,
                    work,
                    cancel,
                )?,
                _ => return Err(invalid(requester, "enumerated path step order differs")),
            }
        }
        cursors = next;
    }
    if segments
        .last()
        .is_none_or(|(kind, _, _)| *kind != Segment::Member)
    {
        return Err(invalid(requester, "enumerated path has no final member"));
    }
    Ok(output)
}

fn enumerate_member(
    inventory: PathInventory<'_>,
    request: PathRequest<'_>,
    (name, count): (&str, i64),
    owner: SemanticId,
    output: &mut Vec<PathTarget>,
    work: &mut dyn Reservation,
    cancel: &CancellationToken,
) -> Result<(), TemplateError> {
    let PathRequest {
        requester,
        root_instance: root,
        segments,
        coordinates,
    } = request;
    let instance = unique(
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == owner),
        requester,
        "member owner absent",
    )?;
    let mut symbols = inventory
        .symbols
        .iter()
        .filter(|row| row.template_id == instance.template_id && row.name == name);
    if let Some(symbol) = symbols.next() {
        if symbols.next().is_some() || usize::try_from(count).ok() != Some(symbol.indexed_by.len())
        {
            return Err(invalid(
                requester,
                "member declaration or coordinate arity is ambiguous",
            ));
        }
        reserve(work, symbol.indexed_by.len(), segments, requester)?;
        let domains = symbol
            .indexed_by
            .iter()
            .map(|name| {
                unique(
                    inventory
                        .domains
                        .iter()
                        .filter(|row| row.instance_id == owner && row.domain_name == *name),
                    requester,
                    "member domain binding absent",
                )
                .map(|row| row.domain_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let product = unique(
            inventory
                .products
                .iter()
                .filter(|row| row.domain_ids == domains),
            requester,
            "member domain product absent or ambiguous",
        )?;
        for tuple in inventory
            .valid_tuples
            .iter()
            .filter(|row| row.product_id == product.product_id)
        {
            cancel.checkpoint()?;
            reserve(
                work,
                coordinates.len() + tuple.tuple.len(),
                segments,
                requester,
            )?;
            let coordinates = coordinates
                .iter()
                .copied()
                .chain(tuple.tuple.iter().copied().map(Coordinate::Member))
                .collect::<Vec<_>>();
            output.push(resolve(
                inventory,
                PathRequest {
                    requester,
                    root_instance: root,
                    segments,
                    coordinates: &coordinates,
                },
                cancel,
            )?);
        }
    } else {
        if count != 0 {
            return Err(invalid(
                requester,
                "non-symbol member cannot have index coordinates",
            ));
        }
        reserve(work, coordinates.len(), segments, requester)?;
        output.push(resolve(
            inventory,
            PathRequest {
                requester,
                root_instance: root,
                segments,
                coordinates,
            },
            cancel,
        )?);
    }
    Ok(())
}

fn reserve(
    work: &mut dyn Reservation,
    coordinates: usize,
    segments: &[(Segment, &str, i64)],
    instance: SemanticId,
) -> Result<(), TemplateError> {
    let invalid = || TemplateError::Binding {
        instance,
        detail: "path enumeration workspace extent overflow".to_owned(),
    };
    let labels = segments.iter().try_fold(0_usize, |sum, (_, name, _)| {
        sum.checked_add(name.len()).ok_or_else(invalid)
    })?;
    let bytes = coordinates
        .checked_add(segments.len())
        .and_then(|count| count.checked_mul(128))
        .and_then(|bytes| bytes.checked_add(labels.checked_mul(4)?))
        .and_then(|bytes| bytes.checked_add(512))
        .ok_or_else(invalid)?;
    work.try_grow(bytes).map_err(pse_ids::CanonError::from)?;
    Ok(())
}

fn invalid(instance: SemanticId, detail: &str) -> TemplateError {
    TemplateError::Binding {
        instance,
        detail: detail.to_owned(),
    }
}

fn instance(
    inventory: PathInventory<'_>,
    owner: SemanticId,
    requester: SemanticId,
) -> Result<&pse_relations::generated::inferred::instances::Row, TemplateError> {
    unique(
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == owner),
        requester,
        "path instance absent or ambiguous",
    )
}
