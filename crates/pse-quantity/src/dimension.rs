// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Rational exponents over the eight base dimensions (blueprint §4.4, §6.2, §8.1).
//!
//! A dimension vector is eight [`Ratio`]s, one per [`BaseDimension`], in the order
//! §6.2's `reference.dimensions` declares. Exponents are rational rather than integral
//! because roots occur in real correlations — a Reynolds-number correlation raises a
//! dimensional group to `1/2`, and rounding that to `0` or `1` would silently change the
//! physics.
//!
//! Three things this module is deliberately strict about:
//!
//! - **One representation per value.** `2/4` and `1/2` are the same exponent, so
//!   [`Ratio`] stores only the reduced form with a positive denominator, and zero is
//!   always `0/1`. [`DimensionVector::canonical_bytes`] can therefore be compared
//!   bytewise, which is what makes it usable as a key and as a hash input.
//! - **Arithmetic is checked.** [`Ratio`] is two `i16`s, matching the
//!   `pse.dimension_vector` storage of §4.4; a product that leaves that range is an error
//!   rather than a wrap, because a wrapped exponent is a wrong answer that type-checks.
//! - **`Display` is for people.** `L·T^-1` is a rendering, never a hash input and never a
//!   key (§5.3's closing rule); [`DimensionVector::canonical_bytes`] is the machine form.

use serde::{Deserialize, Serialize};

use crate::error::DimensionError;

/// The eight base dimensions, in `reference.dimensions` ordinal order (blueprint §6.2).
///
/// `Currency` is a base dimension because costing is part of the model: a `USD_2018`
/// value has a dimension, and conversion between years is a scale by a cost index
/// (§19.5), not a unit-free multiplication.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum BaseDimension {
    /// Length, SI base unit metre.
    Length = 0,
    /// Mass, SI base unit kilogram.
    Mass = 1,
    /// Time, SI base unit second.
    Time = 2,
    /// Thermodynamic temperature, SI base unit kelvin.
    Temperature = 3,
    /// Amount of substance, SI base unit mole.
    Amount = 4,
    /// Electric current, SI base unit ampere.
    Current = 5,
    /// Luminous intensity, SI base unit candela.
    LuminousIntensity = 6,
    /// Currency, with a per-year unit anchored through the cost indices of §19.5.
    Currency = 7,
}

impl BaseDimension {
    /// Every base dimension, in `reference.dimensions` ordinal order.
    pub const ALL: &'static [Self] = &[
        Self::Length,
        Self::Mass,
        Self::Time,
        Self::Temperature,
        Self::Amount,
        Self::Current,
        Self::LuminousIntensity,
        Self::Currency,
    ];

    /// How many base dimensions there are; the width of a [`DimensionVector`].
    pub const COUNT: usize = 8;

    /// The registry spelling of this base dimension (`reference.dimensions.name`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Temperature => "temperature",
            Self::Amount => "amount",
            Self::Current => "current",
            Self::LuminousIntensity => "luminous_intensity",
            Self::Currency => "currency",
        }
    }

    /// The base dimension with this registry spelling, if any.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == text)
    }

    /// The `reference.dimensions.ordinal` of this base dimension, and its slot in a
    /// [`DimensionVector`].
    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    /// The base dimension occupying this slot, if the slot exists.
    pub const fn from_ordinal(ordinal: u8) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Length),
            1 => Some(Self::Mass),
            2 => Some(Self::Time),
            3 => Some(Self::Temperature),
            4 => Some(Self::Amount),
            5 => Some(Self::Current),
            6 => Some(Self::LuminousIntensity),
            7 => Some(Self::Currency),
            _ => None,
        }
    }

    /// The conventional one-or-two character symbol used when rendering a dimension.
    ///
    /// Rendering only: `Display` output is never hashed and never a key.
    const fn symbol(self) -> &'static str {
        match self {
            Self::Length => "L",
            Self::Mass => "M",
            Self::Time => "T",
            Self::Temperature => "Θ",
            Self::Amount => "N",
            Self::Current => "I",
            Self::LuminousIntensity => "J",
            Self::Currency => "$",
        }
    }
}

