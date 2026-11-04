// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit finite-number representation for original persisted numeric controls.

use crate::boundary_projection::Float64Domain;
use std::collections::BTreeMap;

/// Typed numeric control components that have an ordinary finite JSON number form.
/// Explicitly encoded IEEE values and deliberate unbounded alternatives do not
/// implement this contract merely because they contain a binary64 value.
pub trait FiniteNumbers {
    /// Whether all numeric document components have a finite number representation.
    fn finite_numbers(&self) -> bool;
}
impl FiniteNumbers for f64 {
    fn finite_numbers(&self) -> bool {
        Float64Domain::Finite.contains(*self)
    }
}
impl<T: FiniteNumbers> FiniteNumbers for Option<T> {
    fn finite_numbers(&self) -> bool {
        self.as_ref().is_none_or(FiniteNumbers::finite_numbers)
    }
}
impl<T: FiniteNumbers> FiniteNumbers for Vec<T> {
    fn finite_numbers(&self) -> bool {
        self.iter().all(FiniteNumbers::finite_numbers)
    }
}
impl<K: Ord, T: FiniteNumbers> FiniteNumbers for BTreeMap<K, T> {
    fn finite_numbers(&self) -> bool {
        self.values().all(FiniteNumbers::finite_numbers)
    }
}
impl<T: FiniteNumbers, const N: usize> FiniteNumbers for [T; N] {
    fn finite_numbers(&self) -> bool {
        self.iter().all(FiniteNumbers::finite_numbers)
    }
}

/// Serialize original control data without allowing nonfinite numbers to become null.
/// # Errors
/// A value without a supported finite numeric document form refuses before output.
pub fn serialize<S: serde::Serializer, T: FiniteNumbers + serde::Serialize>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if !value.finite_numbers() {
        return Err(serde::ser::Error::custom(
            "nonfinite values have no finite numeric control document form",
        ));
    }
    value.serialize(serializer)
}

/// Decode the original typed field while retaining its finite-number representation.
/// # Errors
/// The underlying native decoder or the declared finite representation refuses.
pub fn deserialize<'de, D: serde::Deserializer<'de>, T: FiniteNumbers + serde::Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    let value = T::deserialize(deserializer)?;
    if !value.finite_numbers() {
        return Err(serde::de::Error::custom(
            "nonfinite values have no finite numeric control document form",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Control {
        #[serde(with = "super")]
        values: Option<Vec<f64>>,
    }

    #[test]
    fn finite_control_documents_preserve_valid_bytes_and_refuse_nonfinite_null_fallback() {
        assert_eq!(
            serde_json::to_string(&Control {
                values: Some(vec![0.0, -0.0, 1.25])
            })
            .unwrap(),
            r#"{"values":[0.0,-0.0,1.25]}"#
        );
        assert_eq!(
            serde_json::to_string(&Control { values: None }).unwrap(),
            r#"{"values":null}"#
        );
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                serde_json::to_string(&Control {
                    values: Some(vec![value])
                })
                .is_err()
            );
        }
        let decoded: Control = serde_json::from_str(r#"{"values":[-0.0,1.25]}"#).unwrap();
        assert_eq!(decoded.values.unwrap()[0].to_bits(), (-0.0_f64).to_bits());
    }
}
