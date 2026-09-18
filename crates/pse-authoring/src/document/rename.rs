// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Full rename binds actual sources, stages coherent bytes/rows, and retains exact evidence.

mod native;
mod owned;
pub use owned::rename_documents;

use super::{DocumentBundle, DocumentEdit, binding, load::contract, value::Value};
use crate::{AuthoringError, SourceSpan};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::model::{ExtensionUse, FieldContract};
use std::collections::BTreeMap;

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
fn entity(
    batches: &super::Batches,
    id: SemanticId,
) -> Result<authored::entities::Row, AuthoringError> {
    let batch = batches
        .get(&authored::entities::RELATION_ID)
        .ok_or_else(|| contract(None, "rename entity inventory absent"))?;
    let mut matches = authored::entities::View::from_checked(batch)?
        .rows()?
        .into_iter()
        .filter(|row| row.entity_id == id);
    let row = matches
        .next()
        .ok_or_else(|| contract(None, "rename identity absent"))?;
    if matches.next().is_some() {
        return Err(contract(None, "rename identity duplicated"));
    }
    Ok(row)
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

struct TargetResolution<'a> {
    batches: &'a super::Batches,
    session: &'a pse_catalog::session::SnapshotSession,
    work: &'a mut dyn pse_ids::Reservation,
    completed: &'a mut crate::native_relations::plans::Completions,
    cancel: &'a pse_ids::CancellationToken,
}

async fn target_replacements(
    bundles: &[DocumentBundle],
    id: SemanticId,
    name: &str,
    replacements: &mut BTreeMap<SemanticId, Vec<(SourceSpan, String)>>,
    execution: &mut TargetResolution<'_>,
) -> Result<(), AuthoringError> {
    let TargetResolution {
        batches,
        session,
        work,
        completed,
        cancel,
    } = execution;
    let registry = session.registry();
    let entities = batches
        .get(&authored::entities::RELATION_ID)
        .map(|batch| authored::entities::View::from_checked(batch)?.rows())
        .transpose()?
        .unwrap_or_default();
    for document in bundles.iter().flat_map(|bundle| &bundle.documents) {
        for section in &document.declaration.sections {
            let relation = registry
                .relation(section.relation)
                .ok_or_else(|| contract(None, "missing document relation"))?;
            let columns = relation
                .columns
                .iter()
                .filter(|column| {
                    column.value_type() == FieldContract::extended(ExtensionUse::TargetPath)
                })
                .collect::<Vec<_>>();
            if columns.is_empty() {
                continue;
            }
            let Some(Value::List(rows)) = document.value.get(section.key) else {
                continue;
            };
            for (ordinal, row) in rows.iter().enumerate() {
                for column in &columns {
                    let Some(text) = row.value.get(column.name()).and_then(Value::text) else {
                        continue;
                    };
                    let span = document
                        .spans
                        .span(&format!("/{}/{ordinal}/{}", section.key, column.name()))
                        .ok_or_else(|| contract(None, "missing original target field span"))?;
                    let mut path = crate::targets::parse(text, span)?;
                    let bound = crate::targets::resolve_native(
                        &path,
                        batches,
                        session,
                        SemanticId::NIL,
                        *work,
                        completed,
                        cancel,
                    )
                    .await?;
                    if rewrite_target(&mut path, &bound, &entities, id, name) {
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
    entities: &[authored::entities::Row],
    id: SemanticId,
    name: &str,
) -> bool {
    let Some(target) = bound.first() else {
        return false;
    };
    let mut changed = false;
    let prefix = path.names.len() - usize::from(!path.instance_wildcard);
    let mut current = entities
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
        current = entity
            .parent_entity_id
            .and_then(|id| entities.iter().find(|entity| entity.entity_id == id));
    }
    let declaration = target
        .member
        .symbol
        .as_ref()
        .map(|value| value.symbol_decl_id)
        .or_else(|| {
            target
                .member
                .group
                .as_ref()
                .map(|value| value.symbol_decl_id)
        })
        .or_else(|| {
            target
                .member
                .equation
                .as_ref()
                .map(|value| value.equation_decl_id)
        });
    if declaration == Some(id)
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
            || before.row_key != after.row_key
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