impl core::fmt::Display for BaseDimension {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The serialization shape of a [`Ratio`]: the two fields, unvalidated.
///
/// [`Ratio`] deserializes through this so that a stored pair cannot install an unreduced
/// or negative-denominator value behind the invariant.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "Ratio")]
struct RatioRepr {
    num: i16,
    den: i16,
}

/// A rational exponent in canonical reduced form (blueprint §4.4 `pse.dimension_vector`).
///
/// The invariant is total: `den > 0`, `gcd(|num|, den) == 1`, and zero is exactly `0/1`.
/// Construction goes through [`Ratio::new`] or [`Ratio::from_parts`], both of which reject
/// anything else, and deserialization re-validates. Two `Ratio`s are equal exactly when
/// they are the same rational number.
///
/// `Ord` is the lexicographic order of the canonical `(num, den)` pair, not numeric order:
/// it exists so that a `Ratio` can key a `BTreeMap` deterministically. Compare magnitudes
/// with [`Ratio::checked_sub`] and [`Ratio::is_zero`] rather than with `<`.
///
/// ```
/// use pse_quantity::Ratio;
///
/// assert_eq!(Ratio::new(2, 4)?, Ratio::new(1, 2)?);
/// assert_eq!(Ratio::new(-1, -2)?, Ratio::new(1, 2)?);
/// assert_eq!(Ratio::new(0, 5)?, Ratio::ZERO);
/// assert!(Ratio::new(1, 0).is_err());
/// # Ok::<(), pse_quantity::DimensionError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(into = "RatioRepr", try_from = "RatioRepr")]
pub struct Ratio {
    num: i16,
    den: i16,
}

impl Ratio {
    /// The exponent zero, canonically `0/1`.
    pub const ZERO: Self = Self { num: 0, den: 1 };

    /// The exponent one, canonically `1/1`.
    pub const ONE: Self = Self { num: 1, den: 1 };

    /// The reduced rational `num/den`.
    ///
    /// # Errors
    ///
    /// [`DimensionError::ZeroDenominator`] when `den` is zero, and
    /// [`DimensionError::Overflow`] when the reduced form does not fit two `i16`s.
    pub fn new(num: i32, den: i32) -> Result<Self, DimensionError> {
        reduce(i64::from(num), i64::from(den))
    }

    /// A [`Ratio`] from a pair that is already in canonical form.
    ///
    /// This is the read path for stored bytes: it validates rather than normalizes, so a
    /// stored `2/4` is reported instead of being silently accepted as `1/2`.
    ///
    /// # Errors
    ///
    /// [`DimensionError::ZeroDenominator`] when `den` is zero and
    /// [`DimensionError::NotReduced`] when the pair is not the canonical form of its value.
    pub fn from_parts(num: i16, den: i16) -> Result<Self, DimensionError> {
        if den == 0 {
            return Err(DimensionError::ZeroDenominator);
        }
        let canonical = reduce(i64::from(num), i64::from(den))?;
        if canonical.num != num || canonical.den != den {
            return Err(DimensionError::NotReduced { num, den });
        }
        Ok(canonical)
    }

    /// The numerator; its sign is the sign of the exponent.
    pub const fn num(self) -> i16 {
        self.num
    }

    /// The denominator, always positive.
    pub const fn den(self) -> i16 {
        self.den
    }

    /// Is this the zero exponent?
    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    /// Is this exponent a whole number?
    pub const fn is_integer(self) -> bool {
        self.den == 1
    }

    /// The sum of two exponents, reduced.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when the reduced sum does not fit two `i16`s.
    pub fn checked_add(self, other: Self) -> Result<Self, DimensionError> {
        let num =
            i64::from(self.num) * i64::from(other.den) + i64::from(other.num) * i64::from(self.den);
        reduce(num, i64::from(self.den) * i64::from(other.den))
    }

    /// The difference of two exponents, reduced.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when the reduced difference does not fit two `i16`s.
    pub fn checked_sub(self, other: Self) -> Result<Self, DimensionError> {
        let num =
            i64::from(self.num) * i64::from(other.den) - i64::from(other.num) * i64::from(self.den);
        reduce(num, i64::from(self.den) * i64::from(other.den))
    }

