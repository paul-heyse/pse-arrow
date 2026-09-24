// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::ProblemError;
/// Sparse callback structure in the exact order used by the oracle's CSC values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pattern {
    pub rows: Vec<i32>,
    pub columns: Vec<i32>,
}
impl Pattern {
    pub(crate) fn new(
        p: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
        lower: bool,
    ) -> Result<Self, ProblemError> {
        let mut rows = Vec::new();
        let mut columns = Vec::new();
        for c in 0..p.ncols() {
            let mut previous = None;
            for r in p.row_idx_of_col(c) {
                if r >= p.nrows() || lower && r < c || previous.is_some_and(|v| v >= r) {
                    return Err(ProblemError::Contract(
                        "noncanonical native sparse pattern".into(),
                    ));
                }
                previous = Some(r);
                rows.push(checked_index(r)?);
                columns.push(checked_index(c)?);
            }
        }
        checked_index(rows.len())?;
        Ok(Self { rows, columns })
    }
}

fn checked_index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Contract("native sparse index overflow".into()))
}
