// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed native SDK adapter. Logical-operation clocks include local queues,
//! definite-conflict retries and complete transport response processing.

use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use surrealdb::types::Object;
use surrealdb::{Surreal, engine::remote::ws::Client, opt::RequestContext};

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "canonical_lifetime_peer_tests.rs"]
mod lifetime_tests;
#[cfg(test)]
#[path = "canonical_transport_tests.rs"]
mod tests;

tokio::task_local! {
    static OPERATION_DEADLINE: tokio::time::Instant;
    static RESPONSE_OWNER: Arc<dyn std::fmt::Debug + Send + Sync>;
}

#[derive(Debug)]
struct ResponseOwners {
    _enclosing: Arc<dyn std::fmt::Debug + Send + Sync>,
    _current: Arc<dyn std::fmt::Debug + Send + Sync>,
}

/// Carry the caller's actual response allocation into every nested native route.
pub(crate) async fn response_ownership_scope<F: Future>(
    owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    future: F,
) -> F::Output {
    let owner: Arc<dyn std::fmt::Debug + Send + Sync> = match RESPONSE_OWNER.try_with(Arc::clone) {
        Ok(enclosing) => Arc::new(ResponseOwners {
            _enclosing: enclosing,
            _current: owner,
        }),
        Err(_) => owner,
    };
    RESPONSE_OWNER.scope(owner, future).await
}

/// Capture before spawning: Tokio tasks do not inherit the submitting scope.
pub(crate) fn current_response_owner() -> Option<Arc<dyn std::fmt::Debug + Send + Sync>> { RESPONSE_OWNER.try_with(Arc::clone).ok() }

pub(crate) fn inherit_response_ownership<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let owner = RESPONSE_OWNER.try_with(Arc::clone).ok();
    async move {
        match owner {
            Some(owner) => RESPONSE_OWNER.scope(owner, future).await,
            None => future.await,
        }
    }
}

pub(crate) fn original_deadline(budget: Duration) -> tokio::time::Instant {
    let offered = tokio::time::Instant::now() + budget;
    OPERATION_DEADLINE
        .try_with(|deadline| (*deadline).min(offered))
        .unwrap_or(offered)
}

pub(crate) async fn within_clock<F, T>(
    deadline: tokio::time::Instant,
    future: F,
) -> Result<T, super::CanonicalError>
where
    F: Future<Output = Result<T, super::CanonicalError>>,
{
    let deadline = OPERATION_DEADLINE
        .try_with(|enclosing| (*enclosing).min(deadline))
        .unwrap_or(deadline);
    tokio::time::timeout_at(deadline, OPERATION_DEADLINE.scope(deadline, future))
        .await
        .map_err(|_| super::CanonicalError::Timeout)?
}

