// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Float canonicalization for the hashing path only (blueprint §5.3 step 5, ADR-0030).
//!
//! Two rules, and the second is the one that surprises people:
//!
//! - **Every NaN hashes as one NaN.** IEEE-754 leaves 2^52 - 1 payloads free per sign for
//!   `f64`; an Arrow kernel, an FMA and a solver may each produce a different one from the
//!   same computation. Hashing the payload would make content identity depend on which
//!   library produced the value.
//! - **`-0.0` is preserved.** The sign of zero is meaningful in Arrow's `totalOrder`
//!   (`null, -1, -0, 0, NaN`, nulls first) and in stored data, so it is data, not noise.
//!   ADR-0030 records that the preservation claim stops at the engine boundary: DataFusion
//!   `GROUP BY`, `DISTINCT` and hash joins merge `-0.0` with `+0.0` and treat NaN as
//!   self-equal, which is why a rule plan never uses a `Float64` column as a distinct or
//!   join key (§14.2 rule 7) and why §19.2 records the collapse for analytics.
//!
//! Both functions are applied to a hashing copy. Data on its way to a solver keeps its
//! NaN payload, because a NaN's provenance is a diagnostic (§18.2).

/// The `f64` quiet NaN with zero payload and positive sign: every `f64` NaN hashes as this.
pub const CANONICAL_F64_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

/// The `f32` quiet NaN with zero payload and positive sign: every `f32` NaN hashes as this.
pub const CANONICAL_F32_NAN_BITS: u32 = 0x7fc0_0000;

/// The bits an `f64` contributes to a hash.
///
/// Every NaN maps to [`CANONICAL_F64_NAN_BITS`]; every other value, `-0.0` included, maps
/// to its own `to_bits`.
///
/// ```
/// use pse_ids::canonical_f64_bits;
///
/// assert_eq!(canonical_f64_bits(f64::NAN), canonical_f64_bits(-f64::NAN));
/// assert_ne!(canonical_f64_bits(-0.0), canonical_f64_bits(0.0));
/// assert_eq!(canonical_f64_bits(1.5), 1.5_f64.to_bits());
/// ```
pub fn canonical_f64_bits(v: f64) -> u64 {
    if v.is_nan() {
        CANONICAL_F64_NAN_BITS
    } else {
        v.to_bits()
    }
}

/// The bits an `f32` contributes to a hash; the [`canonical_f64_bits`] rules at 32 bits.
///
/// ```
/// use pse_ids::canonical_f32_bits;
///
/// assert_eq!(canonical_f32_bits(f32::NAN), 0x7fc0_0000);
/// assert_eq!(canonical_f32_bits(-0.0), (-0.0_f32).to_bits());
/// ```
pub fn canonical_f32_bits(v: f32) -> u32 {
    if v.is_nan() {
        CANONICAL_F32_NAN_BITS
    } else {
        v.to_bits()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// Every bit pattern that is a NaN, whatever its sign or payload.
    fn nan_variants() -> Vec<f64> {
        vec![
            f64::NAN,
            -f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0001),
            f64::from_bits(0xfff8_0000_0000_0000),
            f64::from_bits(0x7ff0_0000_0000_0001),
            f64::from_bits(0xffff_ffff_ffff_ffff),
        ]
    }

    #[test]
    fn every_nan_payload_hashes_as_one_pattern() {
        for value in nan_variants() {
            assert!(value.is_nan());
            assert_eq!(canonical_f64_bits(value), CANONICAL_F64_NAN_BITS);
        }
    }

    #[test]
    fn signed_zero_is_preserved_and_distinguished() {
        assert_eq!(canonical_f64_bits(-0.0), 0x8000_0000_0000_0000);
        assert_eq!(canonical_f64_bits(0.0), 0x0000_0000_0000_0000);
        assert_eq!(canonical_f32_bits(-0.0), 0x8000_0000);
        assert_eq!(canonical_f32_bits(0.0), 0x0000_0000);
    }

    #[test]
    fn f32_nan_variants_collapse_too() {
        for bits in [0x7fc0_0001_u32, 0xffc0_0000, 0x7f80_0001, 0xffff_ffff] {
            assert_eq!(
                canonical_f32_bits(f32::from_bits(bits)),
                CANONICAL_F32_NAN_BITS
            );
        }
    }

    proptest! {
        /// NaN in, one pattern out; anything else in, `to_bits` out.
        #[test]
        fn f64_canonicalization_is_nan_collapse_and_otherwise_identity(bits in any::<u64>()) {
            let value = f64::from_bits(bits);
            if value.is_nan() {
                prop_assert_eq!(canonical_f64_bits(value), CANONICAL_F64_NAN_BITS);
            } else {
                prop_assert_eq!(canonical_f64_bits(value), bits);
            }
        }

        /// The same statement at 32 bits.
        #[test]
        fn f32_canonicalization_is_nan_collapse_and_otherwise_identity(bits in any::<u32>()) {
            let value = f32::from_bits(bits);
            if value.is_nan() {
                prop_assert_eq!(canonical_f32_bits(value), CANONICAL_F32_NAN_BITS);
            } else {
                prop_assert_eq!(canonical_f32_bits(value), bits);
            }
        }

        /// Two values that are not NaN hash equally only when their bits agree, so `-0.0`
        /// and `0.0` stay apart under every generated pair.
        #[test]
        fn distinct_non_nan_bits_stay_distinct(a in any::<u64>(), b in any::<u64>()) {
            let (left, right) = (f64::from_bits(a), f64::from_bits(b));
            prop_assume!(!left.is_nan() && !right.is_nan());
            prop_assert_eq!(canonical_f64_bits(left) == canonical_f64_bits(right), a == b);
        }
    }
}
