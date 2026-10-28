// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed bounded native protocol adapter. Logical-operation clocks include local queues,
//! definite-conflict retries and complete transport response processing.

use crate::transport::client::{Client, RequestContext, ResponseAllocation, RetainedRequest};
pub(crate) use crate::transport::{operation_scope, original_deadline, within_clock};
use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use surrealdb_types::{Object, SurrealValue};

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "canonical_lifetime_peer_tests.rs"]
mod lifetime_tests;
#[cfg(test)]
#[path = "canonical_transport_tests.rs"]
mod tests;

tokio::task_local! {
    static RESPONSE_OWNER: ResponseAllocation;
    static RETAINED_REQUEST: RetainedRequest;
    static SELECTION_CUSTODY: Vec<super::ProtectedSelection>;
    static METADATA_RESPONSE: super::MetadataResponse;
}

/// Retain actual prepaid allocations; composition grants no new allowance.
pub(crate) async fn response_ownership_scope<F: Future>(
    owner: impl Into<ResponseAllocation>,
    future: F,
) -> F::Output {
    let current = owner.into();
    let owner = match RESPONSE_OWNER.try_with(Clone::clone) {
        Ok(enclosing) => ResponseAllocation::joined(enclosing, current),
        Err(_) => current,
    };
    RESPONSE_OWNER.scope(owner, future).await
}

/// Carry issued non-byte read/release owners across every nested native request.
pub(crate) async fn request_custody_scope<F: Future>(
    owner: impl Into<RetainedRequest>,
    future: F,
) -> F::Output {
    let owner = owner.into();
    let owner = match RETAINED_REQUEST.try_with(Clone::clone) {
        Ok(enclosing) => RetainedRequest::combine(enclosing, owner),
        Err(_) => owner,
    };
    RETAINED_REQUEST.scope(owner, future).await
}

/// Keep existing protected acquisition custody through every nested native RPC.
pub(crate) async fn selection_custody_scope<F: Future>(
    selection: super::ProtectedSelection,
    future: F,
) -> F::Output {
    let mut selections = SELECTION_CUSTODY.try_with(Clone::clone).unwrap_or_default();
    selections.push(selection);
    SELECTION_CUSTODY.scope(selections, future).await
}

/// Cleanup may belong to a charged read or a pure source selection. Reuse the
/// existing actual response-owner scope only when a continuing allocation exists.
pub(crate) async fn optional_response_ownership_scope<F: Future>(
    owner: Option<ResponseAllocation>,
    future: F,
) -> F::Output {
    match owner {
        Some(owner) => {
            response_ownership_scope(
                owner,
                metadata_scope(super::MetadataResponse::Protection, future),
            )
            .await
        }
        None => future.await,
    }
}

/// Every nested metadata RPC observes the endpoint's finite response layout,
/// including source/protection reads reached by a selection operation.
pub(crate) async fn metadata_scope<F: Future>(
    profile: super::MetadataResponse,
    future: F,
) -> F::Output {
    METADATA_RESPONSE.scope(profile, future).await
}

/// Capture before spawning: Tokio tasks do not inherit the submitting scope.
pub(crate) fn current_response_owner() -> Option<ResponseAllocation> {
    RESPONSE_OWNER.try_with(Clone::clone).ok()
}

pub(crate) fn inherit_response_ownership<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let owner = RESPONSE_OWNER.try_with(Clone::clone).ok();
    async move {
        match owner {
            Some(owner) => RESPONSE_OWNER.scope(owner, future).await,
            None => future.await,
        }
    }
}

