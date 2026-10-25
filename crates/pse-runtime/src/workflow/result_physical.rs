// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact original retained physical source selection for readable result pages.
use super::{Runtime, PhysicalContext, WorkflowError, contract};
use pse_columnar::{CancellationToken, MemoryConsumer};
use pse_operations::{canonical::Revision, canonical_results::ResultRead};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};

impl Runtime {
    pub(super) async fn original_result_physical(
        &self,
        read: &ResultRead,
        cancel: &CancellationToken,
    ) -> Result<(PhysicalContext, String), WorkflowError> {
        let operations = self.operations()?;
        let response = super::connected_results::response_owner(self, "result:physical-source-selection", pse_operations::canonical::MetadataResponse::Selection)?;
        let sources = pse_operations::canonical::response_ownership_scope(response.clone(), operations.store().analysis_run_sources(&read.run().key)).await?;
        let revisions: Vec<Revision> = sources.into_iter().filter(|source| source.problem.starts_with("physical:rows:") || source.problem.starts_with("physical:context:")).collect();
        let contexts = revisions.iter().filter(|source| source.problem.starts_with("physical:context:")).collect::<Vec<_>>();
        let [receipt] = contexts.as_slice() else { return Err(contract("original physical context receipt unavailable")); };
        let expected = pse_ids::ContentHash::parse_hex(receipt.problem.strip_prefix("physical:context:").ok_or_else(|| contract("physical receipt kind"))?).map_err(|e| contract(e.to_string()))?;
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        hash.str("original.result.physical.closure.v1").u64(revisions.len() as u64);
        for revision in &revisions { hash.str(&revision.key).str(&revision.problem).u64(revision.sequence); }
        let closure = hash.finish_hash().to_string();
        let protection = super::physical_cache::protect(operations, &revisions, revisions.len() * 512).await?;
        let pool = self.shared.pool();
        let mut receipt_seen = false;
        let mut batches: BTreeMap<pse_schema::model::RelationKey, BTreeMap<String, FieldCheckedBatch>> = BTreeMap::new();
        for pin in protection.pins() {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let mut after = String::new();
            loop {
                let page_owner = super::connected_results::response_owner(self, "result:physical-membership-page", pse_operations::canonical::MetadataResponse::Selection)?;
                let page = pse_operations::canonical::response_ownership_scope(page_owner.clone(), operations.store().membership_page(pin, None, &after)).await?;
                if page.is_empty() { break; }
                after = page.last().ok_or_else(|| contract("physical membership page"))?.key.clone();
                for member in page {
                    let extent = operations.store().selected_object_extent(pin, &member.version).await?.ok_or_else(|| contract("original physical object unavailable"))?;
                    let allocation = MemoryConsumer::new("result:physical-source-decode").register(&pool);
                    allocation.try_grow(extent.saturating_mul(8).saturating_add(4096)).map_err(pse_engine::EngineError::from)?;
                    let allocation = pse_columnar::AllocationLease::new(allocation);
                    let object = pse_operations::canonical::response_ownership_scope(allocation.clone(), operations.store().selected_object(pin, &member.version)).await?.ok_or_else(|| contract("original physical object unavailable"))?;
                    if pin.revision() == *receipt && object.kind != "physical:context:v1" { return Err(contract("original physical context receipt contains a non-context payload")); }
                    if object.kind == "physical:context:v1" {
                        if pin.revision() != *receipt || receipt_seen { return Err(contract("original physical context receipt is misplaced or duplicated")); }
                        let (version, actual): (u8, pse_ids::ContentHash) = serde_json::from_slice(object.payload.as_slice()).map_err(|e| contract(e.to_string()))?;
                        if version != 1 || actual != expected || pin.revision() != *receipt { return Err(contract("original physical context receipt differs")); }
                        receipt_seen = true;
                        continue;
                    }
                    if object.kind != "physical:rows:v1" { return Err(contract("original physical row interpretation differs")); }
                    let spec = self.registry().relation(&member.scope).ok_or_else(|| contract("original physical relation unavailable"))?;
                    let mut stream = datafusion::arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(object.payload.as_slice()), None).map_err(|e| contract(e.to_string()))?;
                    let mut count = 0;
                    for batch in &mut stream {
                        count += 1;
                        if count > 1 { return Err(contract("original physical block has multiple batches")); }
                        let batch = batch.map_err(|e| contract(e.to_string()))?;
                        let checked = FieldCheckedBatch::admit(self.registry(), spec, batch, &self.validation_context(), cancel).map_err(super::relation)?.with_export_owner(allocation.clone());
                        if batches.entry(spec.key).or_default().insert(member.logical.clone(), checked).is_some() { return Err(contract("duplicate original physical row coordinate")); }
                    }
                    if count != 1 { return Err(contract("original physical block is empty")); }
                }
            }
        }
        if !receipt_seen { return Err(contract("original physical context receipt payload unavailable")); }
        let batches = batches.into_iter().map(|(key, chunks)| {
            let chunks = chunks.into_values().collect::<Vec<_>>();
            let spec = self.registry().relation_by_key(key).ok_or_else(|| contract("original physical declaration unavailable"))?;
            let batch = FieldCheckedBatch::concat_reserved(self.registry(), spec, &chunks, &pool, cancel).map_err(super::relation)?;
            Ok((key, batch))
        }).collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
        let inventory = pse_admission::physical::PhysicalInventory::from_batches(&pool, self.registry(), batches, cancel).map_err(|e| WorkflowError::Engine(pse_engine::EngineError::Semantic(Arc::new(e))))?;
        let mut physical = PhysicalContext::admitted(Arc::new(inventory));
        if physical.identity() != expected { return Err(contract("original physical rows do not reconstruct the original context")); }
        physical._source_owner = Some(protection);
        Ok((physical, closure))
    }
}
