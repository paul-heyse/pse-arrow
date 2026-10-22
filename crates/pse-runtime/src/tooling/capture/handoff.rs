// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional copied scientific context at a cooperating joined owner. Never reconstruction.
use crate::observation_detail::CaptureError;
use crate::{workflow::{ReviewContextDocument, ReviewContextRequest, RunResult}};
use crate::tooling::{frozen_inspection::FrozenEvidenceSet};
use pse_diagnostics::observation::OperationId;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Component, Path, PathBuf},
    time::Duration,
};

/// Additional selected descriptive output; absent means no handoff retention.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewHandoffRequest {
    /// Existing callback-free descriptive selection.
    pub context: ReviewContextRequest,
    /// Finite total serialized member allowance, independent of detail output.
    pub maximum_bytes: u64,
    /// Finite number of immutable supplied evidence documents.
    pub maximum_members: usize,
}
/// Independent admission; an ACK does not establish any terminal context was retained.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewHandoffAcknowledgement {
    /// Actual requested descriptive selection.
    pub request: ReviewHandoffRequest,
    /// Only this optional handoff was admitted.
    pub accepted: bool,
    /// Explicit final receipt within the selected detail output, not a native locator.
    pub receipt: PathBuf,
    /// Admission refusal, independent of detail and science.
    pub error: Option<String>,
}
/// The handoff's own completion, independent of source detail and scientific completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewHandoffState {
    /// Selected producer can still publish members.
    Active,
    /// Producer joined with all requested publications completed.
    Complete,
    /// Producer stopped or a selected publication failed.
    Incomplete,
}
/// One fully published immutable copied evidence set.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewHandoffMember {
    /// Receipt-relative contained file, never an original source path.
    pub path: String,
    /// Actual finalized file bytes.
    pub bytes: u64,
    /// Original invocation operation when one exists.
    pub operation: Option<OperationId>,
    /// Original semantic run, absent before run establishment.
    pub run: Option<String>,
    /// Exact acknowledged durable locators, absent for ephemeral work.
    pub canonical_run: Option<String>,
    /// Exact original durable attempt key when acknowledged.
    pub canonical_attempt: Option<String>,
}
/// Data-only membership for explicitly requested producer output.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewHandoffReceipt {
    /// Receipt codec version.
    pub version: Version<1>,
    /// Cooperating construction owner, not necessarily the joined invocation.
    pub operation: OperationId,
    /// Actual independently selected descriptive output.
    pub request: ReviewHandoffRequest,
    /// Independent handoff completion state.
    pub state: ReviewHandoffState,
    /// Only fully finalized contained members.
    pub members: Vec<ReviewHandoffMember>,
    /// Bounded descriptive publication failures.
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub(super) struct ReviewHandoffSession {
    directory: PathBuf,
    receipt: ReviewHandoffReceipt,
    expiry: Option<u64>,
    written: u64,
    pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
}
fn invalid(message: impl Into<String>) -> CaptureError {
    CaptureError::Invalid(message.into())
}
impl ReviewHandoffSession {
    pub(super) fn create(
        output: &Path,
        operation: OperationId,
        request: ReviewHandoffRequest,
        expiry: Option<u64>,
    ) -> Result<Self, CaptureError> {
        if request.maximum_bytes == 0
            || request.maximum_bytes > (1 << 30)
            || request.maximum_members == 0
            || request.maximum_members > 1024
        {
            return Err(invalid(
                "review handoff requires finite bytes and at most 1024 members",
            ));
        }
        // Check the selected context through its existing factual admission without callbacks.
        ReviewContextDocument::pre_handle_failure(
            request.context.clone(),
            Some(operation),
            "handoff admission".into(),
            Vec::new(),
        )
        .map_err(|error| invalid(error.to_string()))?;
        let directory = output.join("review");
        std::fs::create_dir(&directory)?;
        let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> = std::sync::Arc::new(
            pse_columnar::GreedyMemoryPool::new(request.context.maximum_bytes.saturating_mul(8)),
        );
        let session = Self {
            directory,
            pool,
            receipt: ReviewHandoffReceipt {
                version: Version,
                operation,
                request,
                state: ReviewHandoffState::Active,
                members: Vec::new(),
                errors: Vec::new(),
            },
            expiry,
            written: 0,
        };
        session.persist()?;
        Ok(session)
    }
    pub(super) fn acknowledgement(&self) -> ReviewHandoffAcknowledgement {
        ReviewHandoffAcknowledgement {
            request: self.receipt.request.clone(),
            accepted: true,
            receipt: self.directory.join("index.json"),
            error: None,
        }
    }
    fn persist(&self) -> Result<(), CaptureError> {
        let temporary = self.directory.join("index.pending");
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            let bytes =
                serde_json::to_vec(&self.receipt).map_err(|error| invalid(error.to_string()))?;
            if bytes.len() > 1 << 20 {
                return Err(invalid(
                    "review handoff receipt exceeds finite metadata allowance",
                ));
            }
            output.write_all(&bytes)?;
            output.flush()?;
            std::fs::rename(&temporary, self.directory.join("index.json"))?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temporary);
        }
        result
    }
    fn checkpoint(&self) -> Result<(), CaptureError> {
        if self.receipt.state != ReviewHandoffState::Active {
            return Err(invalid("review handoff is closed"));
        }
        if let Some(expiry) = self.expiry
            && crate::observation_detail::monotonic_ns().is_none_or(|now| now >= expiry)
        {
            return Err(invalid("review handoff original admission expired"));
        }
        if self.receipt.members.len() >= self.receipt.request.maximum_members {
            return Err(invalid("review handoff member allowance exhausted"));
        }
        Ok(())
    }
    fn working_owner(&self) -> Result<pse_columnar::MemoryReservation, CaptureError> {
        let owner = pse_columnar::MemoryConsumer::new("review:handoff-copy").register(&self.pool);
        owner
            .try_grow(self.receipt.request.context.maximum_bytes.saturating_mul(8))
            .map_err(|error| invalid(error.to_string()))?;
        Ok(owner)
    }
    pub(super) fn failure(&mut self, error: &CaptureError) {
        self.receipt.state = ReviewHandoffState::Incomplete;
        if self.receipt.errors.len() < 16 {
            self.receipt
                .errors
                .push(error.to_string().chars().take(512).collect());
        }
        let _ = self.persist();
    }
    pub(super) fn retain_result(
        &mut self,
        result: &std::sync::Arc<RunResult>,
    ) -> Result<ReviewHandoffReceipt, CaptureError> {
        self.checkpoint()?;
        if self.receipt.members.iter().any(|member| {
            member.run.as_deref() == Some(result.run_id.to_string().as_str())
                && member.canonical_attempt.as_deref() == result.canonical_attempt_key()
        }) {
            return Ok(self.receipt.clone());
        }
        let _allocation = self.working_owner()?;
        let context = result
            .review_context(&self.receipt.request.context)
            .map_err(|error| invalid(error.to_string()))?;
        let invocation = result
            .invocation_evidence()
            .map_err(|error| invalid(error.to_string()))?;
        let operation = invocation.operation;
        let canonical_run = invocation.canonical_run.clone();
        let canonical_attempt = invocation.canonical_attempt.clone();
        // Construct the copied set by decoding under the existing bounded factual codec.
        // This does not recreate source, native, or scientific allocation owners.
        let context_bytes =
            serde_json::to_vec(context.document()).map_err(|error| invalid(error.to_string()))?;
        let invocation_bytes =
            serde_json::to_vec(&*invocation).map_err(|error| invalid(error.to_string()))?;
        let mut evidence = FrozenEvidenceSet::default();
        evidence.contexts.push(
            crate::tooling::frozen_inspection::decode_document::<ReviewContextDocument, 1>(&context_bytes)
                .map_err(|error| invalid(error.to_string()))?,
        );
        evidence.engine_observations.extend(
            evidence.contexts[0]
                .engine_observations()
                .map_err(|error| invalid(error.to_string()))?,
        );
        evidence.invocations.push(
            crate::tooling::frozen_inspection::decode_document::<
                crate::invocation::InvocationEvidenceDocument,
                2,
            >(&invocation_bytes)
            .map_err(|error| invalid(error.to_string()))?,
        );
        if let Some(operation) = operation
            && let Some(reader) = crate::observation::process_diagnostics().reader(operation)
        {
            evidence
                .operations
                .push(reader.snapshot(Duration::from_secs(60)));
        }
        self.publish(
            &evidence,
            operation,
            Some(result.run_id.to_string()),
            canonical_run,
            canonical_attempt,
        )
    }
    pub(super) fn retain_context(
        &mut self,
        context: &ReviewContextDocument,
    ) -> Result<ReviewHandoffReceipt, CaptureError> {
        self.checkpoint()?;
        context
            .admit_handoff(&self.receipt.request.context)
            .map_err(|error| invalid(error.to_string()))?;
        let _allocation = self.working_owner()?;
        context
            .validate()
            .map_err(|error| invalid(error.to_string()))?;
        let bytes = serde_json::to_vec(context).map_err(|error| invalid(error.to_string()))?;
        let mut evidence = FrozenEvidenceSet::default();
        evidence.contexts.push(
            crate::tooling::frozen_inspection::decode_document::<ReviewContextDocument, 1>(&bytes)
                .map_err(|error| invalid(error.to_string()))?,
        );
        evidence.engine_observations.extend(
            evidence.contexts[0]
                .engine_observations()
                .map_err(|error| invalid(error.to_string()))?,
        );
        self.publish(
            &evidence,
            context.operation,
            context.run_id.map(|run| run.to_string()),
            None,
            None,
        )
    }
    fn publish(
        &mut self,
        evidence: &FrozenEvidenceSet,
        operation: Option<OperationId>,
        run: Option<String>,
        canonical_run: Option<String>,
        canonical_attempt: Option<String>,
    ) -> Result<ReviewHandoffReceipt, CaptureError> {
        let outcome = (|| {
            self.checkpoint()?;
            evidence
                .validate()
                .map_err(|error| invalid(error.to_string()))?;
            let name = format!("evidence-{:05}.json", self.receipt.members.len());
            let staging = self.directory.join(format!("{name}.pending"));
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            struct Limited<'a> {
                file: &'a mut std::fs::File,
                remaining: u64,
                written: u64,
                expiry: Option<u64>,
            }
            impl Write for Limited<'_> {
                fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                    if self.expiry.is_some_and(|expiry| {
                        crate::observation_detail::monotonic_ns().is_none_or(|now| now >= expiry)
                    }) {
                        return Err(std::io::Error::other(
                            "review handoff original admission expired",
                        ));
                    }
                    if bytes.len() as u64 > self.remaining {
                        return Err(std::io::Error::other(
                            "review handoff byte allowance exhausted",
                        ));
                    }
                    self.file.write_all(bytes)?;
                    self.remaining -= bytes.len() as u64;
                    self.written += bytes.len() as u64;
                    Ok(bytes.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    self.file.flush()
                }
            }
            let mut limited = Limited {
                file: &mut output,
                remaining: self
                    .receipt
                    .request
                    .maximum_bytes
                    .saturating_sub(self.written),
                written: 0,
                expiry: self.expiry,
            };
            serde_json::to_writer(&mut limited, evidence)
                .map_err(|error| invalid(error.to_string()))?;
            limited.flush()?;
            let bytes = limited.written;
            self.checkpoint()?;
            std::fs::hard_link(&staging, self.directory.join(&name))?;
            std::fs::remove_file(staging)?;
            self.written += bytes;
            self.receipt.members.push(ReviewHandoffMember {
                path: name,
                bytes,
                operation,
                run,
                canonical_run,
                canonical_attempt,
            });
            self.persist()?;
            Ok(self.receipt.clone())
        })();
        if let Err(error) = &outcome {
            self.failure(error);
        }
        outcome
    }
    pub(super) fn finish(&mut self, completed: bool) {
        if self.receipt.state == ReviewHandoffState::Active {
            self.receipt.state = if completed && !self.receipt.members.is_empty() {
                ReviewHandoffState::Complete
            } else {
                ReviewHandoffState::Incomplete
            };
        }
        let _ = self.persist();
    }
}