/// Same-host clock used by fixture owners across the Rust/Python boundary.
pub(crate) fn monotonic_nanoseconds() -> Result<u64, super::CanonicalError> {
    #[cfg(target_os = "linux")]
    {
        let clock = rustix::time::clock_gettime_dynamic(rustix::time::DynamicClockId::Known(
            rustix::time::ClockId::Monotonic,
        ))
        .map_err(|error| super::CanonicalError::Configuration(error.to_string()))?;
        u64::try_from(clock.tv_sec)
            .ok()
            .and_then(|secs| secs.checked_mul(1_000_000_000))
            .and_then(|secs| {
                u64::try_from(clock.tv_nsec)
                    .ok()
                    .and_then(|ns| secs.checked_add(ns))
            })
            .ok_or_else(|| {
                super::CanonicalError::Configuration("fixture monotonic clock overflow".into())
            })
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(super::CanonicalError::Configuration(
            "fixture same-host deadlines require the Linux monotonic clock".into(),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct CanonicalClient {
    inner: Arc<ClientState>,
    pub(crate) access: super::lifetime::ConnectionAccess,
}
impl std::ops::Deref for CanonicalClient {
    type Target = ClientState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) struct ClientState {
    client: Arc<Client>,
    budget: Duration,
    namespace: String,
    database: String,
    selection: tokio::sync::Mutex<(bool, bool)>,
    owner: crate::transport::client::Root,
    viewer: crate::transport::client::Root,
    blocked: AtomicBool,
}

/// Cancellation drops this guard before a late owner acknowledgment can turn an
/// uncertain authorization transition into a reusable context.
struct SelectionTransition<'a>(&'a AtomicBool);
impl SelectionTransition<'_> {
    fn acknowledged(self) {
        self.0.store(false, Ordering::Release);
    }
}

impl std::fmt::Debug for CanonicalClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalClient")
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}

impl CanonicalClient {
    pub(crate) fn new(client: Client, budget: Duration, options: &super::CanonicalOptions) -> Self {
        Self {
            access: super::lifetime::ConnectionAccess::default(),
            inner: Arc::new(ClientState {
                client: Arc::new(client),
                budget,
                namespace: options.namespace.clone(),
                database: options.database.clone(),
                selection: tokio::sync::Mutex::new((false, false)),
                owner: crate::transport::client::Root {
                    username: options.username.clone(),
                    password: options.password.clone(),
                },
                viewer: crate::transport::client::Root {
                    username: options.selection_username.clone(),
                    password: options.selection_password.clone(),
                },
                blocked: AtomicBool::new(false),
            }),
        }
    }

    pub(crate) fn with_access(&self, access: super::lifetime::ConnectionAccess) -> Self {
        Self {
            inner: self.inner.clone(),
            access,
        }
    }
    pub(crate) fn same_driver(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// VIEWER cannot implicitly define catalog entries during USE. Probe one
    /// record under that account, then restore the owner for explicit writes.
    /// Acknowledged setup replays in this same authorization order. A registered
    /// active context cannot be reclaimed while this handle borrows it.
    pub(crate) async fn select_existing(&self) -> Result<bool, super::CanonicalError> {
        let activity = self.access.admit()?;
        activity
            .scope(within_clock(original_deadline(self.budget), async {
                let mut selection = self.selection.lock().await;
                if self.blocked.load(Ordering::Acquire) {
                    return Err(super::CanonicalError::Configuration(
                        "canonical selection authorization is uncertain; reconnect the context"
                            .into(),
                    ));
                }
                if selection.1 {
                    return Ok(true);
                }
                self.blocked.store(true, Ordering::Release);
                let transition = SelectionTransition(&self.blocked);
                let context = || {
                    RequestContext::control(original_deadline(self.budget).into_std())
                        .with_lifetime_owner(activity.clone())
                };
                self.client
                    .signin(self.viewer.clone())
                    .request_context(context())
                    .await?;
                self.client
                    .use_ns(&self.namespace)
                    .use_db(&self.database)
                    .request_context(context())
                    .await?;
                let observed = self
                    .client
                    .select::<Option<Object>>(("canonical_interpretations", "current"))
                    .request_context(context())
                    .await;
                let next = match observed {
                    Ok(_) => (true, true),
                    Err(error) => match error.not_found_details() {
                        Some(surrealdb_types::NotFoundError::Namespace { name })
                            if name == &self.namespace =>
                        {
                            (false, false)
                        }
                        Some(surrealdb_types::NotFoundError::Database { name })
                            if name == &self.database =>
                        {
                            (true, false)
                        }
                        Some(surrealdb_types::NotFoundError::Table { name })
                            if name == "canonical_interpretations" =>
                        {
                            (true, true)
                        }
                        _ => return Err(error.into()),
                    },
                };
                self.client
                    .signin(self.owner.clone())
                    .request_context(
                        RequestContext::control(original_deadline(self.budget).into_std())
                            .with_lifetime_owner(activity.clone()),
                    )
                    .await?;
                *selection = next;
                transition.acknowledged();
                Ok(next.1)
            }))
            .await
    }

    /// Explicit creation starts on a fresh, unselected administrative session.
    /// A failed read-only probe must never become the catalog context for DDL.
    pub(crate) async fn provision(&self) -> Result<(), super::CanonicalError> {
        let activity = self.access.admit()?;
        activity
            .scope(within_clock(original_deadline(self.budget), async {
                let context = || {
                    RequestContext::control(original_deadline(self.budget).into_std())
                        .with_lifetime_owner(activity.clone())
                };
                self.client
                    .signin(self.owner.clone())
                    .request_context(context())
                    .await?;
                // 3.3 plans even built-in clock calls at database context level.
                // Catalog creation precedes that context, so it cannot carry the
                // ordinary in-database UTC fence. Submitted DDL remains uncertain
                // after caller expiry; only acknowledged completion admits selection.
                super::complete_response(
                    super::bounded_query(self.control_query(format!(
                        "DEFINE NAMESPACE IF NOT EXISTS `{}`;",
                        self.namespace,
                    )))
                    .await?,
                )?;
                self.client
                    .use_ns(&self.namespace)
                    .request_context(context())
                    .await?;
                super::complete_response(
                    super::bounded_query(self.control_query(format!(
                        "DEFINE DATABASE IF NOT EXISTS `{}`;",
                        self.database,
                    )))
                    .await?,
                )?;
                Ok(())
            }))
            .await
    }

    pub(crate) fn query<'r>(
        &'r self,
        query: impl Into<Cow<'r, str>>,
    ) -> crate::transport::client::Query<'r> {
        self.query_class(query, false)
    }
    fn query_class<'r>(
        &'r self,
        query: impl Into<Cow<'r, str>>,
        control: bool,
    ) -> crate::transport::client::Query<'r> {
        let deadline = original_deadline(self.budget);
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let utc = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let expiry = i64::try_from(utc.saturating_add(remaining).as_micros()).unwrap_or(i64::MAX);
        let mut clock = Object::new();
        clock.insert("pse_rpc_expires_at", expiry.into_value());
        clock.insert(
            "pse_rpc_timeout",
            surrealdb_types::Duration::from_std(remaining).into_value(),
        );
        let query = self
            .client
            .query(query)
            .request_context(self.context_at(control, deadline.into_std()))
            .bind_guarded(clock);
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let context = pse_diagnostics::observation::current();
        if let Some(context) = &context {
            context.count("canonical.query.builders", "builders", 1);
        }
        QUERY_METRICS.add(CanonicalQueryEvent::Builder, 1);
        let query = query.observe_statistics(Arc::new(CanonicalStatistics {
            context,
            metrics: &QUERY_METRICS,
            invocation: NEXT.fetch_add(1, Ordering::Relaxed),
        }));
        match RESPONSE_OWNER.try_with(Clone::clone) {
            Ok(owner) => query.payload_owner(owner),
            Err(_) => query,
        }
    }

    /// Endpoint-shaped response admission, before the SDK expands wire values.
    pub(crate) fn metadata_query<'r>(
        &'r self,
        profile: super::MetadataResponse,
        query: impl Into<Cow<'r, str>>,
    ) -> crate::transport::client::Query<'r> {
        self.query(query).response_limits(profile.limits())
    }

    pub(crate) fn control_query<'r>(
        &'r self,
        query: impl Into<Cow<'r, str>>,
    ) -> crate::transport::client::Query<'r> {
        self.query_class(query, true)
    }

    pub(crate) fn select<O: SurrealValue>(
        &self,
        resource: (&str, &str),
    ) -> crate::transport::client::Select<O>
    where
        usize: crate::transport::client::QueryResult<O>,
    {
        let context = self.context(false);
        let context = match RESPONSE_OWNER.try_with(Clone::clone) {
            Ok(owner) => owner.apply(context),
            Err(_) => context,
        };
        self.client.select(resource).request_context(context)
    }

    pub(crate) fn script<T>(
        &self,
        script: crate::canonical_script::PreparedScript<T>,
    ) -> crate::transport::client::Query<'static> {
        let (sql, bindings, selection) = script.into_parts();
        let query = self.query(sql).bind_guarded(bindings).into_owned();
        match selection {
            Some(selection) => query.retain_selection(selection),
            None => query,
        }
    }
    pub(crate) fn metadata_script<T>(
        &self,
        profile: super::MetadataResponse,
        script: crate::canonical_script::PreparedScript<T>,
    ) -> crate::transport::client::Query<'static> {
        self.script(script).response_limits(profile.limits())
    }

    fn context(&self, control: bool) -> RequestContext {
        self.context_at(control, original_deadline(self.budget).into_std())
    }
    fn context_at(&self, control: bool, deadline: std::time::Instant) -> RequestContext {
        let context = if control {
            RequestContext::control(deadline)
        } else {
            RequestContext::new(deadline)
        };
        let context = match METADATA_RESPONSE.try_with(|profile| profile.limits()) {
            Ok(limits) => context.with_response_limits(limits),
            Err(_) => context,
        };
        if self.blocked.load(Ordering::Acquire) {
            context.cancel();
        }
        let context = match RETAINED_REQUEST.try_with(Clone::clone) {
            Ok(owner) => context.with_retained_request(owner),
            Err(_) => context,
        };
        let context = SELECTION_CUSTODY
            .try_with(Clone::clone)
            .unwrap_or_default()
            .into_iter()
            .fold(context, RequestContext::with_selection_custody);
        match self.access.admit() {
            Ok(activity) => context.with_lifetime_owner(activity),
            Err(_) => {
                context.cancel();
                context
            }
        }
    }

    /// Restricted terminal cleanup owned by PhysicalLifetime after admission and
    /// continuing work have drained. This is not a general bypass for queries.
    pub(crate) async fn drain_file_owner(
        &self,
        expected: Object,
    ) -> Result<(), super::CanonicalError> {
        let deadline = original_deadline(self.budget);
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .saturating_add(deadline.saturating_duration_since(tokio::time::Instant::now()));
        let expiry =
            i64::try_from(expiry.as_micros()).map_err(|_| super::CanonicalError::PayloadLimit)?;
        let response = self
            .client
            .query(
                crate::canonical_script::GuardedScript::new(
                    crate::canonical_script::Envelope::DeadlineOnly,
                )
                .returning::<surrealdb_types::Value>(
                    "",
                    "fn::pse_files_v1::drain_owner($expiry,$expected)",
                )?
                .into_parts()
                .0,
            )
            .bind(("pse_rpc_expires_at", expiry))
            .bind(("expiry", expiry))
            .bind(("expected", expected))
            .request_context(RequestContext::control(deadline.into_std()))
            .await?;
        response.check()?;
        Ok(())
    }
    pub(crate) async fn disconnect(&self) -> Result<(), surrealdb_types::Error> {
        self.client.disconnect().await
    }
}

