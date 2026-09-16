// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct typed control-row and operation-reference admission.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use pse_ids::{CancellationToken, SemanticId, SnapshotKind};
use pse_relations::{columnar::RelationRow, generated::authored};

use super::{ChangeSetDraft, RevisionBinding};
use crate::CatalogError;
use crate::store::open::Catalog;
use crate::store::sidecar::{RevisionReceipt, RevisionRef, SidecarArtifact};
use crate::store::verify::admission;

fn rows<T: RelationRow>(
    catalog: &Catalog,
    artifact: &SidecarArtifact,
) -> Result<Vec<T>, CatalogError> {
    if !Arc::ptr_eq(&artifact.admission, &catalog.admission) {
        return Err(admission(
            "change set",
            "foreign control artifact must be reopened",
        ));
    }
    let spec =
        T::relation(&catalog.registry).map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
    artifact
        .relation()
        .checked()
        .check_declaration(&catalog.registry, spec)
        .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
    T::rows(artifact.relation().checked()).map_err(|error| CatalogError::Semantic(Arc::new(error)))
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
    let mut headers = rows::<authored::change_sets::Row>(catalog, &draft.header)?;
    if headers.len() != 1 {
        return Err(admission(
            "change set",
            "exactly one typed header is required",
        ));
    }
    let header = headers.remove(0);
    if header.base_revision_id != base {
        return Err(admission(
            "change set",
            "actual header expects another base revision",
        ));
    }
    let operations = rows::<authored::change_ops::Row>(catalog, &draft.operations)?;
    let mut expected = BTreeMap::<String, BTreeSet<u64>>::new();
    for (ordinal, operation) in operations.into_iter().enumerate() {
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
            if usize::try_from(ordinal)
                .ok()
                .is_none_or(|ordinal| ordinal >= staged.relation().rows())
                || staged.relation().contract().canonical.relation_id != operation.relation_id
                || !staged.belongs_to(catalog)
            {
                return Err(admission(
                    "change set",
                    "staged ordinal does not name an actual row of its declared batch",
                ));
            }
            expected.entry(port.clone()).or_default().insert(ordinal);
        }
    }
    if expected.len() != draft.staged.len()
        || draft.staged.iter().any(|(port, batch)| {
            expected
                .get(port)
                .is_none_or(|ordinals| ordinals.len() != batch.relation().rows())
        })
    {
        return Err(admission(
            "change set",
            "staging inventory contains unreferenced or missing batch rows",
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
        let rows = rows::<authored::case_revisions::Row>(catalog, &output.artifact)?;
        let [revision] = rows.as_slice() else {
            return Err(admission(
                "change set",
                "exactly one actual case revision is required",
            ));
        };
        Some(revision.model_revision_id)
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
