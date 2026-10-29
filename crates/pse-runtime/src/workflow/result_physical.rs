// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact original retained physical source selection for readable result pages.
use super::{PhysicalContext, RetainedDataServices, WorkflowError, contract};
use pse_columnar::{CancellationToken, MemoryConsumer};
use pse_operations::{canonical::Revision, canonical_results::ResultRead};
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};

fn integrity(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Integrity(message.into())
}

/// Physical publication records the full versioned RelationKey display, not a
/// latest-version qualified-name selector. Recover the exact declared key before
/// admitting the original Arrow schema and rows.
fn retained_physical_relation<'a>(
    registry: &'a pse_schema::Registry,
    scope: &str,
) -> Result<&'a pse_schema::model::RelationSpec, WorkflowError> {
    let (qualified, version) = scope
        .rsplit_once('@')
        .ok_or_else(|| integrity("original physical relation locator lacks its version"))?;
    let family = registry
        .relation(qualified)
        .ok_or_else(|| contract("original physical relation unavailable"))?;
    let version = version
        .parse::<u32>()
        .map_err(|_| integrity("original physical relation version is invalid"))?;
    let key = pse_schema::model::RelationKey {
        version,
        ..family.key
    };
    let relation = registry
        .relation_by_key(key)
        .ok_or_else(|| contract("original physical relation version unavailable"))?;
    if relation.key.to_string() != scope {
        return Err(integrity(
            "original physical relation locator spelling differs",
        ));
    }
    Ok(relation)
}

