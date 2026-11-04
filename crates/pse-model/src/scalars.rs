// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Validated single-value setting domains (ADR-0116 Outcome 8). A value outside its domain
//! cannot be constructed or decoded: `try_new` and serde decoding both refuse it with the
//! type's typed cause (`ToleranceError`, `FractionError`, `PositiveCountError`,
//! `FiniteBoundError`), and each type's JSON Schema states its constraint. Rules that relate
//! several fields, or a field to the environment, stay admission rules with typed reasons.
use nutype::nutype;
#[macro_use]
mod projection;

projected_scalar! {
/// A finite, strictly positive tolerance, budget or push.
#[nutype(
    const_fn,
    validate(finite, greater = 0.0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
        Display
    ),
    derive_unchecked(schemars::JsonSchema)
)]
pub struct Tolerance(#[schemars(extend("exclusiveMinimum" = 0.0))] f64);
}

projected_scalar! {
/// A finite fraction in (0, 1]: a damping factor, a step fraction or a relative push.
#[nutype(
    const_fn,
    validate(finite, greater = 0.0, less_or_equal = 1.0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
        Display
    ),
    derive_unchecked(schemars::JsonSchema)
)]
pub struct Fraction(#[schemars(extend("exclusiveMinimum" = 0.0, "maximum" = 1.0))] f64);
}

projected_scalar! {
/// A strictly positive count: a dimension, a thread count or an iteration budget.
#[nutype(
    const_fn,
    validate(greater = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize,
        Display
    ),
    derive_unchecked(schemars::JsonSchema)
)]
pub struct PositiveCount(#[schemars(extend("minimum" = 1))] usize);
}

projected_scalar! {
/// A finite real of either sign: a bound, a weight or a stated value.
#[nutype(
    const_fn,
    validate(finite),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
        Display
    ),
    derive_unchecked(schemars::JsonSchema)
)]
pub struct FiniteBound(f64);
}

/// A validated scalar from a literal, checked at compile time: an invalid literal does not
/// compile, so defaults never need a runtime refusal path.
///
/// ```
/// use pse_model::{scalar, scalars::Tolerance};
/// assert_eq!(scalar!(Tolerance(1e-9)).into_inner(), 1e-9);
/// ```
///
/// ```compile_fail
/// use pse_model::{scalar, scalars::Tolerance};
/// let _ = scalar!(Tolerance(-1.0));
/// ```
#[macro_export]
macro_rules! scalar {
    ($ty:ident($value:expr)) => {{
        #[allow(
            clippy::panic,
            reason = "evaluated at compile time: an invalid literal fails to compile"
        )]
        const VALUE: $ty = match $ty::try_new($value) {
            Ok(value) => value,
            Err(_) => panic!(concat!("invalid ", stringify!($ty), " literal")),
        };
        VALUE
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_scalar_domains_match_checked_constructors_for_exact_ieee_values() {
        use crate::boundary_projection::{Float64Domain, ProjectionGraph, ProjectionRecord};
        let samples = [
            0.0,
            -0.0,
            f64::from_bits(1),
            -f64::from_bits(1),
            1.0,
            f64::from_bits(1.0_f64.to_bits() + 1),
            f64::MAX,
            -f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0xfff0_0000_0000_0001),
        ];
        for value in samples {
            assert_eq!(
                Tolerance::try_new(value).is_ok(),
                Float64Domain::PositiveFinite.contains(value)
            );
            assert_eq!(
                Fraction::try_new(value).is_ok(),
                Float64Domain::PositiveUnit.contains(value)
            );
            assert_eq!(
                FiniteBound::try_new(value).is_ok(),
                Float64Domain::Finite.contains(value)
            );
            assert_eq!(
                Tolerance::from_projection_fields((value,)).is_ok(),
                Tolerance::try_new(value).is_ok()
            );
            assert_eq!(
                Fraction::from_projection_fields((value,)).is_ok(),
                Fraction::try_new(value).is_ok()
            );
            if let Ok(native) = FiniteBound::from_projection_fields((value,)) {
                assert_eq!(native.into_projection_fields().0.to_bits(), value.to_bits());
            } else {
                assert!(FiniteBound::try_new(value).is_err());
            }
        }
        assert!(PositiveCount::from_projection_fields((0,)).is_err());
        assert_eq!(
            PositiveCount::from_projection_fields((usize::MAX,))
                .unwrap()
                .into_inner(),
            usize::MAX
        );
        for graph in [
            ProjectionGraph::of::<Tolerance>(),
            ProjectionGraph::of::<Fraction>(),
            ProjectionGraph::of::<FiniteBound>(),
            ProjectionGraph::of::<PositiveCount>(),
        ] {
            graph.unwrap();
        }
    }

    #[test]
    fn scalar_domains_refuse_values_outside_them() {
        assert_eq!(
            Tolerance::try_new(0.0),
            Err(ToleranceError::GreaterViolated)
        );
        assert_eq!(
            Tolerance::try_new(f64::NAN),
            Err(ToleranceError::FiniteViolated)
        );
        assert_eq!(
            Fraction::try_new(1.5),
            Err(FractionError::LessOrEqualViolated)
        );
        assert_eq!(
            PositiveCount::try_new(0),
            Err(PositiveCountError::GreaterViolated)
        );
        assert_eq!(
            FiniteBound::try_new(f64::INFINITY),
            Err(FiniteBoundError::FiniteViolated)
        );
        assert_eq!(scalar!(Fraction(1.0)).into_inner(), 1.0);
        assert_eq!(scalar!(PositiveCount(3)).into_inner(), 3);
    }

    #[test]
    fn scalar_schemas_state_their_domains() {
        let schema = serde_json::to_value(schemars::schema_for!(Fraction)).unwrap();
        assert_eq!(schema["exclusiveMinimum"], 0.0);
        assert_eq!(schema["maximum"], 1.0);
        let schema = serde_json::to_value(schemars::schema_for!(PositiveCount)).unwrap();
        assert_eq!(schema["minimum"], 1);
    }
}
