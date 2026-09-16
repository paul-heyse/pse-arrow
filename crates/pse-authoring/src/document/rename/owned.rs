// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source edits retain the newly parsed construction; no Cell proof snapshot or replay.
use super::{
    binding, compose_edits, contract, edits, entity, mark_rename, name_replacement,
    preserve_expression_bindings, target_replacements, valid_name, yaml_scalar,
};
use crate::{
    AuthoringError, ParseBudget,
    change_set::{AuthoredReader, ChangeSet, OwnedChangeSet},
    document::{DocumentEdit, OwnedDocumentSet},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{authored, enums::IdPolicy};
use std::collections::{BTreeMap, BTreeSet};

/// Construct one explicit-ID rename from exact original sources and native changes.
/// The changed documents are parsed once; their resulting bindings/rows are retained.
/// # Errors
/// Named identity policy, stale original text, shadowing/rebinding, bad edits or resources.
pub async fn rename_owned(
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    entity_id: SemanticId,
    new_name: &str,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<OwnedChangeSet, AuthoringError> {
    let registry = session.registry();
    bundles.validate_registry(registry)?;
    valid_name(new_name)?;
    original(bundles, base)?;
    let mut work = session.reserver().open("authoring:rename");
    work.try_grow(crate::work::sources(bundles.bundles())?)?;
    let before = crate::change_set::base::checked(base, session)?;
    let checked_base = crate::change_set::base::CheckedBase {
        base,
        rows: &before,
    };
    let (source, bindings, mut completed) =
        crate::p1::project_sources(bundles, session, cancel, work.as_mut()).await?;
    super::native::correspondence(&source, &before, session, cancel, &mut completed).await?;
    let declaration = entity(&before, entity_id)?;
    let package = bundles
        .bundles()
        .iter()
        .find(|bundle| bundle.package.package_id == declaration.package_id)
        .ok_or_else(|| contract(None, "rename package absent"))?;
    if package.package.id_policy == IdPolicy::Named {
        return Err(AuthoringError::RenameNamed {
            entity_id,
            qualified_name: declaration.qualified_name,
        });
    }
    if declaration.name == new_name {
        return Ok(OwnedChangeSet::new(ChangeSet::new(header), work));
    }
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
        entity_id,
        new_name,
        &mut replacements,
        &mut super::TargetResolution {
            batches: &before,
            session,
            work: work.as_mut(),
            completed: &mut completed,
            cancel,
        },
    )
    .await?;
    let edits = edits(bundles.bundles(), replacements)?;
    let updated = bundles.edit(
        &edits,
        registry,
        ParseBudget::default(),
        session.reserver(),
        cancel,
    )?;
    let staged = crate::p1::stage_owned(&updated, &checked_base, header, session, cancel).await?;
    let source = staged
        .source()
        .ok_or_else(|| contract(None, "rename source construction absent"))?;
    let after = source.candidate.clone();
    super::native::preserved(&before, &after, false, session, cancel, &mut completed).await?;
    preserve_expression_bindings(bindings.expressions(), source.bindings.expressions())?;
    completed.extend(bindings.completions().iter().cloned());
    let retained = crate::change_set::SourceConstruction {
        documents: updated,
        candidate: source.candidate.clone(),
        bindings: source.bindings.clone(),
        edits,
        renamed: BTreeSet::from([entity_id]),
        completed: completed
            .into_iter()
            .chain(staged.completions().cloned())
            .collect(),
    };
    let mut changes = (*staged).clone();
    mark_rename(&mut changes, entity_id, registry)?;
    changes.attach_source(retained);
    Ok(OwnedChangeSet::new(changes, work))
}

/// Compose edits relative to the retained renamed source, parsing only new bytes.
/// # Errors
/// Stale input, absent source construction, changed entity identity/name/binding or resources.
pub async fn amend_rename_sources_owned(
    changes: &ChangeSet,
    bundles: &OwnedDocumentSet,
    base: &dyn AuthoredReader,
    additional: &[DocumentEdit],
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<OwnedChangeSet, AuthoringError> {
    let registry = session.registry();
    bundles.validate_registry(registry)?;
    original(bundles, base)?;
    if base.revision_id() != changes.header.base_revision_id {
        return Err(contract(None, "rename base changed"));
    }
    let previous = changes
        .source()
        .filter(|source| !source.renamed.is_empty())
        .ok_or_else(|| {
            contract(
                None,
                "source composition requires retained rename construction",
            )
        })?;
    let mut work = session.reserver().open("authoring:rename-amend");
    work.try_grow(crate::work::sources(previous.documents.bundles())?)?;
    let edits = compose_edits(
        bundles.bundles(),
        previous.documents.bundles(),
        &previous.edits,
        additional,
    )?;
    let updated = previous.documents.edit(
        additional,
        registry,
        ParseBudget::default(),
        session.reserver(),
        cancel,
    )?;
    let staged =
        crate::p1::stage_owned(&updated, base, changes.header.clone(), session, cancel).await?;
    let source = staged
        .source()
        .ok_or_else(|| contract(None, "amended source construction absent"))?;
    let mut completed = Vec::new();
    let prior = previous.candidate.clone();
    let after = source.candidate.clone();
    super::native::preserved(&prior, &after, true, session, cancel, &mut completed).await?;
    preserve_expression_bindings(
        previous.bindings.expressions(),
        source.bindings.expressions(),
    )?;
    let retained = crate::change_set::SourceConstruction {
        documents: updated,
        candidate: source.candidate.clone(),
        bindings: source.bindings.clone(),
        edits,
        renamed: previous.renamed.clone(),
        completed: completed
            .into_iter()
            .chain(changes.completions().cloned())
            .chain(staged.completions().cloned())
            .collect(),
    };
    let mut amended = (*staged).clone();
    for id in &previous.renamed {
        mark_rename(&mut amended, *id, registry)?;
    }
    amended.attach_source(retained);
    Ok(OwnedChangeSet::new(amended, work))
}
fn original(bundles: &OwnedDocumentSet, base: &dyn AuthoredReader) -> Result<(), AuthoringError> {
    let sources = bundles
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .map(|document| (document.id, document.text.clone()))
        .collect::<BTreeMap<_, _>>();
    if sources != base.source_documents()? {
        return Err(contract(
            None,
            "rename requires exact complete original source bytes",
        ));
    }
    Ok(())
}
