// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete f64 supplier entry points shared by unit and generated-code controls.
#![allow(
    unsafe_code,
    reason = "stable exported names are test-only generated-code inspection entry points"
)]

use faer::traits::{ComplexField, pulp::Scalar};
use faer::{
    Accum, Conj, Mat, MatMut, MatRef, Par,
    dyn_stack::{MemBuffer, MemStack},
};

#[unsafe(no_mangle)]
#[inline(never)]
pub(crate) fn faer_nonfused_mul_add(a: f64, b: f64, c: f64) -> f64 {
    <f64 as ComplexField>::simd_mul_add(&Scalar, a, b, c)
}

#[unsafe(no_mangle)]
#[inline(never)]
pub(crate) fn faer_nonfused_norm_add(a: f64, c: f64) -> f64 {
    <f64 as ComplexField>::simd_abs2_add(&Scalar, a, c)
}

#[unsafe(no_mangle)]
#[inline(never)]
pub(crate) fn faer_nonfused_matmul(
    dst: MatMut<'_, f64>,
    lhs: MatRef<'_, f64>,
    rhs: MatRef<'_, f64>,
    par: Par,
) {
    faer::linalg::matmul::matmul(dst, Accum::Add, lhs, rhs, 1.0, par);
}

pub(crate) fn adversarial() {
    // Independent exact binary64 counterfactual: multiplication rounds to one,
    // whereas a fused subtraction retains the nonzero low product term.
    let a = std::hint::black_box(1.0 + 2.0_f64.powi(-27));
    let b = std::hint::black_box(1.0 - 2.0_f64.powi(-27));
    assert_eq!(a * b - 1.0, 0.0);
    assert_eq!(a.mul_add(b, -1.0), -2.0_f64.powi(-54));
    assert_eq!(faer_nonfused_mul_add(a, b, -1.0), 0.0);
    let square = a * a;
    assert_ne!(a.mul_add(a, -square), 0.0);
    assert_eq!(faer_nonfused_norm_add(a, -square), 0.0);
    let _: <f64 as ComplexField>::Arch = Scalar;
    for par in [Par::Seq, Par::rayon(0)] {
        // Rank update; col/row-major matvec; nano-sized and large GEMM; general
        // strided fallback. All use actual public dispatch, without forcing Seq.
        for (m, n, k) in [(7, 9, 1), (17, 1, 2), (1, 17, 2), (4, 4, 2), (80, 80, 80)] {
            let lhs = Mat::from_fn(m, k, |_, p| if p == 0 { a } else { 0.0 });
            let rhs = Mat::from_fn(k, n, |p, _| if p == 0 { b } else { 0.0 });
            let mut dst = Mat::from_fn(m, n, |_, _| -1.0);
            faer_nonfused_matmul(dst.as_mut(), lhs.as_ref(), rhs.as_ref(), par);
            assert_eq!(dst.norm_max(), 0.0, "m={m} n={n} k={k} par={par:?}");
            let mut dst = Mat::from_fn(n, m, |_, _| -1.0);
            faer_nonfused_matmul(
                dst.as_mut().transpose_mut(),
                lhs.as_ref(),
                rhs.as_ref(),
                par,
            );
            assert_eq!(dst.norm_max(), 0.0);
        }
    }
    let values = Mat::from_fn(129, 3, |i, j| ((i + j) % 7) as f64 - 3.0);
    let expected: f64 = (0..3)
        .flat_map(|j| (0..129).map(move |i| (((i + j) % 7) as f64 - 3.0).powi(2)))
        .sum();
    assert_eq!(values.squared_norm_l2(), expected);
}

