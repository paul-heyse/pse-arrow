// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit Uno profiles. Shared tolerances, limits and starts remain solve policy.
use crate::ProblemError;
pub use pse_model::generated::enums::NativeUnoMethod as Method;

/// Native SQP/SLP with the shared HiGHS provider and finite trust region.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "UnoSettings")]
pub struct Settings {
    /// SQP uses an L-BFGS Hessian; SLP consumes no Hessian history.
    pub method: Method,
    /// Initial native trust radius, in normalized coordinates.
    pub trust_radius: f64,
    /// Positive L-BFGS history for SQP; exactly zero for SLP.
    pub limited_memory: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Sqp,
            trust_radius: 10.0,
            limited_memory: 6,
        }
    }
}
impl Settings {
    /// Refuse controls that the selected native method would ignore.
    ///
    /// # Errors
    /// Invalid trust radius or unused/incomplete method history.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if !self.trust_radius.is_finite() || self.trust_radius <= 0.0 {
            return Err(ProblemError::Contract(
                "Uno trust radius must be finite and positive".into(),
            ));
        }
        if matches!(self.method, Method::Sqp) && self.limited_memory == 0
            || matches!(self.method, Method::Slp) && self.limited_memory != 0
        {
            return Err(ProblemError::Contract(
                "Uno SQP requires positive L-BFGS history; SLP requires zero history".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_refuses_inert_history_and_nonfinite_radius() {
        assert!(Settings::default().validate().is_ok());
        assert!(
            Settings {
                method: Method::Slp,
                limited_memory: 0,
                ..Settings::default()
            }
            .validate()
            .is_ok()
        );
        assert!(
            Settings {
                method: Method::Slp,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                limited_memory: 0,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Settings {
                trust_radius: f64::INFINITY,
                ..Settings::default()
            }
            .validate()
            .is_err()
        );
    }
}
