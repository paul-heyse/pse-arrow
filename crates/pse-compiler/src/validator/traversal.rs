// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Share source parses by actual admitted source owner throughout cold admission.

use crate::CompilerError;
use pse_authoring::document::OwnedDocumentSet;
use pse_catalog::{Catalog, Snapshot};
use pse_ids::CancellationToken;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(super) struct Documents(tokio::sync::Mutex<Vec<(Arc<Snapshot>, OwnedDocumentSet)>>);

impl Documents {
    pub(super) async fn get(
        &self,
        catalog: &Catalog,
        source: Option<&Arc<Snapshot>>,
        cancel: &CancellationToken,
    ) -> Result<OwnedDocumentSet, CompilerError> {
        let Some(source) = source else {
            return Ok(OwnedDocumentSet::default());
        };
        let mut entries = tokio::select! {
            biased;
            () = cancel.cancelled() => {
                cancel.checkpoint().map_err(pse_catalog::CatalogError::from)?;
                return Err(crate::passes::dag::invalid("cancelled source wait has no cancellation witness"));
            },
            entries = self.0.lock() => entries,
        };
        if let Some((_, documents)) = entries.iter().find(|(owner, _)| Arc::ptr_eq(owner, source)) {
            return Ok(documents.clone());
        }
        let inputs = crate::driver::inputs::inventory(source, catalog.registry())?;
        let rows = crate::driver::inputs::row_inventory(&inputs);
        let documents = crate::driver::inputs::documents(catalog, &rows, cancel)?;
        tracing::debug!(snapshot = %source.snapshot_id(), "cold admission parsed source owner");
        entries.push((Arc::clone(source), documents.clone()));
        Ok(documents)
    }
}
