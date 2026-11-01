// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Session retention is independent of framework-owned handler and stream futures.
use super::{Error, Result};
use std::sync::Mutex;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

pub(super) struct Workers {
    stop: CancellationToken,
    admitted: Mutex<JoinSet<()>>,
}
impl Workers {
    pub(super) fn new(stop: CancellationToken) -> Self {
        Self {
            stop,
            admitted: Mutex::new(JoinSet::new()),
        }
    }

    /// The lock serializes admission with stop; a submitted worker is always retained.
    pub(super) fn spawn(&self, worker: impl Future<Output = ()> + Send + 'static) -> Result<()> {
        let mut admitted = self.admitted.lock().map_err(super::engine::protocol)?;
        if self.stop.is_cancelled() {
            return Err(Error::Unavailable("semantic session is stopping".into()));
        }
        while let Some(result) = admitted.try_join_next() {
            result.map_err(super::engine::protocol)?;
        }
        admitted.spawn(worker);
        Ok(())
    }

    pub(super) fn reap(&self) -> Result<()> {
        let mut admitted = self.admitted.lock().map_err(super::engine::protocol)?;
        while let Some(result) = admitted.try_join_next() {
            result.map_err(super::engine::protocol)?;
        }
        Ok(())
    }

    pub(super) fn stop(&self) -> Result<()> {
        let _admitted = self.admitted.lock().map_err(super::engine::protocol)?;
        self.stop.cancel();
        Ok(())
    }

    pub(super) async fn drain(&self) -> Result<()> {
        let mut admitted = {
            let mut admitted = self.admitted.lock().map_err(super::engine::protocol)?;
            self.stop.cancel();
            std::mem::take(&mut *admitted)
        };
        let mut failure = None;
        while let Some(result) = admitted.join_next().await {
            if let Err(error) = result {
                failure.get_or_insert_with(|| super::engine::protocol(error));
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

/// Losing a delivery future requests cancellation; retained workers still settle their owners.
pub(super) struct CancelOnDrop(pub(super) CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
