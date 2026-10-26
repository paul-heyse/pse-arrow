// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared bounded request clocks and physical lifetimes for independently owned native clients.
//! These operations accept a caller's future; they expose no canonical store or raw client.
use crate::canonical::CanonicalError;
use std::time::Duration;
#[path = "transport_lifetime.rs"]
pub mod lifetime;
/// Existing per-request ceiling; an enclosing operation may only shorten it.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
// Existing finite contention window for sixteen clients sharing a retention guard.
pub(crate) const GUARDED_ATTEMPTS: u32 = 32;
tokio::task_local! { static OPERATION_DEADLINE: tokio::time::Instant; }
/// Observe the enclosing operation clock without extending it.
pub fn current_deadline() -> Option<tokio::time::Instant> {
    OPERATION_DEADLINE.try_with(|deadline| *deadline).ok()
}

/// Select the earlier of the enclosing deadline and this request allowance.
pub fn original_deadline(budget: Duration) -> tokio::time::Instant {
    let offered = tokio::time::Instant::now() + budget;
    OPERATION_DEADLINE
        .try_with(|deadline| (*deadline).min(offered))
        .unwrap_or(offered)
}

/// Enforce one original deadline across the supplied request and nested work.
/// # Errors
/// Timeout or the original typed request failure.
pub async fn within_clock<F, T>(
    deadline: tokio::time::Instant,
    future: F,
) -> Result<T, CanonicalError>
where
    F: Future<Output = Result<T, CanonicalError>>,
{
    let deadline = OPERATION_DEADLINE
        .try_with(|enclosing| (*enclosing).min(deadline))
        .unwrap_or(deadline);
    tokio::time::timeout_at(deadline, OPERATION_DEADLINE.scope(deadline, future))
        .await
        .map_err(|_| CanonicalError::Timeout)?
}

/// Carry a caller's original clock through nested requests without dropping its
/// enclosing effect owner. Individual requests still enforce their own bounds.
pub async fn operation_scope<F: Future>(deadline: std::time::Instant, future: F) -> F::Output {
    let deadline = tokio::time::Instant::from_std(deadline);
    let deadline = OPERATION_DEADLINE
        .try_with(|enclosing| (*enclosing).min(deadline))
        .unwrap_or(deadline);
    OPERATION_DEADLINE.scope(deadline, future).await
}

/// Bound one native client operation, preserving its typed failure.
pub async fn request<F, T>(future: F) -> Result<T, CanonicalError>
where
    F: IntoFuture<Output = Result<T, surrealdb::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        future.into_future().await.map_err(Into::into)
    })
    .await
}

// Transaction failures also mark preceding statements NotExecuted. Preserve the substantive failure.
/// A single checked RPC with a transport deadline; callers retry complete decisions.
pub async fn bounded_query<F>(future: F) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    complete_response(request(future).await?)
}

pub(crate) fn complete_response(
    response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, CanonicalError> {
    let response = checked(response)?;
    if response.num_statements() == 0 {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(response)
}

/// Rebuild a protected immutable read or idempotent lease transaction after a
/// conflict. Every attempt rechecks the live pin and reclamation guard; no
/// scientific decision or partial response is retained across attempts.
pub async fn protected_query<F, Q>(
    operation: &'static str,
    mut build: F,
) -> Result<surrealdb::IndexedResults, CanonicalError>
where
    F: FnMut() -> Result<Q, CanonicalError>,
    Q: IntoFuture<Output = Result<surrealdb::IndexedResults, surrealdb::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        let mut attempt=0;
            loop {
            let result=bounded_query(build()?).await;
            #[cfg(all(test, feature = "canonical-tests"))]
            if matches!(&result,Err(CanonicalError::Driver(error)) if matches!(error.query_details(),Some(surrealdb::types::QueryError::TransactionConflict))) { crate::canonical::pacing_observation::conflict(operation); }
            match result {
                Err(error) if error.is_capture_busy() => {conflict_backoff(4).await;}
                Err(error) if error.is_conflict() && attempt + 1 < GUARDED_ATTEMPTS =>
                {
                    conflict_backoff(attempt).await;
                    attempt+=1;
                }
                result => {
                    if let Err(error) = &result {
                        operation_failed(operation, error);
                    }
                    return result.map_err(|error| error.exhaust_conflict(operation, GUARDED_ATTEMPTS));
                }
            }
        }
    })
    .await
}

/// Attribute an escaping failure without altering its typed cause or retry authority.
/// Labels are supplied by callers; bindings and query text are never logged.
pub(crate) fn operation_failed(operation: &'static str, error: &CanonicalError) {
    tracing::warn!(operation, error = %error, "canonical operation failed");
    #[cfg(any(test, feature = "test-support"))]
    {
        #[allow(
            clippy::print_stderr,
            reason = "explicit test support captures final operation attribution without requiring a tracing subscriber"
        )]
        {
            eprintln!("canonical operation {operation} failed: {error}");
        }
    }
}

// Jitter separates definite conflict retries after the RPC releases its local turn.
// Each owner rebuilds its complete decision under the original request clock.
pub(crate) async fn conflict_backoff(attempt: u32) {
    let jitter = u64::from(uuid::Uuid::new_v4().as_bytes()[0] % 64);
    tokio::time::sleep(Duration::from_millis((10_u64 << attempt.min(4)) + jitter)).await;
}

pub(crate) fn checked(
    mut response: surrealdb::IndexedResults,
) -> Result<surrealdb::IndexedResults, CanonicalError> {
    let errors = response.take_errors().into_values().collect::<Vec<_>>();
    if !errors.is_empty() {
        let substantive = errors.iter().position(|error| {
            !matches!(
                error.query_details(),
                Some(
                    surrealdb::types::QueryError::NotExecuted
                        | surrealdb::types::QueryError::Cancelled
                )
            )
        });
        if let Some(error) = errors.into_iter().nth(substantive.unwrap_or(0)) {
            return Err(error.into());
        }
    }
    Ok(response)
}
