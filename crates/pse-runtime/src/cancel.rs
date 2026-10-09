// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Driver cancellation shared with cooperative loops and asynchronous work.

use pse_columnar::CancellationToken;

use crate::RuntimeError;

/// The driver's cancellation handle. Drivers cancel through this source so asynchronous
/// waiters and CPU-loop checkpoints observe the same event.
#[derive(Clone, Debug)]
pub struct CancelSource {
    token: CancellationToken,
    deadline: Option<std::time::Instant>,
}

impl Default for CancelSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CancelSource {
    /// Opens an uncancelled source.
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            deadline: None,
        }
    }

    /// The original enclosing operation clock, independent of token cancellation.
    pub(crate) fn deadline(&self) -> Option<std::time::Instant> {
        self.deadline
    }

    /// Narrow the same driver's clock without renewing it or changing cancellation.
    pub(crate) fn with_deadline(&self, deadline: Option<std::time::Instant>) -> Self {
        Self {
            token: self.token.clone(),
            deadline: self.deadline.into_iter().chain(deadline).min(),
        }
    }

    /// An independently cancellable child retains the effective enclosing clock.
    pub(crate) fn child_with_deadline(&self, deadline: Option<std::time::Instant>) -> Self {
        let mut child = self.with_deadline(deadline);
        child.token = CancellationToken::new();
        child
    }

    /// A shared token for synchronous loops.
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Cancels all tokens and wakes asynchronous waiters; idempotent.
    pub fn cancel(&self) {
        self.token.cancel();
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
        self.token.cancelled().await;
    }
}
