// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Full rename binds actual sources, stages coherent bytes/rows, and retains exact evidence.

mod owned;
pub use owned::{amend_rename_sources_owned, rename_owned};

use super::{DocumentBundle, DocumentEdit, Rows, binding, load::contract, value::Value};
use crate::{
    AuthoringError, ParseBudget, SourceSpan,
    change_set::{AuthoredReader, ChangeSet},
};
use pse_ids::SemanticId;
use pse_relations::generated::{
    authored,
    enums::{ChangeOpKind, IdPolicy},
};
use pse_schema::{
    Registry,
    model::{Cell, ExtensionUse, LogicalType},
};
use std::collections::BTreeMap;

/// Stage an explicit-policy entity rename with all retained expression/target references,
/// original document replacements, regenerated spans and complete exact-input evidence.
///
/// # Errors
/// Refuses incomplete/stale source inventories, named-policy renames, invalid new names,
/// unresolved/ambiguous bindings, changed reference identities and invalid post-edit rows.
pub fn rename(
    bundles: &[DocumentBundle],
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    entity_id: SemanticId,
    new_name: &str,
    registry: &Registry,
) -> Result<ChangeSet, AuthoringError> {
    valid_name(new_name)?;
    let originals = bundles
        .iter()
        .flat_map(|bundle| bundle.documents.iter())
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    if originals != base.source_documents()? {
        return Err(contract(
            None,
            "rename requires the complete exact original source-byte inventory",
        ));
    }
    if !crate::p1::stage(bundles, base, header.clone(), registry)?
        .ops
        .is_empty()
    {
        return Err(contract(
            None,
            "original documents do not reproduce the exact complete authored base",
        ));
    }
    let rows = crate::change_set::proof::rows(base, registry)?;
    let entity = entity(&rows, entity_id)?;
    let package = bundles
        .iter()
        .find(|bundle| bundle.package.package_id == entity.package_id)
        .ok_or_else(|| {
            contract(
                None,
                "rename package is absent from complete source inventory",
            )
        })?;
    if package.package.id_policy == IdPolicy::Named {
        return Err(AuthoringError::RenameNamed {
            entity_id,
            qualified_name: entity.qualified_name,
        });
    }
    if entity.name == new_name {
        return Ok(ChangeSet::new(header));
    }
    let bindings = binding::bind_sources(bundles, &rows, registry)?;
    let context = crate::targets::TargetContext::from_rows(&rows)?;
    let mut replacements = BTreeMap::<SemanticId, Vec<(SourceSpan, String)>>::new();
    name_replacement(bundles, entity_id, new_name, &mut replacements)?;
    for expression in bindings.expressions() {
        if let Some(text) = binding::rename_expression(expression, entity_id, new_name)? {
            replacements
                .entry(expression.document_id)
                .or_default()
                .push((expression.source_span, yaml_scalar(&text)?));
        }
    }
    target_replacements(
        bundles,
        &context,
        entity_id,
        new_name,
        registry,
        &mut replacements,
    )?;
    let edits = edits(bundles, replacements)?;
    let updated = updated_bundles(bundles, &edits, registry)?;
    let mut changes = crate::p1::stage(&updated, base, header, registry)?;
    let candidate = crate::change_set::apply(base, &changes, registry)?;
    let candidate_rows = candidate
        .relations
        .iter()
        .map(|(id, batch)| {
            let relation = registry
                .relations()
                .iter()
                .find(|relation| relation.id == *id)
                .ok_or_else(|| contract(None, "unregistered candidate relation"))?;
            Ok((
                *id,
                pse_relations::cells::cells_from_batch(registry, relation, batch)
                    .map_err(|error| contract(None, &error.to_string()))?,
            ))
        })
        .collect::<Result<Rows, AuthoringError>>()?;
    preserve_targets(&rows, &candidate_rows)?;
    let after = binding::bind_sources(&updated, &candidate_rows, registry)?;
    preserve_bindings(&bindings, &after)?;
    mark_rename(&mut changes, entity_id, registry)?;
    crate::change_set::proof::capture(&mut changes, base, registry, edits)?;
    Ok(changes)
}

