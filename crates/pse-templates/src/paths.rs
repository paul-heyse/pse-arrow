// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure finite path resolution shared by P5 demand closure and P7 realization.

mod enumeration;
pub use enumeration::enumerate;

use crate::TemplateError;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{enums::ConfigCategory, inferred, normalized};
use pse_schema::model::ExpressionPathSegmentKind;

/// A concrete coordinate obtained by evaluating a source index in its actual context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coordinate {
    /// Exact semantic member identity, independently checked in the selected actual domain.
    Member(SemanticId),
    /// Exact integer value, matched to actual numeric domain coordinates, never ordinals.
    Integer(i64),
}
/// Borrowed admitted row views used by the finite resolver; no graph or symbol output needed.
#[derive(Clone, Copy, Debug)]
pub struct PathInventory<'a> {
    /// Actual realized instances.
    pub instances: &'a [inferred::instances::Row],
    /// Exact registry logical-type identity for `SemanticId` parameters.
    pub semantic_id_type: SemanticId,
    /// Exact prospective parent/submodel/selected-template correspondence.
    pub prospective: &'a [normalized::instance_bindings::Row],
    /// Declared submodel keys and explicit selection contracts.
    pub submodels: &'a [normalized::template_submodels::Row],
    /// Actual per-instance domain bindings.
    pub domains: &'a [normalized::instance_domain_bindings::Row],
    /// Actual coordinate/member facts.
    pub members: &'a [normalized::domain_members::Row],
    /// Actual ordered product declarations.
    pub products: &'a [normalized::domain_products::Row],
    /// Actual admitted tuples, including phase/species restrictions.
    pub valid_tuples: &'a [inferred::valid_index_tuples::Row],
    /// Symbol declaration inventory.
    pub symbols: &'a [normalized::template_symbols::Row],
    /// Parameter declarations.
    pub parameters: &'a [normalized::template_params::Row],
    /// Feature declarations.
    pub features: &'a [normalized::template_features::Row],
    /// Port declarations.
    pub ports: &'a [normalized::template_ports::Row],
    /// Actual parameter assignments under their declared type.
    pub configuration: &'a [normalized::config_values::Row],
}
/// One occurrence's exact requested traversal, without any guessed selected template.
#[derive(Clone, Copy, Debug)]
pub struct PathRequest<'a> {
    /// Instance containing the source occurrence.
    pub requester: SemanticId,
    /// Explicit actual global root, if any.
    pub root_instance: Option<SemanticId>,
    /// Ordered kind/name/index-count projection from the registered source path row.
    pub segments: &'a [(ExpressionPathSegmentKind, &'a str, u16)],
    /// Actual index-expression results, preserving the declared segment partition.
    pub coordinates: &'a [Coordinate],
}
/// Exact final declaration key under the actual selected owning instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedMember {
    /// Symbol declaration plus its ordered actual local member tuple.
    Symbol {
        /// Exact selected symbol declaration.
        declaration: SemanticId,
        /// Ordered actual local domain members.
        index: Vec<SemanticId>,
    },
    /// Declared parameter composite key.
    Parameter(String),
    /// Declared feature composite key.
    Feature(String),
    /// Declared port composite key.
    Port(String),
}
/// Actual path result before any physical symbol realization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathTarget {
    /// Actual selected owning instance.
    pub owner: SemanticId,
    /// Exact selected declaration key.
    pub member: ResolvedMember,
    /// Full concrete path tuple, including child indices in original segment order.
    pub path_index: Vec<SemanticId>,
    /// Exact ordered domains associated with that tuple.
    pub path_domains: Vec<SemanticId>,
}

