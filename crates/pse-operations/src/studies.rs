// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Studies coordinated across workers (Plan 22 O7; architecture S15).
//!
//! **Creation.** One transaction stores the study, its coordinating attempt (kind `study`),
//! its publication intent, its finalization job and every point's job with its attempt.
//! A point without a predecessor is queued at once; a point with one, and the
//! finalization, wait (job state `waiting`, attempt `planned`).
//!
//! **A point follows its job.** Every repository function that moves a job — a claim, a
//! finished try, the stale sweep, a cancellation — calls [`job_changed`] in its own
//! transaction, so a point is pending while its job is waiting or queued, assigned while a
//! try runs, and completed, failed or cancelled with its job. A completed point's result
//! members are recorded in that same transaction.
//!
//! **Predecessors.** A completed point releases the points that name it as their
//! predecessor. A point that fails or is cancelled cancels them instead, transitively:
//! they never ran, so their attempts end cancelled with the `unattempted` termination and
//! their jobs record which predecessor did not complete.
//!
//! **Conclusion.** The transaction that makes the last point terminal ends the study's
//! attempt — completed when every point completed, partial when some did, failed when none
//! did, cancelled when the study was cancelled — and releases the finalization job, which
//! publishes the study. A coordinating attempt never holds a lease
//! ([`crate::lifecycle::COORDINATING`]), so it never goes stale and its intent stays live.
//!
//! Point transitions of one study are serialized by the study row. Lock order: job rows,
//! then their attempts, then the study row, then the jobs and attempts of the points it
//! releases or cancels, then the study's own attempt.

use pse_ids::ContentHash;
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::{catalog as catalog_statements, studies as statements};

use crate::attempts::{
    self, AttemptId, NewAttempt, RuntimeOperationalAttemptsRow, RuntimeTermination, Termination,
    TerminationCode, TransitionNote, Tx,
};
use crate::cancellation::CancelOutcome;
use crate::catalog::{MemberDescriptor, NewIntent, PublicationId};
use crate::error::{Classify, OperationsError, Target};
use crate::jobs::{self, JobId, JobState, NewJob, RuntimeOperationalJobsRow};
use crate::lifecycle::AttemptState;
use crate::store::Store;
pub use pse_model::generated::enums::{StudyPointState, StudyState};
pub use pse_model::generated::identities::StudyId;
pub use pse_model::generated::runtime::operational_studies::RuntimeOperationalStudiesRow;
pub use pse_model::generated::runtime::operational_study_point_members::RuntimeOperationalStudyPointMembersRow;
pub use pse_model::generated::runtime::operational_study_points::RuntimeOperationalStudyPointsRow;

/// The actor recorded on the transitions a study makes itself.
const ACTOR: &str = "study";

/// A study to create, with every identity minted by the runtime.
#[derive(Clone, Debug, PartialEq)]
pub struct NewStudy {
    /// The study identity.
    pub study_id: StudyId,
    /// The coordinating attempt: kind `study`. The study's publication names it.
    pub attempt: NewAttempt,
    /// The publication intent, registered for the coordinating attempt before any point
    /// writes a member under its prefix (Plan 22 X9).
    pub intent: NewIntent,
    /// The study's versioned definition document.
    pub definition: serde_json::Value,
    /// The finalization job: it waits until the last point is terminal.
    pub finalization: NewJob,
    /// The points, in index order.
    pub points: Vec<NewPoint>,
}

/// One point of a new study.
#[derive(Clone, Debug, PartialEq)]
pub struct NewPoint {
    /// The hash of the point's value bindings; unique within the study.
    pub binding_hash: ContentHash,
    /// The earlier point whose stored solution seeds this one; the point waits for it.
    pub predecessor: Option<u32>,
    /// The point's job and first attempt.
    pub job: NewJob,
}

/// What [`Studies::create`] stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StudyCreated {
    /// The study.
    pub study_id: StudyId,
    /// Its coordinating attempt.
    pub attempt_id: AttemptId,
    /// Its publication's registered intent.
    pub publication_id: PublicationId,
    /// Its finalization job.
    pub finalization_job: JobId,
    /// Each point's job, in index order.
    pub point_jobs: Vec<JobId>,
}