#[unsafe(no_mangle)]
#[inline(never)]
pub(crate) fn faer_nonfused_decompositions() {
    use faer::linalg::{evd, svd};
    use faer::prelude::Solve;
    for par in [Par::Seq, Par::rayon(0)] {
        for n in [2, 40] {
            let matrix = Mat::from_fn(n, n, |i, j| {
                if i == j {
                    4.0
                } else if i.abs_diff(j) == 1 {
                    0.25
                } else {
                    0.0
                }
            });
            let rhs = Mat::from_fn(n, 2, |i, j| (i + j + 1) as f64);
            let factor = matrix.partial_piv_lu();
            let x = factor.solve(&rhs);
            assert!((&matrix * &x - &rhs).norm_max() < 1e-11);
            let mut singular = faer::diag::Diag::<f64>::zeros(n);
            let mut vectors = Mat::<f64>::zeros(n, n);
            let mut params: svd::SvdParams = faer::auto!(f64);
            params.recursion_threshold = 8;
            let req = svd::svd_scratch::<f64>(
                n,
                n,
                svd::ComputeSvdVectors::No,
                svd::ComputeSvdVectors::Full,
                par,
                params.into(),
            );
            let mut memory = MemBuffer::new(req);
            svd::svd(
                matrix.as_ref(),
                singular.as_mut(),
                None,
                Some(vectors.as_mut()),
                par,
                MemStack::new(&mut memory),
                params.into(),
            )
            .unwrap();
            assert!(
                singular
                    .column_vector()
                    .iter()
                    .all(|s| *s > 3.0 && *s < 5.0)
            );
            let mut eigenvalues = faer::diag::Diag::<f64>::zeros(n);
            let mut params: evd::SelfAdjointEvdParams = faer::auto!(f64);
            params.recursion_threshold = 8;
            let req = evd::self_adjoint_evd_scratch::<f64>(
                n,
                evd::ComputeEigenvectors::No,
                par,
                params.into(),
            );
            let mut memory = MemBuffer::new(req);
            evd::self_adjoint_evd(
                matrix.as_ref(),
                eigenvalues.as_mut(),
                None,
                par,
                MemStack::new(&mut memory),
                params.into(),
            )
            .unwrap();
            assert!(
                eigenvalues
                    .column_vector()
                    .iter()
                    .all(|s| *s > 3.0 && *s < 5.0)
            );
        }
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub(crate) fn faer_nonfused_sparse_modes() {
    use faer::sparse::{
        SparseColMat, Triplet,
        linalg::{SupernodalThreshold, lu},
    };
    for n in [4, 48] {
        let mut triplets = Vec::new();
        for j in 0..n {
            for i in 0..n {
                if i == j || usize::abs_diff(i, j) == 1 {
                    triplets.push(Triplet::new(i, j, if i == j { 4.0 } else { 0.25 }));
                }
            }
        }
        let matrix = SparseColMat::<usize, f64>::try_new_from_triplets(n, n, &triplets).unwrap();
        for threshold in [
            SupernodalThreshold::FORCE_SIMPLICIAL,
            SupernodalThreshold::FORCE_SUPERNODAL,
        ] {
            let symbolic = lu::factorize_symbolic_lu(
                matrix.symbolic(),
                lu::LuSymbolicParams {
                    supernodal_flop_ratio_threshold: threshold,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                format!("{symbolic:?}").contains("raw: Supernodal("),
                threshold == SupernodalThreshold::FORCE_SUPERNODAL
            );
            for par in [Par::Seq, Par::rayon(0)] {
                let mut numeric = lu::NumericLu::new();
                let mut memory = MemBuffer::new(
                    symbolic.factorize_numeric_lu_scratch::<f64>(par, Default::default()),
                );
                let factors = symbolic
                    .factorize_numeric_lu(
                        &mut numeric,
                        matrix.as_ref(),
                        par,
                        MemStack::new(&mut memory),
                        Default::default(),
                    )
                    .unwrap();
                let rhs = Mat::from_fn(n, 2, |i, j| (i + j + 1) as f64);
                let mut x = rhs.clone();
                let mut memory = MemBuffer::new(symbolic.solve_in_place_scratch::<f64>(2, par));
                factors.solve_in_place_with_conj(
                    Conj::No,
                    x.as_mut(),
                    par,
                    MemStack::new(&mut memory),
                );
                assert!((&matrix * &x - &rhs).norm_max() < 1e-11);
            }
        }
    }
}

#[test]
fn faer_nonfused_adversarial_dispatch_routes() {
    adversarial();
}
#[test]
fn faer_nonfused_lu_svd_evd() {
    faer_nonfused_decompositions();
}
#[test]
fn faer_nonfused_simplicial_supernodal() {
    faer_nonfused_sparse_modes();
}
