// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared bounded request clocks and physical lifetimes for independently owned native clients.
//! These operations accept a caller's future; they expose no canonical store or raw client.
use crate::canonical::CanonicalError;
#[path = "canonical_client.rs"]
pub mod client;
use std::{pin::Pin, time::Duration};
#[path = "transport_lifetime.rs"]
pub mod lifetime;
#[path = "transport_retry.rs"]
pub(crate) mod retry;
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
pub fn operation_scope<F: Future>(
    deadline: std::time::Instant,
    future: F,
) -> impl Future<Output = F::Output> {
    OperationScope {
        offered: tokio::time::Instant::from_std(deadline),
        selected: None,
        future: Some(future),
    }
}

pin_project_lite::pin_project! {
    // One inline child, with no async argument/await-state duplication or heap
    // allocation. Select the enclosing minimum when polled, not when constructed.
    struct OperationScope<F> {
        offered: tokio::time::Instant,
        selected: Option<tokio::time::Instant>,
        #[pin]
        future: Option<F>,
    }

    impl<F> PinnedDrop for OperationScope<F> {
        fn drop(this: Pin<&mut Self>) {
            let mut this = this.project();
            if this.future.is_some() && let Some(deadline) = *this.selected {
                OPERATION_DEADLINE.sync_scope(deadline, || this.future.set(None));
            }
            // Before the first poll, the old async scope had not entered its
            // deadline. Leave that child to ordinary ambient destruction.
        }
    }
}

impl<F: Future> Future for OperationScope<F> {
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        let deadline = (*this.selected).unwrap_or(*this.offered);
        let deadline = OPERATION_DEADLINE
            .try_with(|enclosing| (*enclosing).min(deadline))
            .unwrap_or(deadline);
        *this.selected = Some(deadline);
        OPERATION_DEADLINE.sync_scope(deadline, || {
            let Some(future) = this.future.as_mut().as_pin_mut() else {
                return std::task::Poll::Pending;
            };
            let result = future.poll(cx);
            if result.is_ready() {
                // Tokio's TaskLocalFuture also destroys a completed child while
                // its task-local context is still entered.
                this.future.set(None);
            }
            result
        })
    }
}

/// Bound one native client operation, preserving its typed failure.
pub async fn request<F, T>(future: F) -> Result<T, CanonicalError>
where
    F: IntoFuture<Output = Result<T, surrealdb_types::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        future.into_future().await.map_err(Into::into)
    })
    .await
}

// Transaction failures also mark preceding statements NotExecuted. Preserve the substantive failure.
/// A single checked RPC with a transport deadline; callers retry complete decisions.
pub async fn bounded_query<F>(future: F) -> Result<client::IndexedResults, CanonicalError>
where
    F: IntoFuture<Output = Result<client::IndexedResults, surrealdb_types::Error>>,
{
    complete_response(request(future).await?)
}

