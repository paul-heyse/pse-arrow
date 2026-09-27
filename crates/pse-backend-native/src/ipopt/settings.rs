// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed Ipopt settings (ADR-0108 item 10). The linear solver carries its own parameters,
//! the MUMPS ordering is stated rather than inherited from `mumps_pivot_order = 7`, and every
//! option these settings own is reserved from raw native options. Admission refuses a solver
//! the linked library or the process cannot run as selected; nothing is ever substituted.
use super::runtime::{self, Runtime};
use crate::{
    ProblemError,
    solve::{OptionValue, Options},
};

/// The Ipopt adapter's settings type. Its identity derives from serde.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Settings {
    /// Symmetric-indefinite factorization of the KKT systems.
    pub linear: Linear,
    /// Barrier-parameter update strategy (`mu_strategy`).
    pub mu_strategy: MuStrategy,
    /// Minimum absolute distance of a cold initial point from its bounds (`bound_push`).
    pub bound_push: f64,
    /// Minimum relative distance of a cold initial point from its bounds (`bound_frac`).
    pub bound_frac: f64,
}
impl Default for Settings {
    /// MUMPS with METIS ordering, the monotone barrier and Ipopt's own push values, all
    /// stated so that no library default decides them.
    fn default() -> Self {
        Self {
            linear: Linear::default(),
            mu_strategy: MuStrategy::Monotone,
            bound_push: 0.01,
            bound_frac: 0.01,
        }
    }
}

/// One linked symmetric-indefinite factorization with its typed parameters. HSL solvers and
/// the runtime-loaded Pardiso are not representable (ADR-0108 item 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
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
/// The linked linear solver a [`Linear`] selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum LinearSolver {
    /// MUMPS.
    Mumps,
    /// SPRAL SSIDS.
    Spral,
    /// oneMKL Pardiso.
    PardisoMkl,
}
impl LinearSolver {
    /// Every typed solver.
    pub const ALL: [Self; 3] = [Self::Mumps, Self::Spral, Self::PardisoMkl];
    /// Ipopt's `linear_solver` spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mumps => "mumps",
            Self::Spral => "spral",
            Self::PardisoMkl => "pardisomkl",
        }
    }
    /// Bit of this solver in `IpoptGetAvailableLinearSolvers`.
    pub const fn mask(self) -> u32 {
        match self {
            Self::Mumps => pse_ipopt_sys::IPOPTLINEARSOLVER_MUMPS,
            Self::Spral => pse_ipopt_sys::IPOPTLINEARSOLVER_SPRAL,
            Self::PardisoMkl => pse_ipopt_sys::IPOPTLINEARSOLVER_PARDISOMKL,
        }
    }
    /// Whether the solver consumes more than one admitted thread; MUMPS is sequential.
    pub const fn parallel(self) -> bool {
        !matches!(self, Self::Mumps)
    }
}
impl Linear {
    /// The linked solver this selects.
    pub const fn solver(&self) -> LinearSolver {
        match self {
            Self::Mumps { .. } => LinearSolver::Mumps,
            Self::Spral { .. } => LinearSolver::Spral,
            Self::PardisoMkl { .. } => LinearSolver::PardisoMkl,
        }
    }
    fn options(&self) -> Vec<(&'static str, OptionValue)> {
        let text = |v: &str| OptionValue::Text(v.into());
        let mut out = vec![("linear_solver", text(self.solver().as_str()))];
        match *self {
            Self::Mumps { ordering } => {
                out.push(("mumps_pivot_order", OptionValue::Integer(ordering as i32)));
            }
            Self::Spral {
                ordering,
                scaling,
                pivot,
            } => out.extend([
                ("spral_order", text(ordering.as_str())),
                ("spral_scaling", text(scaling.as_str())),
                ("spral_pivot_method", text(pivot.as_str())),
            ]),
            Self::PardisoMkl { ordering, matching } => out.extend([
                ("pardisomkl_order", text(ordering.as_str())),
                ("pardisomkl_matching_strategy", text(matching.as_str())),
            ]),
        }
        out
    }
}
/// MUMPS fill-reducing orderings built into, or linked with, the image's MUMPS. Automatic
/// choice (7) is not representable: the ordering is always stated (T06). SCOTCH is not
/// linked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum MumpsOrdering {
    /// Approximate minimum degree.
    Amd = 0,
    /// Approximate minimum fill.
    Amf = 2,
    /// PORD.
    Pord = 4,
    /// METIS nested dissection (the image's shared METIS).
    Metis = 5,
    /// Approximate minimum degree with quasi-dense row detection.
    Qamd = 6,
}
/// SPRAL elimination orderings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SpralOrdering {
    /// METIS with default settings.
    Metis,
    /// Matching-based elimination ordering.
    Matching,
}
impl SpralOrdering {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Metis => "metis",
            Self::Matching => "matching",
        }
    }
}
/// SPRAL scalings (the dynamic switch family is not representable).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SpralScaling {
    /// No scaling.
    None,
    /// Weighted bipartite matching (MC64).
    Mc64,
    /// Auction algorithm.
    Auction,
    /// Matching-based ordering's scaling.
    Matching,
    /// Ruiz norm equilibration.
    Ruiz,
}
impl SpralScaling {
    const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Mc64 => "mc64",
            Self::Auction => "auction",
            Self::Matching => "matching",
            Self::Ruiz => "ruiz",
        }
    }
}
/// SPRAL pivoting strategies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SpralPivot {
    /// Aggressive a posteriori pivoting.
    Aggressive,
    /// Block a posteriori pivoting.
    Block,
    /// Threshold partial pivoting; SPRAL runs it serially.
    Threshold,
}
impl SpralPivot {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Aggressive => "aggressive",
            Self::Block => "block",
            Self::Threshold => "threshold",
        }
    }
}
/// oneMKL Pardiso orderings (Ipopt's undocumented `one` is not representable).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum PardisoOrdering {
    /// Minimum degree.
    Amd,
    /// METIS nested dissection.
    Metis,
    /// OpenMP-parallel METIS nested dissection.
    ParallelMetis,
}
impl PardisoOrdering {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Amd => "amd",
            Self::Metis => "metis",
            Self::ParallelMetis => "pmetis",
        }
    }
}
/// oneMKL Pardiso symmetric weighted matchings (IPARM(13)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum PardisoMatching {
    /// Complete matching.
    Complete,
    /// Complete matching with 2x2 pivots.
    CompletePlus2x2,
    /// Matching of the constraint block.
    Constraints,
}
impl PardisoMatching {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::CompletePlus2x2 => "complete+2x2",
            Self::Constraints => "constraints",
        }
    }
}
/// Barrier-parameter update strategies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum MuStrategy {
    /// Monotone Fiacco–McCormick decrease.
    Monotone,
    /// Adaptive (Nocedal–Wächter–Waltz) update.
    Adaptive,
}

