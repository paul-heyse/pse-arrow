// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Algebraic and storage admission properties on bounded exact exponents.
use proptest::prelude::*;
use pse_quantity::{DimensionVector, Ratio};
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]
    #[test]
    fn multiplication_division_and_canonical_bytes_round_trip(left in prop::array::uniform8(-20_i16..20), right in prop::array::uniform8(-20_i16..20)) {
        let make=|parts:[i16;8]|->Result<DimensionVector,pse_quantity::DimensionError>{let mut exponents=[Ratio::ZERO;8];for (slot,numerator) in exponents.iter_mut().zip(parts){*slot=Ratio::new(i32::from(numerator),1)?;}Ok(DimensionVector::new(exponents))};
        let left=make(left).expect("bounded exponents");let right=make(right).expect("bounded exponents");
        prop_assert_eq!(left.mul(&right).expect("bounded sum").div(&right).expect("exact subtraction"),left);
        prop_assert_eq!(DimensionVector::from_canonical_bytes(&left.canonical_bytes()).expect("canonical storage"),left);
    }
}
