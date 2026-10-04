// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical-coordinate active-set factors for the library's activity/path operations.
//!
//! This is an active-set KKT adapter, not an interior-point iterate. Original lower
//! row multipliers change sign exactly once into the library's upper-limit observer
//! frame; the factor already answers in physical units, so no natural-unit factor
//! is applied again. Releases remove a named constraint coupling and retain an
//! independent -1 multiplier row, using a fresh FERAL factor of that operator.
use super::{KktFactor, Side};
use crate::{ProblemError, solve::Execution};
use pounce_sens_core::{SensBacksolver, backsolver::BoundRow};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

/// Finite additional work/storage allowance for one activity operation.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum actual library backsolves.
    pub backsolves: usize,
    /// Maximum actual release/pin refactorizations.
    pub refactorizations: usize,
    /// Maximum retained release factor and matrix bytes.
    pub bytes: usize,
}
impl Limits {
    /// Reject empty work or storage allowances before entering a library routine.
    pub fn validate(self) -> Result<(), ProblemError> {
        if self.backsolves == 0 || self.refactorizations == 0 || self.bytes == 0 {
            return Err(ProblemError::Contract(
                "activity operation requires finite positive work and storage allowances".into(),
            ));
        }
        Ok(())
    }
}
/// Actual work; no factorization is inferred from a path segment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    /// Actual backsolve calls, including refused calls.
    pub backsolves: usize,
    /// Actual FERAL refactorization calls.
    pub refactorizations: usize,
    /// Largest retained release factor and matrix charge.
    pub factor_bytes: usize,
}
#[derive(Debug)]
struct ReleasedFactor {
    released: Vec<usize>,
    pinned: Vec<usize>,
    factor: KktFactor,
}
#[derive(Default, Debug)]
struct State {
    backsolves: Cell<usize>,
    refactorizations: Cell<usize>,
    bytes: Cell<usize>,
    failure: RefCell<Option<Arc<ProblemError>>>,
    numerical: RefCell<Option<Arc<ProblemError>>>,
    cache: RefCell<Option<ReleasedFactor>>,
}
/// An explicitly oriented library view over a qualified active-set factor.
#[derive(Clone, Debug)]
pub(super) struct Factor {
    source: KktFactor,
    signs: Vec<f64>,
    bounds: Vec<BoundRow>,
    execution: Execution,
    limits: Limits,
    state: Rc<State>,
}
impl Factor {
    pub(super) fn new(
        source: KktFactor,
        original_variables: usize,
        execution: Execution,
        limits: Limits,
    ) -> Result<Self, ProblemError> {
        limits.validate()?;
        execution.check()?;
        let mut signs = vec![1.; source.dim()];
        for (k, (_, side)) in source.layout.rows.iter().enumerate() {
            if *side == Side::Lower {
                signs[source.layout.variables + k] = -1.;
            }
        }
        // Equal bounds and parameter pins are equations, not releasable limits.
        let base = source.layout.variables + source.layout.rows.len();
        let bounds = source
            .layout
            .bounds
            .iter()
            .enumerate()
            .filter_map(|(k, (column, side))| {
                (column.get() < original_variables && *side != Side::Equal).then_some(BoundRow {
                    row: base + k,
                    var_row: column.get(),
                    lower: *side == Side::Lower,
                })
            })
            .collect();
        Ok(Self {
            source,
            signs,
            bounds,
            execution,
            limits,
            state: Rc::default(),
        })
    }
    pub(super) fn work(&self) -> Work {
        Work {
            backsolves: self.state.backsolves.get(),
            refactorizations: self.state.refactorizations.get(),
            factor_bytes: self.state.bytes.get(),
        }
    }
    pub(super) fn failure(&self) -> Option<Arc<ProblemError>> {
        self.state.failure.borrow().clone()
    }
    pub(super) fn numerical_failure(&self) -> Option<Arc<ProblemError>> {
        self.state.numerical.borrow().clone()
    }
    pub(super) fn sign(&self, row: usize) -> Option<f64> {
        self.signs.get(row).copied()
    }
    fn stop(&self, error: ProblemError) -> bool {
        if self.state.failure.borrow().is_none() {
            *self.state.failure.borrow_mut() = Some(Arc::new(error));
        }
        false
    }
    fn checkpoint(&self) -> bool {
        self.failure().is_none()
            && match self.execution.check() {
                Ok(()) => true,
                Err(error) => self.stop(error),
            }
    }
    fn solve_with(&self, factor: &KktFactor, rhs: &[f64], lhs: &mut [f64]) -> bool {
        if !self.checkpoint() {
            return false;
        }
        if rhs.len() != self.dim() || lhs.len() != self.dim() || rhs.iter().any(|v| !v.is_finite())
        {
            return self.stop(ProblemError::Contract(
                "activity backsolve coordinate extent or value".into(),
            ));
        }
        let count = self.state.backsolves.get();
        if count >= self.limits.backsolves {
            return self.stop(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: "activity backsolve allowance exhausted".into(),
            });
        }
        self.state.backsolves.set(count + 1);
        let rhs = rhs
            .iter()
            .zip(&self.signs)
            .zip(factor.scales.iter())
            .map(|((r, s), p)| r * s * p / factor.objective)
            .collect::<Vec<_>>();
        let output = match factor.solver.solve_refined(&factor.matrix, &rhs) {
            Ok(output) => output,
            Err(
                error @ (feral::FeralError::NumericallyRankDeficient
                | feral::FeralError::SingularBasis { .. }),
            ) => {
                *self.state.numerical.borrow_mut() =
                    Some(Arc::new(crate::conditioning::native(error)));
                return false;
            }
            Err(error) => return self.stop(crate::conditioning::native(error)),
        };
        if output.len() != self.dim() {
            return self.stop(ProblemError::internal(
                "activity factor returned another coordinate extent",
            ));
        }
        if output.iter().any(|v| !v.is_finite()) {
            return self.stop(ProblemError::numerical("nonfinite activity backsolve"));
        }
        if !self.checkpoint() {
            return false;
        }
        self.state.numerical.borrow_mut().take();
        for (((destination, value), sign), scale) in lhs
            .iter_mut()
            .zip(output)
            .zip(&self.signs)
            .zip(factor.scales.iter())
        {
            *destination = value * sign * scale;
        }
        true
    }
    fn released(
        &self,
        released: &[usize],
        pinned: &[usize],
    ) -> Result<Option<KktFactor>, ProblemError> {
        self.execution.check()?;
        let mut released = released.to_vec();
        released.sort_unstable();
        released.dedup();
        let mut pinned = pinned.to_vec();
        pinned.sort_unstable();
        pinned.dedup();
        let layout = self.source.layout();
        if released.iter().any(|&r| {
            r < layout.variables
                || r >= layout.dim()
                || (r < layout.variables + layout.rows.len()
                    && layout.rows[r - layout.variables].1 == Side::Equal)
                || (r >= layout.variables + layout.rows.len()
                    && !self.bounds.iter().any(|b| b.row == r))
        }) || pinned.iter().any(|&r| r >= layout.variables)
        {
            return Err(ProblemError::Contract(
                "activity release/pin names a nonreleasable coordinate".into(),
            ));
        }
        if let Some(cached) = self.state.cache.borrow().as_ref()
            && cached.released == released
            && cached.pinned == pinned
        {
            return Ok(Some(cached.factor.clone()));
        }
        let count = self.state.refactorizations.get();
        if count >= self.limits.refactorizations {
            return Err(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: "activity refactorization allowance exhausted".into(),
            });
        }
        let entries = self.source.matrix.values.len();
        let allowance = entries
            .checked_mul(64)
            .and_then(|v| layout.dim().checked_mul(32).and_then(|d| v.checked_add(d)))
            .ok_or_else(|| ProblemError::memory("activity release matrix extent"))?;
        if allowance > self.limits.bytes {
            return Err(ProblemError::memory(
                "activity release matrix exceeds admitted storage allowance",
            ));
        }
        let mut matrix = (*self.source.matrix).clone();
        let stiffness = matrix.values.iter().fold(1_f64, |a, v| a.max(v.abs()));
        for column in 0..matrix.n {
            for slot in matrix.col_ptr[column]..matrix.col_ptr[column + 1] {
                let row = matrix.row_idx[slot];
                if released.binary_search(&row).is_ok() || released.binary_search(&column).is_ok() {
                    matrix.values[slot] = if row == column { -1. } else { 0. };
                }
                if row == column && pinned.binary_search(&row).is_ok() {
                    matrix.values[slot] += stiffness;
                }
            }
        }
        // Replace, rather than accumulate, mutable released systems.
        self.state.cache.borrow_mut().take();
        self.state.refactorizations.set(count + 1);
        let mut solver = feral::Solver::new().with_parallel(false);
        match solver.factor(&matrix, None) {
            feral::FactorStatus::Success | feral::FactorStatus::WrongInertia { .. } => {}
            feral::FactorStatus::Singular => return Ok(None),
            feral::FactorStatus::FatalError(error) => {
                return Err(crate::conditioning::native(error));
            }
        }
        let inertia = solver
            .inertia()
            .ok_or_else(|| ProblemError::internal("activity factor omitted inertia"))?;
        if inertia.total() != matrix.n {
            return Err(ProblemError::internal("activity factor inertia extent"));
        }
        if inertia.zero > 0 {
            return Ok(None);
        }
        self.execution.check()?;
        let factor = KktFactor {
            solver: Arc::new(solver),
            matrix: Arc::new(matrix),
            layout: self.source.layout.clone(),
            scales: self.source.scales.clone(),
            objective: self.source.objective,
            bound_rows: self.source.bound_rows.clone(),
        };
        let bytes = factor.bytes();
        self.state.bytes.set(self.state.bytes.get().max(bytes));
        if bytes > self.limits.bytes {
            return Err(ProblemError::memory(
                "activity release factor exceeds admitted storage allowance",
            ));
        }
        *self.state.cache.borrow_mut() = Some(ReleasedFactor {
            released,
            pinned,
            factor: factor.clone(),
        });
        Ok(Some(factor))
    }
}
impl SensBacksolver for Factor {
    fn dim(&self) -> usize {
        self.source.dim()
    }
    fn solve(&self, rhs: &[f64], lhs: &mut [f64]) -> bool {
        self.solve_with(&self.source, rhs, lhs)
    }
    fn bound_rows(&self) -> Option<&[BoundRow]> {
        Some(&self.bounds)
    }
    fn supports_release(&self) -> bool {
        true
    }
    fn solve_released(&self, released: &[usize], rhs: &[f64], lhs: &mut [f64]) -> bool {
        if !self.checkpoint() {
            return false;
        }
        match self.released(released, &[]) {
            Ok(Some(f)) => self.solve_with(&f, rhs, lhs),
            Ok(None) => false,
            Err(error) => self.stop(error),
        }
    }
    fn solve_released_step(&self, released: &[usize], rhs: &[f64], lhs: &mut [f64]) -> bool {
        // This zero-barrier active-set operator has no sigma/multiplier RHS shift.
        // RowLimitView supplies its own observer shift when requested.
        self.solve_released(released, rhs, lhs)
    }
    fn solve_released_pinned(
        &self,
        released: &[usize],
        pinned: &[usize],
        rhs: &[f64],
        lhs: &mut [f64],
    ) -> bool {
        if !self.checkpoint() {
            return false;
        }
        match self.released(released, pinned) {
            Ok(Some(f)) => self.solve_with(&f, rhs, lhs),
            Ok(None) => false,
            Err(error) => self.stop(error),
        }
    }
}
