// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable adapter of the shared occurrence policy. Operations own transactional I/O;
//! the policy consumes scientific permissions supplied by operation owners. Queue claims
//! admit acquisition only, and native dispatch is fenced by the current revision and lease.
//! Outcomes retain every try, structured diagnostics, start provenance and effect facts.

use crate::study_policy;
use pse_ids::ContentHash;
use pse_model::study::*;
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
    /// Reusable binding content identity; equal bindings can be requested repeatedly.
    pub binding_hash: pse_ids::roles::BindingHash,
    /// Mechanically derived occurrence policy from the immutable study definition.
    pub policy: PointPolicy,
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

/// Claimed execution and occurrence revision to recheck under the durable locks.
/// Possession of this value grants no dispatch or publication permission.
#[derive(Clone, Copy, Debug)]
pub struct DispatchFence<'a> {
    /// Study whose immutable definition owns the occurrence.
    pub study: StudyId,
    /// Occurrence identity, independent of binding content.
    pub key: OccurrenceKey,
    /// Claimed point job.
    pub job: JobId,
    /// Current try owned by the worker.
    pub attempt: AttemptId,
    /// Worker whose live lease must still own the attempt.
    pub worker: &'a str,
    /// Revision observed during acquisition or after dispatch.
    pub expected_revision: u64,
}

/// Scientific observation and exact owner ticket persisted before native publication.
/// The ticket remains opaque to the durable adapter and its effect is initially unknown.
#[derive(Debug)]
pub struct PreEffectReceipt {
    /// Operation-owned aggregate scientific permission.
    pub scientific: ScientificFacts,
    /// Typed diagnostic retained with the scientific observation.
    pub diagnostic: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Exact native publication ticket, interpreted only by its owner.
    pub receipt: serde_json::Value,
}

/// One point with its job's state and current attempt.
#[derive(Clone, Debug)]
pub struct PointStatus {
    /// Occurrence identity, independent of position and binding content.
    pub point_index: u32,
    /// Reusable binding identity; duplicates are allowed.
    pub binding_hash: pse_ids::roles::BindingHash,
    /// Current point lifecycle.
    pub state: StudyPointState,
    /// Revision fences acquisition and dispatch.
    pub revision: u64,
    /// None for a preserved historical row requiring explicit readmission.
    pub policy: Option<PointPolicy>,
    /// Historical attribution retained without granting current execution permission.
    pub legacy: Option<LegacyUnavailable>,
    /// None for a historical row with unavailable aggregate facts.
    pub outcome: Option<PointOutcome>,
    /// Exact pre-effect native publication ticket, interpreted only by its owner.
    pub receipt: Option<serde_json::Value>,
    /// Original try owning the currently attached member inventory.
    pub member_attempt: Option<AttemptId>,
    /// Job identity and lifecycle.
    pub job_id: JobId,
    /// Job lifecycle.
    pub job_state: JobState,
    /// Current try identity.
    pub attempt_id: AttemptId,
    /// Current try lifecycle.
    pub attempt_state: AttemptState,
    /// Current try number.
    pub tries: u32,
    /// Retained terminal detail, never a rendered message as authority.
    pub termination_detail: Option<serde_json::Value>,
    /// Human-readable job audit context.
    pub last_error: Option<String>,
}

/// Explicit historical marker; it never supplies current scientific permissions.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LegacyUnavailable {
    /// Marker codec version.
    pub version: pse_model::document::Version<1>,
    /// Exact closed marker spelling.
    pub kind: LegacyKind,
    /// Former positional predecessor retained as historical attribution only.
    #[serde(default)]
    pub predecessor: Option<u32>,
    /// Former content identity, retained without recomputation.
    #[serde(default)]
    pub binding_hash: Option<ContentHash>,
}
/// Registry-owned closed historical marker vocabulary.
pub use pse_model::generated::enums::StudyLegacyKind as LegacyKind;
fn legacy_marker(
    document: &serde_json::Value,
    column: &'static str,
) -> Result<Option<LegacyUnavailable>, OperationsError> {
    if document.get("kind").and_then(serde_json::Value::as_str) != Some("legacy_unavailable") {
        return Ok(None);
    }
    serde::Deserialize::deserialize(document)
        .map(Some)
        .map_err(|error| OperationsError::CorruptValue {
            column,
            detail: error.to_string(),
        })
}
fn decode_current<T: serde::de::DeserializeOwned>(
    document: &serde_json::Value,
    column: &'static str,
) -> Result<Option<T>, OperationsError> {
    if legacy_marker(document, column)?.is_some() {
        return Ok(None);
    }
    T::deserialize(document)
        .map(Some)
        .map_err(|error| OperationsError::CorruptValue {
            column,
            detail: error.to_string(),
        })
}

/// Operational envelope retaining the pre-effect receipt alongside scientific facts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredPointOutcome {
    version: pse_model::document::Version<1>,
    outcome: PointOutcome,
    receipt: Option<serde_json::Value>,
    member_attempt: Option<AttemptId>,
}
fn stored_outcome(
    outcome: &PointOutcome,
    receipt: Option<serde_json::Value>,
    member_attempt: Option<AttemptId>,
) -> StoredPointOutcome {
    StoredPointOutcome {
        version: Default::default(),
        outcome: outcome.clone(),
        receipt,
        member_attempt,
    }
}

fn encode(value: &impl serde::Serialize) -> Result<serde_json::Value, OperationsError> {
    serde_json::to_value(value).map_err(|error| OperationsError::InvalidRequest {
        reason: format!("study fact encoding: {error}"),
    })
}

/// A study as the store holds it.
#[derive(Clone, Debug)]
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
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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
    point: &PointStatus,
    outcome: &PointOutcome,
) -> Result<(), OperationsError> {
    let changed = statements::set_point_outcome()
        .params(
            tx,
            &statements::SetPointOutcomeParams {
                state: outcome.lifecycle,
                outcome: &encode(&stored_outcome(
                    outcome,
                    point.receipt.clone(),
                    point.member_attempt,
                ))?,
                study_id: study,
                point_index: index(point.point_index)?,
                expected_revision: i64::try_from(point.revision).map_err(|_| {
                    OperationsError::InvalidRequest {
                        reason: "study revision exceeds storage".into(),
                    }
                })?,
            },
        )
        .await
        .classify(target)?;
    if changed != 1 {
        return Err(OperationsError::InvalidRequest {
            reason: "study point revision changed while applying policy".into(),
        });
    }
    Ok(())
}

