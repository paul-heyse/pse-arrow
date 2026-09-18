// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded, weak single-flight coordination. Last-waiter drop cancels population.
use datafusion::common::{DataFusionError, Result};
use futures_util::{
    FutureExt,
    future::{BoxFuture, Shared},
};
use std::{
    collections::HashMap,
    future::Future,
    hash::Hash,
    sync::{Arc, Mutex, Weak},
};

type SharedLoad<V> = Shared<BoxFuture<'static, std::result::Result<Arc<V>, Arc<DataFusionError>>>>;
struct Flight<V>(SharedLoad<V>);
pub(super) struct Flights<K, V> {
    entries: Mutex<HashMap<K, Weak<Flight<V>>>>,
    limit: usize,
}
impl<K, V> Flights<K, V>
where
    K: Eq + Hash,
    V: Send + Sync + 'static,
{
    pub(super) fn new(limit: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            limit,
        }
    }
    pub(super) async fn load<F>(&self, key: K, create: impl FnOnce() -> F) -> Result<Arc<V>>
    where
        F: Future<Output = Result<Arc<V>>> + Send + 'static,
    {
        let flight = {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| DataFusionError::Internal("cache flight lock poisoned".into()))?;
            entries.retain(|_, value| value.strong_count() > 0);
            if let Some(flight) = entries.get(&key).and_then(Weak::upgrade) {
                flight
            } else {
                let flight = Arc::new(Flight(
                    create()
                        .map(|result| result.map_err(Arc::new))
                        .boxed()
                        .shared(),
                ));
                // Overflow requests still run through the bounded loader's native
                // semaphore. They do not grow this coordination map without bound.
                if entries.len() < self.limit {
                    entries.insert(key, Arc::downgrade(&flight));
                }
                flight
            }
        };
        let result = flight.0.clone().await.map_err(DataFusionError::Shared);
        drop(flight);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[tokio::test]
    async fn concurrent_waiters_share_work_and_last_drop_releases_it() {
        let flights = Flights::<u8, usize>::new(1);
        let calls = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(tokio::sync::Notify::new());
        let create = || {
            let calls = calls.clone();
            let gate = gate.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                gate.notified().await;
                Ok(Arc::new(7))
            }
        };
        let mut first = Box::pin(flights.load(1, create));
        assert!(futures_util::poll!(first.as_mut()).is_pending());
        let mut second = Box::pin(flights.load(1, create));
        assert!(futures_util::poll!(second.as_mut()).is_pending());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(first);
        gate.notify_one();
        assert_eq!(*second.await.unwrap(), 7);
        let mut abandoned = Box::pin(flights.load(2, create));
        assert!(futures_util::poll!(abandoned.as_mut()).is_pending());
        drop(abandoned);
        let mut retry = Box::pin(flights.load(2, create));
        assert!(futures_util::poll!(retry.as_mut()).is_pending());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
    #[tokio::test]
    async fn failure_is_not_a_negative_cache_entry() {
        let flights = Flights::<u8, usize>::new(1);
        assert!(
            flights
                .load(1, || async {
                    Err(DataFusionError::Execution("load failed".into()))
                })
                .await
                .is_err()
        );
        assert_eq!(
            *flights.load(1, || async { Ok(Arc::new(8)) }).await.unwrap(),
            8
        );
    }
}
