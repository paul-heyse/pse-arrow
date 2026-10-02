// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed store failures. Retry decisions read the variant, never the message.
//!
//! Driver failures are classified by `SqlState` constant and connection state
//! (ADR-0114 Outcome 27) and keep their cause as a [`DriverError`], so no driver type
//! appears in this crate's public API.

use std::fmt;
use std::sync::Arc;

use pse_diagnostics::DiagnosticCode;
pub use pse_model::generated::enums::InvariantKind;
use pse_model::generated::enums::RetentionPhase;
use pse_model::generated::identities::{AttemptId, PublicationId, ReaderLeaseId, WorkspaceId};
use tokio_postgres::error::SqlState;

use crate::lifecycle::AttemptState;

/// The connection target a failure concerns, without credentials (architecture §9.9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target(Arc<str>);

impl Target {
    /// A target described by its socket directory or host, port, database and user.
    pub(crate) fn describe(config: &tokio_postgres::Config) -> Self {
        let place = config
            .get_hosts()
            .iter()
            .map(|host| match host {
                tokio_postgres::config::Host::Tcp(name) => name.clone(),
                #[cfg(unix)]
                tokio_postgres::config::Host::Unix(path) => path.display().to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let port = config
            .get_ports()
            .first()
            .map_or_else(|| "5432".to_owned(), u16::to_string);
        let database = config.get_dbname().unwrap_or("<default>");
        let user = config.get_user().unwrap_or("<process user>");
        Self(Arc::from(format!(
            "postgres {place}:{port}/{database} as {user}"
        )))
    }

    pub(crate) fn migration() -> Self {
        Self(Arc::from("operational schema migration"))
    }

    /// The description, safe to show: it never contains a password.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The cause of a classified store failure: the driver's or the pool's error, opaque.
#[derive(Debug)]
pub struct DriverError(Box<dyn std::error::Error + Send + Sync>);

impl DriverError {
    pub(crate) fn new(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self(Box::new(error))
    }
}

/// The driver's message with its whole cause chain (the server's `ERROR: ...` included):
/// the chain is opaque, so it is rendered here rather than exposed as sources.
impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        let mut cause = self.0.source();
        while let Some(error) = cause {
            write!(f, ": {error}")?;
            cause = error.source();
        }
        Ok(())
    }
}

impl std::error::Error for DriverError {}

/// A failure of the operational store or of a store-owned contract.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OperationsError {
    /// SQLSTATE 40001 or 40P01: a serialization failure, or a deadlock the server broke by
    /// aborting this transaction. Retry the whole transaction.
    #[error("transaction aborted for a retry at {target}: {source}")]
    Retryable {
        /// Where the transaction ran.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// SQLSTATE 23505: a unique constraint refused a second row with the same key.
    #[error("duplicate key{} at {target}: {source}", constraint.as_deref().map(|c| format!(" ({c})")).unwrap_or_default())]
    Duplicate {
        /// The violated constraint, when the server named it.
        constraint: Option<String>,
        /// Where the refusal happened.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// SQLSTATE 23514 or 23503: a row violated a declared invariant of a store relation,
    /// a row check or a reference. A platform or request defect, never retried.
    #[error(
        "{} violates {}{} at {target}: {source}",
        table.as_deref().unwrap_or("a store value"),
        match kind { InvariantKind::ForeignKey => "reference ", _ => "check " },
        constraint.as_deref().unwrap_or("<unnamed>")
    )]
    InvariantViolation {
        /// The store table, when the server named it.
        table: Option<String>,
        /// The registry name of the violated check or reference (`step_nonnegative`,
        /// `attempt_id`), when the server named the constraint.
        constraint: Option<String>,
        /// [`InvariantKind::Check`] for 23514, [`InvariantKind::ForeignKey`] for 23503.
        kind: InvariantKind,
        /// Where the refusal happened.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// SQLSTATE 55P03: a `NOWAIT` or lock-timeout acquisition failed.
    #[error("lock unavailable at {target}: {source}")]
    LockUnavailable {
        /// Where the lock was refused.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// SQLSTATE 57014: the statement was cancelled (statement timeout or cancel request).
    #[error("statement cancelled at {target}: {source}")]
    Cancelled {
        /// Where the statement ran.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// The server could not be reached, refused new connections, or the connection was
    /// lost (SQLSTATE class 08, 57P01–57P03, a closed connection or a pool failure).
    #[error("operational store unavailable at {target}: {source}")]
    Unavailable {
        /// The connection target.
        target: Target,
        /// The driver or pool error.
        source: DriverError,
    },
    /// Any other driver or server failure; a platform bug or an unexpected server state.
    #[error("operational store failure at {target}: {source}")]
    Internal {
        /// Where the failure happened.
        target: Target,
        /// The driver error.
        source: DriverError,
    },
    /// An absent schema requires explicit creation; ordinary opening never writes DDL.
    #[error("{target} has no pse_ops schema; explicitly create it")]
    SchemaAbsent {
        /// The store.
        target: Target,
    },
    /// The store's schema is not the one this build generates: another fingerprint is
    /// recorded, or none. It is never migrated or reset implicitly (ADR-0114 Outcome 23).
    #[error(
        "{target} holds a pse_ops schema recorded as {}, but this build expects {expected}",
        recorded.as_deref().unwrap_or("<no fingerprint>")
    )]
    SchemaMismatch {
        /// The store.
        target: Target,
        /// The recorded schema fingerprint, if any.
        recorded: Option<String>,
        /// This build's schema fingerprint.
        expected: &'static str,
    },
    /// An explicit schema transition or generation admission was refused without fallback.
    #[error("operational schema transition refused ({reason:?}): {detail}")]
    MigrationRefused {
        /// Typed source, history, readiness or quiescence cause.
        reason: crate::schema::MigrationRefusal,
        /// Attributable context, never interpreted as policy.
        detail: String,
    },
    /// The connection URL or the server does not satisfy the deployment contract.
    #[error("invalid operational store configuration: {reason}")]
    Configuration {
        /// What is wrong.
        reason: String,
    },
    /// The one transition table refuses this state change (DP-03).
    #[error("attempt {attempt}: illegal transition {} -> {}", from.as_str(), to.as_str())]
    IllegalTransition {
        /// The attempt.
        attempt: AttemptId,
        /// Its current state.
        from: AttemptState,
        /// The refused target state.
        to: AttemptState,
    },
    /// The request cannot be satisfied as stated (a boundary refusal, not a store failure).
    #[error("invalid request: {reason}")]
    InvalidRequest {
        /// What is wrong.
        reason: String,
    },
    /// The worker no longer owns a live lease on the attempt.
    #[error("attempt {attempt}: worker {worker} holds no live lease")]
    LeaseLost {
        /// The attempt.
        attempt: AttemptId,
        /// The worker that asked.
        worker: String,
    },
    /// A referenced row does not exist.
    #[error("{entity} {id} does not exist")]
    NotFound {
        /// What was looked up.
        entity: &'static str,
        /// Its identity.
        id: String,
    },
    /// The catalog head moved: the commit named a parent that is no longer the head. The
    /// publisher re-prepares against `current` and never rebases.
    #[error("workspace {workspace}: expected head {}, found {}", show(*.expected), show(*.current))]
    PublicationConflict {
        /// The workspace.
        workspace: WorkspaceId,
        /// The parent the commit named.
        expected: Option<PublicationId>,
        /// The head actually found.
        current: Option<PublicationId>,
    },
    /// The publication is expiring or deleted; no new reader lease is granted.
    #[error("publication {publication} is {}; no new reader lease", phase.as_str())]
    PublicationRetiring {
        /// The publication.
        publication: PublicationId,
        /// Its retention phase.
        phase: RetentionPhase,
    },
    /// The workspace head is protected from retention.
    #[error("publication {publication} is the head of its workspace and is protected")]
    ProtectedPublication {
        /// The publication.
        publication: PublicationId,
    },
    /// Maintenance must wait: reader leases are still live.
    #[error("publication {publication} still has {active} live reader lease(s)")]
    ReadersActive {
        /// The publication.
        publication: PublicationId,
        /// Live lease count.
        active: i64,
    },
    /// A publication or attempt identity names a different request than the one stored:
    /// another workspace, attempt or member vector, or an attempt already published as
    /// another publication (Plan 22 X9). Never retried.
    #[error("publication {publication}: identity reused for a different request ({reason})")]
    PublicationIdentityReused {
        /// The publication the request named.
        publication: PublicationId,
        /// What differs.
        reason: String,
    },
    /// The publication intent was abandoned (or reclaimed); it can never commit.
    #[error("publication intent {publication} was abandoned and can never commit")]
    IntentAbandoned {
        /// The publication.
        publication: PublicationId,
    },
    /// A retained member or input names a table version no live publication selects: it
    /// may be collected at any time, so a new publication cannot depend on it.
    #[error("{table_uri}@{delta_version} is selected by no live publication")]
    InputRetired {
        /// The table.
        table_uri: String,
        /// The version.
        delta_version: i64,
    },
    /// A reader lease expired or was released before it was renewed; the reader must stop.
    #[error("reader lease {lease} lapsed")]
    ReaderLeaseLapsed {
        /// The lease.
        lease: ReaderLeaseId,
    },
    /// A stored value violates the Rust-side contract (for example an unknown enum spelling).
    #[error("corrupt value in column {column}: {detail}")]
    CorruptValue {
        /// The column read.
        column: &'static str,
        /// What was wrong.
        detail: String,
    },
}

fn show(id: Option<PublicationId>) -> String {
    id.map_or_else(|| "<none>".to_owned(), |id| id.to_string())
}

/// The class of a driver failure (ADR-0114 Outcome 27).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    /// 40001, 40P01.
    Retryable,
    /// 55P03.
    LockUnavailable,
    /// 57014.
    Cancelled,
    /// 23505.
    Duplicate,
    /// 23514 (a check) and 23503 (a reference).
    Invariant(InvariantKind),
    /// Class 08, 57P01–57P03 and 53300, or a closed connection.
    Unavailable,
    /// Anything else.
    Internal,
}

