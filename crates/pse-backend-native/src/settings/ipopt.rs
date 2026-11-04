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

include!("ipopt/declarations.rs");
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
        if !crate::solve::RELATIVE_PUSH_DOMAIN.contains(self.bound_frac.into_inner()) {
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
            && (!MUMPS_PIVOT_DOMAIN.contains(pivot_tolerance)
                || !MUMPS_PIVOT_DOMAIN.contains(maximum_pivot_tolerance)
                || maximum_pivot_tolerance < pivot_tolerance
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