pub(crate) fn complete_response(
    response: client::IndexedResults,
) -> Result<client::IndexedResults, CanonicalError> {
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
) -> Result<client::IndexedResults, CanonicalError>
where
    F: FnMut() -> Result<Q, CanonicalError>,
    Q: IntoFuture<Output = Result<client::IndexedResults, surrealdb_types::Error>>,
{
    within_clock(original_deadline(REQUEST_TIMEOUT), async {
        let result = retry::retry_conflicts(retry::RetryPolicy::Guarded, operation, || {
            let query = build();
            async move {
                let result = match query {
                    Ok(query) => bounded_query(query).await,
                    Err(error) => Err(error),
                };
                #[cfg(all(test, feature = "canonical-tests"))]
                if result
                    .as_ref()
                    .err()
                    .is_some_and(CanonicalError::is_conflict)
                {
                    crate::canonical::pacing_observation::conflict(operation);
                }
                retry::Attempt::settled(result)
            }
        })
        .await;
        if let Err(error) = &result {
            operation_failed(operation, error);
        }
        result
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

pub(crate) fn checked(
    mut response: client::IndexedResults,
) -> Result<client::IndexedResults, CanonicalError> {
    let errors = response
        .take_errors()
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let errors = errors.into_values().collect::<Vec<_>>();
    if !errors.is_empty() {
        let substantive = errors.iter().position(|error| {
            !matches!(
                error.query_details(),
                Some(
                    surrealdb_types::QueryError::NotExecuted
                        | surrealdb_types::QueryError::Cancelled
                )
            )
        });
        if let Some(error) = errors.into_iter().nth(substantive.unwrap_or(0)) {
            return Err(error.into());
        }
    }
    Ok(response)
}

#[cfg(test)]
mod operation_scope_unit {
    use super::*;
    use std::{
        pin::Pin,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        task::{Context, Poll},
    };

    type ClockObservations = Arc<Mutex<Vec<Option<tokio::time::Instant>>>>;

    struct ClockProbe {
        clocks: ClockObservations,
        ready: Arc<AtomicBool>,
        dropped: Arc<AtomicBool>,
    }
    impl Future for ClockProbe {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
            self.clocks.lock().unwrap().push(current_deadline());
            if self.ready.load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
    impl Drop for ClockProbe {
        fn drop(&mut self) {
            self.clocks.lock().unwrap().push(current_deadline());
            self.dropped.store(true, Ordering::Release);
        }
    }
    fn clock_probe(
        ready: bool,
    ) -> (
        ClockProbe,
        ClockObservations,
        Arc<AtomicBool>,
        Arc<AtomicBool>,
    ) {
        let clocks = Arc::new(Mutex::new(Vec::new()));
        let ready = Arc::new(AtomicBool::new(ready));
        let dropped = Arc::new(AtomicBool::new(false));
        (
            ClockProbe {
                clocks: clocks.clone(),
                ready: ready.clone(),
                dropped: dropped.clone(),
            },
            clocks,
            ready,
            dropped,
        )
    }

    #[tokio::test]
    async fn operation_scope_selects_parent_on_first_poll_and_drops_ready_child_in_scope() {
        let parent = std::time::Instant::now() + Duration::from_secs(1);
        let expected = tokio::time::Instant::from_std(parent);
        let (probe, clocks, _, dropped) = clock_probe(true);
        // The child's construction has no parent. Its first poll must inherit one.
        let child = crate::canonical::operation_scope(parent + Duration::from_secs(30), probe);
        operation_scope(parent, child).await;
        assert_eq!(
            *clocks.lock().unwrap(),
            vec![Some(expected), Some(expected)]
        );
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(current_deadline(), None);
    }

    #[tokio::test]
    async fn operation_scope_keeps_expired_minimum_across_repoll_and_parent_departure() {
        let parent = std::time::Instant::now() + Duration::from_millis(10);
        let expected = tokio::time::Instant::from_std(parent);
        let earlier = parent - Duration::from_millis(1);
        let (probe, clocks, ready, dropped) = clock_probe(false);
        let mut child = Box::pin(operation_scope(parent + Duration::from_secs(30), probe));
        operation_scope(
            parent,
            std::future::poll_fn(|cx| {
                assert!(child.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            }),
        )
        .await;
        tokio::time::sleep_until(expected + Duration::from_millis(1)).await;
        assert!(
            !dropped.load(Ordering::Acquire),
            "expiry carries the owner; it does not drop it"
        );
        std::future::poll_fn(|cx| {
            assert!(child.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        operation_scope(
            earlier,
            std::future::poll_fn(|cx| {
                assert!(child.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            }),
        )
        .await;
        ready.store(true, Ordering::Release);
        child.as_mut().await;
        let earlier = tokio::time::Instant::from_std(earlier);
        assert_eq!(
            *clocks.lock().unwrap(),
            vec![
                Some(expected),
                Some(expected),
                Some(earlier),
                Some(earlier),
                Some(earlier)
            ]
        );
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(current_deadline(), None);
    }

    #[tokio::test]
    async fn operation_scope_cancel_drop_restores_original_clock_and_unpolled_drop_is_ambient() {
        let parent = std::time::Instant::now() + Duration::from_secs(1);
        let expected = tokio::time::Instant::from_std(parent);
        let (probe, clocks, _, dropped) = clock_probe(false);
        let mut child = Box::pin(operation_scope(parent + Duration::from_secs(30), probe));
        operation_scope(
            parent,
            std::future::poll_fn(|cx| {
                assert!(child.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            }),
        )
        .await;
        drop(child);
        assert_eq!(
            *clocks.lock().unwrap(),
            vec![Some(expected), Some(expected)]
        );
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(current_deadline(), None);

        let (probe, clocks, _, dropped) = clock_probe(false);
        drop(operation_scope(parent, probe));
        assert_eq!(*clocks.lock().unwrap(), vec![None]);
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(current_deadline(), None);
    }

    #[test]
    fn operation_scope_retains_one_inline_child_without_async_state_duplication() {
        let payload = [0_u8; 64 * 1024];
        let child = async move {
            std::future::pending::<()>().await;
            std::hint::black_box(payload)
        };
        let child_bytes = size_of_val(&child);
        let scoped = crate::canonical::operation_scope(std::time::Instant::now(), child);
        assert!(size_of_val(&scoped) <= child_bytes + 4 * size_of::<tokio::time::Instant>());
    }
}
