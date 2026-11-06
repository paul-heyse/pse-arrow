// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consume the original parent's granted restoration envelope on the child lane.
//! This is requested backing accounting, not an allocator or whole-process bound.
use super::MathRuntimeError;
use pse_kernels::ExecutionScope;
use pse_math::{AllocationOwner, MathError, construction::ConstructionAdmission};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub(super) struct RestorationAdmission {
    limit: usize,
    used: AtomicUsize,
    scope: ExecutionScope,
}
impl RestorationAdmission {
    // Only the authenticated handler constructs this after receiving a matching
    // grant. Neither an input snapshot nor its mathematical limits call this.
    pub(super) fn new(limit: usize, scope: ExecutionScope) -> Result<Arc<Self>, MathRuntimeError> {
        if limit == 0 {
            return Err(MathRuntimeError::Limit("empty child restoration grant"));
        }
        scope.check().map_err(MathError::Scope)?;
        Ok(Arc::new(Self {
            limit,
            used: AtomicUsize::new(0),
            scope,
        }))
    }
    pub(super) fn charge(
        self: &Arc<Self>,
        bytes: usize,
    ) -> Result<Arc<dyn AllocationOwner>, MathRuntimeError> {
        self.try_grow(bytes)?;
        Ok(self.clone())
    }
}
impl ConstructionAdmission for RestorationAdmission {
    fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
        self.scope.check().map_err(MathError::Scope)?;
        self.used
            .try_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .map(|_| ())
            .map_err(|used| {
                used.checked_add(bytes).map_or(
                    MathError::Limit("child restoration backing overflow"),
                    |required| MathError::ByteLimit {
                        resource: "child restoration backing",
                        required,
                        available: self.limit,
                    },
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::Cancellation;
    #[test]
    fn restoration_grant_is_shared_nonrenewable_and_denied_growth_preserves_remaining() {
        let budget =
            RestorationAdmission::new(64, ExecutionScope::new(Cancellation::default(), None))
                .unwrap();
        let start = Arc::new(std::sync::Barrier::new(8));
        let threads = (0..8)
            .map(|_| {
                let budget = budget.clone();
                let start = start.clone();
                std::thread::spawn(move || {
                    start.wait();
                    budget.charge(16)
                })
            })
            .collect::<Vec<_>>();
        let mut owners = Vec::new();
        for thread in threads {
            match thread.join().unwrap() {
                Ok(owner) => owners.push(owner),
                Err(MathRuntimeError::Math(MathError::ByteLimit {
                    resource: "child restoration backing",
                    ..
                })) => {}
                other => panic!("unexpected admission outcome: {other:?}"),
            }
        }
        assert_eq!(owners.len(), 4);
        assert_eq!(budget.used.load(Ordering::Acquire), 64);
        // Dropping a buffer owner does not renew a phase-monotonic envelope.
        owners.pop();
        assert!(budget.charge(1).is_err());
        let lifetime = Arc::downgrade(&budget);
        drop(budget);
        assert!(lifetime.upgrade().is_some());
        drop(owners);
        assert!(lifetime.upgrade().is_none());
    }
    #[test]
    fn restoration_grant_keeps_original_scope_and_refuses_overflow_without_a_debit() {
        let stop = Cancellation::default();
        let budget =
            RestorationAdmission::new(32, ExecutionScope::new(stop.clone(), None)).unwrap();
        budget.charge(7).unwrap();
        assert!(matches!(
            budget.try_grow(usize::MAX),
            Err(MathError::Limit("child restoration backing overflow"))
        ));
        assert_eq!(budget.used.load(Ordering::Acquire), 7);
        stop.cancel();
        assert!(matches!(
            budget.try_grow(1),
            Err(MathError::Scope(pse_kernels::ProviderError::Cancelled))
        ));
        assert_eq!(budget.used.load(Ordering::Acquire), 7);

        let expired = ExecutionScope::new(
            Cancellation::default(),
            Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        );
        assert!(matches!(
            RestorationAdmission::new(32, expired),
            Err(MathRuntimeError::Math(MathError::Scope(
                pse_kernels::ProviderError::Deadline
            )))
        ));
    }
}
