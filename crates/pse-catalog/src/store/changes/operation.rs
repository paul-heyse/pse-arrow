// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete receipt work executes against captured before/after and artifact inputs.

use super::{ChangeSetDraft, ChangeSetReceipt, ChangeSetRef};
use crate::store::{
    membership::AdmissionContext,
    operation::{PreparedStoreOperation, StoreCommand},
    publish::RelationDraft,
    refs::RefState,
    sidecar::RevisionReceipt,
};
use crate::{Catalog, CatalogError, LoadedRelation, Snapshot, session::SnapshotSession};
use datafusion::{common::TableReference, logical_expr::lit};
use pse_ids::CancellationToken;
use pse_schema::model::provider::{OperationPurpose, ProviderScope};
use std::{collections::BTreeMap, sync::Arc};

impl Catalog {
    /// Publish a complete change receipt through its actual native operation.
    /// # Errors
    /// Foreign or incomplete sources, invalid changes, allocation, cancellation or I/O failure.
    pub async fn publish_change_set(
        &self,
        draft: ChangeSetDraft,
        base: Option<(&RefState, &Snapshot)>,
        output: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<ChangeSetReceipt, CatalogError> {
        Ok(self
            .prepare_change_publication(draft, base, output, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Bind actual source generations and every operation artifact before publication.
    /// Preparation neither validates row values nor performs storage I/O.
    /// # Errors
    /// Invalid declarations, incompatible source bindings, policy or allocation failure.
    pub fn prepare_change_publication(
        &self,
        draft: ChangeSetDraft,
        base: Option<(&RefState, &Snapshot)>,
        output: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<ChangeSetReceipt>, CatalogError> {
        let mut session =
            self.change_session(base.map(|(_, snapshot)| snapshot), &output.snapshot, cancel)?;
        self.bind_change_artifacts(&mut session, &draft)?;
        let base = base.map(|(state, snapshot)| (state.clone(), snapshot.clone()));
        let output = output.clone();
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name: "store.publish_change_set",
                scope: change_scope(),
                purpose: OperationPurpose::Publish,
                arguments: vec![lit(output.reference().revision_id.to_string())],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let receipt = catalog
                        .publish_change_set_inner(
                            draft,
                            base.as_ref().map(|(state, snapshot)| (state, snapshot)),
                            &output,
                            &cancel,
                        )
                        .await?;
                    Ok((receipt, 1))
                })
            }),
            cancel,
        )
    }

    /// Resolve and admit the exact durable receipt in a native read operation.
    /// # Errors
    /// Corrupt, incomplete or unsupported receipt, resource, cancellation or I/O failure.
    pub async fn read_change_set(
        &self,
        reference: &ChangeSetRef,
        cancel: &CancellationToken,
    ) -> Result<ChangeSetReceipt, CatalogError> {
        Ok(self
            .prepare_change_read(reference, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Capture exact identity and encoding claims without reading the store.
    /// # Errors
    /// Policy, native preparation or resource failure.
    pub fn prepare_change_read(
        &self,
        reference: &ChangeSetRef,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<ChangeSetReceipt>, CatalogError> {
        let reference = reference.clone();
        self.prepare_store_operation(
            "store.read_change_set",
            change_scope(),
            OperationPurpose::Resolve,
            vec![
                lit(reference.change_set_id.to_string()),
                lit(reference.encoding_checksum.to_string()),
            ],
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let receipt = catalog.read_change_set_inner(&reference, &cancel).await?;
                    Ok((receipt, 1))
                })
            }),
            cancel,
        )
    }

    /// Execute source-correspondence validation against actual before/after providers.
    /// # Errors
    /// Foreign context, incomplete source history, resource, cancellation or I/O failure.
    pub async fn validate_change_context(
        &self,
        changes: &ChangeSetReceipt,
        base: Option<&Snapshot>,
        output: &Snapshot,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        self.prepare_change_validation(changes, base, output, cancel)?
            .execute(cancel)
            .await?;
        Ok(())
    }

    /// Prepare complete source correspondence as an inspectable native read operation.
    /// # Errors
    /// Incompatible bindings, policy, native preparation or resource failure.
    pub fn prepare_change_validation(
        &self,
        changes: &ChangeSetReceipt,
        base: Option<&Snapshot>,
        output: &Snapshot,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<()>, CatalogError> {
        let mut session = self.change_session(base, output, cancel)?;
        self.bind_change_artifacts(&mut session, changes.artifacts())?;
        let changes = changes.clone();
        let base = base.cloned();
        let output = output.clone();
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name: "store.validate_change_context",
                scope: change_scope(),
                purpose: OperationPurpose::Resolve,
                arguments: vec![lit(changes.reference().change_set_id.to_string())],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    catalog
                        .validate_change_context_inner(&changes, base.as_ref(), &output, &cancel)
                        .await?;
                    Ok(((), 1))
                })
            }),
            cancel,
        )
    }

    fn change_session(
        &self,
        before: Option<&Snapshot>,
        after: &Snapshot,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        let mut parents = BTreeMap::from([("after".into(), Arc::new(after.clone()))]);
        if let Some(before) = before {
            parents.insert("before".into(), Arc::new(before.clone()));
        }
        self.context_session(
            &AdmissionContext {
                parents,
                ..AdmissionContext::default()
            },
            cancel,
        )
    }

    fn bind_change_artifacts(
        &self,
        session: &mut SnapshotSession,
        draft: &ChangeSetDraft,
    ) -> Result<(), CatalogError> {
        self.bind_change_artifact(session, "control", "header", draft.header.relation())?;
        self.bind_change_artifact(
            session,
            "control",
            "operations",
            draft.operations.relation(),
        )?;
        for (port, artifact) in &draft.staged {
            self.bind_change_artifact(session, "staged", port, artifact.relation())?;
        }
        Ok(())
    }

    fn bind_change_artifact(
        &self,
        session: &mut SnapshotSession,
        schema: &str,
        port: &str,
        relation: &LoadedRelation,
    ) -> Result<(), CatalogError> {
        self.bind_draft(
            session,
            TableReference::full("changes", schema, port),
            &RelationDraft {
                contract: Arc::clone(relation.contract()),
                batches: vec![relation.batch().clone()],
            },
        )
    }
}

fn change_scope() -> ProviderScope {
    ProviderScope::Schema("store".into(), "changes".into())
}