/// One point with its job's state and current attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointStatus {
    /// The point.
    pub point_index: u32,
    /// Its value bindings by hash.
    pub binding_hash: ContentHash,
    /// The point that seeds it.
    pub predecessor: Option<u32>,
    /// Its state.
    pub state: StudyPointState,
    /// Its job.
    pub job_id: JobId,
    /// The job's state.
    pub job_state: JobState,
    /// The job's current attempt: the point's latest try.
    pub attempt_id: AttemptId,
    /// That attempt's state.
    pub attempt_state: AttemptState,
    /// Why the job last failed or was cancelled.
    pub last_error: Option<String>,
}

/// A study as the store holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct StudyRecord {
    /// The study row.
    pub study: RuntimeOperationalStudiesRow,
    /// Its coordinating attempt.
    pub attempt: RuntimeOperationalAttemptsRow,
    /// Its finalization job.
    pub finalization: RuntimeOperationalJobsRow,
    /// Every point, in index order.
    pub points: Vec<PointStatus>,
}

/// Which studies [`Studies::list`] returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StudyFilter {
    /// Only studies in these states; every state when empty.
    pub states: Vec<StudyState>,
    /// At most this many, newest first.
    pub limit: i64,
}

impl StudyFilter {
    /// The newest `limit` studies of every state.
    pub const fn newest(limit: i64) -> Self {
        Self {
            states: Vec::new(),
            limit,
        }
    }
}

/// What cancelling a study did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StudyCancel {
    /// Points that had not started, now cancelled.
    pub cancelled: Vec<u32>,
    /// Points whose running try was asked to stop; each ends through its worker.
    pub stopping: Vec<u32>,
    /// The study concluded in this call (no try was still running).
    pub concluded: bool,
    /// The study had already concluded; nothing changed.
    pub already_concluded: bool,
}

fn index(value: u32) -> Result<i32, OperationsError> {
    i32::try_from(value).map_err(|_| OperationsError::InvalidRequest {
        reason: format!("point index {value} exceeds the store's range"),
    })
}

fn stored_index(value: i32) -> Result<u32, OperationsError> {
    u32::try_from(value).map_err(|_| OperationsError::CorruptValue {
        column: "study_points.point_index",
        detail: format!("negative point index {value}"),
    })
}

/// The point state a job's state implies.
const fn point_state(job: JobState) -> StudyPointState {
    match job {
        JobState::Waiting | JobState::Queued => StudyPointState::Pending,
        JobState::Running => StudyPointState::Assigned,
        JobState::Completed => StudyPointState::Completed,
        JobState::Failed => StudyPointState::Failed,
        JobState::Cancelled => StudyPointState::Cancelled,
    }
}

const fn is_terminal(state: StudyPointState) -> bool {
    matches!(
        state,
        StudyPointState::Completed | StudyPointState::Failed | StudyPointState::Cancelled
    )
}

async fn lock_study(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
) -> Result<RuntimeOperationalStudiesRow, OperationsError> {
    statements::lock_study()
        .bind(tx, &study)
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "study",
            id: study.to_string(),
        })
}

async fn set_point(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: i32,
    state: StudyPointState,
) -> Result<(), OperationsError> {
    statements::set_point_state()
        .params(
            tx,
            &statements::SetPointStateParams {
                state,
                study_id: study,
                point_index: point,
            },
        )
        .await
        .classify(target)?;
    Ok(())
}

async fn set_study(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    state: StudyState,
) -> Result<(), OperationsError> {
    statements::set_study_state()
        .params(
            tx,
            &statements::SetStudyStateParams {
                state,
                study_id: study,
            },
        )
        .await
        .classify(target)?;
    Ok(())
}

