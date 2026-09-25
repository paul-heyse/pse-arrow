// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One conversion into the canonical integration clock, before sample binding.
use super::{WorkflowError, contract};
use pse_relations::generated::enums::ObservationTimeBasis;

pub(super) fn observation(
    value: f64,
    unit_scale: f64,
    basis: ObservationTimeBasis,
    origin: f64,
    start: f64,
) -> Result<f64, WorkflowError> {
    let zero = match basis {
        ObservationTimeBasis::Elapsed => start,
        ObservationTimeBasis::ModelClock => origin,
    };
    let result = zero + value * unit_scale;
    if !value.is_finite()
        || !unit_scale.is_finite()
        || unit_scale <= 0.0
        || !zero.is_finite()
        || !result.is_finite()
    {
        return Err(contract("nonfinite observation time coordinate"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elapsed_and_model_clock_bind_the_same_nonzero_origin_sample() {
        assert_eq!(
            observation(2.0, 60.0, ObservationTimeBasis::Elapsed, 100.0, 160.0).unwrap(),
            280.0
        );
        assert_eq!(
            observation(3.0, 60.0, ObservationTimeBasis::ModelClock, 100.0, 160.0).unwrap(),
            280.0
        );
        assert!(observation(f64::MAX, 60.0, ObservationTimeBasis::Elapsed, 0.0, 0.0).is_err());
    }
}
