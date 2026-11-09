// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sequential delivery verifies the exact bytes accepted by the selected sink.
use super::{ProjectBundleReader, WorkflowError, portable_payload};
use std::io::Write;

/// Completed original-member verification and accepted bytes, not destination durability.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ProjectMemberCopy {
    /// Exact original manifest member.
    pub member: String,
    /// All original bytes accepted by the sink.
    pub copied_bytes: u64,
    /// Original plain BLAKE3 commitment verified against the transferred bytes.
    pub digest: String,
}

/// A failed transfer retains its provisional accepted prefix and original cause.
#[derive(Debug, thiserror::Error)]
#[error(
    "project member {member} delivery failed after {accepted_bytes} provisional bytes: {source}"
)]
pub struct ProjectMemberCopyError {
    /// Exact selected member, including an unknown requested member.
    pub member: String,
    /// Bytes accepted by the sink; never a verified-member receipt.
    pub accepted_bytes: u64,
    /// Original read/write, cancellation or commitment refusal.
    #[source]
    pub source: std::io::Error,
}
pse_diagnostics::impl_diagnostic! {
    ProjectMemberCopyError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        facts.observe("member", pse_diagnostics::DiagnosticObservation::Text(this.member.clone()));
        facts.observe("accepted_bytes_decimal", pse_diagnostics::DiagnosticObservation::Text(this.accepted_bytes.to_string()));
        facts.observe("io_error_kind", pse_diagnostics::DiagnosticObservation::Text(format!("{:?}", this.source.kind())));
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for ProjectMemberCopyError {}
pub(super) fn workflow_error(error: ProjectMemberCopyError) -> WorkflowError {
    WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
}

impl ProjectBundleReader {
    /// Deliver one original contained member with bounded scratch and actual-byte verification.
    /// Prefixes remain provisional until this returns. No sink flush, sync or publication is
    /// performed; callers own those effects. Local descriptor checks are not snapshot isolation.
    pub fn copy_member_verified(
        &self,
        id: &str,
        output: &mut impl Write,
        cancel: &crate::CancelSource,
    ) -> Result<ProjectMemberCopy, ProjectMemberCopyError> {
        let failure = |accepted_bytes, source| ProjectMemberCopyError {
            member: id.into(),
            accepted_bytes,
            source,
        };
        let checkpoint = || {
            if cancel.token().is_cancelled() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "project member copy cancelled",
                ))
            } else {
                Ok(())
            }
        };
        let opened = (|| {
            checkpoint()?;
            let member = self
                .manifest
                .members
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "unindexed project member",
                    )
                })?;
            let digest =
                pse_ids::ContentHash::parse_hex(&member.digest).map_err(std::io::Error::other)?;
            if digest.to_hex() != member.digest {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid project member commitment",
                ));
            }
            let file = portable_payload::open_contained(&self.directory, &member.relative_path)?;
            Ok((file, member.copied_bytes, digest))
        })();
        let (file, bytes, digest) = opened.map_err(|source| failure(0, source))?;
        let transferred =
            portable_payload::transfer_member(file, output, bytes, Some(digest), checkpoint)
                .map_err(|error| failure(error.accepted_bytes, error.source))?;
        Ok(ProjectMemberCopy {
            member: id.into(),
            copied_bytes: transferred.bytes,
            digest: transferred.digest.to_hex(),
        })
    }
}