async fn points(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
) -> Result<Vec<PointStatus>, OperationsError> {
    statements::point_status()
        .bind(tx, &study)
        .all()
        .await
        .classify(target)?
        .into_iter()
        .map(|row| {
            Ok(PointStatus {
                point_index: stored_index(row.point_index)?,
                binding_hash: ContentHash::try_from_slice(&row.binding_hash).map_err(
                    |error| OperationsError::CorruptValue {
                        column: "study_points.binding_hash",
                        detail: error.to_string(),
                    },
                )?,
                predecessor: row.predecessor.map(stored_index).transpose()?,
                state: row.state,
                job_id: JobId::from_id(row.job_id),
                job_state: row.job_state,
                attempt_id: AttemptId::from_id(row.attempt_id),
                attempt_state: row.attempt_state,
                last_error: row.last_error,
            })
        })
        .collect()
}

/// Cancel a point that never ran: its waiting or queued job and its planned or queued
/// attempt end cancelled, recording `reason`.
async fn cancel_unstarted(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: &RuntimeOperationalStudyPointsRow,
    termination: RuntimeTermination,
    reason: &str,
) -> Result<(), OperationsError> {
    let job = jobs::lock_job(tx, target, point.job_id).await?;
    if !matches!(job.state, JobState::Waiting | JobState::Queued) {
        return Err(OperationsError::InvalidRequest {
            reason: format!(
                "point {} of study {study} has a {} job; only work that has not started is cancelled here",
                point.point_index,
                job.state.as_str()
            ),
        });
    }
    let note = TransitionNote::by(ACTOR)
        .because(reason)
        .terminated(Termination {
            code: TerminationCode::Runtime(termination),
            detail: None,
        });
    attempts::apply(tx, target, job.attempt_id, AttemptState::Cancelled, &note, None).await?;
    jobs::set_job_state(tx, target, job.job_id, JobState::Cancelled, Some(reason)).await?;
    set_point(tx, target, study, point.point_index, StudyPointState::Cancelled).await
}

/// Cancel the pending points that wait on `point`, and theirs in turn: `point` did not
/// complete, so they can never start.
async fn cancel_dependents(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: i32,
    outcome: StudyPointState,
) -> Result<(), OperationsError> {
    let mut pending = vec![(point, outcome)];
    while let Some((predecessor, outcome)) = pending.pop() {
        let dependents = statements::dependents()
            .params(
                tx,
                &statements::DependentsParams {
                    study_id: study,
                    point_index: predecessor,
                },
            )
            .all()
            .await
            .classify(target)?;
        let reason = format!(
            "predecessor point {predecessor} {}",
            match outcome {
                StudyPointState::Failed => "failed",
                _ => "was cancelled",
            }
        );
        for dependent in dependents {
            cancel_unstarted(
                tx,
                target,
                study,
                &dependent,
                RuntimeTermination::Unattempted,
                &reason,
            )
            .await?;
            pending.push((dependent.point_index, StudyPointState::Cancelled));
        }
    }
    Ok(())
}

/// Release the pending points that wait on a completed point.
async fn release_dependents(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: i32,
) -> Result<(), OperationsError> {
    let dependents = statements::dependents()
        .params(
            tx,
            &statements::DependentsParams {
                study_id: study,
                point_index: point,
            },
        )
        .all()
        .await
        .classify(target)?;
    let note = TransitionNote::by(ACTOR).because(format!("predecessor point {point} completed"));
    for dependent in dependents {
        jobs::release(tx, target, dependent.job_id, &note).await?;
    }
    Ok(())
}

async fn insert_members(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: i32,
    members: &[MemberDescriptor],
) -> Result<(), OperationsError> {
    for member in members {
        let (selection_kind, revision_column, revision_id) =
            crate::catalog::flattened_selection(member)?;
        statements::insert_point_member()
            .params(
                tx,
                &statements::InsertPointMemberParams {
                    study_id: study,
                    point_index: point,
                    catalog_name: member.catalog_name.as_str(),
                    schema_name: member.schema_name.as_str(),
                    table_name: member.table_name.as_str(),
                    relation_id: member.relation_id,
                    relation_version: member.relation_version,
                    contract_fingerprint: member.contract_fingerprint,
                    table_uri: member.table_uri.as_str(),
                    delta_version: member.delta_version,
                    selection_kind,
                    revision_column: revision_column.as_deref(),
                    revision_id,
                },
            )
            .await
            .classify(target)?;
    }
    Ok(())
}

