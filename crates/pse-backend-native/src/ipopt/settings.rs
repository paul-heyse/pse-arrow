// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed Ipopt settings (ADR-0108 item 10). The linear solver carries its own parameters,
//! the MUMPS ordering is stated rather than inherited from `mumps_pivot_order = 7`, and every
//! option these settings own is reserved from raw native options. Admission refuses a solver
//! the linked library or the process cannot run as selected; nothing is ever substituted.
use super::runtime::{self, Runtime};
use crate::{
    ProblemError,
    solve::{IpoptLinearSolver, OptionValue, Options, WarmRestart},
};
/// The registry vocabularies these settings are written in (ADR-0115 Outcome 3); their
/// native option values and codes are the adapter functions below.
pub use pse_model::generated::enums::{
    MuStrategy, MumpsOrdering, PardisoMatching, PardisoOrdering, SpralOrdering, SpralPivot,
    SpralScaling,
};

/// The Ipopt adapter's settings type. Its identity derives from serde, and absent fields
/// take these defaults across the Python boundary (ADR-0113).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Symmetric-indefinite factorization of the KKT systems.
    pub linear: Linear,
    /// Barrier-parameter update strategy (`mu_strategy`).
    pub mu_strategy: MuStrategy,
    /// Minimum absolute distance of a cold initial point from its bounds (`bound_push`).
    pub bound_push: f64,
    /// Minimum relative distance of a cold initial point from its bounds (`bound_frac`).
    pub bound_frac: f64,
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
            bound_push: 0.01,
            bound_frac: 0.01,
            restart: WarmRestart::default(),
        }
    }
}

/// One linked symmetric-indefinite factorization with its typed parameters. HSL solvers and
/// the runtime-loaded Pardiso are not representable (ADR-0108 item 9). Each variant is
/// spelled as its [`IpoptLinearSolver`], Ipopt's native `linear_solver` name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
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
/// Bit of `solver` in `IpoptGetAvailableLinearSolvers`.
pub(crate) const fn mask(solver: IpoptLinearSolver) -> u32 {
    match solver {
        IpoptLinearSolver::Mumps => pse_ipopt_sys::IPOPTLINEARSOLVER_MUMPS,
        IpoptLinearSolver::Spral => pse_ipopt_sys::IPOPTLINEARSOLVER_SPRAL,
        IpoptLinearSolver::Pardisomkl => pse_ipopt_sys::IPOPTLINEARSOLVER_PARDISOMKL,
    }
}
/// Whether `solver` consumes more than one admitted thread; MUMPS is sequential.
const fn parallel(solver: IpoptLinearSolver) -> bool {
    !matches!(solver, IpoptLinearSolver::Mumps)
}
/// MUMPS ICNTL(7) code of an ordering (`mumps_pivot_order`). Automatic choice (7) is not
/// representable: the ordering is always stated (T06). SCOTCH is not linked.
pub(crate) const fn mumps_pivot_order(ordering: MumpsOrdering) -> i32 {
    match ordering {
        MumpsOrdering::Amd => 0,
        MumpsOrdering::Amf => 2,
        MumpsOrdering::Pord => 4,
        MumpsOrdering::Metis => 5,
        MumpsOrdering::Qamd => 6,
    }
}
/// Ipopt's `pardisomkl_order` value (its undocumented `one` is not representable).
const fn pardiso_order(ordering: PardisoOrdering) -> &'static str {
    match ordering {
        PardisoOrdering::Amd => "amd",
        PardisoOrdering::Metis => "metis",
        PardisoOrdering::ParallelMetis => "pmetis",
    }
}
/// Ipopt's `pardisomkl_matching_strategy` value (oneMKL IPARM(13)).
const fn pardiso_matching(matching: PardisoMatching) -> &'static str {
    match matching {
        PardisoMatching::Complete => "complete",
        PardisoMatching::CompletePlus2x2 => "complete+2x2",
        PardisoMatching::Constraints => "constraints",
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
    fn options(&self) -> Vec<(&'static str, OptionValue)> {
        let text = |v: &str| OptionValue::Text(v.into());
        let mut out = vec![("linear_solver", text(self.solver().as_str()))];
        match *self {
            Self::Mumps { ordering } => {
                out.push((
                    "mumps_pivot_order",
                    OptionValue::Integer(mumps_pivot_order(ordering)),
                ));
            }
            Self::Spral {
                ordering,
                scaling,
                pivot,
            } => out.extend([
                // SPRAL's option values are the registry spellings.
                ("spral_order", text(ordering.as_str())),
                ("spral_scaling", text(scaling.as_str())),
                ("spral_pivot_method", text(pivot.as_str())),
            ]),
            Self::PardisoMkl { ordering, matching } => out.extend([
                ("pardisomkl_order", text(pardiso_order(ordering))),
                (
                    "pardisomkl_matching_strategy",
                    text(pardiso_matching(matching)),
                ),
            ]),
        }
        out
    }
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
        self.restart.validate()
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
            // Ipopt's `mu_strategy` values are the registry spellings.
            (
                "mu_strategy".into(),
                OptionValue::Text(self.mu_strategy.as_str().into()),
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
    if runtime.linked & mask(solver) == 0 {
        return Err(ProblemError::Unsupported(format!(
            "Ipopt linear solver {name} is not linked in this build; no other solver is substituted"
        )));
    }
    if threads > 1 && !parallel(solver) {
        return Err(ProblemError::Unsupported(format!(
            "Ipopt linear solver {name} is sequential; {threads} threads need spral or pardisomkl"
        )));
    }
    match solver {
        IpoptLinearSolver::Mumps => {}
        IpoptLinearSolver::Spral => {
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
        IpoptLinearSolver::Pardisomkl => {
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
