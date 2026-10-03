// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A delayed positive global proof cannot extend the enclosing attempt or its cache.
#![allow(
    clippy::unwrap_used,
    reason = "test fixtures panic when an asserted setup contract fails"
)]
use super::*;
use pse_kernels::ExecutionScope;

#[derive(Debug)]
struct DelayedProof {
    inner: Arc<dyn super::super::SelectionVerifier>,
    delay: Duration,
}
impl super::super::SelectionVerifier for DelayedProof {
    fn identity(&self) -> pse_ids::ContentHash {
        self.inner.identity()
    }
    fn workspace_bytes(
        &self,
        programs: &[Arc<crate::factorable::RootIsolationProgram>],
    ) -> Result<usize, MathError> {
        self.inner.workspace_bytes(programs)
    }
    fn certify(
        &self,
        request: &super::super::SelectionProofRequest<'_>,
    ) -> Result<super::super::SelectionEvidence, MathError> {
        assert!(request.time_limit <= Duration::from_millis(40));
        std::thread::sleep(self.delay);
        self.inner.certify(request)
    }
}
#[test]
fn implicit_regimes_outer_deadline_rejects_late_positive_proof_without_cache_or_branch() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut selected, calls) = tests::group_selection(None, false);
    selected.verifier = Some(Arc::new(DelayedProof {
        inner: selected.verifier.take().unwrap(),
        delay: Duration::from_millis(60),
    }));
    selected.scope = Some(ExecutionScope::new(
        cancel.clone(),
        Some(Instant::now() + Duration::from_millis(40)),
    ));
    assert!(matches!(
        selected.evaluate(&[-2.], DerivativeOrder::Second, &cancel),
        Err(MathError::Limit(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(selected.chart.is_none());
    assert!(selected.derivative_branch.is_none());
    assert!(!cancel.load(Ordering::Acquire));
    assert!(matches!(
        selected.evaluate(&[-2.], DerivativeOrder::Second, &cancel),
        Err(MathError::Limit(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}
