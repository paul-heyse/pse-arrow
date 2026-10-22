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
/// Clone the enclosing allocation admission for an explicitly owned execution lane.
/// Install it with [`scoped`] on that lane; retaining this handle does not grant compute
/// capacity or change the admission's original lifetime and cancellation owner.
pub fn current() -> Option<Arc<dyn ConstructionAdmission>> {
    OWNER.with(|owner| owner.borrow().clone())
}
#[track_caller]
pub(crate) fn additional(bytes: usize) -> Result<(), MathError> {
    if bytes == 0 {
        return Ok(());
    }
    let owner = OWNER.with(|owner| owner.borrow().clone());
    // Pure mathematical callers retain their explicit EvaluationLimits. Runtime
    // jobs always install their common-pool owner before construction starts.
    if let Some(owner) = owner {
        let site = std::panic::Location::caller();
        owner.try_grow(bytes).inspect_err(|_| {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.text("math.constructor_refusal_site", &format!("{}:{}", site.file(), site.line()));
                context.count("math.constructor_refused_bytes", "bytes", bytes as u64);
            }
        })?;
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
    fn refused_construction_reports_actual_caller_without_changing_the_cause() {
        #[derive(Debug)]
        struct Refuse;
        impl ConstructionAdmission for Refuse {
            fn try_grow(&self, _: usize) -> Result<(), MathError> {
                Err(MathError::Limit("test constructor refusal"))
            }
        }
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("constructor refusal attribution");
        let context = operation.context();
        let _entered = context.enter();
        let call_line = line!() + 1;
        let outcome = scoped(Arc::new(Refuse), || additional(17));
        assert!(matches!(outcome, Err(MathError::Limit("test constructor refusal"))));
        let snapshot = context.reader().snapshot(std::time::Duration::from_secs(60));
        assert!(snapshot.metrics.iter().any(|metric| metric.name == "math.constructor_refusal_site"
            && matches!(&metric.latest, pse_diagnostics::observation::Value::Text(site) if site.ends_with(&format!("construction.rs:{call_line}")))));
        assert!(snapshot.metrics.iter().any(|metric| metric.name == "math.constructor_refused_bytes"
            && metric.cumulative == pse_diagnostics::observation::Value::Count(17)));
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
