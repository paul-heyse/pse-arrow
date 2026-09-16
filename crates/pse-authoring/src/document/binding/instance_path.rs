// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{BoundPath, Context, PathMeaning, paths};
use crate::{AuthoringError, SourceSpan, document::load::contract, dsl};
use pse_ids::SemanticId;
use pse_schema::model::ExpressionPathSegmentKind;
use std::collections::BTreeSet;

/// One exact declared step with its original index-expression partition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstancePathSegment {
    /// Child, parameter-selected root, or final member.
    pub kind: ExpressionPathSegmentKind,
    /// Exact declared key within the selected owning template.
    pub name: String,
    /// Number of graph-held index expressions attached to this step.
    pub index_count: u16,
}
/// A path remains relative until its actual occurrence and selected templates are known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundInstancePath {
    /// A globally referenced actual instance, otherwise the source occurrence's owner.
    pub root_instance_id: Option<SemanticId>,
    /// Exact ordered steps; no display-path parsing is needed by P7.
    pub segments: Vec<InstancePathSegment>,
    /// Actual known leaf declarations across selected contexts, used to prevent partial rename.
    pub member_entities: Vec<SemanticId>,
    /// Exact enum shared by every resolved leaf declaration, when enum-valued.
    pub member_enum: Option<SemanticId>,
    /// The actual selected owner is required before admitting the leaf's type.
    pub deferred: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Owner {
    instance: Option<SemanticId>,
    template: SemanticId,
}

pub(super) fn bind(
    path: &dsl::Path,
    span: dsl::Span,
    owner: SemanticId,
    context: &Context,
    at: SourceSpan,
) -> Result<Option<BoundPath>, AuthoringError> {
    if path.segments.len() < 2 {
        return Ok(None);
    }
    let first = &path.segments[0];
    let child = context
        .submodels
        .iter()
        .any(|row| row.template_id == owner && row.name == first.name);
    let parameter = context
        .params
        .iter()
        .find(|row| row.template_id == owner && row.name == first.name);
    let mut entities = vec![None; path.segments.len()];
    let (root_instance, start, mut owners) = if child || parameter.is_some() {
        (None, 0, owners(owner, context))
    } else if let Some((instance, prefix)) = actual_root(path, context, &mut entities)? {
        (
            Some(instance.instance_id),
            prefix,
            vec![Owner {
                instance: Some(instance.instance_id),
                template: instance.template_id,
            }],
        )
    } else {
        return Ok(None);
    };
    let mut segments = Vec::new();
    let mut deferred = false;
    for (ordinal, segment) in path.segments.iter().enumerate().skip(start) {
        let final_member = ordinal + 1 == path.segments.len();
        let kind = if ordinal == 0 && parameter.is_some() {
            let parameter = parameter.ok_or_else(|| contract(Some(at), "parameter root absent"))?;
            if parameter.logical_type_id != context.semantic_id_type || !segment.indices.is_empty()
            {
                return Err(contract(
                    Some(at),
                    "an instance-valued root requires an unsubscripted SemanticId parameter",
                ));
            }
            deferred = true;
            ExpressionPathSegmentKind::InstanceParameter
        } else if final_member {
            ExpressionPathSegmentKind::Member
        } else {
            ExpressionPathSegmentKind::Child
        };
        segments.push(InstancePathSegment {
            kind,
            name: segment.name.clone(),
            index_count: u16::try_from(segment.indices.len())
                .map_err(|_| contract(Some(at), "path index count overflow"))?,
        });
        if !final_member && kind == ExpressionPathSegmentKind::Child && !deferred {
            let (next, unresolved) = descend(&owners, segment, context, at)?;
            deferred = unresolved;
            owners = next;
        }
    }
    let (members, member_enum) = if deferred {
        (Vec::new(), None)
    } else {
        members(
            &owners,
            path.segments
                .last()
                .ok_or_else(|| contract(Some(at), "path has no member"))?,
            context,
            at,
        )?
    };
    if let [member] = members.as_slice()
        && let Some(last) = entities.last_mut()
    {
        *last = Some(*member);
    }
    Ok(Some(BoundPath {
        span,
        meaning: PathMeaning::InstancePath(BoundInstancePath {
            root_instance_id: root_instance,
            segments,
            member_entities: members,
            member_enum,
            deferred,
        }),
        segment_entities: entities,
    }))
}
fn owners(template: SemanticId, context: &Context) -> Vec<Owner> {
    let actual = context
        .prospective
        .iter()
        .filter(|row| row.template_id == template)
        .map(|row| Owner {
            instance: Some(row.instance_id),
            template,
        })
        .collect::<Vec<_>>();
    if actual.is_empty() {
        vec![Owner {
            instance: None,
            template,
        }]
    } else {
        actual
    }
}
fn descend(
    owners: &[Owner],
    segment: &dsl::PathSegment,
    context: &Context,
    at: SourceSpan,
) -> Result<(Vec<Owner>, bool), AuthoringError> {
    let mut next = BTreeSet::new();
    let mut deferred = false;
    for owner in owners {
        let choices = context
            .submodels
            .iter()
            .filter(|row| row.template_id == owner.template && row.name == segment.name)
            .collect::<Vec<_>>();
        let [declaration] = choices.as_slice() else {
            return Err(contract(
                Some(at),
                "submodel path key is missing or ambiguous under the actual selected template",
            ));
        };
        if segment.indices.len() != usize::from(declaration.multiplicity_domain.is_some()) {
            return Err(contract(
                Some(at),
                "submodel path index arity differs from its actual declared axes",
            ));
        }
        let children = context
            .prospective
            .iter()
            .filter(|child| {
                child.parent_instance_id == owner.instance
                    && owner.instance.is_some()
                    && child.submodel_template_id == Some(owner.template)
                    && child.submodel_name.as_ref() == Some(&segment.name)
            })
            .collect::<Vec<_>>();
        if !children.is_empty() {
            for child in children {
                if declaration
                    .child_template_id
                    .is_some_and(|template| template != child.template_id)
                    || !context
                        .templates
                        .iter()
                        .any(|template| template.template_id == child.template_id)
                {
                    return Err(contract(
                        Some(at),
                        "prospective child differs from its declared selected template",
                    ));
                }
                next.insert(Owner {
                    instance: Some(child.instance_id),
                    template: child.template_id,
                });
            }
        } else if let Some(template) = declaration.child_template_id {
            next.insert(Owner {
                instance: None,
                template,
            });
        } else if declaration.child_from_param.is_none() {
            return Err(contract(
                Some(at),
                "submodel has no explicit template selection",
            ));
        } else {
            deferred = true;
        }
    }
    Ok((next.into_iter().collect(), deferred))
}
fn members(
    owners: &[Owner],
    segment: &dsl::PathSegment,
    context: &Context,
    at: SourceSpan,
) -> Result<(Vec<SemanticId>, Option<SemanticId>), AuthoringError> {
    let mut entities = BTreeSet::new();
    let mut enumerations = BTreeSet::new();
    for owner in owners {
        let meaning =
            paths::local(owner.template, &segment.name, context, at)?.ok_or_else(|| {
                contract(
                    Some(at),
                    "member is absent under the actual selected template",
                )
            })?;
        let enumeration = match meaning {
            PathMeaning::Symbol { symbol_id, .. } => {
                let symbol = context
                    .symbols
                    .iter()
                    .find(|symbol| symbol.symbol_decl_id == symbol_id)
                    .ok_or_else(|| contract(Some(at), "bound symbol declaration missing"))?;
                if segment.indices.len() != symbol.indexed_by.len() {
                    return Err(contract(
                        Some(at),
                        "symbol path index arity differs from its actual declared axes",
                    ));
                }
                entities.insert(symbol_id);
                None
            }
            PathMeaning::Parameter { template_id, name } if segment.indices.is_empty() => context
                .params
                .iter()
                .find(|row| row.template_id == template_id && row.name == name)
                .and_then(|row| row.enum_id),
            PathMeaning::Feature { template_id, name } if segment.indices.is_empty() => context
                .features
                .iter()
                .find(|row| row.template_id == template_id && row.name == name)
                .and_then(|row| row.enum_id),
            PathMeaning::Port { .. } if segment.indices.is_empty() => None,
            _ => {
                return Err(contract(
                    Some(at),
                    "path member is not an admitted scalar/indexed value",
                ));
            }
        };
        enumerations.insert(enumeration);
    }
    if enumerations.len() > 1 {
        return Err(contract(
            Some(at),
            "selected submodel members disagree on their exact enum type",
        ));
    }
    Ok((
        entities.into_iter().collect(),
        enumerations.pop_first().flatten(),
    ))
}
fn actual_root<'a>(
    path: &dsl::Path,
    context: &'a Context,
    entities: &mut [Option<SemanticId>],
) -> Result<
    Option<(
        &'a pse_relations::generated::authored::instances::Row,
        usize,
    )>,
    AuthoringError,