/// One actual SDK response sample. The compact wire representation fits the
/// default observation text bound even at the maximum integer values. Statement
/// index and server time belong to the same invocation atomically; history may
/// coalesce intermediate responses and reports that loss explicitly.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CanonicalQuerySample {
    /// The request failed without a statement response; no server time is known.
    #[serde(rename = "r")]
    RequestFailure {
        /// Process-local diagnostic invocation, unrelated to scientific identity.
        #[serde(rename = "q")]
        invocation: u64,
    },
    /// A statement in an actual response, copied before caller consumption.
    #[serde(rename = "s")]
    Statement {
        /// Process-local diagnostic invocation, unrelated to scientific identity.
        #[serde(rename = "q")]
        invocation: u64,
        /// Actual zero-based statement index in that response.
        #[serde(rename = "s")]
        statement: usize,
        /// Exact server-reported duration in nanoseconds, when supplied. This
        /// does not locate a local active interval or establish total server CPU.
        #[serde(rename = "ns")]
        server_nanoseconds: Option<u128>,
        /// Actual statement result status, independent of the request transport.
        #[serde(rename = "ok")]
        succeeded: bool,
    },
}

/// Actual-invocation statistics, copied before any caller consumes or checks the response.
#[derive(Debug)]
struct CanonicalStatistics {
    context: Option<pse_diagnostics::observation::ObservationContext>,
    metrics: &'static CanonicalMetricState,
    invocation: u64,
}
impl CanonicalStatistics {
    fn sample(&self, sample: CanonicalQuerySample) {
        let Some(context) = &self.context else {
            return;
        };
        context.source_ref(
            "canonical.server.statistics",
            "SurrealDB actual response; server wall-clock sample; no local active interval",
        );
        match serde_json::to_string(&sample) {
            Ok(sample) => context.text("canonical.query.latest_sample", &sample),
            Err(_) => context.unavailable(
                "canonical.query.latest_sample",
                "server sample serialization unavailable",
            ),
        }
    }
}
impl crate::transport::client::QueryStatisticsObserver for CanonicalStatistics {
    fn request_failed(&self) {
        self.metrics.add(CanonicalQueryEvent::RequestFailed, 1);
        if let Some(context) = &self.context {
            context.count("canonical.requests.failed", "requests", 1);
        }
        self.sample(CanonicalQuerySample::RequestFailure {
            invocation: self.invocation,
        });
    }
    fn observe(&self, statement: usize, stats: crate::transport::client::Stats, succeeded: bool) {
        self.metrics.add(CanonicalQueryEvent::Statement, 1);
        self.metrics.add(
            if succeeded {
                CanonicalQueryEvent::StatementSucceeded
            } else {
                CanonicalQueryEvent::StatementFailed
            },
            1,
        );
        self.metrics.add(
            if stats.execution_time.is_some() {
                CanonicalQueryEvent::StatementTimed
            } else {
                CanonicalQueryEvent::StatementUntimed
            },
            1,
        );
        if let Some(duration) = stats.execution_time {
            let nanos = duration.as_nanos();
            let value = u64::try_from(nanos).unwrap_or(u64::MAX);
            if nanos > u128::from(u64::MAX) {
                self.metrics.saturated.store(true, Ordering::Relaxed);
            }
            self.metrics.add_duration(value);
        }
        if let Some(context) = &self.context {
            context.count("canonical.statements.received", "statements", 1);
            context.count(
                if succeeded {
                    "canonical.statements.succeeded"
                } else {
                    "canonical.statements.failed"
                },
                "statements",
                1,
            );
            context.count(
                if stats.execution_time.is_some() {
                    "canonical.statements.timed"
                } else {
                    "canonical.statements.untimed"
                },
                "statements",
                1,
            );
        }
        self.sample(CanonicalQuerySample::Statement {
            invocation: self.invocation,
            statement,
            server_nanoseconds: stats.execution_time.map(|duration| duration.as_nanos()),
            succeeded,
        });
    }
}