    /// The product of two exponents, reduced.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when the reduced product does not fit two `i16`s.
    pub fn checked_mul(self, other: Self) -> Result<Self, DimensionError> {
        reduce(
            i64::from(self.num) * i64::from(other.num),
            i64::from(self.den) * i64::from(other.den),
        )
    }
}

impl core::fmt::Display for Ratio {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl From<Ratio> for RatioRepr {
    fn from(value: Ratio) -> Self {
        Self {
            num: value.num,
            den: value.den,
        }
    }
}

impl TryFrom<RatioRepr> for Ratio {
    type Error = DimensionError;

    fn try_from(value: RatioRepr) -> Result<Self, Self::Error> {
        Self::from_parts(value.num, value.den)
    }
}

/// The greatest common divisor of two magnitudes; `gcd(a, 0) == a`.
const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

/// Reduces `num/den` to canonical form, or says why it cannot.
fn reduce(num: i64, den: i64) -> Result<Ratio, DimensionError> {
    if den == 0 {
        return Err(DimensionError::ZeroDenominator);
    }
    if num == 0 {
        return Ok(Ratio::ZERO);
    }
    let negative = (num < 0) != (den < 0);
    let divisor = gcd(num.unsigned_abs(), den.unsigned_abs());
    // `den != 0`, so `divisor >= 1` and both divisions are exact.
    let magnitude = num.unsigned_abs() / divisor;
    let denominator = den.unsigned_abs() / divisor;
    // Apply the sign before narrowing: -32768 is representable although +32768 is not.
    let signed = i64::try_from(magnitude).map_err(|_| DimensionError::Overflow { num, den })?;
    let signed = if negative { -signed } else { signed };
    let numerator = i16::try_from(signed).map_err(|_| DimensionError::Overflow { num, den })?;
    let denominator =
        i16::try_from(denominator).map_err(|_| DimensionError::Overflow { num, den })?;
    Ok(Ratio {
        num: numerator,
        den: denominator,
    })
}

/// Rational exponents over the eight base dimensions (blueprint §4.4, §8.1).
///
/// The slots are indexed by [`BaseDimension`] and stored in its ordinal order, which is
/// also the order of the `FixedSizeList<Struct<num, den>, 8>` storage of
/// `pse.dimension_vector`.
///
/// Dimension arithmetic is *not* physical typing: `dimensionless` is the dimension of a
/// mole fraction and of a Reynolds number alike, and two quantities with the same
/// dimension vector may still be incompatible (§8.1). This type answers only the
/// dimensional half; the complete answer is a `QuantityType`.
///
/// ```
/// use pse_quantity::{BaseDimension, DimensionVector, Ratio};
///
/// let velocity = DimensionVector::base(BaseDimension::Length)
///     .div(&DimensionVector::base(BaseDimension::Time))?;
/// assert_eq!(velocity.exponent(BaseDimension::Length), Ratio::ONE);
/// assert_eq!(velocity.to_string(), "L·T^-1");
/// # Ok::<(), pse_quantity::DimensionError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DimensionVector([Ratio; BaseDimension::COUNT]);

impl DimensionVector {
    /// The dimension of a pure number: every exponent zero.
    pub const DIMENSIONLESS: Self = Self([Ratio::ZERO; BaseDimension::COUNT]);

    /// How many bytes [`DimensionVector::canonical_bytes`] produces.
    pub const CANONICAL_BYTES: usize = 4 * BaseDimension::COUNT;

    /// A dimension vector from its eight exponents, in [`BaseDimension`] ordinal order.
    pub const fn new(exponents: [Ratio; BaseDimension::COUNT]) -> Self {
        Self(exponents)
    }

    /// The dimension of one base dimension raised to the first power.
    pub fn base(dimension: BaseDimension) -> Self {
        let mut exponents = [Ratio::ZERO; BaseDimension::COUNT];
        exponents[dimension.ordinal() as usize] = Ratio::ONE;
        Self(exponents)
    }

    /// The exponent of one base dimension.
    pub fn exponent(&self, dimension: BaseDimension) -> Ratio {
        self.0[dimension.ordinal() as usize]
    }

    /// The eight exponents, in [`BaseDimension`] ordinal order.
    pub const fn exponents(&self) -> &[Ratio; BaseDimension::COUNT] {
        &self.0
    }

