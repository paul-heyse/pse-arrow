// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native stage lookup, admission and conditional index publication.

use super::{OwnedStageHint, StageHint, StageInputs};
use crate::store::{
    operation::{PreparedStoreOperation, StoreCommand},
    publish::RelationDraft,
    sidecar::SidecarArtifact,
};
use crate::{Catalog, CatalogError, Snapshot};
use datafusion::{common::TableReference, logical_expr::lit};
use pse_ids::{CancellationToken, ContentHash};
use pse_schema::model::provider::{OperationPurpose, ProviderScope};
use std::sync::Arc;

/// One admitted output with its actual pass-record artifact.
pub type OpenedStageHint = (Arc<Snapshot>, SidecarArtifact);

impl Catalog {
    /// Resolve stage lookup hints through native policy and resource admission.
    /// # Errors
    /// Invalid encoding, allocation, cancellation or backend failure.
    pub async fn stage_hints(
        &self,
        key: ContentHash,
        cancel: &CancellationToken,
    ) -> Result<Vec<OwnedStageHint>, CatalogError> {
        Ok(self
            .prepare_stage_lookup(key, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Prepare a stage-index read; lookup results never certify reuse.
    /// # Errors
    /// Native preparation, policy or resource failure.
    pub fn prepare_stage_lookup(
        &self,
        key: ContentHash,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<Vec<OwnedStageHint>>, CatalogError> {
        self.prepare_store_operation(
            "store.stage_hints",
            index_scope(),
            OperationPurpose::Resolve,
            vec![lit(key.to_string())],
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let hints = catalog.stage_hints_inner(key, &cancel).await?;
                    let count =
                        u64::try_from(hints.len()).map_err(|_| crate::store::encode::overflow())?;
                    Ok((hints, count))
                })
            }),
            cancel,
        )
    }

    /// Validate actual stage handles and conditionally publish an auxiliary lookup hint.
    /// # Errors
    /// Invalid stage, policy, limits, cancellation, conflict or backend failure.
    pub async fn write_stage_hint(
        &self,
        key: ContentHash,
        inputs: &StageInputs,
        output: &Snapshot,
        pass_record: &SidecarArtifact,
        cancel: &CancellationToken,
    ) -> Result<OwnedStageHint, CatalogError> {
        Ok(self
            .prepare_stage_hint_publication(key, inputs, output, pass_record, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Capture the completed stage, exact parent roles and pass record without writing.
    /// # Errors
    /// Missing producer, incompatible inputs, policy or resource failure.
    pub fn prepare_stage_hint_publication(
        &self,
        key: ContentHash,
        inputs: &StageInputs,
        output: &Snapshot,
        pass_record: &SidecarArtifact,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<OwnedStageHint>, CatalogError> {
        let pass_id = output.stage_pass().ok_or_else(|| {
            crate::store::verify::admission("stage index", "output has no registered producer")
        })?;
        let context = self.stage_context(pass_id, inputs)?;
        let mut session = self.context_session(&context, cancel)?;
        let output = Arc::new(output.clone());
        self.bind_snapshot(&mut session, "stage_index", "output", &output, cancel)?;
        self.bind_draft(
            &mut session,
            TableReference::full("stage_index", "control", "pass_record"),
            &RelationDraft {
                contract: Arc::clone(pass_record.relation().contract()),
                batches: vec![pass_record.relation().batch().clone()],
            },
        )?;
        let inputs = inputs.clone();
        let pass_record = pass_record.clone();
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name: "store.write_stage_hint",
                scope: index_scope(),
                purpose: OperationPurpose::Publish,
                arguments: vec![
                    lit(key.to_string()),
                    lit(output.manifest_ref().manifest_checksum.to_string()),
                ],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let hint = catalog
                        .write_stage_hint_inner(key, &inputs, &output, &pass_record, &cancel)
                        .await?;
                    Ok((hint, 1))
                })
            }),
            cancel,
        )
    }

    /// Reopen an exact stage through the same native resolution/admission operation.
    /// # Errors
    /// Incompatible or corrupt stage, resource, cancellation or backend failure.
    pub async fn open_stage_hint(
        &self,
        hint: &StageHint,
        inputs: &StageInputs,
        cancel: &CancellationToken,
    ) -> Result<OpenedStageHint, CatalogError> {
        Ok(self
            .prepare_stage_hint_admission(hint, inputs, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Bind exact current input roles before performing cold stage admission.
    /// # Errors
    /// Incompatible inputs, policy, native preparation or resource failure.
    pub fn prepare_stage_hint_admission(
        &self,
        hint: &StageHint,
        inputs: &StageInputs,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<OpenedStageHint>, CatalogError> {
        let context = self.stage_context(hint.pass_id, inputs)?;
        let session = self.context_session(&context, cancel)?;
        let hint = hint.clone();
        let inputs = inputs.clone();
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name: "store.open_stage_hint",
                scope: index_scope(),
                purpose: OperationPurpose::Resolve,
                arguments: vec![
                    lit(hint.output.manifest_checksum.to_string()),
                    lit(hint.pass_id.to_string()),
                ],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let stage = catalog
                        .open_stage_hint_inner(&hint, &inputs, &cancel)
                        .await?;
                    Ok((stage, 1))
                })
            }),
            cancel,
        )
    }
}

fn index_scope() -> ProviderScope {
    ProviderScope::Schema("store".into(), "stage_index".into())
}
