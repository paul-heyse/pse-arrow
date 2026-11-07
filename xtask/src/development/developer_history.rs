// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit pilot storage for authenticated native test observations.
//! The source owner supplies the facts. History never grants execution or file custody.

use pse_operations::{
    canonical::CanonicalError,
    transport::{
        self,
        client::{Client, Database, Query, RequestContext, ResponseLimits},
        lifetime::ConnectionAccess,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{io, net::SocketAddr, time::Instant};
use surrealdb_types::SurrealValue;

mod expiry;
pub use expiry::RetirementReceipt;

const MAX_FAILURES: usize = 64;
const MAX_FEATURES: usize = 64;
const MAX_FILTERS: usize = 32;
const MAX_STRING_BYTES: usize = 2_048;
const MAX_FACT_BYTES: usize = 64 * 1024;
const MAX_PREVIEW_ROWS: usize = 64;
const PILOT_MAX_LIFETIME_MS: i64 = 30 * 24 * 60 * 60 * 1_000;
const RESPONSE_LIMITS: ResponseLimits = ResponseLimits {
    wire_bytes: 128 * 1024,
    nodes: 12 * 1024,
    // The protobuf response nests typed objects and their value variants around the
    // publication, failed cases, and feature lists.
    depth: 32,
    container_items: 12 * 1024,
    string_bytes: 96 * 1024,
    string_length: MAX_STRING_BYTES,
    blob_bytes: 0,
    blob_length: 0,
};

fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Exact pilot DDL. Provision through a separate database administrator.
pub fn schema() -> String {
    include_str!("developer_history/schema.surql")
        .replace("__FAILURES__", &MAX_FAILURES.to_string())
        .replace("__FEATURES__", &MAX_FEATURES.to_string())
        .replace("__FILTERS__", &MAX_FILTERS.to_string())
}

/// Publication and readback failures, kept separate from the source test verdict.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum HistoryError {
    /// The fact, retention intent or explicit deployment is invalid.
    #[error("developer history rejected {0}")]
    Invalid(&'static str),
    /// A reader or writer does not identify the explicitly provisioned deployment.
    #[error("developer history deployment is unavailable or changed")]
    WrongDeployment,
    /// An invocation identity already means different immutable facts or retention.
    #[error("developer history invocation identity was reused with different content")]
    ReusedIdentity,
    /// The original invocation was already retired.
    #[error("developer history invocation identity was retired")]
    RetiredIdentity,
    /// The selected expiry intent no longer matches the original record.
    #[error("developer history expiry selection is absent or changed")]
    StaleExpiry,
    /// An independently owned positive hold protects this history record.
    #[error("developer history expiry is held by its cohort owner")]
    ExpiryHeld,
    /// The selected record has not reached its expiry in database time.
    #[error("developer history expiry is not due")]
    ExpiryNotDue,
    /// Independent readback could not verify the original operation.
    #[error("developer history {operation} for {invocation} remains unsettled")]
    Unsettled {
        /// Original operation.
        operation: &'static str,
        /// Exact original invocation to inspect before retrying.
        invocation: String,
        /// Original acknowledgement failure, or an acknowledged but unverified result.
        #[source]
        source: Box<CanonicalError>,
        /// Independent readback failure, if the reader was unavailable.
        readback: Option<Box<HistoryError>>,
    },
    /// Original typed failure from the existing bounded transport.
    #[error(transparent)]
    Transport(#[from] CanonicalError),
    /// Original native history selection or settlement was unavailable.
    #[cfg(feature = "host-contracts")]
    #[error("developer history could not acquire the original native test")]
    Original(#[source] Box<pse_host::HostError>),
}

impl From<surrealdb_types::Error> for HistoryError {
    fn from(error: surrealdb_types::Error) -> Self {
        Self::Transport(error.into())
    }
}

/// Failed selection reconciled against the original compiler and Nextest inventory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, SurrealValue)]
pub struct FailedCase {
    /// Cargo package name.
    pub package: String,
    /// Original Cargo target kind.
    pub target_kind: String,
    /// Original Cargo target name.
    pub target_name: String,
    /// Exact effective enabled features.
    pub enabled_features: Vec<String>,
    /// Features expressly required to remain absent.
    pub absent_features: Vec<String>,
    /// Original native binary ID.
    pub binary_id: String,
    /// Original exact harness test name.
    pub test_name: String,
}

/// Native filter input, preserving its two grammars and default-filter policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, SurrealValue)]
pub struct HistoryFilters {
    /// Whether the native default filter was bypassed.
    pub ignore_default_filter: bool,
    /// Original bare words.
    pub words: Vec<String>,
    /// Original full filtersets.
    pub expressions: Vec<String>,
}

/// Small persistence boundary populated from one authenticated, settled native test.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, SurrealValue)]
pub struct HistoryFact {
    /// Original CLI invocation ID.
    pub invocation: String,
    /// Original repository identity.
    pub repository_id: String,
    /// Original checkout identity.
    pub checkout_id: String,
    /// Original prepared control manifest.
    pub control_manifest_sha256: String,
    /// Original revision when independently observed.
    pub revision: Option<String>,
    /// Original invocation creation clock.
    pub created_unix_ms: i64,
    /// Original report interpretation version.
    pub interpretation: i64,
    /// Original reconciled native outcome.
    pub outcome: String,
    /// Original report commitment.
    pub report_sha256: String,
    /// Original inventory commitment.
    pub inventory_sha256: String,
    /// Original manager settlement commitment.
    pub settlement_sha256: String,
    /// Original selected population size.
    pub selected_count: i64,
    /// Original dependency omissions, when reported.
    pub omitted_evidence: Option<i64>,
    /// Bounded original typed diagnostic code occurrences.
    pub failure_codes: Vec<String>,
    /// Exact failures from complete reconciliation.
    pub failed: Vec<FailedCase>,
    /// Original effective test profile.
    pub profile: String,
    /// Original explicit target triple, if selected.
    pub platform: Option<String>,
    /// Original native filters.
    pub filters: HistoryFilters,
    /// Original ignored-test policy.
    pub ignored_policy: String,
}

#[cfg(feature = "host-contracts")]
impl TryFrom<pse_host::test_workflow::NativeTestHistory> for HistoryFact {
    type Error = HistoryError;

    fn try_from(original: pse_host::test_workflow::NativeTestHistory) -> Result<Self, Self::Error> {
        let label = |value: serde_json::Value| match value {
            serde_json::Value::String(value) => Ok(value),
            _ => Err(HistoryError::Invalid("original native vocabulary")),
        };
        let outcome = label(
            serde_json::to_value(original.outcome)
                .map_err(|_| HistoryError::Invalid("original native outcome"))?,
        )?;
        let ignored_policy = label(
            serde_json::to_value(original.ignored)
                .map_err(|_| HistoryError::Invalid("original native ignored policy"))?,
        )?;
        let count = |value| {
            i64::try_from(value).map_err(|_| HistoryError::Invalid("original population count"))
        };
        let failed = original
            .failed_cases
            .into_iter()
            .map(|case| {
                if case.features.defaults || case.features.all {
                    return Err(HistoryError::Invalid("failure requires effective features"));
                }
                Ok(FailedCase {
                    package: case.package,
                    target_kind: case
                        .target
                        .kind
                        .ok_or(HistoryError::Invalid("failure target kind is absent"))?,
                    target_name: case.target.name,
                    enabled_features: case.features.enabled.into_iter().collect(),
                    absent_features: case.features.absent.into_iter().collect(),
                    binary_id: case.binary_id,
                    test_name: case.test_name,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            invocation: original.invocation.into(),
            repository_id: original.repository_id,
            checkout_id: original.checkout_id,
            control_manifest_sha256: original.control_manifest_sha256,
            revision: None,
            created_unix_ms: i64::try_from(original.created_unix_ms)
                .map_err(|_| HistoryError::Invalid("original creation clock"))?,
            interpretation: i64::from(original.report_version),
            outcome,
            report_sha256: original.report_sha256,
            inventory_sha256: original.inventory_sha256,
            settlement_sha256: original.settlement_sha256,
            selected_count: count(original.selected_tests)?,
            omitted_evidence: original.omitted_evidence.map(count).transpose()?,
            failure_codes: original.failure_codes,
            failed,
            profile: original.profile,
            platform: original.platform,
            filters: HistoryFilters {
                ignore_default_filter: original.filters.ignore_default_filter,
                words: original.filters.words,
                expressions: original.filters.expressions,
            },
            ignored_policy,
        })
    }
}

fn lower_hex(value: &str, size: usize) -> bool {
    value.len() == size
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

fn bounded_list(values: &[String], limit: usize) -> bool {
    values.len() <= limit && values.iter().all(|value| value.len() <= MAX_STRING_BYTES)
}

fn bounded_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_STRING_BYTES
}

impl HistoryFact {
    fn validate(&self, expires_unix_ms: i64) -> Result<(), HistoryError> {
        if !uuid(&self.invocation)
            || ![
                &self.repository_id,
                &self.checkout_id,
                &self.control_manifest_sha256,
                &self.report_sha256,
                &self.inventory_sha256,
                &self.settlement_sha256,
            ]
            .into_iter()
            .all(|value| lower_hex(value, 64))
            || self
                .revision
                .as_ref()
                .is_some_and(|value| !lower_hex(value, 40) && !lower_hex(value, 64))
        {
            return Err(HistoryError::Invalid("original identity or commitments"));
        }
        if self.created_unix_ms <= 0
            || expires_unix_ms <= self.created_unix_ms
            || expires_unix_ms - self.created_unix_ms > PILOT_MAX_LIFETIME_MS
        {
            return Err(HistoryError::Invalid("pilot retention interval"));
        }
        if self.interpretation <= 0
            || self.selected_count < 0
            || self.omitted_evidence.is_some_and(|count| count < 0)
            || self.failed.len() > MAX_FAILURES
            || self.failed.len() as i64 > self.selected_count
            || !bounded_list(&self.failure_codes, MAX_FAILURES)
        {
            return Err(HistoryError::Invalid("original observation bounds"));
        }
        if !matches!(
            self.outcome.as_str(),
            "tests_succeeded" | "failed" | "zero_match" | "pending" | "incomplete"
        ) || (!self.failed.is_empty() && self.outcome != "failed")
            || (self.selected_count != 0 && self.outcome == "zero_match")
            || (self.selected_count == 0 && self.outcome == "tests_succeeded")
        {
            return Err(HistoryError::Invalid("original test verdict"));
        }
        if !matches!(self.ignored_policy.as_str(), "default" | "only" | "all")
            || !bounded_identity(&self.profile)
            || self.platform.as_deref().is_some_and(|p| !bounded_identity(p))
            || !bounded_list(&self.filters.words, MAX_FILTERS)
            || !bounded_list(&self.filters.expressions, MAX_FILTERS)
        {
            return Err(HistoryError::Invalid("original test selection bounds"));
        }
        for (index, case) in self.failed.iter().enumerate() {
            if ![
                &case.package,
                &case.target_kind,
                &case.target_name,
                &case.binary_id,
                &case.test_name,
            ]
            .into_iter()
            .all(|value| bounded_identity(value))
                || !bounded_list(&case.enabled_features, MAX_FEATURES)
                || !bounded_list(&case.absent_features, MAX_FEATURES)
                || self.failed[..index].contains(case)
            {
                return Err(HistoryError::Invalid("failed case identity or feature bounds"));
            }
        }
        Ok(())
    }

    fn content_sha256(&self, expires_unix_ms: i64) -> Result<String, HistoryError> {
        self.validate(expires_unix_ms)?;
        struct BoundedDigest {
            hash: Sha256,
            remaining: usize,
        }
        impl io::Write for BoundedDigest {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.remaining = self
                    .remaining
                    .checked_sub(bytes.len())
                    .ok_or_else(|| io::Error::other("pilot projection exceeds byte bound"))?;
                self.hash.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut digest = BoundedDigest {
            hash: Sha256::new(),
            remaining: MAX_FACT_BYTES,
        };
        serde_json::to_writer(&mut digest, &(1, expires_unix_ms, self))
            .map_err(|_| HistoryError::Invalid("encoded projection exceeds the pilot bound"))?;
        Ok(hex(&digest.hash.finalize()))
    }
}

/// One immutable publication and the expiry selected by the history owner.
#[derive(Clone, Debug, PartialEq, Eq, SurrealValue)]
pub struct HistoryPublication {
    /// Original fact projection and expiry commitment.
    pub content_sha256: String,
    /// Earliest owner expiry eligibility, in Unix milliseconds.
    pub expires_unix_ms: i64,
    /// Selected native facts.
    pub fact: HistoryFact,
}

impl HistoryPublication {
    /// Validate and bind a finite expiry to this exact publication intent.
    pub fn new(fact: HistoryFact, expires_unix_ms: i64) -> Result<Self, HistoryError> {
        let content_sha256 = fact.content_sha256(expires_unix_ms)?;
        Ok(Self {
            content_sha256,
            expires_unix_ms,
            fact,
        })
    }

    fn verify(&self, invocation: &str) -> Result<(), HistoryError> {
        if self.fact.invocation != invocation
            || self.content_sha256 != self.fact.content_sha256(self.expires_unix_ms)?
        {
            return Err(HistoryError::Invalid("stored record commitment"));
        }
        Ok(())
    }
}

/// Private database login with redacted diagnostic formatting.
pub struct HistoryCredential {
    /// Database-scoped user.
    pub username: String,
    /// Credential for that user.
    pub password: String,
}

impl std::fmt::Debug for HistoryCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryCredential").finish_non_exhaustive()
    }
}

/// Explicit scope and two distinct database logins.
#[derive(Debug)]
pub struct HistoryOptions {
    /// Selected literal loopback address.
    pub endpoint: SocketAddr,
    /// Dedicated pilot namespace.
    pub namespace: String,
    /// Dedicated pilot database.
    pub database: String,
    /// Marker of the originally provisioned deployment.
    pub issuer: String,
    /// Read-only user.
    pub reader: HistoryCredential,
    /// Publishing user.
    pub writer: HistoryCredential,
}

/// Operation roles within the provisioned pilot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryRole {
    /// Separately authenticated read-only connection.
    Reader,
    /// Separately authenticated publication connection.
    Writer,
}

#[derive(PartialEq, Eq)]
struct Scope {
    endpoint: SocketAddr,
    namespace: String,
    database: String,
    issuer: String,
    schema_sha256: String,
}

#[derive(SurrealValue)]
struct Deployment {
    issuer: String,
    schema_sha256: String,
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

impl Scope {
    fn from_options(options: &HistoryOptions) -> Result<Self, HistoryError> {
        if !options.endpoint.ip().is_loopback()
            || options.endpoint.port() == 0
            || !options.namespace.starts_with("pse_developer_history_")
            || !identifier(&options.namespace)
            || !identifier(&options.database)
            || !uuid(&options.issuer)
            || !identifier(&options.reader.username)
            || !identifier(&options.writer.username)
            || options.reader.username == options.writer.username
            || options.reader.password == options.writer.password
            || !bounded_identity(&options.reader.password)
            || !bounded_identity(&options.writer.password)
        {
            return Err(HistoryError::Invalid("explicit deployment or scoped credentials"));
        }
        Ok(Self {
            endpoint: options.endpoint,
            namespace: options.namespace.clone(),
            database: options.database.clone(),
            issuer: options.issuer.clone(),
            schema_sha256: digest(schema().as_bytes()),
        })
    }

    fn verify(&self, deployment: Option<&Deployment>) -> Result<(), HistoryError> {
        if deployment.is_some_and(|marker| {
            marker.issuer == self.issuer && marker.schema_sha256 == self.schema_sha256
        }) {
            Ok(())
        } else {
            Err(HistoryError::WrongDeployment)
        }
    }
}

/// Outcome of exact publication and independent readback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Published {
    /// A new row was acknowledged and independently read.
    Created,
    /// A matching immutable publication was already present.
    Existing,
    /// An ambiguous acknowledgement was resolved by exact independent readback.
    Reconciled,
}

/// One exact read, distinguishing absence from an expired publication awaiting owner release.
#[derive(Debug, PartialEq, Eq)]
pub enum HistoryRead {
    /// No matching row in the original database.
    Missing,
    /// Original facts remain within the selected expiry.
    Present(Box<HistoryPublication>),
    /// Expired record retained until its owner settles release.
    Expired(ExpiryCandidate),
    /// Expired facts were removed and their finite retirement receipt remains.
    Retired(RetirementReceipt),
}

/// Indexed lifecycle observation. Carries no release authority.
#[derive(Clone, Debug, PartialEq, Eq, SurrealValue)]
pub struct ExpiryCandidate {
    /// Exact native invocation.
    pub invocation: String,
    /// Exact content and expiry commitment.
    pub content_sha256: String,
    /// Recorded expiry in Unix milliseconds.
    pub expires_unix_ms: i64,
}

#[derive(SurrealValue)]
struct ExactRead {
    deployment: Option<Deployment>,
    publication: Option<HistoryPublication>,
    retirement: Option<RetirementReceipt>,
}

#[derive(SurrealValue)]
struct ExpiryRead {
    deployment: Option<Deployment>,
    candidates: Vec<ExpiryCandidate>,
}

/// Selected connection to one explicit developer-history database.
pub struct HistoryStore {
    client: Client,
    scope: Scope,
    role: HistoryRole,
    access: ConnectionAccess,
    #[cfg(test)]
    ack_fault: std::sync::atomic::AtomicU8,
}

impl std::fmt::Debug for HistoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryStore")
            .field("role", &self.role)
            .finish_non_exhaustive()
    }
}

const SELECT_DEPLOYMENT: &str =
    "SELECT issuer, schema_sha256 FROM ONLY developer_history_deployment:current";
const READ_EXACT: &str = "
    RETURN {
        deployment: (SELECT issuer, schema_sha256 FROM ONLY developer_history_deployment:current),
        publication: (SELECT content_sha256, expires_unix_ms, fact
            FROM ONLY type::record('developer_test_history', $invocation)),
        retirement: (SELECT invocation, content_sha256, expires_unix_ms,
            retired_at_unix_us, publication_closes_unix_ms
            FROM ONLY type::record('developer_test_history_retired', $invocation))
    };";
const READ_EXPIRY: &str = "
    RETURN {
        deployment: (SELECT issuer, schema_sha256 FROM ONLY developer_history_deployment:current),
        candidates: (SELECT fact.invocation AS invocation, content_sha256, expires_unix_ms
            FROM developer_test_history WITH INDEX developer_test_history_expiry
            WHERE expires_unix_ms <= $cutoff AND hold = NONE ORDER BY expires_unix_ms ASC LIMIT $limit)
    };";
#[cfg(any(test, feature = "host-contracts"))]
const CREATE_EXACT: &str = "
    BEGIN;
    LET $marker = SELECT issuer, schema_sha256, closed_through_unix_ms
        FROM ONLY developer_history_deployment:current;
    IF $marker = NONE OR $marker.issuer != $issuer OR $marker.schema_sha256 != $schema_sha256 {
        THROW 'developer history deployment changed';
    };
    IF (SELECT id FROM ONLY type::record('developer_test_history_retired', $invocation)) != NONE {
        THROW 'developer history invocation was retired';
    };
    LET $now = time::micros() / 1000;
    IF $row.fact.invocation != $invocation OR $row.fact.created_unix_ms <= 0
        OR $row.fact.created_unix_ms > $now OR $row.expires_unix_ms <= $row.fact.created_unix_ms
        OR $row.expires_unix_ms - $row.fact.created_unix_ms > $maximum_lifetime_ms
        OR $row.expires_unix_ms <= $now OR $row.expires_unix_ms <= $marker.closed_through_unix_ms {
        THROW 'developer history publication is outside its original window';
    };
    CREATE ONLY type::record('developer_test_history', $invocation) CONTENT $row RETURN NONE;
    COMMIT;";

impl HistoryStore {
    /// Open an independently authenticated role and verify the original pilot marker.
    pub async fn open(
        options: &HistoryOptions,
        role: HistoryRole,
        deadline: Instant,
    ) -> Result<Self, HistoryError> {
        let scope = Scope::from_options(options)?;
        let credential = match role {
            HistoryRole::Reader => &options.reader,
            HistoryRole::Writer => &options.writer,
        };
        transport::operation_scope(deadline, async move {
            let deadline = transport::original_deadline(transport::REQUEST_TIMEOUT);
            let client = transport::within_clock(deadline, async {
                Client::connect(scope.endpoint, RESPONSE_LIMITS.wire_bytes)
                    .await
                    .map_err(Into::into)
            })
            .await?;
            let store = Self {
                client,
                scope,
                role,
                access: ConnectionAccess::default(),
                #[cfg(test)]
                ack_fault: Default::default(),
            };
            let context = || RequestContext::control(deadline.into_std());
            let authenticated = async {
                transport::request(
                    store
                        .client
                        .signin(Database {
                            namespace: store.scope.namespace.clone(),
                            database: store.scope.database.clone(),
                            username: credential.username.clone(),
                            password: credential.password.clone(),
                        })
                        .request_context(context()),
                )
                .await?;
                transport::request(
                    store
                        .client
                        .use_ns(&store.scope.namespace)
                        .use_db(&store.scope.database)
                        .request_context(context()),
                )
                .await?;
                let mut reply = transport::bounded_query(store.query(SELECT_DEPLOYMENT)?).await?;
                store.scope.verify(reply.take::<Option<Deployment>>(0)?.as_ref())
            }
            .await;
            if let Err(error) = authenticated {
                let _ = store.disconnect().await;
                return Err(error);
            }
            Ok(store)
        })
        .await
    }

    fn query<'a>(&'a self, statement: &'a str) -> Result<Query<'a>, CanonicalError> {
        let activity = self.access.admit()?;
        let deadline = transport::original_deadline(transport::REQUEST_TIMEOUT);
        Ok(self.client.query(statement).request_context(
            RequestContext::new(deadline.into_std())
                .with_lifetime_owner(activity)
                .with_response_limits(RESPONSE_LIMITS),
        ))
    }

    async fn exact(&self, invocation: &str) -> Result<ExactRead, HistoryError> {
        if !uuid(invocation) {
            return Err(HistoryError::Invalid("native invocation identity"));
        }
        let mut response = transport::bounded_query(
            self.query(READ_EXACT)?.bind(("invocation", invocation)),
        )
        .await?;
        let reply = response
            .take::<Option<ExactRead>>(0)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        self.scope.verify(reply.deployment.as_ref())?;
        if let Some(publication) = &reply.publication {
            publication.verify(invocation)?;
        }
        if let Some(receipt) = &reply.retirement {
            receipt.verify(invocation)?;
            if reply.publication.is_some() {
                return Err(HistoryError::Invalid("live history and retirement both exist"));
            }
        }
        Ok(reply)
    }

    /// Independently read one exact native invocation, using the supplied expiry clock.
    pub async fn read(
        &self,
        invocation: &str,
        now_unix_ms: i64,
        deadline: Instant,
    ) -> Result<HistoryRead, HistoryError> {
        transport::operation_scope(deadline, async {
            let exact = self.exact(invocation).await?;
            if let Some(receipt) = exact.retirement {
                return Ok(HistoryRead::Retired(receipt));
            }
            Ok(match exact.publication {
                None => HistoryRead::Missing,
                Some(row) if row.expires_unix_ms <= now_unix_ms => {
                    HistoryRead::Expired(ExpiryCandidate {
                        invocation: row.fact.invocation,
                        content_sha256: row.content_sha256,
                        expires_unix_ms: row.expires_unix_ms,
                    })
                }
                Some(row) => HistoryRead::Present(Box::new(row)),
            })
        })
        .await
    }

    /// Acquire one exact original native test, then publish and independently read it.
    #[cfg(feature = "host-contracts")]
    pub async fn publish_original(
        &self,
        reader: &Self,
        source: &pse_host::invocation::InvocationStore,
        invocation: &pse_host::envelope::CliInvocationId,
        expires_unix_ms: i64,
        deadline: Instant,
    ) -> Result<Published, HistoryError> {
        let facts = pse_host::test_workflow::native_test_history(source, invocation)
            .map_err(|error| HistoryError::Original(Box::new(error)))?
            .try_into()?;
        let publication = HistoryPublication::new(facts, expires_unix_ms)?;
        self.publish(reader, &publication, deadline).await
    }

    #[cfg(any(test, feature = "host-contracts"))]
    async fn publish(
        &self,
        reader: &Self,
        publication: &HistoryPublication,
        deadline: Instant,
    ) -> Result<Published, HistoryError> {
        self.require_writer_reader(reader)?;
        let invocation = publication.fact.invocation.as_str();
        publication.verify(invocation)?;
        transport::operation_scope(deadline, async {
            let before = reader.exact(invocation).await?;
            if before.retirement.is_some() {
                return Err(HistoryError::RetiredIdentity);
            }
            if let Some(row) = before.publication {
                return compare(&row, publication, Published::Existing);
            }
            let acknowledgement = transport::protected_query("developer_history::publish", || {
                Ok(self
                    .query(CREATE_EXACT)?
                    .bind(("issuer", self.scope.issuer.as_str()))
                    .bind(("schema_sha256", self.scope.schema_sha256.as_str()))
                    .bind(("invocation", invocation))
                    .bind(("maximum_lifetime_ms", PILOT_MAX_LIFETIME_MS))
                    .bind(("row", publication.clone())))
            })
            .await;
            #[cfg(test)]
            let acknowledgement = self.fault_ack(reader, acknowledgement).await;
            let readback = match reader.exact(invocation).await {
                Ok(observed) if observed.retirement.is_some() => {
                    return Err(HistoryError::RetiredIdentity);
                }
                Ok(observed) => observed.publication,
                Err(readback) => {
                    return Err(unsettled("publication", invocation, acknowledgement, Some(readback)));
                }
            };
            match readback {
                Some(row) => {
                    let status = match acknowledgement {
                        Ok(_) => Published::Created,
                        Err(error) if error.effect_is_unknown() => Published::Reconciled,
                        Err(_) => Published::Existing,
                    };
                    compare(&row, publication, status)
                }
                None => match acknowledgement {
                    Err(error) if !error.effect_is_unknown() => Err(error.into()),
                    uncertain => Err(unsettled("publication", invocation, uncertain, None)),
                },
            }
        })
        .await
    }

    /// Preview expiry using the pilot's indexed lifetime field.
    pub async fn preview_expired(
        &self,
        cutoff_unix_ms: i64,
        limit: usize,
        deadline: Instant,
    ) -> Result<Vec<ExpiryCandidate>, HistoryError> {
        if limit == 0 || limit > MAX_PREVIEW_ROWS {
            return Err(HistoryError::Invalid("expiry preview row limit"));
        }
        transport::operation_scope(deadline, async {
            let mut response = transport::bounded_query(
                self.query(READ_EXPIRY)?
                    .bind(("cutoff", cutoff_unix_ms))
                    .bind(("limit", limit as i64)),
            )
            .await?;
            let reply = response
                .take::<Option<ExpiryRead>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?;
            self.scope.verify(reply.deployment.as_ref())?;
            if reply.candidates.len() > limit
                || reply.candidates.iter().any(|row| {
                    !uuid(&row.invocation)
                        || !lower_hex(&row.content_sha256, 64)
                        || row.expires_unix_ms > cutoff_unix_ms
                })
            {
                return Err(HistoryError::Invalid("expiry preview response"));
            }
            Ok(reply.candidates)
        })
        .await
    }

    fn require_writer_reader(&self, reader: &Self) -> Result<(), HistoryError> {
        if self.role == HistoryRole::Writer
            && reader.role == HistoryRole::Reader
            && self.scope == reader.scope
        {
            Ok(())
        } else {
            Err(HistoryError::WrongDeployment)
        }
    }

    /// Close admission and drain the actual connection through its transport owner.
    pub async fn disconnect(&self) -> Result<(), HistoryError> {
        let client = self.client.clone();
        self.access
            .disconnect(move || async move { client.disconnect().await.map_err(Into::into) })
            .await?;
        Ok(())
    }
}

#[cfg(any(test, feature = "host-contracts"))]
fn compare(
    original: &HistoryPublication,
    requested: &HistoryPublication,
    matching: Published,
) -> Result<Published, HistoryError> {
    if original == requested {
        Ok(matching)
    } else {
        Err(HistoryError::ReusedIdentity)
    }
}

fn unsettled(
    operation: &'static str,
    invocation: &str,
    acknowledgement: Result<transport::client::IndexedResults, CanonicalError>,
    readback: Option<HistoryError>,
) -> HistoryError {
    HistoryError::Unsettled {
        operation,
        invocation: invocation.into(),
        source: Box::new(acknowledgement.err().unwrap_or(CanonicalError::IncompleteResponse)),
        readback: readback.map(Box::new),
    }
}

#[cfg(test)]
#[path = "developer_history/tests.rs"]
mod tests;
