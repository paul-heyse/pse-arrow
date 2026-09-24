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

/// Executable ordered scalar arithmetic, independent of literal-fold eligibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarKernel {
    /// Ordered addition.
    Add,
    /// Ordered subtraction.
    Sub,
    /// Ordered multiplication.
    Mul,
    /// Checked division.
    Div,
    /// Exponentiation; evaluation does not imply fold eligibility.
    Pow,
    /// Sign inversion.
    Neg,
    /// Absolute value.
    Abs,
    /// Nonnegative square root.
    Sqrt,
}
/// Checked arithmetic refusal; consumers attach their own node/contract context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticFault {
    /// Exact integer result exceeds the intermediate width.
    Overflow,
    /// Divisor is mathematical zero.
    ZeroDivisor,
    /// Input or result is outside the finite domain.
    Domain,
    /// Wrong operand count.
    Arity,
}
impl ScalarKernel {
    /// Resolve a scalar arithmetic implementation, not an opcode display name.
    pub const fn for_opcode(opcode: crate::Opcode) -> Option<Self> {
        use crate::Opcode;
        Some(match opcode {
            Opcode::Add => Self::Add,
            Opcode::Sub => Self::Sub,
            Opcode::Mul => Self::Mul,
            Opcode::Div => Self::Div,
            Opcode::Pow => Self::Pow,
            Opcode::Neg => Self::Neg,
            Opcode::Abs => Self::Abs,
            Opcode::Sqrt => Self::Sqrt,
            _ => return None,
        })
    }
    /// Exact i128 evaluation. `None` means nonintegral division or no integer sqrt route.
    /// # Errors
    /// Overflow, zero divisor, or wrong operand arity.
    pub fn integer(self, values: &[i128]) -> Result<Option<i128>, ArithmeticFault> {
        use ArithmeticFault as Fault;
        let value = match (self, values) {
            (Self::Add, [a, b]) => a.checked_add(*b),
            (Self::Sub, [a, b]) => a.checked_sub(*b),
            (Self::Mul, [a, b]) => a.checked_mul(*b),
            (Self::Div, [a, b]) => {
                let remainder = a.checked_rem(*b).ok_or(if *b == 0 {
                    Fault::ZeroDivisor
                } else {
                    Fault::Overflow
                })?;
                if remainder != 0 {
                    return Ok(None);
                }
                a.checked_div(*b)
            }
            (Self::Pow, [a, b]) => u32::try_from(*b)
                .ok()
                .and_then(|power| a.checked_pow(power)),
            (Self::Neg, [a]) => a.checked_neg(),
            (Self::Abs, [a]) => a.checked_abs(),
            (Self::Sqrt, [_]) => return Ok(None),
            _ => return Err(Fault::Arity),
        };
        value.map(Some).ok_or(Fault::Overflow)
    }
    /// Ordered Float64 evaluation without contraction or silent nonfinite results.
    /// # Errors
    /// Domain/nonfinite result, zero divisor, or wrong operand arity.
    pub fn real(self, values: &[f64]) -> Result<f64, ArithmeticFault> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ArithmeticFault::Domain);
        }
        let value = match (self, values) {
            (Self::Add, [a, b]) => a + b,
            (Self::Sub, [a, b]) => a - b,
            (Self::Mul, [a, b]) => a * b,
            (Self::Div, [_, b]) if *b == 0.0 => return Err(ArithmeticFault::ZeroDivisor),
            (Self::Div, [a, b]) => a / b,
            (Self::Pow, [a, b]) => a.powf(*b),
            (Self::Neg, [a]) => -a,
            (Self::Abs, [a]) => a.abs(),
            (Self::Sqrt, [a]) => a.sqrt(),
            _ => return Err(ArithmeticFault::Arity),
        };
        value
            .is_finite()
            .then_some(value)
            .ok_or(ArithmeticFault::Domain)
    }
}

#[cfg(test)]
mod graph_contract_unit {
    use super::*;
    #[test]
    fn exact_kernel_keeps_wide_results_and_refuses_overflow_without_a_float_roundtrip() {
        assert_eq!(
            ScalarKernel::Div.integer(&[i128::from(i64::MIN), -1]),
            Ok(Some(9_223_372_036_854_775_808))
        );
        assert_eq!(
            ScalarKernel::Div.integer(&[i128::MIN, -1]),
            Err(ArithmeticFault::Overflow)
        );
        assert_eq!(ScalarKernel::Div.integer(&[7, 2]), Ok(None));
        assert_eq!(
            ScalarKernel::Div.integer(&[7, 0]),
            Err(ArithmeticFault::ZeroDivisor)
        );
        assert_eq!(
            ScalarKernel::Mul.integer(&[i128::MAX, 2]),
            Err(ArithmeticFault::Overflow)
        );
        assert_eq!(
            ScalarKernel::Neg.real(&[0.0]).unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(ScalarKernel::Pow.integer(&[3, 4]), Ok(Some(81)));
    }
}
