// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One immutable admitted study definition used by both executors. Durable creation stores
//! exact source bundles, operations, physically admitted bindings and typed point policy.
//! Workers acquire inputs, ask the shared policy for a fenced start, and retain outcomes
//! independently of result availability. Finalization publishes every recorded member and
//! one structured outcome for each requested occurrence.

use super::{
    AdmittedBinding, OperationRequest, OperationSource, Operations, PointOverlay,
    PreparationSettings, Runtime, StudyOperation, WorkflowError, contract,
    publication::{Published, Workspace, candidate_record},
    worker::{
        JOB_PAYLOAD_VERSION, JobPayload, JobTask, StudyFinalization, StudyOperationJob,
        StudyPointBinding,
    },
};
use datafusion::common::ResolvedTableReference;
use pse_catalog::artifact::{ArtifactPlan, RelationOutput};
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, SemanticId};
use pse_model::study::{OccurrenceGraph, PointPolicy};
pub use pse_model::study::{PointAttemptOutcome, PointOutcome};
use pse_model::{document::Version, generated::enums::PublicationKind};
use pse_operations::{
    OperationsError,
    attempts::{AttemptId, AttemptKind, NewAttempt, RunId},
    catalog::{MemberDescriptor, NewIntent, PublicationCommit, PublicationId, WorkspaceId},
    jobs::{JobState, NewJob, RetryPolicy},
    lifecycle::AttemptState,
    studies::{
        NewPoint, NewStudy, StudyCancel, StudyFilter, StudyId, StudyPointState, StudyRecord,
        StudyState,
    },
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::runtime::{publication_manifests, study_outcomes},
};
use std::{collections::BTreeMap, time::Duration};

/// The most points one study holds.
pub const MAXIMUM_STUDY_POINTS: usize = 100_000;

/// How often a commit re-reads a workspace head that moved under it.
const COMMIT_ATTEMPTS: usize = 64;

/// The authored sources of a package closure, each document's exact bytes by path (a
/// data document's included, ADR-0125): what a worker loads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSources {
    /// The physical package's documents.
    pub physical: BTreeMap<String, Vec<u8>>,
    /// The modeling package closure's documents, one map per package in load order.
    pub modeling: Vec<BTreeMap<String, Vec<u8>>>,
}

/// Authored policy choices; seed consumption is supplied only by operation admission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointPolicy {
    /// Requested occurrence identity.
    pub key: pse_model::study::OccurrenceKey,
    /// Explicit ordering and scientific-result dependencies.
    pub dependencies: Vec<pse_model::study::Dependency>,
    /// Explicit start/seed selection.
    pub start: pse_model::study::StartPolicy,
    /// Maximum tries including the first.
    pub attempt_limit: u32,
}

/// One point of a study to start.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPoint {
    /// Supported existing operation and its complete inputs.
    pub operation: OperationRequest,
    /// Existing preparation controls, serialized without a second default authority.
    pub preparation: PreparationSettings,
    /// Submitted physical assignments, admitted once before any point is scheduled.
    pub overlay: PointOverlay,
    /// Occurrence identity and explicit dependency/start policy.
    pub policy: StudyPointPolicy,
}

/// Raw request admitted once into the immutable executable definition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyRequest {
    /// Document version.
    pub version: Version<1>,
    /// Ordered occurrence requests.
    pub points: Vec<StudyPoint>,
}

/// A durable study to start.
#[derive(Clone, Debug)]
pub struct StudyPlan {
    /// The package closure's sources, stored content-addressed for the workers.
    pub sources: PackageSources,
    /// The points, in index order.
    pub points: Vec<StudyPoint>,
    /// How often each point and the finalization may be tried.
    pub retry: RetryPolicy,
    /// The priority of the study's jobs.
    pub priority: i32,
}

/// Version 3 of a study's definition: the store's `definition` document and the content of
/// the study's request identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyDefinition {
    /// Document version.
    pub version: Version<3>,
    /// The source bundle of the physical package.
    pub physical: ContentHash,
    /// The source bundles of the modeling package closure, in load order.
    pub modeling: Vec<ContentHash>,
    /// The points, in index order.
    pub points: Vec<StudyPointDefinition>,
}

