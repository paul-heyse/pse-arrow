// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0/P1/P2 candidates become visible through one conditional revision-ref update.
mod prepare;
mod publish;

use super::{BaseReader, Driver, inputs};
use crate::{
    CompilerError,
    passes::{dag::invalid, p2},
};
use pse_authoring::{
    change_set::{AuthoredReader, OwnedChangeSet},
    document::{DocumentBundle, OwnedDocumentSet},
};
use pse_catalog::{RefName, Snapshot, store::refs::RefState};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::authored;
use pse_rules::invariants::InvariantReport;
use pse_schema::model::{Authority, SnapshotClass};
use std::{collections::BTreeMap, sync::Arc};

/// An observed revision alias and its exact admitted target, retained through CAS.
#[derive(Clone, Debug)]
pub struct CommitBase {
    /// Both backend conditional tokens and the actual typed revision receipt.
    pub observed: RefState,
    /// Admitted target matching the exact observed manifest.
    pub snapshot: Arc<Snapshot>,
}
/// Explicit revision identities, assignable before authoring case bindings.
#[derive(Clone, Copy, Debug)]
pub struct CommitRevisionIds {
    /// New model revision, exactly referenced by every candidate case row.
    pub model: SemanticId,
    /// New case revision for the atomic commit alias.
    pub case: SemanticId,
}
/// A complete desired authoring universe, or a previously staged full change-set.
#[derive(Clone, Debug)]
pub struct CommitRequest {
    /// Mutable commit alias to update only after successful P2 and immutable writes.
    pub reference: RefName,
    /// Optional preassigned IDs; absent selects fresh `UUIDv7` identities.
    pub revision_ids: Option<CommitRevisionIds>,
    /// Exact observed revision; absent only for first creation.
    pub base: Option<CommitBase>,
    /// Exact desired source inventory. Immutable parsed documents are retained by source construction.
    pub documents: Vec<DocumentBundle>,
    /// Typed operation header with an explicit expected base revision.
    pub header: authored::change_sets::Row,
    /// Optional staged envelope, including retained renamed sources when applicable.
    pub changes: Option<OwnedChangeSet>,
}
/// Commit outcome, retaining all P2 findings even when nothing is published.
#[derive(Debug)]
pub struct CommitReport {
    /// Actual P2 evidence, including every violating and undecided key.
    pub validation: Arc<InvariantReport>,
    /// New model snapshot, present only on successful commit.
    pub model: Option<Arc<Snapshot>>,
    /// New case tip whose model parent is published with it through one alias.
    pub tip: Option<Arc<Snapshot>>,
    /// Explicit new model revision identity, absent on validation refusal.
    pub model_revision_id: Option<SemanticId>,
    /// Explicit new case revision identity, absent on validation refusal.
    pub revision_id: Option<SemanticId>,
    /// Complete typed terminal sidecars for P0, P1 and P2.
    pub attempts: Vec<pse_catalog::store::sidecar::SidecarArtifact>,
}
impl CommitReport {
    fn rejected(
        validation: Arc<InvariantReport>,
        attempts: Vec<pse_catalog::store::sidecar::SidecarArtifact>,
    ) -> Self {
        Self {
            validation,
            model: None,
            tip: None,
            model_revision_id: None,
            revision_id: None,
            attempts,
        }
    }
}
impl Driver {
    /// Reopen exact original source bytes for a retained commit base.
    /// # Errors
    /// Ref/snapshot disagreement, absent revision receipt or source admission failure.
    pub async fn base_reader(
        &self,
        base: &CommitBase,
        cancel: &CancellationToken,
    ) -> Result<BaseReader, CompilerError> {
        if base.observed.manifest_ref() != base.snapshot.manifest_ref() {
            return Err(invalid("commit snapshot differs from exact observed ref"));
        }
        let revision = base
            .observed
            .revision_ref()
            .ok_or_else(|| invalid("commit base ref has no typed revision receipt"))?;
        let artifact = self
            .catalog
            .read_sidecar(&revision.artifact, cancel)
            .await?;
        self.catalog
            .revision_receipt(&artifact, revision.revision_id, &base.snapshot)?;
        let bindings = inputs::inventory(&base.snapshot, self.registry())?;
        let rows = inputs::row_inventory(&bindings);
        let documents = inputs::documents(&self.catalog, &rows, cancel)?;
        let mut work = self.sessions.reserver().open("compiler:base-sidecars");
        let extent = pse_authoring::document::workspace_extent(&documents)?;
        work.try_grow(extent).map_err(pse_ids::CanonError::from)?;
        let mut base_rows = inputs::primitive_rows(&rows, self.registry());
        let mut checked = bindings
            .iter()
            .filter(|(key, _)| {
                self.registry()
                    .relation(&key.qualified_name())
                    .is_some_and(|spec| {
                        matches!(spec.authority, Authority::Authored | Authority::Reference)
                    })
            })
            .map(|(_, binding)| (binding.relation_id(), binding.relation().checked().clone()))
            .collect::<pse_authoring::document::Batches>();
        let mut sidecars =
            BTreeMap::<SemanticId, Vec<pse_relations::columnar::FieldCheckedBatch>>::new();
        for bundle in documents.bundles() {
            for (id, batch) in &bundle.batches {
                let spec = self
                    .registry()
                    .relation_by_id(*id)
                    .ok_or_else(|| invalid("source relation absent"))?;
                if spec.snapshot_class == SnapshotClass::Sidecar {
                    sidecars.entry(*id).or_default().push(batch.clone());
                }
            }
        }
        for (id, batches) in sidecars {
            let spec = self
                .registry()
                .relation_by_id(id)
                .ok_or_else(|| invalid("sidecar relation absent"))?;
            let batch = pse_relations::columnar::FieldCheckedBatch::concat(
                self.registry(),
                spec,
                &batches,
            )?;
            base_rows.insert(id, batch.batch().clone());
            checked.insert(id, batch);
        }
        Ok(BaseReader {
            revision: revision.revision_id,
            rows: base_rows,
            checked: Some(checked),
            documents,
        })
    }
    /// Reparse sources, apply exact typed changes, run P2 and publish one complete revision.
    /// # Errors
    /// Syntax/binding/base disagreement, rule execution, resource/cancel or immutable/CAS failure.
    pub async fn commit(
        &mut self,
        request: CommitRequest,
        cancel: &CancellationToken,
    ) -> Result<CommitReport, CompilerError> {
        let input = request
            .base
            .as_ref()
            .map(|base| base.snapshot.snapshot_id());
        let mut attempts = Vec::new();
        let p0 = self.attempt("P0")?;
        let initial = match self.prepare_initial(&request, cancel).await {
            Ok(initial) => initial,
            Err(error) => return Err(p0.failed(&self.catalog, input, error).await),
        };
        attempts.push(
            p0.success(
                &self.catalog,
                crate::records::Observation::success(input),
                cancel,
            )
            .await?,
        );
        let p1 = self.attempt("P1")?;
        let prepared = match prepare::source(
            self,
            &request,
            &initial.base,
            initial.ids,
            initial.documents,
            cancel,
        )
        .await
        {
            Ok(prepared) => prepared,
            Err(error) => return Err(p1.failed(&self.catalog, input, error).await),
        };
        attempts.push(
            p1.success(
                &self.catalog,
                crate::records::Observation::success(input),
                cancel,
            )
            .await?,
        );
        let p2 = self.attempt("P2")?;
        let completed = match self
            .validate_commit(
                &request,
                &prepared,
                initial.ids,
                initial.parent_model,
                cancel,
            )
            .await
        {
            Ok(completed) => completed,
            Err(error) => return Err(p2.failed(&self.catalog, input, error).await),
        };
        let (validation, engine, published) = completed;
        if validation.error_count() != 0 {
            let error = pse_rules::RuleError::InvariantViolations {
                count: validation.error_count(),
                findings: validation
                    .findings()
                    .iter()
                    .map(|batch| batch.batch().clone())
                    .collect(),
            }
            .into();
            match p2.failed(&self.catalog, input, error).await {
                CompilerError::AttemptFailed { record, .. } => attempts.push(*record),
                recording_failure => return Err(recording_failure),
            }
            return Ok(CommitReport::rejected(validation, attempts));
        }
        let published =
            published.ok_or_else(|| invalid("successful validation has no published commit"))?;
        let findings = validation
            .findings()
            .iter()
            .map(|batch| batch.batch().clone())
            .collect::<Vec<_>>();
        let plans = validation
            .completion()
            .map(|completed| completed.prepared().observation().clone())
            .into_iter()
            .collect::<Vec<_>>();
        let terminal = p2
            .success(
                &self.catalog,
                crate::records::Observation {
                    status: crate::PassStatus::Ok,
                    input,
                    output: Some(published.tip.snapshot_id()),
                    engine: Some(engine),
                    findings: &findings,
                    derivations: &[],
                    plans: &plans,
                },
                cancel,
            )
            .await?;
        self.move_commit_ref(&request, &published, cancel)
            .await
            .map_err(|source| CompilerError::CommitPublication {
                pass_run_id: p2.id,
                output: published.tip.snapshot_id(),
                record: Box::new(terminal.clone()),
                source: Box::new(source),
            })?;
        attempts.push(terminal);
        Ok(CommitReport {
            validation,
            model: Some(published.model),
            tip: Some(published.tip),
            model_revision_id: Some(initial.ids.model),
            revision_id: Some(initial.ids.case),
            attempts,
        })
    }
    fn attempt(&self, name: &str) -> Result<crate::records::Attempt, CompilerError> {
        self.registry()
            .pass(name)
            .map(crate::records::Attempt::new)
            .ok_or_else(|| invalid("commit pass absent"))
    }
    async fn prepare_initial(
        &self,
        request: &CommitRequest,
        cancel: &CancellationToken,
    ) -> Result<prepare::Initial, CompilerError> {
        cancel.checkpoint()?;
        let base = if let Some(base) = &request.base {
            if base.observed.name() != &request.reference {
                return Err(invalid("commit base belongs to another ref"));
            }
            self.base_reader(base, cancel).await?
        } else {
            let checked = self.empty_primitives(cancel)?;
            BaseReader {
                revision: SemanticId::NIL,
                rows: checked
                    .iter()
                    .map(|(id, batch)| (*id, batch.batch().clone()))
                    .collect(),
                checked: Some(checked),
                documents: OwnedDocumentSet::default(),
            }
        };
        if request.header.base_revision_id != base.revision_id() {
            return Err(invalid("change-set expects a different actual revision"));
        }
        let history = self.revision_history(request.base.as_ref(), cancel).await?;
        let ids = request.revision_ids.unwrap_or_else(|| CommitRevisionIds {
            model: pse_authoring::ids::uuid_v7(),
            case: pse_authoring::ids::uuid_v7(),
        });
        if ids.model == ids.case
            || [ids.model, ids.case]
                .iter()
                .any(|id| *id == SemanticId::NIL || history.ids.contains(id))
        {
            return Err(invalid(
                "new revision IDs must be distinct, nonzero and absent from history",
            ));
        }
        let documents = pse_authoring::document::load_bundles_owned(
            &request.documents,
            self.registry(),
            self.sessions.reserver().as_ref(),
            cancel,
        )?;
        let mut source_work = self.sessions.reserver().open("compiler:P0-source-work");
        source_work
            .try_grow(pse_authoring::document::workspace_extent(&documents)?)
            .map_err(pse_ids::CanonError::from)?;
        cancel.checkpoint()?;
        let packages = pse_authoring::p1::source_batches(documents.bundles(), self.registry())?
            .remove(&authored::packages::RELATION_ID)
            .ok_or_else(|| invalid("package source absent"))?;
        let source_session =
            self.sessions
                .candidate(BTreeMap::new(), Arc::clone(self.registry()), cancel)?;
        pse_authoring::p0::resolve(&packages, &source_session, cancel).await?;
        cancel.checkpoint()?;
        Ok(prepare::Initial {
            base,
            ids,
            parent_model: history.model,
            documents,
        })
    }
    async fn validate_commit(
        &self,
        request: &CommitRequest,
        prepared: &prepare::Prepared,
        ids: CommitRevisionIds,
        parent_model: Option<SemanticId>,
        cancel: &CancellationToken,
    ) -> Result<
        (
            Arc<InvariantReport>,
            pse_ids::ContentHash,
            Option<publish::Published>,
        ),
        CompilerError,
    > {
        cancel.checkpoint()?;
        let session = self.sessions.candidate_checked(
            prepared.checked.clone(),
            Arc::clone(self.registry()),
            cancel,
        )?;
        let validation =
            Arc::new(p2::validate(&prepared.rows, &session, self.registry(), cancel).await?);
        let engine = session.profile_hash();
        if validation.error_count() != 0 {
            return Ok((validation, engine, None));
        }
        let result = async {
            let completed =
                self.catalog
                    .complete_sources(Arc::new(crate::validator::SourceArguments {
                        candidate: prepared.candidate.clone(),
                        rows: prepared.checked.clone(),
                        validation: Arc::clone(&validation),
                        registry: Arc::clone(self.registry()),
                    }))?;
            let mut published = self
                .publish_revisions(completed, request, ids, parent_model, cancel)
                .await?;
            published.receipt = self
                .publish_changes(request, &prepared.candidate.changes, &published, cancel)
                .await?;
            Ok(published)
        }
        .await
        .map_err(|source| CompilerError::PassFailure {
            source: Box::new(source),
            findings: validation
                .findings()
                .iter()
                .map(|batch| batch.batch().clone())
                .collect(),
        })?;
        Ok((validation, engine, Some(result)))
    }
    fn empty_primitives(
        &self,
        cancel: &CancellationToken,
    ) -> Result<pse_authoring::document::Batches, CompilerError> {
        let registry = self.registry();
        let mut rows = BTreeMap::new();
        let declared = registry
            .schema_rows_ref()
            .iter()
            .map(|(key, rows)| (*key, rows))
            .collect::<BTreeMap<_, _>>();
        for spec in registry
            .relations()
            .iter()
            .filter(|spec| matches!(spec.authority, Authority::Authored | Authority::Reference))
        {
            let values = declared
                .get(&spec.key)
                .map_or(&[][..], |rows| rows.as_slice());
            rows.insert(
                spec.id,
                pse_relations::columnar::FieldCheckedBatch::admit(
                    registry,
                    spec,
                    pse_relations::cells::batch_from_cells_owned(
                        registry,
                        spec,
                        values,
                        self.sessions.reserver().as_ref(),
                        cancel,
                    )?,
                )?,
            );
        }
        Ok(rows)
    }
}
