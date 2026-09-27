// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed store failures. Retry decisions read the variant, never the message.

use std::fmt;
use std::sync::Arc;

use pse_diagnostics::DiagnosticCode;
use pse_ids::SemanticId;

use crate::lifecycle::AttemptState;

/// The connection target a failure concerns, without credentials (architecture §9.9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target(Arc<str>);

impl Target {
    /// A target described by its socket directory or host, port and database.
    pub(crate) fn describe(options: &sqlx::postgres::PgConnectOptions) -> Self {
        let place = options.get_socket().map_or_else(
            || format!("{}:{}", options.get_host(), options.get_port()),
            |socket| format!("{}:{}", socket.display(), options.get_port()),
        );
        let database = options.get_database().unwrap_or("<default>");
        Self(Arc::from(format!(
            "postgres {place}/{database} as {}",
            options.get_username()
        )))
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

/// A failure of the operational store or of a store-owned contract.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OperationsError {
    /// SQLSTATE 40001: a serialization failure. Retry the whole transaction.
    #[error("serialization conflict at {target}: {source}")]
    Conflict {
        /// Where the conflict happened.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// SQLSTATE 40P01: the server broke a deadlock by aborting this transaction.
    #[error("deadlock detected at {target}: {source}")]
    Deadlock {
        /// Where the deadlock happened.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// SQLSTATE 23505: a unique constraint refused a second row with the same key.
    #[error("duplicate key{} at {target}: {source}", constraint.as_deref().map(|c| format!(" ({c})")).unwrap_or_default())]
    Duplicate {
        /// The violated constraint, when the server named it.
        constraint: Option<String>,
        /// Where the refusal happened.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// SQLSTATE 55P03: a `NOWAIT` or lock-timeout acquisition failed.
    #[error("lock unavailable at {target}: {source}")]
    LockUnavailable {
        /// Where the lock was refused.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// SQLSTATE 57014: the statement was cancelled (statement timeout or cancel request).
    #[error("statement cancelled at {target}: {source}")]
    Cancelled {
        /// Where the statement ran.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// The server could not be reached or the connection was lost.
    #[error("operational store unavailable at {target}: {source}")]
    Unavailable {
        /// The connection target.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// Any other driver or server failure; a platform bug or an unexpected server state.
    #[error("operational store failure at {target}: {source}")]
    Internal {
        /// Where the failure happened.
        target: Target,
        /// The driver error.
        source: sqlx::Error,
    },
    /// Applying or inspecting the embedded migrations failed.
    #[error("migration failure at {target}: {source}")]
    Migration {
        /// The migrated database.
        target: Target,
        /// The migrator error.
        source: sqlx::migrate::MigrateError,
    },
    /// The connection URL or the server does not satisfy the deployment contract.
    #[error("invalid operational store configuration: {reason}")]
    Configuration {
        /// What is wrong.
        reason: String,
    },
    /// The one transition table refuses this state change (DP-03).
    #[error("attempt {attempt}: illegal transition {from} -> {to}")]
    IllegalTransition {
        /// The attempt.
        attempt: SemanticId,
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
        attempt: SemanticId,
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
        workspace: SemanticId,
        /// The parent the commit named.
        expected: Option<SemanticId>,
        /// The head actually found.
        current: Option<SemanticId>,
    },
    /// The publication is expiring or deleted; no new reader lease is granted.
    #[error("publication {publication} is {phase}; no new reader lease")]
    PublicationRetiring {
        /// The publication.
        publication: SemanticId,
        /// Its retention phase.
        phase: String,
    },
    /// The workspace head is protected from retention.
    #[error("publication {publication} is the head of its workspace and is protected")]
    ProtectedPublication {
        /// The publication.
        publication: SemanticId,
    },
    /// Maintenance must wait: reader leases are still live.
    #[error("publication {publication} still has {active} live reader lease(s)")]
    ReadersActive {
        /// The publication.
        publication: SemanticId,
        /// Live lease count.
        active: i64,
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

fn show(id: Option<SemanticId>) -> String {
    id.map_or_else(|| "<none>".to_owned(), |id| id.to_string())
}

impl OperationsError {
    /// Classify a driver error by SQLSTATE and connection failure (ADR-0112 Outcome 11).
    pub(crate) fn from_sqlx(source: sqlx::Error, target: &Target) -> Self {
        let target = target.clone();
        let code = match &source {
            sqlx::Error::Database(database) => database.code().map(|c| c.into_owned()),
            sqlx::Error::Io(_)
            | sqlx::Error::Tls(_)
            | sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::WorkerCrashed => return Self::Unavailable { target, source },
            sqlx::Error::Migrate(_) => None,
            _ => return Self::Internal { target, source },
        };
        match code.as_deref() {
            Some("40001") => Self::Conflict { target, source },
            Some("40P01") => Self::Deadlock { target, source },
            Some("23505") => {
                let constraint = match &source {
                    sqlx::Error::Database(database) => database.constraint().map(str::to_owned),
                    _ => None,
                };
                Self::Duplicate {
                    constraint,
                    target,
                    source,
                }
            }
            Some("55P03") => Self::LockUnavailable { target, source },
            Some("57014") => Self::Cancelled { target, source },
            // Connection exceptions, administrator/crash shutdown, cannot connect now and
            // too many connections: the server is not usable from here.
            Some(code)
                if code.starts_with("08")
                    || matches!(code, "57P01" | "57P02" | "57P03" | "53300") =>
            {
                Self::Unavailable { target, source }
            }
            _ => Self::Internal { target, source },
        }
    }

    /// Whether repeating the whole operation may succeed. Decided by the type alone.
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Conflict { .. }
                | Self::Deadlock { .. }
                | Self::LockUnavailable { .. }
                | Self::Unavailable { .. }
        )
    }

    const fn code(&self) -> DiagnosticCode {
        match self {
            Self::Cancelled { .. } => DiagnosticCode::RuntimeCancelled,
            Self::Internal { .. } | Self::CorruptValue { .. } => DiagnosticCode::InternalInvariant,
            Self::Configuration { .. } => DiagnosticCode::ConfigInvalid,
            Self::IllegalTransition { .. }
            | Self::NotFound { .. }
            | Self::InvalidRequest { .. } => DiagnosticCode::ValidationInvariant,
            Self::Conflict { .. }
            | Self::Deadlock { .. }
            | Self::Duplicate { .. }
            | Self::LockUnavailable { .. }
            | Self::Unavailable { .. }
            | Self::Migration { .. }
            | Self::LeaseLost { .. }
            | Self::PublicationConflict { .. }
            | Self::PublicationRetiring { .. }
            | Self::ProtectedPublication { .. }
            | Self::ReadersActive { .. } => DiagnosticCode::RuntimeInfrastructure,
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
            OperationsError::Migration { .. } => Some(Box::new("run `just db-migrate`")),
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

impl<T> Classify<T> for Result<T, sqlx::Error> {
    fn classify(self, target: &Target) -> Result<T, OperationsError> {
        self.map_err(|source| OperationsError::from_sqlx(source, target))
    }
}

#[cfg(test)]
mod error_unit {
    use super::*;

    fn target() -> Target {
        Target(Arc::from("postgres test"))
    }

    #[test]
    fn connection_failures_are_unavailable_and_retryable() {
        let error = OperationsError::from_sqlx(sqlx::Error::PoolTimedOut, &target());
        assert!(matches!(error, OperationsError::Unavailable { .. }));
        assert!(error.is_retryable());
        assert_eq!(error.code(), DiagnosticCode::RuntimeInfrastructure);
        assert!(error.to_string().contains("postgres test"));
    }

    #[test]
    fn domain_refusals_are_not_retryable() {
        let error = OperationsError::IllegalTransition {
            attempt: SemanticId::NIL,
            from: AttemptState::Completed,
            to: AttemptState::Running,
        };
        assert!(!error.is_retryable());
        assert_eq!(error.code(), DiagnosticCode::ValidationInvariant);
        let rendered = miette::Diagnostic::code(&error).map(|c| c.to_string());
        assert_eq!(rendered.as_deref(), Some("validation::invariant"));
    }
}