/// Native options the typed settings own; raw options may not set them. `hsllib` and
/// `pardisolib` load excluded libraries (HSL, runtime Pardiso) and stay reserved.
pub(crate) const RESERVED: [&str; 12] = [
    "linear_solver",
    "hsllib",
    "pardisolib",
    "mumps_pivot_order",
    "spral_order",
    "spral_scaling",
    "spral_pivot_method",
    "pardisomkl_order",
    "pardisomkl_matching_strategy",
    "mu_strategy",
    "bound_push",
    "bound_frac",
];

impl Settings {
    /// Finite push values inside Ipopt's registered ranges.
    ///
    /// # Errors
    /// A push value outside its native range.
    pub fn validate(&self) -> Result<(), ProblemError> {
        if !(self.bound_push.is_finite() && self.bound_push > 0.0)
            || !(self.bound_frac > 0.0 && self.bound_frac <= 0.5)
        {
            return Err(ProblemError::Contract(
                "Ipopt bound_push must be positive and bound_frac in (0, 0.5]".into(),
            ));
        }
        Ok(())
    }
    /// The native option table these settings set on every solve.
    pub(crate) fn options(&self) -> Options {
        let mut out: Options = self
            .linear
            .options()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
        out.extend([
            (
                "mu_strategy".into(),
                OptionValue::Text(
                    match self.mu_strategy {
                        MuStrategy::Monotone => "monotone",
                        MuStrategy::Adaptive => "adaptive",
                    }
                    .into(),
                ),
            ),
            ("bound_push".into(), OptionValue::Real(self.bound_push)),
            ("bound_frac".into(), OptionValue::Real(self.bound_frac)),
        ]);
        out
    }
}

/// Admit `settings` at `threads` against the observed process `runtime`. An unlinked solver,
/// a sequential solver asked for threads, SPRAL without its OpenMP environment and Pardiso
/// outside the pinned oneMKL CBWR branch are refused before any native construction; there
/// is no fallback to another solver (ADR-0108 items 11–14).
///
/// # Errors
/// A typed `Unsupported` refusal naming the missing library or precondition, or invalid
/// settings.
pub fn admit(settings: &Settings, threads: usize, runtime: &Runtime) -> Result<(), ProblemError> {
    settings.validate()?;
    let solver = settings.linear.solver();
    let name = solver.as_str();
    if runtime.linked & solver.mask() == 0 {
        return Err(ProblemError::Unsupported(format!(
            "Ipopt linear solver {name} is not linked in this build; no other solver is substituted"
        )));
    }
    if threads > 1 && !solver.parallel() {
        return Err(ProblemError::Unsupported(format!(
            "Ipopt linear solver {name} is sequential; {threads} threads need spral or pardisomkl"
        )));
    }
    match solver {
        LinearSolver::Mumps => {}
        LinearSolver::Spral => {
            if !runtime.cancellation {
                return Err(ProblemError::Unsupported(
                    "SPRAL requires OMP_CANCELLATION=TRUE in the process environment".into(),
                ));
            }
            if runtime.proc_bind == runtime::OMP_PROC_BIND_FALSE {
                return Err(ProblemError::Unsupported(
                    "SPRAL requires OMP_PROC_BIND=TRUE in the process environment".into(),
                ));
            }
        }
        LinearSolver::PardisoMkl => {
            let pinned = runtime::pinned_cbwr();
            if runtime.cbwr != pinned {
                return Err(ProblemError::Unsupported(format!(
                    "oneMKL Pardiso requires the pinned MKL_CBWR branch {}; the process runs {}",
                    runtime::cbwr_name(pinned),
                    runtime::cbwr_name(runtime.cbwr)
                )));
            }
            if runtime.mkl_dynamic {
                return Err(ProblemError::Unsupported(
                    "oneMKL Pardiso requires MKL_DYNAMIC=FALSE so admitted threads are used".into(),
                ));
            }
        }
    }
    Ok(())
}
