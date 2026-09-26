// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! DataFusion error adaptation for neutral completion-owned flights.
use datafusion::common::{DataFusionError, Result};
use pse_columnar::flight::FlightError;
use std::{future::Future, hash::Hash, sync::Arc};
/// Data-source loads use DataFusion's typed error boundary.
#[derive(Debug)]
pub struct Flights<K, V>(pse_columnar::flight::Flights<K, V, DataFusionError>);
impl<K, V> Flights<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Send + Sync + 'static,
{
    /// Bounded concurrent keys and waiters.
    pub fn new(limit: usize) -> Self {
        Self(pse_columnar::flight::Flights::new(limit))
    }
    /// Loads still owned by their completion supervisor.
    pub fn active(&self) -> usize {
        self.0.active()
    }
    /// Last-waiter cancellation drops an async load. Native work uses the neutral
    /// `load_owned` boundary and retains its leases through actual native exit.
    pub async fn load<F>(
        &self,
        key: K,
        create: impl FnOnce() -> F + Send + 'static,
    ) -> Result<Arc<V>>
    where
        F: Future<Output = Result<Arc<V>>> + Send + 'static,
    {
        self.0.load_owned(key, move |cancel| async move {
            tokio::select! { result = create() => result, () = cancel.cancelled() => Err(DataFusionError::Execution("cache load cancelled".into())) }
        }).await.map_err(|e| match e {
            FlightError::Load(e) => DataFusionError::Shared(e),
            FlightError::Capacity => DataFusionError::ResourcesExhausted("cache flight capacity".into()),
            FlightError::Retiring => DataFusionError::ResourcesExhausted("cache flight retiring; retry after completion".into()),
            FlightError::Panicked => DataFusionError::Internal("cache loader panicked".into()),
        })
    }
}