/// Resolve one finite occurrence using actual parent, parameter and domain rows.
/// # Errors
/// Missing/ambiguous actual keys, a disabled child, wrong member coordinates, incompatible
/// declaration selection, malformed path partitions or cancellation.
pub fn resolve(
    inventory: PathInventory<'_>,
    request: PathRequest<'_>,
    cancel: &CancellationToken,
) -> Result<PathTarget, TemplateError> {
    let invalid = |detail: &str| TemplateError::Binding {
        instance: request.requester,
        detail: detail.to_owned(),
    };
    unique(
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == request.requester),
        request.requester,
        "path requester is absent or ambiguous",
    )?;
    let mut owner = request.root_instance.unwrap_or(request.requester);
    let mut cursor = 0_usize;
    let mut path_index = Vec::new();
    let mut path_domains = Vec::new();
    for (ordinal, (kind, name, count)) in request.segments.iter().enumerate() {
        cancel.checkpoint()?;
        let instance = unique(
            inventory
                .instances
                .iter()
                .filter(|row| row.instance_id == owner),
            request.requester,
            "path owner is absent or ambiguous",
        )?;
        let end = cursor
            .checked_add(usize::from(*count))
            .ok_or_else(|| invalid("path index extent overflow"))?;
        let coordinates = request
            .coordinates
            .get(cursor..end)
            .ok_or_else(|| invalid("path index partition exceeds supplied coordinates"))?;
        let final_step = ordinal + 1 == request.segments.len();
        match kind {
            ExpressionPathSegmentKind::InstanceParameter
                if ordinal == 0
                    && !final_step
                    && *count == 0
                    && request.root_instance.is_none() =>
            {
                owner = parameter_instance(inventory, instance, name, request.requester)?;
            }
            ExpressionPathSegmentKind::Child if !final_step => {
                let declaration = unique(
                    inventory
                        .submodels
                        .iter()
                        .filter(|row| row.template_id == instance.template_id && row.name == *name),
                    request.requester,
                    "child composite key missing or ambiguous",
                )?;
                let axis_names = declaration
                    .multiplicity_domain
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let (tuple, domains) = coordinates_for(inventory, owner, &axis_names, coordinates)?;
                let child = unique(
                    inventory.prospective.iter().filter(|row| {
                        row.parent_instance_id == Some(owner)
                            && row.submodel_template_id == Some(instance.template_id)
                            && row.submodel_name.as_deref() == Some(name)
                            && row.index == tuple
                    }),
                    request.requester,
                    "exact child index is absent or ambiguous",
                )?;
                validate_child(inventory, instance, declaration, child, request.requester)?;
                path_index.extend(tuple);
                path_domains.extend(domains);
                owner = child.instance_id;
            }
            ExpressionPathSegmentKind::Member if final_step => {
                if end != request.coordinates.len() {
                    return Err(invalid("path leaves unused coordinate expressions"));
                }
                let (member, tuple, domains) =
                    member(inventory, instance, name, coordinates, request.requester)?;
                path_index.extend(tuple);
                path_domains.extend(domains);
                return Ok(PathTarget {
                    owner,
                    member,
                    path_index,
                    path_domains,
                });
            }
            _ => return Err(invalid("path step order or root contract differs")),
        }
        cursor = end;
    }
    Err(invalid("path has no final member"))
}

fn unique<'a, T>(
    mut values: impl Iterator<Item = &'a T>,
    instance: SemanticId,
    detail: &str,
) -> Result<&'a T, TemplateError> {
    let first = values.next();
    match (first, values.next()) {
        (Some(value), None) => Ok(value),
        _ => Err(TemplateError::Binding {
            instance,
            detail: detail.to_owned(),
        }),
    }
}
fn parameter_instance(
    inventory: PathInventory<'_>,
    instance: &inferred::instances::Row,
    name: &str,
    requester: SemanticId,
) -> Result<SemanticId, TemplateError> {
    let declaration = unique(
        inventory
            .parameters
            .iter()
            .filter(|row| row.template_id == instance.template_id && row.name == name),
        requester,
        "instance parameter declaration absent or ambiguous",
    )?;
    let value = unique(
        inventory.configuration.iter().filter(|row| {
            row.owner_id == instance.instance_id
                && row.category == ConfigCategory::Parameter
                && row.name == name
        }),
        requester,
        "actual instance parameter value absent or ambiguous",
    )?;
    if declaration.logical_type_id != inventory.semantic_id_type
        || declaration.enum_id.is_some()
        || value.value.kind.as_str() != "semantic_id"
    {
        return Err(TemplateError::Binding {
            instance: requester,
            detail: "instance root is not an admitted SemanticId parameter".to_owned(),
        });
    }
    let selected = value
        .value
        .semantic_id
        .as_ref()
        .map(|arm| arm.value)
        .ok_or_else(|| TemplateError::Binding {
            instance: requester,
            detail: "instance parameter has no actual identity".to_owned(),
        })?;
    unique(
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == selected),
        requester,
        "parameter-selected actual instance absent or ambiguous",
    )?;
    Ok(selected)
}
fn validate_child(
    inventory: PathInventory<'_>,
    parent: &inferred::instances::Row,
    declaration: &normalized::template_submodels::Row,
    child: &normalized::instance_bindings::Row,
    requester: SemanticId,
) -> Result<(), TemplateError> {
    let invalid = |detail: &str| TemplateError::Binding {
        instance: requester,
        detail: detail.to_owned(),
    };
    let realized = unique(
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == child.instance_id),
        requester,
        "path demands an unrealized child",
    )?;
    if realized.parent_instance_id != child.parent_instance_id
        || realized.template_id != child.template_id
        || realized.index != child.index
        || realized.path != child.path
    {
        return Err(invalid(
            "realized child differs from complete prospective correspondence",
        ));
    }
    // P3 has already resolved fixed, parameter-selected and property-package state
    // templates. Traversal consumes that admitted correspondence; a selector's
    // SemanticId may denote a package rather than the selected template itself.
    if child.parent_instance_id != Some(parent.instance_id)
        || child.submodel_template_id != Some(declaration.template_id)
        || child.submodel_name.as_deref() != Some(declaration.name.as_str())
        || declaration.template_id != parent.template_id
    {
        return Err(invalid(
            "child differs from its exact submodel correspondence",
        ));
    }
    Ok(())
}