/// Conclude a study whose points are all terminal: end its coordinating attempt and
/// release its finalization job.
async fn conclude(
    tx: &Tx<'_>,
    target: &Target,
    study: &RuntimeOperationalStudiesRow,
) -> Result<(), OperationsError> {
    let points = points(tx, target, study.study_id).await?;
    let total = points.len();
    let completed = points
        .iter()
        .filter(|point| point.state == StudyPointState::Completed)
        .count();
    let attempt = attempts::fetch(tx, target, study.attempt_id).await?;
    let summary = format!("{completed} of {total} points completed");
    let (state, note) = if attempt.cancel_requested {
        (
            AttemptState::Cancelled,
            TransitionNote::by(ACTOR)
                .because(format!("study cancelled; {summary}"))
                .terminated(Termination {
                    code: TerminationCode::Runtime(RuntimeTermination::Cancelled),
                    detail: None,
                }),
        )
    } else {
        let state = if completed == total {
            AttemptState::Completed
        } else if completed > 0 {
            AttemptState::Partial
        } else {
            AttemptState::Failed
        };
        (state, TransitionNote::by(ACTOR).because(summary))
    };
    attempts::apply(tx, target, study.attempt_id, state, &note, None).await?;
    let release = TransitionNote::by(ACTOR).because("every point is terminal");
    jobs::release(tx, target, study.finalization_job, &release).await?;
    set_study(tx, target, study.study_id, StudyState::Concluded).await
}

/// Bring the study point a job runs in line with the job, in the transaction that moved
/// the job; `members` are the result members a completed try wrote. Does nothing for a job
/// that runs no point.
///
/// # Errors
/// [`OperationsError::InvalidRequest`] for members recorded with anything but a completed
/// point, or for a terminal point whose job moved again; classified driver failures.
pub(crate) async fn job_changed(
    tx: &Tx<'_>,
    target: &Target,
    job: JobId,
    members: &[MemberDescriptor],
) -> Result<(), OperationsError> {
    let Some(found) = statements::point_of_job()
        .bind(tx, &job)
        .opt()
        .await
        .classify(target)?
    else {
        if members.is_empty() {
            return Ok(());
        }
        return Err(OperationsError::InvalidRequest {
            reason: format!("job {job} runs no study point, so it records no members"),
        });
    };
    let study = lock_study(tx, target, found.study_id).await?;
    // Read again under the study's lock: point rows change only under it.
    let point = statements::point()
        .params(
            tx,
            &statements::PointParams {
                study_id: found.study_id,
                point_index: found.point_index,
            },
        )
        .one()
        .await
        .classify(target)?;
    let job_row = jobs::lock_job(tx, target, job).await?;
    let next = point_state(job_row.state);
    if !members.is_empty() && next != StudyPointState::Completed {
        return Err(OperationsError::InvalidRequest {
            reason: format!(
                "point {} of study {} is {}; members are recorded only with a completed point",
                point.point_index,
                study.study_id,
                next.as_str()
            ),
        });
    }
    if next == point.state {
        return Ok(());
    }
    if is_terminal(point.state) {
        return Err(OperationsError::InvalidRequest {
            reason: format!(
                "point {} of study {} is already {}; its job cannot become {}",
                point.point_index,
                study.study_id,
                point.state.as_str(),
                job_row.state.as_str()
            ),
        });
    }
    set_point(tx, target, study.study_id, point.point_index, next).await?;
    match next {
        StudyPointState::Completed => {
            insert_members(tx, target, study.study_id, point.point_index, members).await?;
            release_dependents(tx, target, study.study_id, point.point_index).await?;
        }
        StudyPointState::Failed | StudyPointState::Cancelled => {
            cancel_dependents(tx, target, study.study_id, point.point_index, next).await?;
        }
        StudyPointState::Pending | StudyPointState::Assigned => return Ok(()),
    }
    let unfinished = statements::unfinished_points()
        .bind(tx, &study.study_id)
        .one()
        .await
        .classify(target)?;
    if unfinished == 0 && study.state == StudyState::Open {
        conclude(tx, target, &study).await?;
    }
    Ok(())
}

