// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned orchestration reuses the exact source/edit/identity checks of the DTO API.
use super::{
    binding, compose_edits, contract, edits, entity, mark_rename, name_replacement,
    preserve_entity_names, preserve_expression_bindings, preserve_targets, target_replacements,
    valid_name, yaml_scalar,
};
use crate::{
    AuthoringError, ParseBudget,
    change_set::{AuthoredReader, ChangeSet},
    document::{DocumentEdit, Rows},
};
use crate::{change_set::OwnedChangeSet, document::OwnedDocumentSet};
use pse_ids::SemanticId;
use pse_ids::{CancellationToken, MemoryReserver, Reservation};
use pse_relations::generated::{
    authored,
    enums::{ChangeOpKind, IdPolicy},
};
use pse_schema::Registry;
use std::collections::BTreeMap;

type Resources<'a> = (&'a dyn MemoryReserver, &'a CancellationToken);

/// Stage a complete rename with retained parser, staging and proof reservations.
/// # Errors
/// All actual identity/source failures from `rename`, cancellation or memory refusal.
pub fn rename_owned(
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    entity_id: SemanticId,
    new_name: &str,
    registry: &Registry,
    resources: Resources<'_>,
) -> Result<OwnedChangeSet, AuthoringError> {
    let (reserver, cancel) = resources;
    bundles.validate_registry(registry)?;
    valid_name(new_name)?;
    let mut work = reserve(bundles, base, registry, resources)?;
    let source_bytes = bundles
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .try_fold(0, |n, document| crate::work::add(n, document.text.len()))?;
    work.try_grow(crate::work::mul(
        crate::work::mul(source_bytes, new_name.len())?,
        8,
    )?)?;
    original(bundles, base, header.clone(), registry, resources)?;
    let before = crate::change_set::proof::rows(base, registry)?;
    let entity = entity(&before, entity_id)?;
    let package = bundles
        .bundles()
        .iter()
        .find(|bundle| bundle.package.package_id == entity.package_id)
        .ok_or_else(|| contract(None, "rename package absent from complete inventory"))?;
    if package.package.id_policy == IdPolicy::Named {
        return Err(AuthoringError::RenameNamed {
            entity_id,
            qualified_name: entity.qualified_name,
        });
    }
    if entity.name == new_name {
        return Ok(OwnedChangeSet::new(ChangeSet::new(header), work));
    }
    let bindings = binding::bind_sources_owned(bundles, &before, registry, reserver, cancel)?;
    let context = crate::targets::TargetContext::from_rows(&before)?;
    let mut replacements = BTreeMap::new();
    name_replacement(bundles.bundles(), entity_id, new_name, &mut replacements)?;
    for expression in bindings.expressions() {
        if let Some(text) = binding::rename_expression(expression, entity_id, new_name)? {
            replacements
                .entry(expression.document_id)
                .or_default()
                .push((expression.source_span, yaml_scalar(&text)?));
        }
    }
    target_replacements(
        bundles.bundles(),
        &context,
        entity_id,
        new_name,
        registry,
        &mut replacements,
    )?;
    let edits = edits(bundles.bundles(), replacements)?;
    let updated = update(bundles, &edits, registry, resources)?;
    let staged = crate::p1::stage_owned(&updated, base, header, registry, reserver, cancel)?;
    let candidate = crate::change_set::apply_owned(base, &staged, registry, reserver, cancel)?;
    let after = decode_candidate(&candidate, registry, work.as_mut())?;
    preserve_targets(&before, &after)?;
    let rebound = binding::bind_sources_owned(&updated, &after, registry, reserver, cancel)?;
    preserve_expression_bindings(bindings.expressions(), rebound.expressions())?;
    reserve_changes(&staged, work.as_mut())?;
    let mut changes = (*staged).clone();
    mark_rename(&mut changes, entity_id, registry)?;
    crate::change_set::proof::capture(&mut changes, base, registry, edits)?;
    cancel.checkpoint()?;
    Ok(OwnedChangeSet::new(changes, work))
}