/// Compose explicit additional source edits with a previously verified complete rename.
/// Each extra preimage is the already renamed document text. The final sources are
/// reparsed and restaged against the original base; identities and reference bindings
/// remain fixed. An explicit case revision update can therefore share the atomic commit.
/// # Errors
/// A stale rename/base, missing or duplicate source, wrong additional preimage, changed
/// entity naming or reference binding, or any final parse/staging error is refused.
pub fn amend_rename_sources(
    changes: &ChangeSet,
    bundles: &[DocumentBundle],
    base: &dyn AuthoredReader,
    additional: &[DocumentEdit],
    registry: &Registry,
) -> Result<ChangeSet, AuthoringError> {
    if !crate::change_set::proof::verify(changes, base, registry)? {
        return Err(contract(
            None,
            "source composition requires a verified rename",
        ));
    }
    let originals = bundles
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    if originals != base.source_documents()?
        || !crate::p1::stage(bundles, base, changes.header.clone(), registry)?
            .ops
            .is_empty()
    {
        return Err(contract(
            None,
            "composition requires the complete exact original source inventory",
        ));
    }
    let previous = updated_bundles(bundles, changes.document_edits(), registry)?;
    let edits = compose_edits(bundles, &previous, changes.document_edits(), additional)?;
    let updated = updated_bundles(bundles, &edits, registry)?;
    let mut amended = crate::p1::stage(&updated, base, changes.header.clone(), registry)?;
    let before = crate::change_set::proof::rows(base, registry)?;
    let after = candidate_rows(base, &amended, registry)?;
    preserve_targets(&before, &after)?;
    preserve_bindings(
        &binding::bind_sources(bundles, &before, registry)?,
        &binding::bind_sources(&updated, &after, registry)?,
    )?;
    preserve_entity_names(&candidate_rows(base, changes, registry)?, &after)?;
    let entities = registry
        .relation("authored.entities")
        .ok_or_else(|| contract(None, "entity relation missing"))?;
    for operation in changes
        .ops
        .iter()
        .filter(|operation| operation.op == ChangeOpKind::Rename)
    {
        if operation.relation_id != entities.id {
            return Err(contract(None, "rename proof has a non-entity rename"));
        }
        let staged = changes
            .staged
            .get(&operation.row_key.staged_port)
            .ok_or_else(|| contract(None, "rename preimage absent"))?;
        let rows = pse_relations::cells::cells_from_batch(registry, entities, &staged.batch)
            .map_err(|error| contract(None, &error.to_string()))?;
        let [row] = rows.as_slice() else {
            return Err(contract(None, "rename preimage is not one entity"));
        };
        let entity = authored::entities::Row::from_cells(row.clone())
            .map_err(|error| contract(None, &error.to_string()))?;
        mark_rename(&mut amended, entity.entity_id, registry)?;
    }
    crate::change_set::proof::capture(&mut amended, base, registry, edits)?;
    Ok(amended)
}