impl RetainedDataServices {
    pub(super) async fn original_result_physical(
        &self,
        read: &ResultRead,
        cancel: &CancellationToken,
    ) -> Result<(PhysicalContext, String), WorkflowError> {
        let store = self.canonical_store()?;
        let response = super::connected_results::pooled_response_owner(
            &self.pool(),
            "result:physical-source-selection",
            pse_operations::canonical::MetadataResponse::Selection,
        )?;
        let sources = pse_operations::canonical::response_ownership_scope(
            response.clone(),
            store.analysis_run_sources(&read.run().key),
        )
        .await?;
        let revisions: Vec<Revision> = sources
            .into_iter()
            .filter(|source| {
                source.problem.starts_with("physical:rows:")
                    || source.problem.starts_with("physical:context:")
            })
            .collect();
        let contexts = revisions
            .iter()
            .filter(|source| source.problem.starts_with("physical:context:"))
            .collect::<Vec<_>>();
        let [receipt] = contexts.as_slice() else {
            return Err(contract("original physical context receipt unavailable"));
        };
        let expected = pse_ids::ContentHash::parse_hex(
            receipt
                .problem
                .strip_prefix("physical:context:")
                .ok_or_else(|| contract("physical receipt kind"))?,
        )
        .map_err(|e| contract(e.to_string()))?;
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        hash.str("original.result.physical.closure.v1")
            .u64(revisions.len() as u64);
        for revision in &revisions {
            hash.str(&revision.key)
                .str(&revision.problem)
                .u64(revision.sequence);
        }
        let closure = hash.finish_hash().to_string();
        let protection =
            super::physical_cache::protect_store(store, &self.pool(), &revisions, revisions.len() * 512).await?;
        let pool = self.pool();
        let mut receipt_seen = false;
        let mut batches: BTreeMap<
            pse_schema::model::RelationKey,
            BTreeMap<String, FieldCheckedBatch>,
        > = BTreeMap::new();
        for pin in protection.pins() {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let mut after = String::new();
            loop {
                let page_owner = super::connected_results::pooled_response_owner(
                    &self.pool(),
                    "result:physical-membership-page",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    page_owner.clone(),
                    store.membership_page(pin, None, &after),
                )
                .await?;
                if page.is_empty() {
                    break;
                }
                after = page
                    .last()
                    .ok_or_else(|| contract("physical membership page"))?
                    .key
                    .clone();
                for member in page {
                    let extent = store
                        .selected_object_extent(pin, &member.version)
                        .await?
                        .ok_or_else(|| contract("original physical object unavailable"))?;
                    let allocation =
                        MemoryConsumer::new("result:physical-source-decode").register(&pool);
                    allocation
                        .try_grow(extent.saturating_mul(8).saturating_add(4096))
                        .map_err(pse_engine::EngineError::from)?;
                    let allocation = pse_columnar::AllocationLease::new(allocation);
                    let object = pse_operations::canonical::response_ownership_scope(
                        allocation.clone(),
                        store.selected_object(pin, &member.version),
                    )
                    .await?
                    .ok_or_else(|| contract("original physical object unavailable"))?;
                    if pin.revision() == *receipt && object.kind != "physical:context:v1" {
                        return Err(integrity(
                            "original physical context receipt contains a non-context payload",
                        ));
                    }
                    if object.kind == "physical:context:v1" {
                        if pin.revision() != *receipt || receipt_seen {
                            return Err(integrity(
                                "original physical context receipt is misplaced or duplicated",
                            ));
                        }
                        let (version, actual): (u8, pse_ids::ContentHash) =
                            serde_json::from_slice(object.payload.as_slice())
                                .map_err(|e| integrity(e.to_string()))?;
                        if version != 1 || actual != expected || pin.revision() != *receipt {
                            return Err(integrity("original physical context receipt differs"));
                        }
                        receipt_seen = true;
                        continue;
                    }
                    if object.kind != "physical:rows:v1" {
                        return Err(integrity("original physical row interpretation differs"));
                    }
                    let spec = retained_physical_relation(self.registry(), &member.scope)?;
                    let mut stream = pse_columnar::ipc_stream::StreamReader::try_new(
                        std::io::Cursor::new(object.payload.as_slice()),
                        object.payload.len() as u64,
                        pse_columnar::ipc_stream::Limits {
                            message_bytes: pse_operations::canonical::PAYLOAD_BYTES,
                            metadata_bytes: pse_operations::canonical::PAYLOAD_BYTES,
                        },
                        pool.clone(),
                        cancel.clone(),
                    )
                    .map_err(|e| integrity(e.to_string()))?;
                    let mut count = 0;
                    while let Some(decoded) =
                        stream.next_batch().map_err(|e| integrity(e.to_string()))?
                    {
                        count += 1;
                        if count > 1 {
                            return Err(integrity("original physical block has multiple batches"));
                        }
                        let checked = FieldCheckedBatch::admit(
                            self.registry(),
                            spec,
                            decoded.batch.into_batch(),
                            &self.validation_context(),
                            cancel,
                        )
                        .map_err(super::relation)?
                        .with_export_owner(decoded.working_owner);
                        if batches
                            .entry(spec.key)
                            .or_default()
                            .insert(member.logical.clone(), checked)
                            .is_some()
                        {
                            return Err(integrity("duplicate original physical row coordinate"));
                        }
                    }
                    if count != 1 {
                        return Err(integrity("original physical block is empty"));
                    }
                }
            }
        }
        if !receipt_seen {
            return Err(contract(
                "original physical context receipt payload unavailable",
            ));
        }
        let batches = batches
            .into_iter()
            .map(|(key, chunks)| {
                let chunks = chunks.into_values().collect::<Vec<_>>();
                let spec = self
                    .registry()
                    .relation_by_key(key)
                    .ok_or_else(|| contract("original physical declaration unavailable"))?;
                let batch = FieldCheckedBatch::concat_reserved(
                    self.registry(),
                    spec,
                    &chunks,
                    &pool,
                    cancel,
                )
                .map_err(super::relation)?;
                Ok((key, batch))
            })
            .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
        let inventory = pse_admission::physical::PhysicalInventory::from_batches(
            &pool,
            self.registry(),
            batches,
            cancel,
        )
        .map_err(|e| WorkflowError::Engine(pse_engine::EngineError::Semantic(Arc::new(e))))?;
        let mut physical = PhysicalContext::admitted(Arc::new(inventory));
        if physical.identity() != expected {
            return Err(integrity(
                "original physical rows do not reconstruct the original context",
            ));
        }
        physical._source_owner = Some(protection);
        Ok((physical, closure))
    }
}

#[cfg(test)]
mod physical_relation_locator_unit {
    use super::*;

    #[test]
    fn physical_relation_locator_consumes_the_producer_versioned_scope() {
        let registry = pse_schema::registry().unwrap();
        for name in [
            "reference.units",
            "reference.quantity_types",
            "reference.unit_sets",
            "authored.packages",
        ] {
            let original = registry.relation(name).unwrap();
            let published_scope = original.key.to_string();
            assert!(registry.relation(&published_scope).is_none());
            let selected = retained_physical_relation(registry, &published_scope).unwrap();
            assert_eq!(selected.key, original.key);
            assert_eq!(selected.id, original.id);
        }
    }

    #[test]
    fn physical_relation_locator_refuses_missing_unknown_or_noncanonical_version() {
        let registry = pse_schema::registry().unwrap();
        let original = registry.relation("reference.units").unwrap();
        for scope in [
            original.key.qualified_name(),
            "reference.units@invalid".into(),
            "reference.units@4294967295".into(),
            format!("reference.units@0{}", original.key.version),
            format!("reference.absent@{}", original.key.version),
        ] {
            assert!(
                retained_physical_relation(registry, &scope).is_err(),
                "{scope}"
            );
        }
    }
}