/// One point of a study's definition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointDefinition {
    /// Reconstructable admitted descriptor of one existing operation.
    pub operation: StudyOperation,
    /// The hash of the point's value bindings.
    pub binding_hash: ContentHash,
    /// Canonical physically admitted member assignments; original paths are attribution.
    pub binding: AdmittedBinding,
    /// Shared occurrence/dependency/start policy.
    pub policy: PointPolicy,
}
impl StudyDefinition {
    /// Validate supported producer and consumer roles for every immutable descriptor.
    pub fn validate_roles(&self) -> Result<(), WorkflowError> {
        pse_operations::study_policy::admit(&self.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        for point in &self.points {
            match &point.policy.start {
                pse_model::study::StartPolicy::Continuation(edge) => {
                    point.operation.admit_seed_role(edge.role)?;
                    let predecessor = self
                        .points
                        .iter()
                        .find(|point| point.policy.key == edge.predecessor)
                        .ok_or_else(|| contract("missing admitted predecessor"))?;
                    predecessor.operation.admit_seed_role(edge.role)?;
                }
                pse_model::study::StartPolicy::Explicit { role, .. } => {
                    point.operation.admit_seed_role(*role)?
                }
                pse_model::study::StartPolicy::Fresh => {}
            }
        }
        Ok(())
    }
    /// Mechanically derive the shared policy graph from the one executable definition.
    pub fn graph(&self) -> OccurrenceGraph {
        OccurrenceGraph {
            points: self
                .points
                .iter()
                .map(|point| point.policy.clone())
                .collect(),
        }
    }
}

/// Where a study point's try writes its result members: the study's intent.
#[derive(Clone, Debug)]
pub(crate) struct PointContext {
    pub(crate) study_id: StudyId,
    pub(crate) job_id: pse_operations::jobs::JobId,
    pub(crate) attempt_id: AttemptId,
    pub(crate) point_index: u32,
    pub(crate) revision: u64,
    pub(crate) source_revision: ContentHash,
    pub(crate) start: Option<pse_model::study::StartProvenance>,
    pub(crate) scientific: pse_model::study::ScientificFacts,
    pub(crate) diagnostic: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    pub(crate) effect: pse_model::study::EffectState,
    pub(crate) publication_id: PublicationId,
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) member_prefix: url::Url,
}

/// The catalog a completed point's result members are published under.
fn point_catalog(point: u32) -> String {
    format!("point_{point}")
}

fn identity(
    frame: pse_ids::Frame,
    value: &impl serde::Serialize,
) -> Result<ContentHash, WorkflowError> {
    pse_backend_native::identity::of(frame, value)
        .map_err(|e| WorkflowError::Math(crate::math::MathRuntimeError::from(e)))
}

fn document(value: &impl serde::Serialize, what: &str) -> Result<serde_json::Value, WorkflowError> {
    serde_json::to_value(value).map_err(|e| contract(format!("{what}: {e}")))
}

fn prefix_of(uri: &str) -> Result<url::Url, WorkflowError> {
    url::Url::parse(uri).map_err(|error| contract(format!("intent member prefix {uri}: {error}")))
}

impl Operations {
    /// Where a study point's try writes its result members: the study's registered intent.
    ///
    /// # Errors
    /// An unknown study or intent; store failures.
    pub(crate) async fn point_context(
        &self,
        binding: &StudyPointBinding,
    ) -> Result<PointContext, WorkflowError> {
        let studies = self.store().studies();
        let point = studies.point(binding.study_id, binding.point_index).await?;
        if point.binding_hash != binding.binding_hash {
            return Err(contract(format!(
                "point {} of study {} is bound to other values than the job",
                binding.point_index, binding.study_id
            )));
        }
        let study = studies.row(binding.study_id).await?;
        let intent = self
            .store()
            .catalog()
            .intent(study.publication_id)
            .await?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication intent",
                id: study.publication_id.to_string(),
            })?;
        Ok(PointContext {
            study_id: binding.study_id,
            job_id: point.job_id,
            attempt_id: point.attempt_id,
            point_index: binding.point_index,
            revision: point.revision,
            source_revision: binding.operation.source.revision,
            start: None,
            scientific: Default::default(),
            diagnostic: None,
            effect: pse_model::study::EffectState::Absent,
            publication_id: intent.publication_id,
            workspace_id: intent.workspace_id,
            member_prefix: prefix_of(&intent.member_prefix)?,
        })
    }
}

