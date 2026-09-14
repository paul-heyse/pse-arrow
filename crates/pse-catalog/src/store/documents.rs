// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact authored source objects, checked independently of relational fingerprints.

use std::collections::BTreeMap;

use bytes::Bytes;
use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::datatypes::{DataType, Field};
use pse_ids::{CancellationToken, ContentHash, EncodingChecksum, ReservationLease, SemanticId};
use pse_schema::model::{Cell, RelationKey};

use super::open::Catalog;
use super::verify::admission;
use crate::CatalogError;

/// Receipt for an immutable authored source object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentArtifact {
    /// The document identity carried by authored rows and source spans.
    pub document_id: SemanticId,
    /// Plain actual-byte content identity, using the existing encoding checksum entry.
    pub content_hash: ContentHash,
    /// Actual complete source extent.
    pub bytes: u64,
}

impl Catalog {
    /// Store exact authored source bytes before publishing relations that reference them.
    /// An existing object must equal the complete intended bytes before idempotent success.
    ///
    /// # Errors
    /// Extent/resource limits, cancellation, corruption or backend failure.
    pub async fn put_document(
        &self,
        document_id: SemanticId,
        source: &[u8],
        cancel: &CancellationToken,
    ) -> Result<DocumentArtifact, CatalogError> {
        cancel.checkpoint()?;
        if source.len() > self.limits.max_object_bytes {
            return Err(admission(
                "document",
                "source exceeds supported object extent",
            ));
        }
        let checksum = pse_ids::encoding_checksum(source);
        let mut reservation = self.reserver.open("store:document-copy");
        reservation.try_grow(
            source
                .len()
                .checked_mul(2)
                .ok_or_else(super::encode::overflow)?,
        )?;
        let lease = ReservationLease::new(reservation);
        let bytes = pse_ids::owned_buffer::attach_bytes(source.to_vec(), lease)?;
        self.ensure_create(
            &super::layout::document_path(document_id, &checksum),
            bytes,
            cancel,
        )
        .await?;
        Ok(DocumentArtifact {
            document_id,
            content_hash: checksum.content_hash(),
            bytes: u64::try_from(source.len()).map_err(|_| super::encode::overflow())?,
        })
    }

    /// Read exact authored source bytes, verifying actual object extent and checksum.
    /// This establishes byte integrity; document parsing belongs to authoring admission.
    ///
    /// # Errors
    /// Missing/truncated/corrupt source, resource limits, cancellation or backend failure.
    pub async fn read_document(
        &self,
        document_id: SemanticId,
        content_hash: ContentHash,
        cancel: &CancellationToken,
    ) -> Result<Bytes, CatalogError> {
        let checksum = EncodingChecksum(content_hash);
        let path = super::layout::document_path(document_id, &checksum);
        let bytes = self
            .read_bytes(&path, self.limits.max_object_bytes, cancel)
            .await?;
        let actual = pse_ids::encoding_checksum(&bytes);
        if actual != checksum {
            return Err(CatalogError::CorruptObject {
                path: path.to_string(),
                expected: checksum.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(bytes)
    }

    pub(crate) async fn admit_documents(
        &self,
        rows: &BTreeMap<RelationKey, RecordBatch>,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let mut lengths = BTreeMap::new();
        if let Some(spec) = self.registry.relation("authored.documents")
            && let Some(batch) = rows.get(&spec.key)
        {
            let schema = batch.schema();
            let id = schema
                .index_of("document_id")
                .map_err(super::encode::arrow)?;
            let content = schema
                .index_of("content_hash")
                .map_err(super::encode::arrow)?;
            let documents = pse_relations::cells::cells_from_batch(&self.registry, spec, batch)
                .map_err(|error| admission("authored.documents", &error.to_string()))?;
            for row in documents {
                let (Cell::Id(document), Cell::Hash(hash)) = (&row[id], &row[content]) else {
                    return Err(admission(
                        "authored.documents",
                        "source identity columns do not carry their declared logical values",
                    ));
                };
                let bytes = self.read_document(*document, *hash, cancel).await?;
                lengths.insert(
                    *document,
                    u64::try_from(bytes.len()).map_err(|_| super::encode::overflow())?,
                );
            }
        }
        for (key, batch) in rows {
            cancel.checkpoint()?;
            let spec = self
                .registry
                .relations()
                .iter()
                .find(|spec| &spec.key == key)
                .ok_or_else(|| {
                    admission(
                        "document source spans",
                        "missing exact relation declaration",
                    )
                })?;
            if !batch.schema().fields().iter().any(|field| has_span(field)) {
                continue;
            }
            let values = pse_relations::cells::cells_from_batch(&self.registry, spec, batch)
                .map_err(|error| admission("document source spans", &error.to_string()))?;
            for row in values {
                for (field, value) in batch.schema().fields().iter().zip(&row) {
                    check_span(field, value, &lengths)?;
                }
            }
        }
        Ok(())
    }
}
fn has_span(field: &Field) -> bool {
    if field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.source_span")
    {
        return true;
    }
    match field.data_type() {
        DataType::Struct(children) => children.iter().any(|child| has_span(child)),
        DataType::List(child) | DataType::FixedSizeList(child, _) => has_span(child),
        _ => false,
    }
}
fn check_span(
    field: &Field,
    value: &Cell,
    lengths: &BTreeMap<SemanticId, u64>,
) -> Result<(), CatalogError> {
    if matches!(value, Cell::Null) {
        return Ok(());
    }
    if field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.source_span")
    {
        let Cell::Struct(parts) = value else {
            return Err(admission(
                field.name(),
                "source span is not its declared struct",
            ));
        };
        let [Cell::Id(document), Cell::U64(start), Cell::U64(end)] = parts.as_slice() else {
            return Err(admission(
                field.name(),
                "source span identity/range is malformed",
            ));
        };
        if lengths
            .get(document)
            .is_none_or(|len| start > end || end > len)
        {
            return Err(admission(
                field.name(),
                "source span lies outside its actual admitted document bytes",
            ));
        }
        return Ok(());
    }
    match (field.data_type(), value) {
        (DataType::Struct(children), Cell::Struct(values)) => {
            for (child, value) in children.iter().zip(values) {
                check_span(child, value, lengths)?;
            }
        }
        (DataType::List(child) | DataType::FixedSizeList(child, _), Cell::List(values)) => {
            for value in values {
                check_span(child, value, lengths)?;
            }
        }
        _ => {}
    }
    Ok(())
}