// The ticket belongs to the native publication owner. Scientific transitions update
// only facts and member attribution, preserving that ticket under the same row fence.
async fn set_point_facts(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: &PointStatus,
    outcome: &PointOutcome,
) -> Result<(), OperationsError> {
    let changed = statements::set_point_facts()
        .params(
            tx,
            &statements::SetPointFactsParams {
                state: outcome.lifecycle,
                outcome: &encode(outcome)?,
                member_attempt: &encode(&point.member_attempt)?,
                study_id: study,
                point_index: index(point.point_index)?,
                expected_revision: i64::try_from(point.revision).map_err(|_| {
                    OperationsError::InvalidRequest {
                        reason: "study revision exceeds storage".into(),
                    }
                })?,
            },
        )
        .await
        .classify(target)?;
    if changed != 1 {
        return Err(OperationsError::InvalidRequest {
            reason: "study point revision changed while applying policy".into(),
        });
    }
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

/// A point status from a projected status row (the statements project the same columns).
macro_rules! point_status {
    ($row:expr) => {{
        let row = $row;
        let stored: Option<StoredPointOutcome> =
            decode_current(&row.outcome, "study_points.outcome")?;
        let (outcome, receipt, member_attempt) = stored.map_or((None, None, None), |stored| {
            (Some(stored.outcome), stored.receipt, stored.member_attempt)
        });
        Ok::<_, OperationsError>(PointStatus {
            point_index: stored_index(row.point_index)?,
            binding_hash: ContentHash::try_from_slice(&row.binding_hash)
                .map(pse_ids::roles::BindingHash::from)
                .map_err(|error| OperationsError::CorruptValue {
                    column: "study_points.binding_hash",
                    detail: error.to_string(),
                })?,
            revision: u64::try_from(row.revision).map_err(|error| {
                OperationsError::CorruptValue {
                    column: "study_points.revision",
                    detail: error.to_string(),
                }
            })?,
            legacy: legacy_marker(&row.policy, "study_points.policy")?,
            policy: decode_current(&row.policy, "study_points.policy")?,
            outcome,
            receipt,
            member_attempt,
            tries: u32::try_from(row.tries).map_err(|error| OperationsError::CorruptValue {
                column: "jobs.tries",
                detail: error.to_string(),
            })?,
            termination_detail: row.termination_detail,
            state: row.state,
            job_id: JobId::from_id(row.job_id),
            job_state: row.job_state,
            attempt_id: AttemptId::from_id(row.attempt_id),
            attempt_state: row.attempt_state,
            last_error: row.last_error,
        })
    }};
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
        .map(|row| point_status!(row))
        .collect()
}

// The returned statuses carry facts only. Updates use set_point_facts unless shared
// policy explicitly admits a new dispatch and retires its predecessor ticket. Omission
// here is never evidence that no ticket exists in the native row.
async fn policy_points(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
) -> Result<Vec<PointStatus>, OperationsError> {
    statements::point_policy_status()
        .bind(tx, &study)
        .all()
        .await
        .classify(target)?
        .into_iter()
        .map(|row| point_status!(row))
        .collect()
}

async fn one_point(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: u32,
) -> Result<PointStatus, OperationsError> {
    let row = statements::one_point_status()
        .params(
            tx,
            &statements::OnePointStatusParams {
                study_id: study,
                point_index: index(point)?,
            },
        )
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "study point",
            id: format!("{study}/{point}"),
        })?;
    point_status!(row)
}

fn policy_snapshot(
    points: &[PointStatus],
    seed: Option<(OccurrenceKey, SeedFact)>,
) -> Result<(OccurrenceGraph, Vec<PointFacts>), OperationsError> {
    let unavailable = || {
        OperationsError::InvalidRequest { reason: "historical study policy or scientific facts are unavailable; explicit readmission is required".into() }
    };
    let graph = OccurrenceGraph {
        points: points
            .iter()
            .map(|point| point.policy.clone().ok_or_else(unavailable))
            .collect::<Result<_, _>>()?,
    };
    let facts = points
        .iter()
        .map(|point| {
            let outcome = point.outcome.as_ref().ok_or_else(unavailable)?;
            let policy = point.policy.as_ref().ok_or_else(unavailable)?;
            if policy.key.0 != point.point_index || outcome.key != policy.key {
                return Err(OperationsError::CorruptValue {
                    column: "study_points.policy",
                    detail: "occurrence keys differ from the stored row".into(),
                });
            }
            let selected = seed
                .as_ref()
                .filter(|(key, _)| *key == policy.key)
                .map(|(_, fact)| fact.clone())
                .or(match &policy.start {
                    StartPolicy::Fresh => None,
                    StartPolicy::Continuation(edge) => Some(SeedFact {
                        role: edge.role,
                        availability: SeedAvailability::Unresolved,
                    }),
                    StartPolicy::Explicit { role, .. } => Some(SeedFact {
                        role: *role,
                        availability: SeedAvailability::Unresolved,
                    }),
                });
            let retry_failure = point
                .termination_detail
                .as_ref()
                .and_then(|detail| detail.get("retry_failure"))
                .filter(|value| !value.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|error| OperationsError::CorruptValue {
                    column: "attempts.termination_detail",
                    detail: error.to_string(),
                })?;
            Ok(PointFacts {
                key: policy.key,
                revision: point.revision,
                lifecycle: outcome.lifecycle,
                native_started: outcome.start.is_some()
                    && outcome.lifecycle == StudyPointState::Assigned,
                scientific: outcome.scientific.clone(),
                attempt_count: point.tries,
                retry_failure,
                effect: outcome.effect,
                seed: selected,
            })
        })
        .collect::<Result<_, OperationsError>>()?;
    Ok((graph, facts))
}

async fn apply_unstarted(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: &PointStatus,
    action: &ActionKind,
) -> Result<(), OperationsError> {
    let mut outcome = point
        .outcome
        .clone()
        .ok_or_else(|| OperationsError::InvalidRequest {
            reason: "historical occurrence requires readmission".into(),
        })?;
    let (state, reason) = match action {
        ActionKind::Cancel => (StudyPointState::Cancelled, "study cancelled".to_owned()),
        ActionKind::Refuse(refusal) => {
            outcome.diagnostic = Some(refusal.boundary_diagnostic());
            (
                StudyPointState::Failed,
                format!("study dependency policy refused: {refusal:?}"),
            )
        }
        _ => return Ok(()),
    };
    let job = jobs::lock_job(tx, target, point.job_id).await?;
    if !matches!(job.state, JobState::Waiting | JobState::Queued) {
        return Ok(());
    }
    attempts::apply(
        tx,
        target,
        job.attempt_id,
        AttemptState::Cancelled,
        &TransitionNote::by(ACTOR)
            .because(&reason)
            .terminated(Termination {
                code: TerminationCode::Runtime(RuntimeTermination::Unattempted),
                detail: outcome.diagnostic.as_ref().map(encode).transpose()?,
            }),
        None,
    )
    .await?;
    jobs::set_job_state(
        tx,
        target,
        point.job_id,
        if state == StudyPointState::Cancelled {
            JobState::Cancelled
        } else {
            JobState::Failed
        },
        Some(&reason),
    )
    .await?;
    outcome.lifecycle = state;
    set_point_facts(tx, target, study, point, &outcome).await
}

