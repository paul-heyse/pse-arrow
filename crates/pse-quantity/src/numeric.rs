// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked scalar conversion and comparison helpers (blueprint §7.3).

/// Return an integer only when the finite floating value denotes that exact integer.
#[expect(
    clippy::cast_possible_truncation,
    reason = "finite integral value is range-checked before casting"
)]
pub fn exact_i64_from_f64(value: f64) -> Option<i64> {
    if value.is_finite()
        && value.fract() == 0.0
        && (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value)
    {
        Some(value as i64)
    } else {
        None
    }
}
/// Return a float only when converting it back reproduces the original integer.
#[expect(
    clippy::cast_precision_loss,
    reason = "the exact inverse conversion rejects every precision loss"
)]
pub fn exact_f64_from_i64(value: i64) -> Option<f64> {
    let float = value as f64;
    (exact_i64_from_f64(float) == Some(value)).then_some(float)
}
/// The finite comparison contract of blueprint §7.3.
/// Invalid tolerances and non-finite inputs/intermediates never compare equal.
pub fn approx_eq(a: f64, b: f64, atol: f64, rtol: f64) -> bool {
    if !a.is_finite()
        || !b.is_finite()
        || !atol.is_finite()
        || !rtol.is_finite()
        || atol < 0.0
        || rtol < 0.0
    {
        return false;
    }
    let difference = (a - b).abs();
    let tolerance = atol + rtol * a.abs().max(b.abs());
    difference.is_finite() && tolerance.is_finite() && difference <= tolerance
}
/// Mathematical zero, preserving the original value's sign in all storage paths.
pub fn is_zero(value: f64) -> bool {
    value == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions_check_boundaries_and_exact_integer_round_trips() {
        assert_eq!(
            exact_f64_from_i64(i64::MIN),
            Some(-9_223_372_036_854_775_808.0)
        );
        assert_eq!(exact_f64_from_i64(i64::MAX), None);
        assert_eq!(exact_f64_from_i64(9_007_199_254_740_993), None);
        assert_eq!(
            exact_f64_from_i64(9_007_199_254_740_994),
            Some(9_007_199_254_740_994.0)
        );
        for value in [f64::NAN, f64::INFINITY, 1.5, 9_223_372_036_854_775_808.0] {
            assert_eq!(exact_i64_from_f64(value), None);
        }
    }
    #[test]
    fn finite_comparison_does_not_accept_nan_or_infinity() {
        assert!(approx_eq(1.0, 1.0001, 0.001, 0.0));
        assert!(!approx_eq(f64::NAN, f64::NAN, 1.0, 1.0));
        assert!(!approx_eq(f64::INFINITY, f64::INFINITY, 1.0, 1.0));
        assert!(!approx_eq(1.0, 1.0, -1.0, 0.0));
        assert!(is_zero(-0.0));
        assert!(!is_zero(f64::NAN));
    }
}
