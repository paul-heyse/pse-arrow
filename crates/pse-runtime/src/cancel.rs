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
    pub fn with_deadline(&self, deadline: Option<std::time::Instant>) -> Self {
        Self {
            token: self.token.clone(),
            deadline: self.deadline.into_iter().chain(deadline).min(),
        }
    }

    /// A child inherits parent cancellation and the effective enclosing clock,
    /// while its own cancellation cannot reach its parent or siblings.
    pub(crate) fn child_with_deadline(&self, deadline: Option<std::time::Instant>) -> Self {
        let mut child = self.with_deadline(deadline);
        child.token = self.token.child_token();
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

    /// Bridge a tree cancellation to the native callback's flat leaf and retain
    /// the owning operation until its destruction/join completes. A ready stop
    /// wins a simultaneously ready candidate; the typed drain result is retained.
    pub(crate) async fn join_native<T, Drain>(
        &self,
        native: &pse_diagnostics::Cancellation,
        work: impl Future<Output = T>,
        on_stop: impl FnOnce() -> Drain,
    ) -> T {
        pse_diagnostics::stop::drain_on_stop(
            async { self.cancelled().await; pse_diagnostics::stop::StopReason::Cancelled },
            work,
            |_| { native.cancel(); on_stop() },
        ).await.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn cancellation_children_inherit_parent_without_upward_or_sibling_cancellation() {
        let parent = CancelSource::new();
        let child = parent.child_with_deadline(None);
        let sibling = parent.child_with_deadline(None);
        let descendant = child.child_with_deadline(None);
        child.cancel();
        assert!(child.token().is_cancelled());
        assert!(descendant.token().is_cancelled());
        assert!(!parent.token().is_cancelled());
        assert!(!sibling.token().is_cancelled());
        parent.cancel();
        assert!(sibling.token().is_cancelled());
        assert!(parent.child_with_deadline(None).token().is_cancelled());
    }

    #[test]
    fn cancellation_child_keeps_the_original_minimum_clock() {
        let early = Instant::now() + Duration::from_secs(1);
        let later = early + Duration::from_secs(1);
        let parent = CancelSource::new().with_deadline(Some(early));
        assert_eq!(parent.child_with_deadline(None).deadline(), Some(early));
        assert_eq!(
            parent.child_with_deadline(Some(later)).deadline(),
            Some(early)
        );
        let child = CancelSource::new().child_with_deadline(Some(later));
        assert_eq!(
            child.child_with_deadline(Some(early)).deadline(),
            Some(early)
        );
        assert_eq!(parent.deadline(), Some(early));
    }

    #[tokio::test]
    async fn cancellation_child_wait_observes_parent_before_and_after_registration() {
        let parent = CancelSource::new();
        let child = parent.child_with_deadline(None);
        parent.cancel();
        child.cancelled().await;
        parent.child_with_deadline(None).cancelled().await;
        assert!(matches!(child.checkpoint(), Err(RuntimeError::Cancelled)));
        let parent = CancelSource::new();
        let child = parent.child_with_deadline(None);
        let mut waiting = Box::pin(child.cancelled());
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        parent.cancel();
        waiting.await;
    }
}
