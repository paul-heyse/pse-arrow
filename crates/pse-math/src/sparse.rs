// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! faer owns canonical sparse structure; this module retains a mechanical refill map.
use crate::{
    MathError,
    index::{Addend, Entry, TiVec},
};
use faer::sparse::{Pair, SparseColMat, SymbolicSparseColMat, SymbolicSparseColMatRef};

/// Reusable numeric matrix with an original-contribution to canonical-entry map.
#[derive(Clone, Debug)]
pub struct AssemblyMatrix {
    matrix: SparseColMat<usize, f64>,
    /// Each original contribution's position in faer's canonical value storage.
    refill: TiVec<Addend, usize>,
    strict_lower: Option<StrictLowerLayout>,
}
#[derive(Clone, Debug)]
struct StrictLowerLayout {
    starts: Vec<usize>,
    counts: Vec<usize>,
}
impl AssemblyMatrix {
    /// Known owned CSC and refill buffers, excluding allocator overhead.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of_val(self.matrix.symbolic().col_ptr())
            + size_of_val(self.matrix.row_idx())
            + size_of_val(self.matrix.val())
            + self.refill.capacity() * size_of::<usize>()
            + self.strict_lower.as_ref().map_or(0, |layout| {
                (layout.starts.capacity() + layout.counts.capacity()) * size_of::<usize>()
            })
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
        Ok(Self {
            matrix,
            refill,
            strict_lower: None,
        })
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
    /// Admit the lower-triangle layout consumed by completed symmetric actions.
    pub(crate) fn lower_hessian<C>(
        order: usize,
        entries: &[Entry<C, C>],
        limit: usize,
    ) -> Result<Self, MathError>
    where
        C: Copy + Into<usize>,
    {
        let mut result = Self::hessian(order, entries, limit)?;
        let pattern = result.matrix.symbolic();
        let mut starts = Vec::with_capacity(order + 1);
        let mut counts = Vec::with_capacity(order);
        for column in 0..order {
            let begin = pattern.col_ptr()[column];
            let end = pattern.col_ptr()[column + 1];
            if pattern.row_idx()[begin..end]
                .iter()
                .any(|&row| row < column)
            {
                return Err(MathError::Contract(
                    "Hessian must use the lower triangle".into(),
                ));
            }
            let strict = begin
                + usize::from(
                    pattern
                        .row_idx()
                        .get(begin)
                        .is_some_and(|&row| begin < end && row == column),
                );
            starts.push(strict);
            counts.push(end - strict);
        }
        starts.push(pattern.col_ptr()[order]);
        result.strict_lower = Some(StrictLowerLayout { starts, counts });
        Ok(result)
    }
    /// Native sparse storage; numeric zeros retain their declared support.
    pub fn matrix(&self) -> &SparseColMat<usize, f64> {
        &self.matrix
    }
    /// Compile an original contribution's canonical numeric destination once.
    pub fn contribution_index(&self, addend: Addend) -> Result<usize, MathError> {
        self.refill
            .get(addend)
            .copied()
            .ok_or_else(|| MathError::Contract("refill index".into()))
    }
    /// Clear values without rebuilding, sorting or reallocating structure.
    pub fn clear(&mut self) {
        self.matrix.val_mut().fill(0.0);
    }
    /// Discard numerical entries before exclusive scratch transfer. Structural CSC
    /// and refill mappings remain immutable; the next evaluation must clear/refill.
    pub(crate) fn poison(&mut self) {
        self.matrix.val_mut().fill(f64::NAN);
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
    /// Apply an already completed lower symmetric matrix through faer's products.
    pub(crate) fn symmetric_product(
        &self,
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        if self.matrix.nrows() != self.matrix.ncols() {
            return Err(MathError::Contract(
                "symmetric product requires a square matrix".into(),
            ));
        }
        let layout = self
            .strict_lower
            .as_ref()
            .ok_or_else(|| MathError::Contract("missing lower symmetric product layout".into()))?;
        self.product(direction, output)?;
        // faer's uncompressed CSC view skips each diagonal in the existing value
        // storage. No numeric copy or cancellation-based diagonal removal is needed.
        let strict = faer::sparse::SparseColMatRef::new(
            SymbolicSparseColMatRef::new_checked(
                self.matrix.nrows(),
                self.matrix.ncols(),
                &layout.starts,
                Some(&layout.counts),
                self.matrix.row_idx(),
            ),
            self.matrix.val(),
        );
        faer::sparse::linalg::matmul::dense_sparse_matmul(
            faer::MatMut::from_row_major_slice_mut(output, 1, self.matrix.ncols()),
            faer::Accum::Add,
            faer::MatRef::from_row_major_slice(direction, 1, self.matrix.nrows()),
            strict,
            1.0,
            faer::Par::Seq,
        );
        if output.iter().any(|value| !value.is_finite()) {
            return Err(MathError::Contract(
                "nonfinite symmetric matrix product".into(),
            ));
        }
        Ok(())
    }
}

/// Content identity of canonical CSC structure, excluding numerical values. Column
/// lengths frame the support of each variable; concatenated row numbers are ambiguous.
pub fn pattern_key(pattern: SymbolicSparseColMatRef<'_, usize>) -> pse_ids::ContentHash {
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
    fn completed_symmetric_product_uses_strict_lower_without_diagonal_cancellation() {
        let entries = [
            Entry::new(GlobalCol::new(1), GlobalCol::new(0)),
            Entry::new(GlobalCol::new(1), GlobalCol::new(1)),
            Entry::new(GlobalCol::new(2), GlobalCol::new(1)),
        ];
        let mut matrix = AssemblyMatrix::lower_hessian(3, &entries, 10).unwrap();
        matrix.add(Addend::new(0), 1e16).unwrap();
        matrix.add(Addend::new(1), -1e16).unwrap();
        matrix.add(Addend::new(2), 1.).unwrap();
        let mut output = [999.; 3];
        matrix.symmetric_product(&[1.; 3], &mut output).unwrap();
        assert_eq!(output, [1e16, 1., 1.]);
        let entries = [Entry::new(GlobalCol::new(0), GlobalCol::new(0))];
        let mut diagonal = AssemblyMatrix::lower_hessian(1, &entries, 10).unwrap();
        diagonal.add(Addend::new(0), 1e308).unwrap();
        diagonal.symmetric_product(&[1.], &mut output[..1]).unwrap();
        assert_eq!(output[0], 1e308);
        assert!(
            matrix
                .strict_lower
                .as_ref()
                .is_some_and(|layout| layout.starts.len() == 4 && layout.counts.len() == 3)
        );
    }
    #[test]
    fn coefficient_upper_triangle_remains_valid_but_is_not_a_lower_action_layout() {
        let entries = [Entry::new(GlobalCol::new(0), GlobalCol::new(1))];
        let mut matrix = AssemblyMatrix::hessian(2, &entries, 10).unwrap();
        matrix.add(Addend::new(0), 3.).unwrap();
        assert_eq!(matrix.matrix().to_dense()[(0, 1)], 3.);
        assert!(AssemblyMatrix::lower_hessian(2, &entries, 10).is_err());
        let mut output = [17., 19.];
        assert!(matrix.symmetric_product(&[1., 1.], &mut output).is_err());
        assert_eq!(output, [17., 19.]);
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