> {
    let prefix = path.segments[..path.segments.len() - 1]
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".");
    let choices = context
        .entities
        .iter()
        .filter(|entity| {
            entity.qualified_name == prefix
                || entity.qualified_name.ends_with(&format!(".{prefix}"))
        })
        .filter_map(|entity| {
            context
                .instances
                .iter()
                .find(|instance| instance.instance_id == entity.entity_id)
        })
        .collect::<Vec<_>>();
    if choices.is_empty() {
        return Ok(None);
    }
    let [instance] = choices.as_slice() else {
        return Err(contract(None, "instance path root is ambiguous"));
    };
    let prefix_count = path.segments.len() - 1;
    if path.segments[..prefix_count]
        .iter()
        .any(|part| !part.indices.is_empty())
    {
        return Err(contract(
            None,
            "an explicit actual instance root cannot carry child indices",
        ));
    }
    for (position, slot) in entities.iter_mut().enumerate().take(prefix_count) {
        let prefix = path.segments[..=position]
            .iter()
            .map(|part| part.name.as_str())
            .collect::<Vec<_>>()
            .join(".");
        let matches = context
            .entities
            .iter()
            .filter(|entity| {
                entity.qualified_name == prefix
                    || entity.qualified_name.ends_with(&format!(".{prefix}"))
            })
            .collect::<Vec<_>>();
        if let [entity] = matches.as_slice() {
            *slot = Some(entity.entity_id);
        }
    }
    Ok(Some((instance, prefix_count)))
}
