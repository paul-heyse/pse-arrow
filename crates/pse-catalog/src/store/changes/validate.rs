// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct typed control-row and operation-reference admission.

use std::collections::BTreeSet;
use std::sync::Arc;

use pse_ids::{CancellationToken, SemanticId, SnapshotKind};
use pse_relations::generated::authored;
use pse_schema::model::Cell;

use super::{ChangeSetDraft, RevisionBinding};
use crate::CatalogError;
use crate::store::open::Catalog;
use crate::store::sidecar::{RevisionReceipt, RevisionRef, SidecarArtifact};
use crate::store::verify::admission;

fn rows(
    catalog: &Catalog,
    artifact: &SidecarArtifact,
    name: &str,
) -> Result<Vec<Vec<Cell>>, CatalogError> {
    if !Arc::ptr_eq(&artifact.admission, &catalog.admission) {
        return Err(admission(
            "change set",
            "foreign control artifact must be reopened",
        ));
    }
    let spec = catalog
        .registry
        .relation(name)
        .ok_or_else(|| admission("change set", "control declaration missing"))?;
    if artifact.relation().contract().canonical.relation_id != spec.id {
        return Err(admission(
            "change set",
            "control artifact has the wrong relation",
        ));
    }
    pse_relations::cells::cells_from_batch(&catalog.registry, spec, artifact.relation().batch())
        .map_err(|error| CatalogError::Semantic(Arc::new(error)))
}
pub(super) fn envelope(
    catalog: &Catalog,
    draft: &ChangeSetDraft,
    base: SemanticId,
) -> Result<SemanticId, CatalogError> {
    let mut reservation = catalog.reserver.open("store:change-control-admission");
    let extent = crate::store::membership::validation_extent(draft.header.relation().batch())?
        .checked_add(crate::store::membership::validation_extent(
            draft.operations.relation().batch(),
        )?)
        .ok_or_else(crate::store::encode::overflow)?;
    reservation.try_grow(extent)?;
    let mut headers = rows(catalog, &draft.header, "authored.change_sets")?;
    if headers.len() != 1 {
        return Err(admission(
            "change set",
            "exactly one typed header is required",
        ));
    }
    let header = authored::change_sets::Row::from_cells(headers.remove(0))
        .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
    if header.base_revision_id != base {
        return Err(admission(
            "change set",
            "actual header expects another base revision",
        ));
    }
    let operations = rows(catalog, &draft.operations, "authored.change_ops")?;
    let mut expected = BTreeSet::new();
    for (ordinal, row) in operations.into_iter().enumerate() {
        let operation = authored::change_ops::Row::from_cells(row)
            .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
        if operation.change_set_id != header.change_set_id
            || usize::try_from(operation.ordinal).ok() != Some(ordinal)
        {
            return Err(admission(
                "change set",
                "operation identity or ordinal differs from its header",
            ));
        }
        let mut references = vec![(
            &operation.row_key.staged_port,
            operation.row_key.staged_ordinal,
        )];
        if let Some(row) = &operation.row {
            references.push((&row.staged_port, row.staged_ordinal));
        }
        for (port, ordinal) in references {
            let staged = draft
                .staged
                .get(port)
                .ok_or_else(|| admission("change set", "referenced staging port is missing"))?;
            if ordinal != 0
                || staged.relation().rows() != 1
                || staged.relation().contract().canonical.relation_id != operation.relation_id
                || !staged.belongs_to(catalog)
            {
                return Err(admission(
                    "change set",
                    "staged reference does not name its actual single declared row",
                ));
            }
            expected.insert(port.clone());
        }
    }
    if expected != draft.staged.keys().cloned().collect() {
        return Err(admission(
            "change set",
            "staging inventory contains unreferenced or missing ports",
        ));
    }
    Ok(header.change_set_id)
}

pub(super) async fn supporting(
    catalog: &Catalog,
    revisions: &[RevisionRef],
    output: &RevisionReceipt,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    let mut seen = BTreeSet::new();
    let model = if output.snapshot.manifest().snapshot_kind == SnapshotKind::Case {
        let rows = rows(catalog, &output.artifact, "authored.case_revisions")?;
        let key = output
            .artifact
            .relation()
            .batch()
            .schema()
            .index_of("model_revision_id")
            .map_err(crate::store::encode::arrow)?;
        let Some(Cell::Id(id)) = rows.first().and_then(|row| row.get(key)) else {
            return Err(admission(
                "change set",
                "case revision lacks actual model revision identity",
            ));
        };
        Some(*id)
    } else {
        None
    };
    for revision in revisions {
        if !seen.insert(revision.revision_id) {
            return Err(admission("change set", "duplicate supporting revision"));
        }
        let artifact = catalog.read_sidecar(&revision.artifact, cancel).await?;
        if Some(revision.revision_id) == model {
            let parent = output
                .snapshot
                .parents()
                .get("model")
                .ok_or_else(|| admission("change set", "case has no actual model parent"))?;
            catalog.revision_receipt(&artifact, revision.revision_id, parent)?;
        } else {
            return Err(admission(
                "change set",
                "supporting revision is not an explicitly required model parent",
            ));
        }
    }
    if model.is_some_and(|model| !seen.contains(&model)) {
        return Err(admission(
            "change set",
            "case model revision artifact must be retained explicitly",
        ));
    }
    Ok(())
}

pub(super) async fn revision_binding(
    catalog: &Catalog,
    binding: &RevisionBinding,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    let artifact = catalog
        .read_sidecar(&binding.revision.artifact, cancel)
        .await?;
    let kind = match artifact.relation().member().name.as_str() {
        "model_revisions" => SnapshotKind::Model,
        "case_revisions" => SnapshotKind::Case,
        _ => return Err(admission("change set", "binding is not a typed revision")),
    };
    catalog.check_revision(
        &artifact,
        binding.revision.revision_id,
        binding.manifest.snapshot_id,
        kind,
    )
}
