// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    reason = "test fixtures panic when an asserted setup contract fails"
)]
use super::*;
#[test]
fn execution_scope_clones_share_absolute_deadline_and_cancellation_owner() {
    let cancel = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_secs(1);
    let original = ExecutionScope::new(cancel.clone(), Some(deadline));
    let child = original.clone();
    assert_eq!(child.deadline(), Some(deadline));
    assert!(Arc::ptr_eq(child.cancellation(), &cancel));
    assert!(child.remaining(Duration::from_secs(20)).unwrap() <= Duration::from_secs(1));
    let expired = ExecutionScope::new(cancel.clone(), Some(Instant::now()));
    assert!(matches!(expired.check(), Err(ProviderError::Limit(_))));
    assert!(!cancel.load(Ordering::Acquire));
    cancel.store(true, Ordering::Release);
    assert!(matches!(expired.check(), Err(ProviderError::Cancelled)));
}
