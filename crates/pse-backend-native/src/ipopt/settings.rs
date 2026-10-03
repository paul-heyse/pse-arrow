// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The Ipopt adapter's reading of its settings document (`crate::settings::ipopt`): native
//! option values and codes, every option the settings own reserved from raw native options,
//! and admission. Admission refuses a solver the linked library or the process cannot run as
//! selected; nothing is ever substituted.
use super::runtime::{self, Runtime};
use crate::{
    ProblemError,
    settings::ipopt::{Linear, MumpsOrdering, PardisoMatching, PardisoOrdering, Settings},
    solve::{IpoptLinearSolver, OptionValue, Options},
};

/// Bit of `solver` in `IpoptGetAvailableLinearSolvers`.
pub(crate) const fn mask(solver: IpoptLinearSolver) -> u32 {
    match solver {
        IpoptLinearSolver::Mumps => pse_ipopt_sys::IPOPTLINEARSOLVER_MUMPS,
        IpoptLinearSolver::Spral => pse_ipopt_sys::IPOPTLINEARSOLVER_SPRAL,
        IpoptLinearSolver::Pardisomkl => pse_ipopt_sys::IPOPTLINEARSOLVER_PARDISOMKL,
    }
}
/// Equality of the exact runtime observations consumed by this selected linear solver.
pub(crate) fn same_admission(settings: &Settings, before: &Runtime, after: &Runtime) -> bool {
    let solver = settings.linear.solver();
    before.linked & mask(solver) == after.linked & mask(solver)
        && match solver {
            IpoptLinearSolver::Mumps => true,
            IpoptLinearSolver::Spral => {
                before.cancellation == after.cancellation && before.proc_bind == after.proc_bind
            }
            IpoptLinearSolver::Pardisomkl => {
                before.cbwr == after.cbwr && before.mkl_dynamic == after.mkl_dynamic
            }
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
/// The native options of one linear solver selection.
fn linear_options(linear: &Linear) -> Vec<(&'static str, OptionValue)> {
    let text = |v: &str| OptionValue::Text(v.into());
    let mut out = vec![("linear_solver", text(linear.solver().as_str()))];
    match *linear {
        Linear::Mumps { ordering } => {
            out.push((
                "mumps_pivot_order",
                OptionValue::Integer(mumps_pivot_order(ordering)),
            ));
        }
        Linear::Spral {
            ordering,
            scaling,
            pivot,
        } => out.extend([
            // SPRAL's option values are the registry spellings.
            ("spral_order", text(ordering.as_str())),
            ("spral_scaling", text(scaling.as_str())),
            ("spral_pivot_method", text(pivot.as_str())),
        ]),
        Linear::PardisoMkl { ordering, matching } => out.extend([
            ("pardisomkl_order", text(pardiso_order(ordering))),
            (
                "pardisomkl_matching_strategy",
                text(pardiso_matching(matching)),
            ),
        ]),
    }
    out
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

/// The native option table `settings` set on every solve.
pub(crate) fn options(settings: &Settings) -> Options {
    let mut out: Options = linear_options(&settings.linear)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    out.extend([
        // Ipopt's `mu_strategy` values are the registry spellings.
        (
            "mu_strategy".into(),
            OptionValue::Text(settings.mu_strategy.as_str().into()),
        ),
        (
            "bound_push".into(),
            OptionValue::Real(settings.bound_push.into_inner()),
        ),
        (
            "bound_frac".into(),
            OptionValue::Real(settings.bound_frac.into_inner()),
        ),
    ]);
    out
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

#[cfg(test)]
mod contextual_tests {
    use super::*;
    use crate::settings::ipopt::{
        Linear, PardisoMatching, PardisoOrdering, SpralOrdering, SpralPivot, SpralScaling,
    };
    #[test]
    fn contextual_unit_selected_snapshot_consumes_only_its_linear_solver_prerequisites() {
        let before = Runtime {
            linked: mask(IpoptLinearSolver::Mumps)
                | mask(IpoptLinearSolver::Spral)
                | mask(IpoptLinearSolver::Pardisomkl),
            cancellation: true,
            proc_bind: 1,
            cbwr: 7,
            mkl_dynamic: false,
        };
        let mumps = Settings::default();
        let mut after = before;
        after.cancellation = false;
        after.proc_bind = 0;
        after.cbwr = 99;
        after.mkl_dynamic = true;
        after.linked &= !mask(IpoptLinearSolver::Spral);
        assert!(same_admission(&mumps, &before, &after));
        after.linked &= !mask(IpoptLinearSolver::Mumps);
        assert!(!same_admission(&mumps, &before, &after));
        let spral = Settings {
            linear: Linear::Spral {
                ordering: SpralOrdering::Metis,
                scaling: SpralScaling::Matching,
                pivot: SpralPivot::Block,
            },
            ..Settings::default()
        };
        after = before;
        after.cbwr = 99;
        after.mkl_dynamic = true;
        assert!(same_admission(&spral, &before, &after));
        after.cancellation = false;
        assert!(!same_admission(&spral, &before, &after));
        after = before;
        after.proc_bind = 0;
        assert!(!same_admission(&spral, &before, &after));
        let pardiso = Settings {
            linear: Linear::PardisoMkl {
                ordering: PardisoOrdering::Metis,
                matching: PardisoMatching::CompletePlus2x2,
            },
            ..Settings::default()
        };
        after = before;
        after.cancellation = false;
        after.proc_bind = 0;
        assert!(same_admission(&pardiso, &before, &after));
        after.cbwr = 99;
        assert!(!same_admission(&pardiso, &before, &after));
        after = before;
        after.mkl_dynamic = true;
        assert!(!same_admission(&pardiso, &before, &after));
    }
}
