// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable studies across workers (Plan 22 O7; architecture S15).
//!
//! [`Runtime::start_study`] stores a study's sources, its typed definition and, in one store
//! transaction, the study's coordinating attempt, its one publication intent (Plan 22 X9),
//! a job per point (payload v3 with the point's [`StudyPointBinding`]) and a waiting
//! finalization job. Workers claim points concurrently:
//! - a point with a predecessor becomes claimable only once the predecessor completed, and
//!   starts from the predecessor's stored solution (`StartSource::Stored`), falling back to
//!   its authored start with a recorded reason when that solution does not fit;
//! - a completed try writes the point's result members under the intent's prefix
//!   (`points/{index}/{attempt}/`), named in catalog `point_{index}`, with receipts naming
//!   the point's attempt, and records them with the point;
//! - a failed point contributes no members and is recorded in the summary; the points
//!   that wait on it are cancelled.
//!
//! The transaction that makes the last point terminal ends the study's attempt and
//! releases the finalization job, which writes the summary relation `runtime.study_outcomes`
//! under the same prefix and commits one publication of the study's attempt containing
//! the summary and every completed point's members. The ephemeral in-process study
//! ([`super::ModelingPackage::study`]) stays the library path; this is the one durable path.
use super::{
    Operations, Runtime, WorkflowError, contract,
    publication::{Published, Workspace, candidate_record},
    worker::{
        AppliedStart, JOB_PAYLOAD_VERSION, JobPayload, JobStart, JobTask, ModelingJob,
        PointOverlay, StudyFinalization, StudyPointBinding,
    },
};
use datafusion::common::ResolvedTableReference;
use pse_catalog::artifact::{ArtifactPlan, RelationOutput};
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, SemanticId};
use pse_model::{
    document::Version,
    generated::{
        enums::{ModelingAnalysisRoute, PublicationKind},
        identities::DeclarationId,
    },
};
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

/// The authored sources of a package closure, as texts by path: what a worker loads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageSources {
    /// The physical package's documents.
    pub physical: BTreeMap<String, String>,
    /// The modeling package closure's documents, one map per package in load order.
    pub modeling: Vec<BTreeMap<String, String>>,
}

/// One point of a study to start.
#[derive(Clone, Debug, PartialEq)]
pub struct StudyPoint {
    /// The authored case the point solves.
    pub case: DeclarationId,
    /// The values the point replaces in its case.
    pub overlay: PointOverlay,
    /// The earlier point whose stored solution this one starts from.
    pub predecessor: Option<u32>,
}

/// A durable study to start.
#[derive(Clone, Debug)]
pub struct StudyPlan {
    /// The package closure's sources, stored content-addressed for the workers.
    pub sources: PackageSources,
    /// The analysis route of every point.
    pub route: ModelingAnalysisRoute,
    /// The solve settings of every point.
    pub settings: crate::math::settings::SolveSettings,
    /// The points, in index order.
    pub points: Vec<StudyPoint>,
    /// How often each point and the finalization may be tried.
    pub retry: RetryPolicy,
    /// The priority of the study's jobs.
    pub priority: i32,
}

/// Version 1 of a study's definition: the store's `definition` document and the content of
/// the study's request identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyDefinition {
    /// Document version.
    pub version: Version<1>,
    /// The source bundle of the physical package.
    pub physical: ContentHash,
    /// The source bundles of the modeling package closure, in load order.
    pub modeling: Vec<ContentHash>,
    /// The analysis route of every point.
    pub route: ModelingAnalysisRoute,
    /// The solve settings of every point.
    pub settings: crate::math::settings::SolveSettings,
    /// The points, in index order.
    pub points: Vec<StudyPointDefinition>,
}

/// One point of a study's definition.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointDefinition {
    /// The authored case.
    pub case: DeclarationId,
    /// The hash of the point's value bindings.
    pub binding_hash: ContentHash,
    /// The values the point replaces in its case.
    #[serde(default)]
    pub overlay: PointOverlay,
    /// The earlier point that seeds it.
    #[serde(default)]
    pub predecessor: Option<u32>,
}