/// Import only the explicitly supplied handoff receipt's finalized contained members.
/// This never searches original paths or opens a scientific deployment.
pub fn read_review_handoff(path: &Path) -> Result<FrozenEvidenceSet, CaptureError> {
    let receipt = crate::tooling::frozen_inspection::read_document::<ReviewHandoffReceipt, 1>(path)
        .map_err(|error| invalid(error.to_string()))?;
    if receipt.members.len() > receipt.request.maximum_members
        || receipt.request.maximum_members > 1024
        || receipt.request.maximum_members == 0
        || receipt.request.maximum_bytes == 0
        || receipt.request.maximum_bytes > 1 << 30
    {
        return Err(invalid("invalid handoff member bound"));
    }
    let directory = path
        .parent()
        .ok_or_else(|| invalid("handoff receipt needs its selected directory"))?;
    let mut result = FrozenEvidenceSet::default();
    let mut total = 0u64;
    let mut seen = std::collections::BTreeSet::new();
    for member in &receipt.members {
        let relative = Path::new(&member.path);
        if relative.components().count() != 1
            || !relative
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            || !seen.insert(&member.path)
        {
            return Err(invalid("unsafe or duplicate handoff member"));
        }
        let selected = directory.join(relative);
        let metadata = std::fs::symlink_metadata(&selected)?;
        total = total
            .checked_add(member.bytes)
            .filter(|bytes| *bytes <= receipt.request.maximum_bytes)
            .ok_or_else(|| invalid("handoff exceeds selected byte allowance"))?;
        if total > crate::tooling::frozen_inspection::MAXIMUM_DOCUMENT_BYTES as u64 {
            return Err(invalid(
                "aggregate handoff exceeds frozen document admission before decoding",
            ));
        }
        if !metadata.is_file() || metadata.len() != member.bytes {
            return Err(invalid("handoff member is not its finalized regular file"));
        }
        let mut evidence =
            crate::tooling::frozen_inspection::read_document::<FrozenEvidenceSet, 2>(&selected)
                .map_err(|error| invalid(error.to_string()))?;
        if evidence.invocations.iter().any(|invocation| {
            Some(invocation.run_id.to_string()) != member.run
                || invocation.operation != member.operation
                || invocation.canonical_run != member.canonical_run
                || invocation.canonical_attempt != member.canonical_attempt
        }) {
            return Err(invalid(
                "handoff member invocation identity conflicts with receipt",
            ));
        }
        evidence
            .validate()
            .map_err(|error| invalid(error.to_string()))?;
        result.checkpoints.append(&mut evidence.checkpoints);
        result.operations.append(&mut evidence.operations);
        result.invocations.append(&mut evidence.invocations);
        result.contexts.append(&mut evidence.contexts);
        result.inventories.append(&mut evidence.inventories);
        result
            .engine_observations
            .append(&mut evidence.engine_observations);
    }
    result
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    Ok(result)
}

