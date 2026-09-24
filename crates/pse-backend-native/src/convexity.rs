// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{CoefficientProblem, ProblemError};
pub use pse_math::coefficients::GramCertificate;
impl CoefficientProblem {
    /// A convex native QP route requires exact, current, sense-aware PSD evidence.
    pub fn validate_convex(
        &self,
        certificate: Option<&GramCertificate>,
    ) -> Result<(), ProblemError> {
        self.validate()?;
        if let Some(q) = &self.hessian {
            if q.val().iter().any(|v| *v != 0.0) {
                certificate
                    .ok_or_else(|| {
                        ProblemError::Contract(
                            "quadratic degree does not establish convexity".into(),
                        )
                    })?
                    .validate(q, self.sense.sign())?;
            }
        }
        Ok(())
    }
}
