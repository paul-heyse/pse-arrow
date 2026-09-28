// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! faer owns canonical sparse structure; this module retains a mechanical refill map.
use crate::{
    MathError,
    index::{Addend, Entry, TiVec},
};
use faer::sparse::{Pair, SparseColMat, SymbolicSparseColMat};

/// Reusable numeric matrix with an original-contribution to canonical-entry map.
#[derive(Clone, Debug)]
pub struct AssemblyMatrix {
    matrix: SparseColMat<usize, f64>,
    /// Each original contribution's position in faer's canonical value storage.
    refill: TiVec<Addend, usize>,
}
impl AssemblyMatrix {
    /// Known owned CSC and refill buffers, excluding allocator overhead.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of_val(self.matrix.symbolic().col_ptr())
            + size_of_val(self.matrix.row_idx())
            + size_of_val(self.matrix.val())
            + self.refill.capacity() * size_of::<usize>()
    }
    /// Canonicalize once, retaining duplicates as independent refill contributions: the
    /// entry at position `k` of `entries` is [`Addend`] `k`.
    pub fn new<R, C>(
        rows: usize,
        cols: usize,
        entries: &[Entry<R, C>],
        limit: usize,
    ) -> Result<Self, MathError>
    where
        R: Copy + Into<usize>,
        C: Copy + Into<usize>,
    {
        if rows > limit
            || cols > limit
            || entries.len() > limit
            || limit > isize::MAX as usize
            || entries
                .iter()
                .any(|e| e.row.into() >= rows || e.col.into() >= cols)
        {
            return Err(MathError::Limit(
                "sparse dimensions, indices or contributions",
            ));
        }
        // faer's interior speaks `usize`; the typed spaces end here.
        let indices: Vec<_> = entries
            .iter()
            .map(|e| Pair::new(e.row.into(), e.col.into()))
            .collect();
        let (pattern, order) = SymbolicSparseColMat::try_new_from_indices(rows, cols, &indices)
            .map_err(|e| MathError::Library(e.to_string()))?;
        let refill = indices
            .iter()
            .map(|&Pair { row: r, col: c }| {
                let start = pattern.col_ptr()[c];
                let end = pattern.col_ptr()[c + 1];
                pattern.row_idx()[start..end]
                    .binary_search(&r)
                    .map(|i| start + i)
                    .map_err(|_| {
                        MathError::Contract("library sparse ordering omitted an entry".into())
                    })
            })
            .collect::<Result<TiVec<_, _>, _>>()?;
        let matrix = SparseColMat::new_from_argsort(pattern, &order, &vec![0.0; indices.len()])
            .map_err(|e| MathError::Library(e.to_string()))?;
        Ok(Self { matrix, refill })
    }
    /// A square `order × order` matrix of `(column, column)` entries, such as a
    /// lower-triangle Hessian. Both indices share one space, so a Jacobian's
    /// `(row, column)` entries are refused at compile time.
    ///
    /// ```compile_fail,E0308
    /// use pse_math::{index::{Entry, GlobalCol, GlobalRow}, sparse::AssemblyMatrix};
    ///
    /// let jacobian = [Entry::new(GlobalRow::new(0), GlobalCol::new(0))];
    /// let _ = AssemblyMatrix::hessian(1, &jacobian, 10);
    /// ```
    pub fn hessian<C>(
        order: usize,
        entries: &[Entry<C, C>],
        limit: usize,
    ) -> Result<Self, MathError>
    where
        C: Copy + Into<usize>,
    {
        Self::new(order, order, entries, limit)
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
    pub fn add(&mut self, addend: Addend, value: f64) -> Result<(), MathError> {
        let index = *self
            .refill
            .get(addend)
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
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::SparsePatternV1);
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
    use crate::index::{GlobalCol, GlobalRow};
    fn entry(row: usize, col: usize) -> Entry<GlobalRow, GlobalCol> {
        Entry::new(row.into(), col.into())
    }
    #[test]
    fn sparse_layout_identity_frames_column_boundaries() {
        let a = AssemblyMatrix::new(2, 2, &[entry(1, 0)], 10).unwrap();
        let b = AssemblyMatrix::new(2, 2, &[entry(1, 1)], 10).unwrap();
        assert_ne!(
            pattern_key(a.matrix().symbolic()),
            pattern_key(b.matrix().symbolic())
        );
        let zero = AssemblyMatrix::new::<GlobalRow, GlobalCol>(2, 2, &[], 10).unwrap();
        assert_ne!(
            pattern_key(a.matrix().symbolic()),
            pattern_key(zero.matrix().symbolic())
        );
    }
}
