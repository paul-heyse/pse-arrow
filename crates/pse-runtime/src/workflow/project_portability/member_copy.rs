// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sequential delivery verifies the exact bytes accepted by the selected sink.
use super::{ProjectBundleReader, WorkflowError, portable_payload};
use std::io::{Read as _, Write};

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

struct Accepted<'a, W> {
    output: &'a mut W,
    hash: pse_ids::EncodingHasher,
    bytes: u64,
}
impl<W: Write> Write for Accepted<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("project accepted-byte count overflow"))?;
        let written = self.output.write(bytes)?;
        self.hash.update(&bytes[..written]);
        self.bytes += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
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
        let mut accepted = Accepted {
            output,
            hash: pse_ids::EncodingHasher::new(),
            bytes: 0,
        };
        let transfer = (|| -> std::io::Result<()> {
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
            let mut file =
                portable_payload::open_contained(&self.directory, &member.relative_path)?;
            let observe = |file: &std::fs::File| {
                pse_host::artifacts::descriptor_identity(file).map_err(std::io::Error::other)
            };
            let before = observe(&file)?;
            if before.bytes != member.copied_bytes {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "project member extent changed",
                ));
            }
            portable_payload::transfer_exact(
                &mut file,
                &mut accepted,
                member.copied_bytes,
                checkpoint,
            )?;
            let mut extra = [0];
            if file.read(&mut extra)? != 0
                || observe(&file)? != before
                || accepted.hash.clone().finish().content_hash().to_hex() != member.digest
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "consumed project member changed or differs from original commitment",
                ));
            }
            checkpoint()
        })();
        transfer.map_err(|source| ProjectMemberCopyError {
            member: id.into(),
            accepted_bytes: accepted.bytes,
            source,
        })?;
        Ok(ProjectMemberCopy {
            member: id.into(),
            copied_bytes: accepted.bytes,
            digest: accepted.hash.finish().content_hash().to_hex(),
        })
    }
}