/// Closed events of the actual canonical query/statistics owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalQueryEvent {
    /// `builder` owner population.
    Builder,
    /// `request_failed` owner population.
    RequestFailed,
    /// `statement` owner population.
    Statement,
    /// `statement_succeeded` owner population.
    StatementSucceeded,
    /// `statement_failed` owner population.
    StatementFailed,
    /// `statement_timed` owner population.
    StatementTimed,
    /// `statement_untimed` owner population.
    StatementUntimed,
}
impl CanonicalQueryEvent {
    /// Complete finite event domain, independent of observation order.
    pub const ALL: [Self; 7] = [
        Self::Builder,
        Self::RequestFailed,
        Self::Statement,
        Self::StatementSucceeded,
        Self::StatementFailed,
        Self::StatementTimed,
        Self::StatementUntimed,
    ];
    /// Stable telemetry attribute value issued by this owner.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Builder => "builder",
            Self::RequestFailed => "request_failed",
            Self::Statement => "statement",
            Self::StatementSucceeded => "statement_succeeded",
            Self::StatementFailed => "statement_failed",
            Self::StatementTimed => "statement_timed",
            Self::StatementUntimed => "statement_untimed",
        }
    }
    /// Select the event's population from a copied source image.
    pub fn count(self, sample: &CanonicalQueryMetrics) -> u64 {
        match self {
            Self::Builder => sample.builders,
            Self::RequestFailed => sample.request_failures,
            Self::Statement => sample.statements,
            Self::StatementSucceeded => sample.statements_succeeded,
            Self::StatementFailed => sample.statements_failed,
            Self::StatementTimed => sample.statements_timed,
            Self::StatementUntimed => sample.statements_untimed,
        }
    }
}

