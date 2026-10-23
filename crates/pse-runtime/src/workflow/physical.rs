// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted physical context shared by authored models and explicit analysis requests.
use super::{Runtime, WorkflowError};
pub use pse_admission::physical::PhysicalAdmissionReceipt;
use pse_admission::physical::PhysicalPackage;
use pse_ids::ContentHash;
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};
/// Immutable physical declarations retaining their actual admitted source rows.
#[derive(Clone, Debug)]
pub struct PhysicalContext {
    pub(super) engine_receipt: Option<PhysicalAdmissionReceipt>,
    pub(super) documents: Option<pse_admission::authoring_driver::document::OwnedDocumentSet>,
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
            engine_receipt: None,
            documents: None,
            quantities,
            preconditions,
            key,
            sources: inventory.source_batches().clone(),
            package: None,
            _inventory: Some(inventory),
            _source_owner: None,
        }
    }
    /// Copied observations from the session that actually admitted these physical documents.
    pub fn engine_receipt(&self) -> Option<&PhysicalAdmissionReceipt> {
        self.engine_receipt.as_ref()
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
    /// Heap metadata copied by a context clone; arrays/quantity owners remain shared.
    pub(super) fn metadata_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        self.sources
            .len()
            .saturating_mul(
                size_of::<FieldCheckedBatch>() + size_of::<pse_schema::model::RelationKey>() + 256,
            )
            .saturating_add(self.package.as_ref().map_or(0, |package| {
                package.header.heap_bytes() + package.name.capacity() + 256
            }))
            .saturating_add(size_of::<Self>() + 256)
    }
    /// Renew only affected checks under this actual invocation, sharing immutable arrays.
    pub(super) fn readmit_context(
        &self,
        runtime: &Runtime,
        context: &pse_relations::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, WorkflowError> {
        let mut result = self.clone();
        for (key, batch) in &mut result.sources {
            let spec = runtime
                .registry()
                .relation_by_key(*key)
                .ok_or_else(|| super::contract("physical source relation absent"))?;
            *batch = batch
                .readmit_context(runtime.registry(), spec, context, cancel)
                .map_err(super::relation)?;
        }
        Ok(result)
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
            .saturating_add(
                self.engine_receipt
                    .as_ref()
                    .map_or(0, PhysicalAdmissionReceipt::retained_bytes),
            )
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
        self.physical_from_documents_with_receipt(documents, cancel, None)
            .await
    }
    /// Optional caller-owned copy survives admission failure or dropped/cancelled work.
    pub async fn physical_from_documents_with_receipt(
        &self,
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        cancel: &pse_columnar::CancellationToken,
        mut receipt: Option<&mut PhysicalAdmissionReceipt>,
    ) -> Result<PhysicalContext, WorkflowError> {
        if let Some(receipt) = receipt.as_deref_mut() {
            receipt.reset();
        }
        let activity = self.admit_activity()?;
        activity
            .scope(PhysicalContext::from_documents_with_receipt(
                documents,
                &self.model,
                cancel,
                receipt,
            ))
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
        Self::from_documents_with_receipt(documents, model, cancel, None).await
    }
    async fn from_documents_with_receipt(
        documents: &pse_admission::authoring_driver::document::OwnedDocumentSet,
        model: &pse_engine::session::BoundModel,
        cancel: &pse_columnar::CancellationToken,
        receipt: Option<&mut PhysicalAdmissionReceipt>,
    ) -> Result<Self, WorkflowError> {
        let mut retained = PhysicalAdmissionReceipt::new(16 << 20);
        let receipt = receipt.unwrap_or(&mut retained);
        let admitted = pse_admission::physical::PhysicalDocuments::from_documents_with_receipt(
            documents,
            model,
            cancel,
            Some(&mut *receipt),
        )
        .await
        .map_err(|error| match error {
            pse_admission::physical::PhysicalError::Authoring(error) => {
                WorkflowError::Authoring(error)
            }
            error => WorkflowError::Engine(pse_engine::EngineError::Semantic(Arc::new(error))),
        })?;
        let (inventory, sources, package) = admitted.into_parts();
        let mut context = Self::admitted(inventory);
        context.engine_receipt = Some(receipt.clone());
        context.documents = Some(documents.clone());
        context.package = package;
        context.sources = sources;
        Ok(context)
    }
}

impl Runtime {
    /// Preflight and retain exact original physical document bytes for ordinary source publication.
    /// This is shared by studies and project context; no second physical codec is introduced.
    pub(in crate::workflow) fn physical_source_documents(
        &self,
        physical: &PhysicalContext,
    ) -> Result<pse_columnar::Leased<BTreeMap<String, Vec<u8>>>, WorkflowError> {
        let documents = physical.documents.as_ref().ok_or_else(|| {
            super::contract("recording requires exact admitted physical document bytes")
        })?;
        let extent = documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
            .try_fold(0usize, |total, document| {
                total
                    .checked_add(document.bytes().len())
                    .and_then(|n| n.checked_add(document.path.len() + 256))
            })
            .ok_or_else(|| super::contract("physical document extent"))?;
        let owner = self
            .shared
            .math()
            .reserve("physical:document-recording", extent)?;
        let mut sources = BTreeMap::new();
        for document in documents
            .bundles()
            .iter()
            .flat_map(|bundle| &bundle.documents)
        {
            if sources
                .insert(document.path.clone(), document.bytes().to_vec())
                .is_some()
            {
                return Err(super::contract("physical document path is ambiguous"));
            }
        }
        if sources.is_empty() {
            return Err(super::contract(
                "recording requires physical document bytes",
            ));
        }
        Ok(pse_columnar::Leased::new(Arc::new(sources), owner))
    }
    pub(in crate::workflow) async fn record_physical_source(
        &self,
        physical: &PhysicalContext,
        cancel: &crate::CancelSource,
    ) -> Result<super::PhysicalSource, WorkflowError> {
        let documents = self.physical_source_documents(physical)?;
        super::study::creation_effect(cancel, self.operations()?.put_sources(&documents)).await
    }
}
