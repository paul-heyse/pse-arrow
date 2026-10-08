// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Runtime-owned capacity growth at the actual numeric allocation boundary.
//! Mathematical operation/scratch limits are independent of this injected owner.
use crate::MathError;
use std::{cell::RefCell, sync::Arc};

/// The enclosing admission owner reserves positive constructor excess before
/// allocation. Implementations must return immediately, without waiting for space.
pub trait ConstructionAdmission: std::fmt::Debug + Send + Sync + 'static {
    /// Extend the same live admission; an unavailable extent is a resource refusal.
    fn try_grow(&self, bytes: usize) -> Result<(), MathError>;
}
thread_local! {
    static OWNER: RefCell<Option<Arc<dyn ConstructionAdmission>>> = const { RefCell::new(None) };
}
struct Restore(Option<Arc<dyn ConstructionAdmission>>);
impl Drop for Restore {
    fn drop(&mut self) {
        OWNER.with(|owner| {
            owner.replace(self.0.take());
        });
    }
}
/// Install the enclosing owner on its actual constructor thread. Nested scopes
/// restore their parent, including on panic; publication retains the runtime lease.
pub fn scoped<T>(owner: Arc<dyn ConstructionAdmission>, work: impl FnOnce() -> T) -> T {
    let restore = Restore(OWNER.with(|current| current.replace(Some(owner))));
    let result = work();
    drop(restore);
    result
}
pub(crate) fn additional(bytes: usize) -> Result<(), MathError> {
    if bytes == 0 {
        return Ok(());
    }
    let owner = OWNER.with(|owner| owner.borrow().clone());
    // Pure mathematical callers retain their explicit EvaluationLimits. Runtime
    // jobs always install their common-pool owner before construction starts.
    if let Some(owner) = owner {
        owner.try_grow(bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Recorder(AtomicUsize);
    impl ConstructionAdmission for Recorder {
        fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
            self.0.fetch_add(bytes, Ordering::SeqCst);
            Ok(())
        }
    }
    #[test]
    fn construction_owner_is_restored_after_nested_scope_and_panic() {
        let outer = Arc::new(Recorder(AtomicUsize::new(0)));
        let inner = Arc::new(Recorder(AtomicUsize::new(0)));
        scoped(outer.clone(), || {
            additional(3).unwrap();
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                scoped(inner.clone(), || {
                    additional(5).unwrap();
                    panic!("constructor panic");
                });
            }));
            additional(7).unwrap();
        });
        additional(11).unwrap();
        assert_eq!(outer.0.load(Ordering::SeqCst), 10);
        assert_eq!(inner.0.load(Ordering::SeqCst), 5);
    }
}