fn candidate_rows(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    registry: &Registry,
) -> Result<Rows, AuthoringError> {
    crate::change_set::apply(base, changes, registry)?
        .relations
        .iter()
        .map(|(id, batch)| {
            let spec = registry
                .relation_by_id(*id)
                .ok_or_else(|| contract(None, "candidate declaration absent"))?;
            Ok((
                *id,
                pse_relations::cells::cells_from_batch(registry, spec, batch)
                    .map_err(|error| contract(None, &error.to_string()))?,
            ))
        })
        .collect()
}
fn compose_edits(
    originals: &[DocumentBundle],
    previous: &[DocumentBundle],
    current: &[DocumentEdit],
    additional: &[DocumentEdit],
) -> Result<Vec<DocumentEdit>, AuthoringError> {
    let mut edits = current
        .iter()
        .cloned()
        .map(|edit| (edit.document_id, edit))
        .collect::<BTreeMap<_, _>>();
    let mut seen = std::collections::BTreeSet::new();
    for edit in additional {
        if !seen.insert(edit.document_id) {
            return Err(contract(None, "duplicate additional source edit"));
        }
        let document = previous
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .find(|document| document.id == edit.document_id)
            .ok_or_else(|| {
                contract(
                    None,
                    "additional source edit is outside the rename inventory",
                )
            })?;
        if document.path != edit.path || document.text != edit.before {
            return Err(contract(
                None,
                "additional edit does not match exact renamed source bytes",
            ));
        }
        let original = originals
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .find(|document| document.id == edit.document_id)
            .ok_or_else(|| contract(None, "original source document absent"))?;
        edits.insert(
            edit.document_id,
            DocumentEdit {
                document_id: edit.document_id,
                path: edit.path.clone(),
                before: original.text.clone(),
                after: edit.after.clone(),
            },
        );
    }
    Ok(edits.into_values().collect())
}
fn preserve_entity_names(before: &Rows, after: &Rows) -> Result<(), AuthoringError> {
    let names = |rows: &Rows| -> Result<BTreeMap<SemanticId, Vec<Cell>>, AuthoringError> {
        rows.get(&authored::entities::RELATION_ID)
            .into_iter()
            .flatten()
            .cloned()
            .map(|row| {
                let mut entity = authored::entities::Row::from_cells(row)
                    .map_err(|error| contract(None, &error.to_string()))?;
                entity.source_span = None;
                Ok((entity.entity_id, entity.into_cells()))
            })
            .collect()
    };
    if names(before)? != names(after)? {
        return Err(contract(
            None,
            "additional edits changed already validated entity naming",
        ));
    }
    Ok(())
}

fn valid_name(name: &str) -> Result<(), AuthoringError> {
    let expression =
        crate::dsl::parse_expr(name).map_err(|error| contract(None, &error.to_string()))?;
    if !matches!(expression.kind, crate::dsl::ExprKind::Path(ref path) if path.segments.len() == 1 && path.segments[0].name == name && path.segments[0].indices.is_empty())
    {
        return Err(contract(
            None,
            "rename requires one representable identifier",
        ));
    }
    Ok(())
}
fn entity(rows: &Rows, id: SemanticId) -> Result<authored::entities::Row, AuthoringError> {
    let entities = rows
        .get(&authored::entities::RELATION_ID)
        .into_iter()
        .flatten()
        .cloned()
        .map(authored::entities::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| contract(None, &error.to_string()))?;
    let mut matches = entities.into_iter().filter(|entity| entity.entity_id == id);
    let entity = matches
        .next()
        .ok_or_else(|| contract(None, "rename identity is not registered"))?;
    if matches.next().is_some() {
        return Err(contract(None, "rename identity is duplicated"));
    }
    Ok(entity)
}
fn name_replacement(
    bundles: &[DocumentBundle],
    id: SemanticId,
    name: &str,
    replacements: &mut BTreeMap<SemanticId, Vec<(SourceSpan, String)>>,
) -> Result<(), AuthoringError> {
    let mut found = 0;
    for document in bundles.iter().flat_map(|bundle| &bundle.documents) {
        for section in &document.declaration.sections {
            let (Some(identity), Some(name_column)) =
                (section.identity_column, section.name_column)
            else {
                continue;
            };
            let Some(Value::List(rows)) = document.value.get(section.key) else {
                continue;
            };
            for (ordinal, row) in rows.iter().enumerate() {
                if row.value.get(identity).and_then(Value::text) == Some(id.to_string().as_str()) {
                    let span = document
                        .spans
                        .span(&format!("/{}/{ordinal}/{name_column}", section.key))
                        .ok_or_else(|| contract(None, "missing original entity name span"))?;
                    replacements
                        .entry(document.id)
                        .or_default()
                        .push((span, yaml_scalar(name)?));
                    found += 1;
                }
            }
        }
    }
    if found != 1 {
        return Err(contract(
            None,
            "rename must identify exactly one original entity declaration",
        ));
    }
    Ok(())
}
fn yaml_scalar(text: &str) -> Result<String, AuthoringError> {
    serde_saphyr::to_string(&text)
        .map(|text| text.trim_end_matches('\n').to_owned())
        .map_err(|error| contract(None, &error.to_string()))
}