/// Compose explicit source edits with a complete rename while retaining all work.
/// # Errors
/// Stale actual preimages, identity/reference changes, parsing, cancellation or memory refusal.
pub fn amend_rename_sources_owned(
    changes: &ChangeSet,
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    additional: &[DocumentEdit],
    registry: &Registry,
    resources: Resources<'_>,
) -> Result<OwnedChangeSet, AuthoringError> {
    let (reserver, cancel) = resources;
    bundles.validate_registry(registry)?;
    let mut work = reserve(bundles, base, registry, resources)?;
    work.try_grow(crate::change_set::proof::extent(changes, base, registry)?)?;
    let edit_extent = additional.iter().try_fold(0, |n, edit| {
        crate::work::add(n, crate::work::add(edit.before.len(), edit.after.len())?)
    })?;
    work.try_grow(crate::work::mul(edit_extent, 16)?)?;
    if !crate::change_set::proof::verify(changes, base, registry)? {
        return Err(contract(
            None,
            "source composition requires verified rename",
        ));
    }
    original(bundles, base, changes.header.clone(), registry, resources)?;
    let previous = update(bundles, changes.document_edits(), registry, resources)?;
    let edits = compose_edits(
        bundles.bundles(),
        previous.bundles(),
        changes.document_edits(),
        additional,
    )?;
    let updated = update(bundles, &edits, registry, resources)?;
    let staged = crate::p1::stage_owned(
        &updated,
        base,
        changes.header.clone(),
        registry,
        reserver,
        cancel,
    )?;
    let candidate = crate::change_set::apply_owned(base, &staged, registry, reserver, cancel)?;
    let before = crate::change_set::proof::rows(base, registry)?;
    let after = decode_candidate(&candidate, registry, work.as_mut())?;
    preserve_targets(&before, &after)?;
    let old_binding = binding::bind_sources_owned(bundles, &before, registry, reserver, cancel)?;
    let new_binding = binding::bind_sources_owned(&updated, &after, registry, reserver, cancel)?;
    preserve_expression_bindings(old_binding.expressions(), new_binding.expressions())?;
    let prior = crate::change_set::apply_owned(base, changes, registry, reserver, cancel)?;
    preserve_entity_names(&decode_candidate(&prior, registry, work.as_mut())?, &after)?;
    reserve_changes(&staged, work.as_mut())?;
    let mut amended = (*staged).clone();
    mark_previous(changes, &mut amended, registry)?;
    crate::change_set::proof::capture(&mut amended, base, registry, edits)?;
    cancel.checkpoint()?;
    Ok(OwnedChangeSet::new(amended, work))
}
fn original(
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    registry: &Registry,
    resources: Resources<'_>,
) -> Result<(), AuthoringError> {
    let originals = bundles
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    if originals != base.source_documents()?
        || !crate::p1::stage_owned(bundles, base, header, registry, resources.0, resources.1)?
            .ops
            .is_empty()
    {
        return Err(contract(
            None,
            "rename requires complete actual original sources and base rows",
        ));
    }
    Ok(())
}
fn reserve(
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    registry: &Registry,
    resources: Resources<'_>,
) -> Result<Box<dyn Reservation>, AuthoringError> {
    resources.1.checkpoint()?;
    let mut work = resources.0.open("authoring:rename");
    work.try_grow(crate::work::add(
        crate::work::sources(bundles.bundles())?,
        crate::work::mul(registry.relations().len(), 4096)?,
    )?)?;
    for (_, batch) in base.relations()? {
        work.try_grow(crate::work::mul(pse_ids::validation_extent(&batch)?, 4)?)?;
    }
    for (_, rows) in registry.schema_rows_ref() {
        for row in rows {
            for value in row {
                work.try_grow(crate::work::mul(crate::work::cell(value)?, 4)?)?;
            }
        }
    }
    Ok(work)
}
fn update(
    bundles: &OwnedDocumentSet,
    edits: &[DocumentEdit],
    registry: &Registry,
    resources: Resources<'_>,
) -> Result<OwnedDocumentSet, AuthoringError> {
    let mut work = resources.0.open("authoring:rename-source-edits");
    work.try_grow(crate::work::sources(bundles.bundles())?)?;
    for edit in edits {
        work.try_grow(crate::work::mul(
            crate::work::add(edit.before.len(), edit.after.len())?,
            8,
        )?)?;
    }
    let mut parts = Vec::new();
    for bundle in bundles.bundles() {
        resources.1.checkpoint()?;
        let mut texts = bundle
            .documents
            .iter()
            .map(|document| (document.path.clone(), document.text.clone()))
            .collect();
        let own = edits
            .iter()
            .filter(|edit| {
                bundle
                    .documents
                    .iter()
                    .any(|document| document.id == edit.document_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        crate::document::apply_edits(&mut texts, &own)?;
        parts.push(crate::document::load_package_texts_owned(
            &texts,
            registry,
            ParseBudget::default(),
            resources.0,
            resources.1,
        )?);
    }
    OwnedDocumentSet::try_from_bundles(parts, resources.0, resources.1)
}
fn decode_candidate(
    candidate: &crate::change_set::CandidateSnapshot,
    registry: &Registry,
    work: &mut dyn Reservation,
) -> Result<Rows, AuthoringError> {
    for batch in candidate.relations.values() {
        work.try_grow(pse_ids::validation_extent(batch)?)?;
    }
    candidate
        .relations
        .iter()
        .map(|(id, batch)| {
            let spec = registry
                .relation_by_id(*id)
                .ok_or_else(|| contract(None, "candidate declaration absent"))?;
            Ok((
                *id,
                pse_relations::cells::cells_from_batch(registry, spec, batch)?,
            ))
        })
        .collect()
}
fn reserve_changes(changes: &ChangeSet, work: &mut dyn Reservation) -> Result<(), AuthoringError> {
    work.try_grow(crate::work::mul(changes.ops.len(), 8192)?)?;
    for member in changes.staged.values() {
        work.try_grow(pse_ids::validation_extent(&member.batch)?)?;
    }
    Ok(())
}
fn mark_previous(
    changes: &ChangeSet,
    amended: &mut ChangeSet,
    registry: &Registry,
) -> Result<(), AuthoringError> {
    let entities = registry
        .relation("authored.entities")
        .ok_or_else(|| contract(None, "entity declaration absent"))?;
    for operation in changes
        .ops
        .iter()
        .filter(|operation| operation.op == ChangeOpKind::Rename)
    {
        if operation.relation_id != entities.id {
            return Err(contract(None, "rename has a non-entity operation"));
        }
        let staged = changes
            .staged
            .get(&operation.row_key.staged_port)
            .ok_or_else(|| contract(None, "rename preimage absent"))?;
        let rows = pse_relations::cells::cells_from_batch(registry, entities, &staged.batch)?;
        let [row] = rows.as_slice() else {
            return Err(contract(None, "rename preimage is not one entity"));
        };
        let entity = authored::entities::Row::from_cells(row.clone())?;
        mark_rename(amended, entity.entity_id, registry)?;
    }
    Ok(())
}
