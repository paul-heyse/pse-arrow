// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! faer owns canonical sparse structure; this module retains a mechanical refill map.
use crate::MathError;
use faer::sparse::{Pair, SparseColMat, SymbolicSparseColMat};

/// Reusable numeric matrix with an original-contribution to canonical-entry map.
#[derive(Clone, Debug)]
pub struct AssemblyMatrix {
    matrix: SparseColMat<usize, f64>,
    refill: Vec<usize>,
}
impl AssemblyMatrix {
    /// Canonicalize once, retaining duplicates as independent refill contributions.
    pub fn new(
        rows: usize,
        cols: usize,
        pairs: &[(usize, usize)],
        limit: usize,
    ) -> Result<Self, MathError> {
        if rows > limit
            || cols > limit
            || pairs.len() > limit
            || limit > isize::MAX as usize
            || pairs.iter().any(|&(r, c)| r >= rows || c >= cols)
        {
            return Err(MathError::Limit(
                "sparse dimensions, indices or contributions",
            ));
        }
        let indices: Vec<_> = pairs.iter().map(|&(r, c)| Pair::new(r, c)).collect();
        let (pattern, order) = SymbolicSparseColMat::try_new_from_indices(rows, cols, &indices)
            .map_err(|e| MathError::Library(e.to_string()))?;
        let refill = pairs
            .iter()
            .map(|&(r, c)| {
                let start = pattern.col_ptr()[c];
                let end = pattern.col_ptr()[c + 1];
                pattern.row_idx()[start..end]
                    .binary_search(&r)
                    .map(|i| start + i)
                    .map_err(|_| {
                        MathError::Contract("library sparse ordering omitted an entry".into())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let matrix = SparseColMat::new_from_argsort(pattern, &order, &vec![0.0; pairs.len()])
            .map_err(|e| MathError::Library(e.to_string()))?;
        Ok(Self { matrix, refill })
    }
    /// Native sparse storage; numeric zeros retain their declared support.
    pub fn matrix(&self) -> &SparseColMat<usize, f64> {
        &self.matrix
    }
    /// Clear values without rebuilding, sorting or reallocating structure.
    pub fn clear(&mut self) {
        self.matrix.val_mut().fill(0.0);
    }
    /// Add one original contribution to its canonical slot.
    pub fn add(&mut self, contribution: usize, value: f64) -> Result<(), MathError> {
        let index = *self
            .refill
            .get(contribution)
            .ok_or_else(|| MathError::Contract("refill index".into()))?;
        let next = self.matrix.val()[index] + value;
        if !next.is_finite() {
            return Err(MathError::Contract("nonfinite sparse contribution".into()));
        }
        self.matrix.val_mut()[index] = next;
        Ok(())
    }
    /// Exact sparse matrix-vector product through faer.
    pub fn product(&self, direction: &[f64], output: &mut [f64]) -> Result<(), MathError> {
        if direction.len() != self.matrix.ncols()
            || output.len() != self.matrix.nrows()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "sparse product dimensions or values".into(),
            ));
        }
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            faer::MatMut::from_column_major_slice_mut(output, self.matrix.nrows(), 1),
            faer::Accum::Replace,
            self.matrix.as_ref(),
            faer::MatRef::from_column_major_slice(direction, self.matrix.ncols(), 1),
            1.0,
            faer::Par::Seq,
        );
        if output.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract("nonfinite matrix product".into()));
        }
        Ok(())
    }
}

/// Content identity of canonical CSC structure, excluding numerical values. Column
/// lengths frame the support of each variable; concatenated row numbers are ambiguous.
pub fn pattern_key(
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
) -> pse_ids::ContentHash {
    let mut h = pse_ids::FramedHasher::new("pse.sparse.pattern.v1");
    h.u64(pattern.nrows() as u64).u64(pattern.ncols() as u64);
    for c in 0..pattern.ncols() {
        let rows = pattern.row_idx_of_col(c);
        h.u64(c as u64).u64(rows.len() as u64);
        for r in rows {
            h.u64(r as u64);
        }
    }
    h.finish_hash()
}
#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn sparse_layout_identity_frames_column_boundaries() {
        let a = AssemblyMatrix::new(2, 2, &[(1, 0)], 10).unwrap();
        let b = AssemblyMatrix::new(2, 2, &[(1, 1)], 10).unwrap();
        assert_ne!(
            pattern_key(a.matrix().symbolic()),
            pattern_key(b.matrix().symbolic())
        );
        let zero = AssemblyMatrix::new(2, 2, &[], 10).unwrap();
        assert_ne!(
            pattern_key(a.matrix().symbolic()),
            pattern_key(zero.matrix().symbolic())
        );
    }
}
