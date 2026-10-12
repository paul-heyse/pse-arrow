// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure preparation backfill from an offline snapshot. This module cannot connect,
//! write, release roots or activate a database. The effect owner must retain the
//! supervisor's actual closed-admission and physical-drain authority, the immutable
//! snapshot and borrower inventory through marker-last activation. A manifest is
//! derived evidence, never an offline permit.

use crate::{
    canonical::{CanonicalError, descriptor_for_product},
    canonical_staging::{
        BODY_CONTENT_KIND, ProductBlob, SELECTION_RECEIPT_KIND, SOURCE_BLOCK_BYTES,
    },
    generated::surreal as wire,
};
use pse_ids::{Frame, FramedHasher, derive_hash};
use pse_model::{
    SemanticFrame,
    generated::runtime::{
        canonical_body_contents::Row as Content, canonical_interpretations::Row as Marker,
        canonical_payload_blocks::Row as Block, canonical_product_associations::Row as Association,
        canonical_product_descriptors::Row as Descriptor, canonical_products::Row as Product,
        canonical_roots::Row as Root, canonical_selection_receipts::Row as Receipt,
        canonical_staged_edits::Row as StagedEdit, canonical_stages::Row as Stage,
        canonical_version_manifests::Row as Manifest,
        canonical_version_receipts::Row as VersionReceipt, canonical_versions::Row as Version,
    },
};
use std::{collections::BTreeMap, sync::Arc};

/// Reconstructed access selectors from exact original scientific IPC columns.
#[derive(Debug)]
pub struct HistoricalProjection {
    /// Complete current typed scalar keys; no scalar values are copied.
    pub rows: Vec<pse_model::generated::runtime::canonical_result_rows::Row>,
    /// Dense coverage independently reconstructed from original symbol/time columns.
    pub outputs: Vec<pse_model::generated::runtime::canonical_result_block_outputs::Row>,
}

/// Scientific decoding belongs to the retained native registry/validation owner.
/// The offline adapter supplies an original immutable batch and its exact metadata;
/// this owner must preflight IPC and derive typed locators from original columns.
pub trait HistoricalResultRebuilder: std::fmt::Debug + Send + Sync {
    /// Validate one original scientific block and derive its current typed scalar keys.
    /// Dense and non-indexed relations return no scalar locators.
    /// # Errors
    /// Unknown registry relation, malformed IPC, local admission or allocation refusal.
    fn rebuild(
        &self,
        relation: &str,
        batch: &pse_model::generated::runtime::canonical_result_batches::Row,
        metadata: &pse_model::generated::runtime::canonical_result_blocks::Row,
    ) -> Result<HistoricalProjection, CanonicalError>;
}

/// One original rooted publication and its exact historical transport bytes.
/// Historical rows, roots and blobs are never rewritten by the derived output.
#[derive(Debug)]
pub struct HistoricalProduct {
    /// Original immutable association, including its original key/revision/producer.
    pub product: Product,
    /// Original exact product root from the same offline snapshot.
    pub root: Root,
    /// Original combined blob, whose CanonicalProductV1 framing is checked.
    pub blob: Version,
}

/// Supervisor-bound inputs for the future effect executor. Every field must be
/// checked against its retained offline authority immediately before any effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivationBinding {
    /// Stable operator-created migration identity, unchanged on resume.
    pub migration: String,
    /// Persistent store UUID recorded in the first durable migration manifest.
    pub issuing_store: String,
    /// Original reader/schema fence; never replaced before all checks finish.
    pub old_schema_digest: String,
    /// Exact immutable snapshot identity established by its snapshot owner.
    pub snapshot: String,
    /// Exact inventory of outstanding borrowers, including retained exports.
    pub borrowers: String,
}

