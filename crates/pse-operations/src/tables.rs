// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Paged, filtered reads of the operational relations: what the query surface scans
//! (Plan 22 O9).
//!
//! Each [`Scan`] is the typed filter of one relation's generated statement
//! (`queries/tables.sql`). A page is the registry rows after a keyset position in
//! primary-key order, so a caller streams a relation in bounded pages and holds no
//! connection between them. An empty list or an open bound leaves its column
//! unconstrained; time bounds are inclusive. Pages are read at READ COMMITTED: a scan
//! returns every row that existed throughout it exactly once, and may or may not return a
//! row written while it runs.

use std::fmt::Debug;
use std::future::Future;

use chrono::{DateTime, Utc};
use pse_model::generated::enums::{
    AttemptState, JobState, PublicationMemberRole, StudyPointState, StudyState,
};
use pse_model::generated::identities::{
    AttemptId, JobId, PublicationId, RunId, SettlementId, SolutionId, StudyId, WorkspaceId,
};
use pse_model::generated::runtime::{
    operational_attempt_transitions::RuntimeOperationalAttemptTransitionsRow,
    operational_attempts::RuntimeOperationalAttemptsRow,
    operational_incumbents::RuntimeOperationalIncumbentsRow,
    operational_jobs::RuntimeOperationalJobsRow,
    operational_progress_events::RuntimeOperationalProgressEventsRow,
    operational_progress_values::RuntimeOperationalProgressValuesRow,
    operational_publication_members::RuntimeOperationalPublicationMembersRow,
    operational_publications::RuntimeOperationalPublicationsRow,
    operational_settlements::RuntimeOperationalSettlementsRow,
    operational_solutions::RuntimeOperationalSolutionsRow,
    operational_studies::RuntimeOperationalStudiesRow,
    operational_study_points::RuntimeOperationalStudyPointsRow,
    operational_workspaces::RuntimeOperationalWorkspacesRow,
};
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::tables as statements;

use crate::error::{Classify, OperationsError};
use crate::store::Store;

/// An inclusive interval of instants; an absent end is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeRange {
    /// The earliest instant included.
    pub from: Option<DateTime<Utc>>,
    /// The latest instant included.
    pub to: Option<DateTime<Utc>>,
}

/// The typed filter of one operational relation's paged statement.
pub trait Scan: Clone + Debug + Default + PartialEq + Send + Sync + 'static {
    /// The registry row the statement returns.
    type Row: Send + 'static;
    /// The primary key a page continues after.
    type Key: Clone + Debug + PartialEq + Send + Sync + 'static;

    /// The store table, which is the relation's name under `pse_ops`.
    const TABLE: &'static str;

    /// The primary key of a row.
    fn key(row: &Self::Row) -> Self::Key;

    /// The rows this filter selects after `after` in primary-key order, at most `limit`.
    fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<Self::Row>, OperationsError>> + Send;
}

fn positive(table: &str, limit: i64) -> Result<(), OperationsError> {
    if limit > 0 {
        Ok(())
    } else {
        Err(OperationsError::InvalidRequest {
            reason: format!("a page of {table} needs a positive limit, not {limit}"),
        })
    }
}

/// `runtime.operational_attempts`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttemptScan {
    /// Only these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only the attempts of these runs.
    pub runs: Vec<RunId>,
    /// Only attempts in these states.
    pub states: Vec<AttemptState>,
    /// Only attempts created in this interval.
    pub created: TimeRange,
}

impl Scan for AttemptScan {
    type Row = RuntimeOperationalAttemptsRow;
    type Key = AttemptId;
    const TABLE: &'static str = "attempts";