/// SQLSTATEs by class; every entry is a `SqlState` constant, never a spelling.
fn class_of(state: &SqlState) -> Class {
    const RETRYABLE: [SqlState; 2] = [
        SqlState::T_R_SERIALIZATION_FAILURE,
        SqlState::T_R_DEADLOCK_DETECTED,
    ];
    // SQLSTATE class 08 (connection exception) as the driver knows it, the three
    // "operator intervention" shutdown states, and too many connections.
    const UNAVAILABLE: [SqlState; 11] = [
        SqlState::CONNECTION_EXCEPTION,
        SqlState::SQLCLIENT_UNABLE_TO_ESTABLISH_SQLCONNECTION,
        SqlState::CONNECTION_DOES_NOT_EXIST,
        SqlState::SQLSERVER_REJECTED_ESTABLISHMENT_OF_SQLCONNECTION,
        SqlState::CONNECTION_FAILURE,
        SqlState::TRANSACTION_RESOLUTION_UNKNOWN,
        SqlState::PROTOCOL_VIOLATION,
        SqlState::ADMIN_SHUTDOWN,
        SqlState::CRASH_SHUTDOWN,
        SqlState::CANNOT_CONNECT_NOW,
        SqlState::TOO_MANY_CONNECTIONS,
    ];
    if RETRYABLE.contains(state) {
        Class::Retryable
    } else if *state == SqlState::LOCK_NOT_AVAILABLE {
        Class::LockUnavailable
    } else if *state == SqlState::QUERY_CANCELED {
        Class::Cancelled
    } else if *state == SqlState::UNIQUE_VIOLATION {
        Class::Duplicate
    } else if *state == SqlState::CHECK_VIOLATION {
        Class::Invariant(InvariantKind::Check)
    } else if *state == SqlState::FOREIGN_KEY_VIOLATION {
        Class::Invariant(InvariantKind::ForeignKey)
    } else if UNAVAILABLE.contains(state) {
        Class::Unavailable
    } else {
        Class::Internal
    }
}

