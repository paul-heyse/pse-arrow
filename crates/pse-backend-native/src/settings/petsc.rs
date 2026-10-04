// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed serial PETSc profiles. Each optional block belongs to exactly one method.
use crate::ProblemError;
pub use pse_model::generated::enums::{
    NativePetscLinear as Linear, NativePetscMethod as Method,
    NativePetscPreconditioner as Preconditioner,
};
/// Normalized radius bounds consumed by the native trust-region method.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct TrustRegion {
    /// Positive lower radius; reaching it stops the native trajectory.
    pub minimum: f64,
    /// Positive finite maximum radius.
    pub maximum: f64,
    /// Initial radius in normalized coordinates.
    pub initial: f64,
}
impl Default for TrustRegion {
    fn default() -> Self {
        Self {
            minimum: 1e-12,
            maximum: 1e8,
            initial: 1.0,
        }
    }
}
/// Finite step growth and rejection controls for artificial pseudo-time.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct PseudoTime {
    /// Positive artificial initial time step.
    pub initial: f64,
    /// Explicit rejection floor; a step below this ends the attempt.
    pub minimum: f64,
    /// Finite maximum time step.
    pub maximum: f64,
    /// Successful-step growth greater than one.
    pub growth: f64,
    /// Failed-stage shrink strictly between zero and one.
    pub failed_scale: f64,
    /// Finite allowed rejected steps.
    pub rejections: u32,
    /// Finite allowed nonlinear failures.
    pub nonlinear_failures: u32,
}
impl Default for PseudoTime {
    fn default() -> Self {
        Self {
            initial: 1e-2,
            minimum: 1e-12,
            maximum: 1e6,
            growth: 1.1,
            failed_scale: 0.5,
            rejections: 20,
            nonlinear_failures: 20,
        }
    }
}
/// Explicit correction damping and overlap extension for nonlinear Schwarz.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
pub struct Schwarz {
    /// Native nonlinear correction damping in (0,1].
    pub damping: f64,
    /// Restrict the extension to each block's declared interior.
    pub restricted: bool,
}
impl Default for Schwarz {
    fn default() -> Self {
        Self {
            damping: 1.0,
            restricted: true,
        }
    }
}
/// Methods consume analytic original equation/Jacobian actions and no arbitrary bounds.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "PetscSettings")]
pub struct Settings {
    /// Selected native nonlinear method.
    pub method: Method,
    /// Selected native KSP method.
    pub linear: Linear,
    /// Selected native PC method.
    pub preconditioner: Preconditioner,
    /// Inner trust-region profile used by SNES and TSPSEUDO correction.
    pub trust: TrustRegion,
    /// Required precisely for the pseudo-transient profile.
    pub pseudo: Option<PseudoTime>,
    /// Required precisely for nonlinear additive Schwarz.
    pub schwarz: Option<Schwarz>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::NewtonTrustRegion,
            linear: Linear::Gmres,
            preconditioner: Preconditioner::Ilu,
            trust: TrustRegion::default(),
            pseudo: None,
            schwarz: None,
        }
    }
}
impl Settings {
    /// Refuse inconsistent, nonfinite or unused method controls.
    ///
    /// # Errors
    /// Method configuration does not describe an executable finite profile.
    pub fn validate(&self) -> Result<(), ProblemError> {
        let positive = |v: f64| v.is_finite() && v > 0.0;
        if ![self.trust.minimum, self.trust.maximum, self.trust.initial]
            .into_iter()
            .all(positive)
            || self.trust.minimum > self.trust.initial
            || self.trust.initial > self.trust.maximum
            || (self.method == Method::PseudoTransient) != self.pseudo.is_some()
            || (self.method == Method::NonlinearAdditiveSchwarz) != self.schwarz.is_some()
            || self.linear == Linear::Preonly
                && !matches!(
                    self.preconditioner,
                    Preconditioner::Lu | Preconditioner::Ilu
                )
        {
            return Err(ProblemError::Contract("PETSc method must consume its finite trust, pseudo-time, Schwarz and linear controls".into()));
        }
        if let Some(p) = self.pseudo
            && (![p.initial, p.minimum, p.maximum, p.growth, p.failed_scale]
                .into_iter()
                .all(positive)
                || p.minimum > p.initial
                || p.initial > p.maximum
                || p.growth <= 1.0
                || p.failed_scale >= 1.0
                || p.rejections > i32::MAX as u32
                || p.nonlinear_failures > i32::MAX as u32)
        {
            return Err(ProblemError::Contract("PETSc pseudo-time controls require a finite step range, growth, shrink and failure caps".into()));
        }
        if self
            .schwarz
            .is_some_and(|s| !positive(s.damping) || s.damping > 1.0)
        {
            return Err(ProblemError::Contract(
                "PETSc Schwarz damping must be in (0,1]".into(),
            ));
        }
        Ok(())
    }
}
