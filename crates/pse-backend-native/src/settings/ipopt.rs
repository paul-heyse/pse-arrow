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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[schemars(rename = "IpoptLinear")]
pub enum Linear {
    /// Sequential MUMPS 5.9 (`linear_solver = mumps`).
    Mumps {
        /// Fill-reducing ordering (`mumps_pivot_order`, MUMPS ICNTL(7)).
        ordering: MumpsOrdering,
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
        }
    }
}
impl Linear {
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
        self.restart.validate()
    }
}