fn target_replacements(
    bundles: &[DocumentBundle],
    context: &crate::targets::TargetContext,
    id: SemanticId,
    name: &str,
    registry: &Registry,
    replacements: &mut BTreeMap<SemanticId, Vec<(SourceSpan, String)>>,
) -> Result<(), AuthoringError> {
    for document in bundles.iter().flat_map(|bundle| &bundle.documents) {
        for section in &document.declaration.sections {
            let relation = registry
                .relation(section.relation)
                .ok_or_else(|| contract(None, "missing document relation"))?;
            let columns = relation
                .columns
                .iter()
                .filter(|column| column.logical_type == LogicalType::Ext(ExtensionUse::TargetPath))
                .collect::<Vec<_>>();
            if columns.is_empty() {
                continue;
            }
            let Some(Value::List(rows)) = document.value.get(section.key) else {
                continue;
            };
            for (ordinal, row) in rows.iter().enumerate() {
                for column in &columns {
                    let Some(text) = row.value.get(column.name).and_then(Value::text) else {
                        continue;
                    };
                    let span = document
                        .spans
                        .span(&format!("/{}/{ordinal}/{}", section.key, column.name))
                        .ok_or_else(|| contract(None, "missing original target field span"))?;
                    let mut path = crate::targets::parse(text, span)?;
                    let bound = crate::targets::resolve(&path, context, SemanticId::NIL)?;
                    if rewrite_target(&mut path, &bound, context, id, name) {
                        replacements
                            .entry(document.id)
                            .or_default()
                            .push((span, yaml_scalar(&render_target(&path))?));
                    }
                }
            }
        }
    }
    Ok(())
}
fn rewrite_target(
    path: &mut crate::targets::TargetPath,
    bound: &[crate::targets::TargetRow],
    context: &crate::targets::TargetContext,
    id: SemanticId,
    name: &str,
) -> bool {
    let Some(target) = bound.first() else {
        return false;
    };
    let mut changed = false;
    let prefix = path.names.len() - usize::from(!path.instance_wildcard);
    let mut current = context
        .entities
        .iter()
        .find(|entity| entity.entity_id == target.instance_id);
    for index in (0..prefix).rev() {
        let Some(entity) = current else {
            break;
        };
        if path.names[index] != entity.name {
            break;
        }
        if entity.entity_id == id {
            name.clone_into(&mut path.names[index]);
            changed = true;
        }
        current = entity.parent_entity_id.and_then(|id| {
            context
                .entities
                .iter()
                .find(|entity| entity.entity_id == id)
        });
    }
    if (target.symbol_decl_id == Some(id) || target.equation_decl_id == Some(id))
        && let Some(last) = path.names.last_mut()
    {
        name.clone_into(last);
        changed = true;
    }
    changed
}
fn render_target(path: &crate::targets::TargetPath) -> String {
    let mut text = path.names.join(".");
    if path.instance_wildcard {
        text.push_str(".*");
    }
    if !path.indices.is_empty() {
        text.push('[');
        text.push_str(
            &path
                .indices
                .iter()
                .map(|selector| match selector {
                    crate::targets::IndexSelector::Wildcard => "*".to_owned(),
                    crate::targets::IndexSelector::Value(value) => value.clone(),
                    crate::targets::IndexSelector::Label(value) => format!("'{value}'"),
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        text.push(']');
    }
    text
}

fn edits(
    bundles: &[DocumentBundle],
    replacements: BTreeMap<SemanticId, Vec<(SourceSpan, String)>>,
) -> Result<Vec<DocumentEdit>, AuthoringError> {
    replacements
        .into_iter()
        .map(|(id, mut replacements)| {
            let document = bundles
                .iter()
                .flat_map(|bundle| &bundle.documents)
                .find(|document| document.id == id)
                .ok_or_else(|| contract(None, "missing source document for edits"))?;
            replacements.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
            let mut end = document.text.len();
            let mut after = document.text.clone();
            for (span, text) in replacements {
                let start = usize::try_from(span.start).unwrap_or(usize::MAX);
                let finish = usize::try_from(span.end).unwrap_or(usize::MAX);
                if finish > end
                    || start >= finish
                    || !after.is_char_boundary(start)
                    || !after.is_char_boundary(finish)
                {
                    return Err(contract(
                        Some(span),
                        "overlapping or invalid original source ranges",
                    ));
                }
                after.replace_range(start..finish, &text);
                end = start;
            }
            Ok(DocumentEdit {
                document_id: id,
                path: document.path.clone(),
                before: document.text.clone(),
                after,
            })
        })
        .collect()
}
fn updated_bundles(
    bundles: &[DocumentBundle],
    edits: &[DocumentEdit],
    registry: &Registry,
) -> Result<Vec<DocumentBundle>, AuthoringError> {
    bundles
        .iter()
        .map(|bundle| {
            let texts = bundle
                .documents
                .iter()
                .map(|document| {
                    (
                        document.path.clone(),
                        edits
                            .iter()
                            .find(|edit| edit.document_id == document.id)
                            .map_or_else(|| document.text.clone(), |edit| edit.after.clone()),
                    )
                })
                .collect();
            super::load_package_texts(texts, registry, ParseBudget::default())
        })
        .collect()
}
fn preserve_targets(before: &Rows, after: &Rows) -> Result<(), AuthoringError> {
    for id in [
        authored::case_spec_targets::RELATION_ID,
        authored::case_activation_targets::RELATION_ID,
        authored::observation_targets::RELATION_ID,
    ] {
        let first = before.get(&id).map_or(&[][..], Vec::as_slice);
        let second = after.get(&id).map_or(&[][..], Vec::as_slice);
        if first.len() != second.len()
            || !first
                .iter()
                .zip(second)
                .all(|(first, second)| equal(first, second))
        {
            return Err(contract(
                None,
                "rename changed actual bound target identities",
            ));
        }
    }
    Ok(())
}
fn preserve_bindings(
    before: &binding::SourceBindings,
    after: &binding::SourceBindings,
) -> Result<(), AuthoringError> {
    preserve_expression_bindings(before.expressions(), after.expressions())
}
fn preserve_expression_bindings(
    before: &[binding::SourceExpression],
    after: &[binding::SourceExpression],
) -> Result<(), AuthoringError> {
    if before.len() != after.len() {
        return Err(contract(
            None,
            "rename changed the complete expression inventory",
        ));
    }
    for (before, after) in before.iter().zip(after) {
        if before.document_id != after.document_id
            || before.document_path != after.document_path
            || before.relation_id != after.relation_id
            || !equal(&before.row_key, &after.row_key)
            || before
                .paths
                .iter()
                .map(|path| &path.meaning)
                .ne(after.paths.iter().map(|path| &path.meaning))
        {
            return Err(contract(
                None,
                "rename changed an actual expression reference binding",
            ));
        }
    }
    Ok(())
}
fn equal(first: &[Cell], second: &[Cell]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|(first, second)| first.literal_spec() == second.literal_spec())
}
fn mark_rename(
    changes: &mut ChangeSet,
    id: SemanticId,
    registry: &Registry,
) -> Result<(), AuthoringError> {
    let spec = registry
        .relation("authored.entities")
        .ok_or_else(|| contract(None, "missing entity relation"))?;
    for operation in &mut changes.ops {
        if operation.relation_id != spec.id || operation.op != ChangeOpKind::Update {
            continue;
        }
        let staged = changes
            .staged
            .get(&operation.row_key.staged_port)
            .ok_or_else(|| contract(None, "missing entity preimage"))?;
        let rows = pse_relations::cells::cells_from_batch(registry, spec, &staged.batch)
            .map_err(|error| contract(None, &error.to_string()))?;
        if rows.first().and_then(|row| row.first()) == Some(&Cell::Id(id)) {
            operation.op = ChangeOpKind::Rename;
            return Ok(());
        }
    }
    Err(contract(
        None,
        "rename did not produce the expected entity operation",
    ))
}
