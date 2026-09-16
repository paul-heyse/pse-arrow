// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete immutable revision objects precede the sole mutable publication CAS.
use super::{CommitRequest, CommitRevisionIds, Driver};
use crate::{CompilerError, passes::dag::invalid};
use pse_authoring::change_set::ChangeSet;
use pse_catalog::{
    Snapshot,
    store::{publish::RelationDraft, sidecar::RevisionReceipt},
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::authored;
use pse_schema::model::Cell;
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct Published {
    pub(super) model: Arc<Snapshot>,
    pub(super) tip: Arc<Snapshot>,
    model_receipt: RevisionReceipt,
    pub(super) receipt: RevisionReceipt,
}
impl Driver {
    pub(super) async fn publish_revisions(
        &self,
        completed: pse_catalog::source_production::CompletedSources,
        request: &CommitRequest,
        ids: CommitRevisionIds,
        parent_model: Option<SemanticId>,
        cancel: &CancellationToken,
    ) -> Result<Published, CompilerError> {
        let (model, tip) = self.catalog.publish_sources(completed, cancel).await?;
        let model_revision = ids.model;
        let revision = ids.case;
        let model_record = crate::records::publish_rows(
            &self.catalog,
            "authored.model_revisions",
            &[vec![
                Cell::Id(model_revision),
                parent_model.map_or(Cell::Null, Cell::Id),
                Cell::Hash(model.snapshot_id().content_hash()),
                Cell::I64(request.header.created_at),
                Cell::text(&request.header.author),
                Cell::text(&request.header.message),
            ]],
            cancel,
        )
        .await?;
        let model_receipt = self
            .catalog
            .revision_receipt(&model_record, model_revision, &model)?;
        let record = crate::records::publish_rows(
            &self.catalog,
            "authored.case_revisions",
            &[vec![
                Cell::Id(revision),
                Cell::Id(model_revision),
                request
                    .base
                    .as_ref()
                    .map_or(Cell::Null, |_| Cell::Id(request.header.base_revision_id)),
                Cell::Hash(tip.snapshot_id().content_hash()),
                Cell::I64(request.header.created_at),
                Cell::text(&request.header.author),
                Cell::text(&request.header.message),
            ]],
            cancel,
        )
        .await?;
        let receipt = self.catalog.revision_receipt(&record, revision, &tip)?;
        Ok(Published {
            model,
            tip,
            model_receipt,
            receipt,
        })
    }
    pub(super) async fn publish_changes(
        &self,
        request: &CommitRequest,
        changes: &ChangeSet,
        published: &Published,
        cancel: &CancellationToken,
    ) -> Result<RevisionReceipt, CompilerError> {
        let registry = self.registry();
        let header_artifact = crate::records::publish_rows(
            &self.catalog,
            "authored.change_sets",
            &[request.header.clone().into_cells()],
            cancel,
        )
        .await?;
        let operations = crate::records::publish_rows(
            &self.catalog,
            "authored.change_ops",
            &changes
                .ops
                .iter()
                .cloned()
                .map(authored::change_ops::Row::into_cells)
                .collect::<Vec<_>>(),
            cancel,
        )
        .await?;
        let mut staged = BTreeMap::new();
        for (port, member) in &changes.staged {
            let spec = registry
                .relation_by_id(member.relation_id)
                .ok_or_else(|| invalid("staged relation is undeclared"))?;
            let artifact = self
                .catalog
                .publish_staged_batch(
                    RelationDraft {
                        contract: Arc::new(pse_catalog::RelationContract::from_spec(
                            registry,
                            spec,
                            pse_catalog::EncodingPolicy::IpcFile,
                        )?),
                        batches: vec![member.batch.clone()],
                    },
                    cancel,
                )
                .await?;
            staged.insert(port.clone(), artifact);
        }
        let changes = self
            .catalog
            .publish_change_set(
                pse_catalog::store::changes::ChangeSetDraft {
                    header: header_artifact,
                    operations,
                    staged,
                    supporting_revisions: vec![published.model_receipt.reference().clone()],
                },
                request
                    .base
                    .as_ref()
                    .map(|base| (&base.observed, base.snapshot.as_ref())),
                &published.receipt,
                cancel,
            )
            .await?;
        Ok(self.catalog.with_change_set(&published.receipt, &changes)?)
    }
    pub(super) async fn move_commit_ref(
        &self,
        request: &CommitRequest,
        published: &Published,
        cancel: &CancellationToken,
    ) -> Result<(), CompilerError> {
        self.catalog
            .compare_and_swap_revision_ref(
                &request.reference,
                request.base.as_ref().map(|base| &base.observed),
                &published.tip,
                &published.receipt,
                cancel,
            )
            .await?;
        Ok(())
    }
}