/// Classify a failure by its SQLSTATE and whether the connection is gone.
pub(crate) fn classify(state: Option<&SqlState>, closed: bool) -> Class {
    match (state, closed) {
        (_, true) => Class::Unavailable,
        (Some(state), false) => class_of(state),
        (None, false) => Class::Internal,
    }
}

/// The registry name of a generated constraint: `<table>_<name>_check` and
/// `<table>_<name>_fkey` are `<name>`; other constraints keep their name without the
/// table prefix (a field domain such as `elapsed_seconds_finite`).
fn registry_name(table: Option<&str>, constraint: &str) -> String {
    let local = table
        .and_then(|table| constraint.strip_prefix(table))
        .and_then(|rest| rest.strip_prefix('_'))
        .unwrap_or(constraint);
    local
        .strip_suffix("_check")
        .or_else(|| local.strip_suffix("_fkey"))
        .unwrap_or(local)
        .to_owned()
}

impl OperationsError {
    /// A classified failure of `class` with its cause and the server's constraint names.
    fn classified(
        class: Class,
        target: &Target,
        source: DriverError,
        table: Option<String>,
        constraint: Option<String>,
    ) -> Self {
        let target = target.clone();
        match class {
            Class::Retryable => Self::Retryable { target, source },
            Class::LockUnavailable => Self::LockUnavailable { target, source },
            Class::Cancelled => Self::Cancelled { target, source },
            Class::Duplicate => Self::Duplicate {
                constraint,
                target,
                source,
            },
            Class::Invariant(kind) => Self::InvariantViolation {
                constraint: constraint.map(|c| registry_name(table.as_deref(), &c)),
                table,
                kind,
                target,
                source,
            },
            Class::Unavailable => Self::Unavailable { target, source },
            Class::Internal => Self::Internal { target, source },
        }
    }

