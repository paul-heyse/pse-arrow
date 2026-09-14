// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{BoundPath, Context, PathMeaning};
use crate::document::load::contract;
use crate::{AuthoringError, SourceSpan, dsl};
use pse_ids::SemanticId;
use std::collections::BTreeSet;

pub(super) fn bind(
    path: &dsl::Path,
    span: dsl::Span,
    owner: SemanticId,
    context: &Context,
    locals: &BTreeSet<String>,
    at: SourceSpan,
) -> Result<BoundPath, AuthoringError> {
    let names = path
        .segments
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>();
    let first = names
        .first()
        .ok_or_else(|| contract(Some(at), "empty expression path"))?;
    if locals.contains(*first) {
        if names.len() != 1 {
            return Err(contract(
                Some(at),
                "a lexical value cannot own qualified members",
            ));
        }
        return Ok(BoundPath {
            span,
            meaning: PathMeaning::Local((*first).to_owned()),
            segment_entities: vec![None; names.len()],
        });
    }
    let mut segments = vec![None; names.len()];
    let meaning = if names.len() == 1 {
        match local(owner, first, context, at)? {
            Some(meaning) => meaning,
            None => global(&names, context, &mut segments, at)?,
        }
    } else {
        qualified(owner, &names, context, &mut segments, at)?
    };
    if let Some(last) = segments.last_mut() {
        *last = match meaning {
            PathMeaning::Symbol { symbol_id, .. } => Some(symbol_id),
            PathMeaning::Equation { equation_id, .. } => Some(equation_id),
            PathMeaning::Entity(id) => Some(id),
            _ => *last,
        };
    }
    Ok(BoundPath {
        span,
        meaning,
        segment_entities: segments,
    })
}

fn local(
    owner: SemanticId,
    name: &str,
    context: &Context,
    at: SourceSpan,
) -> Result<Option<PathMeaning>, AuthoringError> {
    let mut choices = Vec::new();
    choices.extend(
        context
            .targets
            .symbols
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Symbol {
                template_id: owner,
                symbol_id: row.symbol_decl_id,
            }),
    );
    choices.extend(
        context
            .targets
            .equations
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Equation {
                template_id: owner,
                equation_id: row.equation_decl_id,
            }),
    );
    choices.extend(
        context
            .targets
            .ports
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Port {
                template_id: owner,
                name: row.name.clone(),
            }),
    );
    choices.extend(
        context
            .targets
            .template_domains
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Domain {
                template_id: owner,
                name: row.name.clone(),
            }),
    );
    choices.extend(
        context
            .params
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Parameter {
                template_id: owner,
                name: row.name.clone(),
            }),
    );
    choices.extend(
        context
            .features
            .iter()
            .filter(|row| row.template_id == owner && row.name == name)
            .map(|row| PathMeaning::Feature {
                template_id: owner,
                name: row.name.clone(),
            }),
    );
    if choices.is_empty() {
        return Ok(None);
    }
    let [meaning] = choices.as_slice() else {
        return Err(contract(
            Some(at),
            &format!("unbound or ambiguous template path `{name}`"),
        ));
    };
    Ok(Some(meaning.clone()))
}

pub(super) fn unbound_bare(
    name: &str,
    owner: SemanticId,
    context: &Context,
    locals: &BTreeSet<String>,
    at: SourceSpan,
) -> Result<bool, AuthoringError> {
    if locals.contains(name) || local(owner, name, context, at)?.is_some() {
        return Ok(false);
    }
    Ok(!context.targets.entities.iter().any(|entity| {
        entity.qualified_name == name || entity.qualified_name.ends_with(&format!(".{name}"))
    }))
}

fn qualified(
    owner: SemanticId,
    names: &[&str],
    context: &Context,
    segments: &mut [Option<SemanticId>],
    at: SourceSpan,
) -> Result<PathMeaning, AuthoringError> {
    let mut template = owner;
    let mut traversed = 0;
    for (index, name) in names[..names.len() - 1].iter().enumerate() {
        let children = context
            .submodels
            .iter()
            .filter(|row| row.template_id == template && row.name == *name)
            .collect::<Vec<_>>();
        let [child] = children.as_slice() else {
            break;
        };
        template = child.child_template_id.ok_or_else(|| {
            contract(
                Some(at),
                "a parameter-selected submodel requires an explicit resolved template binding",
            )
        })?;
        traversed = index + 1;
    }
    if traversed == names.len() - 1 {
        return local(template, names[names.len() - 1], context, at)?
            .ok_or_else(|| contract(Some(at), "unbound submodel member"));
    }
    if traversed > 0 {
        return Err(contract(Some(at), "unbound nested submodel path"));
    }
    if let Ok(meaning) = global(names, context, segments, at) {
        return Ok(meaning);
    }
    let prefix = names[..names.len() - 1].join(".");
    let owners = context
        .targets
        .entities
        .iter()
        .filter(|row| {
            row.qualified_name == prefix || row.qualified_name.ends_with(&format!(".{prefix}"))
        })
        .flat_map(|entity| {
            context
                .targets
                .instances
                .iter()
                .filter(move |row| row.instance_id == entity.entity_id)
        })
        .collect::<Vec<_>>();
    let [instance] = owners.as_slice() else {
        return Err(contract(
            Some(at),
            &format!("unbound or ambiguous expression path `{}`", names.join(".")),
        ));
    };
    for index in 0..names.len() - 1 {
        let prefix = names[..=index].join(".");
        let entities = context
            .targets
            .entities
            .iter()
            .filter(|row| {
                row.qualified_name == prefix || row.qualified_name.ends_with(&format!(".{prefix}"))
            })
            .collect::<Vec<_>>();
        if let [entity] = entities.as_slice() {
            segments[index] = Some(entity.entity_id);
        }
    }
    local(instance.template_id, names[names.len() - 1], context, at)?
        .ok_or_else(|| contract(Some(at), "unbound instance member"))
}

fn global(
    names: &[&str],
    context: &Context,
    segments: &mut [Option<SemanticId>],
    at: SourceSpan,
) -> Result<PathMeaning, AuthoringError> {
    let name = names.join(".");
    let entities = context
        .targets
        .entities
        .iter()
        .filter(|entity| {
            entity.qualified_name == name || entity.qualified_name.ends_with(&format!(".{name}"))
        })
        .collect::<Vec<_>>();
    let [entity] = entities.as_slice() else {
        return Err(contract(
            Some(at),
            &format!("unbound or ambiguous global path `{name}`"),
        ));
    };
    let mut current = Some(*entity);
    for index in (0..names.len()).rev() {
        if let Some(entity) = current {
            if entity.name != names[index] {
                break;
            }
            segments[index] = Some(entity.entity_id);
            current = entity.parent_entity_id.and_then(|id| {
                context
                    .targets
                    .entities
                    .iter()
                    .find(|entity| entity.entity_id == id)
            });
        }
    }
    Ok(PathMeaning::Entity(entity.entity_id))
}