fn coordinates_for(
    inventory: PathInventory<'_>,
    owner: SemanticId,
    names: &[&str],
    coordinates: &[Coordinate],
) -> Result<(Vec<SemanticId>, Vec<SemanticId>), TemplateError> {
    let invalid = |detail: &str| TemplateError::Binding {
        instance: owner,
        detail: detail.to_owned(),
    };
    if names.len() != coordinates.len() {
        return Err(invalid(
            "path coordinate count differs from declared local axes",
        ));
    }
    let mut members = Vec::new();
    let mut domains = Vec::new();
    for (name, coordinate) in names.iter().zip(coordinates) {
        let binding = unique(
            inventory
                .domains
                .iter()
                .filter(|row| row.instance_id == owner && row.domain_name == *name),
            owner,
            "actual path domain binding missing or ambiguous",
        )?;
        let member = unique(
            inventory.members.iter().filter(|row| {
                row.domain_id == binding.domain_id
                    && match coordinate {
                        Coordinate::Member(member) => row.member_id == *member,
                        Coordinate::Integer(integer) => row.coordinate.is_some_and(|value| {
                            value.is_finite()
                                && value
                                    .to_string()
                                    .parse::<i64>()
                                    .is_ok_and(|value| value == *integer)
                        }),
                    }
            }),
            owner,
            "path coordinate has no unique actual domain member",
        )?;
        members.push(member.member_id);
        domains.push(binding.domain_id);
    }
    let product = unique(
        inventory
            .products
            .iter()
            .filter(|row| row.domain_ids == domains),
        owner,
        "path domain product missing or ambiguous",
    )?;
    if product.product_id != crate::identity::domain_product_id(&domains)
        || !inventory
            .valid_tuples
            .iter()
            .any(|row| row.product_id == product.product_id && row.tuple == members)
    {
        return Err(invalid("path tuple is outside its actual admitted product"));
    }
    Ok((members, domains))
}
type MemberResolution = (ResolvedMember, Vec<SemanticId>, Vec<SemanticId>);
fn member(
    inventory: PathInventory<'_>,
    instance: &inferred::instances::Row,
    name: &str,
    coordinates: &[Coordinate],
    requester: SemanticId,
) -> Result<MemberResolution, TemplateError> {
    let invalid = |detail: &str| TemplateError::Binding {
        instance: requester,
        detail: detail.to_owned(),
    };
    let mut choices = Vec::new();
    for declaration in inventory
        .symbols
        .iter()
        .filter(|row| row.template_id == instance.template_id && row.name == name)
    {
        let names = declaration
            .indexed_by
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let (tuple, domains) =
            coordinates_for(inventory, instance.instance_id, &names, coordinates)?;
        choices.push((
            ResolvedMember::Symbol {
                declaration: declaration.symbol_decl_id,
                index: tuple.clone(),
            },
            tuple,
            domains,
        ));
    }
    for (kind, count) in [
        (
            ResolvedMember::Parameter(name.to_owned()),
            inventory
                .parameters
                .iter()
                .filter(|row| row.template_id == instance.template_id && row.name == name)
                .count(),
        ),
        (
            ResolvedMember::Feature(name.to_owned()),
            inventory
                .features
                .iter()
                .filter(|row| row.template_id == instance.template_id && row.name == name)
                .count(),
        ),
        (
            ResolvedMember::Port(name.to_owned()),
            inventory
                .ports
                .iter()
                .filter(|row| row.template_id == instance.template_id && row.name == name)
                .count(),
        ),
    ] {
        if count > 1 {
            return Err(invalid("duplicate selected member declaration"));
        }
        if count == 1 {
            if !coordinates.is_empty() {
                return Err(invalid(
                    "scalar configuration/port member carries undeclared indices",
                ));
            }
            choices.push((kind, Vec::new(), Vec::new()));
        }
    }
    if choices.len() != 1 {
        return Err(invalid(
            "path leaf declaration is missing or ambiguous under selected owner",
        ));
    }
    choices.pop().ok_or_else(|| invalid("path member absent"))
}