/// What a point's binding hash frames: its case, route and overlay.
#[derive(serde::Serialize)]
struct BindingContent<'a> {
    case: DeclarationId,
    route: ModelingAnalysisRoute,
    overlay: &'a PointOverlay,
}

/// Where a study point's try writes its result members: the study's intent.
#[derive(Clone, Debug)]
pub(crate) struct PointContext {
    pub(crate) point_index: u32,
    pub(crate) publication_id: PublicationId,
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) member_prefix: url::Url,
}

/// The catalog a completed point's result members are published under.
fn point_catalog(point: u32) -> String {
    format!("point_{point}")
}

fn identity(frame: pse_ids::Frame, value: &impl serde::Serialize) -> Result<ContentHash, WorkflowError> {
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
            point_index: binding.point_index,
            publication_id: intent.publication_id,
            workspace_id: intent.workspace_id,
            member_prefix: prefix_of(&intent.member_prefix)?,
        })
    }
}

/// Start a study point from its predecessor's stored solution: the newest one the
/// predecessor's completed try stored for this preparation's coordinates and backend. When
/// none fits, the point starts from its authored start and the event records why.
///
/// # Errors
/// A predecessor that has not completed; store failures.
pub(super) async fn predecessor_start(
    operations: &Operations,
    binding: &StudyPointBinding,
    predecessor: u32,
    prepared: super::ModelingSolvePreparation,
) -> Result<(super::ModelingSolvePreparation, AppliedStart), WorkflowError> {
    let applied = |solution, fresh: Option<String>| AppliedStart {
        requested: "predecessor",
        solution,
        fresh,
    };
    let point = operations
        .store()
        .studies()
        .point(binding.study_id, predecessor)
        .await?;
    if point.state != StudyPointState::Completed {
        return Err(contract(format!(
            "predecessor point {predecessor} of study {} is {}; a point starts only after its predecessor completed",
            binding.study_id,
            point.state.as_str()
        )));
    }
    let (Some(target), Some(preparation)) = (
        prepared.solve.compatibility().cloned(),
        prepared.solve.seed_preparation_identity(),
    ) else {
        return Ok((
            prepared,
            applied(None, Some("a constant evaluation consumes no seed".to_owned())),
        ));
    };
    let found = operations
        .store()
        .solutions()
        .latest_of_attempt(point.attempt_id, &target.layout, &preparation, target.backend)
        .await?;
    let Some(found) = found else {
        return Ok((
            prepared,
            applied(
                None,
                Some(format!(
                    "predecessor point {predecessor} stored no solution for this point's coordinates and backend"
                )),
            ),
        ));
    };
    let solution = found.solution_id;
    match prepared
        .clone()
        .with_stored_start(operations, super::StoredStart::Solution(solution))
        .await
    {
        Ok(seeded) => Ok((seeded, applied(Some(solution), None))),
        Err(error @ WorkflowError::Operations(_)) => Err(error),
        Err(error) => Ok((
            prepared,
            applied(
                None,
                Some(format!(
                    "predecessor point {predecessor}'s stored solution {solution} does not fit: {error}"
                )),
            ),
        )),
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
            publication_id: publication_id.into(),
            workspace_id: workspace_id.into(),
            parent_publication_id: None,
            attempt_id: attempt.into(),
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
        let (command, _ticket) =
            artifact.prepare_publication(header, destinations, vec![], cancel)?;
        let completed = command.execute(cancel).await?;
        Ok(candidate_record(&completed, &self.registry)?.members)
    }

    /// Start a durable study (Plan 22 O7): store its sources and definition, and create
    /// the study, its coordinating attempt, its one publication intent in `workspace`, a
    /// job per point and its waiting finalization, in one store transaction. Workers run
    /// the points; the returned handle follows, cancels and reads the study.
    ///
    /// # Errors
    /// An ephemeral runtime; no points, more than [`MAXIMUM_STUDY_POINTS`], a predecessor
    /// that is not an earlier point, or two points with the same bindings; an invalid
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
        for (index, point) in plan.points.iter().enumerate() {
            if point
                .predecessor
                .is_some_and(|predecessor| predecessor as usize >= index)
            {
                return Err(contract(format!(
                    "study point {index}'s predecessor must identify an earlier point"
                )));
            }
        }
        let physical = operations.put_sources(&plan.sources.physical).await?;
        let mut modeling = Vec::with_capacity(plan.sources.modeling.len());
        for bundle in &plan.sources.modeling {
            modeling.push(operations.put_sources(bundle).await?);
        }
        let mut points = Vec::with_capacity(plan.points.len());
        let mut bindings = std::collections::BTreeSet::new();
        for (index, point) in plan.points.iter().enumerate() {
            let binding_hash = identity(
                pse_ids::Frame::DurableStudyPointBindingV1,
                &BindingContent {
                    case: point.case,
                    route: plan.route,
                    overlay: &point.overlay,
                },
            )?;
            if !bindings.insert(binding_hash) {
                return Err(contract(format!(
                    "study point {index} repeats the bindings of an earlier point"
                )));
            }
            points.push(StudyPointDefinition {
                case: point.case,
                binding_hash,
                overlay: point.overlay.clone(),
                predecessor: point.predecessor,
            });
        }
        let definition = StudyDefinition {
            version: Version,
            physical,
            modeling,
            route: plan.route,
            settings: plan.settings,
            points,
        };
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
                priority: plan.priority,
                retry: plan.retry,
            })
        };
        let mut new_points = Vec::with_capacity(definition.points.len());
        for (index, point) in definition.points.iter().enumerate() {
            let point_index = u32::try_from(index)
                .map_err(|_| contract("study point index exceeds its range"))?;
            let modeling_job = ModelingJob {
                physical: definition.physical,
                modeling: definition.modeling.clone(),
                case: point.case,
                route: definition.route,
                settings: definition.settings.clone(),
                start: JobStart::Fresh,
                study: Some(StudyPointBinding {
                    study_id,
                    point_index,
                    binding_hash: point.binding_hash,
                    overlay: point.overlay.clone(),
                    predecessor: point.predecessor,
                }),
            };
            let attempt = NewAttempt {
                attempt_id: pse_operations::mint_id(),
                run_id: pse_operations::mint_id(),
                kind: AttemptKind::Modeling,
                request_identity: modeling_job.request_identity()?,
                preparation_identity: None,
                parent_attempt: None,
            };
            new_points.push(NewPoint {
                binding_hash: point.binding_hash,
                predecessor: point.predecessor,
                job: job(
                    attempt,
                    format!("study:{study_id}:point:{index}"),
                    JobTask::Modeling(modeling_job),
                )?,
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
        let intent = catalog
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
        let summary = self.study_outcomes(&record, &definition)?;
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
            )
            .await?;
        members.extend(
            store
                .studies()
                .completed_members(study)
                .await?
                .into_iter()
                .map(|(_, member)| member),
        );
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
    ) -> Result<FieldCheckedBatch, WorkflowError> {
        let mut rows = study_outcomes::Builder::with_registry(&self.registry, record.points.len())
            .map_err(super::relation)?;
        for point in &record.points {
            let defined = definition
                .points
                .get(point.point_index as usize)
                .ok_or_else(|| contract("study point outside its definition"))?;
            rows.push(study_outcomes::Row {
                study_id: record.study.study_id,
                point_index: i64::from(point.point_index),
                case_id: defined.case,
                binding_hash: point.binding_hash,
                predecessor: point.predecessor.map(i64::from),
                state: point.state,
                attempt_id: point.attempt_id,
                attempt_state: point.attempt_state,
                member_catalog: (point.state == StudyPointState::Completed)
                    .then(|| point_catalog(point.point_index)),
                error: match point.state {
                    StudyPointState::Failed | StudyPointState::Cancelled => Some(
                        point
                            .last_error
                            .clone()
                            .unwrap_or_else(|| format!("the point's try ended {}", point.attempt_state.as_str())),
                    ),
                    _ => None,
                },
            })
            .map_err(super::relation)?;
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
        point: &PointContext,
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
            )
            .await
    }
}

/// One point of a study's status.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointStatus {
    /// The point.
    pub point_index: u32,
    /// Its state.
    pub state: StudyPointState,
    /// The earlier point that seeds it.
    pub predecessor: Option<u32>,
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
                    predecessor: point.predecessor,
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
