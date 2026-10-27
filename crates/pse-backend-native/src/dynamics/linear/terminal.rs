// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Attempt-local transport across Diffsol's infallible linearization callback.
use crate::{ProblemError, LimitKind};
use std::{cell::RefCell, rc::Rc};
use diffsol_la::error::LinearSolverError;
#[derive(Clone, Default)]
pub(super) struct Attempt {
    cause: Rc<RefCell<Option<ProblemError>>>,
    pub(super) allowance: Option<usize>,
}
thread_local! { static CURRENT: RefCell<Option<Attempt>> = const { RefCell::new(None) }; }
pub(crate) struct Scope(Option<Attempt>);
impl Drop for Scope { fn drop(&mut self) { CURRENT.with(|c| *c.borrow_mut()=self.0.take()); } }
pub(crate) fn scope(allowance: Option<usize>) -> Scope {
    let allowance=allowance.or_else(|| CURRENT.with(|c| c.borrow().as_ref().and_then(|a|a.allowance)));
    Scope(CURRENT.with(|c| c.borrow_mut().replace(Attempt { allowance, ..Default::default() })))
}
pub(super) fn current() -> Attempt { CURRENT.with(|c| c.borrow().clone().unwrap_or_default()) }
pub(crate) fn take() -> Option<ProblemError> { CURRENT.with(|c| c.borrow().as_ref().and_then(|a| a.cause.borrow_mut().take())) }
impl Attempt {
    pub(super) fn failed(&self) -> bool { self.cause.borrow().is_some() }
    pub(super) fn defer(&self, error: ProblemError) -> LinearSolverError {
        let terminal = matches!(&error,ProblemError::Limit{kind:LimitKind::Memory,..}|ProblemError::Contract(_)|ProblemError::Internal(_));
        if terminal {
            let mut held=self.cause.borrow_mut();
            if held.is_none() { *held=Some(error); }
            LinearSolverError::Other("terminal native linear solver cause retained by attempt".into())
        } else { LinearSolverError::LuSolveFailed }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_terminal_is_first_owned_and_scoped_while_singularity_can_recover() {
        let scope1=scope(None);
        let context=current();
        context.defer(ProblemError::numerical("trial singularity"));
        assert!(take().is_none());
        context.defer(ProblemError::memory("injected allocation refusal"));
        context.defer(ProblemError::Contract("later contract".into()));
        assert!(matches!(take(),Some(ProblemError::Limit{kind:LimitKind::Memory,..})));
        context.defer(ProblemError::Contract("old attempt".into()));
        { let _scope2=scope(None); assert!(take().is_none()); }
        assert!(matches!(take(),Some(ProblemError::Contract(_))));
        drop(scope1);
        assert!(take().is_none());
    }
}
