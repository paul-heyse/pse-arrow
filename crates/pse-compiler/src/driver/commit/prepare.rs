// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Accounted P0/P1 source ownership before candidate validation or publication.
use super::{BaseReader, CommitRequest, CommitRevisionIds, Driver};
use crate::{CompilerError, passes::dag::invalid};
use pse_authoring::{change_set::OwnedCandidateSnapshot, document::OwnedDocumentSet};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

pub(super) struct Initial {
    pub(super) base: BaseReader,
    pub(super) ids: CommitRevisionIds,
    pub(super) parent_model: Option<SemanticId>,
    pub(super) documents: OwnedDocumentSet,
}
pub(super) struct Prepared {
    pub(super) rows: BTreeMap<RelationKey, RecordBatch>,
    pub(super) checked: BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
    pub(super) candidate: OwnedCandidateSnapshot,
}

pub(super) async fn source(
    driver: &Driver,
    request: &CommitRequest,
    base: &BaseReader,
    ids: CommitRevisionIds,
    documents: OwnedDocumentSet,
    cancel: &CancellationToken,
) -> Result<Prepared, CompilerError> {
    cancel.checkpoint()?;
    let registry = driver.registry();
    let session =
        driver
            .sessions
            .candidate(BTreeMap::new(), std::sync::Arc::clone(registry), cancel)?;
    let candidate = if let Some(changes) = &request.changes {
        if changes.header != request.header {
            return Err(invalid("staged header differs from commit header"));
        }
        let applied =
            pse_authoring::change_set::apply_owned(base, changes, &session, cancel).await?;
        if changes.documents().is_some() {
            applied
        } else {
            pse_authoring::p1::admit_source_candidate(&documents, &applied, &session, cancel)
                .await?
        }
    } else {
        pse_authoring::p1::construct(&documents, base, request.header.clone(), &session, cancel)
            .await?
    };
    let mut checked = BTreeMap::new();
    for (id, batch) in candidate.checked_relations() {
        let spec = registry
            .relation_by_id(*id)
            .ok_or_else(|| invalid("candidate relation absent"))?;
        checked.insert(spec.key, batch.clone());
    }
    let case_spec = registry
        .relation("authored.cases")
        .ok_or_else(|| invalid("case declaration absent"))?;
    let case_batch = checked
        .get(&case_spec.key)
        .ok_or_else(|| invalid("case inventory absent"))?;
    let cases = authored::cases::View::from_checked(case_batch)?;
    for ordinal in 0..case_batch.batch().num_rows() {
        if cases.row(ordinal)?.model_revision_id != ids.model {
            return Err(invalid(
                "authored case model_revision_id differs from selected commit model revision",
            ));
        }
    }
    Ok(Prepared {
        rows: checked
            .iter()
            .map(|(key, batch)| (*key, batch.batch().clone()))
            .collect(),
        checked,
        candidate,
    })
}
