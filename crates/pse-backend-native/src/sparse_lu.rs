// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared faer mechanics. Factors, scientific qualification and storage leases
//! stay with each consumer; scratch is charged against its original allowance.
use crate::ProblemError;
use faer::{
    Conj, MatMut, Par,
    dyn_stack::{MemBuffer, MemStack},
    sparse::{
        SparseColMatRef, SymbolicSparseColMatRef,
        linalg::{
            LuError,
            lu::{self, LuRef, NumericLu, SymbolicLu},
        },
    },
};

#[derive(Clone, Copy)]
pub(crate) enum Qualification {
    CompleteMatching,
    Unqualified,
}

pub(crate) fn sparse_error(error: faer::sparse::FaerError) -> ProblemError {
    match error {
        faer::sparse::FaerError::OutOfMemory => {
            ProblemError::memory("faer sparse allocation failed")
        }
        faer::sparse::FaerError::IndexOverflow => {
            ProblemError::Contract("faer sparse index overflow".into())
        }
        other => ProblemError::internal(format!("faer sparse failure: {other:?}")),
    }
}
pub(crate) fn factor_error(error: LuError, qualification: Qualification) -> ProblemError {
    match error {
        LuError::Generic(error) => sparse_error(error),
        LuError::SymbolicSingular { index } => match qualification {
            Qualification::CompleteMatching => ProblemError::Contract(format!(
                "faer symbolic pivot {index} contradicts authenticated complete matching"
            )),
            Qualification::Unqualified => {
                ProblemError::numerical(format!("faer symbolic rank loss at pivot {index}"))
            }
        },
    }
}
pub(crate) fn opaque_allowance(n: usize) -> Option<usize> {
    n.checked_mul(n)?
        .checked_mul(64)?
        .checked_add(n.checked_mul(512)?)?
        .checked_add(4096)
}
pub(crate) fn symbolic(
    pattern: SymbolicSparseColMatRef<'_, usize>,
    name: &str,
) -> Result<SymbolicLu<usize>, ProblemError> {
    pattern
        .nrows()
        .checked_mul(pattern.ncols())
        .ok_or_else(|| ProblemError::Contract("faer symbolic extent overflow".into()))?;
    crate::observation::scoped(name, || {
        lu::factorize_symbolic_lu(pattern, Default::default())
    })
    .map_err(sparse_error)
}
pub(crate) fn scratch(
    symbolic: &SymbolicLu<usize>,
    nrhs: usize,
    parallel: Par,
    allowance: usize,
    factor: bool,
) -> Result<(MemBuffer, usize), ProblemError> {
    symbolic
        .nrows()
        .checked_mul(nrhs)
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or_else(|| ProblemError::Contract("faer RHS extent overflow".into()))?;
    let solve = symbolic.solve_in_place_scratch::<f64>(nrhs, parallel);
    let request = if factor {
        solve.or(symbolic.factorize_numeric_lu_scratch::<f64>(parallel, Default::default()))
    } else {
        solve
    };
    let bytes = request.size_bytes();
    if bytes > allowance {
        return Err(ProblemError::memory(
            "faer scratch exceeds admitted allowance",
        ));
    }
    Ok((
        MemBuffer::try_new(request)
            .map_err(|_| ProblemError::memory("faer scratch allocation failed"))?,
        bytes,
    ))
}
pub(crate) fn factor(
    symbolic: &SymbolicLu<usize>,
    numeric: &mut NumericLu<usize, f64>,
    matrix: SparseColMatRef<'_, usize, f64>,
    parallel: Par,
    memory: &mut MemBuffer,
    qualification: Qualification,
    name: &str,
) -> Result<(), ProblemError> {
    if matrix.nrows() != symbolic.nrows() || matrix.ncols() != symbolic.ncols() {
        return Err(ProblemError::Contract(
            "faer factor shape differs from symbolic plan".into(),
        ));
    }
    if matrix.val().iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite faer factor matrix"));
    }
    crate::observation::scoped(name, || {
        symbolic.factorize_numeric_lu(
            numeric,
            matrix,
            parallel,
            MemStack::new(memory),
            Default::default(),
        )
    })
    .map(|_| ())
    .map_err(|e| factor_error(e, qualification))
}
pub(crate) fn solve(
    symbolic: &SymbolicLu<usize>,
    numeric: &NumericLu<usize, f64>,
    conj: Conj,
    rhs: MatMut<'_, f64>,
    parallel: Par,
    memory: &mut MemBuffer,
    name: &str,
) -> Result<(), ProblemError> {
    solve_operation(
        symbolic,
        numeric,
        SolveOperation::Forward(conj),
        rhs,
        parallel,
        memory,
        name,
    )
}
pub(crate) fn solve_transpose(
    symbolic: &SymbolicLu<usize>,
    numeric: &NumericLu<usize, f64>,
    conj: Conj,
    rhs: MatMut<'_, f64>,
    parallel: Par,
    memory: &mut MemBuffer,
    name: &str,
) -> Result<(), ProblemError> {
    solve_operation(
        symbolic,
        numeric,
        SolveOperation::Transpose(conj),
        rhs,
        parallel,
        memory,
        name,
    )
}
enum SolveOperation {
    Forward(Conj),
    Transpose(Conj),
}
fn solve_operation(
    symbolic: &SymbolicLu<usize>,
    numeric: &NumericLu<usize, f64>,
    operation: SolveOperation,
    rhs: MatMut<'_, f64>,
    parallel: Par,
    memory: &mut MemBuffer,
    name: &str,
) -> Result<(), ProblemError> {
    if rhs.nrows() != symbolic.nrows() {
        return Err(ProblemError::Contract("faer solve RHS shape".into()));
    }
    let mut rhs = rhs;
    let lu = LuRef::new_unchecked(symbolic, numeric);
    crate::observation::scoped(name, || match operation {
        SolveOperation::Transpose(conj) => lu.solve_transpose_in_place_with_conj(
            conj,
            rhs.as_mut(),
            parallel,
            MemStack::new(memory),
        ),
        SolveOperation::Forward(conj) => {
            lu.solve_in_place_with_conj(conj, rhs.as_mut(), parallel, MemStack::new(memory))
        }
    });
    if rhs
        .as_ref()
        .col_iter()
        .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(ProblemError::numerical(
            "singular or nonfinite faer solution",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classification_preserves_qualification_memory_and_extent() {
        assert!(matches!(
            sparse_error(faer::sparse::FaerError::OutOfMemory),
            ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            }
        ));
        assert!(matches!(
            sparse_error(faer::sparse::FaerError::IndexOverflow),
            ProblemError::Contract(_)
        ));
        assert!(matches!(
            factor_error(
                LuError::SymbolicSingular { index: 0 },
                Qualification::CompleteMatching
            ),
            ProblemError::Contract(_)
        ));
        assert!(matches!(
            factor_error(
                LuError::SymbolicSingular { index: 0 },
                Qualification::Unqualified
            ),
            ProblemError::Numerical { .. }
        ));
        assert_eq!(opaque_allowance(usize::MAX), None);
    }
    #[test]
    fn scratch_admits_before_allocation_and_multi_rhs_refresh_is_finite() {
        let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 2.0),
                faer::sparse::Triplet::new(1, 1, 4.0),
            ],
        )
        .unwrap();
        let symbolic = symbolic(matrix.symbolic(), "test.faer.symbolic").unwrap();
        let required = symbolic
            .solve_in_place_scratch::<f64>(2, Par::Seq)
            .or(symbolic.factorize_numeric_lu_scratch::<f64>(Par::Seq, Default::default()))
            .size_bytes();
        assert!(required > 0);
        assert!(scratch(&symbolic, 2, Par::Seq, required - 1, true).is_err());
        assert!(scratch(&symbolic, usize::MAX, Par::Seq, usize::MAX, true).is_err());
        let (mut memory, bytes) = scratch(&symbolic, 2, Par::Seq, required, true).unwrap();
        assert_eq!(bytes, required);
        let mut numeric = NumericLu::new();
        factor(
            &symbolic,
            &mut numeric,
            matrix.as_ref(),
            Par::Seq,
            &mut memory,
            Qualification::CompleteMatching,
            "test.faer.factor",
        )
        .unwrap();
        let mut rhs = faer::mat![[2.0, 4.0], [4.0, 8.0]];
        solve(
            &symbolic,
            &numeric,
            Conj::No,
            rhs.as_mut(),
            Par::Seq,
            &mut memory,
            "test.faer.solve",
        )
        .unwrap();
        assert_eq!(rhs, faer::mat![[1.0, 2.0], [1.0, 2.0]]);
    }
}
