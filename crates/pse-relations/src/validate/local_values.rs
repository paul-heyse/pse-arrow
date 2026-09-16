// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared scalar extension predicates for generated construction and imported values.

pub(crate) const BOUND_ERROR: &str =
    "finite bound requires a finite value; unbounded requires an absent value";
pub(crate) const DIMENSION_ERROR: &str =
    "dimension exponent must be reduced with a positive denominator";
pub(crate) const SPAN_ERROR: &str =
    "source span requires bounded nonnegative offsets and start <= end";
pub(crate) const QUANTITY_ERROR: &str =
    "quantity value requires a finite value and explicit quantity and unit identities";

pub(crate) fn bound(kind: &str, value: Option<f64>) -> bool {
    matches!((kind, value), ("finite", Some(value)) if value.is_finite())
        || matches!((kind, value), ("unbounded", None))
}

pub(crate) fn dimension_exponent(num: i64, den: i64) -> bool {
    if den <= 0 {
        return false;
    }
    let (mut one, mut two) = (num.unsigned_abs(), den.unsigned_abs());
    while two != 0 {
        (one, two) = (two, one % two);
    }
    one == 1
}

pub(crate) const fn source_span(start: i64, end: i64) -> bool {
    pse_schema::model::IntegerRange::SOURCE_OFFSET.contains(start)
        && pse_schema::model::IntegerRange::SOURCE_OFFSET.contains(end)
        && start <= end
}

pub(crate) const fn quantity(value: f64) -> bool {
    value.is_finite()
}
