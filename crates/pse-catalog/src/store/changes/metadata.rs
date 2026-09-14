// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Size actual borrowed receipt/source metadata before cloning and preserve its owner.
use super::{ChangeSetDraft, ChangeSetWire, DocumentVersion, RevisionBinding};
use crate::store::{
    control::{add, revision_extent, vector},
    stage_owned::{map_slots, member_extent},
};
use crate::{CatalogError, Snapshot};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn draft_extent(draft: &ChangeSetDraft) -> Result<usize, CatalogError> {
    let mut bytes = add(size_of::<ChangeSetDraft>() + 64, map_slots(&draft.staged)?)?;
    bytes = add(bytes, member_extent(draft.header.reference().member())?)?;
    bytes = add(bytes, member_extent(draft.operations.reference().member())?)?;
    for (port, artifact) in &draft.staged {
        bytes = add(
            bytes,
            add(
                port.capacity(),
                member_extent(artifact.reference().0.member())?,
            )?,
        )?;
    }
    bytes = add(bytes, vector(&draft.supporting_revisions)?)?;
    for revision in &draft.supporting_revisions {
        bytes = add(bytes, revision_extent(revision)?)?;
    }
    Ok(bytes)
}
pub(super) fn wire_extent(wire: &ChangeSetWire) -> Result<usize, CatalogError> {
    let mut bytes = add(size_of::<ChangeSetWire>() + 64, map_slots(&wire.staged)?)?;
    bytes = add(bytes, member_extent(wire.header.member())?)?;
    bytes = add(bytes, member_extent(wire.operations.member())?)?;
    for (port, staged) in &wire.staged {
        bytes = add(
            bytes,
            add(port.capacity(), member_extent(staged.0.member())?)?,
        )?;
    }
    bytes = add(bytes, vector(&wire.sources)?)?;
    for source in &wire.sources {
        for version in source.before.iter().chain(source.after.iter()) {
            bytes = add(bytes, version.path.capacity())?;
        }
    }
    bytes = add(bytes, vector(&wire.supporting_revisions)?)?;
    for revision in &wire.supporting_revisions {
        bytes = add(bytes, revision_extent(revision)?)?;
    }
    for RevisionBinding { revision, .. } in wire.base.iter().chain(std::iter::once(&wire.output)) {
        bytes = add(bytes, revision_extent(revision)?)?;
    }
    Ok(bytes)
}
pub(super) fn source_extent(snapshot: &Snapshot) -> Result<usize, CatalogError> {
    let mut bytes = 4096;
    let mut stack = vec![snapshot];
    let mut visited = BTreeSet::new();
    while let Some(snapshot) = stack.pop() {
        if !visited.insert(std::ptr::from_ref(snapshot)) {
            continue;
        }
        if let Some(relation) = snapshot.relation("authored", "documents") {
            bytes = add(
                bytes,
                crate::store::membership::validation_extent(relation.batch())?,
            )?;
            bytes = add(
                bytes,
                relation
                    .batch()
                    .num_rows()
                    .checked_mul(size_of::<DocumentVersion>() * 4)
                    .ok_or_else(crate::store::encode::overflow)?,
            )?;
        }
        stack.extend(snapshot.parents().values().map(Arc::as_ref));
    }
    Ok(bytes)
}

pub(super) fn publication(
    catalog: &crate::Catalog,
    draft: &ChangeSetDraft,
    base: Option<(&crate::store::refs::RefState, &Snapshot)>,
    output: &crate::store::sidecar::RevisionReceipt,
) -> Result<Box<dyn pse_ids::Reservation>, CatalogError> {
    let mut metadata = catalog.reserver.open("store:change-set-metadata-write");
    let forecast = add(draft_extent(draft)?, revision_extent(&output.reference)?)?;
    let forecast = add(forecast, source_extent(&output.snapshot)?)?;
    let forecast = if let Some((state, snapshot)) = base {
        let revision = state.revision_ref().map_or(Ok(0), revision_extent)?;
        add(forecast, add(revision, source_extent(snapshot)?)?)?
    } else {
        forecast
    };
    metadata.try_grow(crate::store::control::mul(add(forecast, 4096)?, 4)?)?;
    Ok(metadata)
}
