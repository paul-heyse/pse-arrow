// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted physical context shared by authored models and explicit analysis requests.
use super::{Runtime, WorkflowError, contract, relation};
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
    pub(super) _inventory: Option<Arc<crate::physical::PhysicalInventory>>,
}
impl PhysicalContext {
    /// Read the admitted registry used to resolve public physical type names.
    pub fn quantities(&self) -> &QuantityRegistry {
        &self.quantities
    }
    /// Custom contexts retain the very rows admitted by PhysicalInventory.
    pub fn admitted(inventory: Arc<crate::physical::PhysicalInventory>) -> Self {
        let quantities = Arc::new(inventory.quantities().clone());
        let preconditions = inventory.compiler_preconditions();
        let key = pse_compiler::workspace::physical_identity(&quantities, &preconditions);
        Self {
            quantities,
            preconditions,
            key,
            sources: inventory.source_batches().clone(),
            _inventory: Some(inventory),
        }
    }
    /// Full exact physical context identity, not a caller's revision assertion.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
}
impl Runtime {
    /// Admit the exact physical sources through the existing registry/session boundary.
    pub async fn physical_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<PhysicalContext, WorkflowError> {
        PhysicalContext::from_documents(documents, self.registry.clone(), &self.sessions, cancel)
            .await
    }
}
impl PhysicalContext {
    /// Shared physical admission for workflows and standalone pure checks.
    pub(crate) async fn from_documents(
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        registry: Arc<pse_schema::Registry>,
        sessions: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, WorkflowError> {
        documents.validate_registry(&registry)?;
        let batches = crate::authoring_driver::p1::source_batches(documents.bundles(), &registry)?;
        let keys = crate::physical::input_keys(&registry);
        let roots = keys
            .iter()
            .filter_map(|key| registry.relation(&key.qualified_name()).map(|spec| spec.id))
            .collect();
        let support = pse_schema::product::support_closure(&registry, &roots)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let mut physical = BTreeMap::new();
        let mut retained_support = BTreeMap::new();
        for (id, batch) in batches {
            let spec = registry
                .relation_by_id(id)
                .ok_or_else(|| contract("unknown physical declaration"))?;
            if keys.contains(&spec.key) {
                physical.insert(spec.key, batch);
            } else if support.contains(&id) {
                retained_support.insert(
                    spec.key,
                    batch.retained(sessions.pool(), cancel).map_err(relation)?,
                );
            }
        }
        let session = sessions.candidate_checked(physical, registry.clone(), cancel)?;
        let inventory = crate::physical::PhysicalInventory::load(&session, &registry, cancel)
            .await
            .map_err(|e| pse_engine::EngineError::Semantic(Arc::new(e)))?;
        let mut context = PhysicalContext::admitted(Arc::new(inventory));
        context.sources.extend(retained_support);
        Ok(context)
    }
}
