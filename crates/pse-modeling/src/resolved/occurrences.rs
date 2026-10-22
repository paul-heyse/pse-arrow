// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original attribution is selected separately from reusable numerical construction.
use super::*;
use pse_model::diagnostic::SourceLocation;
use std::collections::BTreeSet;
/// Stable local diagnostic role, qualified by every actual selected caller operation.
/// Neither authored IDs nor byte positions enter this local token.
pub fn occurrence_token(path: &[usize], predicate: bool, ordinal: usize) -> SemanticId {
    let mut hash = FramedHasher::new(pse_ids::Frame::MathLocalOccurrenceV3);
    hash.str("resolved-occurrence").u64(path.len() as u64);
    for component in path {
        hash.u64(*component as u64);
    }
    hash.bool(predicate).u64(ordinal as u64);
    hash.finish_id()
}
impl ResolvedSyntax {
    /// Exact finite local source-token demand, including typed function call paths.
    pub fn occurrence_tokens(&self, limit: usize) -> Result<BTreeSet<SemanticId>> {
        let mut output = BTreeSet::new();
        let mut remaining = limit;
        walk(
            self,
            None,
            &mut Vec::new(),
            &mut BTreeSet::new(),
            &mut remaining,
            &mut |token, _source| {
                if !output.insert(token) {
                    return Err(invalid(
                        SemanticId::NIL,
                        "one local source token has two construction roles",
                    ));
                }
                Ok(())
            },
        )?;
        Ok(output)
    }
}
fn location(model: &crate::SpecializedModel, source: &Source) -> Result<SourceLocation> {
    let field = source
        .coordinate
        .as_ref()
        .and_then(|_| model.source_occurrences().get(&source.field));
    let span = field.map(|field| field.source).or_else(|| {
        model
            .source_declarations
            .get(&source.field.declaration)
            .copied()
    });
    let original = source.coordinate.is_some();
    Ok(SourceLocation {
        source: source.field.declaration.as_id(),
        revision: None,
        document: span.map(|span| span.document_id),
        occurrence_role: original.then(|| source.field.role.clone()),
        occurrence_position: if original {
            Some(u32::try_from(source.field.position).map_err(|_| {
                invalid(
                    source.field.declaration,
                    "source field ordinal exceeds diagnostic representation",
                )
            })?)
        } else {
            None
        },
        path: format!("declaration/{}", source.field.declaration),
        name: None,
        // Deliberately retain the actual whole field/declaration range. No assumption
        // about relative AST byte offsets is used to claim subexpression precision.
        start: span.map(|span| span.start),
        end: span.map(|span| span.end),
    })
}
pub(super) fn collect(
    model: &crate::SpecializedModel,
    syntax: &ResolvedSyntax,
    attribution: &RegionAttribution,
) -> Result<BTreeMap<SemanticId, SourceLocation>> {
    let mut output = BTreeMap::new();
    let mut remaining = if model.construction_limit == 0 {
        crate::specialize::Limits::default().items
    } else {
        model.construction_limit
    };
    walk(
        syntax,
        Some(attribution),
        &mut Vec::new(),
        &mut BTreeSet::new(),
        &mut remaining,
        &mut |token, source| {
            let source = source.ok_or_else(|| {
                invalid(
                    SemanticId::NIL,
                    "reached occurrence has no actual original source correspondence",
                )
            })?;
            let value = location(model, source)?;
            if output.insert(token, value).is_some() {
                return Err(invalid(
                    source.field.declaration,
                    "one local occurrence has two original source owners",
                ));
            }
            Ok(())
        },
    )?;
    Ok(output)
}
fn walk(
    syntax: &ResolvedSyntax,
    attribution: Option<&RegionAttribution>,
    path: &mut Vec<usize>,
    active: &mut BTreeSet<ContentHash>,
    remaining: &mut usize,
    visit: &mut impl FnMut(SemanticId, Option<&Source>) -> Result<()>,
) -> Result<()> {
    if path.len() >= 128 {
        return Err(crate::ModelingError::Budget(
            "source call-path depth exceeded".into(),
        ));
    }
    let layout = syntax.canonical_layout()?;
    for (predicate, values) in [(false, layout.nodes.len()), (true, layout.predicates.len())] {
        for ordinal in 0..values {
            *remaining = remaining.checked_sub(1).ok_or_else(|| {
                crate::ModelingError::Budget(
                    "source call-path correspondence allowance exceeded".into(),
                )
            })?;
            let source = if predicate {
                let slot = layout.predicates[ordinal];
                attribution
                    .and_then(|map| map.predicates.get(&slot))
                    .or_else(|| syntax.predicate_sources.get(&slot))
            } else {
                let slot = layout.nodes[ordinal];
                attribution
                    .and_then(|map| map.nodes.get(&slot))
                    .or_else(|| syntax.node_sources.get(&slot))
            };
            visit(occurrence_token(path, predicate, ordinal), source)?;
        }
    }
    for (ordinal, slot) in layout.nodes.iter().enumerate() {
        if let Node::Apply { function, .. } | Node::Partial { function, .. } = &syntax.nodes[*slot]
        {
            let product = &syntax.functions[*function].product;
            if !active.insert(product.identity) {
                return Err(invalid(
                    product.declaration,
                    "selected source function call-path is cyclic",
                ));
            }
            path.push(ordinal);
            walk(&product.syntax, None, path, active, remaining, visit)?;
            path.pop();
            active.remove(&product.identity);
        }
    }
    Ok(())
}
/// Re-select original DTOs by the selector's exact typed coordinate relocation.
/// Root and nested call-path ordinals are mapped explicitly; no source text or span equality runs.
pub(super) fn reselect(
    parent: &ResolvedSyntax,
    selected: &ResolvedSyntax,
    original: &BTreeMap<SemanticId, SourceLocation>,
    relocation: &selection::Relocation,
) -> Result<BTreeMap<SemanticId, SourceLocation>> {
    let mut output = BTreeMap::new();
    let mut remaining = original.len();
    reselect_walk(
        parent,
        selected,
        Some(relocation),
        original,
        (&mut Vec::new(), &mut Vec::new()),
        &mut remaining,
        &mut output,
    )?;
    Ok(output)
}
fn reselect_walk(
    parent: &ResolvedSyntax,
    selected: &ResolvedSyntax,
    relocation: Option<&selection::Relocation>,
    original: &BTreeMap<SemanticId, SourceLocation>,
    paths: (&mut Vec<usize>, &mut Vec<usize>),
    remaining: &mut usize,
    output: &mut BTreeMap<SemanticId, SourceLocation>,
) -> Result<()> {
    let (old_path, new_path) = paths;
    if new_path.len() >= 128 {
        return Err(crate::ModelingError::Budget(
            "term source call-path depth exceeded".into(),
        ));
    }
    let old = parent.canonical_layout()?;
    let new = selected.canonical_layout()?;
    let inverse_nodes = relocation.map(|map| {
        map.nodes
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>()
    });
    let inverse_predicates = relocation.map(|map| {
        map.predicates
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>()
    });
    let old_nodes = old
        .nodes
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, slot)| (slot, ordinal))
        .collect::<BTreeMap<_, _>>();
    let old_predicates = old
        .predicates
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, slot)| (slot, ordinal))
        .collect::<BTreeMap<_, _>>();
    for (predicate, count) in [(false, new.nodes.len()), (true, new.predicates.len())] {
        for ordinal in 0..count {
            *remaining = remaining.checked_sub(1).ok_or_else(|| {
                crate::ModelingError::Budget(
                    "term source correspondence exceeds actual parent extent".into(),
                )
            })?;
            let previous = if predicate {
                let slot = new.predicates[ordinal];
                let old = inverse_predicates
                    .as_ref()
                    .map_or(Some(slot), |map| map.get(&slot).copied())
                    .ok_or_else(|| invalid(SemanticId::NIL, "term guard left exact relocation"))?;
                *old_predicates.get(&old).ok_or_else(|| {
                    invalid(SemanticId::NIL, "term guard left original canonical walk")
                })?
            } else {
                let slot = new.nodes[ordinal];
                let old = inverse_nodes
                    .as_ref()
                    .map_or(Some(slot), |map| map.get(&slot).copied())
                    .ok_or_else(|| {
                        invalid(SemanticId::NIL, "term operation left exact relocation")
                    })?;
                *old_nodes.get(&old).ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "term operation left original canonical walk",
                    )
                })?
            };
            let value = original
                .get(&occurrence_token(old_path, predicate, previous))
                .ok_or_else(|| {
                    invalid(
                        SemanticId::NIL,
                        "term occurrence has no actual original DTO",
                    )
                })?;
            output.insert(
                occurrence_token(new_path, predicate, ordinal),
                value.clone(),
            );
        }
    }
    for (ordinal, slot) in new.nodes.iter().enumerate() {
        if let Node::Apply { function, .. } | Node::Partial { function, .. } =
            &selected.nodes[*slot]
        {
            let old_slot = inverse_nodes
                .as_ref()
                .map_or(Some(*slot), |map| map.get(slot).copied())
                .ok_or_else(|| invalid(SemanticId::NIL, "term call left exact relocation"))?;
            let old_function = match parent.nodes.get(old_slot) {
                Some(Node::Apply { function, .. } | Node::Partial { function, .. }) => *function,
                _ => {
                    return Err(invalid(
                        SemanticId::NIL,
                        "term call changed its original typed role",
                    ));
                }
            };
            let product = &selected.functions[*function].product;
            let old_product = &parent.functions[old_function].product;
            if !Arc::ptr_eq(product, old_product) {
                return Err(invalid(
                    product.declaration,
                    "term selector changed its actual immutable function owner",
                ));
            }
            old_path.push(*old_nodes.get(&old_slot).ok_or_else(|| {
                invalid(
                    product.declaration,
                    "term call absent from original canonical correspondence",
                )
            })?);
            new_path.push(ordinal);
            reselect_walk(
                &old_product.syntax,
                &product.syntax,
                None,
                original,
                (old_path, new_path),
                remaining,
                output,
            )?;
            old_path.pop();
            new_path.pop();
        }
    }
    Ok(())
}