#[derive(Debug)]
struct CanonicalMetricState {
    counts: [std::sync::atomic::AtomicU64; CanonicalQueryEvent::ALL.len()],
    duration: std::sync::atomic::AtomicU64,
    started: std::sync::OnceLock<u128>,
    saturated: AtomicBool,
}
impl CanonicalMetricState {
    const fn new() -> Self {
        Self {
            counts: [const { std::sync::atomic::AtomicU64::new(0) };
                CanonicalQueryEvent::ALL.len()],
            duration: std::sync::atomic::AtomicU64::new(0),
            started: std::sync::OnceLock::new(),
            saturated: AtomicBool::new(false),
        }
    }
    fn add(&self, event: CanonicalQueryEvent, value: u64) {
        self.add_to(&self.counts[event as usize], value);
    }
    fn add_duration(&self, value: u64) {
        self.add_to(&self.duration, value);
    }
    fn add_to(&self, counter: &std::sync::atomic::AtomicU64, value: u64) {
        self.started.get_or_init(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros()
        });
        let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            let next = current.checked_add(value);
            if next.is_none() {
                self.saturated.store(true, Ordering::Relaxed);
            }
            Some(next.unwrap_or(u64::MAX))
        });
    }
    fn snapshot(&self) -> CanonicalQueryMetrics {
        let read = |event: CanonicalQueryEvent| self.counts[event as usize].load(Ordering::Relaxed);
        CanonicalQueryMetrics {
            source_started_unix_micros: self.started.get().copied(),
            builders: read(CanonicalQueryEvent::Builder),
            request_failures: read(CanonicalQueryEvent::RequestFailed),
            statements: read(CanonicalQueryEvent::Statement),
            statements_succeeded: read(CanonicalQueryEvent::StatementSucceeded),
            statements_failed: read(CanonicalQueryEvent::StatementFailed),
            statements_timed: read(CanonicalQueryEvent::StatementTimed),
            statements_untimed: read(CanonicalQueryEvent::StatementUntimed),
            server_duration_nanos: self.duration.load(Ordering::Relaxed),
            saturated: self.saturated.load(Ordering::Relaxed),
        }
    }
}
static QUERY_METRICS: CanonicalMetricState = CanonicalMetricState::new();
/// Process-cumulative actual query builder / SDK response populations. Not every SDK
/// select/control API uses this query adapter; no server-wide coverage is claimed.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalQueryMetrics {
    /// First observed query-owner event; absent before any owner activity.
    pub source_started_unix_micros: Option<u128>,
    /// Query builders constructed; never interpreted as dispatched requests.
    pub builders: u64,
    /// SDK observer request failures without statement responses.
    pub request_failures: u64,
    /// Actual statements received before caller consumption, including failures.
    pub statements: u64,
    /// Successful statement results.
    pub statements_succeeded: u64,
    /// Failed statement results.
    pub statements_failed: u64,
    /// Statements carrying server execution time.
    pub statements_timed: u64,
    /// Statements lacking server execution time; absent is not zero duration.
    pub statements_untimed: u64,
    /// Sum of supplied server statement wall times, in nanoseconds; not CPU time.
    pub server_duration_nanos: u64,
    /// A count or duration saturated; exact increases are unavailable after this point.
    pub saturated: bool,
}
/// Cheap copied source state; independent reads are not an atomic response census.
pub fn query_metrics() -> CanonicalQueryMetrics {
    QUERY_METRICS.snapshot()
}