    /// Is every exponent zero?
    pub fn is_dimensionless(&self) -> bool {
        self.0.iter().all(|exponent| exponent.is_zero())
    }

    /// The dimension of a product: exponents added.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when a resulting exponent leaves the `i16` pair.
    pub fn mul(&self, other: &Self) -> Result<Self, DimensionError> {
        self.zip_with(other, Ratio::checked_add)
    }

    /// The dimension of a quotient: exponents subtracted.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when a resulting exponent leaves the `i16` pair.
    pub fn div(&self, other: &Self) -> Result<Self, DimensionError> {
        self.zip_with(other, Ratio::checked_sub)
    }

    /// The dimension of a power: every exponent multiplied by `exponent`.
    ///
    /// # Errors
    ///
    /// [`DimensionError::Overflow`] when a resulting exponent leaves the `i16` pair.
    pub fn pow(&self, exponent: Ratio) -> Result<Self, DimensionError> {
        let mut out = [Ratio::ZERO; BaseDimension::COUNT];
        for (slot, value) in out.iter_mut().zip(self.0.iter()) {
            *slot = value.checked_mul(exponent)?;
        }
        Ok(Self(out))
    }

    /// The dimension of an `n`-th root: every exponent divided by `n`.
    ///
    /// # Errors
    ///
    /// [`DimensionError::ZeroDenominator`] when `degree` is zero, and
    /// [`DimensionError::Overflow`] when a resulting exponent leaves the `i16` pair.
    pub fn root(&self, degree: u8) -> Result<Self, DimensionError> {
        self.pow(Ratio::new(1, i32::from(degree))?)
    }

    /// The canonical byte form: eight `(num, den)` pairs as little-endian `i16`s.
    ///
    /// This is the machine form — a key, a hash input, the `pse.dimension_vector` payload.
    /// Because [`Ratio`] admits one representation per value, byte equality here is value
    /// equality.
    pub fn canonical_bytes(&self) -> [u8; 32] {
        let mut out = [0_u8; 32];
        for (index, exponent) in self.0.iter().enumerate() {
            let offset = index * 4;
            out[offset..offset + 2].copy_from_slice(&exponent.num().to_le_bytes());
            out[offset + 2..offset + 4].copy_from_slice(&exponent.den().to_le_bytes());
        }
        out
    }

    /// Reads the canonical byte form, re-validating every exponent.
    ///
    /// # Errors
    ///
    /// [`DimensionError::ZeroDenominator`] or [`DimensionError::NotReduced`] when a stored
    /// pair is not the canonical form of its value.
    pub fn from_canonical_bytes(bytes: &[u8; 32]) -> Result<Self, DimensionError> {
        let mut out = [Ratio::ZERO; BaseDimension::COUNT];
        for (index, slot) in out.iter_mut().enumerate() {
            let offset = index * 4;
            let num = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            let den = i16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
            *slot = Ratio::from_parts(num, den)?;
        }
        Ok(Self(out))
    }

