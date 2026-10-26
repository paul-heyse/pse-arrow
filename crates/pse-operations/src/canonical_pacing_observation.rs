// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Task-scoped native qualification observations; absent from production builds.

use super::QueryEffect;
use std::{
    collections::BTreeMap,
    future::Future,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct GateObservation {
    pub submitted: u64,
    pub reads: u64,
    pub mutations: u64,
    pub queued: Duration,
    pub longest_queue: Duration,
    pub held: Duration,
    pub conflicts: u64,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Probe(Arc<Mutex<BTreeMap<String, GateObservation>>>);
impl Probe {
    pub(crate) fn snapshot(&self) -> BTreeMap<String, GateObservation> {
        self.0
            .lock()
            .map(|observations| observations.clone())
            .unwrap_or_default()
    }
    pub(crate) async fn scope<F: Future>(&self, future: F) -> F::Output {
        CURRENT.scope(self.clone(), future).await
    }
}
tokio::task_local! { static CURRENT: Probe; }

pub(super) struct TurnObservation {
    probe: Probe,
    key: String,
    entered: Instant,
}
impl Drop for TurnObservation {
    fn drop(&mut self) {
        if let Ok(mut observations) = self.probe.0.lock() {
            observations.entry(self.key.clone()).or_default().held += self.entered.elapsed();
        }
    }
}
pub(super) fn submitted(key: &str) {
    let _ = CURRENT.try_with(|probe| {
        if let Ok(mut observations) = probe.0.lock() {
            observations.entry(key.to_owned()).or_default().submitted += 1;
        }
    });
}
pub(super) fn entered(key: &str, effect: QueryEffect, queued: Duration) -> Option<TurnObservation> {
    CURRENT
        .try_with(|probe| {
            if let Ok(mut observations) = probe.0.lock() {
                let observation = observations.entry(key.to_owned()).or_default();
                match effect {
                    QueryEffect::Read => observation.reads += 1,
                    QueryEffect::Mutation => observation.mutations += 1,
                }
                observation.queued += queued;
                observation.longest_queue = observation.longest_queue.max(queued);
            }
            TurnObservation {
                probe: probe.clone(),
                key: key.to_owned(),
                entered: Instant::now(),
            }
        })
        .ok()
}
pub(crate) fn conflict(key: &str) {
    let _ = CURRENT.try_with(|probe| {
        if let Ok(mut observations) = probe.0.lock() {
            observations.entry(key.to_owned()).or_default().conflicts += 1;
        }
    });
}
