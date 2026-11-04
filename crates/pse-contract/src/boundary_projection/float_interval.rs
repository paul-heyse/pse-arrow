// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact finite interval declarations for original numeric setting domains.

/// A source-declared finite binary64 interval; endpoints retain their exact IEEE bits.
/// Construction records metadata. Graph admission verifies that its domain is nonempty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Float64Interval {
    lower_bits: u64,
    lower_inclusive: bool,
    upper_bits: u64,
    upper_inclusive: bool,
}

impl Float64Interval {
    /// Exact original endpoint bits and inclusion, without normalizing signed zero.
    pub const fn endpoints(self) -> (u64, bool, u64, bool) {
        (self.lower_bits, self.lower_inclusive, self.upper_bits, self.upper_inclusive)
    }
    /// Declare original finite bounds and whether each endpoint belongs to the domain.
    pub const fn new(lower: f64, lower_inclusive: bool, upper: f64, upper_inclusive: bool) -> Self {
        Self {
            lower_bits: lower.to_bits(),
            lower_inclusive,
            upper_bits: upper.to_bits(),
            upper_inclusive,
        }
    }

    /// Whether finite endpoints define at least one represented binary64 value.
    pub const fn is_valid(self) -> bool {
        let lower = f64::from_bits(self.lower_bits);
        let upper = f64::from_bits(self.upper_bits);
        lower.is_finite()
            && upper.is_finite()
            && lower <= upper
            && (lower < upper || self.lower_inclusive && self.upper_inclusive)
            && (self.bit_intervals()[0].is_some() || self.bit_intervals()[1].is_some())
    }

    /// The actual native predicate, including the numeric equality of both signed zeros.
    pub const fn contains(self, value: f64) -> bool {
        if !self.is_valid() {
            return false;
        }
        let lower = f64::from_bits(self.lower_bits);
        let upper = f64::from_bits(self.upper_bits);
        value.is_finite()
            && (if self.lower_inclusive {
                value >= lower
            } else {
                value > lower
            })
            && (if self.upper_inclusive {
                value <= upper
            } else {
                value < upper
            })
    }

    /// At most two increasing inclusive unsigned-bit intervals: positive and negative.
    /// Endpoints are derived from the same original finite numeric declaration.
    pub const fn bit_intervals(self) -> [Option<(u64, u64)>; 2] {
        let lower = f64::from_bits(self.lower_bits);
        let upper = f64::from_bits(self.upper_bits);
        if !lower.is_finite() || !upper.is_finite() || lower > upper {
            return [None, None];
        }
        let lower_open = if self.lower_inclusive { 0 } else { 1 };
        let upper_open = if self.upper_inclusive { 0 } else { 1 };
        let positive = if upper < 0.0 || upper == 0.0 && !self.upper_inclusive {
            None
        } else {
            let minimum = if lower > 0.0 {
                self.lower_bits + lower_open
            } else if lower == 0.0 && !self.lower_inclusive {
                1
            } else {
                0
            };
            let maximum = if upper > 0.0 {
                self.upper_bits - upper_open
            } else {
                0
            };
            if minimum <= maximum {
                Some((minimum, maximum))
            } else {
                None
            }
        };
        let negative = if lower > 0.0 || lower == 0.0 && !self.lower_inclusive {
            None
        } else {
            let minimum = if upper < 0.0 {
                self.upper_bits + upper_open
            } else if upper == 0.0 && !self.upper_inclusive {
                (-0.0_f64).to_bits() + 1
            } else {
                (-0.0_f64).to_bits()
            };
            let maximum = if lower < 0.0 {
                self.lower_bits - lower_open
            } else {
                (-0.0_f64).to_bits()
            };
            if minimum <= maximum {
                Some((minimum, maximum))
            } else {
                None
            }
        };
        [positive, negative]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_numeric_intervals_match_transported_bits_at_signed_endpoints() {
        let intervals = [
            Float64Interval::new(0.0, true, 1.0, true),
            Float64Interval::new(0.0, false, 1.0, false),
            Float64Interval::new(1.0, false, 2.0, true),
            Float64Interval::new(1.0, true, f64::MAX, true),
            Float64Interval::new(-2.0, true, -1.0, false),
            Float64Interval::new(-1.0, true, 1.0, false),
            Float64Interval::new(-0.0, true, 0.0, true),
        ];
        let mut samples = vec![
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            0.0,
            -0.0,
            f64::MAX,
            -f64::MAX,
            f64::from_bits(1),
            -f64::from_bits(1),
        ];
        for endpoint in [-2.0_f64, -1.0, 1.0, 2.0] {
            samples.extend([
                endpoint,
                f64::from_bits(endpoint.to_bits() - 1),
                f64::from_bits(endpoint.to_bits() + 1),
            ]);
        }
        for interval in intervals {
            assert!(interval.is_valid());
            let bit_intervals = interval.bit_intervals();
            for &value in &samples {
                let bits = value.to_bits();
                let represented = bit_intervals
                    .iter()
                    .flatten()
                    .any(|&(minimum, maximum)| (minimum..=maximum).contains(&bits));
                assert_eq!(
                    interval.contains(value),
                    represented,
                    "{interval:?} {value:?}"
                );
            }
        }
        for interval in [
            Float64Interval::new(f64::NAN, true, 1.0, true),
            Float64Interval::new(0.0, true, f64::INFINITY, true),
            Float64Interval::new(2.0, true, 1.0, true),
            Float64Interval::new(1.0, false, 1.0, true),
            Float64Interval::new(1.0, false, f64::from_bits(1.0_f64.to_bits() + 1), false),
        ] {
            assert!(!interval.is_valid(), "{interval:?}");
        }
    }
}
