// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Revision continuity from actual typed receipt rows and explicit links.
use super::{CommitBase, Driver};
use crate::{CompilerError, passes::dag::invalid};
use pse_catalog::store::sidecar::SidecarArtifact;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::authored;
use std::collections::BTreeSet;

pub(super) struct History {
    pub(super) ids: BTreeSet<SemanticId>,
    pub(super) model: Option<SemanticId>,
}
impl Driver {
    pub(super) async fn revision_history(
        &self,
        base: Option<&CommitBase>,
        cancel: &CancellationToken,
    ) -> Result<History, CompilerError> {
        let mut history = History {
            ids: BTreeSet::new(),
            model: None,
        };
        let Some(base) = base else {
            return Ok(history);
        };
        let mut current = base
            .observed
            .revision_ref()
            .cloned()
            .ok_or_else(|| invalid("commit base has no explicit revision"))?;
        let mut manifest = base.snapshot.manifest_ref();
        let mut expected_model = None;
        loop {
            cancel.checkpoint()?;
            if history.ids.len() >= 131_070 {
                return Err(invalid(
                    "revision history exceeds explicit 65535 commit bound",
                ));
            }
            if current.revision_id == SemanticId::NIL || !history.ids.insert(current.revision_id) {
                return Err(invalid("revision history repeats a semantic ID"));
            }
            let receipt = self
                .catalog
                .read_change_set(
                    current.change_set.as_ref().ok_or_else(|| {
                        invalid("revision history lacks its complete change receipt")
                    })?,
                    cancel,
                )
                .await?;
            let (output, target) = receipt.output();
            if output.revision_id != current.revision_id
                || output.artifact != current.artifact
                || target != manifest
            {
                return Err(invalid(
                    "history output disagrees with exact linked revision and manifest",
                ));
            }
            let artifact = self.catalog.read_sidecar(&current.artifact, cancel).await?;
            let case = authored::case_revisions::Row::from_cells(
                self.revision_row(&artifact, "authored.case_revisions")?,
            )?;
            let model_ref = receipt
                .artifacts()
                .supporting_revisions
                .iter()
                .find(|revision| revision.revision_id == case.model_revision_id)
                .ok_or_else(|| invalid("history lacks exact model revision row"))?;
            let artifact = self
                .catalog
                .read_sidecar(&model_ref.artifact, cancel)
                .await?;
            let model = authored::model_revisions::Row::from_cells(
                self.revision_row(&artifact, "authored.model_revisions")?,
            )?;
            if case.case_revision_id != current.revision_id
                || model.model_revision_id != case.model_revision_id
                || !history.ids.insert(model.model_revision_id)
                || model.model_revision_id == SemanticId::NIL
                || expected_model.is_some_and(|expected| Some(model.model_revision_id) != expected)
            {
                return Err(invalid(
                    "typed revision rows or model predecessor continuity disagree",
                ));
            }
            history.model.get_or_insert(model.model_revision_id);
            if let Some((revision, target)) = receipt.base() {
                if case.parent_case_id != Some(revision.revision_id)
                    || model.parent_revision_id.is_none()
                {
                    return Err(invalid("revision parent differs from explicit base link"));
                }
                expected_model = Some(model.parent_revision_id);
                current = revision.clone();
                manifest = target;
            } else {
                if case.parent_case_id.is_some() || model.parent_revision_id.is_some() {
                    return Err(invalid("initial revision has an undeclared predecessor"));
                }
                break;
            }
        }
        Ok(history)
    }
    fn revision_row(
        &self,
        artifact: &SidecarArtifact,
        name: &str,
    ) -> Result<Vec<pse_schema::model::Cell>, CompilerError> {
        let spec = self
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("revision declaration absent"))?;
        if artifact.relation().contract().canonical.relation_id != spec.id {
            return Err(invalid("foreign revision row relation"));
        }
        let mut rows = pse_relations::cells::cells_from_batch(
            self.registry(),
            spec,
            artifact.relation().batch(),
        )?;
        if rows.len() != 1 {
            return Err(invalid("revision receipt must contain exactly one row"));
        }
        Ok(rows.remove(0))
    }
}