impl super::ModelingPackage {
    /// Admit operations and physical bindings before either executor schedules an occurrence.
    pub async fn admit_study_points(
        &self,
        physical: ContentHash,
        modeling: Vec<ContentHash>,
        points: &[StudyPoint],
        cancel: &crate::CancelSource,
    ) -> Result<StudyDefinition, WorkflowError> {
        if points.is_empty() {
            return Err(WorkflowError::Typed(
                pse_model::diagnostic::DiagnosticCause::new(
                    pse_operations::study_policy::PolicyError::Empty,
                ),
            ));
        }
        if points.len() > MAXIMUM_STUDY_POINTS {
            let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::InvalidModel,
                pse_diagnostics::DiagnosticStage::StudyAdmission,
                [],
                pse_diagnostics::DiagnosticRule::StudyPolicyAdmission,
            );
            diagnostic.observations.insert(
                "occurrences".into(),
                pse_model::diagnostic::Observation::Integer(
                    i64::try_from(points.len()).unwrap_or(i64::MAX),
                ),
            );
            diagnostic.observations.insert(
                "maximum_occurrences".into(),
                pse_model::diagnostic::Observation::Integer(MAXIMUM_STUDY_POINTS as i64),
            );
            return Err(diagnostic.into());
        }
        let mut admitted = Vec::with_capacity(points.len());
        for point in points {
            let mut operation = StudyOperation {
                version: Version,
                source: OperationSource::of(self),
                preparation: point.preparation.clone(),
                operation: point.operation.clone(),
                admitted_horizon: None,
            };
            operation.admit_horizon_values(self, cancel).await?;
            let binding = self
                .admit_operation_overlay(&operation, &point.overlay, cancel)
                .await?;
            let prepared = self
                .prepare_bound_operation(&operation, &binding, cancel)
                .await?;
            let policy = PointPolicy {
                key: point.policy.key,
                dependencies: point.policy.dependencies.clone(),
                start: point.policy.start.clone(),
                attempt_limit: point.policy.attempt_limit,
                seed_need: prepared.seed_need(),
            };
            admitted.push(StudyPointDefinition {
                binding_hash: binding.identity(),
                binding,
                operation,
                policy,
            });
        }
        let definition = StudyDefinition {
            version: Version,
            physical,
            modeling,
            points: admitted,
        };
        pse_operations::study_policy::admit(&definition.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        definition.validate_roles()?;
        Ok(definition)
    }
}