/// Exact deterministic records ready for idempotent compare-or-create. The future
/// executor verifies existing rows byte-for-byte, verifies all rooted ACK oracles,
/// verifies that every originally rooted publication is represented (unrooted
/// archive rows remain untouched), then installs `marker` last. Interruption before that leaves the old marker in
/// charge. Repeating this derivation with the same binding produces the same rows.
#[derive(Debug)]
pub struct PreparationBackfill {
    /// Original offline authority identities; this is not authority itself.
    pub binding: ActivationBinding,
    /// Semantic identity of the exact old rows/roots/blob bytes actually checked.
    pub source_digest: String,
    /// New immutable body records, deduplicated by full equality.
    pub contents: Vec<Content>,
    /// New immutable complete selection records, deduplicated by full equality.
    pub receipts: Vec<Receipt>,
    /// Original publication keys/revisions retained in derived associations.
    pub associations: Vec<Association>,
    /// Fully checked descriptor points, choosing the old lowest-key rooted oracle.
    pub descriptors: Vec<Descriptor>,
    /// New physical transport manifests only; original manifests remain unchanged.
    pub manifests: Vec<Manifest>,
    /// Permanent collision receipts for the new physical versions.
    pub version_receipts: Vec<VersionReceipt>,
    /// New bounded transport blocks; original historical blocks remain unchanged.
    pub blocks: Vec<Block>,
    /// Activated derived frontiers, keeping orphan reclamation reachable after roots drop.
    pub stages: Vec<Stage>,
    /// Two exact child versions per derived frontier; original stage rows stay unchanged.
    pub staged_edits: Vec<StagedEdit>,
    /// Target marker. Installation is the final effect, not part of derivation.
    pub marker: Marker,
    _owner: Arc<dyn std::fmt::Debug + Send + Sync>,
}

fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn insert_exact<T: PartialEq>(
    rows: &mut BTreeMap<String, T>,
    key: &str,
    row: T,
) -> Result<(), CanonicalError> {
    if let Some(previous) = rows.get(key) {
        if previous != &row {
            return Err(invalid("offline backfill immutable identity collision"));
        }
    } else {
        rows.insert(key.into(), row);
    }
    Ok(())
}
impl PreparationBackfill {
    /// Conservative output/scratch extent. The caller retains its actual lease in
    /// `derive`; the input snapshot's allocation and storage owners remain separate.
    pub fn allocation_bytes(products: &[HistoricalProduct]) -> Result<usize, CanonicalError> {
        products.iter().try_fold(4096usize, |total, item| {
            total
                .checked_add(
                    item.blob
                        .payload
                        .len()
                        .checked_mul(70)
                        .ok_or(CanonicalError::PayloadLimit)?,
                )
                .and_then(|n| n.checked_add(item.product.request.len().checked_mul(8)?))
                .and_then(|n| n.checked_add(512 * 1024))
                .ok_or(CanonicalError::PayloadLimit)
        })
    }
    /// Validate original bytes and roots and derive records, without any effects.
    /// The binding is supplied by an authentic supervisor/snapshot owner. Passing
    /// an arbitrary binding here cannot authorize application of this manifest.
    pub fn derive(
        binding: ActivationBinding,
        mut products: Vec<HistoricalProduct>,
        owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<Self, CanonicalError> {
        if binding.migration.is_empty()
            || binding.migration.len() > 4096
            || binding.snapshot.is_empty()
            || binding.borrowers.is_empty()
            || binding.old_schema_digest.is_empty()
            || uuid::Uuid::parse_str(&binding.issuing_store).is_err()
        {
            return Err(invalid(
                "offline activation requires exact persistent migration/store/snapshot/borrower identities",
            ));
        }
        products.sort_by(|a, b| a.product.key.cmp(&b.product.key));
        let mut source = FramedHasher::new(Frame::CanonicalSchemaV1);
        source
            .str("offline-preparation-backfill.v1")
            .str(&binding.migration)
            .str(&binding.issuing_store)
            .str(&binding.old_schema_digest)
            .str(&binding.snapshot)
            .str(&binding.borrowers);
        let mut contents = BTreeMap::new();
        let mut receipts = BTreeMap::new();
        let mut associations = BTreeMap::new();
        let mut descriptors = BTreeMap::<String, Descriptor>::new();
        let mut stages = BTreeMap::new();
        let mut staged_edits = BTreeMap::new();
        let mut manifests = BTreeMap::<String, Manifest>::new();
        let mut version_receipts = BTreeMap::new();
        let mut blocks = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        for item in products {
            let original = &item.product;
            if !seen.insert(original.key.clone()) {
                return Err(invalid(
                    "offline snapshot repeats an original publication key",
                ));
            }
            if [
                &original.key,
                &original.problem,
                &original.revision,
                &original.producer,
                &original.interpretation,
                &item.root.key,
                &item.root.problem,
                &item.root.revision,
                &item.root.owner,
                &item.blob.key,
            ]
            .iter()
            .any(|v| v.len() > 4096)
            {
                return Err(CanonicalError::PayloadLimit);
            }
            if original.key != item.root.key
                || item.root.owner_kind != "product"
                || item.root.owner != original.key
                || item.root.problem != original.problem
                || item.root.revision != original.revision
                || !crate::canonical::historical_product_interpretation_supported(
                    &original.interpretation,
                )
                || !original.dependencies.is_empty()
            {
                return Err(invalid(
                    "offline historical product/root provenance differs",
                ));
            }
            original.frame(&mut source);
            item.root.frame(&mut source);
            item.blob.frame(&mut source);
            let old = ProductBlob::decode(original.payload.as_slice())?;
            let original_descriptor = serde_json::to_vec(&old)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
            if original_descriptor.as_slice() != original.payload.as_slice() {
                return Err(invalid(
                    "historical descriptor is not the original canonical encoding; old ACK equality cannot be preserved by inference",
                ));
            }
            if old.receipt.is_some()
                || item.blob.key != old.version
                || item.blob.kind != "pse.product-blob.v1"
                || item.blob.logical != "pse.product-blob.v1"
                || item.blob.interpretation != original.interpretation
            {
                return Err(invalid("offline historical combined blob identity differs"));
            }
            let (payload, selected) = old.split(item.blob.payload.as_slice())?;
            // Parse the complete original receipt; a byte hash cannot certify its
            // interpretation or serve as permission for a changed receiving context.
            crate::canonical_selection::validate_historical_dependency_encoding(
                selected,
                &original.interpretation,
            )?;
            let split = ProductBlob::from_parts(payload, selected, &original.producer)?;
            let receipt = split.receipt.as_ref().ok_or(CanonicalError::PayloadLimit)?;
            let mut offered = original.clone();
            offered.payload = serde_json::to_vec(&split)
                .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                .into();
            let mut descriptor = descriptor_for_product(&offered, &binding.issuing_store)?;
            let content = Content {
                key: split.version.clone(),
                version: split.version.clone(),
                digest: split.digest.clone(),
                bytes: payload.len() as u64,
                producer: original.producer.clone(),
                interpretation: wire::INTERPRETATION.into(),
                format: BODY_CONTENT_KIND.into(),
            };
            let selected_receipt = Receipt {
                key: receipt.version.clone(),
                version: receipt.version.clone(),
                digest: receipt.digest.clone(),
                bytes: selected.len() as u64,
                interpretation: wire::INTERPRETATION.into(),
                format: SELECTION_RECEIPT_KIND.into(),
            };
            insert_exact(&mut contents, &content.key.clone(), content)?;
            insert_exact(
                &mut receipts,
                &selected_receipt.key.clone(),
                selected_receipt,
            )?;
            let association = Association {
                key: original.key.clone(),
                issuing_store: binding.issuing_store.clone(),
                problem: original.problem.clone(),
                revision: original.revision.clone(),
                request: original.request.clone(),
                payload: offered.payload,
                dependencies: original.dependencies.clone(),
                content: split.version.clone(),
                receipt: receipt.version.clone(),
                descriptor: descriptor.key.clone(),
                producer: original.producer.clone(),
                interpretation: original.interpretation.clone(),
                historical_blob: Some(old.version),
            };
            insert_exact(&mut associations, &association.key.clone(), association)?;
            descriptor.association = Some(original.key.clone());
            if let Some(previous) = descriptors.get(&descriptor.key) {
                // Input is sorted by the original key, exactly matching the prior
                // ORDER BY key oracle. Compare every equality field independently.
                let mut compared = descriptor.clone();
                compared.association = previous.association.clone();
                if previous != &compared {
                    return Err(invalid(
                        "offline descriptor collision or old ACK oracle disagreement",
                    ));
                }
            } else {
                descriptors.insert(descriptor.key.clone(), descriptor);
            }
            let frontier = derive_hash(
                Frame::CanonicalEditV1,
                &[
                    b"offline-preparation-frontier.v1",
                    binding.migration.as_bytes(),
                    original.key.as_bytes(),
                    split.version.as_bytes(),
                    receipt.version.as_bytes(),
                ],
            )
            .to_hex();
            insert_exact(
                &mut stages,
                &frontier,
                Stage {
                    key: frontier.clone(),
                    problem: original.problem.clone(),
                    expected_head: None,
                    request_digest: frontier.clone(),
                    expires_at: 0,
                    closed: true,
                    activated: true,
                    edit_count: 2,
                    generation: 1,
                    abandoned: false,
                    cleanup_complete: false,
                },
            )?;
            for (ordinal, (kind, key, bytes)) in [
                (BODY_CONTENT_KIND, split.version.as_str(), payload),
                (SELECTION_RECEIPT_KIND, receipt.version.as_str(), selected),
            ]
            .into_iter()
            .enumerate()
            {
                let child_key = format!("{frontier}:{ordinal}");
                insert_exact(
                    &mut staged_edits,
                    &child_key,
                    StagedEdit {
                        key: child_key.clone(),
                        stage: frontier.clone(),
                        ordinal: ordinal as u64,
                        logical: kind.into(),
                        scope: kind.into(),
                        name: kind.into(),
                        version: Some(key.into()),
                    },
                )?;
                let digest = derive_hash(Frame::CanonicalPayloadV1, &[bytes]).to_hex();
                let len = bytes.len() as u64;
                let content_digest = derive_hash(
                    Frame::CanonicalVersionV1,
                    &[
                        kind.as_bytes(),
                        kind.as_bytes(),
                        wire::INTERPRETATION.as_bytes(),
                        &len.to_le_bytes(),
                        digest.as_bytes(),
                        b"[]",
                    ],
                )
                .to_hex();
                let mut manifest = Manifest {
                    key: key.into(),
                    logical: kind.into(),
                    kind: kind.into(),
                    interpretation: wire::INTERPRETATION.into(),
                    payload_digest: digest,
                    payload_len: len,
                    block_count: bytes.len().div_ceil(SOURCE_BLOCK_BYTES) as u64,
                    reference_count: 0,
                    creator_stage: frontier.clone(),
                    closed: true,
                };
                if let Some(previous) = manifests.get(key) {
                    manifest.creator_stage = previous.creator_stage.clone();
                }
                insert_exact(&mut manifests, key, manifest)?;
                insert_exact(
                    &mut version_receipts,
                    key,
                    VersionReceipt {
                        key: key.into(),
                        content_digest,
                    },
                )?;
                for (ordinal, part) in bytes.chunks(SOURCE_BLOCK_BYTES).enumerate() {
                    let key_block = format!("{key}:{ordinal}");
                    insert_exact(
                        &mut blocks,
                        &key_block.clone(),
                        Block {
                            key: key_block,
                            version: key.into(),
                            ordinal: ordinal as u64,
                            payload: part.to_vec().into(),
                            digest: derive_hash(Frame::CanonicalPayloadV1, &[part]).to_hex(),
                        },
                    )?;
                }
            }
        }
        let marker = Marker {
            key: "current".into(),
            interpretation: wire::INTERPRETATION.into(),
            schema_digest: wire::SCHEMA_DIGEST.into(),
            issuing_store: binding.issuing_store.clone(),
        };
        Ok(Self {
            binding,
            source_digest: source.finish_hash().to_hex(),
            contents: contents.into_values().collect(),
            receipts: receipts.into_values().collect(),
            associations: associations.into_values().collect(),
            descriptors: descriptors.into_values().collect(),
            manifests: manifests.into_values().collect(),
            version_receipts: version_receipts.into_values().collect(),
            blocks: blocks.into_values().collect(),
            stages: stages.into_values().collect(),
            staged_edits: staged_edits.into_values().collect(),
            marker,
            _owner: owner,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "pure offline snapshot controls fail on malformed fixture construction"
    )]
    use super::*;
    fn binding() -> ActivationBinding {
        ActivationBinding {
            migration: "migration-one".into(),
            issuing_store: "00000000-0000-4000-8000-000000000001".into(),
            old_schema_digest: "original-schema".into(),
            snapshot: "retained-snapshot".into(),
            borrowers: "exact-borrowers".into(),
        }
    }
    fn historical(key: &str, revision: &str, body: &[u8]) -> HistoricalProduct {
        historical_interpretation(key, revision, body, wire::INTERPRETATION)
    }
    fn historical_interpretation(
        key: &str,
        revision: &str,
        body: &[u8],
        interpretation: &str,
    ) -> HistoricalProduct {
        let selected =
            serde_json::to_vec(&serde_json::json!({"interpretation":interpretation,"premises":[]}))
                .unwrap();
        let (blob, edit) = crate::canonical_staging::product_blob(body, &selected).unwrap();
        let product = Product {
            key: key.into(),
            problem: "problem".into(),
            revision: revision.into(),
            request: b"request".to_vec().into(),
            payload: serde_json::to_vec(&blob).unwrap().into(),
            dependencies: Vec::new().into(),
            producer: "producer".into(),
            interpretation: interpretation.into(),
        };
        let root = Root {
            key: key.into(),
            problem: "problem".into(),
            revision: revision.into(),
            sequence: 1,
            owner_kind: "product".into(),
            owner: key.into(),
        };
        let mut version = edit.version.unwrap();
        version.interpretation = interpretation.into();
        HistoricalProduct {
            product,
            root,
            blob: version,
        }
    }
    #[test]
    fn offline_v3_history_keeps_original_receipt_interpretation_and_rooted_ack_oracle() {
        let first = historical_interpretation(
            "z-old",
            "old-revision",
            b"unchanged-body",
            "pse.substrate.v3",
        );
        let old_blob = first.blob.clone();
        let old_descriptor = ProductBlob::decode(first.product.payload.as_slice()).unwrap();
        let (_, original_selected) = old_descriptor.split(old_blob.payload.as_slice()).unwrap();
        let backfill = PreparationBackfill::derive(
            binding(),
            vec![
                first,
                historical_interpretation(
                    "a-old",
                    "original-revision",
                    b"unchanged-body",
                    "pse.substrate.v3",
                ),
                historical("v4", "current-revision", b"unchanged-body"),
            ],
            Arc::new(()),
        )
        .unwrap();
        assert_eq!(backfill.contents.len(), 1);
        assert_eq!(backfill.receipts.len(), 2);
        let association = backfill
            .associations
            .iter()
            .find(|row| row.key == "z-old")
            .unwrap();
        assert_eq!(association.interpretation, "pse.substrate.v3");
        assert_eq!(association.producer, "producer");
        assert_eq!(
            association.historical_blob.as_deref(),
            Some(old_blob.key.as_str())
        );
        let receipt_bytes = backfill
            .blocks
            .iter()
            .filter(|row| row.version == association.receipt)
            .flat_map(|row| row.payload.as_slice())
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(receipt_bytes, original_selected);
        assert_eq!(
            backfill
                .descriptors
                .iter()
                .find(|row| row.key == association.descriptor)
                .unwrap()
                .association
                .as_deref(),
            Some("a-old")
        );
        assert_eq!(backfill.marker.interpretation, wire::INTERPRETATION);
        assert_eq!(backfill.marker.issuing_store, binding().issuing_store);
    }
    #[test]
    fn offline_backfill_preserves_original_frames_and_lowest_rooted_ack_oracle() {
        let old = historical("z-origin", "revision-z", b"body");
        let original = old.product.clone();
        let original_blob = old.blob.clone();
        let derived = PreparationBackfill::derive(
            binding(),
            vec![
                old,
                historical("a-origin", "revision-a", b"body"),
                historical("other", "revision-c", b"other-body"),
            ],
            Arc::new(()),
        )
        .unwrap();
        assert_eq!(derived.contents.len(), 2);
        assert_eq!(derived.receipts.len(), 1);
        assert_eq!(derived.associations.len(), 3);
        let descriptor = derived
            .descriptors
            .iter()
            .find(|d| d.association.as_deref() == Some("a-origin"))
            .unwrap();
        let association = derived
            .associations
            .iter()
            .find(|a| a.key == original.key)
            .unwrap();
        assert_eq!(association.descriptor, descriptor.key);
        assert_eq!(association.revision, original.revision);
        assert_eq!(
            association.historical_blob.as_deref(),
            Some(original_blob.key.as_str())
        );
        assert_eq!(association.request, original.request);
        assert_eq!(association.producer, original.producer);
        let second = PreparationBackfill::derive(
            binding(),
            vec![
                historical("other", "revision-c", b"other-body"),
                historical("a-origin", "revision-a", b"body"),
                historical("z-origin", "revision-z", b"body"),
            ],
            Arc::new(()),
        )
        .unwrap();
        assert_eq!(derived.source_digest, second.source_digest);
        assert_eq!(derived.descriptors, second.descriptors);
        assert_eq!(derived.marker.issuing_store, binding().issuing_store);
    }
    #[test]
    fn offline_backfill_refuses_changed_bytes_roots_and_unknown_receipt_meaning() {
        let mut changed = historical("origin", "revision", b"body");
        changed.blob.payload = b"changed".to_vec().into();
        assert!(PreparationBackfill::derive(binding(), vec![changed], Arc::new(())).is_err());
        let mut wrong = historical("origin", "revision", b"body");
        wrong.root.revision = "other".into();
        assert!(PreparationBackfill::derive(binding(), vec![wrong], Arc::new(())).is_err());
        let mut wrong = historical("origin", "revision", b"body");
        wrong.product.interpretation = "unknown".into();
        assert!(PreparationBackfill::derive(binding(), vec![wrong], Arc::new(())).is_err());
        let mut wrong = binding();
        wrong.issuing_store = "transient-not-a-store-uuid".into();
        assert!(PreparationBackfill::derive(wrong, vec![], Arc::new(())).is_err());
    }
}
