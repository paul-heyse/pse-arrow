// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The Ipopt settings document (ADR-0108 item 10). The linear solver carries its own
//! parameters, and the MUMPS ordering is stated rather than inherited from
//! `mumps_pivot_order = 7`. Native option values, codes and admission against the linked
//! library are the adapter's (`crate::ipopt`).
use crate::{
    ProblemError,
    solve::{IpoptLinearSolver, WarmRestart},
};
/// The registry vocabularies these settings are written in (ADR-0115 Outcome 3).
pub use pse_model::generated::enums::{
    MuStrategy, MumpsOrdering, PardisoMatching, PardisoOrdering, SpralOrdering, SpralPivot,
    SpralScaling,
};
use pse_model::scalar;
use pse_model::scalars::{Fraction, Tolerance};

/// Process state that linear-solver admission depends on, observed on the calling thread.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    /// Linear solvers the linked Ipopt was built with (`IpoptGetAvailableLinearSolvers`);
    /// the runtime library loader is excluded.
    pub linked: u32,
    /// OpenMP cancellation (`OMP_CANCELLATION`), fixed when the OpenMP runtime started.
    pub cancellation: bool,
    /// OpenMP thread-binding policy (`OMP_PROC_BIND`); `0` is unbound.
    pub proc_bind: i32,
    /// oneMKL conditional-numerical-reproducibility branch in force.
    pub cbwr: i32,
    /// oneMKL dynamic thread adjustment (`MKL_DYNAMIC`).
    pub mkl_dynamic: bool,
}

/// The Ipopt adapter's settings type. Its identity derives from serde, and absent fields
/// take these defaults across the Python boundary (ADR-0113).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "IpoptSettings")]
pub struct Settings {
    /// Symmetric-indefinite factorization of the KKT systems.
    pub linear: Linear,
    /// Barrier-parameter update strategy (`mu_strategy`).
    pub mu_strategy: MuStrategy,
    /// Minimum absolute distance of a cold initial point from its bounds (`bound_push`).
    pub bound_push: Tolerance,
    /// Minimum relative distance of a cold initial point from its bounds (`bound_frac`);
    /// admission refuses more than one half, Ipopt's registered range.
    pub bound_frac: Fraction,
    /// Restart of a submitted primal-dual seed: barrier and pushes (L-N3).
    pub restart: WarmRestart,
}
impl Default for Settings {
    /// MUMPS with METIS ordering, the monotone barrier and Ipopt's own push values, all
    /// stated so that no library default decides them.
    fn default() -> Self {
        Self {
            linear: Linear::default(),
            mu_strategy: MuStrategy::Monotone,
            bound_push: scalar!(Tolerance(0.01)),
            bound_frac: scalar!(Fraction(0.01)),
            restart: WarmRestart::default(),
        }
    }
}