impl Runtime {
    /// Resolve stopped point writes through exact native receipts before queue dispatch.
    pub(crate) async fn reconcile_study_receipts(&self) -> Result<(), WorkflowError> {
        use pse_model::study::{EffectState, OccurrenceKey};
        let operations = self.operations()?;
        let records = operations
            .store()
            .studies()
            .list(&StudyFilter {
                states: vec![StudyState::Open],
                limit: i64::MAX,
            })
            .await?;
        let state = std::sync::Arc::new(self.sessions.native_state().clone());
        for row in records {
            let record = operations.store().studies().get(row.study_id).await?;
            let intent = operations
                .store()
                .catalog()
                .intent(row.publication_id)
                .await?
                .ok_or_else(|| contract("study intent unavailable during receipt recovery"))?;
            for point in record.points {
                if point.job_state == JobState::Running
                    || point
                        .outcome
                        .as_ref()
                        .is_none_or(|o| o.effect != EffectState::Unknown)
                {
                    continue;
                }
                let Some(document) = point.receipt.as_ref() else {
                    // No member write can begin before its ticket is durably recorded.
                    operations
                        .store()
                        .studies()
                        .reconcile_effect(
                            row.study_id,
                            OccurrenceKey(point.point_index),
                            point.revision,
                            EffectState::Absent,
                        )
                        .await?;
                    continue;
                };
                let ticket: pse_catalog::delta::ticket::PublicationTicket =
                    serde_json::from_value(document.clone())
                        .map_err(|e| contract(format!("point receipt: {e}")))?;
                if ticket.publication_id() != row.publication_id
                    || ticket.workspace_id() != intent.workspace_id
                {
                    return Err(contract("point receipt differs from study intent"));
                }
                let outcome = point
                    .outcome
                    .as_ref()
                    .ok_or_else(|| contract("receipt has no modern point facts"))?;
                if ticket.attempt_id() != point.attempt_id
                    && !outcome
                        .attempts
                        .iter()
                        .any(|attempt| attempt.attempt_id == Some(ticket.attempt_id()))
                {
                    return Err(contract(
                        "point receipt attempt is outside occurrence history",
                    ));
                }
                let Ok(receipts) = ticket
                    .recover_members(&state, &CancellationToken::new())
                    .await
                else {
                    continue;
                };
                // Native absence/partial receipt observations cannot fence an old writer.
                // Only the entire exact inventory closes this member effect idempotently.
                if !receipts.complete && receipts.members.is_empty() {
                    continue;
                }
                let effect = if receipts.complete {
                    EffectState::Idempotent
                } else {
                    EffectState::Unknown
                };
                operations
                    .store()
                    .studies()
                    .reconcile_receipt(
                        row.study_id,
                        OccurrenceKey(point.point_index),
                        point.revision,
                        effect,
                        Some(ticket.attempt_id()),
                        &receipts.members,
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

impl Runtime {
    /// Write `tables` as unpublished members under `prefix` (`{prefix}{schema}/{table}/`),
    /// named in `catalog`, for the publication `publication_id` of `workspace_id`. The
    /// member receipts name `attempt`, which also names the writes: writing the same tables
    /// again for the same attempt recovers the members already written. Returns every
    /// member with its actual version; nothing becomes visible.
    #[expect(
        clippy::too_many_arguments,
        reason = "one member write: tables, their names and destination, and the publication and attempt the receipts name"
    )]
    async fn write_members(
        &self,
        tables: &BTreeMap<SemanticId, FieldCheckedBatch>,
        catalog: &str,
        prefix: &url::Url,
        publication_id: PublicationId,
        workspace_id: WorkspaceId,
        attempt: AttemptId,
        cancel: &CancellationToken,
        point: Option<&mut PointContext>,
    ) -> Result<Vec<MemberDescriptor>, WorkflowError> {
        let spec = |id: &SemanticId| {
            self.registry
                .relation_by_id(*id)
                .ok_or_else(|| contract("result declaration missing"))
        };
        let mut rows = BTreeMap::new();
        for (id, batch) in tables {
            rows.insert(spec(id)?.key, batch.clone());
        }
        let session = self
            .sessions
            .candidate_checked(rows, self.registry.clone(), cancel)?;
        let mut outputs = BTreeMap::new();
        let mut destinations = BTreeMap::new();
        for id in tables.keys() {
            let key = spec(id)?.key;
            let source = ResolvedTableReference {
                catalog: "workspace".into(),
                schema: key.namespace.as_str().into(),
                table: key.name.into(),
            };
            let output = ResolvedTableReference {
                catalog: catalog.into(),
                schema: source.schema.clone(),
                table: source.table.clone(),
            };
            destinations.insert(
                output.clone(),
                prefix
                    .join(&format!("{}/{}/", output.schema, output.table))
                    .map_err(|error| contract(error.to_string()))?,
            );
            outputs.insert(
                output,
                RelationOutput {
                    relation_id: *id,
                    plan: session.relation_plan(&source)?.plan().clone(),
                },
            );
        }
        let artifact =
            ArtifactPlan::new(session, outputs, cancel)?.with_operation(attempt.into())?;
        let header = publication_manifests::Row {
            publication_id,
            workspace_id,
            parent_publication_id: None,
            attempt_id: attempt,
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
            windows: vec![],
            exported_at: None,
            export_lease_id: None,
            export_expires_at: None,
            maintenance_epoch: None,
            store_fingerprint: None,
        };
        let (command, ticket) =
            artifact.prepare_publication(header, destinations, vec![], cancel)?;
        if let Some(point) = point {
            let operations = self.operations()?;
            operations
                .store()
                .studies()
                .record_receipt(
                    point.study_id,
                    pse_model::study::OccurrenceKey(point.point_index),
                    point.job_id,
                    point.attempt_id,
                    operations.worker(),
                    point.revision,
                    point.scientific.clone(),
                    point.diagnostic.clone(),
                    document(&ticket, "point receipt")?,
                )
                .await?;
            point.revision += 1;
            point.effect = pse_model::study::EffectState::Unknown;
            let completed = command.execute(cancel).await?;
            let members = candidate_record(&completed, &self.registry)?.members;
            point.effect = pse_model::study::EffectState::Idempotent;
            return Ok(members);
        }
        let completed = command.execute(cancel).await?;
        Ok(candidate_record(&completed, &self.registry)?.members)
    }

    /// Start a durable study (Plan 22 O7): store its sources and definition, and create
    /// the study, its coordinating attempt, its one publication intent in `workspace`, a
    /// job per point and its waiting finalization, in one store transaction. Workers run
    /// the points; the returned handle follows, cancels and reads the study.
    ///
    /// # Errors
    /// An ephemeral runtime; no points, more than [`MAXIMUM_STUDY_POINTS`], an invalid
    /// occurrence graph or physically incompatible assignment; an invalid
    /// workspace root; store failures.
    pub async fn start_study(
        &self,
        workspace: &Workspace,
        plan: StudyPlan,
    ) -> Result<StudyHandle, WorkflowError> {
        let operations = self.operations()?;
        if plan.points.is_empty() || plan.points.len() > MAXIMUM_STUDY_POINTS {
            return Err(contract(format!(
                "a study holds between 1 and {MAXIMUM_STUDY_POINTS} points"
            )));
        }
        let physical = operations.put_sources(&plan.sources.physical).await?;
        let mut modeling = Vec::with_capacity(plan.sources.modeling.len());
        for bundle in &plan.sources.modeling {
            modeling.push(operations.put_sources(bundle).await?);
        }
        let cancel = crate::CancelSource::new();
        let package = self.package_from_sources(
            &plan.sources.modeling,
            self.physical_from_sources(&plan.sources.physical, &cancel)
                .await?,
        )?;
        let definition = package
            .admit_study_points(physical, modeling, &plan.points, &cancel)
            .await?;
        self.start_defined_study(
            workspace,
            plan.sources,
            definition,
            plan.retry,
            plan.priority,
        )
        .await
    }

    /// Persist and execute the same admitted definition accepted by the in-process adapter.
    pub async fn start_defined_study(
        &self,
        workspace: &Workspace,
        sources: PackageSources,
        definition: StudyDefinition,
        retry: RetryPolicy,
        priority: i32,
    ) -> Result<StudyHandle, WorkflowError> {
        let operations = self.operations()?;
        let physical = operations.put_sources(&sources.physical).await?;
        let mut modeling = Vec::with_capacity(sources.modeling.len());
        for bundle in &sources.modeling {
            modeling.push(operations.put_sources(bundle).await?);
        }
        if physical != definition.physical || modeling != definition.modeling {
            return Err(contract("admitted study source bundle identities differ"));
        }
        let cancel = crate::CancelSource::new();
        let package = self.package_from_sources(
            &sources.modeling,
            self.physical_from_sources(&sources.physical, &cancel)
                .await?,
        )?;
        pse_operations::study_policy::admit(&definition.graph()).map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        definition.validate_roles()?;
        if definition.points.len() > MAXIMUM_STUDY_POINTS {
            return Err(contract("study point extent exceeds its limit"));
        }
        for point in &definition.points {
            if point.binding.identity() != point.binding_hash {
                return Err(contract("immutable study binding identity differs"));
            }
            let prepared = package
                .prepare_bound_operation(&point.operation, &point.binding, &cancel)
                .await?;
            if prepared.seed_need() != point.policy.seed_need {
                return Err(contract("immutable study seed need differs from owner"));
            }
        }
        let request_identity = identity(pse_ids::Frame::DurableStudyRequestV1, &definition)?;
        let study_id: StudyId = pse_operations::mint_id();
        let attempt_id: AttemptId = pse_operations::mint_id();
        let run_id: RunId = pse_operations::mint_id();
        let publication_id: PublicationId = pse_operations::mint_id();
        let job = |attempt: NewAttempt, key: String, task: JobTask| {
            Ok::<_, WorkflowError>(NewJob {
                attempt,
                idempotency_key: key,
                payload_version: JOB_PAYLOAD_VERSION,
                payload: document(&JobPayload::new(task), "job payload")?,
                priority,
                retry,
            })
        };
        let mut new_points = Vec::with_capacity(definition.points.len());
        for point in &definition.points {
            let point_index = point.policy.key.0;
            let operation_job = StudyOperationJob {
                physical: definition.physical,
                modeling: definition.modeling.clone(),
                point: StudyPointBinding {
                    study_id,
                    point_index,
                    binding_hash: point.binding_hash,
                    binding: point.binding.clone(),
                    policy: point.policy.clone(),
                    operation: point.operation.clone(),
                },
            };
            let attempt = NewAttempt {
                attempt_id: pse_operations::mint_id(),
                run_id: pse_operations::mint_id(),
                kind: match point.operation.operation {
                    OperationRequest::DeclaredCase(_) | OperationRequest::Horizon(_) => {
                        AttemptKind::Modeling
                    }
                    OperationRequest::Simulation(_) => AttemptKind::Simulation,
                    OperationRequest::Fit(_) => AttemptKind::Fit,
                },
                request_identity: identity(pse_ids::Frame::DurableJobRequestV2, &operation_job)?,
                preparation_identity: None,
                parent_attempt: None,
            };
            let mut point_job = job(
                attempt,
                format!("study:{study_id}:point:{point_index}"),
                JobTask::StudyOperation(Box::new(operation_job)),
            )?;
            point_job.retry.max_tries = point.policy.attempt_limit;
            new_points.push(NewPoint {
                binding_hash: point.binding_hash,
                policy: point.policy.clone(),
                job: point_job,
            });
        }
        let finalization = job(
            NewAttempt {
                attempt_id: pse_operations::mint_id(),
                run_id,
                kind: AttemptKind::StudyFinalization,
                request_identity,
                preparation_identity: None,
                parent_attempt: None,
            },
            format!("study:{study_id}:finalization"),
            JobTask::StudyFinalization(StudyFinalization { study_id }),
        )?;
        let member_prefix = workspace.member_prefix(attempt_id, publication_id)?;
        operations
            .store()
            .studies()
            .create(&NewStudy {
                study_id,
                attempt: NewAttempt {
                    attempt_id,
                    run_id,
                    kind: AttemptKind::Study,
                    request_identity,
                    preparation_identity: None,
                    parent_attempt: None,
                },
                intent: NewIntent {
                    publication_id,
                    workspace_id: workspace.workspace_id,
                    attempt_id,
                    member_prefix: member_prefix.to_string(),
                },
                definition: document(&definition, "study definition")?,
                finalization,
                points: new_points,
            })
            .await?;
        Ok(self.study(study_id))
    }

    /// A handle on a durable study of this runtime's store.
    pub fn study(&self, study_id: StudyId) -> StudyHandle {
        StudyHandle {
            runtime: self.clone(),
            study_id,
        }
    }

    /// The store's studies, newest first, as the registry relation
    /// `runtime.operational_studies`.
    ///
    /// # Errors
    /// An ephemeral runtime; store failures; a stored value outside the registry contract.
    pub async fn studies(&self, filter: &StudyFilter) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::operational_studies as studies;
        let records = self.operations()?.store().studies().list(filter).await?;
        let mut rows = studies::Builder::with_registry(&self.registry, records.len())
            .map_err(super::relation)?;
        for row in records {
            rows.push(row).map_err(super::relation)?;
        }
        rows.finish().map_err(super::relation)
    }

    /// Publish a concluded study (its finalization job's task): write the summary relation
    /// `runtime.study_outcomes` under the study's intent and commit one publication of the
    /// study's attempt with the summary and every completed point's members, re-reading
    /// the workspace head when another publication advanced it. Idempotent: a study whose
    /// publication is committed is only marked published.
    ///
    /// # Errors
    /// A study that has not concluded; a head that kept moving; member write, admission or
    /// store failures.
    pub(super) async fn finalize_study(
        &self,
        operations: &Operations,
        study: StudyId,
        cancel: &crate::CancelSource,
    ) -> Result<Published, WorkflowError> {
        let store = operations.store();
        let record = store.studies().get(study).await?;
        let catalog = store.catalog();
        let publication_id = record.study.publication_id;
        let intent =
            catalog
                .intent(publication_id)
                .await?
                .ok_or_else(|| OperationsError::NotFound {
                    entity: "publication intent",
                    id: publication_id.to_string(),
                })?;
        let published = |parent| Published {
            publication_id,
            workspace_id: intent.workspace_id,
            parent,
            attempt_id: record.study.attempt_id,
        };
        if let Some(existing) = catalog.publication(publication_id).await? {
            store.studies().mark_published(study).await?;
            return Ok(published(existing.publication.parent_publication));
        }
        if record.study.state != StudyState::Concluded {
            return Err(contract(format!(
                "study {study} is {}; only a concluded study is published",
                record.study.state.as_str()
            )));
        }
        let definition: StudyDefinition = serde_json::from_str(&record.study.definition)
            .map_err(|e| contract(format!("study {study} definition: {e}")))?;
        let point_members = store.studies().available_members(study).await?;
        let available = point_members.iter().map(|(key, _)| *key).collect();
        let summary = self.study_outcomes(&record, &definition, &available)?;
        let token = cancel.token();
        let prefix = prefix_of(&intent.member_prefix)?
            .join("summary/")
            .map_err(|error| contract(error.to_string()))?;
        let mut members = self
            .write_members(
                &BTreeMap::from([(study_outcomes::RELATION_ID, summary)]),
                "study",
                &prefix,
                publication_id,
                intent.workspace_id,
                record.study.attempt_id,
                &token,
                None,
            )
            .await?;
        members.extend(point_members.into_iter().map(|(_, member)| member));
        for _ in 0..COMMIT_ATTEMPTS {
            let head = catalog.head(intent.workspace_id).await?;
            let request = PublicationCommit {
                publication_id,
                workspace_id: intent.workspace_id,
                attempt_id: record.study.attempt_id,
                expected_parent: head,
                kind: PublicationKind::Relations,
                members: members.clone(),
                inputs: Vec::new(),
                windows: Vec::new(),
            };
            match catalog.commit(&request).await {
                Ok(_) => {
                    store.studies().mark_published(study).await?;
                    return Ok(published(head));
                }
                Err(OperationsError::PublicationConflict { .. }) => {}
                // A superseded try of this finalization committed the study meanwhile.
                Err(error @ OperationsError::PublicationIdentityReused { .. }) => {
                    let Some(existing) = catalog.publication(publication_id).await? else {
                        return Err(error.into());
                    };
                    store.studies().mark_published(study).await?;
                    return Ok(published(existing.publication.parent_publication));
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(contract(format!(
            "the head of workspace {} moved {COMMIT_ATTEMPTS} times while study {study} was published",
            intent.workspace_id
        )))
    }

    /// The summary relation of a concluded study: one row per point.
    fn study_outcomes(
        &self,
        record: &StudyRecord,
        definition: &StudyDefinition,
        available: &std::collections::BTreeSet<u32>,
    ) -> Result<FieldCheckedBatch, WorkflowError> {
        let mut rows = study_outcomes::Builder::with_registry(&self.registry, record.points.len())
            .map_err(super::relation)?;
        for point in &record.points {
            let defined = definition
                .points
                .iter()
                .find(|defined| defined.policy.key.0 == point.point_index)
                .ok_or_else(|| contract("study occurrence outside its definition"))?;
            let outcome = point.outcome.as_ref().ok_or_else(|| {
                contract("historical study outcome unavailable; explicit readmission required")
            })?;
            let mut row = super::study_tables::outcome_row(record.study.study_id, defined, outcome);
            row.member_catalog = available
                .contains(&point.point_index)
                .then(|| point_catalog(point.point_index));
            rows.push(row).map_err(super::relation)?;
        }
        rows.finish().map_err(super::relation)
    }
}

impl super::RunResult {
    /// Write this completed study point try's result tables under the study's intent
    /// (`points/{index}/{attempt}/`), named in catalog `point_{index}`, with receipts
    /// naming `attempt`. Nothing becomes visible until the study's publication commits.
    pub(crate) async fn write_point_members(
        &self,
        point: &mut PointContext,
        attempt: AttemptId,
    ) -> Result<Vec<MemberDescriptor>, WorkflowError> {
        let tables = self.tables().map_err(WorkflowError::Shared)?;
        let prefix = point
            .member_prefix
            .join(&format!("points/{}/{attempt}/", point.point_index))
            .map_err(|error| contract(error.to_string()))?;
        self.runtime
            .write_members(
                tables,
                &point_catalog(point.point_index),
                &prefix,
                point.publication_id,
                point.workspace_id,
                attempt,
                &CancellationToken::new(),
                Some(point),
            )
            .await
    }
}

/// One point of a study's status.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointStatus {
    /// The point.
    pub point_index: u32,
    /// Its state.
    pub state: StudyPointState,
    /// The earlier point that seeds it.
    pub outcome: Option<PointOutcome>,
    /// Historical unavailable policy attribution, preserved without readmission.
    pub legacy: Option<pse_operations::studies::LegacyUnavailable>,
    /// Its latest try.
    pub attempt_id: AttemptId,
    /// That try's state.
    pub attempt_state: AttemptState,
    /// Its job's state.
    pub job_state: JobState,
    /// Why the point failed or was cancelled.
    pub error: Option<String>,
}

/// A durable study's status: its coordination state, its own attempt's state, its
/// finalization and publication, and every point.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyStatus {
    /// The study.
    pub study_id: StudyId,
    /// Open, concluded or published.
    pub state: StudyState,
    /// The study's own attempt: the attempt its publication names.
    pub attempt_id: AttemptId,
    /// Queued while points run; then completed, partial, failed or cancelled.
    pub attempt_state: AttemptState,
    /// Its publication's identity, registered at creation.
    pub publication_id: PublicationId,
    /// The finalization job's state.
    pub finalization: JobState,
    /// Why the finalization last failed.
    pub finalization_error: Option<String>,
    /// Every point, in index order.
    pub points: Vec<PointStatus>,
}

impl From<StudyRecord> for StudyStatus {
    fn from(record: StudyRecord) -> Self {
        Self {
            study_id: record.study.study_id,
            state: record.study.state,
            attempt_id: record.study.attempt_id,
            attempt_state: record.attempt.state,
            publication_id: record.study.publication_id,
            finalization: record.finalization.state,
            finalization_error: record.finalization.last_error,
            points: record
                .points
                .into_iter()
                .map(|point| PointStatus {
                    point_index: point.point_index,
                    state: point.state,
                    outcome: point.outcome,
                    legacy: point.legacy,
                    attempt_id: point.attempt_id,
                    attempt_state: point.attempt_state,
                    job_state: point.job_state,
                    error: point.last_error,
                })
                .collect(),
        }
    }
}

/// A durable study: status, cancellation, waiting for its publication.
#[derive(Clone, Debug)]
pub struct StudyHandle {
    runtime: Runtime,
    study_id: StudyId,
}

impl StudyHandle {
    /// The study.
    pub const fn study_id(&self) -> StudyId {
        self.study_id
    }

    /// The study as the store holds it now.
    ///
    /// # Errors
    /// An ephemeral runtime; an unknown study; store failures.
    pub async fn status(&self) -> Result<StudyStatus, WorkflowError> {
        Ok(self
            .runtime
            .operations()?
            .store()
            .studies()
            .get(self.study_id)
            .await?
            .into())
    }

    /// Cancel the study: points that have not started are cancelled, running tries are
    /// asked to stop, and the study concludes as cancelled once none runs; what completed
    /// is still published.
    ///
    /// # Errors
    /// An ephemeral runtime; an unknown study; store failures.
    pub async fn cancel(&self) -> Result<StudyCancel, WorkflowError> {
        let operations = self.runtime.operations()?;
        Ok(operations
            .store()
            .studies()
            .cancel(self.study_id, operations.worker())
            .await?)
    }

    /// The study's publication, once committed.
    ///
    /// # Errors
    /// An ephemeral runtime; an unknown study; store failures.
    pub async fn result(&self) -> Result<Option<Published>, WorkflowError> {
        let operations = self.runtime.operations()?;
        let store = operations.store();
        let study = store.studies().row(self.study_id).await?;
        if study.state != StudyState::Published {
            return Ok(None);
        }
        let record = store
            .catalog()
            .publication(study.publication_id)
            .await?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication",
                id: study.publication_id.to_string(),
            })?;
        Ok(Some(Published {
            publication_id: study.publication_id,
            workspace_id: record.publication.workspace_id,
            parent: record.publication.parent_publication,
            attempt_id: study.attempt_id,
        }))
    }

    /// Wait until the study is published, polling its state every `poll`.
    ///
    /// # Errors
    /// A finalization that failed or was cancelled for good; an ephemeral runtime; store
    /// failures.
    pub async fn wait(&self, poll: Duration) -> Result<Published, WorkflowError> {
        loop {
            if let Some(published) = self.result().await? {
                return Ok(published);
            }
            let store = self.runtime.operations()?.store();
            let study = store.studies().row(self.study_id).await?;
            let finalization = store.jobs().get(study.finalization_job).await?;
            if matches!(finalization.state, JobState::Failed | JobState::Cancelled) {
                return Err(contract(format!(
                    "study {} was not published: its finalization {}{}",
                    self.study_id,
                    finalization.state.as_str(),
                    finalization
                        .last_error
                        .map(|error| format!(" ({error})"))
                        .unwrap_or_default()
                )));
            }
            tokio::time::sleep(poll).await;
        }
    }
}
