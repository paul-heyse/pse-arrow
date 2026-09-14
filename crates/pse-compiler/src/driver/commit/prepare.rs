// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Accounted P0/P1 source ownership before candidate validation or publication.
use super::{BaseReader, CommitRequest, CommitRevisionIds, Driver};
use crate::{
    CompilerError,
    passes::{dag::invalid, p0},
};
use pse_authoring::{
    ParseBudget,
    change_set::{ChangeSet, OwnedCandidateSnapshot},
    document::OwnedDocumentSet,
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

pub(super) struct Initial {
    pub(super) base: BaseReader,
    pub(super) ids: CommitRevisionIds,
    pub(super) parent_model: Option<SemanticId>,
    pub(super) documents: OwnedDocumentSet,
}
pub(super) struct Prepared {
    pub(super) documents: OwnedDocumentSet,
    pub(super) rows: BTreeMap<RelationKey, RecordBatch>,
    pub(super) candidate: OwnedCandidateSnapshot,
}

pub(super) fn source(
    driver: &Driver,
    request: &CommitRequest,
    base: &BaseReader,
    ids: CommitRevisionIds,
    documents: OwnedDocumentSet,
    cancel: &CancellationToken,
) -> Result<Prepared, CompilerError> {
    cancel.checkpoint()?;
    let registry = driver.registry();
    let reserver = driver.sessions.reserver().as_ref();
    let candidate = if let Some(changes) = &request.changes {
        if changes.header != request.header {
            return Err(invalid("staged header differs from commit header"));
        }
        pse_authoring::change_set::apply_owned(base, changes, registry, reserver, cancel)?
    } else {
        let changes = pse_authoring::p1::stage_owned(
            &documents,
            base,
            request.header.clone(),
            registry,
            reserver,
            cancel,
        )?;
        pse_authoring::change_set::apply_owned(base, &changes, registry, reserver, cancel)?
    };
    let mut rows = BTreeMap::new();
    for (id, batch) in &candidate.relations {
        let spec = registry
            .relation_by_id(*id)
            .ok_or_else(|| invalid("candidate relation absent"))?;
        rows.insert(spec.key, batch.clone());
    }
    for (id, batch) in &base.rows {
        let spec = registry
            .relation_by_id(*id)
            .ok_or_else(|| invalid("primitive relation absent"))?;
        rows.entry(spec.key).or_insert_with(|| batch.clone());
    }
    let documents = apply_source_edits(documents, &candidate.changes, driver, cancel)?;
    let proposed = BaseReader {
        revision: base.revision,
        rows: candidate.relations.clone(),
        documents: base.documents.clone(),
    };
    let reloaded = pse_authoring::p1::stage_owned(
        &documents,
        &proposed,
        request.header.clone(),
        registry,
        reserver,
        cancel,
    )?;
    if !reloaded.ops.is_empty() {
        return Err(invalid(
            "candidate rows differ from complete exact source reparse and target bindings",
        ));
    }
    let case_spec = registry
        .relation("authored.cases")
        .ok_or_else(|| invalid("case declaration absent"))?;
    let case_batch = rows
        .get(&case_spec.key)
        .ok_or_else(|| invalid("case inventory absent"))?;
    let mut workspace = reserver.open("compiler:case-revision-binding");
    workspace
        .try_grow(pse_ids::validation_extent(case_batch)?)
        .map_err(pse_ids::CanonError::from)?;
    for cells in pse_relations::cells::cells_from_batch(registry, case_spec, case_batch)? {
        if authored::cases::Row::from_cells(cells)?.model_revision_id != ids.model {
            return Err(invalid(
                "authored case model_revision_id differs from selected commit model revision",
            ));
        }
    }
    p0::resolve(&rows, registry, reserver, cancel)?;
    Ok(Prepared {
        documents,
        rows,
        candidate,
    })
}
fn apply_source_edits(
    documents: OwnedDocumentSet,
    changes: &ChangeSet,
    driver: &Driver,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentSet, CompilerError> {
    if changes.document_edits().is_empty() {
        return Ok(documents);
    }
    let reserver = driver.sessions.reserver().as_ref();
    let mut work = reserver.open("compiler:source-edits");
    let source_bytes = documents
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .try_fold(0_usize, |n, document| {
            n.checked_add(document.text.len())
                .ok_or_else(|| invalid("source-edit extent overflow"))
        })?;
    let edit_bytes = changes
        .document_edits()
        .iter()
        .try_fold(source_bytes, |n, edit| {
            n.checked_add(edit.after.len())
                .ok_or_else(|| invalid("source-edit extent overflow"))
        })?;
    work.try_grow(
        edit_bytes
            .checked_mul(8)
            .and_then(|n| n.checked_add(8192))
            .ok_or_else(|| invalid("source-edit extent overflow"))?,
    )
    .map_err(pse_ids::CanonError::from)?;
    let mut parts = Vec::new();
    for bundle in documents.bundles() {
        cancel.checkpoint()?;
        let mut texts = bundle
            .documents
            .iter()
            .map(|document| (document.path.clone(), document.text.clone()))
            .collect();
        let own = changes
            .document_edits()
            .iter()
            .filter(|edit| {
                bundle
                    .documents
                    .iter()
                    .any(|document| document.id == edit.document_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        pse_authoring::document::apply_edits(&mut texts, &own)?;
        parts.push(pse_authoring::document::load_package_texts_owned(
            &texts,
            driver.registry(),
            ParseBudget::default(),
            reserver,
            cancel,
        )?);
    }
    Ok(OwnedDocumentSet::try_from_bundles(parts, reserver, cancel)?)
}