    /// Classify a driver error by SQLSTATE and connection state (ADR-0114 Outcome 27).
    pub(crate) fn from_driver(error: tokio_postgres::Error, target: &Target) -> Self {
        // An I/O cause is a lost or refused connection, whatever the driver's kind.
        let io =
            std::error::Error::source(&error).is_some_and(|cause| cause.is::<std::io::Error>());
        let class = classify(error.code(), error.is_closed() || io);
        let db = error.as_db_error();
        let table = db.and_then(|db| db.table()).map(str::to_owned);
        let constraint = db.and_then(|db| db.constraint()).map(str::to_owned);
        Self::classified(class, target, DriverError::new(error), table, constraint)
    }

    /// A pool that could not hand out a connection: the store is unavailable.
    pub(crate) fn from_pool(error: deadpool_postgres::PoolError, target: &Target) -> Self {
        Self::Unavailable {
            target: target.clone(),
            source: DriverError::new(error),
        }
    }

    /// Whether repeating the whole operation may succeed. Decided by the type alone.
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Retryable { .. } | Self::LockUnavailable { .. } | Self::Unavailable { .. }
        )
    }

    const fn code(&self) -> DiagnosticCode {
        match self {
            Self::Cancelled { .. } => DiagnosticCode::RuntimeCancelled,
            Self::Internal { .. } | Self::CorruptValue { .. } => DiagnosticCode::InternalInvariant,
            Self::Configuration { .. }
            | Self::SchemaAbsent { .. }
            | Self::SchemaMismatch { .. }
            | Self::MigrationRefused { .. } => DiagnosticCode::ConfigInvalid,
            Self::IllegalTransition { .. }
            | Self::NotFound { .. }
            | Self::InvalidRequest { .. }
            | Self::InvariantViolation { .. }
            | Self::PublicationIdentityReused { .. }
            | Self::IntentAbandoned { .. }
            | Self::InputRetired { .. } => DiagnosticCode::ValidationInvariant,
            Self::Retryable { .. }
            | Self::Duplicate { .. }
            | Self::LockUnavailable { .. }
            | Self::Unavailable { .. }
            | Self::LeaseLost { .. }
            | Self::PublicationConflict { .. }
            | Self::PublicationRetiring { .. }
            | Self::ProtectedPublication { .. }
            | Self::ReadersActive { .. }
            | Self::ReaderLeaseLapsed { .. } => DiagnosticCode::RuntimeInfrastructure,
        }
    }
}

pse_diagnostics::impl_diagnostic! {
    OperationsError,
    code(this) { Some(this.code()) },
    forward(_this) { None },
    help(this) {
        match this {
            OperationsError::Unavailable { .. } => Some(Box::new(
                "check `just db-status`; durable work needs the operational store, ephemeral work does not",
            )),
            OperationsError::SchemaAbsent { .. } => Some(Box::new("explicitly create an absent store with `just db-create`; ordinary open performs no DDL")),
            OperationsError::SchemaMismatch { .. } => Some(Box::new(
                "quiesce workers and close opened store generations, then use `just db-migrate`; unsupported baselines refuse without reset",
            )),
            _ => None,
        }
    },
    related(_this) { None },
    source(_this) { None }
}

/// Map driver results to typed store errors that name the connection target.
pub(crate) trait Classify<T> {
    /// Classify the error, if any.
    fn classify(self, target: &Target) -> Result<T, OperationsError>;
}

impl<T> Classify<T> for Result<T, tokio_postgres::Error> {
    fn classify(self, target: &Target) -> Result<T, OperationsError> {
        self.map_err(|source| OperationsError::from_driver(source, target))
    }
}