    /// Combines two dimension vectors slot by slot with a checked rational operation.
    fn zip_with<F>(&self, other: &Self, op: F) -> Result<Self, DimensionError>
    where
        F: Fn(Ratio, Ratio) -> Result<Ratio, DimensionError>,
    {
        let mut out = [Ratio::ZERO; BaseDimension::COUNT];
        for (index, slot) in out.iter_mut().enumerate() {
            *slot = op(self.0[index], other.0[index])?;
        }
        Ok(Self(out))
    }
}

impl core::fmt::Display for DimensionVector {
    /// Renders for a human: `L·T^-1`, `dimensionless` for the empty product.
    ///
    /// Never a hash input, never a key, never parsed back (§5.3): use
    /// [`DimensionVector::canonical_bytes`] for any of those.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_dimensionless() {
            return f.write_str("dimensionless");
        }
        let mut first = true;
        for dimension in BaseDimension::ALL {
            let exponent = self.exponent(*dimension);
            if exponent.is_zero() {
                continue;
            }
            if !first {
                f.write_str("·")?;
            }
            first = false;
            f.write_str(dimension.symbol())?;
            if exponent != Ratio::ONE {
                write!(f, "^{exponent}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BaseDimension, DimensionVector, Ratio};
    use crate::error::DimensionError;

    #[test]
    fn a_ratio_is_stored_reduced() {
        assert_eq!(Ratio::new(2, 4), Ratio::new(1, 2));
        assert_eq!(Ratio::new(3, 3), Ok(Ratio::ONE));
        assert_eq!(Ratio::new(6, 3), Ratio::new(2, 1));
        assert_eq!(Ratio::new(0, 7), Ok(Ratio::ZERO));
    }

    #[test]
    fn a_ratio_keeps_the_sign_on_the_numerator() {
        let value = Ratio::new(1, -2).expect("a nonzero denominator reduces");
        assert_eq!(value.num(), -1);
        assert_eq!(value.den(), 2);
        assert_eq!(Ratio::new(-1, -2), Ratio::new(1, 2));
    }

    #[test]
    fn a_zero_denominator_is_an_error() {
        assert_eq!(Ratio::new(1, 0), Err(DimensionError::ZeroDenominator));
        assert_eq!(
            Ratio::from_parts(1, 0),
            Err(DimensionError::ZeroDenominator)
        );
    }

    #[test]
    fn an_exponent_outside_the_i16_pair_is_an_error() {
        // 40_000 does not fit an `i16`, so the reduced form cannot be stored.
        assert!(matches!(
            Ratio::new(40_000, 1),
            Err(DimensionError::Overflow { .. })
        ));
        let big = Ratio::new(30_000, 1).expect("30000/1 fits");
        assert!(matches!(
            big.checked_add(big),
            Err(DimensionError::Overflow { .. })
        ));
        let third = Ratio::new(1, 30_000).expect("1/30000 fits");
        assert!(matches!(
            third.checked_mul(third),
            Err(DimensionError::Overflow { .. })
        ));
    }

    #[test]
    fn from_parts_rejects_an_unreduced_pair() {
        assert_eq!(
            Ratio::from_parts(2, 4),
            Err(DimensionError::NotReduced { num: 2, den: 4 })
        );
        assert_eq!(
            Ratio::from_parts(1, -2),
            Err(DimensionError::NotReduced { num: 1, den: -2 })
        );
        assert_eq!(
            Ratio::from_parts(0, 5),
            Err(DimensionError::NotReduced { num: 0, den: 5 })
        );
        assert_eq!(Ratio::from_parts(0, 1), Ok(Ratio::ZERO));
    }

    #[test]
    fn rational_arithmetic_reduces_its_results() {
        let half = Ratio::new(1, 2).expect("1/2 reduces");
        let third = Ratio::new(1, 3).expect("1/3 reduces");
        assert_eq!(half.checked_add(third), Ratio::new(5, 6));
        assert_eq!(half.checked_sub(half), Ok(Ratio::ZERO));
        assert_eq!(half.checked_mul(third), Ratio::new(1, 6));
        assert_eq!(half.checked_add(half), Ok(Ratio::ONE));
    }

    /// Velocity times time is length again: `mul` and `div` are inverses.
    #[test]
    fn multiplication_and_division_round_trip() {
        let length = DimensionVector::base(BaseDimension::Length);
        let time = DimensionVector::base(BaseDimension::Time);
        let velocity = length.div(&time).expect("L/T");
        assert_eq!(velocity.mul(&time), Ok(length));
        assert_eq!(
            Ok(velocity.exponent(BaseDimension::Time)),
            Ratio::new(-1, 1)
        );
    }

    /// An area's square root is a length: `pow` and `root` are inverses.
    #[test]
    fn powers_and_roots_round_trip() {
        let length = DimensionVector::base(BaseDimension::Length);
        let area = length.pow(Ratio::new(2, 1).expect("2/1")).expect("L^2");
        assert_eq!(Ok(area.exponent(BaseDimension::Length)), Ratio::new(2, 1));
        assert_eq!(area.root(2), Ok(length));
        assert_eq!(
            length.root(2).and_then(|half| half.pow(Ratio::new(2, 1)?)),
            Ok(length)
        );
    }

    #[test]
    fn a_zero_degree_root_is_an_error() {
        let length = DimensionVector::base(BaseDimension::Length);
        assert_eq!(length.root(0), Err(DimensionError::ZeroDenominator));
    }

    #[test]
    fn the_canonical_bytes_round_trip() {
        let squared = Ratio::new(2, 1).expect("2/1");
        let length_squared = DimensionVector::base(BaseDimension::Length)
            .pow(squared)
            .expect("L^2");
        let time_squared = DimensionVector::base(BaseDimension::Time)
            .pow(squared)
            .expect("T^2");
        let energy = DimensionVector::base(BaseDimension::Mass)
            .mul(&length_squared)
            .expect("M·L^2")
            .div(&time_squared)
            .expect("M·L^2·T^-2");
        let bytes = energy.canonical_bytes();
        assert_eq!(bytes.len(), DimensionVector::CANONICAL_BYTES);
        assert_eq!(DimensionVector::from_canonical_bytes(&bytes), Ok(energy));
    }

    #[test]
    fn the_canonical_bytes_are_little_endian_pairs() {
        let half_length = DimensionVector::base(BaseDimension::Length)
            .root(2)
            .expect("L^1/2");
        let bytes = half_length.canonical_bytes();
        assert_eq!(&bytes[0..4], &[1, 0, 2, 0]);
        assert_eq!(&bytes[4..8], &[0, 0, 1, 0]);
    }

    #[test]
    fn reading_bytes_re_validates_the_reduced_form() {
        let mut bytes = DimensionVector::DIMENSIONLESS.canonical_bytes();
        bytes[0] = 2;
        bytes[2] = 4;
        assert_eq!(
            DimensionVector::from_canonical_bytes(&bytes),
            Err(DimensionError::NotReduced { num: 2, den: 4 })
        );
        bytes[0] = 1;
        bytes[2] = 0;
        assert_eq!(
            DimensionVector::from_canonical_bytes(&bytes),
            Err(DimensionError::ZeroDenominator)
        );
    }

    #[test]
    fn dimensionless_is_the_multiplicative_identity() {
        let mass = DimensionVector::base(BaseDimension::Mass);
        assert!(DimensionVector::DIMENSIONLESS.is_dimensionless());
        assert!(!mass.is_dimensionless());
        assert_eq!(mass.mul(&DimensionVector::DIMENSIONLESS), Ok(mass));
        assert!(mass.div(&mass).expect("M/M").is_dimensionless());
    }

    #[test]
    fn display_is_for_people() {
        let velocity = DimensionVector::base(BaseDimension::Length)
            .div(&DimensionVector::base(BaseDimension::Time))
            .expect("L/T");
        assert_eq!(velocity.to_string(), "L·T^-1");
        assert_eq!(DimensionVector::DIMENSIONLESS.to_string(), "dimensionless");
        assert_eq!(
            DimensionVector::base(BaseDimension::Length)
                .root(2)
                .expect("L^1/2")
                .to_string(),
            "L^1/2"
        );
    }

    #[test]
    fn base_dimension_ordinals_match_the_registry_order() {
        assert_eq!(BaseDimension::ALL.len(), BaseDimension::COUNT);
        for (index, dimension) in BaseDimension::ALL.iter().enumerate() {
            assert_eq!(usize::from(dimension.ordinal()), index);
            assert_eq!(
                BaseDimension::from_ordinal(dimension.ordinal()),
                Some(*dimension)
            );
            assert_eq!(BaseDimension::parse(dimension.as_str()), Some(*dimension));
        }
        assert_eq!(BaseDimension::from_ordinal(8), None);
        assert_eq!(BaseDimension::parse("mols"), None);
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::Ratio;
    #[test]
    fn minimum_signed_numerator_is_valid_but_its_negation_overflows() {
        let minimum = Ratio::new(-32768, 1).expect("minimum i16");
        assert_eq!(Ratio::from_parts(i16::MIN, 1), Ok(minimum));
        assert_eq!(minimum.checked_mul(Ratio::ONE), Ok(minimum));
        assert!(Ratio::new(32768, 1).is_err());
        assert!(
            minimum
                .checked_mul(Ratio::new(-1, 1).expect("minus one"))
                .is_err()
        );
    }
}