/// One linked symmetric-indefinite factorization with its typed parameters. HSL solvers and
/// the runtime-loaded Pardiso are not representable (ADR-0108 item 9). Each kind is spelled
/// as its [`IpoptLinearSolver`], Ipopt's native `linear_solver` name.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[schemars(rename = "IpoptLinear")]
pub enum Linear {
    /// Sequential MUMPS 5.9 (`linear_solver = mumps`).
    Mumps {
        /// Fill-reducing ordering (`mumps_pivot_order`, MUMPS ICNTL(7)).
        ordering: MumpsOrdering,
        /// Sparsity/stability pivot tolerance (`mumps_pivtol`), inclusive `[0,1]`.
        #[schemars(extend("minimum" = 0.0, "maximum" = 1.0))]
        pivot_tolerance: f64,
        /// Maximum refinement-requested pivot tolerance (`mumps_pivtolmax`).
        #[schemars(extend("minimum" = 0.0, "maximum" = 1.0))]
        maximum_pivot_tolerance: f64,
        /// Extra estimated workspace percentage (`mumps_mem_percent`), zero allowed.
        #[schemars(extend("minimum" = 0))]
        memory_percent: i32,
        /// Native ICNTL(6) permuting/scaling strategy, inclusive `[0,7]`.
        #[schemars(extend("maximum" = 7))]
        permuting_scaling: u8,
    },
    /// SPRAL SSIDS on OpenMP threads (`linear_solver = spral`).
    Spral {
        /// Elimination ordering (`spral_order`).
        ordering: SpralOrdering,
        /// Matrix scaling (`spral_scaling`).
        scaling: SpralScaling,
        /// A posteriori pivoting strategy (`spral_pivot_method`).
        pivot: SpralPivot,
    },
    /// oneMKL Pardiso on MKL threads (`linear_solver = pardisomkl`), under the pinned CBWR
    /// branch.
    PardisoMkl {
        /// Fill-reducing ordering (`pardisomkl_order`).
        ordering: PardisoOrdering,
        /// Symmetric weighted matching (`pardisomkl_matching_strategy`).
        matching: PardisoMatching,
    },
}
impl Default for Linear {
    fn default() -> Self {
        Self::Mumps {
            ordering: MumpsOrdering::Metis,
            pivot_tolerance: 1e-6,
            maximum_pivot_tolerance: 0.1,
            memory_percent: 1000,
            permuting_scaling: 7,
        }
    }
}
impl Linear {
    /// Whether this selected native factorization consumes an admitted thread team.
    pub const fn parallel(&self) -> bool {
        match self {
            Self::Mumps { .. } => false,
            Self::Spral { .. } | Self::PardisoMkl { .. } => true,
        }
    }
    /// The linked solver this selects.
    pub const fn solver(&self) -> IpoptLinearSolver {
        match self {
            Self::Mumps { .. } => IpoptLinearSolver::Mumps,
            Self::Spral { .. } => IpoptLinearSolver::Spral,
            Self::PardisoMkl { .. } => IpoptLinearSolver::Pardisomkl,
        }
    }
}
impl Settings {
    /// The cross-field rules the scalar types cannot state: a relative push inside Ipopt's
    /// registered range (0, 0.5], and the restart's.
    ///
    /// # Errors
    /// A relative push above one half.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if self.bound_frac.into_inner() > 0.5 {
            return Err(ProblemError::Contract(
                "Ipopt bound_frac must be at most 0.5".into(),
            ));
        }
        if let Linear::Mumps {
            pivot_tolerance,
            maximum_pivot_tolerance,
            memory_percent,
            permuting_scaling,
            ..
        } = self.linear
            && (!pivot_tolerance.is_finite()
                || !(0. ..=1.).contains(&pivot_tolerance)
                || !maximum_pivot_tolerance.is_finite()
                || !(pivot_tolerance..=1.).contains(&maximum_pivot_tolerance)
                || memory_percent < 0
                || permuting_scaling > 7)
        {
            return Err(ProblemError::Contract(
                "MUMPS controls outside registered native ranges".into(),
            ));
        }
        self.restart.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings(pivot: f64, maximum: f64, memory: i32, scaling: u8) -> Settings {
        Settings {
            linear: Linear::Mumps {
                ordering: MumpsOrdering::Metis,
                pivot_tolerance: pivot,
                maximum_pivot_tolerance: maximum,
                memory_percent: memory,
                permuting_scaling: scaling,
            },
            ..Default::default()
        }
    }
    #[test]
    fn mumps_controls_match_registered_ranges_including_zero() {
        assert!(Settings::default().validate().is_ok());
        for valid in [
            settings(0., 0., 0, 0),
            settings(1., 1., i32::MAX, 7),
            settings(1e-6, 0.1, 1000, 7),
        ] {
            assert!(valid.validate().is_ok());
            let encoded = serde_json::to_vec(&valid).unwrap();
            let decoded: Settings = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(decoded, valid);
        }
        for invalid in [
            settings(-1., 0.1, 1000, 7),
            settings(f64::NAN, 0.1, 1000, 7),
            settings(0.2, 0.1, 1000, 7),
            settings(0., f64::INFINITY, 1000, 7),
            settings(0., 1.1, 1000, 7),
            settings(0., 0.1, -1, 7),
            settings(0., 0.1, 1000, 8),
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
