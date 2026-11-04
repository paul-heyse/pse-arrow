// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The KINSOL settings document: the caller-selected method controls. Characteristic scales
//! and the scaled-step tolerance are never caller inputs; the adapter derives them from the
//! resolved numerical policy (`crate::kinsol::Settings::from_policy`).
use crate::solve::Preconditioner;
/// KINSOL nonlinear strategy, without a project-owned Newton method, and the Anderson
/// acceleration QR orthogonalization (`KINSetOrthAA`, fixed at allocation): registry
/// vocabularies (ADR-0115 Outcome 3).
pub use pse_model::generated::enums::{
    KinsolOrthogonalization as Orthogonalization, KinsolStrategy as Strategy,
};
use pse_model::scalar;
use pse_model::scalars::{Fraction, PositiveCount};

include!("kinsol/declarations.rs");
impl Linear {
    /// Krylov subspace dimension of a matrix-free route.
    pub fn krylov(self) -> Option<usize> {
        match self {
            Self::Spgmr { dimension }
            | Self::Spfgmr { dimension }
            | Self::Spbcgs { dimension }
            | Self::Sptfqmr { dimension } => Some(dimension.into_inner()),
            Self::Klu | Self::Dense { .. } => None,
        }
    }
}

impl Default for Method {
    fn default() -> Self {
        Self {
            strategy: Strategy::LineSearch,
            linear: Linear::Klu,
            anderson: 0,
            damping: scalar!(Fraction(1.0)),
            setup_interval: 10,
            max_newton_step: None,
            eta: Eta::default(),
            preconditioner: Preconditioner::None,
            orthogonalization: Orthogonalization::ModifiedGramSchmidt,
            anderson_delay: 0,
        }
    }
}

impl Method {
    /// Validate method-only controls before publishing a selected capability row.
    /// Dimension, scales and declared function representation are admitted separately.
    /// # Errors
    /// Controls are invalid or the selected native method does not consume them.
    pub fn validate(&self) -> Result<(), crate::ProblemError> {
        use crate::ProblemError;
        let m = self;
        if m.setup_interval == 0 || m.damping.into_inner() <= 0. {
            return Err(ProblemError::Contract(
                "KINSOL setup interval or damping is not positive".into(),
            ));
        }
        match m.linear {
            linear if linear.krylov().is_some_and(|d| i32::try_from(d).is_err()) => {
                return Err(ProblemError::Contract("KINSOL Krylov dimension".into()));
            }
            _ => {}
        }
        let krylov = m.linear.krylov().is_some();
        let fixed = m.strategy == Strategy::FixedPoint;
        let map = matches!(m.strategy, Strategy::FixedPoint | Strategy::Picard);
        let defaults = Method::default();
        if m.preconditioner == Preconditioner::BlockFactor && map {
            return Err(ProblemError::Unsupported(
                "block factor preconditioning requires original analytic equation Jacobians".into(),
            ));
        }
        if (!map && (m.anderson != 0 || m.damping != defaults.damping))
            || (map && m.max_newton_step.is_some())
            || (fixed
                && (m.linear != defaults.linear
                    || m.setup_interval != defaults.setup_interval
                    || m.eta != defaults.eta
                    || m.preconditioner != defaults.preconditioner))
        {
            return Err(ProblemError::Unsupported(
                "KINSOL selected strategy does not consume the requested method controls".into(),
            ));
        }
        if m.max_newton_step
            .is_some_and(|v| !NEWTON_STEP_DOMAIN.contains(v))
            || match m.eta {
                Eta::Choice1 => false,
                Eta::Choice2 { alpha, .. } => !ETA_POWER_DOMAIN.contains(alpha),
                Eta::Constant { .. } => false,
            }
            || (m.eta != Eta::default() && !krylov)
            || (m.preconditioner != Preconditioner::None && (!krylov || fixed))
            || (m.anderson == 0
                && (m.anderson_delay != 0
                    || m.orthogonalization != Orthogonalization::ModifiedGramSchmidt))
            || std::ffi::c_long::try_from(m.anderson_delay).is_err()
        {
            return Err(ProblemError::Contract(
                "KINSOL Newton-step, forcing-term, preconditioner or Anderson control".into(),
            ));
        }
        Ok(())
    }
    /// Every equation Krylov route consumes the original residual action independently
    /// of any assembled analytic Jacobian needed by its preconditioner.
    pub fn consumes_jvp(&self) -> bool {
        self.linear.krylov().is_some()
            && matches!(self.strategy, Strategy::Newton | Strategy::LineSearch)
    }
    /// Matrix-free equation execution without an assembled-Jacobian preconditioner.
    pub fn consumes_directional_only(&self) -> bool {
        self.consumes_jvp() && self.preconditioner == Preconditioner::None
    }
}
