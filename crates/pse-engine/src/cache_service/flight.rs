// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completion-owned bounded single flights. Native work owns its entry until actual exit.
use datafusion::common::{DataFusionError, Result};
use futures_util::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
/// Cooperative cancellation shared with a completion-owned native worker.
#[derive(Clone, Debug, Default)]
pub struct FlightCancellation {
    flag: Arc<AtomicBool>,
    notify: Arc<tokio::sync::Notify>,
}
impl FlightCancellation {
    /// Flag passed to foreign-library abort callbacks.
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }
    /// Request cancellation; ownership remains until completion.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }
    /// Wait without losing a cancellation notification.
    pub async fn cancelled(&self) {
        loop {
            let notified = self.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.flag.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }
}
/// Coordination failures preserve the original typed loader failure.
#[derive(Debug, thiserror::Error)]
pub enum FlightError<E> {
    /// Original shared failure.
    #[error("shared flight load failed")]
    Load(Arc<E>),
    /// Finite key/waiter limit reached.
    #[error("flight admission capacity")]
    Capacity,
    /// All prior waiters left and this flight is retiring.
    #[error("flight is retiring after cancellation")]
    Cancelled,
    /// Worker panicked or lost its completion channel.
    #[error("flight worker panicked")]
    Panicked,
}
impl<E: std::fmt::Debug + Send + Sync + 'static> miette::Diagnostic for FlightError<E> {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(match self {
            Self::Capacity => "runtime::resource_limit",
            Self::Cancelled => "runtime::cancelled",
            Self::Panicked => "runtime::infrastructure",
            Self::Load(_) => "runtime::infrastructure",
        }))
    }
}
impl<E> Clone for FlightError<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Load(e) => Self::Load(e.clone()),
            Self::Capacity => Self::Capacity,
            Self::Cancelled => Self::Cancelled,
            Self::Panicked => Self::Panicked,
        }
    }
}
type SharedLoad<V, E> = Shared<BoxFuture<'static, std::result::Result<Arc<V>, FlightError<E>>>>;
struct Flight<V, E> {
    result: SharedLoad<V, E>,
    cancel: FlightCancellation,
    waiters: AtomicUsize,
}
struct Waiter<V, E>(Arc<Flight<V, E>>);
impl<V, E> Drop for Waiter<V, E> {
    fn drop(&mut self) {
        if self.0.waiters.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.cancel.cancel();
        }
    }
}
/// One entry per live key, retained by completion independently of caller futures.
type FlightEntries<K, V, E> = Arc<Mutex<HashMap<K, Arc<Flight<V, E>>>>>;
/// Shared table of pending keyed loads.
pub struct Flights<K, V, E = DataFusionError> {
    entries: FlightEntries<K, V, E>,
    limit: usize,
}
impl<K, V, E> Flights<K, V, E>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    /// Finite maximum simultaneous keys and waiters per key. Zero refuses all work.
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Arc::default(),
            limit,
        }
    }
    /// Active keys including cancelled native work that has not exited.
    pub fn active(&self) -> usize {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
    /// Share typed completion; the callback must retain all native leases through actual exit.
    pub async fn load_owned<F>(
        &self,
        key: K,
        create: impl FnOnce(FlightCancellation) -> F + Send + 'static,
    ) -> std::result::Result<Arc<V>, FlightError<E>>
    where
        F: Future<Output = std::result::Result<Arc<V>, E>> + Send + 'static,
    {
        let waiter = {
            let mut entries = self
                .entries
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(f) = entries.get(&key) {
                if f.cancel.flag.load(Ordering::Acquire) {
                    return Err(FlightError::Cancelled);
                }
                if f.waiters.load(Ordering::Acquire) >= self.limit {
                    return Err(FlightError::Capacity);
                }
                f.waiters.fetch_add(1, Ordering::AcqRel);
                Waiter(f.clone())
            } else {
                if entries.len() >= self.limit {
                    return Err(FlightError::Capacity);
                }
                let (tx, rx) = tokio::sync::oneshot::channel();
                let result = async move { rx.await.unwrap_or(Err(FlightError::Panicked)) }
                    .boxed()
                    .shared();
                let f = Arc::new(Flight {
                    result,
                    cancel: FlightCancellation::default(),
                    waiters: AtomicUsize::new(1),
                });
                entries.insert(key.clone(), f.clone());
                let owner = self.entries.clone();
                let active = f.clone();
                tokio::spawn(async move {
                    let outcome =
                        std::panic::AssertUnwindSafe(async { create(active.cancel.clone()).await })
                            .catch_unwind()
                            .await;
                    let result = match outcome {
                        Ok(v) => v.map_err(|e| FlightError::Load(Arc::new(e))),
                        Err(_) => Err(FlightError::Panicked),
                    };
                    // Remove before waking waiters: a failure can immediately be retried.
                    let mut entries = owner
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if entries
                        .get(&key)
                        .is_some_and(|entry| Arc::ptr_eq(entry, &active))
                    {
                        entries.remove(&key);
                    }
                    drop(entries);
                    let _ = tx.send(result);
                });
                Waiter(f)
            }
        };
        let result = waiter.0.result.clone().await;
        drop(waiter);
        result
    }
}
impl<K, V> Flights<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Send + Sync + 'static,
{
    /// Async-only loader: last-waiter cancellation drops the underlying future.
    /// Native workers use `load_owned` and join before completion instead.
    pub async fn load<F>(
        &self,
        key: K,
        create: impl FnOnce() -> F + Send + 'static,
    ) -> Result<Arc<V>>
    where
        F: Future<Output = Result<Arc<V>>> + Send + 'static,
    {
        self.load_owned(key,move|cancel|async move{tokio::select!{result=create()=>result,()=cancel.cancelled()=>Err(DataFusionError::Execution("cache load cancelled".into()))}}).await.map_err(|e|match e{
            FlightError::Load(e)=>DataFusionError::Shared(e),FlightError::Capacity=>DataFusionError::ResourcesExhausted("cache flight capacity".into()),FlightError::Cancelled=>DataFusionError::Execution("cache flight retiring".into()),FlightError::Panicked=>DataFusionError::Internal("cache loader panicked".into())})
    }
}
impl<K, V, E> std::fmt::Debug for Flights<K, V, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flights")
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_flight_error_implements_diagnostic() {
        fn diagnostic<T: std::error::Error + miette::Diagnostic>() {}
        diagnostic::<FlightError<std::io::Error>>();
    }

    #[tokio::test]
    async fn completion_retains_identity_and_other_waiters_survive() {
        let flights = Arc::new(Flights::<u8, usize, &'static str>::new(2));
        let gate = Arc::new(tokio::sync::Notify::new());
        let entered = Arc::new(tokio::sync::Notify::new());
        let calls = Arc::new(AtomicUsize::new(0));
        let create = {
            let gate = gate.clone();
            let calls = calls.clone();
            let entered = entered.clone();
            move |_: FlightCancellation| async move {
                calls.fetch_add(1, Ordering::SeqCst);
                entered.notify_one();
                gate.notified().await;
                Ok(Arc::new(7))
            }
        };
        let mut a = Box::pin(flights.load_owned(1, create));
        assert!(futures_util::poll!(a.as_mut()).is_pending());
        entered.notified().await;
        let mut b = Box::pin(flights.load_owned(1, |_| async { Err("duplicate") }));
        assert!(futures_util::poll!(b.as_mut()).is_pending());
        drop(a);
        gate.notify_one();
        assert_eq!(*b.await.unwrap(), 7);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(flights.active(), 0);
    }
    #[tokio::test]
    async fn last_waiter_cancel_does_not_admit_duplicate_until_exit() {
        let f = Flights::<u8, usize, &'static str>::new(1);
        let exit = Arc::new(tokio::sync::Notify::new());
        let observed = Arc::new(tokio::sync::Notify::new());
        let e = exit.clone();
        let o = observed.clone();
        let mut a = Box::pin(f.load_owned(1, move |c| async move {
            c.cancelled().await;
            o.notify_one();
            e.notified().await;
            Err("cancelled")
        }));
        assert!(futures_util::poll!(a.as_mut()).is_pending());
        drop(a);
        observed.notified().await;
        assert_eq!(f.active(), 1);
        assert!(matches!(
            f.load_owned(1, |_| async { Ok(Arc::new(1)) }).await,
            Err(FlightError::Cancelled)
        ));
        assert!(matches!(
            f.load_owned(2, |_| async { Ok(Arc::new(1)) }).await,
            Err(FlightError::Capacity)
        ));
        exit.notify_one();
        while f.active() != 0 {
            tokio::task::yield_now().await;
        }
        assert_eq!(
            *f.load_owned(1, |_| async { Ok(Arc::new(9)) })
                .await
                .unwrap(),
            9
        );
    }
    #[tokio::test]
    async fn failure_and_panic_are_retryable() {
        let f = Flights::<u8, usize, &'static str>::new(1);
        assert!(f.load_owned(1, |_| async { Err("failed") }).await.is_err());
        assert!(matches!(
            f.load_owned(1, |_| async { panic!("fixture") }).await,
            Err(FlightError::Panicked)
        ));
        assert_eq!(
            *f.load_owned(1, |_| async { Ok(Arc::new(3)) })
                .await
                .unwrap(),
            3
        );
    }
}
