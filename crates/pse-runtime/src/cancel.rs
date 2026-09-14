// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Driver cancellation shared with cooperative loops and asynchronous work.

use std::sync::Arc;

use pse_ids::CancellationToken;
use tokio::sync::watch;

use crate::RuntimeError;

/// The driver's cancellation handle. Drivers cancel through this source so asynchronous
/// waiters and CPU-loop checkpoints observe the same event.
#[derive(Clone, Debug)]
pub struct CancelSource {
    token: CancellationToken,
    notification: Arc<watch::Sender<bool>>,
}

impl Default for CancelSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CancelSource {
    /// Opens an uncancelled source.
    pub fn new() -> Self {
        let (notification, _) = watch::channel(false);
        Self {
            token: CancellationToken::new(),
            notification: Arc::new(notification),
        }
    }

    /// A shared token for synchronous loops.
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Cancels all tokens and wakes asynchronous waiters; idempotent.
    pub fn cancel(&self) {
        self.token.cancel();
        self.notification.send_replace(true);
    }

    /// Refuses cancelled work before starting or publishing it.
    ///
    /// # Errors
    /// [`RuntimeError::Cancelled`] after cancellation.
    pub fn checkpoint(&self) -> Result<(), RuntimeError> {
        if self.token.is_cancelled() {
            Err(RuntimeError::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Waits for driver cancellation, including cancellation before this call.
    pub async fn cancelled(&self) {
        let mut receiver = self.notification.subscribe();
        while !self.token.is_cancelled() {
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}
