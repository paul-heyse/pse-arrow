//! Opt-in cancellation shared by descendants of one read-only database request.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// A monotonic cancellation request for an explicitly forked database branch.
///
/// Cloned database handles in this branch retain their own query stacks, local
/// cancellation tokens and cancellation-suppression state. Requesting the group
/// never modifies a database revision or a handle outside the branch. A requested
/// group cannot be reset; start a new branch for the next independent request.
#[derive(Clone, Debug, Default)]
pub struct CancellationGroup(Arc<AtomicBool>);

impl CancellationGroup {
    /// Request cooperative cancellation at the normal Salsa cancellation points.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    /// Whether cancellation of this group has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
