// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library BTF diagonal blocks and library sparse LU, used only as a right preconditioner.
use super::{Function, ProblemError};
use crate::solve::Execution;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct Block {
    rows: Vec<usize>,
    columns: Vec<usize>,
    // Canonical local CSC entries map to original coefficient ordinals. Explicit zeros
    // remain stored, so numeric zero crossings cannot change the retained pattern.
    coefficients: Vec<usize>,
    matrix: feral::SparseColMatrix,
    rhs: Vec<f64>,
    factor: Option<feral::SparseLu>,
}
/// Immutable semantic projection plus mutable factors owned by one KINSOL session.
#[derive(Debug)]
pub(super) struct Blocks {
    blocks: Vec<Block>,
    result: Vec<f64>,
    retained_allowance: usize,
    pub(super) factor_calls: u64,
    pub(super) factors_completed: u64,
    pub(super) solve_calls: u64,
}
fn checked_extent(n: usize) -> Result<usize, ProblemError> {
    // FERAL 0.18 Markowitz keeps sparse row/column projections, LU/CSR copies and
    // length-n work vectors. Its row_cols memberships and lazy heaps can retain
    // historical fill entries: at most n pivot rounds x n rows x n column updates.
    // The cubic term covers those entries, heap pairs and growing capacities;
    // the quadratic term covers full live fill and simultaneous factor scratch.
    // No FT updates or sparse/hyper solves are used here.
    n.checked_mul(n)
        .and_then(|square| {
            square.checked_mul(2048).and_then(|live| {
                square
                    .checked_mul(n)
                    .and_then(|history| history.checked_mul(256))
                    .and_then(|history| live.checked_add(history))
            })
        })
        .and_then(|bytes| {
            n.checked_mul(4096)
                .and_then(|linear| bytes.checked_add(linear))
        })
        .and_then(|bytes| bytes.checked_add(size_of::<Block>() + size_of::<feral::SparseLu>()))
        .ok_or_else(|| ProblemError::memory("block LU storage extent"))
}
impl Blocks {
    pub(super) fn admit(
        function: &Function,
        execution: &Execution,
        linear: super::Linear,
    ) -> Result<Self, ProblemError> {
        execution.check()?;
        let Function::Equations(oracle) = function else {
            return Err(ProblemError::Unsupported(
                "block factor preconditioning needs original analytic equation Jacobians".into(),
            ));
        };
        let contract = oracle.contract();
        let n = contract.variables.len();
        let pattern = oracle.jacobian_pattern();
        if linear.krylov().is_none() {
            return Err(ProblemError::Unsupported(
                "block factor requires a native Krylov route".into(),
            ));
        }
        let native_workspace = super::native_krylov_storage(n, linear)?;
        let memory = execution.memory.filter(|bytes| *bytes > 0).ok_or_else(|| {
            ProblemError::memory("block factor preconditioner needs finite admitted storage")
        })?;
        let source = oracle.structural_analysis();
        let factor_plan = match source {
            Some(analysis) => analysis.blocks.iter().try_fold(0usize, |bytes, block| {
                bytes
                    .checked_add(checked_extent(block.members.columns.len())?)
                    .ok_or_else(|| ProblemError::memory("block factor aggregate extent"))
            })?,
            None => checked_extent(n)?,
        };
        let projection = crate::structural::construction_bytes(contract, pattern)?;
        // A supplied witness avoids another matching allocation. Its linear checking
        // and the mechanical maps still need bounded storage; fresh analysis includes
        // the owner's qualified matching stack in construction_bytes.
        let construction = if source.is_some() {
            projection.saturating_sub(pse_structural::incidence::MATCHING_STACK)
        } else {
            projection
        };
        let scratch_bytes = n
            .checked_mul(2 * size_of::<f64>())
            .ok_or_else(|| ProblemError::memory("block application storage extent"))?;
        let factor_plan = factor_plan
            .checked_add(scratch_bytes)
            .ok_or_else(|| ProblemError::memory("block retained storage extent"))?;
        let required = factor_plan
            .checked_add(construction)
            .and_then(|bytes| bytes.checked_add(native_workspace))
            .ok_or_else(|| ProblemError::memory("block factor admission extent"))?;
        if required > memory {
            return Err(ProblemError::memory(
                "block factor plan exceeds admitted storage",
            ));
        }
        let bounds = vec![(0., 0.); n];
        let generated = if source.is_none() {
            Some(crate::structural::oracle_structure_with_cancel(
                contract,
                pattern,
                &bounds,
                false,
                &execution.cancel,
            )?)
        } else {
            None
        };
        let analysis = source
            .or_else(|| generated.as_ref().map(|structure| &*structure.witness))
            .ok_or_else(|| ProblemError::Internal("block factor matching unavailable".into()))?;
        crate::structural::check(
            contract,
            pattern,
            &bounds,
            crate::structural::Mode::Roots,
            Some(analysis),
        )?;
        if analysis.blocks.is_empty() {
            return Err(ProblemError::Contract(
                "block factor matching has no complete blocks".into(),
            ));
        }
        let rows: BTreeMap<_, _> = contract
            .rows
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        let columns: BTreeMap<_, _> = contract
            .variables
            .iter()
            .enumerate()
            .map(|(index, v)| (v.id, index))
            .collect();
        let mut covered_rows = BTreeSet::new();
        let mut covered_columns = BTreeSet::new();
        let mut row_block = vec![usize::MAX; n];
        let mut column_block = vec![usize::MAX; n];
        let mut blocks = Vec::with_capacity(analysis.blocks.len());
        for (ordinal, block) in analysis.blocks.iter().enumerate() {
            let members = &block.members;
            if members.rows.is_empty() || members.rows.len() != members.columns.len() {
                return Err(ProblemError::Contract(
                    "block factor diagonal is not square".into(),
                ));
            }
            let row_slots = members
                .rows
                .iter()
                .map(|id| {
                    rows.get(id)
                        .copied()
                        .filter(|slot| covered_rows.insert(*slot))
                        .ok_or_else(|| ProblemError::Contract("block factor row coverage".into()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let column_slots = members
                .columns
                .iter()
                .map(|id| {
                    columns
                        .get(id)
                        .copied()
                        .filter(|slot| covered_columns.insert(*slot))
                        .ok_or_else(|| {
                            ProblemError::Contract("block factor column coverage".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            for &row in &row_slots {
                row_block[row] = ordinal;
            }
            for &column in &column_slots {
                column_block[column] = ordinal;
            }
            let local_rows: BTreeMap<_, _> = row_slots
                .iter()
                .enumerate()
                .map(|(local, original)| (*original, local))
                .collect();
            let mut col_ptr = Vec::with_capacity(column_slots.len() + 1);
            let mut row_idx = Vec::new();
            let mut coefficients = Vec::new();
            col_ptr.push(0);
            for &column in &column_slots {
                let mut entries = pattern
                    .col_range(column)
                    .filter_map(|index| {
                        local_rows
                            .get(&pattern.row_idx()[index])
                            .map(|&row| (row, index))
                    })
                    .collect::<Vec<_>>();
                entries.sort_unstable_by_key(|&(row, _)| row);
                for (row, index) in entries {
                    row_idx.push(row);
                    coefficients.push(index);
                }
                col_ptr.push(row_idx.len());
            }
            let matrix = feral::SparseColMatrix {
                m: row_slots.len(),
                col_ptr,
                row_idx,
                values: vec![0.0; coefficients.len()],
            };
            matrix.validate().map_err(crate::conditioning::native)?;
            blocks.push(Block {
                rhs: vec![0.; row_slots.len()],
                rows: row_slots,
                columns: column_slots,
                coefficients,
                matrix,
                factor: None,
            });
        }
        if covered_rows.len() != n || covered_columns.len() != n {
            return Err(ProblemError::Contract(
                "block factor matching omits original coordinates".into(),
            ));
        }
        // Verify the retained prerequisite-first BTF against actual Jacobian support.
        // Off-block lower couplings remain in the true JVP; they are not solved here.
        for (column, &block) in column_block.iter().enumerate() {
            for row in pattern.row_idx_of_col(column) {
                if block > row_block[row] {
                    return Err(ProblemError::Contract(
                        "block factor witness contradicts original BTF support".into(),
                    ));
                }
            }
        }
        execution.check()?;
        Ok(Self {
            blocks,
            result: vec![0.; n],
            retained_allowance: factor_plan,
            factor_calls: 0,
            factors_completed: 0,
            solve_calls: 0,
        })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        // FERAL does not expose allocation capacities; retain the admitted full-fill
        // allowance rather than report factor_nnz as allocated storage.
        self.retained_allowance.saturating_add(size_of::<Self>())
    }
    pub(super) fn reset_counts(&mut self) {
        self.factor_calls = 0;
        self.factors_completed = 0;
        self.solve_calls = 0;
    }
    pub(super) fn setup(
        &mut self,
        values: &[f64],
        execution: &Execution,
    ) -> Result<(), ProblemError> {
        for block in &mut self.blocks {
            block.factor = None;
        }
        let result = (|| {
            for block in &mut self.blocks {
                execution.check()?;
                for (slot, &index) in block.matrix.values.iter_mut().zip(&block.coefficients) {
                    *slot = values
                        .get(index)
                        .copied()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| {
                            ProblemError::numerical("block factor nonfinite Jacobian coefficient")
                        })?;
                }
                self.factor_calls = self
                    .factor_calls
                    .checked_add(1)
                    .ok_or_else(|| ProblemError::Internal("block factor count overflow".into()))?;
                let factored = crate::observation::scoped("kinsol.block.feral.factor", || {
                    feral::SparseLu::factor_markowitz(&block.matrix, feral::LuParams::default())
                });
                crate::observation::count(
                    if factored.is_ok() {
                        "kinsol.block.feral.factor.succeeded"
                    } else {
                        "kinsol.block.feral.factor.failed"
                    },
                    1,
                );
                let factor = factored.map_err(crate::conditioning::native)?;
                self.factors_completed =
                    self.factors_completed.checked_add(1).ok_or_else(|| {
                        ProblemError::Internal("block successful factor count overflow".into())
                    })?;
                execution.check()?;
                block.factor = Some(factor);
            }
            Ok(())
        })();
        if result.is_err() {
            for block in &mut self.blocks {
                block.factor = None;
            }
        }
        result
    }
    pub(super) fn solve(
        &mut self,
        rhs: &[f64],
        execution: &Execution,
    ) -> Result<&[f64], ProblemError> {
        if rhs.len() != self.result.len() || rhs.iter().any(|value| !value.is_finite()) {
            return Err(ProblemError::Contract(
                "block preconditioner right-hand side extent or values".into(),
            ));
        }
        self.result.fill(0.);
        for block in &mut self.blocks {
            execution.check()?;
            let factor = block.factor.as_mut().ok_or_else(|| {
                ProblemError::Internal("block solve lacks completed factor setup".into())
            })?;
            for (slot, &row) in block.rhs.iter_mut().zip(&block.rows) {
                *slot = rhs[row];
            }
            self.solve_calls = self
                .solve_calls
                .checked_add(1)
                .ok_or_else(|| ProblemError::Internal("block solve count overflow".into()))?;
            let solved = crate::observation::scoped("kinsol.block.feral.solve", || {
                factor.ftran(&mut block.rhs)
            });
            crate::observation::count(
                if solved.is_ok() {
                    "kinsol.block.feral.solve.succeeded"
                } else {
                    "kinsol.block.feral.solve.failed"
                },
                1,
            );
            solved.map_err(crate::conditioning::native)?;
            execution.check()?;
            if block.rhs.iter().any(|value| !value.is_finite()) {
                return Err(ProblemError::numerical(
                    "block preconditioner nonfinite solution",
                ));
            }
            for (&column, value) in block.columns.iter().zip(&block.rhs) {
                self.result[column] = *value;
            }
        }
        Ok(&self.result)
    }
    pub(super) fn count(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NleOracle, OracleContract, Variable, solve::*};
    use pse_kernels::DerivativeOrder;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    #[derive(Debug)]
    pub(super) struct Coupled {
        contract: OracleContract,
        matrix: faer::sparse::SparseColMat<usize, f64>,
        products: Arc<AtomicU64>,
        changed: bool,
        singular: bool,
    }
    impl Coupled {
        pub(super) fn new(products: Arc<AtomicU64>, changed: bool, singular: bool) -> Self {
            let mut entries = vec![(0, 0), (1, 0), (0, 1), (1, 1), (2, 1), (2, 2)];
            if changed {
                entries.push((0, 2));
            }
            let triplets = entries
                .into_iter()
                .map(|(row, column)| faer::sparse::Triplet::new(row, column, 1.))
                .collect::<Vec<_>>();
            Self {
                contract: OracleContract {
                    identity: pse_ids::ContentHash::from_bytes([61; 32]),
                    variables: (1..=3)
                        .map(|id| Variable {
                            id: crate::solver_tests::id(id),
                            lower: f64::NEG_INFINITY,
                            upper: f64::INFINITY,
                        })
                        .collect(),
                    rows: (20..=22).map(crate::solver_tests::id).collect(),
                    derivatives: DerivativeOrder::First,
                    smoothness: DerivativeOrder::First,
                },
                matrix: faer::sparse::SparseColMat::try_new_from_triplets(3, 3, &triplets).unwrap(),
                products,
                changed,
                singular,
            }
        }
    }
    impl NleOracle for Coupled {
        fn contract(&self) -> &OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.matrix.symbolic()
        }
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            if self.singular {
                out.fill(1.);
            } else {
                out[0] = 0.5 * x[0] * x[0] + 2. * x[1] - 2.;
                out[1] = 3. * x[0] + x[1] - 1.;
                out[2] = 4. * x[1] + 5. * x[2] - 14.;
            }
            Ok(())
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            let mut index = 0;
            for column in 0..3 {
                for row in self.matrix.symbolic().row_idx_of_col(column) {
                    out[index] = if self.singular {
                        0.
                    } else {
                        match (row, column) {
                            (0, 0) => x[0],
                            (1, 0) => 3.,
                            (0, 1) => 2.,
                            (1, 1) => 1.,
                            (2, 1) => 4.,
                            (2, 2) => 5.,
                            (0, 2) if self.changed => 0.,
                            _ => {
                                return Err(ProblemError::Internal(
                                    "fixture Jacobian support mismatch".into(),
                                ));
                            }
                        }
                    };
                    index += 1;
                }
            }
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            self.products.fetch_add(1, Ordering::Relaxed);
            if self.singular {
                out.fill(0.);
            } else {
                out[0] = x[0] * v[0] + 2. * v[1];
                out[1] = 3. * v[0] + v[1];
                out[2] = 4. * v[1] + 5. * v[2];
            }
            Ok(())
        }
    }
    fn linear() -> super::super::Linear {
        super::super::Linear::Spgmr {
            dimension: pse_model::scalars::PositiveCount::try_new(3).unwrap(),
        }
    }
    fn execution() -> Execution {
        let mut execution = crate::solver_tests::execution();
        execution.memory = Some(128 << 20);
        execution
    }
    #[test]
    fn fixed_csc_refill_preserves_storage_zero_crossings_and_invalidates_failed_setup() {
        let function = Function::Equations(Box::new(Coupled::new(Arc::default(), false, false)));
        let execution = execution();
        let mut blocks = Blocks::admit(&function, &execution, linear()).unwrap();
        let storage = blocks
            .blocks
            .iter()
            .map(|b| {
                (
                    b.matrix.col_ptr.as_ptr(),
                    b.matrix.row_idx.as_ptr(),
                    b.matrix.values.as_ptr(),
                )
            })
            .collect::<Vec<_>>();
        // The first entry crosses zero and Markowitz selects pivots afresh each time.
        for first in [0.0, 2.0, -1.0] {
            blocks
                .setup(&[first, 3., 2., 1., 4., 5.], &execution)
                .unwrap();
            // Independent block RHS for x = [2,-1,3]. The third row deliberately
            // excludes the true off-block coupling, as the right preconditioner must.
            let x = blocks
                .solve(&[first * 2. - 2., 5., 15.], &execution)
                .unwrap();
            for (actual, expected) in x.iter().zip([2., -1., 3.]) {
                assert!((actual - expected).abs() < 1e-12);
            }
            for (block, pointers) in blocks.blocks.iter().zip(&storage) {
                assert_eq!(
                    (
                        block.matrix.col_ptr.as_ptr(),
                        block.matrix.row_idx.as_ptr(),
                        block.matrix.values.as_ptr()
                    ),
                    *pointers
                );
            }
        }
        // Failure in a later block must also discard earlier newly completed factors.
        assert!(
            blocks
                .setup(&[2., 3., 2., 1., 4., f64::NAN], &execution)
                .is_err()
        );
        assert!(blocks.blocks.iter().all(|block| block.factor.is_none()));
        assert!(blocks.solve(&[1.; 3], &execution).is_err());
        blocks.setup(&[0., 3., 2., 1., 4., 5.], &execution).unwrap();
        assert!(blocks.solve(&[2., 1., 14.], &execution).is_ok());
    }
    #[test]
    fn retained_block_application_buffers_survive_repeats_failure_and_recovery() {
        let function = Function::Equations(Box::new(Coupled::new(
            Arc::new(AtomicU64::new(0)),
            false,
            false,
        )));
        let execution = execution();
        let mut blocks = Blocks::admit(&function, &execution, linear()).unwrap();
        blocks.setup(&[0., 3., 2., 1., 4., 5.], &execution).unwrap();
        let global = blocks.result.as_ptr();
        let local = blocks
            .blocks
            .iter()
            .map(|b| b.rhs.as_ptr())
            .collect::<Vec<_>>();
        for _ in 0..32 {
            assert_eq!(
                blocks.solve(&[2., 1., 14.], &execution).unwrap(),
                &[0., 1., 2.8]
            );
            assert_eq!(blocks.result.as_ptr(), global);
            assert_eq!(
                blocks
                    .blocks
                    .iter()
                    .map(|b| b.rhs.as_ptr())
                    .collect::<Vec<_>>(),
                local
            );
        }
        assert!(blocks.solve(&[1.; 2], &execution).is_err());
        assert!(blocks.setup(&[0.; 6], &execution).is_err());
        assert!(blocks.solve(&[2., 1., 14.], &execution).is_err());
        blocks.setup(&[0., 3., 2., 1., 4., 5.], &execution).unwrap();
        assert!(blocks.solve(&[2., 1., 14.], &execution).is_ok());
        let stopped = execution.clone();
        stopped.cancel.store(true, Ordering::Relaxed);
        assert!(blocks.solve(&[2., 1., 14.], &stopped).is_err());
        stopped.cancel.store(false, Ordering::Relaxed);
        assert!(blocks.solve(&[2., 1., 14.], &execution).is_ok());
        assert_eq!(blocks.result.as_ptr(), global);
    }
    #[test]
    fn repeated_block_applies_allocate_zero_after_native_scratch_warmup() {
        let function = Function::Equations(Box::new(Coupled::new(Arc::default(), false, false)));
        let execution = execution();
        let mut blocks = Blocks::admit(&function, &execution, linear()).unwrap();
        blocks.setup(&[0., 3., 2., 1., 4., 5.], &execution).unwrap();
        blocks.solve(&[2., 1., 14.], &execution).unwrap();
        let measured = allocation_counter::measure(|| {
            for _ in 0..32 {
                std::hint::black_box(blocks.solve(&[2., 1., 14.], &execution).unwrap());
            }
        });
        assert_eq!(
            measured.count_total, 0,
            "application only; setup/factors excluded: {measured:?}"
        );
        assert_eq!(measured.bytes_total, 0);
    }
    #[test]
    fn library_btf_factor_action_keeps_coupling_in_true_product_and_refuses_unowned_storage() {
        let products = Arc::new(AtomicU64::new(0));
        let function = Function::Equations(Box::new(Coupled::new(products.clone(), false, false)));
        let execution = execution();
        let mut blocks = Blocks::admit(&function, &execution, linear()).unwrap();
        assert_eq!(blocks.count(), 2);
        blocks.setup(&[0., 3., 2., 1., 4., 5.], &execution).unwrap();
        let action = blocks.solve(&[2., 1., 14.], &execution).unwrap();
        assert_eq!(action[0], 0.);
        assert_eq!(action[1], 1.);
        assert_eq!(action[2], 2.8);
        // The block action differs from Jacobi (which would leave rhs[0] == 2).
        // It also differs from the full solve because the original lower coupling
        // belongs to the actual Krylov product, not the preconditioner inverse.
        let Function::Equations(mut oracle) = function else {
            panic!("fixture must supply original equations")
        };
        let mut actual = vec![0.; 3];
        oracle
            .jacobian_product(&[0.; 3], &action, &mut actual)
            .unwrap();
        assert_eq!(actual, vec![2., 1., 18.]);
        assert_eq!(products.load(Ordering::Relaxed), 1);
        assert_eq!(blocks.factor_calls, 2);
        assert_eq!(blocks.factors_completed, 2);
        assert_eq!(blocks.solve_calls, 2);
        let function = Function::Equations(oracle);
        for memory in [None, Some(0), Some(1)] {
            let mut denied = execution.clone();
            denied.memory = memory;
            assert!(matches!(
                Blocks::admit(&function, &denied, linear()),
                Err(ProblemError::Limit {
                    kind: crate::LimitKind::Memory,
                    ..
                })
            ));
        }
        let enormous = super::super::Linear::Spfgmr {
            dimension: pse_model::scalars::PositiveCount::try_new(1_000_000).unwrap(),
        };
        assert!(matches!(
            Blocks::admit(&function, &execution, enormous),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        let overflow = super::super::Linear::Spgmr {
            dimension: pse_model::scalars::PositiveCount::try_new(usize::MAX).unwrap(),
        };
        assert!(matches!(
            Blocks::admit(&function, &execution, overflow),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        let contract = function.contract();
        let Function::Equations(oracle) = &function else {
            panic!("fixture must supply original equations")
        };
        let projection =
            crate::structural::construction_bytes(contract, oracle.jacobian_pattern()).unwrap();
        // A full-live-fill-only allowance excludes Markowitz's possible historical
        // cancellation/refill memberships and must be refused before any factor call.
        let former_live_fill =
            3 * 3 * 2048 + 3 * 4096 + size_of::<Block>() + size_of::<feral::SparseLu>();
        let mut insufficient_history = execution.clone();
        insufficient_history.memory = Some(projection + former_live_fill);
        assert!(matches!(
            Blocks::admit(&function, &insufficient_history, linear()),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        let failure = blocks.setup(&[0.; 6], &execution).unwrap_err();
        assert!(matches!(
            failure,
            ProblemError::Linear {
                kind: crate::LinearFailureKind::Numerical,
                ..
            }
        ));
        assert_eq!(blocks.factor_calls, 3);
        assert_eq!(blocks.factors_completed, 2);
        let error = blocks.solve(&[2., 1., 14.], &execution).unwrap_err();
        assert!(
            matches!(error, ProblemError::Internal(_)),
            "failed factors cannot silently apply Jacobi"
        );
        let mut stopped = execution.clone();
        stopped.time_limit = std::time::Duration::ZERO;
        assert!(matches!(
            Blocks::admit(&function, &stopped, linear()),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Time,
                ..
            })
        ));
        assert!(!execution.cancel.load(Ordering::Relaxed));
    }
    #[test]
    fn actual_kinsol_block_factors_solve_coupled_original_and_rebuild_changed_sources() {
        use super::super::{Linear, Method, Session, Settings, Strategy};
        let dimension = pse_model::scalars::PositiveCount::try_new(3).unwrap();
        for linear in [
            Linear::Spgmr { dimension },
            Linear::Spfgmr { dimension },
            Linear::Spbcgs { dimension },
            Linear::Sptfqmr { dimension },
        ] {
            let products = Arc::new(AtomicU64::new(0));
            let function =
                || Function::Equations(Box::new(Coupled::new(products.clone(), false, false)));
            let controls = Controls::default();
            let tolerance = crate::quality::Tolerances {
                variables: vec![1e-8; 3],
                rows: vec![1e-8; 3],
                integrality: 1e-8,
            };
            let normalization = pse_math::normalization::Normalization {
                variables: vec![1.; 3],
                rows: vec![1.; 3],
                objective: 1.,
            };
            let accuracy =
                ResolvedAccuracy::resolve(&Default::default(), &tolerance, &normalization).unwrap();
            let settings = Settings::from_policy(
                Method {
                    strategy: Strategy::LineSearch,
                    linear,
                    preconditioner: Preconditioner::BlockFactor,
                    setup_interval: 1,
                    ..Default::default()
                },
                &tolerance,
                &normalization,
                accuracy.feasibility,
            );
            let stamp = crate::solver_tests::stamp(Backend::Kinsol);
            let mut session =
                Session::new(function(), settings.clone(), execution(), stamp.clone()).unwrap();
            let mem = session.mem;
            for pass in 0..2 {
                if pass == 1 {
                    let mut changed = stamp.clone();
                    changed.data = pse_ids::ContentHash::from_bytes([62; 32]);
                    session
                        .replace(function(), settings.clone(), changed)
                        .unwrap();
                    assert_eq!(session.mem, mem);
                    assert!(session.callback.blocks.is_none());
                }
                let report = session
                    .solve(
                        &[0.; 3],
                        &controls,
                        &accuracy,
                        execution(),
                        &tolerance,
                        None,
                    )
                    .unwrap();
                assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
                for (actual, expected) in report
                    .candidate
                    .as_ref()
                    .unwrap()
                    .primal
                    .iter()
                    .zip([0., 1., 2.])
                {
                    assert!((actual - expected).abs() < 1e-7, "{report:?}");
                }
                assert!(products.load(Ordering::Relaxed) > 0);
                assert_eq!(
                    report.metrics["preconditioner.block.count"],
                    Metric::Integer(2)
                );
                assert!(
                    matches!(report.metrics["preconditioner.block.factor_calls"],Metric::Integer(n) if n>=2)
                );
                assert!(
                    matches!(report.metrics["preconditioner.block.solve_calls"],Metric::Integer(n) if n>=2)
                );
                assert_eq!(report.evidence.work.factorizations, None);
                // Escaping retained ownership includes the cloned native vector headers,
                // separately from queried payload words and library factor ownership.
                assert!(
                    session.retained_bytes()
                        >= session.callback.blocks.as_ref().unwrap().retained_bytes()
                            + super::super::native_header_bytes(linear).unwrap()
                );
            }
            let mut changed = session.compatibility.clone();
            changed.data = pse_ids::ContentHash::from_bytes([63; 32]);
            let error = session.replace(
                Function::Equations(Box::new(Coupled::new(products, false, true))),
                settings.clone(),
                changed.clone(),
            );
            assert!(error.is_ok());
            let failed = session
                .solve(
                    &[0.; 3],
                    &controls,
                    &accuracy,
                    execution(),
                    &tolerance,
                    None,
                )
                .unwrap();
            assert!(
                matches!(failed.callback_failure(),Some(ProblemError::Linear {kind:crate::LinearFailureKind::Numerical,cause}) if matches!(cause.as_ref(),feral::FeralError::SingularBasis {..})),
                "{failed:?}"
            );
            let changed_pattern = Function::Equations(Box::new(Coupled::new(
                Arc::new(AtomicU64::new(0)),
                true,
                false,
            )));
            assert!(matches!(
                session.replace(changed_pattern, settings, changed),
                Err(ProblemError::Unsupported(_))
            ));
            assert_eq!(session.mem, mem);
        }
    }
}

#[cfg(test)]
mod comparisons;
