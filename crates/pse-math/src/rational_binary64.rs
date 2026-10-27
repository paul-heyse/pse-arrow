// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Certified conversion of a bounded exact rational to binary64.
use crate::MathError;
use symbolica::domains::{
    float::{Float, RoundingDirection},
    rational::Rational,
};

pub(crate) fn bits(value: &Rational) -> Result<usize, MathError> {
    usize::try_from(
        value
            .numerator_ref()
            .significant_bits()
            .max(value.denominator_ref().significant_bits()),
    )
    .map_err(|_| MathError::Limit("rational binary64 extent"))
}

fn precision(value: &Rational) -> Result<u32, MathError> {
    value
        .numerator_ref()
        .significant_bits()
        .checked_add(value.denominator_ref().significant_bits())
        .and_then(|bits| bits.checked_add(1078))
        .and_then(|bits| u32::try_from(bits).ok())
        .ok_or(MathError::Limit("rational binary64 precision"))
}

/// Peak mathematical storage for the two sequential directed conversions,
/// exact dyadic comparison and current numerical projection. 128 rational/
/// integer/float slots include canonicalization scratch and old/new overlap;
/// their limb extent is bounded by the same N+D+1078 precision used below.
/// Callers retain one high-water reservation across their constant population.
pub(crate) fn conversion_bytes(value: &Rational) -> Result<usize, MathError> {
    let bits = usize::try_from(precision(value)?)
        .map_err(|_| MathError::Limit("rational conversion extent"))?;
    bits.checked_add(7)
        .and_then(|n| n.checked_div(8))
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| n.checked_add(size_of::<Rational>()))
        .and_then(|n| n.checked_mul(128))
        .and_then(|n| n.checked_add(4096))
        .ok_or(MathError::Limit("rational conversion extent"))
}

/// The tight adjacent-binary64 enclosure of the exact rational. Conversion is
/// certified first; comparison with its exact dyadic image then selects the
/// outward neighbour. Overflow and subnormal/zero transitions stay explicit.
pub(crate) fn enclosure(value: &Rational) -> Result<(f64, f64), MathError> {
    let point = nearest(value)?;
    if point == f64::INFINITY {
        return Ok((f64::MAX, f64::INFINITY));
    }
    if point == f64::NEG_INFINITY {
        return Ok((f64::NEG_INFINITY, -f64::MAX));
    }
    let image = Rational::try_from(point).map_err(|error| MathError::Library(error.to_string()))?;
    Ok(match image.cmp(value) {
        std::cmp::Ordering::Less => (point, point.next_up()),
        std::cmp::Ordering::Equal => (point, point),
        std::cmp::Ordering::Greater => (point.next_down(), point),
    })
}

/// Two MPFR directed enclosures followed by Rug's nearest-even binary64
/// conversion must have bit-identical images. No 53-bit intermediate is used.
///
/// A non-dyadic n/d differs from every binary64 rounding boundary k/2^1075
/// by at least 1/(d*2^1075). With N and D the numerator/denominator bit
/// lengths, precision N+D+1078 gives absolute enclosure error <2^(-D-1076).
/// A dyadic boundary is represented exactly at this precision, including ties
/// at zero, the normal/subnormal transition and the finite/overflow boundary.
/// Numerica 3.0.1 float-mpfr delegates from_rational_round to MPFR set_q;
/// Rug 1.30.0 Float::to_f64 uses nearest-even, including gradual underflow.
pub(crate) fn nearest(value: &Rational) -> Result<f64, MathError> {
    let precision = precision(value)?;
    let lower = Float::from_rational_round(value, precision, RoundingDirection::Down)
        .as_raw()
        .to_f64();
    let upper = Float::from_rational_round(value, precision, RoundingDirection::Up)
        .as_raw()
        .to_f64();
    if lower.to_bits() != upper.to_bits() {
        return Err(MathError::Contract(
            "directed rational conversion did not certify binary64".into(),
        ));
    }
    Ok(lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::domains::integer::Integer;

    fn dyadic(n: i64, exponent: u64) -> Rational {
        (Integer::from(n), Integer::from(2).pow(exponent)).into()
    }

    #[test]
    fn rational_binary64_outward_enclosures_include_subnormal_and_overflow_values() {
        for value in [
            Rational::from((1, 3)),
            dyadic(1, 1075),
            dyadic(-1, 1075),
            Rational::from(Integer::from(2).pow(1024)),
            Rational::from(-Integer::from(2).pow(1024)),
        ] {
            let (lower, upper) = enclosure(&value).unwrap();
            if lower.is_finite() {
                assert!(Rational::try_from(lower).unwrap() <= value);
            }
            if upper.is_finite() {
                assert!(value <= Rational::try_from(upper).unwrap());
            }
            assert!(lower == upper || lower.next_up() == upper);
            assert!(conversion_bytes(&value).unwrap() > 0);
        }
    }
    #[test]
    fn rational_binary64_exact_ties_subnormals_and_overflow() {
        // Expected images follow directly from IEEE binary64 even-significand
        // ties; no output of this converter supplies an expected value.
        for (value, expected) in [
            (Rational::zero(), 0_u64),
            (dyadic(1, 1074), 1),
            (dyadic(1, 1075), 0),
            (dyadic(3, 1075), 2),
            (dyadic(-1, 1075), 1_u64 << 63),
            (dyadic(-3, 1075), (1_u64 << 63) | 2),
            (Rational::one() + dyadic(1, 53), 1.0_f64.to_bits()),
            (Rational::one() + dyadic(3, 53), 1.0_f64.to_bits() + 2),
        ] {
            assert_eq!(nearest(&value).unwrap().to_bits(), expected);
        }
        for exponent in [-1074_i32, -1022, -1021, -1, 0, 1, 1023] {
            let power: Rational = if exponent < 0 {
                (
                    Integer::one(),
                    Integer::from(2).pow(u64::from(exponent.unsigned_abs())),
                )
                    .into()
            } else {
                Integer::from(2).pow(exponent as u64).into()
            };
            let expected = if exponent < -1022 {
                1_u64 << (exponent + 1074)
            } else {
                ((exponent + 1023) as u64) << 52
            };
            assert_eq!(nearest(&power).unwrap().to_bits(), expected);
            assert_eq!(
                nearest(&(-power)).unwrap().to_bits(),
                expected | (1_u64 << 63)
            );
        }
        assert_eq!(
            nearest(&(dyadic(1, 1075) - dyadic(1, 1200)))
                .unwrap()
                .to_bits(),
            0
        );
        assert_eq!(
            nearest(&(dyadic(1, 1075) + dyadic(1, 1200)))
                .unwrap()
                .to_bits(),
            1
        );
        let min_normal = Rational::try_from(f64::MIN_POSITIVE).unwrap();
        assert_eq!(
            nearest(&(min_normal - dyadic(1, 1075))).unwrap(),
            f64::MIN_POSITIVE
        );
        let overflow: Rational = (Integer::from(2).pow(1024) - Integer::from(2).pow(970)).into();
        assert_eq!(nearest(&overflow).unwrap(), f64::INFINITY);
        assert_eq!(nearest(&(overflow - Rational::one())).unwrap(), f64::MAX);
        assert_eq!(
            nearest(&Rational::from((1, 3))).unwrap().to_bits(),
            0x3fd5_5555_5555_5555
        );
    }
}
