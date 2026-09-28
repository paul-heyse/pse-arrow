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

/// Selected native linear algebra. Dense allocation has an explicit dimension ceiling;
/// the matrix-free Krylov routes use the analytic Jacobian-vector product.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(rename = "KinsolLinear")]
pub enum Linear {
    /// Vendored SuiteSparse KLU with analytic CSC Jacobian.
    Klu,
    /// Serial dense factorization for explicitly small systems.
    Dense {
        /// Maximum admitted dimension.
        limit: PositiveCount,
    },
    /// Matrix-free native GMRES.
    Spgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
    },
    /// Matrix-free native flexible GMRES.
    Spfgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
    },
    /// Matrix-free native BiCGStab.
    Spbcgs {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
    },
    /// Matrix-free native transpose-free QMR.
    Sptfqmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
    },
}
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
/// Inexact-Newton forcing term of the Krylov routes (`KINSetEtaForm`).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(rename = "KinsolEta")]
pub enum Eta {
    /// Eisenstat-Walker choice 1, KINSOL's default.
    #[default]
    Choice1,
    /// Eisenstat-Walker choice 2 (`KINSetEtaParams`).
    Choice2 {
        /// Safeguard factor in (0, 1].
        gamma: Fraction,
        /// Power in (1, 2].
        alpha: f64,
    },
    /// A constant forcing term in (0, 1] (`KINSetEtaConstValue`).
    Constant {
        /// The forcing term.
        value: Fraction,
    },
}
/// Caller-selected KINSOL method controls: the adapter's pse-owned settings type.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "KinsolSettings")]
pub struct Method {
    /// Nonlinear strategy.
    pub strategy: Strategy,
    /// Native linear solver for equation profiles.
    pub linear: Linear,
    /// Native Anderson history; zero disables acceleration.
    pub anderson: usize,
    /// Damping in (0,1].
    pub damping: Fraction,
    /// Maximum nonlinear iterations between linear setups.
    pub setup_interval: u32,
    /// Maximum scaled Newton step (`KINSetMaxNewtonStep`), at least one scaled unit
    /// because KINSOL raises a smaller cap to one; `None` keeps KINSOL's
    /// `1000 * ||D_u u_0||`.
    pub max_newton_step: Option<f64>,
    /// Inexact-Newton forcing term; Krylov routes only.
    pub eta: Eta,
    /// Krylov preconditioner from the analytic Jacobian (`KINSetPreconditioner`);
    /// KINSOL preconditions on the right.
    pub preconditioner: Preconditioner,
    /// Anderson QR orthogonalization; Anderson acceleration only.
    pub orthogonalization: Orthogonalization,
    /// Iterations before Anderson acceleration starts (`KINSetDelayAA`); Anderson only.
    pub anderson_delay: usize,
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
