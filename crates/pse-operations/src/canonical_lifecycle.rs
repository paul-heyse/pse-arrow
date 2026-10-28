// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public lifecycle explanations and exact original-owner release dispatch.
use crate::{
    canonical::{CanonicalError, CanonicalStore, protected_query},
    canonical_codec as codec,
    canonical_script::{Envelope, GuardedScript, ResultSlot},
};
use surrealdb_types::{Object, Value};

/// Each effect has a different original owner. Visibility is a project navigation
/// operation and physical file collection remains the existing file owner.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleKind {
    /// Withdraw a deliberate source-history root; this does not collect its bytes.
    SourceHistory,
    /// Retire run payloads and drive one original bounded result collection page.
    RunResults,
    /// Withdraw a study's result retention; runs remain independently explicit.
    StudyResults,
    /// Release settled run history and drive one original compaction page.
    RunHistory,
    /// Release settled study history and drive one original compaction page.
    StudyHistory,
    /// Release an unreferenced, noncurrent raw draft version, retaining its summary.
    DraftVersion,
    /// Observe exact evidence ownership. Release requires the original artifact intent.
    EvidenceArtifact,
}
impl LifecycleKind {
    fn name(self) -> &'static str {
        match self {
            Self::SourceHistory => "source_history",
            Self::RunResults => "run_results",
            Self::StudyResults => "study_results",
            Self::RunHistory => "run_history",
            Self::StudyHistory => "study_history",
            Self::DraftVersion => "draft_version",
            Self::EvidenceArtifact => "evidence_artifact",
        }
    }
    fn from_name(name: &str) -> Result<Self, CanonicalError> {
        match name {
            "source_history" => Ok(Self::SourceHistory),
            "run_results" => Ok(Self::RunResults),
            "study_results" => Ok(Self::StudyResults),
            "run_history" => Ok(Self::RunHistory),
            "study_history" => Ok(Self::StudyHistory),
            "draft_version" => Ok(Self::DraftVersion),
            "evidence_artifact" => Ok(Self::EvidenceArtifact),
            _ => Err(codec::CodecError::Domain("lifecycle effect kind").into()),
        }
    }
}
/// Explicit target selection. An explanation never implies consent to release.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LifecycleTarget {
    /// Original owner/effect, separate from project visibility and file collection.
    pub kind: LifecycleKind,
    /// Exact original owner identity.
    pub key: String,
}
/// Current owner coordinates, rechecked in the same transaction as release.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LifecycleSelection {
    /// Exact authority; another installation cannot consume this selection.
    pub issuing_store: String,
    /// Original effect and key.
    pub target: LifecycleTarget,
    /// Immutable source revision, never a newer source substitution.
    pub revision: String,
    /// Original current run/study/draft generation or source sequence.
    pub generation: u64,
    /// Project generation for a raw draft-version release.
    pub project_generation: Option<u64>,
    /// Current editing-head generation for a raw draft-version release.
    pub draft_generation: Option<u64>,
}
/// One observed original retention owner; it is not a derived index reference.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LifecycleBlocker {
    /// Actual owner/protection category.
    pub kind: String,
    /// Original owner's exact identity.
    pub identity: String,
}
/// Byte categories never imply filesystem space has been reclaimed.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct LifecycleBytes {
    /// Known logical payload extent; absent for unsupported or unbounded metadata.
    pub logical: Option<u64>,
    /// Actual copied extent is known by an export/backup receipt, not retention metadata.
    pub copied: Option<u64>,
    /// Physical allocated space requires the host resource owner's independent observation.
    pub physical: Option<u64>,
    /// No engine row deletion establishes physical reclaimed space.
    pub reclaimed: Option<u64>,
}
/// Bounded current observations. The original owner is the final release authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LifecycleExplanation {
    /// Exact checked selection; retaining this document grants no deletion capability.
    pub selection: LifecycleSelection,
    /// Actual observed references, readers or unsettled effects.
    pub blockers: Vec<LifecycleBlocker>,
    /// False: these bounded observations do not enumerate every original-owner predicate.
    pub blockers_complete: bool,
    /// Logical, copied, physical and reclaimed extents, with unknown values explicit.
    pub bytes: LifecycleBytes,
    /// Raw draft detail was released while the original version summary remains.
    pub detail_released: bool,
    /// Evidence artifacts require release_artifact_intent and collect_artifact with
    /// the exact original immutable intent; an observation key cannot release them.
    pub requires_original_artifact_intent: bool,
}
/// Result of the explicit effect, separate from any earlier explanation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRelease {
    /// Exact selected owner and effect.
    pub selection: LifecycleSelection,
    /// Whether this original bounded release/cleanup operation has finished.
    pub complete: bool,
    /// Always unknown here; original physical collection has its own owner.
    pub physical_reclaimed_bytes: Option<u64>,
}
fn object(value: Value) -> Result<Object, CanonicalError> {
    if let Value::Object(value) = value {
        Ok(value)
    } else {
        Err(codec::CodecError::Domain("lifecycle object").into())
    }
}
fn optional_uint(row: &mut Object, key: &'static str) -> Result<Option<u64>, CanonicalError> {
    match row.remove(key) {
        None | Some(Value::None | Value::Null) => Ok(None),
        Some(value) => Ok(Some(codec::decode_uint(value)?)),
    }
}
fn decode_selection(mut row: Object) -> Result<LifecycleSelection, CanonicalError> {
    let kind =
        LifecycleKind::from_name(&codec::decode_string(codec::required(&mut row, "kind")?)?)?;
    Ok(LifecycleSelection {
        issuing_store: codec::decode_string(codec::required(&mut row, "issuing_store")?)?,
        target: LifecycleTarget {
            kind,
            key: codec::decode_string(codec::required(&mut row, "key")?)?,
        },
        revision: codec::decode_string(codec::required(&mut row, "revision")?)?,
        generation: codec::decode_uint(codec::required(&mut row, "generation")?)?,
        project_generation: optional_uint(&mut row, "project_generation")?,
        draft_generation: optional_uint(&mut row, "draft_generation")?,
    })
}
impl LifecycleSelection {
    fn encode(&self) -> Result<Object, CanonicalError> {
        let mut row = Object::new();
        row.insert("issuing_store", self.issuing_store.clone());
        row.insert("kind", self.target.kind.name());
        row.insert("key", self.target.key.clone());
        row.insert("revision", self.revision.clone());
        row.insert("generation", codec::encode_uint(self.generation)?);
        row.insert(
            "project_generation",
            self.project_generation
                .map(codec::encode_uint)
                .transpose()?
                .unwrap_or(Value::None),
        );
        row.insert(
            "draft_generation",
            self.draft_generation
                .map(codec::encode_uint)
                .transpose()?
                .unwrap_or(Value::None),
        );
        Ok(row)
    }
    fn validate(&self, store: &CanonicalStore) -> Result<(), CanonicalError> {
        if self.issuing_store != store.issuing_store()? {
            return Err(CanonicalError::OperationReused);
        }
        if self.target.key.is_empty()
            || self.target.key.len() > 4096
            || self.revision.is_empty()
            || self.revision.len() > 4096
        {
            return Err(CanonicalError::PayloadLimit);
        }
        Ok(())
    }
}
impl CanonicalStore {
    /// Select exact current owner coordinates. This reads metadata and acquires no payload.
    pub async fn lifecycle_select(
        &self,
        target: &LifecycleTarget,
    ) -> Result<LifecycleSelection, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            if target.key.is_empty() || target.key.len()>4096 {return Err(CanonicalError::PayloadLimit);}
            let mut response=protected_query("canonical_lifecycle::select",|| Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("", "fn::pse_lifecycle_v1::selection($pse_rpc_expires_at,$issuer,$kind,$key)")?)
                .bind(("issuer",self.issuing_store()?.to_owned())).bind(("kind",target.kind.name())).bind(("key",target.key.clone())))).await?;
            let row=ResultSlot::<Option<Object>>::returned().take(&mut response)?.ok_or(CanonicalError::IncompleteResponse)?;
            decode_selection(row)
        }).await
    }
    /// Explain observed original protection and references without changing their ownership.
    pub async fn lifecycle_explain(
        &self,
        selection: &LifecycleSelection,
    ) -> Result<LifecycleExplanation, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                selection.validate(self)?;
                let encoded = selection.encode()?;
                let mut response = protected_query("canonical_lifecycle::explain", || {
                    Ok(self
                        .db
                        .script(
                            GuardedScript::new(Envelope::DeadlineOnly)
                                .returning::<Option<Object>>(
                                    "",
                                    "fn::pse_lifecycle_v1::explain($pse_rpc_expires_at,$selection)",
                                )?,
                        )
                        .bind(("selection", encoded.clone())))
                })
                .await?;
                let mut row = ResultSlot::<Option<Object>>::returned()
                    .take(&mut response)?
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let current = decode_selection(object(codec::required(&mut row, "selection")?)?)?;
                let Value::Array(raw) = codec::required(&mut row, "blockers")? else {
                    return Err(CanonicalError::IncompleteResponse);
                };
                let blockers = raw
                    .into_iter()
                    .map(|v| {
                        let mut r = object(v)?;
                        Ok(LifecycleBlocker {
                            kind: codec::decode_string(codec::required(&mut r, "kind")?)?,
                            identity: codec::decode_string(codec::required(&mut r, "identity")?)?,
                        })
                    })
                    .collect::<Result<Vec<_>, CanonicalError>>()?;
                Ok(LifecycleExplanation {
                    selection: current,
                    blockers,
                    blockers_complete: codec::decode_boolean(codec::required(
                        &mut row,
                        "blockers_complete",
                    )?)?,
                    bytes: LifecycleBytes {
                        logical: optional_uint(&mut row, "logical_bytes")?,
                        ..LifecycleBytes::default()
                    },
                    detail_released: codec::decode_boolean(codec::required(
                        &mut row,
                        "detail_released",
                    )?)?,
                    requires_original_artifact_intent: selection.target.kind
                        == LifecycleKind::EvidenceArtifact,
                })
            })
            .await
    }
    /// Explicit exact original-owner release. A prior explanation is never consumed as a grant.
    /// Repeat to resume bounded original cleanup; this never claims physical file reclamation.
    pub async fn lifecycle_release(
        &self,
        selection: &LifecycleSelection,
    ) -> Result<LifecycleRelease, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            selection.validate(self)?;
            if selection.target.kind==LifecycleKind::EvidenceArtifact {
                return Err(CanonicalError::Configuration("evidence release requires the original Artifact intent through release_artifact_intent and collect_artifact".into()));
            }
            self.ensure_writes()?; let encoded=selection.encode()?;
            let mut response=protected_query("canonical_lifecycle::release",|| Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>("", "fn::pse_lifecycle_v1::release($pse_rpc_expires_at,$selection)")?).bind(("selection",encoded.clone())))).await?;
            let mut row=ResultSlot::<Option<Object>>::returned().take(&mut response)?.ok_or(CanonicalError::IncompleteResponse)?;
            Ok(LifecycleRelease {selection:selection.clone(),complete:codec::decode_boolean(codec::required(&mut row,"complete")?)?,physical_reclaimed_bytes:None})
        }).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    unsafe_code,
    reason = "owned synthetic terminal fixture exercises retention, not scientific qualification"
)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "isolated lifecycle fixtures assert exact original-owner outcomes"
)]
mod canonical_lifecycle_tests {
    use super::*;
    use crate::{
        canonical::{CanonicalOptions, bounded_query},
        canonical_execution::{RunRequest, SeedEligibility, TerminalClass},
        canonical_projects::ProjectNavigation,
    };
    use std::{path::Path, time::Duration};
    async fn fixture() -> CanonicalStore {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("supervised disposable fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_lifecycle_{}", uuid::Uuid::new_v4().simple());
        crate::testing::canonical_fixture_with_options(&options, true).unwrap()
    }
    #[tokio::test]
    async fn referenced_source_stale_and_foreign_selections_cannot_release() {
        let store = fixture().await;
        let source = store
            .edit("source", None, "source-revision", &[])
            .await
            .unwrap();
        store
            .create_project("create", "project", "Example")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let intent = store
            .project_fork_intent(&project, "branch", "branch-problem", &source, "fork")
            .unwrap();
        let head = store.edit_project(&intent, &[]).await.unwrap();
        let selection = store
            .lifecycle_select(&LifecycleTarget {
                kind: LifecycleKind::SourceHistory,
                key: head.key.clone(),
            })
            .await
            .unwrap();
        let explanation = store.lifecycle_explain(&selection).await.unwrap();
        assert!(
            explanation
                .blockers
                .iter()
                .any(|b| b.kind == "retained_source_owner")
        );
        assert!(store.lifecycle_release(&selection).await.is_err());
        let mut foreign = selection.clone();
        foreign.issuing_store = uuid::Uuid::new_v4().to_string();
        assert!(store.lifecycle_release(&foreign).await.is_err());
        let mut stale = selection.clone();
        stale.generation += 1;
        assert!(store.lifecycle_explain(&stale).await.is_err());
        assert!(store.lifecycle_release(&stale).await.is_err());
        assert!(store.revision(&head.key).await.unwrap().is_some());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn borrowed_result_release_rechecks_current_owner_and_reports_unknown_physical_bytes() {
        let store = fixture().await;
        let revision = store
            .edit("result-source", None, "result-revision", &[])
            .await
            .unwrap();
        let request = RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: "run".into(),
            revision,
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        store.begin_run(&request).await.unwrap();
        let fence = store
            .claim_run("run", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        store
            .append_result_batch(
                &fence,
                "append",
                "values",
                0,
                crate::canonical_execution::ResultPayload::new(vec![17]).unwrap(),
                1,
            )
            .await
            .unwrap();
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture admits its own one-byte synthetic terminal payload.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal",
                    TerminalClass::Partial,
                    &[1],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        let selection = store
            .lifecycle_select(&LifecycleTarget {
                kind: LifecycleKind::RunResults,
                key: "run".into(),
            })
            .await
            .unwrap();
        let unborrowed = store.lifecycle_explain(&selection).await.unwrap();
        assert!(unborrowed.bytes.physical.is_none());
        let reader = store.read_results("run", fence.attempt()).await.unwrap();
        let payload = store
            .result_payload(&reader, &reader.sets()[0].key, 0)
            .await
            .unwrap();
        let borrowed = store.lifecycle_explain(&selection).await.unwrap();
        assert!(
            borrowed
                .blockers
                .iter()
                .any(|b| b.kind == "active_result_borrower")
        );
        assert!(
            store.lifecycle_release(&selection).await.is_err(),
            "an earlier unborrowed explanation is no grant"
        );
        drop(reader);
        assert!(
            store.lifecycle_release(&selection).await.is_err(),
            "escaped payload retains the borrower"
        );
        drop(payload);
        let mut released = None;
        for _ in 0..64 {
            match store.lifecycle_release(&selection).await {
                Ok(receipt) => {
                    let complete = receipt.complete;
                    released = Some(receipt);
                    if complete {
                        break;
                    }
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        }
        let receipt = released.expect("drained original owner permits release");
        assert!(receipt.complete);
        assert_eq!(receipt.physical_reclaimed_bytes, None);
        assert!(
            store.canonical_run("run").await.unwrap().is_some(),
            "original summary survives raw release"
        );
        assert!(store.read_results("run", fence.attempt()).await.is_err());
        let history = store
            .lifecycle_select(&LifecycleTarget {
                kind: LifecycleKind::RunHistory,
                key: "run".into(),
            })
            .await
            .unwrap();
        let mut complete = false;
        for _ in 0..16 {
            if store.lifecycle_release(&history).await.unwrap().complete {
                complete = true;
                break;
            }
        }
        assert!(complete, "bounded history detail is collected");
        assert!(
            store.lifecycle_release(&history).await.unwrap().complete,
            "lost final reply settles original retirement after header deletion"
        );
        assert!(
            store
                .lifecycle_explain(&history)
                .await
                .unwrap()
                .detail_released
        );
        let mut stale = history.clone();
        stale.generation += 1;
        assert!(
            store.lifecycle_release(&stale).await.is_err(),
            "retained retirement authenticates original generation"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn evidence_observation_requires_original_intent_and_preserves_owners() {
        use crate::canonical_artifacts::{artifact_block, seal_artifact_request};
        let store = fixture().await;
        let revision = store
            .edit("evidence", None, "evidence-revision", &[])
            .await
            .unwrap();
        let mut intent = store
            .new_artifact(&revision, "research-evidence", vec![1], 1, 3)
            .await
            .unwrap();
        let blocks = vec![artifact_block(
            &intent.key,
            0,
            "evidence",
            vec![1, 2, 3],
            "original",
            1,
        )];
        seal_artifact_request(
            &mut intent,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "audit",
            "scientific-owner",
        );
        store
            .persist_artifact(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "audit",
                "scientific-owner",
            )
            .await
            .unwrap();
        let selection = store
            .lifecycle_select(&LifecycleTarget {
                kind: LifecycleKind::EvidenceArtifact,
                key: intent.key.clone(),
            })
            .await
            .unwrap();
        let read = store.read_artifact(&intent.key).await.unwrap();
        let explanation = store.lifecycle_explain(&selection).await.unwrap();
        assert!(explanation.requires_original_artifact_intent);
        assert_eq!(explanation.bytes.logical, Some(3));
        assert!(
            explanation
                .blockers
                .iter()
                .any(|b| b.kind == "retained_artifact_obligation")
        );
        assert!(
            explanation
                .blockers
                .iter()
                .any(|b| b.kind == "active_artifact_borrower")
        );
        assert!(store.lifecycle_release(&selection).await.is_err());
        assert!(
            store.artifact(&intent.key).await.is_ok(),
            "an observation key cannot release original evidence"
        );
        read.release().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn draft_release_preserves_summary_and_visibility_never_releases_payload() {
        let store = fixture().await;
        let origin = store
            .edit("origin", None, "origin-revision", &[])
            .await
            .unwrap();
        store
            .create_project("create", "project", "Drafts")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let intent = store
            .project_fork_intent(&project, "branch", "branch-owner", &origin, "fork")
            .unwrap();
        let head = store.edit_project(&intent, &[]).await.unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("branch").await.unwrap().unwrap();
        store
            .save_project_draft(
                "first",
                &project,
                &branch,
                "draft",
                &head.key,
                None,
                b"original malformed",
                None,
            )
            .await
            .unwrap();
        let original = store
            .lifecycle_select(&LifecycleTarget {
                kind: LifecycleKind::DraftVersion,
                key: "first".into(),
            })
            .await
            .unwrap();
        assert!(
            store
                .lifecycle_explain(&original)
                .await
                .unwrap()
                .blockers
                .iter()
                .any(|b| b.kind == "current_draft_head")
        );
        assert!(store.lifecycle_release(&original).await.is_err());
        store
            .save_project_draft(
                "second",
                &project,
                &branch,
                "draft",
                &head.key,
                Some(1),
                b"new malformed",
                None,
            )
            .await
            .unwrap();
        assert!(
            store.lifecycle_release(&original).await.is_err(),
            "draft generation changed"
        );
        let selected = store.lifecycle_select(&original.target).await.unwrap();
        store
            .project_navigate(
                "hide",
                &project,
                &ProjectNavigation::ProjectVisibility { deleted: true },
            )
            .await
            .unwrap();
        assert_eq!(
            store
                .project_draft_version("first")
                .await
                .unwrap()
                .unwrap()
                .content
                .as_slice(),
            b"original malformed"
        );
        assert!(
            store.lifecycle_release(&selected).await.is_err(),
            "project generation changed"
        );
        let selected = store.lifecycle_select(&original.target).await.unwrap();
        assert!(store.lifecycle_release(&selected).await.unwrap().complete);
        let summary = store.project_draft_version("first").await.unwrap().unwrap();
        assert!(summary.retired);
        assert!(summary.content.is_empty());
        assert_eq!(summary.base_revision, head.key);
        assert!(
            store
                .lifecycle_explain(&selected)
                .await
                .unwrap()
                .detail_released
        );
        let mut rows = bounded_query(
            store
                .db
                .query("SELECT key FROM canonical_roots WHERE key='project-draft-version:first';"),
        )
        .await
        .unwrap();
        assert!(rows.take::<Vec<Object>>(0).unwrap().is_empty());
        store.remove_isolated_fixture().await.unwrap();
    }
}
