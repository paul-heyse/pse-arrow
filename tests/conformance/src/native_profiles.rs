// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compile/link and representation controls only; convergence belongs to M22.
#[test]
fn coefficient_and_cone_profiles_link() {
    let _ = highs::RowProblem::new();
    let matrix = clarabel::algebra::CscMatrix::<f64>::identity(2);
    assert_eq!(matrix.n, 2);
    assert_eq!(size_of::<highs_sys::HighsInt>(), 4);
    std::hint::black_box(highs_sys::Highs_create);
    std::hint::black_box(highs_sys::Highs_destroy);
    std::hint::black_box(highs_sys::Highs_setBoolOptionValue);
}

#[test]
fn kinsol_abi_profile_is_double_precision() {
    assert_eq!(size_of::<sundials_sys::sunrealtype>(), 8);
    assert_eq!(size_of::<sundials_sys::sunindextype>(), 8);
    std::hint::black_box(sundials_sys::KINCreate);
    std::hint::black_box(sundials_sys::KINInit);
    std::hint::black_box(sundials_sys::KINSetUserData);
    std::hint::black_box(sundials_sys::KINSetConstraints);
    std::hint::black_box(sundials_sys::KINSetJacFn);
    std::hint::black_box(sundials_sys::KINSetNumMaxIters);
    std::hint::black_box(sundials_sys::KINGetNumNonlinSolvIters);
}

#[test]
fn dynamic_faer_builder_profile_compiles() {
    // A builder is an interface control; no integration trajectory is claimed.
    let _builder = diffsol::OdeBuilder::<diffsol::FaerMat<f64>>::new();
}
