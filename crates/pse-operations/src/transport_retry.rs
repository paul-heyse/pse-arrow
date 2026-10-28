// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pacing for complete, settled operation attempts. Classification belongs to
//! the effect owner: a lost acknowledgement must be reconciled before another
//! attempt can be labelled Conflict or Busy.
use crate::canonical::CanonicalError;
use std::time::Duration;

pub(crate) enum Attempt<T> {
    Done(Result<T, CanonicalError>),
    Conflict(CanonicalError),
    Busy(CanonicalError),
}

impl<T> Attempt<T> {
    /// Use only after the owning step has settled any uncertain effect.
    pub(crate) fn settled(result: Result<T, CanonicalError>) -> Self {
        match result {
            Err(error) if error.is_conflict() => Self::Conflict(error),
            Err(error) if error.is_capture_busy() => Self::Busy(error),
            result => Self::Done(result),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum RetryPolicy {
    Guarded,
    StageMutation,
    Initialization,
    Retention,
    BeginStage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pace {
    Guarded(u32),
    Initialization(u32),
    BeginStage(u32),
    Yield,
}

impl RetryPolicy {
    fn attempts(self) -> u32 {
        match self {
            Self::Guarded | Self::StageMutation => 32,
            Self::Initialization | Self::Retention | Self::BeginStage => 8,
        }
    }
    fn conflict(self, index: u32) -> Pace {
        match self {
            Self::Guarded | Self::StageMutation => Pace::Guarded(index),
            Self::Initialization => Pace::Initialization(index),
            Self::Retention => Pace::Yield,
            Self::BeginStage => Pace::BeginStage(index),
        }
    }
}

async fn pace(pace: Pace) {
    let delay = match pace {
        Pace::Guarded(index) => {
            let jitter = u64::from(uuid::Uuid::new_v4().as_bytes()[0] % 64);
            Duration::from_millis((10_u64 << index.min(4)) + jitter)
        }
        Pace::Initialization(index) => {
            let jitter = u64::from(uuid::Uuid::new_v4().as_bytes()[0] % 16);
            Duration::from_millis((1_u64 << index) + jitter)
        }
        Pace::BeginStage(index) => Duration::from_millis(10_u64 << index.min(4)),
        Pace::Yield => return tokio::task::yield_now().await,
    };
    tokio::time::sleep(delay).await;
}

/// Runs inline under the caller's original clock. No detached task, renewed
/// deadline or implicit retry of a failed RPC is introduced here.
pub(crate) async fn retry_conflicts<T, F, Q>(
    policy: RetryPolicy,
    operation: &'static str,
    step: F,
) -> Result<T, CanonicalError>
where
    F: FnMut() -> Q,
    Q: Future<Output = Attempt<T>>,
{
    retry_with_state(policy, operation, step, |mut step| {
        let pending = step();
        async move { (step, pending.await) }
    }).await
}

/// Carry a borrowed-operation builder by ownership between attempts, without
/// lending an async closure or introducing a mutex/boxed retry future.
pub(crate) async fn retry_with_state<T, S, F, Q>(
    policy: RetryPolicy,
    operation: &'static str,
    state: S,
    step: F,
) -> Result<T, CanonicalError>
where
    F: FnMut(S) -> Q,
    Q: Future<Output = (S, Attempt<T>)>,
{
    retry_with_pacing(policy, operation, state, step, pace).await
}

async fn retry_with_pacing<T, S, F, Q, P, W>(
    policy: RetryPolicy,
    operation: &'static str,
    mut state: S,
    mut step: F,
    mut pacing: P,
) -> Result<T, CanonicalError>
where
    F: FnMut(S) -> Q,
    Q: Future<Output = (S, Attempt<T>)>,
    P: FnMut(Pace) -> W,
    W: Future<Output = ()>,
{
    let mut conflicts = 0;
    loop {
        let (next, outcome) = step(state).await;
        state = next;
        match outcome {
            Attempt::Conflict(error) if error.is_conflict() => {
                conflicts += 1;
                if conflicts == policy.attempts() {
                    return Err(error.exhaust_conflict(operation, conflicts));
                }
                pacing(policy.conflict(conflicts - 1)).await;
            }
            Attempt::Busy(error)
                if error.is_capture_busy() && matches!(policy, RetryPolicy::Guarded) =>
            {
                pacing(Pace::Guarded(4)).await;
            }
            Attempt::Done(result) => return result.map_err(|error| error.exhaust_conflict(operation, policy.attempts())),
            Attempt::Conflict(error) | Attempt::Busy(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, sync::atomic::{AtomicU32, Ordering}};

    fn conflict() -> CanonicalError {
        surrealdb::Error::query("rolled back".into(), surrealdb::types::QueryError::TransactionConflict).into()
    }

    #[tokio::test]
    async fn caps_count_complete_conflict_attempts_and_pacing_is_policy_owned() {
        for (policy, expected, first) in [
            (RetryPolicy::Guarded, 32, Pace::Guarded(0)),
            (RetryPolicy::StageMutation, 32, Pace::Guarded(0)),
            (RetryPolicy::Initialization, 8, Pace::Initialization(0)),
            (RetryPolicy::Retention, 8, Pace::Yield),
            (RetryPolicy::BeginStage, 8, Pace::BeginStage(0)),
        ] {
            let attempts = AtomicU32::new(0);
            let pauses = RefCell::new(Vec::new());
            let result = retry_with_pacing::<(), _, _, _, _, _>(policy, "retry contract", (), |()| async {
                attempts.fetch_add(1, Ordering::Relaxed);
                ((), Attempt::Conflict(conflict()))
            }, |pause| { pauses.borrow_mut().push(pause); std::future::ready(()) }).await;
            assert!(matches!(result, Err(CanonicalError::ConflictExhausted { attempts, .. }) if attempts == expected));
            assert_eq!(attempts.load(Ordering::Relaxed), expected);
            assert_eq!(pauses.borrow().len(), expected as usize - 1);
            assert_eq!(pauses.borrow()[0], first);
        }
    }

    #[tokio::test]
    async fn busy_waits_preserve_the_full_conflict_allowance() {
        let calls = AtomicU32::new(0);
        let pauses = RefCell::new(Vec::new());
        let result = retry_with_pacing::<(), _, _, _, _, _>(RetryPolicy::Guarded, "mixed retry", (), |()| async {
            let call = calls.fetch_add(1, Ordering::Relaxed);
            let outcome = if call < 5 {
                Attempt::settled(Err(CanonicalError::from(surrealdb::Error::thrown(
                    "PSE_REFUSAL:study_capture_publication_busy".into(),
                ))))
            } else {
                Attempt::Conflict(conflict())
            };
            ((), outcome)
        }, |pause| { pauses.borrow_mut().push(pause); std::future::ready(()) }).await;
        assert!(matches!(result, Err(CanonicalError::ConflictExhausted { attempts: 32, .. })));
        assert_eq!(calls.load(Ordering::Relaxed), 37);
        assert_eq!(&pauses.borrow()[..5], &[Pace::Guarded(4); 5]);
        assert_eq!(pauses.borrow()[5], Pace::Guarded(0));
        assert_eq!(pauses.borrow().len(), 36);
    }

    #[tokio::test]
    async fn uncertain_or_misclassified_failures_never_gain_retry_permission() {
        let attempts = AtomicU32::new(0);
        let result = retry_with_pacing::<(), _, _, _, _, _>(RetryPolicy::Guarded, "uncertain", (), |()| async {
            attempts.fetch_add(1, Ordering::Relaxed);
            ((), Attempt::Conflict(CanonicalError::IncompleteResponse))
        }, |_| async { panic!("uncertain effect must not be paced") }).await;
        assert!(matches!(result, Err(CanonicalError::IncompleteResponse)));
        assert_eq!(attempts.load(Ordering::Relaxed), 1);
    }
}
