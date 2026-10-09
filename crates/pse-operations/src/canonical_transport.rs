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
use surrealdb::{Surreal, engine::remote::ws::Client, opt::RequestContext};

#[cfg(test)]
#[path = "canonical_transport_tests.rs"]
mod tests;

tokio::task_local! {
    static OPERATION_DEADLINE: tokio::time::Instant;
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

pub(crate) struct CanonicalClient {
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
        }
    }

    /// VIEWER cannot implicitly define catalog entries during USE. Probe one
    /// record under that account, then restore the owner for explicit writes.
    /// Acknowledged setup replays in this same authorization order. A registered
    /// active context cannot be reclaimed while this handle borrows it.
    pub(crate) async fn select_existing(&self) -> Result<bool, super::CanonicalError> {
        within_clock(original_deadline(self.budget), async {
            let mut selection = self.selection.lock().await;
            if self.blocked.load(Ordering::Acquire) {
                return Err(super::CanonicalError::Configuration(
                    "canonical selection authorization is uncertain; reconnect the context".into(),
                ));
            }
            if selection.1 {
                return Ok(true);
            }
            self.blocked.store(true, Ordering::Release);
            let transition = SelectionTransition(&self.blocked);
            let context = || RequestContext::control(original_deadline(self.budget).into_std());
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
                .request_context(RequestContext::selection_checkpoint(
                    original_deadline(self.budget).into_std(),
                    self.checkpoint,
                ))
                .await?;
            *selection = next;
            transition.acknowledged();
            Ok(next.1)
        })
        .await
    }

    /// Explicit creation starts on a fresh, unselected administrative session.
    /// A failed read-only probe must never become the catalog context for DDL.
    pub(crate) async fn provision(&self) -> Result<(), super::CanonicalError> {
        within_clock(original_deadline(self.budget), async {
            let context = || RequestContext::control(original_deadline(self.budget).into_std());
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
        })
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
        self.client
            .query(query)
            .request_context(self.context(false))
            .bind(("pse_rpc_expires_at", expiry))
            .bind((
                "pse_rpc_timeout",
                surrealdb::types::Duration::from_std(remaining),
            ))
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
        self.client
            .select(resource)
            .request_context(self.context(false))
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
        context
    }

    pub(crate) async fn disconnect(&self) -> Result<(), surrealdb::Error> {
        self.client.disconnect().await
    }
}
