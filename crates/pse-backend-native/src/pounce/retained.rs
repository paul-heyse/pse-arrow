// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exclusive, worker-owned leases of actual FERAL backends. Returning a lease keeps
//! library symbolic state; every new application must refill/refactor its numeric matrix.
use pounce_common::types::{Index, Number};
use pounce_feral::{FeralConfig, FeralSolverInterface};
use pounce_linsol::{
    EMatrixFormat, ESymSolverStatus, FactorPattern, SparseSymLinearSolverInterface,
    summary::LinearSolverSummary,
};
use pounce_rs::pounce_algorithm::alg_builder::LinearBackendFactory;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Phase {
    Main,
    Restoration,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Role {
    phase: Phase,
    ordinal: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Counts {
    pub created: u64,
    pub reused: u64,
    pub pattern_kept: u64,
    pub structure_refreshes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pattern {
    dim: Index,
    ia: Vec<Index>,
    ja: Vec<Index>,
}
struct Held {
    #[cfg(test)]
    id: u64,
    backend: FeralSolverInterface,
    pattern: Option<Pattern>,
}
pub(super) struct Pool {
    config: FeralConfig,
    identity: serde_json::Value,
    idle: BTreeMap<Role, Vec<Held>>,
    counts: Counts,
    active: usize,
}
impl Pool {
    pub(super) fn new(config: FeralConfig, identity: serde_json::Value) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            config,
            identity,
            idle: BTreeMap::new(),
            counts: Counts::default(),
            active: 0,
        }))
    }
    pub(super) fn matches(&self, identity: &serde_json::Value) -> bool {
        &self.identity == identity
    }
    pub(super) fn counts(&self) -> Counts {
        self.counts
    }
    /// Observable owned pattern buffers only. Full opaque FERAL allocations must be
    /// charged using the finite foreign allowance retained by the execution owner.
    pub(super) fn layout_bytes(&self) -> usize {
        self.idle
            .values()
            .flatten()
            .fold(size_of::<Self>(), |bytes, held| {
                bytes.saturating_add(size_of::<Held>()).saturating_add(
                    held.pattern.as_ref().map_or(0, |p| {
                        (p.ia.capacity() + p.ja.capacity()).saturating_mul(size_of::<Index>())
                    }),
                )
            })
    }
    fn checkout(
        pool: &Rc<RefCell<Self>>,
        role: Role,
        sink: Arc<Mutex<LinearSolverSummary>>,
    ) -> Lease {
        let held = {
            let mut owner = pool.borrow_mut();
            let held = owner.idle.get_mut(&role).and_then(Vec::pop);
            let held = if let Some(held) = held {
                owner.counts.reused += 1;
                held
            } else {
                owner.counts.created += 1;
                Held {
                    #[cfg(test)]
                    id: owner.counts.created,
                    backend: FeralSolverInterface::with_config(owner.config.clone()),
                    pattern: None,
                }
            };
            owner.active += 1;
            held
        };
        Lease {
            held: Some(held),
            pool: pool.clone(),
            role,
            sink,
            new_application: true,
            healthy: true,
            quality_changed: false,
        }
    }
}
pub(super) fn factory(
    pool: Rc<RefCell<Pool>>,
    sink: Arc<Mutex<LinearSolverSummary>>,
    phase: Phase,
) -> LinearBackendFactory {
    let mut ordinal = 0;
    Box::new(move |_choice| {
        // The enclosing adapter admits and reserves linear_solver=feral before building.
        // This infallible library factory constructs only that admitted backend.
        // Pinned AlgorithmBuilder asks main first and Hessian-bypass second. Further
        // requests get their own ordinal; simultaneous leases never share a mutable factor.
        let role = Role { phase, ordinal };
        ordinal += 1;
        Box::new(Pool::checkout(&pool, role, sink.clone()))
    })
}
struct Lease {
    held: Option<Held>,
    pool: Rc<RefCell<Pool>>,
    role: Role,
    sink: Arc<Mutex<LinearSolverSummary>>,
    new_application: bool,
    healthy: bool,
    quality_changed: bool,
}
impl Lease {
    #[allow(
        clippy::expect_used,
        reason = "Only Drop takes the owned backend; library calls occur while the exclusive lease is live"
    )]
    fn held(&self) -> &Held {
        self.held.as_ref().expect("live FERAL lease")
    }
    #[allow(
        clippy::expect_used,
        reason = "Only Drop takes the owned backend; library calls occur while the exclusive lease is live"
    )]
    fn held_mut(&mut self) -> &mut Held {
        self.held.as_mut().expect("live FERAL lease")
    }
    fn record(&self, before: LinearSolverSummary, after: LinearSolverSummary) {
        let factors = after.n_factors.saturating_sub(before.n_factors);
        if factors == 0 {
            return;
        }
        if let Ok(mut sink) = self.sink.lock() {
            sink.solver_name = "feral".into();
            sink.n_factors += factors;
            sink.n_pattern_reuse += after.n_pattern_reuse.saturating_sub(before.n_pattern_reuse);
            sink.n_pattern_changes += after
                .n_pattern_changes
                .saturating_sub(before.n_pattern_changes);
            sink.last_inertia = after.last_inertia;
            sink.last_nnz_a = after.last_nnz_a;
            sink.last_nnz_l = after.last_nnz_l;
            // Pinned FERAL exposes lifetime extrema only. They cannot be reset without
            // replacing the solver and cannot truthfully become this attempt's extrema.
        }
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut owner = self.pool.borrow_mut();
        owner.active -= 1;
        // The pinned interface has no operation to restore construction-time pivot
        // settings after quality escalation. Do not carry that altered profile into
        // a later attempt claiming the originally requested effective settings.
        if self.healthy
            && !self.quality_changed
            && let Some(held) = self.held.take()
        {
            owner.idle.entry(self.role).or_default().push(held);
        }
    }
}
impl SparseSymLinearSolverInterface for Lease {
    fn initialize_structure(
        &mut self,
        dim: Index,
        nonzeros: Index,
        ia: &[Index],
        ja: &[Index],
    ) -> ESymSolverStatus {
        if nonzeros < 0 || dim < 0 || ia.len() != nonzeros as usize || ja.len() != nonzeros as usize
        {
            self.healthy = false;
            return ESymSolverStatus::FatalError;
        }
        let unchanged = self
            .held()
            .pattern
            .as_ref()
            .is_some_and(|p| p.dim == dim && p.ia == ia && p.ja == ja);
        self.new_application = true;
        if unchanged {
            self.pool.borrow_mut().counts.pattern_kept += 1;
            self.held_mut().backend.values_array_mut().fill(0.0);
            return ESymSolverStatus::Success;
        }
        self.pool.borrow_mut().counts.structure_refreshes += 1;
        let status = self
            .held_mut()
            .backend
            .initialize_structure(dim, nonzeros, ia, ja);
        if status == ESymSolverStatus::Success {
            self.held_mut().pattern = Some(Pattern {
                dim,
                ia: ia.to_vec(),
                ja: ja.to_vec(),
            });
        } else {
            self.healthy = false;
        }
        status
    }
    fn values_array_mut(&mut self) -> &mut [Number] {
        self.held_mut().backend.values_array_mut()
    }
    fn multi_solve(
        &mut self,
        new_matrix: bool,
        ia: &[Index],
        ja: &[Index],
        nrhs: Index,
        rhs_vals: &mut [Number],
        check_neg_evals: bool,
        number_of_neg_evals: Index,
    ) -> ESymSolverStatus {
        let before = self.held().backend.summary();
        let fresh = new_matrix || self.new_application;
        let status = self.held_mut().backend.multi_solve(
            fresh,
            ia,
            ja,
            nrhs,
            rhs_vals,
            check_neg_evals,
            number_of_neg_evals,
        );
        let after = self.held().backend.summary();
        self.record(before, after);
        if status == ESymSolverStatus::Success {
            self.new_application = false;
        } else if status == ESymSolverStatus::FatalError {
            self.healthy = false;
        }
        status
    }
    fn number_of_neg_evals(&self) -> Index {
        self.held().backend.number_of_neg_evals()
    }
    fn increase_quality(&mut self) -> bool {
        let changed = self.held_mut().backend.increase_quality();
        self.quality_changed |= changed;
        changed
    }
    fn provides_inertia(&self) -> bool {
        self.held().backend.provides_inertia()
    }
    fn multi_solve_matches_single_solve(&self, nrhs: usize) -> bool {
        self.held().backend.multi_solve_matches_single_solve(nrhs)
    }
    fn matrix_format(&self) -> EMatrixFormat {
        self.held().backend.matrix_format()
    }
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        self.held().backend.as_any()
    }
    fn provides_degeneracy_detection(&self) -> bool {
        self.held().backend.provides_degeneracy_detection()
    }
    fn determine_dependent_rows(
        &mut self,
        n_rows: Index,
        n_cols: Index,
        irn: &[Index],
        jcn: &[Index],
        vals: &[Number],
        c_deps: &mut Vec<Index>,
    ) -> ESymSolverStatus {
        self.held_mut()
            .backend
            .determine_dependent_rows(n_rows, n_cols, irn, jcn, vals, c_deps)
    }
    fn factor_pattern(&self, want_values: bool) -> Option<FactorPattern> {
        self.held().backend.factor_pattern(want_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn factor(lease: &mut Lease, matrix: [f64; 3]) {
        assert_eq!(
            lease.initialize_structure(2, 3, &[1, 2, 2], &[1, 1, 2]),
            ESymSolverStatus::Success
        );
        lease.values_array_mut().copy_from_slice(&matrix);
        let mut rhs = [1.0, 2.0];
        assert_eq!(
            lease.multi_solve(true, &[1, 2, 2], &[1, 1, 2], 1, &mut rhs, false, 0),
            ESymSolverStatus::Success
        );
        assert!((matrix[0] * rhs[0] + matrix[1] * rhs[1] - 1.0).abs() < 1e-10);
        assert!((matrix[1] * rhs[0] + matrix[2] * rhs[1] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn retained_actual_feral_solver_reuses_symbolic_factor_and_resets_attempt_counts() {
        let pool = Pool::new(
            FeralConfig {
                parallel: Some(false),
                ..FeralConfig::default()
            },
            serde_json::Value::Null,
        );
        let role = Role {
            phase: Phase::Main,
            ordinal: 0,
        };
        let first_sink = Arc::new(Mutex::new(LinearSolverSummary::default()));
        let mut first = Pool::checkout(&pool, role, first_sink.clone());
        let id = first.held().id;
        factor(&mut first, [4.0, 1.0, 3.0]);
        assert_eq!(first_sink.lock().unwrap().n_factors, 1);
        drop(first);
        let second_sink = Arc::new(Mutex::new(LinearSolverSummary::default()));
        let mut second = Pool::checkout(&pool, role, second_sink.clone());
        assert_eq!(second.held().id, id);
        factor(&mut second, [5.0, 1.0, 4.0]);
        let summary = second_sink.lock().unwrap();
        assert_eq!(summary.n_factors, 1);
        assert_eq!(summary.n_pattern_reuse, 1);
        assert_eq!(summary.n_pattern_changes, 0);
        assert!(summary.min_abs_pivot.is_none());
        drop(summary);
        drop(second);
        assert_eq!(pool.borrow().counts().created, 1);
        assert_eq!(pool.borrow().counts().reused, 1);
        assert_eq!(pool.borrow().counts().pattern_kept, 1);
        assert!(pool.borrow().layout_bytes() > size_of::<Pool>());
    }
    #[test]
    fn main_restoration_bypass_and_simultaneous_same_role_get_distinct_mutable_factors() {
        let pool = Pool::new(
            FeralConfig {
                parallel: Some(false),
                ..FeralConfig::default()
            },
            serde_json::Value::Null,
        );
        let sink = Arc::new(Mutex::new(LinearSolverSummary::default()));
        let mut main = Pool::checkout(
            &pool,
            Role {
                phase: Phase::Main,
                ordinal: 0,
            },
            sink.clone(),
        );
        let mut bypass = Pool::checkout(
            &pool,
            Role {
                phase: Phase::Main,
                ordinal: 1,
            },
            sink.clone(),
        );
        let mut restoration = Pool::checkout(
            &pool,
            Role {
                phase: Phase::Restoration,
                ordinal: 0,
            },
            sink.clone(),
        );
        let duplicate = Pool::checkout(
            &pool,
            Role {
                phase: Phase::Main,
                ordinal: 0,
            },
            sink,
        );
        assert_ne!(main.held().id, bypass.held().id);
        assert_ne!(main.held().id, restoration.held().id);
        assert_ne!(main.held().id, duplicate.held().id);
        factor(&mut main, [4.0, 1.0, 3.0]);
        factor(&mut bypass, [5.0, 0.0, 2.0]);
        factor(&mut restoration, [3.0, 0.0, 7.0]);
        assert_eq!(pool.borrow().active, 4);
        drop((main, bypass, restoration, duplicate));
        assert_eq!(pool.borrow().active, 0);
        assert_eq!(pool.borrow().counts().created, 4);
    }
    #[test]
    fn changed_actual_pattern_refreshes_structure_without_using_previous_numeric_factor() {
        let pool = Pool::new(
            FeralConfig {
                parallel: Some(false),
                ..FeralConfig::default()
            },
            serde_json::Value::Null,
        );
        let sink = Arc::new(Mutex::new(LinearSolverSummary::default()));
        let role = Role {
            phase: Phase::Main,
            ordinal: 0,
        };
        let mut lease = Pool::checkout(&pool, role, sink.clone());
        factor(&mut lease, [4.0, 1.0, 3.0]);
        drop(lease);
        let mut lease = Pool::checkout(&pool, role, sink.clone());
        assert_eq!(
            lease.initialize_structure(2, 2, &[1, 2], &[1, 2]),
            ESymSolverStatus::Success
        );
        lease.values_array_mut().copy_from_slice(&[2.0, 4.0]);
        let mut rhs = [2.0, 8.0];
        // Even a caller's first new_matrix=false cannot borrow the preceding numeric factor.
        assert_eq!(
            lease.multi_solve(false, &[1, 2], &[1, 2], 1, &mut rhs, false, 0),
            ESymSolverStatus::Success
        );
        assert_eq!(rhs, [1.0, 2.0]);
        assert_eq!(sink.lock().unwrap().n_pattern_changes, 2);
    }
}