impl Drop for ReviewHandoffSession {
    fn drop(&mut self) {
        self.finish(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::ObservationHub;
    #[test]
    fn review_handoff_keeps_prehandle_context_without_science_and_relocates() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("handoff.prehandle");
        let request = ReviewHandoffRequest {
            context: ReviewContextRequest::default(),
            maximum_bytes: 1 << 20,
            maximum_members: 4,
        };
        let mut handoff = ReviewHandoffSession::create(
            directory.path(),
            operation.context().id(),
            request.clone(),
            None,
        )
        .unwrap();
        let context = ReviewContextDocument::pre_handle_failure(
            request.context,
            Some(operation.context().id()),
            "original admission failure".into(),
            Vec::new(),
        )
        .unwrap();
        handoff.retain_context(&context).unwrap();
        handoff.finish(true);
        let moved = directory.path().join("relocated");
        std::fs::rename(directory.path().join("review"), &moved).unwrap();
        let copied = read_review_handoff(&moved.join("index.json")).unwrap();
        assert_eq!(copied.contexts.len(), 1);
        assert_eq!(
            copied.contexts[0].failure.as_deref(),
            Some("original admission failure")
        );
        assert!(operation.context().is_active());
    }
    #[test]
    fn supplied_handoff_respects_its_admitted_selection_and_byte_bound() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("handoff.selection");
        let request = ReviewHandoffRequest {
            context: ReviewContextRequest {
                steps: vec![0],
                include_sources: false,
                include_native_records: false,
                maximum_bytes: 4096,
            },
            maximum_bytes: 1 << 20,
            maximum_members: 4,
        };
        let mut handoff = ReviewHandoffSession::create(
            directory.path(),
            operation.context().id(),
            request.clone(),
            None,
        )
        .unwrap();
        let wider = ReviewContextDocument::pre_handle_failure(
            ReviewContextRequest::default(),
            Some(operation.context().id()),
            "wider".into(),
            Vec::new(),
        )
        .unwrap();
        assert!(
            handoff
                .retain_context(&wider)
                .unwrap_err()
                .to_string()
                .contains("selection")
        );
        assert!(handoff.receipt.members.is_empty());
        let oversized = ReviewContextDocument::pre_handle_failure(
            ReviewContextRequest {
                maximum_bytes: 1 << 20,
                ..request.context.clone()
            },
            Some(operation.context().id()),
            "x".repeat(8192),
            Vec::new(),
        )
        .unwrap();
        assert!(handoff.retain_context(&oversized).is_err());
        assert!(handoff.receipt.members.is_empty());
        let narrow = ReviewContextDocument::pre_handle_failure(
            request.context,
            Some(operation.context().id()),
            "actual".into(),
            Vec::new(),
        )
        .unwrap();
        handoff.retain_context(&narrow).unwrap();
        assert_eq!(handoff.receipt.members.len(), 1);
        assert!(operation.context().is_active());
    }
    #[test]
    fn aggregate_handoff_refuses_before_decoding_oversized_members() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("handoff.aggregate");
        let request = ReviewHandoffRequest {
            context: ReviewContextRequest::default(),
            maximum_bytes: 1 << 30,
            maximum_members: 4,
        };
        let handoff =
            ReviewHandoffSession::create(directory.path(), operation.context().id(), request, None)
                .unwrap();
        let mut receipt = handoff.receipt.clone();
        let bytes = crate::tooling::frozen_inspection::MAXIMUM_DOCUMENT_BYTES as u64 + 1;
        let file = std::fs::File::create(handoff.directory.join("too-large.json")).unwrap();
        file.set_len(bytes).unwrap();
        receipt.members.push(ReviewHandoffMember {
            path: "too-large.json".into(),
            bytes,
            operation: None,
            run: None,
            canonical_run: None,
            canonical_attempt: None,
        });
        std::fs::write(
            handoff.directory.join("supplied.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        let error = read_review_handoff(&handoff.directory.join("supplied.json")).unwrap_err();
        assert!(error.to_string().contains("before decoding"));
    }
    #[test]
    fn review_handoff_refusal_and_unlisted_partial_bytes_do_not_change_source() {
        let directory = tempfile::tempdir().unwrap();
        let operation = ObservationHub::new(Default::default()).start("handoff.refused");
        let request = ReviewHandoffRequest {
            context: ReviewContextRequest::default(),
            maximum_bytes: 1,
            maximum_members: 1,
        };
        let mut handoff = ReviewHandoffSession::create(
            directory.path(),
            operation.context().id(),
            request.clone(),
            None,
        )
        .unwrap();
        let context = ReviewContextDocument::pre_handle_failure(
            request.context,
            Some(operation.context().id()),
            "original".into(),
            Vec::new(),
        )
        .unwrap();
        assert!(handoff.retain_context(&context).is_err());
        handoff.finish(true);
        let receipt: ReviewHandoffReceipt = crate::tooling::frozen_inspection::read_document::<_, 1>(
            &directory.path().join("review/index.json"),
        )
        .unwrap();
        assert_eq!(receipt.state, ReviewHandoffState::Incomplete);
        assert!(receipt.members.is_empty());
        assert!(!receipt.errors.is_empty());
        assert!(operation.context().is_active());
    }
}