/// Carry a caller's original clock through nested requests without dropping its
/// enclosing effect owner. Individual requests still enforce their own bounds.
pub(crate) async fn operation_scope<F: Future>(
    deadline: std::time::Instant,
    future: F,
) -> F::Output {
    let deadline = tokio::time::Instant::from_std(deadline);
    let deadline = OPERATION_DEADLINE
        .try_with(|enclosing| (*enclosing).min(deadline))
        .unwrap_or(deadline);
    OPERATION_DEADLINE.scope(deadline, future).await
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
    client: Arc<Surreal<Client>>,
    budget: Duration,
    namespace: String,
    database: String,
    selection: tokio::sync::Mutex<(bool, bool)>,
    owner: surrealdb::opt::auth::Root,
    viewer: surrealdb::opt::auth::Root,
    blocked: AtomicBool,
    checkpoint: uuid::Uuid,
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
    pub(crate) fn new(
        client: Surreal<Client>,
        budget: Duration,
        options: &super::CanonicalOptions,
    ) -> Self {
        Self {
            access: super::lifetime::ConnectionAccess::default(),
            inner: Arc::new(ClientState {
                client: Arc::new(client),
                budget,
                namespace: options.namespace.clone(),
                database: options.database.clone(),
                selection: tokio::sync::Mutex::new((false, false)),
                owner: surrealdb::opt::auth::Root {
                    username: options.username.clone(),
                    password: options.password.clone(),
                },
                viewer: surrealdb::opt::auth::Root {
                    username: options.selection_username.clone(),
                    password: options.selection_password.clone(),
                },
                blocked: AtomicBool::new(false),
                checkpoint: uuid::Uuid::new_v4(),
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
                        .with_lifetime_owner(Arc::new(activity.clone()))
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
                    .select::<Option<surrealdb::types::Object>>((
                        "canonical_interpretations",
                        "current",
                    ))
                    .request_context(context())
                    .await;
                let next = match observed {
                    Ok(_) => (true, true),
                    Err(error) => match error.not_found_details() {
                        Some(surrealdb::types::NotFoundError::Namespace { name })
                            if name == &self.namespace =>
                        {
                            (false, false)
                        }
                        Some(surrealdb::types::NotFoundError::Database { name })
                            if name == &self.database =>
                        {
                            (true, false)
                        }
                        Some(surrealdb::types::NotFoundError::Table { name })
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
                        RequestContext::selection_checkpoint(
                            original_deadline(self.budget).into_std(),
                            self.checkpoint,
                        )
                        .with_lifetime_owner(Arc::new(activity.clone())),
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
                        .with_lifetime_owner(Arc::new(activity.clone()))
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
    ) -> surrealdb::method::Query<'r, Client> {
        let deadline = original_deadline(self.budget);
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let utc = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let expiry = i64::try_from(utc.saturating_add(remaining).as_micros()).unwrap_or(i64::MAX);
        let query = self
            .client
            .query(query)
            .request_context(self.context(false))
            .bind(("pse_rpc_expires_at", expiry))
            .bind((
                "pse_rpc_timeout",
                surrealdb::types::Duration::from_std(remaining),
            ));
        match RESPONSE_OWNER.try_with(Arc::clone) {
            Ok(owner) => query.payload_owner(owner),
            Err(_) => query,
        }
    }

    pub(crate) fn control_query<'r>(
        &'r self,
        query: impl Into<Cow<'r, str>>,
    ) -> surrealdb::method::Query<'r, Client> {
        self.query(query).request_context(self.context(true))
    }

    pub(crate) fn select<O>(
        &self,
        resource: impl surrealdb::opt::IntoResource<O>,
    ) -> surrealdb::method::Select<'_, Client, O> {
        let context = self.context(false);
        let context = match RESPONSE_OWNER.try_with(Arc::clone) {
            Ok(owner) => context.with_payload_owner(owner),
            Err(_) => context,
        };
        self.client.select(resource).request_context(context)
    }

    fn context(&self, control: bool) -> RequestContext {
        let deadline = original_deadline(self.budget).into_std();
        let context = if control {
            RequestContext::control(deadline)
        } else {
            RequestContext::new(deadline)
        };
        if self.blocked.load(Ordering::Acquire) {
            context.cancel();
        }
        match self.access.admit() {
            Ok(activity) => context.with_lifetime_owner(Arc::new(activity)),
            Err(_) => {
                context.cancel();
                context
            }
        }
    }

    /// Restricted terminal cleanup owned by PhysicalLifetime after admission and
    /// continuing work have drained. This is not a general bypass for queries.
    pub(crate) async fn drain_file_owner(&self,expected:Object)->Result<(),super::CanonicalError>{
        let deadline=original_deadline(self.budget);
        let expiry=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().saturating_add(deadline.saturating_duration_since(tokio::time::Instant::now()));
        let expiry=i64::try_from(expiry.as_micros()).map_err(|_|super::CanonicalError::PayloadLimit)?;
        let response=self.client.query("BEGIN; RETURN fn::pse_files_v1::drain_owner($expiry,$expected); COMMIT;").bind(("expiry",expiry)).bind(("expected",expected)).request_context(RequestContext::control(deadline.into_std())).await?;
        response.check()?;Ok(())
    }
    pub(crate) async fn disconnect(&self) -> Result<(), surrealdb::Error> {
        self.client.disconnect().await
    }
}
