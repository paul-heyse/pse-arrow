use crate::internal_prelude::*;
use crate::linalg::matmul::triangular::DiagonalKind;
use crate::{MatRef, assert};
use linalg::matmul::triangular::BlockStructure;
pub fn has_spicy_matmul<T: ComplexField>() -> bool {
	// Native GEMM is excluded by the maintained nonfused arithmetic policy.
	false
}
pub fn spicy_matmul_scratch<T: ComplexField>(
	nrows: usize,
	ncols: usize,
	depth: usize,
	gather: bool,
	diag: bool,
) -> StackReq {

	let diag = temp_mat_scratch::<T>(nrows, if diag { depth } else { 0 });
	let gather = temp_mat_scratch::<T>(nrows, if gather { ncols } else { 0 });
	diag.and(gather)
}
pub fn spicy_matmul<I: Index, T: ComplexField>(
	C: MatMut<'_, T>,
	C_block: BlockStructure,
	row_idx: Option<&[I]>,
	col_idx: Option<&[I]>,
	beta: Accum,
	A: MatRef<'_, T>,
	conj_A: Conj,
	B: MatRef<'_, T>,
	conj_B: Conj,
	D: Option<DiagRef<'_, T>>,
	alpha: T,
	par: Par,
	stack: &mut MemStack,
) {
	let mut C = C;
	assert!(all(
		A.ncols() == B.nrows(),
		A.nrows() == row_idx.map(|idx| idx.len()).unwrap_or(C.nrows()),
		B.ncols() == col_idx.map(|idx| idx.len()).unwrap_or(C.ncols()),
	));
	let nrows = A.nrows();
	let ncols = B.ncols();
	let depth = A.ncols();
	if nrows == 0 || ncols == 0 {
		return;
	}
	let par = if (nrows * ncols).saturating_mul(depth)
		> 32usize * 32usize * 32usize
	{
		par
	} else {
		Par::Seq
	};
	if let Some(row_idx) = row_idx {
		for &i in row_idx {
			assert!(i.zx() < C.nrows());
		}
	}
	if let Some(col_idx) = col_idx {
		for &j in col_idx {
			assert!(j.zx() < C.ncols());
		}
	}
	// Always use the generic route and its matching gather/diagonal scratch.

	let (mut out, stack) = unsafe {
		temp_mat_uninit::<T, _, _>(
			nrows,
			if row_idx.is_some() || col_idx.is_some() {
				ncols
			} else {
				0
			},
			stack,
		)
	};
	let mut out = out.as_mat_mut();
	let (mut scaled, stack) = unsafe {
		temp_mat_uninit::<T, _, _>(
			nrows,
			if D.is_some() { depth } else { 0 },
			stack,
		)
	};
	let mut scaled = scaled.as_mat_mut();
	let _ = stack;
	let A = if let Some(D) = D {
		for k in 0..depth {
			let ref d = D[k].real();
			zip!(scaled.rb_mut().col_mut(k), A.col(k))
				.for_each(|unzip!(x, y)| *x = y.mul_real(d));
		}
		scaled.rb()
	} else {
		A
	};
	{
		let mut C = if row_idx.is_some() || col_idx.is_some() {
			out.rb_mut()
		} else {
			C.rb_mut()
		};
		let size = Ord::min(nrows, ncols);
		if C_block.is_dense() {
			linalg::matmul::triangular::matmul_with_conj(
				C,
				C_block,
				if row_idx.is_some() || col_idx.is_some() {
					Accum::Replace
				} else {
					beta
				},
				A,
				BlockStructure::Rectangular,
				conj_A,
				B,
				BlockStructure::Rectangular,
				conj_B,
				alpha,
				par,
			);
		} else {
			linalg::matmul::triangular::matmul_with_conj(
				C.rb_mut().get_mut(..size, ..size),
				C_block,
				if row_idx.is_some() || col_idx.is_some() {
					Accum::Replace
				} else {
					beta
				},
				A.get(..size, ..),
				BlockStructure::Rectangular,
				conj_A,
				B.get(.., ..size),
				BlockStructure::Rectangular,
				conj_B,
				alpha.copy(),
				par,
			);
			if C_block.is_lower() && nrows > ncols {
				linalg::matmul::matmul_with_conj(
					C.rb_mut().get_mut(size.., ..size),
					if row_idx.is_some() || col_idx.is_some() {
						Accum::Replace
					} else {
						beta
					},
					A.get(size.., ..),
					conj_A,
					B.get(.., ..size),
					conj_B,
					alpha,
					par,
				);
			} else if ncols > nrows {
				linalg::matmul::matmul_with_conj(
					C.rb_mut().get_mut(..size, size..),
					if row_idx.is_some() || col_idx.is_some() {
						Accum::Replace
					} else {
						beta
					},
					A.get(..size, ..),
					conj_A,
					B.get(.., size..),
					conj_B,
					alpha,
					par,
				);
			}
		}
	}
	let lower = C_block.is_lower();
	let upper = C_block.is_upper();
	let diag = matches!(C_block.diag_kind(), DiagonalKind::Generic) as usize;
	match (row_idx, col_idx) {
		(Some(row_idx), Some(col_idx)) => match beta {
			Accum::Replace => {
				for (j, &jj) in col_idx.iter().enumerate() {
					for (i, &ii) in row_idx.iter().enumerate() {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(ii.zx(), jj.zx())] = out[(i, j)].copy();
						}
					}
				}
			},
			Accum::Add => {
				for (j, &jj) in col_idx.iter().enumerate() {
					for (i, &ii) in row_idx.iter().enumerate() {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(ii.zx(), jj.zx())] += &out[(i, j)];
						}
					}
				}
			},
		},
		(Some(row_idx), None) => match beta {
			Accum::Replace => {
				for j in 0..ncols {
					for (i, &ii) in row_idx.iter().enumerate() {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(ii.zx(), j)] = out[(i, j)].copy();
						}
					}
				}
			},
			Accum::Add => {
				for j in 0..ncols {
					for (i, &ii) in row_idx.iter().enumerate() {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(ii.zx(), j)] += &out[(i, j)];
						}
					}
				}
			},
		},
		(None, Some(col_idx)) => match beta {
			Accum::Replace => {
				for (j, &jj) in col_idx.iter().enumerate() {
					for i in 0..nrows {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(i, jj.zx())] = out[(i, j)].copy();
						}
					}
				}
			},
			Accum::Add => {
				for (j, &jj) in col_idx.iter().enumerate() {
					for i in 0..nrows {
						if (lower && j < i + diag) || (upper && i < j + diag) {
							C[(i, jj.zx())] += &out[(i, j)];
						}
					}
				}
			},
		},
		(None, None) => {},
	}
}
