// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authored source bundles for job execution, content-addressed by the §6.1 package
//! content hash (ADR-0114 Outcome 14). A bundle is immutable: storing it again is a no-op.
//! The runtime computes the hashes with the authoring driver's one definition and verifies
//! them again when it loads a bundle.

use pse_ids::ContentHash;
use sqlx::Row;

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;

/// One authored document of a bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDocument {
    /// The path within the package, as the loader keys it.
    pub path: String,
    /// The document content hash.
    pub content_hash: ContentHash,
    /// The authored text.
    pub content: String,
}

/// A content-addressed set of authored documents.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceBundle {
    /// The package content hash over the bundle's path/text pairs.
    pub bundle_hash: ContentHash,
    /// A JSON manifest naming the documents.
    pub manifest: serde_json::Value,
    /// The documents, by path.
    pub documents: Vec<SourceDocument>,
}

/// The source bundle repository.
#[derive(Clone, Copy, Debug)]
pub struct Sources<'s> {
    store: &'s Store,
}

impl<'s> Sources<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Store a bundle and its documents in one transaction. A bundle already stored under
    /// the same hash is left as it is.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn put(&self, bundle: &SourceBundle) -> Result<(), OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let created = sqlx::query(
            "INSERT INTO pse_ops.source_bundles (bundle_hash, manifest) VALUES ($1, $2) \
             ON CONFLICT (bundle_hash) DO NOTHING",
        )
        .bind(codec::hash_bytes(&bundle.bundle_hash))
        .bind(&bundle.manifest)
        .execute(&mut *tx)
        .await
        .classify(target)?
        .rows_affected();
        if created == 1 {
            sqlx::query(
                "INSERT INTO pse_ops.source_documents (bundle_hash, path, content_hash, content) \
                 SELECT $1, d.path, d.content_hash, d.content \
                 FROM UNNEST($2::text[], $3::bytea[], $4::text[]) AS d (path, content_hash, content)",
            )
            .bind(codec::hash_bytes(&bundle.bundle_hash))
            .bind(
                bundle
                    .documents
                    .iter()
                    .map(|d| d.path.as_str())
                    .collect::<Vec<_>>(),
            )
            .bind(
                bundle
                    .documents
                    .iter()
                    .map(|d| codec::hash_bytes(&d.content_hash))
                    .collect::<Vec<_>>(),
            )
            .bind(
                bundle
                    .documents
                    .iter()
                    .map(|d| d.content.as_str())
                    .collect::<Vec<_>>(),
            )
            .execute(&mut *tx)
            .await
            .classify(target)?;
        }
        tx.commit().await.classify(target)?;
        Ok(())
    }

    /// Read a bundle and its documents, ordered by path.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown hash; classified driver failures.
    pub async fn get(&self, bundle: &ContentHash) -> Result<SourceBundle, OperationsError> {
        let target = self.target();
        let manifest: serde_json::Value = sqlx::query_scalar(
            "SELECT manifest FROM pse_ops.source_bundles WHERE bundle_hash = $1",
        )
        .bind(codec::hash_bytes(bundle))
        .fetch_optional(self.store.pool())
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "source bundle",
            id: bundle.to_prefixed(),
        })?;
        let documents = sqlx::query(
            "SELECT path, content_hash, content FROM pse_ops.source_documents \
             WHERE bundle_hash = $1 ORDER BY path",
        )
        .bind(codec::hash_bytes(bundle))
        .fetch_all(self.store.pool())
        .await
        .classify(target)?
        .iter()
        .map(|row| {
            Ok(SourceDocument {
                path: row.try_get("path")?,
                content_hash: codec::hash(row, "content_hash")?,
                content: row.try_get("content")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .classify(target)?;
        Ok(SourceBundle {
            bundle_hash: *bundle,
            manifest,
            documents,
        })
    }
}