/// The study repository.
#[derive(Clone, Copy, Debug)]
pub struct Studies<'s> {
    store: &'s Store,
}

impl<'s> Studies<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Create a study in one transaction: its coordinating attempt (queued), its
    /// publication intent, its finalization job (waiting), and each point's job with its
    /// first attempt — queued, or waiting for its predecessor.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a study without points, an attempt of the
    /// wrong kind, an intent for another attempt, a predecessor that is not an earlier
    /// point, an invalid job, a registered publication identity or a reused idempotency
    /// key; [`OperationsError::Duplicate`] for a reused identity, binding or intent
    /// prefix; classified driver failures.
    pub async fn create(&self, study: &NewStudy) -> Result<StudyCreated, OperationsError> {
        use pse_model::generated::enums::AttemptKind;
        let invalid = |reason: String| Err(OperationsError::InvalidRequest { reason });
        if study.points.is_empty() {
            return invalid(format!("study {} has no points", study.study_id));
        }
        if study.attempt.kind != AttemptKind::Study {
            return invalid("a study's own attempt is of kind study".to_owned());
        }
        if study.finalization.attempt.kind != AttemptKind::StudyFinalization {
            return invalid("a study's finalization attempt is of kind study_finalization".to_owned());
        }
        if study.intent.attempt_id != study.attempt.attempt_id {
            return invalid("a study's publication intent names the study's own attempt".to_owned());
        }
        for (position, point) in study.points.iter().enumerate() {
            if crate::lifecycle::coordinates(point.job.attempt.kind)
                || point.job.attempt.kind == AttemptKind::StudyFinalization
            {
                return invalid(format!("point {position} runs a study attempt"));
            }
            if point
                .predecessor
                .is_some_and(|predecessor| predecessor as usize >= position)
            {
                return invalid(format!(
                    "point {position}'s predecessor is not an earlier point"
                ));
            }
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        attempts::insert(&tx, target, &study.attempt, Some(ACTOR)).await?;
        let open = TransitionNote::by(ACTOR).because("points are queued");
        attempts::apply(&tx, target, study.attempt.attempt_id, AttemptState::Queued, &open, None)
            .await?;
        let intent = catalog_statements::insert_intent()
            .params(
                &tx,
                &catalog_statements::InsertIntentParams {
                    publication_id: study.intent.publication_id,
                    workspace_id: study.intent.workspace_id,
                    attempt_id: study.intent.attempt_id,
                    member_prefix: study.intent.member_prefix.as_str(),
                },
            )
            .await
            .classify(target)?;
        if intent == 0 {
            return invalid(format!(
                "publication {} is already registered",
                study.intent.publication_id
            ));
        }
        let finalization_job: JobId = crate::mint_id();
        attempts::insert(&tx, target, &study.finalization.attempt, Some(ACTOR)).await?;
        if !jobs::insert_job(&tx, target, finalization_job, &study.finalization, JobState::Waiting)
            .await?
        {
            return invalid(format!(
                "idempotency key {} already names a job",
                study.finalization.idempotency_key
            ));
        }
        statements::insert_study()
            .params(
                &tx,
                &statements::InsertStudyParams {
                    study_id: study.study_id,
                    attempt_id: study.attempt.attempt_id,
                    publication_id: study.intent.publication_id,
                    finalization_job,
                    definition: &study.definition,
                    state: StudyState::Open,
                },
            )
            .await
            .classify(target)?;
        let mut point_jobs = Vec::with_capacity(study.points.len());
        let mut queued = false;
        for (position, point) in study.points.iter().enumerate() {
            let job_id: JobId = crate::mint_id();
            attempts::insert(&tx, target, &point.job.attempt, Some(ACTOR)).await?;
            let state = if point.predecessor.is_some() {
                JobState::Waiting
            } else {
                let note = TransitionNote::by(ACTOR).because("study point queued");
                attempts::apply(
                    &tx,
                    target,
                    point.job.attempt.attempt_id,
                    AttemptState::Queued,
                    &note,
                    None,
                )
                .await?;
                queued = true;
                JobState::Queued
            };
            if !jobs::insert_job(&tx, target, job_id, &point.job, state).await? {
                return invalid(format!(
                    "idempotency key {} already names a job",
                    point.job.idempotency_key
                ));
            }
            statements::insert_point()
                .params(
                    &tx,
                    &statements::InsertPointParams {
                        study_id: study.study_id,
                        point_index: index(u32::try_from(position).unwrap_or(u32::MAX))?,
                        binding_hash: point.binding_hash,
                        predecessor: point.predecessor.map(index).transpose()?,
                        job_id,
                        state: StudyPointState::Pending,
                    },
                )
                .await
                .classify(target)?;
            point_jobs.push(job_id);
        }
        if queued {
            attempts::notify(&tx, target, jobs::JOBS_CHANNEL, &study.study_id.to_string()).await?;
        }
        tx.commit().await.classify(target)?;
        Ok(StudyCreated {
            study_id: study.study_id,
            attempt_id: study.attempt.attempt_id,
            publication_id: study.intent.publication_id,
            finalization_job,
            point_jobs,
        })
    }

    /// Read one study: its row, coordinating attempt, finalization job and points.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn get(&self, study: StudyId) -> Result<StudyRecord, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        // One transaction, so the rows are read from one connection in one place.
        let tx = client.transaction().await.classify(target)?;
        let row = statements::study()
            .bind(&tx, &study)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "study",
                id: study.to_string(),
            })?;
        let attempt = attempts::fetch(&tx, target, row.attempt_id).await?;
        let finalization = pse_operations_queries::queries::jobs::job()
            .bind(&tx, &row.finalization_job)
            .one()
            .await
            .classify(target)?;
        let points = points(&tx, target, study).await?;
        tx.commit().await.classify(target)?;
        Ok(StudyRecord {
            study: row,
            attempt,
            finalization,
            points,
        })
    }

    /// The study whose coordinating attempt is `attempt`, if any.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn of_attempt(
        &self,
        attempt: AttemptId,
    ) -> Result<Option<RuntimeOperationalStudiesRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::study_of_attempt()
            .bind(&client, &attempt)
            .opt()
            .await
            .classify(self.target())
    }

    /// The point a job runs, if any.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn point_of_job(
        &self,
        job: JobId,
    ) -> Result<Option<RuntimeOperationalStudyPointsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::point_of_job()
            .bind(&client, &job)
            .opt()
            .await
            .classify(self.target())
    }

    /// Studies newest first, at most `filter.limit`.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a non-positive limit; classified driver
    /// failures.
    pub async fn list(
        &self,
        filter: &StudyFilter,
    ) -> Result<Vec<RuntimeOperationalStudiesRow>, OperationsError> {
        if filter.limit <= 0 {
            return Err(OperationsError::InvalidRequest {
                reason: format!("study listing limit {} is not positive", filter.limit),
            });
        }
        let client = self.store.client().await?;
        statements::list_studies()
            .params(
                &client,
                &statements::ListStudiesParams {
                    states: filter.states.as_slice(),
                    limit: filter.limit,
                },
            )
            .all()
            .await
            .classify(self.target())
    }

    /// The result members of the completed points, each with its point, in point and
    /// name order: what the study's publication commits besides its summary.
    ///
    /// # Errors
    ///
    /// [`OperationsError::CorruptValue`] for an inconsistent stored selection; classified
    /// driver failures.
    pub async fn completed_members(
        &self,
        study: StudyId,
    ) -> Result<Vec<(u32, MemberDescriptor)>, OperationsError> {
        let client = self.store.client().await?;
        statements::completed_members()
            .bind(&client, &study)
            .all()
            .await
            .classify(self.target())?
            .into_iter()
            .map(|row| {
                let selection = crate::catalog::stored_selection(
                    row.selection_kind,
                    row.revision_column.as_ref(),
                    row.revision_id,
                )
                .ok_or_else(|| OperationsError::CorruptValue {
                    column: "study_point_members.selection_kind",
                    detail: format!(
                        "member {}.{}.{} of point {} has an inconsistent selection",
                        row.catalog_name, row.schema_name, row.table_name, row.point_index
                    ),
                })?;
                Ok((
                    stored_index(row.point_index)?,
                    MemberDescriptor {
                        catalog_name: row.catalog_name,
                        schema_name: row.schema_name,
                        table_name: row.table_name,
                        relation_id: row.relation_id,
                        relation_version: row.relation_version,
                        contract_fingerprint: row.contract_fingerprint,
                        table_uri: row.table_uri,
                        delta_version: row.delta_version,
                        selection,
                    },
                ))
            })
            .collect()
    }

    /// Cancel a study: its points that have not started are cancelled, each running try
    /// is asked to stop (it ends through its worker), and the study concludes as cancelled
    /// once no try runs. A concluded study is left unchanged.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn cancel(&self, study: StudyId, actor: &str) -> Result<StudyCancel, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        // Every point job first, in job order, as the job queue locks them; then the study.
        statements::lock_point_jobs()
            .bind(&tx, &study)
            .all()
            .await
            .classify(target)?;
        let row = lock_study(&tx, target, study).await?;
        let mut outcome = StudyCancel::default();
        if row.state != StudyState::Open {
            tx.rollback().await.classify(target)?;
            outcome.already_concluded = true;
            return Ok(outcome);
        }
        let flagged = pse_operations_queries::queries::cancellation::request_cancel()
            .bind(&tx, &row.attempt_id)
            .await
            .classify(target)?;
        if flagged != 1 {
            return Err(OperationsError::NotFound {
                entity: "attempt",
                id: row.attempt_id.to_string(),
            });
        }
        let reason = format!("study cancelled by {actor}");
        for point in points(&tx, target, study).await? {
            match point.state {
                StudyPointState::Pending => {
                    let stored = statements::point()
                        .params(
                            &tx,
                            &statements::PointParams {
                                study_id: study,
                                point_index: index(point.point_index)?,
                            },
                        )
                        .one()
                        .await
                        .classify(target)?;
                    cancel_unstarted(
                        &tx,
                        target,
                        study,
                        &stored,
                        RuntimeTermination::Cancelled,
                        &reason,
                    )
                    .await?;
                    outcome.cancelled.push(point.point_index);
                }
                StudyPointState::Assigned => {
                    pse_operations_queries::queries::cancellation::request_cancel()
                        .bind(&tx, &point.attempt_id)
                        .await
                        .classify(target)?;
                    attempts::notify(
                        &tx,
                        target,
                        crate::cancellation::CANCEL_CHANNEL,
                        &point.attempt_id.to_string(),
                    )
                    .await?;
                    outcome.stopping.push(point.point_index);
                }
                StudyPointState::Completed
                | StudyPointState::Failed
                | StudyPointState::Cancelled => {}
            }
        }
        if outcome.stopping.is_empty() {
            conclude(&tx, target, &row).await?;
            outcome.concluded = true;
        }
        tx.commit().await.classify(target)?;
        Ok(outcome)
    }

    /// Cancel the study whose coordinating attempt is `attempt`, reported as a
    /// cancellation of that attempt.
    pub(crate) async fn cancel_attempt(
        &self,
        attempt: AttemptId,
        actor: &str,
    ) -> Result<CancelOutcome, OperationsError> {
        let study = self
            .of_attempt(attempt)
            .await?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "study of attempt",
                id: attempt.to_string(),
            })?;
        let cancelled = self.cancel(study.study_id, actor).await?;
        Ok(if cancelled.already_concluded {
            CancelOutcome::AlreadyFinished(self.store.attempts().get(attempt).await?.state)
        } else if cancelled.concluded {
            CancelOutcome::CancelledBeforeStart
        } else {
            CancelOutcome::Requested
        })
    }

    /// Record that the study's publication committed: concluded becomes published.
    /// Idempotent.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a study that has not concluded;
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn mark_published(&self, study: StudyId) -> Result<(), OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let row = lock_study(&tx, target, study).await?;
        match row.state {
            StudyState::Published => {}
            StudyState::Concluded => set_study(&tx, target, study, StudyState::Published).await?,
            StudyState::Open => {
                return Err(OperationsError::InvalidRequest {
                    reason: format!("study {study} has not concluded; it cannot be published"),
                });
            }
        }
        tx.commit().await.classify(target)
    }
}