    fn key(row: &Self::Row) -> Self::Key {
        row.attempt_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_attempts()
            .params(
                &client,
                &statements::ScanAttemptsParams {
                    after: after.copied(),
                    attempts: self.attempts.as_slice(),
                    runs: self.runs.as_slice(),
                    states: self.states.as_slice(),
                    created_from: self.created.from,
                    created_to: self.created.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_attempt_transitions`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransitionScan {
    /// Only the transitions of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only transitions made in this interval.
    pub at: TimeRange,
}

impl Scan for TransitionScan {
    type Row = RuntimeOperationalAttemptTransitionsRow;
    type Key = (AttemptId, i32);
    const TABLE: &'static str = "attempt_transitions";

    fn key(row: &Self::Row) -> Self::Key {
        (row.attempt_id, row.seq)
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_attempt_transitions()
            .params(
                &client,
                &statements::ScanAttemptTransitionsParams {
                    after_attempt: after.map(|key| key.0),
                    after_seq: after.map(|key| key.1),
                    attempts: self.attempts.as_slice(),
                    at_from: self.at.from,
                    at_to: self.at.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_jobs`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobScan {
    /// Only these jobs.
    pub jobs: Vec<JobId>,
    /// Only the jobs whose current try is one of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only jobs in these states.
    pub states: Vec<JobState>,
    /// Only jobs enqueued in this interval.
    pub enqueued: TimeRange,
}

impl Scan for JobScan {
    type Row = RuntimeOperationalJobsRow;
    type Key = JobId;
    const TABLE: &'static str = "jobs";

    fn key(row: &Self::Row) -> Self::Key {
        row.job_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_jobs()
            .params(
                &client,
                &statements::ScanJobsParams {
                    after: after.copied(),
                    jobs: self.jobs.as_slice(),
                    attempts: self.attempts.as_slice(),
                    states: self.states.as_slice(),
                    enqueued_from: self.enqueued.from,
                    enqueued_to: self.enqueued.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_progress_events`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgressEventScan {
    /// Only the events of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only events observed in this interval.
    pub at: TimeRange,
}

impl Scan for ProgressEventScan {
    type Row = RuntimeOperationalProgressEventsRow;
    type Key = (AttemptId, i64);
    const TABLE: &'static str = "progress_events";

    fn key(row: &Self::Row) -> Self::Key {
        (row.attempt_id, row.seq)
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_progress_events()
            .params(
                &client,
                &statements::ScanProgressEventsParams {
                    after_attempt: after.map(|key| key.0),
                    after_seq: after.map(|key| key.1),
                    attempts: self.attempts.as_slice(),
                    at_from: self.at.from,
                    at_to: self.at.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_progress_values`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgressValueScan {
    /// Only the values of these attempts' events.
    pub attempts: Vec<AttemptId>,
}

impl Scan for ProgressValueScan {
    type Row = RuntimeOperationalProgressValuesRow;
    type Key = (AttemptId, i64, String);
    const TABLE: &'static str = "progress_values";

    fn key(row: &Self::Row) -> Self::Key {
        (row.attempt_id, row.seq, row.name.clone())
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_progress_values()
            .params(
                &client,
                &statements::ScanProgressValuesParams {
                    after_attempt: after.map(|key| key.0),
                    after_seq: after.map(|key| key.1),
                    after_name: after.map(|key| key.2.as_str()),
                    attempts: self.attempts.as_slice(),
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_incumbents`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncumbentScan {
    /// Only the incumbents of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only incumbents observed in this interval.
    pub at: TimeRange,
}

impl Scan for IncumbentScan {
    type Row = RuntimeOperationalIncumbentsRow;
    type Key = (AttemptId, i64);
    const TABLE: &'static str = "incumbents";

    fn key(row: &Self::Row) -> Self::Key {
        (row.attempt_id, row.seq)
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_incumbents()
            .params(
                &client,
                &statements::ScanIncumbentsParams {
                    after_attempt: after.map(|key| key.0),
                    after_seq: after.map(|key| key.1),
                    attempts: self.attempts.as_slice(),
                    at_from: self.at.from,
                    at_to: self.at.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_solutions`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SolutionScan {
    /// Only these solutions.
    pub solutions: Vec<SolutionId>,
    /// Only the solutions these attempts stored.
    pub creators: Vec<AttemptId>,
    /// Only solutions stored in this interval.
    pub created: TimeRange,
}

impl Scan for SolutionScan {
    type Row = RuntimeOperationalSolutionsRow;
    type Key = SolutionId;
    const TABLE: &'static str = "solutions";

    fn key(row: &Self::Row) -> Self::Key {
        row.solution_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_solutions()
            .params(
                &client,
                &statements::ScanSolutionsParams {
                    after: after.copied(),
                    solutions: self.solutions.as_slice(),
                    creators: self.creators.as_slice(),
                    created_from: self.created.from,
                    created_to: self.created.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_studies`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StudyScan {
    /// Only these studies.
    pub studies: Vec<StudyId>,
    /// Only the studies coordinated by these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only studies in these states.
    pub states: Vec<StudyState>,
    /// Only studies created in this interval.
    pub created: TimeRange,
}

impl Scan for StudyScan {
    type Row = RuntimeOperationalStudiesRow;
    type Key = StudyId;
    const TABLE: &'static str = "studies";

    fn key(row: &Self::Row) -> Self::Key {
        row.study_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_studies()
            .params(
                &client,
                &statements::ScanStudiesParams {
                    after: after.copied(),
                    studies: self.studies.as_slice(),
                    attempts: self.attempts.as_slice(),
                    states: self.states.as_slice(),
                    created_from: self.created.from,
                    created_to: self.created.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_study_points`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StudyPointScan {
    /// Only the points of these studies.
    pub studies: Vec<StudyId>,
    /// Only the points these jobs run.
    pub jobs: Vec<JobId>,
    /// Only points in these states.
    pub states: Vec<StudyPointState>,
}

impl Scan for StudyPointScan {
    type Row = RuntimeOperationalStudyPointsRow;
    type Key = (StudyId, i32);
    const TABLE: &'static str = "study_points";

    fn key(row: &Self::Row) -> Self::Key {
        (row.study_id, row.point_index)
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_study_points()
            .params(
                &client,
                &statements::ScanStudyPointsParams {
                    after_study: after.map(|key| key.0),
                    after_index: after.map(|key| key.1),
                    studies: self.studies.as_slice(),
                    jobs: self.jobs.as_slice(),
                    states: self.states.as_slice(),
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_workspaces`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceScan {
    /// Only these workspaces.
    pub workspaces: Vec<WorkspaceId>,
}

impl Scan for WorkspaceScan {
    type Row = RuntimeOperationalWorkspacesRow;
    type Key = WorkspaceId;
    const TABLE: &'static str = "workspaces";

    fn key(row: &Self::Row) -> Self::Key {
        row.workspace_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_workspaces()
            .params(
                &client,
                &statements::ScanWorkspacesParams {
                    after: after.copied(),
                    workspaces: self.workspaces.as_slice(),
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_publications`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublicationScan {
    /// Only these publications.
    pub publications: Vec<PublicationId>,
    /// Only the publications of these workspaces.
    pub workspaces: Vec<WorkspaceId>,
    /// Only the publications of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only publications committed in this interval.
    pub committed: TimeRange,
}

impl Scan for PublicationScan {
    type Row = RuntimeOperationalPublicationsRow;
    type Key = PublicationId;
    const TABLE: &'static str = "publications";

    fn key(row: &Self::Row) -> Self::Key {
        row.publication_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_publications()
            .params(
                &client,
                &statements::ScanPublicationsParams {
                    after: after.copied(),
                    publications: self.publications.as_slice(),
                    workspaces: self.workspaces.as_slice(),
                    attempts: self.attempts.as_slice(),
                    committed_from: self.committed.from,
                    committed_to: self.committed.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_publication_members`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PublicationMemberScan {
    /// Only the members of these publications.
    pub publications: Vec<PublicationId>,
}

impl Scan for PublicationMemberScan {
    type Row = RuntimeOperationalPublicationMembersRow;
    type Key = (PublicationId, PublicationMemberRole, String, String, String);
    const TABLE: &'static str = "publication_members";

    fn key(row: &Self::Row) -> Self::Key {
        (
            row.publication_id,
            row.role,
            row.catalog_name.clone(),
            row.schema_name.clone(),
            row.table_name.clone(),
        )
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_publication_members()
            .params(
                &client,
                &statements::ScanPublicationMembersParams {
                    after_publication: after.map(|key| key.0),
                    after_role: after.map(|key| key.1),
                    after_catalog: after.map(|key| key.2.as_str()),
                    after_schema: after.map(|key| key.3.as_str()),
                    after_table: after.map(|key| key.4.as_str()),
                    publications: self.publications.as_slice(),
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}

/// `runtime.operational_settlements`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettlementScan {
    /// Only the settlements of these attempts.
    pub attempts: Vec<AttemptId>,
    /// Only settlements made in this interval.
    pub settled: TimeRange,
}

impl Scan for SettlementScan {
    type Row = RuntimeOperationalSettlementsRow;
    type Key = SettlementId;
    const TABLE: &'static str = "settlements";

    fn key(row: &Self::Row) -> Self::Key {
        row.settlement_id
    }

    async fn page(
        &self,
        store: &Store,
        after: Option<&Self::Key>,
        limit: i64,
    ) -> Result<Vec<Self::Row>, OperationsError> {
        positive(Self::TABLE, limit)?;
        let client = store.client().await?;
        statements::scan_settlements()
            .params(
                &client,
                &statements::ScanSettlementsParams {
                    after: after.copied(),
                    attempts: self.attempts.as_slice(),
                    settled_from: self.settled.from,
                    settled_to: self.settled.to,
                    limit,
                },
            )
            .all()
            .await
            .classify(store.target())
    }
}