async fn apply_policy(
    tx: &Tx<'_>,
    target: &Target,
    study: &RuntimeOperationalStudiesRow,
) -> Result<StudyDecision, OperationsError> {
    loop {
        let snapshot = policy_points(tx, target, study.study_id).await?;
        let (graph, facts) = policy_snapshot(&snapshot, None)?;
        let cancel = attempts::fetch(tx, target, study.attempt_id)
            .await?
            .cancel_requested;
        let decision = study_policy::transition(&graph, &facts, cancel).map_err(|error| {
            OperationsError::InvalidRequest {
                reason: error.to_string(),
            }
        })?;
        let mut changed = false;
        for (point, action) in snapshot.iter().zip(&decision.actions) {
            if point.revision != action.expected_revision {
                return Err(OperationsError::InvalidRequest {
                    reason: "stale study policy action".into(),
                });
            }
            match &action.kind {
                ActionKind::Start(_) | ActionKind::Wait(WaitReason::SeedResolution { .. })
                    if point.job_state == JobState::Waiting =>
                {
                    jobs::release(
                        tx,
                        target,
                        point.job_id,
                        &TransitionNote::by(ACTOR)
                            .because("study policy admitted input acquisition"),
                    )
                    .await?;
                }
                ActionKind::Cancel | ActionKind::Refuse(_)
                    if matches!(point.job_state, JobState::Waiting | JobState::Queued) =>
                {
                    apply_unstarted(tx, target, study.study_id, point, &action.kind).await?;
                    changed = true;
                }
                _ => {}
            }
        }
        if !changed {
            return Ok(decision);
        }
    }
}

