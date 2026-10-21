// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-prepared wrapper function CSC; no numerical point is retained here.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct FunctionLayout {
    pub(super) matrix: pse_math::sparse::AssemblyMatrix,
    constants: Vec<usize>,
}
impl FunctionLayout {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.matrix.retained_bytes()
            + self.constants.capacity() * size_of::<usize>()
            + 128
    }
    pub(super) fn slot(&self, row: usize, column: usize) -> Option<usize> {
        let matrix = self.matrix.matrix();
        if column >= matrix.ncols() || row >= matrix.nrows() {
            return None;
        }
        let range = matrix.col_range(column);
        matrix.row_idx()[range.clone()]
            .binary_search(&row)
            .ok()
            .map(|offset| range.start + offset)
    }
    pub(super) fn initialize(&self, matrix: &mut faer::sparse::SparseColMat<usize, f64>) {
        matrix.val_mut().fill(0.0);
        for &slot in &self.constants {
            matrix.val_mut()[slot] += 1.0;
        }
    }
}
pub(super) fn functions<O: Oracle>(
    oracle: &Anchored<O>,
    allowance: usize,
) -> Result<BTreeMap<(usize, Function), FunctionLayout>, ProblemError> {
    let mut result = BTreeMap::new();
    let mut retained = 0usize;
    for mode in 0..oracle.contract.events.len() {
        for function in roles(&oracle.contract, oracle.nq, mode) {
            let support = oracle.support(mode, function);
            // Include the simultaneously live source, sorting/index/order/refill
            // construction and canonical destination before asking faer to build it.
            let construction = support
                .capacity()
                .checked_mul(256)
                .and_then(|n| n.checked_add(oracle.width().checked_add(1)?.checked_mul(32)?))
                .and_then(|n| n.checked_add(size_of::<FunctionLayout>() + 512))
                .and_then(|n| n.checked_add(retained))
                .ok_or_else(|| ProblemError::memory("anchored function layout extent"))?;
            if construction > allowance {
                return Err(ProblemError::memory("anchored function layout allowance"));
            }
            let rows = match function {
                Function::Initial | Function::Rhs | Function::Reset(_) => {
                    oracle.contract.states.len()
                }
                Function::Output => oracle.contract.outputs.len(),
                Function::QuadratureFlux => oracle.nq,
                Function::Roots => oracle.contract.events[mode].len(),
                Function::Inventory | Function::Transfer(_) => oracle.contract.balances.len(),
            };
            let matrix = pse_math::sparse::AssemblyMatrix::new(
                rows,
                oracle.width(),
                &support,
                allowance.min(isize::MAX as usize),
            )?;
            let mut layout = FunctionLayout {
                matrix,
                constants: Vec::new(),
            };
            let constants: Vec<_> = match function {
                Function::Initial => oracle
                    .anchors
                    .iter()
                    .enumerate()
                    .map(|(k, row)| (*row, oracle.anchor(k)))
                    .collect(),
                Function::Output => (0..oracle.states())
                    .map(|q| (oracle.m + q, oracle.n + q))
                    .collect(),
                Function::Reset(_) => (0..oracle.states())
                    .map(|q| (oracle.n + q, oracle.n + q))
                    .collect(),
                _ => Vec::new(),
            };
            for (row, column) in constants {
                layout.constants.push(
                    layout.slot(row, column).ok_or_else(|| {
                        ProblemError::internal("anchored structural entry absent")
                    })?,
                );
            }
            retained = retained
                .checked_add(layout.bytes())
                .ok_or_else(|| ProblemError::memory("anchored retained layouts extent"))?;
            if retained > allowance {
                return Err(ProblemError::memory("anchored retained layouts allowance"));
            }
            result.insert((mode, function), layout);
        }
    }
    Ok(result)
}
fn roles(contract: &Contract, inner_quadratures: usize, mode: usize) -> Vec<Function> {
    let mut roles = vec![Function::Initial, Function::Rhs, Function::Output];
    if inner_quadratures > 0 {
        roles.push(Function::QuadratureFlux);
    }
    if !contract.balances.is_empty() {
        roles.push(Function::Inventory);
    }
    if !contract.events[mode].is_empty() {
        roles.push(Function::Roots);
    }
    for event in 0..contract.events[mode].len() {
        roles.push(Function::Reset(event));
        if !contract.balances.is_empty() {
            roles.push(Function::Transfer(event));
        }
    }
    roles
}
