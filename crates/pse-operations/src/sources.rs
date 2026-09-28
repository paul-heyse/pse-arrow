// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authored source bundles for job execution, content-addressed by the §6.1 package
//! content hash (ADR-0114 Outcome 14). A bundle is immutable: storing it again is a no-op.
//! The runtime computes the hashes with the authoring driver's one definition and verifies
//! them again when it loads a bundle.

use pse_ids::ContentHash;
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::sources as statements;
use tokio_postgres::types::ToSql;

use crate::bulk::{Cells, copy_in};
use crate::error::{Classify, OperationsError, Target};
use crate::generated::copy;
use crate::store::Store;
pub use pse_model::generated::identities::SourceBundleId;

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
    pub bundle_hash: SourceBundleId,
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
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let created = statements::insert_bundle()
            .params(
                &tx,
                &statements::InsertBundleParams {
                    bundle_hash: bundle.bundle_hash,
                    manifest: &bundle.manifest,
                },
            )
            .await
            .classify(target)?;
        if created == 1 {
            let hash: &(dyn ToSql + Sync) = &bundle.bundle_hash;
            let mut rows: Vec<Cells<'_>> = Vec::with_capacity(bundle.documents.len());
            for document in &bundle.documents {
                rows.push(vec![
                    hash,
                    &document.path,
                    &document.content_hash,
                    &document.content,
                ]);
            }
            copy_in(&tx, target, &copy::SOURCE_DOCUMENTS, &rows).await?;
        }
        tx.commit().await.classify(target)?;
        Ok(())
    }

    /// Read a bundle and its documents, ordered by path.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown hash; [`OperationsError::CorruptValue`]
    /// for a manifest that is not JSON; classified driver failures.
    pub async fn get(&self, bundle: &SourceBundleId) -> Result<SourceBundle, OperationsError> {
        let target = self.target();
        let client = self.store.client().await?;
        let stored = statements::bundle()
            .bind(&client, bundle)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "source bundle",
                id: bundle.as_id().to_prefixed(),
            })?;
        let documents = statements::documents()
            .bind(&client, bundle)
            .all()
            .await
            .classify(target)?
            .into_iter()
            .map(|document| SourceDocument {
                path: document.path,
                content_hash: document.content_hash,
                content: document.content,
            })
            .collect();
        Ok(SourceBundle {
            bundle_hash: stored.bundle_hash,
            manifest: serde_json::from_str(&stored.manifest).map_err(|error| {
                OperationsError::CorruptValue {
                    column: "source_bundles.manifest",
                    detail: error.to_string(),
                }
            })?,
            documents,
        })
    }
}