impl<T> Classify<T> for Result<T, deadpool_postgres::PoolError> {
    fn classify(self, target: &Target) -> Result<T, OperationsError> {
        self.map_err(|source| OperationsError::from_pool(source, target))
    }
}

#[cfg(test)]
mod error_unit {
    use super::*;
    use pse_ids::SemanticId;

    fn target() -> Target {
        Target(Arc::from("postgres test"))
    }

    #[test]
    fn sqlstate_classified_by_constant() {
        for (state, class) in [
            (SqlState::T_R_SERIALIZATION_FAILURE, Class::Retryable),
            (SqlState::T_R_DEADLOCK_DETECTED, Class::Retryable),
            (SqlState::LOCK_NOT_AVAILABLE, Class::LockUnavailable),
            (SqlState::QUERY_CANCELED, Class::Cancelled),
            (SqlState::UNIQUE_VIOLATION, Class::Duplicate),
            (
                SqlState::CHECK_VIOLATION,
                Class::Invariant(InvariantKind::Check),
            ),
            (
                SqlState::FOREIGN_KEY_VIOLATION,
                Class::Invariant(InvariantKind::ForeignKey),
            ),
            (SqlState::CONNECTION_FAILURE, Class::Unavailable),
            (SqlState::PROTOCOL_VIOLATION, Class::Unavailable),
            (SqlState::ADMIN_SHUTDOWN, Class::Unavailable),
            (SqlState::CRASH_SHUTDOWN, Class::Unavailable),
            (SqlState::CANNOT_CONNECT_NOW, Class::Unavailable),
            (SqlState::INSUFFICIENT_PRIVILEGE, Class::Internal),
            (SqlState::NOT_NULL_VIOLATION, Class::Internal),
        ] {
            assert_eq!(classify(Some(&state), false), class, "{}", state.code());
        }
        // A closed connection is unavailable whatever the server said last.
        assert_eq!(
            classify(Some(&SqlState::UNIQUE_VIOLATION), true),
            Class::Unavailable
        );
        assert_eq!(classify(None, true), Class::Unavailable);
        assert_eq!(classify(None, false), Class::Internal);
    }

    #[test]
    fn invariant_violations_name_the_registry_rule() {
        assert_eq!(
            registry_name(
                Some("progress_events"),
                "progress_events_step_nonnegative_check"
            ),
            "step_nonnegative"
        );
        assert_eq!(
            registry_name(
                Some("progress_values"),
                "progress_values_progress_event_fkey"
            ),
            "progress_event"
        );
        assert_eq!(
            registry_name(Some("incumbents"), "incumbents_objective_finite"),
            "objective_finite"
        );
        assert_eq!(
            registry_name(None, "content_hash_width"),
            "content_hash_width"
        );
        let violation = OperationsError::classified(
            Class::Invariant(InvariantKind::Check),
            &target(),
            DriverError::new(std::io::Error::other("refused")),
            Some("attempts".to_owned()),
            Some("attempts_running_holds_lease_check".to_owned()),
        );
        assert!(
            matches!(
                &violation,
                OperationsError::InvariantViolation { table: Some(t), constraint: Some(c), kind: InvariantKind::Check, .. }
                    if t == "attempts" && c == "running_holds_lease"
            ),
            "{violation:?}"
        );
        assert!(!violation.is_retryable());
        assert_eq!(violation.code(), DiagnosticCode::ValidationInvariant);
    }

    #[test]
    fn connection_failures_are_unavailable_and_retryable() {
        let error = OperationsError::classified(
            classify(None, true),
            &target(),
            DriverError::new(std::io::Error::other("connection reset")),
            None,
            None,
        );
        assert!(matches!(error, OperationsError::Unavailable { .. }));
        assert!(error.is_retryable());
        assert_eq!(error.code(), DiagnosticCode::RuntimeInfrastructure);
        assert!(error.to_string().contains("postgres test"));
    }

    #[test]
    fn domain_refusals_are_not_retryable() {
        let error = OperationsError::IllegalTransition {
            attempt: AttemptId::from_id(SemanticId::NIL),
            from: AttemptState::Completed,
            to: AttemptState::Running,
        };
        assert!(!error.is_retryable());
        assert_eq!(error.code(), DiagnosticCode::ValidationInvariant);
        let rendered = miette::Diagnostic::code(&error).map(|c| c.to_string());
        assert_eq!(rendered.as_deref(), Some("validation::invariant"));
    }
}
