// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted physical context shared by authored models and explicit analysis requests.
use super::{Runtime, WorkflowError};
use pse_admission::physical::PhysicalPackage;
use pse_ids::ContentHash;
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};
/// Immutable physical declarations retaining their actual admitted source rows.
#[derive(Clone, Debug)]
pub struct PhysicalContext {
    pub(crate) quantities: Arc<QuantityRegistry>,
    pub(crate) preconditions: Arc<PhysicalPreconditions>,
    pub(crate) sources: BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
    pub(crate) key: ContentHash,
    /// The package whose physical document declared the names (ADR-0123 Outcome 6),
    /// when the context was admitted from documents.
    pub(crate) package: Option<PhysicalPackage>,
    pub(super) _inventory: Option<Arc<pse_admission::physical::PhysicalInventory>>,
    pub(super) _source_owner: Option<Arc<super::physical_cache::PhysicalOwner>>,
}
impl PhysicalContext {
    /// Read the admitted registry used to resolve public physical type names.
    pub fn quantities(&self) -> &QuantityRegistry {
        &self.quantities
    }
    /// Actual immutable prerequisites admitted with this physical inventory.
    pub fn preconditions(&self) -> &PhysicalPreconditions {
        &self.preconditions
    }
    /// Custom contexts retain the very rows admitted by PhysicalInventory.
    pub fn admitted(inventory: Arc<pse_admission::physical::PhysicalInventory>) -> Self {
        let quantities = Arc::new(inventory.quantities().clone());
        let preconditions = inventory.compiler_preconditions();
        let key = pse_compiler::workspace::physical_identity(&quantities, &preconditions);
        Self {
            quantities,
            preconditions,
            key,
            sources: inventory.source_batches().clone(),
            package: None,
            _inventory: Some(inventory),
            _source_owner: None,
        }
    }
    /// Full exact physical context identity, not a caller's revision assertion.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
    pub(super) fn same_sources(&self, other: &Self) -> bool {
        self.key == other.key
            && Arc::ptr_eq(&self.quantities, &other.quantities)
            && Arc::ptr_eq(&self.preconditions, &other.preconditions)
            && self.package == other.package
            && self.sources.len() == other.sources.len()
            && self.sources.iter().all(|(key, batch)| {
                other
                    .sources
                    .get(key)
                    .is_some_and(|other| batch.same_source(other))
            })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        let sources = self.sources.values().fold(0_usize, |bytes, batch| {
            bytes
                .saturating_add(
                    batch
                        .owned()
                        .and_then(|owned| owned.retained_bytes().ok())
                        .unwrap_or_else(|| batch.batch().get_array_memory_size()),
                )
                .saturating_add(512)
        });
        sources
            .saturating_add(self.quantities.allocation_extent())
            .saturating_add(self.preconditions.allocation_extent())
            .saturating_add(
                self.package
                    .as_ref()
                    .map_or(0, |p| p.header.heap_bytes() + p.name.len() + 256),
            )
            .saturating_add(1024)
    }
}

impl Runtime {
    /// Admit the exact physical sources through the existing registry/session boundary.
    pub async fn physical_from_documents(
        &self,
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<PhysicalContext, WorkflowError> {
        let activity = self.canonical_store().admit_activity()?;
        activity
            .scope(async { PhysicalContext::from_documents(documents, &self.model, cancel).await })
            .await
    }
}
impl PhysicalContext {
    /// Admit physical documents through the lower native source owner.
    pub(crate) async fn from_documents(
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        model: &pse_engine::session::BoundModel,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, WorkflowError> {
        let admitted =
            pse_admission::physical::PhysicalDocuments::from_documents(documents, model, cancel)
                .await
                .map_err(|error| match error {
                    pse_admission::physical::PhysicalError::Authoring(error) => {
                        WorkflowError::Authoring(error)
                    }
                    error => {
                        WorkflowError::Engine(pse_engine::EngineError::Semantic(Arc::new(error)))
                    }
                })?;
        let (inventory, sources, package) = admitted.into_parts();
        let mut context = Self::admitted(inventory);
        context.package = package;
        context.sources = sources;
        Ok(context)
    }
}
