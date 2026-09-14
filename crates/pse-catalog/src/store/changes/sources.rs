// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source changes come from actual admitted document inventories and complete bytes.

use bytes::Bytes;
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::model::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::{DocumentArtifact, DocumentVersion, SourceChange};
use crate::store::open::Catalog;
use crate::store::verify::admission;
use crate::{CatalogError, Snapshot};

async fn inventory(
    catalog: &Catalog,
    snapshot: &Snapshot,
    cancel: &CancellationToken,
) -> Result<BTreeMap<SemanticId, (DocumentVersion, Bytes)>, CatalogError> {
    let mut result = BTreeMap::new();
    let Some(spec) = catalog.registry.relation("authored.documents") else {
        return Ok(result);
    };
    let mut stack = vec![snapshot];
    let mut visited = BTreeSet::new();
    while let Some(snapshot) = stack.pop() {
        if !visited.insert(std::ptr::from_ref(snapshot)) {
            continue;
        }
        if let Some(relation) = snapshot.relation("authored", "documents") {
            let batch = relation.batch();
            let mut reservation = catalog.reserver.open("store:source-inventory");
            reservation.try_grow(crate::store::membership::validation_extent(batch)?)?;
            let schema = batch.schema();
            let id = schema
                .index_of("document_id")
                .map_err(crate::store::encode::arrow)?;
            let path = schema
                .index_of("path")
                .map_err(crate::store::encode::arrow)?;
            let hash = schema
                .index_of("content_hash")
                .map_err(crate::store::encode::arrow)?;
            let rows = pse_relations::cells::cells_from_batch(&catalog.registry, spec, batch)
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
            for row in rows {
                let (Cell::Id(document), Cell::Text(path), Cell::Hash(hash)) =
                    (&row[id], &row[path], &row[hash])
                else {
                    return Err(admission(
                        "change sources",
                        "invalid declared source columns",
                    ));
                };
                if result.contains_key(document) {
                    continue;
                }
                let bytes = catalog.read_document(*document, *hash, cancel).await?;
                let version = DocumentVersion {
                    path: path.clone(),
                    document: DocumentArtifact {
                        document_id: *document,
                        content_hash: *hash,
                        bytes: u64::try_from(bytes.len())
                            .map_err(|_| crate::store::encode::overflow())?,
                    },
                };
                result.insert(*document, (version, bytes));
            }
        }
        stack.extend(snapshot.parents().values().map(Arc::as_ref));
    }
    Ok(result)
}
pub(super) async fn changes(
    catalog: &Catalog,
    base: Option<&Snapshot>,
    output: &Snapshot,
    cancel: &CancellationToken,
) -> Result<Vec<SourceChange>, CatalogError> {
    let before = if let Some(base) = base {
        inventory(catalog, base, cancel).await?
    } else {
        BTreeMap::new()
    };
    let after = inventory(catalog, output, cancel).await?;
    let identities = before
        .keys()
        .chain(after.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    for id in identities {
        let left = before.get(&id);
        let right = after.get(&id);
        // Byte equality and actual paths determine a change; digests only label the
        // exact source versions already read and checked above.
        if left
            .zip(right)
            .is_some_and(|((a, ab), (b, bb))| a.path == b.path && ab == bb)
        {
            continue;
        }
        changes.push(SourceChange {
            document_id: id,
            before: left.map(|(version, _)| version.clone()),
            after: right.map(|(version, _)| version.clone()),
        });
    }
    Ok(changes)
}
pub(super) async fn verify(
    catalog: &Catalog,
    changes: &[SourceChange],
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    let mut seen = BTreeSet::new();
    for change in changes {
        if !seen.insert(change.document_id) || (change.before.is_none() && change.after.is_none()) {
            return Err(admission(
                "change sources",
                "duplicate or empty source change",
            ));
        }
        for version in change.before.iter().chain(change.after.iter()) {
            if version.document.document_id != change.document_id {
                return Err(admission(
                    "change sources",
                    "source version names a different document",
                ));
            }
            let bytes = catalog
                .read_document(change.document_id, version.document.content_hash, cancel)
                .await?;
            if u64::try_from(bytes.len()).ok() != Some(version.document.bytes) {
                return Err(admission(
                    "change sources",
                    "actual source extent differs from receipt",
                ));
            }
        }
    }
    Ok(())
}