fn attached_member(
    row: &RuntimeOperationalStudyPointMembersRow,
) -> Result<MemberDescriptor, OperationsError> {
    Ok(MemberDescriptor {
        catalog_name: row.catalog_name.clone(),
        schema_name: row.schema_name.clone(),
        table_name: row.table_name.clone(),
        relation_id: row.relation_id,
        relation_version: row.relation_version,
        contract_fingerprint: row.contract_fingerprint,
        table_uri: row.table_uri.clone(),
        delta_version: row.delta_version,
        selection: crate::catalog::stored_selection(
            row.selection_kind,
            row.revision_column.as_ref(),
            row.revision_id,
        )
        .ok_or_else(|| OperationsError::CorruptValue {
            column: "study_point_members.selection_kind",
            detail: "inconsistent selection".into(),
        })?,
    })
}
fn point_terminal_diagnostic(
    detail: &serde_json::Value,
    attempt_id: AttemptId,
    lifecycle: AttemptState,
) -> Result<Option<PointAttemptOutcome>, OperationsError> {
    let Some(diagnostic) = detail
        .get("cause")
        .and_then(|cause| cause.get("diagnostic"))
    else {
        return Ok(None);
    };
    let diagnostic = serde_json::from_value(diagnostic.clone()).map_err(|error| {
        OperationsError::CorruptValue {
            column: "attempts.termination_detail",
            detail: error.to_string(),
        }
    })?;
    Ok(Some(PointAttemptOutcome {
        attempt_id: Some(attempt_id),
        lifecycle: Some(lifecycle),
        diagnostic: Some(diagnostic),
        scientific: ScientificFacts::default(),
        start: None,
        effect: EffectState::Absent,
    }))
}
fn retained_diagnostic(
    detail: &serde_json::Value,
) -> Result<Option<pse_model::diagnostic::BoundaryDiagnostic>, OperationsError> {
    let value = detail
        .get("point")
        .and_then(|point| point.get("diagnostic"))
        .filter(|value| !value.is_null())
        .or_else(|| {
            detail
                .get("cause")
                .and_then(|cause| cause.get("diagnostic"))
                .filter(|value| !value.is_null())
        });
    value
        .map(|value| {
            serde_json::from_value(value.clone()).map_err(|error| OperationsError::CorruptValue {
                column: "attempts.termination_detail",
                detail: error.to_string(),
            })
        })
        .transpose()
}
fn stale_attempt_outcome(
    prior: &RuntimeOperationalAttemptsRow,
    outcome: &PointOutcome,
) -> Result<PointAttemptOutcome, OperationsError> {
    use pse_model::diagnostic::{
        BoundaryClass, BoundaryDiagnostic, DiagnosticRule, DiagnosticStage, Observation,
    };
    let mut diagnostic = BoundaryDiagnostic::new(
        BoundaryClass::Infrastructure,
        DiagnosticStage::Workflow,
        [],
        DiagnosticRule::WorkflowOperations,
    );
    diagnostic.observations.insert(
        "attempt_id".into(),
        Observation::Text(prior.attempt_id.to_string()),
    );
    diagnostic.observations.insert(
        "attempt_state".into(),
        Observation::Text(prior.state.as_str().into()),
    );
    diagnostic.observations.insert(
        "event".into(),
        Observation::Text(
            if prior.state == AttemptState::Stale {
                "lease_expired"
            } else {
                "attempt_superseded"
            }
            .into(),
        ),
    );
    if let Some(worker) = &prior.worker {
        diagnostic
            .observations
            .insert("worker".into(), Observation::Text(worker.clone()));
    }
    if let Some(expiry) = prior.lease_expires_at {
        diagnostic
            .observations
            .insert("lease_expires_at".into(), Observation::Integer(expiry));
    }
    if let Some(detail) = &prior.termination_detail {
        let detail =
            serde_json::from_str(detail).map_err(|error| OperationsError::CorruptValue {
                column: "attempts.termination_detail",
                detail: error.to_string(),
            })?;
        if let Some(mut retained) = retained_diagnostic(&detail)? {
            retained.causes.push(diagnostic);
            diagnostic = retained;
        }
    }
    Ok(PointAttemptOutcome {
        attempt_id: Some(prior.attempt_id),
        lifecycle: Some(prior.state),
        diagnostic: Some(diagnostic),
        scientific: outcome.scientific.clone(),
        start: outcome.start.clone(),
        effect: if outcome.start.is_some() {
            EffectState::Unknown
        } else {
            EffectState::Absent
        },
    })
}
fn includes_diagnostic(
    existing: &pse_model::diagnostic::BoundaryDiagnostic,
    incoming: &pse_model::diagnostic::BoundaryDiagnostic,
) -> Result<bool, OperationsError> {
    if encode(existing)? == encode(incoming)? {
        return Ok(true);
    }
    for cause in &existing.causes {
        if includes_diagnostic(cause, incoming)? {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Enrich repeated terminal callbacks without replacing already retained scientific,
/// start or effect facts. The operational row supplies the current attempt lifecycle.
fn retain_attempt(
    outcome: &mut PointOutcome,
    incoming: PointAttemptOutcome,
) -> Result<bool, OperationsError> {
    if let Some(existing) = outcome
        .attempts
        .iter_mut()
        .find(|existing| existing.attempt_id == incoming.attempt_id)
    {
        let previous = existing.diagnostic.as_ref().map(encode).transpose()?;
        existing.lifecycle = incoming.lifecycle.or(existing.lifecycle);
        if let Some(diagnostic) = incoming.diagnostic {
            match &mut existing.diagnostic {
                Some(retained) if !includes_diagnostic(retained, &diagnostic)? => {
                    retained.causes.push(diagnostic)
                }
                None => existing.diagnostic = Some(diagnostic),
                _ => {}
            }
        }
        if outcome.diagnostic.as_ref().map(encode).transpose()? == previous
            || outcome.diagnostic.is_none()
        {
            outcome.diagnostic = existing.diagnostic.clone();
        }
        return Ok(false);
    }
    outcome.attempts.push(incoming);
    Ok(true)
}
fn exact_member_receipt(
    existing: &MemberDescriptor,
    incoming: &MemberDescriptor,
) -> Result<(), OperationsError> {
    if existing == incoming {
        return Ok(());
    }
    Err(OperationsError::InvalidRequest {
        reason: "exact native member receipt conflicts with attached descriptor/version".into(),
    })
}
fn member_attachment(
    previous: Option<AttemptId>,
    incoming: AttemptId,
    history: &[PointAttemptOutcome],
) -> Result<bool, OperationsError> {
    if previous.is_none() || previous == Some(incoming) {
        return Ok(false);
    }
    if !history.iter().any(|attempt| {
        attempt.attempt_id == previous
            && matches!(
                attempt.effect,
                EffectState::Absent | EffectState::Idempotent
            )
    }) {
        return Err(OperationsError::InvalidRequest {
            reason: "member inventory supersession requires known-safe original effect".into(),
        });
    }
    Ok(true)
}
async fn insert_members(
    tx: &Tx<'_>,
    target: &Target,
    study: StudyId,
    point: &mut PointStatus,
    owner: AttemptId,
    outcome: &PointOutcome,
    members: &[MemberDescriptor],
) -> Result<(), OperationsError> {
    if members.is_empty() {
        return Ok(());
    }
    if member_attachment(point.member_attempt, owner, &outcome.attempts)? {
        statements::remove_point_members()
            .params(
                tx,
                &statements::RemovePointMembersParams {
                    study_id: study,
                    point_index: index(point.point_index)?,
                },
            )
            .await
            .classify(target)?;
    }
    let mut attached = std::collections::BTreeMap::new();
    for row in statements::available_members()
        .bind(tx, &study)
        .all()
        .await
        .classify(target)?
    {
        if row.point_index == index(point.point_index)? {
            let member = attached_member(&row)?;
            attached.insert(
                (
                    member.catalog_name.clone(),
                    member.schema_name.clone(),
                    member.table_name.clone(),
                ),
                member,
            );
        }
    }
    for member in members {
        let name = (
            member.catalog_name.clone(),
            member.schema_name.clone(),
            member.table_name.clone(),
        );
        if let Some(existing) = attached.get(&name) {
            exact_member_receipt(existing, member)?;
            continue;
        }
        let (selection_kind, revision_column, revision_id) =
            crate::catalog::flattened_selection(member)?;
        statements::insert_point_member()
            .params(
                tx,
                &statements::InsertPointMemberParams {
                    study_id: study,
                    point_index: index(point.point_index)?,
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
        attached.insert(name, member.clone());
    }
    point.member_attempt = Some(owner);
    Ok(())
}

/// Conclude a study whose points are all terminal: end its coordinating attempt and
/// release its finalization job.
async fn conclude(
    tx: &Tx<'_>,
    target: &Target,
    study: &RuntimeOperationalStudiesRow,
) -> Result<(), OperationsError> {
    let decision = apply_policy(tx, target, study).await?;
    if decision
        .actions
        .iter()
        .any(|action| matches!(action.kind, ActionKind::Reconcile))
        || decision.conclusion.lifecycle == StudyLifecycle::Active
        || policy_points(tx, target, study.study_id)
            .await?
            .iter()
            .any(|point| !is_terminal(point.state))
    {
        return Ok(());
    }
    let state = if decision.conclusion.lifecycle == StudyLifecycle::Cancelled {
        AttemptState::Cancelled
    } else {
        match decision.conclusion.availability {
            Availability::Complete => AttemptState::Completed,
            Availability::Partial => AttemptState::Partial,
            Availability::None => AttemptState::Failed,
        }
    };
    let note = TransitionNote::by(ACTOR).because(format!(
        "study scientific availability: {:?}",
        decision.conclusion.availability
    ));
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
    let point = one_point(tx, target, study.study_id, stored_index(found.point_index)?).await?;
    if point.job_id != job {
        return Err(OperationsError::NotFound {
            entity: "study point job",
            id: job.to_string(),
        });
    }
    // Legacy unavailable rows remain readable but cannot reenter the current executor.
    let Some(mut outcome) = point.outcome.clone() else {
        return Ok(());
    };
    let current = attempts::fetch(tx, target, point.attempt_id).await?;
    if outcome.lifecycle == StudyPointState::Assigned
        && point.attempt_state != AttemptState::Running
    {
        let prior = if matches!(
            current.state,
            AttemptState::Stale | AttemptState::Superseded
        ) {
            Some(current.clone())
        } else if let Some(parent) = current.parent_attempt {
            Some(attempts::fetch(tx, target, parent).await?)
        } else {
            None
        };
        if let Some(prior) = prior
            .filter(|prior| matches!(prior.state, AttemptState::Stale | AttemptState::Superseded))
        {
            let attempt = stale_attempt_outcome(&prior, &outcome)?;
            outcome.effect = attempt.effect;
            outcome.diagnostic = attempt.diagnostic.clone();
            retain_attempt(&mut outcome, attempt)?;
        }
    }
    outcome.lifecycle = point_state(point.job_state);
    if outcome.effect == EffectState::Unknown && point.job_state == JobState::Queued {
        jobs::set_job_state(
            tx,
            target,
            point.job_id,
            JobState::Waiting,
            Some("publication effect requires reconciliation"),
        )
        .await?;
    }
    if let Some(detail) = &point.termination_detail {
        let terminal = if current.termination_detail.is_some() {
            current.clone()
        } else if let Some(parent) = current.parent_attempt {
            attempts::fetch(tx, target, parent).await?
        } else {
            current.clone()
        };
        let mut supplied = detail
            .get("point")
            .filter(|value| !value.is_null())
            .map(|value| {
                serde_json::from_value::<PointAttemptOutcome>(value.clone()).map_err(|error| {
                    OperationsError::CorruptValue {
                        column: "attempts.termination_detail",
                        detail: error.to_string(),
                    }
                })
            })
            .transpose()?;
        // Decode/version failure can precede runtime PointContext. Preserve its typed
        // terminal cause using the actual row, without granting scientific permission.
        if supplied.is_none()
            && detail
                .get("cause")
                .and_then(|cause| cause.get("diagnostic"))
                .is_some()
        {
            supplied = point_terminal_diagnostic(detail, terminal.attempt_id, terminal.state)?;
        }
        if let Some(mut attempt) = supplied {
            if attempt.attempt_id != Some(terminal.attempt_id) {
                return Err(OperationsError::CorruptValue {
                    column: "attempts.termination_detail",
                    detail: "terminal point envelope names a different attempt".into(),
                });
            }
            attempt.lifecycle = Some(terminal.state);
            if retain_attempt(&mut outcome, attempt.clone())? {
                outcome.scientific = attempt.scientific.clone();
                outcome.diagnostic = attempt.diagnostic.clone();
                outcome.start = attempt.start.clone();
                outcome.effect = attempt.effect;
            }
        }
    }
    if outcome.lifecycle == StudyPointState::Pending {
        outcome.start = None;
    }
    let owner = outcome
        .attempts
        .last()
        .and_then(|attempt| attempt.attempt_id)
        .unwrap_or(point.attempt_id);
    let mut attached = point.clone();
    insert_members(
        tx,
        target,
        study.study_id,
        &mut attached,
        owner,
        &outcome,
        members,
    )
    .await?;
    set_point_facts(tx, target, study.study_id, &attached, &outcome).await?;
    if study.state == StudyState::Open {
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
            return invalid(
                "a study's finalization attempt is of kind study_finalization".to_owned(),
            );
        }
        if study.intent.attempt_id != study.attempt.attempt_id {
            return invalid(
                "a study's publication intent names the study's own attempt".to_owned(),
            );
        }
        for (position, point) in study.points.iter().enumerate() {
            if crate::lifecycle::coordinates(point.job.attempt.kind)
                || point.job.attempt.kind == AttemptKind::StudyFinalization
            {
                return invalid(format!("point {position} runs a study attempt"));
            }
        }
        let graph = OccurrenceGraph {
            points: study
                .points
                .iter()
                .map(|point| point.policy.clone())
                .collect(),
        };
        study_policy::admit(&graph).map_err(|error| OperationsError::InvalidRequest {
            reason: error.to_string(),
        })?;
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        attempts::insert(&tx, target, &study.attempt, Some(ACTOR)).await?;
        let open = TransitionNote::by(ACTOR).because("points are queued");
        attempts::apply(
            &tx,
            target,
            study.attempt.attempt_id,
            AttemptState::Queued,
            &open,
            None,
        )
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
        if !jobs::insert_job(
            &tx,
            target,
            finalization_job,
            &study.finalization,
            JobState::Waiting,
        )
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
        for point in &study.points {
            let job_id: JobId = crate::mint_id();
            attempts::insert(&tx, target, &point.job.attempt, Some(ACTOR)).await?;
            let state = if !point.policy.dependencies.is_empty()
                || matches!(point.policy.start, StartPolicy::Continuation(_))
            {
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
            let mut point_job = point.job.clone();
            point_job.retry.max_tries = point.policy.attempt_limit;
            if !jobs::insert_job(&tx, target, job_id, &point_job, state).await? {
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
                        point_index: index(point.policy.key.0)?,
                        binding_hash: point.binding_hash.as_id(),
                        policy: &encode(&point.policy)?,
                        outcome: &encode(&stored_outcome(
                            &PointOutcome {
                                key: point.policy.key,
                                lifecycle: StudyPointState::Pending,
                                scientific: ScientificFacts::default(),
                                diagnostic: None,
                                start: None,
                                effect: EffectState::Absent,
                                attempts: vec![],
                            },
                            None,
                            None,
                        ))?,
                        job_id,
                        state: StudyPointState::Pending,
                    },
                )
                .await
                .classify(target)?;
            point_jobs.push(job_id);
        }
        let stored = lock_study(&tx, target, study.study_id).await?;
        apply_policy(&tx, target, &stored).await?;
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
        self.read_record(study, true).await
    }

    /// Scientific facts and operational fences for the publication summary, without
    /// opaque point tickets. Ticket absence in this projection grants no effect knowledge.
    ///
    /// # Errors
    /// [`OperationsError::NotFound`]; classified driver failures or corrupt facts.
    pub async fn summary_record(&self, study: StudyId) -> Result<StudyRecord, OperationsError> {
        self.read_record(study, false).await
    }

    async fn read_record(
        &self,
        study: StudyId,
        include_receipts: bool,
    ) -> Result<StudyRecord, OperationsError> {
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
        let points = if include_receipts {
            points(&tx, target, study).await?
        } else {
            policy_points(&tx, target, study).await?
        };
        tx.commit().await.classify(target)?;
        Ok(StudyRecord {
            study: row,
            attempt,
            finalization,
            points,
        })
    }

    /// The study row alone, without its points.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn row(
        &self,
        study: StudyId,
    ) -> Result<RuntimeOperationalStudiesRow, OperationsError> {
        let client = self.store.client().await?;
        statements::study()
            .bind(&client, &study)
            .opt()
            .await
            .classify(self.target())?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "study",
                id: study.to_string(),
            })
    }

    /// One point of a study with its job's state and current attempt.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn point(&self, study: StudyId, point: u32) -> Result<PointStatus, OperationsError> {
        let client = self.store.client().await?;
        let row = statements::one_point_status()
            .params(
                &client,
                &statements::OnePointStatusParams {
                    study_id: study,
                    point_index: index(point)?,
                },
            )
            .opt()
            .await
            .classify(self.target())?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "study point",
                id: format!("{study}/{point}"),
            })?;
        point_status!(row)
    }

    /// Stopped points whose native writes still need receipt-owner reconciliation.
    /// Known effects and running dispatches do not load their publication tickets.
    ///
    /// # Errors
    /// Classified driver failures or corrupt selected point facts.
    pub async fn unresolved_points(
        &self,
        study: StudyId,
    ) -> Result<Vec<PointStatus>, OperationsError> {
        let client = self.store.client().await?;
        statements::unresolved_point_status()
            .bind(&client, &study)
            .all()
            .await
            .classify(self.target())?
            .into_iter()
            .map(|row| point_status!(row))
            .collect()
    }

    /// Recompute shared policy under study and job locks, fence the current lease and
    /// record the chosen start before native dispatch. A claim admits acquisition only.
    pub async fn admit_dispatch(
        &self,
        fence: DispatchFence<'_>,
        seed: Option<SeedFact>,
    ) -> Result<ActionKind, OperationsError> {
        let DispatchFence {
            study,
            key,
            job,
            attempt,
            worker,
            expected_revision,
        } = fence;
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let locked = jobs::lock_job(&tx, target, job).await?;
        let owner = statements::dispatch_owner()
            .bind(&tx, &attempt)
            .opt()
            .await
            .classify(target)?
            .flatten();
        if locked.state != JobState::Running
            || locked.attempt_id != attempt
            || owner.as_deref() != Some(worker)
        {
            return Err(OperationsError::LeaseLost {
                attempt,
                worker: worker.into(),
            });
        }
        let row = lock_study(&tx, target, study).await?;
        let snapshot = policy_points(&tx, target, study).await?;
        let point = snapshot
            .iter()
            .find(|point| point.point_index == key.0 && point.job_id == job)
            .ok_or_else(|| OperationsError::NotFound {
                entity: "study point",
                id: format!("{study}/{}", key.0),
            })?;
        if point.revision != expected_revision {
            return Err(OperationsError::InvalidRequest {
                reason: "study acquisition revision changed".into(),
            });
        }
        let (graph, facts) = policy_snapshot(&snapshot, seed.map(|fact| (key, fact)))?;
        let cancelled = attempts::fetch(&tx, target, row.attempt_id)
            .await?
            .cancel_requested
            || attempts::fetch(&tx, target, attempt)
                .await?
                .cancel_requested;
        let decision = study_policy::transition(&graph, &facts, cancelled).map_err(|error| {
            OperationsError::InvalidRequest {
                reason: error.to_string(),
            }
        })?;
        let action = decision
            .actions
            .into_iter()
            .find(|action| action.occurrence == key)
            .ok_or_else(|| OperationsError::InvalidRequest {
                reason: "missing policy action".into(),
            })?;
        if let ActionKind::Start(start) = &action.kind {
            let mut outcome =
                point
                    .outcome
                    .clone()
                    .ok_or_else(|| OperationsError::InvalidRequest {
                        reason: "historical occurrence requires readmission".into(),
                    })?;
            outcome.start = Some(start.clone());
            let mut dispatched = point.clone();
            // An admitted new dispatch retires the old ticket before its replacement
            // is fenced and recorded. Other scientific transitions preserve it.
            dispatched.receipt = None;
            set_point(&tx, target, study, &dispatched, &outcome).await?;
        }
        tx.commit().await.classify(target)?;
        Ok(action.kind)
    }

    /// Persist the exact receipt before the native write begins, fenced by dispatch and lease.
    pub async fn record_receipt(
        &self,
        fence: DispatchFence<'_>,
        observation: PreEffectReceipt,
    ) -> Result<(), OperationsError> {
        let DispatchFence {
            study,
            key,
            job,
            attempt,
            worker,
            expected_revision,
        } = fence;
        let PreEffectReceipt {
            scientific,
            diagnostic,
            receipt,
        } = observation;
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let locked = jobs::lock_job(&tx, target, job).await?;
        let owner = statements::dispatch_owner()
            .bind(&tx, &attempt)
            .opt()
            .await
            .classify(target)?
            .flatten();
        if locked.state != JobState::Running
            || locked.attempt_id != attempt
            || owner.as_deref() != Some(worker)
        {
            return Err(OperationsError::LeaseLost {
                attempt,
                worker: worker.into(),
            });
        }
        lock_study(&tx, target, study).await?;
        let point = one_point(&tx, target, study, key.0).await?;
        if point.job_id != job {
            return Err(OperationsError::InvalidRequest {
                reason: "receipt occurrence differs from dispatch".into(),
            });
        }
        let mut outcome = point
            .outcome
            .clone()
            .ok_or_else(|| OperationsError::InvalidRequest {
                reason: "historical occurrence cannot write".into(),
            })?;
        if point.revision != expected_revision || outcome.start.is_none() || point.receipt.is_some()
        {
            return Err(OperationsError::InvalidRequest {
                reason: "receipt dispatch revision changed or ticket already exists".into(),
            });
        }
        outcome.scientific = scientific;
        outcome.diagnostic = diagnostic;
        outcome.effect = EffectState::Unknown;
        let mut updated = point.clone();
        updated.receipt = Some(receipt);
        set_point(&tx, target, study, &updated, &outcome).await?;
        tx.commit().await.classify(target)
    }

    /// Record receipt-owner reconciliation knowledge under a revision fence. Unknown
    /// effects never release work; a present non-idempotent effect cannot be retried.
    pub async fn reconcile_effect(
        &self,
        study: StudyId,
        key: OccurrenceKey,
        expected_revision: u64,
        effect: EffectState,
    ) -> Result<(), OperationsError> {
        self.reconcile_receipt(study, key, expected_revision, effect, None, &[])
            .await
    }

    /// Apply exact native member receipts without promoting scientific availability.
    pub async fn reconcile_receipt(
        &self,
        study: StudyId,
        key: OccurrenceKey,
        expected_revision: u64,
        effect: EffectState,
        receipt_attempt: Option<AttemptId>,
        members: &[MemberDescriptor],
    ) -> Result<(), OperationsError> {
        if effect == EffectState::Unknown && members.is_empty() {
            return Err(OperationsError::InvalidRequest {
                reason: "reconciliation supplied no new receipt knowledge".into(),
            });
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::lock_point_jobs()
            .bind(&tx, &study)
            .all()
            .await
            .classify(target)?;
        let row = lock_study(&tx, target, study).await?;
        let point = one_point(&tx, target, study, key.0).await?;
        let mut outcome = point
            .outcome
            .clone()
            .ok_or_else(|| OperationsError::InvalidRequest {
                reason: "historical policy requires readmission".into(),
            })?;
        if point.revision != expected_revision || outcome.effect != EffectState::Unknown {
            return Err(OperationsError::InvalidRequest {
                reason: "reconciliation facts changed".into(),
            });
        }
        if effect == EffectState::Absent
            && (point.receipt.is_some() || point.job_state == JobState::Running)
        {
            return Err(OperationsError::InvalidRequest {
                reason: "unwritten effect requires a closed dispatch fence".into(),
            });
        }
        outcome.effect = effect;
        if effect != EffectState::Unknown
            && let Some(attempt) = outcome
                .attempts
                .iter_mut()
                .rev()
                .find(|a| receipt_attempt.is_none_or(|id| a.attempt_id == Some(id)))
        {
            attempt.effect = effect;
        }
        if effect == EffectState::Present && point.job_state == JobState::Waiting {
            let note = TransitionNote::by(ACTOR).because("present effect cannot be retried");
            attempts::apply(
                &tx,
                target,
                point.attempt_id,
                AttemptState::Cancelled,
                &note,
                None,
            )
            .await?;
            jobs::set_job_state(
                &tx,
                target,
                point.job_id,
                JobState::Failed,
                note.reason.as_deref(),
            )
            .await?;
            outcome.lifecycle = StudyPointState::Failed;
        }
        let mut attached = point.clone();
        let owner = receipt_attempt.unwrap_or(point.attempt_id);
        insert_members(&tx, target, study, &mut attached, owner, &outcome, members).await?;
        set_point_facts(&tx, target, study, &attached, &outcome).await?;
        let revised = policy_points(&tx, target, study).await?;
        let (graph, facts) = policy_snapshot(&revised, None)?;
        let cancelled = attempts::fetch(&tx, target, row.attempt_id)
            .await?
            .cancel_requested;
        let decision = study_policy::transition(&graph, &facts, cancelled).map_err(|e| {
            OperationsError::InvalidRequest {
                reason: e.to_string(),
            }
        })?;
        if matches!(effect, EffectState::Absent | EffectState::Idempotent)
            && point.job_state == JobState::Failed
            && decision
                .actions
                .iter()
                .any(|a| a.occurrence == key && matches!(a.kind, ActionKind::Start(_)))
        {
            let job = jobs::lock_job(&tx, target, point.job_id).await?;
            jobs::requeue(
                &tx,
                target,
                &job,
                crate::mint_id(),
                "receipt established safe transient retry",
            )
            .await?;
            job_changed(&tx, target, point.job_id, &[]).await?;
        }
        conclude(&tx, target, &row).await?;
        tx.commit().await.classify(target)
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

    /// All recorded result members, independent of terminal lifecycle, each with its point, in point and
    /// name order: what the study's publication commits besides its summary.
    ///
    /// # Errors
    ///
    /// [`OperationsError::CorruptValue`] for an inconsistent stored selection; classified
    /// driver failures.
    pub async fn available_members(
        &self,
        study: StudyId,
    ) -> Result<Vec<(u32, MemberDescriptor)>, OperationsError> {
        let client = self.store.client().await?;
        statements::available_members()
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
    pub async fn cancel(
        &self,
        study: StudyId,
        actor: &str,
    ) -> Result<StudyCancel, OperationsError> {
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
        for point in policy_points(&tx, target, study).await? {
            if point.state == StudyPointState::Assigned {
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
            } else if point.state == StudyPointState::Pending {
                outcome.cancelled.push(point.point_index);
            }
        }
        let _ = actor;
        conclude(&tx, target, &row).await?;
        outcome.concluded = lock_study(&tx, target, study).await?.state == StudyState::Concluded;
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

#[cfg(test)]
mod study_codec_unit {
    use super::*;
    #[test]
    fn pre_context_failure_unit_retains_typed_terminal_diagnostic_without_scientific_promotion() {
        let id = crate::mint_id();
        let mut diagnostic = Refusal::SeedInternal {
            predecessor: Some(OccurrenceKey(2)),
            role: SeedRole::PrimalSolution,
        }
        .boundary_diagnostic();
        diagnostic.causes.push(
            Refusal::SeedUnavailable {
                predecessor: Some(OccurrenceKey(2)),
                role: SeedRole::PrimalSolution,
                reason: SeedUnavailable::Incompatible,
            }
            .boundary_diagnostic(),
        );
        let detail = serde_json::json!({"version":2,"cause":{"kind":"error","diagnostic":diagnostic},"point":null});
        let projected = point_terminal_diagnostic(&detail, id, AttemptState::Failed)
            .unwrap()
            .unwrap();
        assert_eq!(projected.attempt_id, Some(id));
        assert_eq!(projected.lifecycle, Some(AttemptState::Failed));
        let retained = projected.diagnostic.unwrap();
        assert_eq!(retained.code, diagnostic.code);
        assert_eq!(retained.rule, diagnostic.rule);
        assert_eq!(retained.causes[0].code, diagnostic.causes[0].code);
        assert_eq!(
            encode(&retained.observations).unwrap(),
            encode(&diagnostic.observations).unwrap()
        );
        assert_eq!(encode(&retained).unwrap(), encode(&diagnostic).unwrap());
        assert_eq!(projected.scientific, ScientificFacts::default());
        assert!(projected.start.is_none());
        assert_eq!(projected.effect, EffectState::Absent);
    }

    #[test]
    fn stale_history_unit_repeated_callbacks_enrich_one_attempt_without_promoting_facts() {
        use pse_model::generated::enums::AttemptKind;
        let id = crate::mint_id();
        let original = Refusal::SeedInternal {
            predecessor: None,
            role: SeedRole::PrimalSolution,
        }
        .boundary_diagnostic();
        let mut prior = RuntimeOperationalAttemptsRow {
            attempt_id: id,
            run_id: crate::mint_id(),
            kind: AttemptKind::Modeling,
            operational_job_identity: ContentHash::from_bytes([7; 32]),
            operational_job_frame: None,
            preparation_identity: None,
            state: AttemptState::Stale,
            state_version: 3,
            parent_attempt: None,
            worker: Some("original-worker".into()),
            lease_expires_at: Some(91),
            heartbeat_at: Some(81),
            cancel_requested: false,
            cancel_requested_at: None,
            termination_class: None,
            termination_native: None,
            termination_run_state: None,
            termination_trajectory: None,
            termination_runtime: None,
            termination_rule: None,
            termination_detail: None,
            created_at: 0,
            updated_at: 91,
            started_at: Some(1),
            finished_at: Some(91),
        };
        let mut outcome = PointOutcome {
            key: OccurrenceKey(2),
            lifecycle: StudyPointState::Assigned,
            scientific: ScientificFacts {
                usable: false,
                seed_permission: true,
                candidate_use: Some(pse_model::generated::enums::CandidateUse::SeedOnly),
            },
            diagnostic: None,
            start: Some(StartProvenance::Fresh),
            effect: EffectState::Absent,
            attempts: vec![],
        };
        let stale = stale_attempt_outcome(&prior, &outcome).unwrap();
        assert_eq!(stale.effect, EffectState::Unknown);
        assert_eq!(
            encode(&stale.diagnostic.as_ref().unwrap().observations["event"]).unwrap(),
            serde_json::json!({"kind":"text","value":"lease_expired"})
        );
        assert!(retain_attempt(&mut outcome, stale).unwrap());
        let mut enrichment = point_terminal_diagnostic(
            &serde_json::json!({"cause":{"diagnostic":original},"point":null}),
            id,
            AttemptState::Superseded,
        )
        .unwrap()
        .unwrap();
        assert!(!retain_attempt(&mut outcome, enrichment.clone()).unwrap());
        assert!(!retain_attempt(&mut outcome, enrichment.clone()).unwrap());
        assert_eq!(outcome.attempts.len(), 1);
        let retained = &outcome.attempts[0];
        assert_eq!(retained.lifecycle, Some(AttemptState::Superseded));
        assert_eq!(retained.start, Some(StartProvenance::Fresh));
        assert_eq!(retained.scientific, outcome.scientific);
        assert_eq!(retained.effect, EffectState::Unknown);
        assert_eq!(retained.diagnostic.as_ref().unwrap().causes.len(), 1);
        assert_eq!(
            retained.diagnostic.as_ref().unwrap().causes[0].code,
            original.code
        );
        // A row with an existing typed cause keeps that cause as its primary diagnosis.
        prior.state = AttemptState::Superseded;
        prior.termination_detail =
            Some(serde_json::json!({"cause":{"diagnostic":original},"point":null}).to_string());
        enrichment = stale_attempt_outcome(&prior, &outcome).unwrap();
        assert_eq!(enrichment.diagnostic.as_ref().unwrap().code, original.code);
        assert_eq!(
            encode(&enrichment.diagnostic.as_ref().unwrap().causes[0].observations["event"])
                .unwrap(),
            serde_json::json!({"kind":"text","value":"attempt_superseded"})
        );
    }

    #[test]
    fn member_receipt_unit_is_exact_and_retry_supersession_requires_safe_original_effect() {
        use pse_model::generated::structures::MemberDescriptorSelection;
        let member = MemberDescriptor {
            catalog_name: "point_2".into(),
            schema_name: "results".into(),
            table_name: "x".into(),
            relation_id: pse_ids::SemanticId::from_bytes([1; 16]),
            relation_version: 1,
            contract_fingerprint: ContentHash::from_bytes([1; 32]),
            table_uri: "memory:///point2/try1/x".into(),
            delta_version: 4,
            selection: MemberDescriptorSelection::from_full(),
        };
        assert!(exact_member_receipt(&member, &member).is_ok());
        let mut changed = member.clone();
        changed.delta_version += 1;
        assert!(exact_member_receipt(&member, &changed).is_err());
        let mut changed = member.clone();
        changed.table_uri = "memory:///point2/try2/x".into();
        assert!(exact_member_receipt(&member, &changed).is_err());
        let first = crate::mint_id();
        let second = crate::mint_id();
        let mut history = vec![PointAttemptOutcome {
            attempt_id: Some(first),
            lifecycle: Some(AttemptState::Failed),
            diagnostic: None,
            scientific: ScientificFacts::default(),
            start: Some(StartProvenance::Fresh),
            effect: EffectState::Unknown,
        }];
        assert!(!member_attachment(Some(first), first, &history).unwrap());
        assert!(member_attachment(Some(first), second, &history).is_err());
        history[0].effect = EffectState::Idempotent;
        assert!(member_attachment(Some(first), second, &history).unwrap());
    }

    #[test]
    fn pre_effect_receipt_envelope_unit_preserves_scientific_facts_and_closed_version() {
        let outcome = PointOutcome {
            key: OccurrenceKey(4),
            lifecycle: StudyPointState::Assigned,
            scientific: ScientificFacts {
                usable: false,
                candidate_use: Some(pse_model::generated::enums::CandidateUse::SeedOnly),
                seed_permission: true,
            },
            diagnostic: Some(
                Refusal::SeedInternal {
                    predecessor: None,
                    role: SeedRole::PrimalSolution,
                }
                .boundary_diagnostic(),
            ),
            start: Some(StartProvenance::Fresh),
            effect: EffectState::Unknown,
            attempts: vec![],
        };
        let receipt = serde_json::json!({"native":"owner request"});
        let json = encode(&stored_outcome(&outcome, Some(receipt.clone()), None)).unwrap();
        let decoded = decode_current::<StoredPointOutcome>(&json, "outcome")
            .unwrap()
            .unwrap();
        assert_eq!(decoded.receipt, Some(receipt));
        assert_eq!(decoded.outcome.scientific, outcome.scientific);
        assert_eq!(decoded.outcome.effect, EffectState::Unknown);
        assert_eq!(
            decoded.outcome.diagnostic.unwrap().code,
            outcome.diagnostic.unwrap().code
        );
        let mut broken = json;
        broken["version"] = serde_json::json!(2);
        assert!(decode_current::<StoredPointOutcome>(&broken, "outcome").is_err());
        assert!(
            decode_current::<StoredPointOutcome>(&serde_json::json!({"usable":true}), "outcome")
                .is_err()
        );
    }

    #[test]
    fn legacy_markers_retain_identity_and_never_grant_policy_or_usable_facts() {
        let marker = serde_json::json!({"version":1,"kind":"legacy_unavailable","predecessor":2,"binding_hash":ContentHash::from_bytes([4;32])});
        let decoded: LegacyUnavailable = serde_json::from_value(marker.clone()).unwrap();
        assert_eq!(decoded.predecessor, Some(2));
        assert_eq!(decoded.binding_hash, Some(ContentHash::from_bytes([4; 32])));
        let encoded = marker;
        assert!(
            decode_current::<PointPolicy>(&encoded, "policy")
                .unwrap()
                .is_none()
        );
        assert!(
            decode_current::<PointOutcome>(&encoded, "outcome")
                .unwrap()
                .is_none()
        );
        for broken in [
            serde_json::json!({"version":2,"kind":"legacy_unavailable"}),
            serde_json::json!({"version":1,"kind":"legacy_unavailable","usable":true}),
            serde_json::json!({"version":1,"kind":"legacy_unavailable","predecessor":"2"}),
        ] {
            assert!(decode_current::<PointPolicy>(&broken, "policy").is_err());
        }
        let point = PointStatus {
            point_index: 7,
            binding_hash: pse_ids::roles::BindingHash::from_bytes([4; 32]),
            state: StudyPointState::Completed,
            revision: 0,
            policy: None,
            legacy: None,
            outcome: None,
            receipt: None,
            member_attempt: None,
            job_id: crate::mint_id(),
            job_state: JobState::Completed,
            attempt_id: crate::mint_id(),
            attempt_state: AttemptState::Completed,
            tries: 1,
            termination_detail: None,
            last_error: None,
        };
        assert!(policy_snapshot(&[point], None).is_err());
    }
}
