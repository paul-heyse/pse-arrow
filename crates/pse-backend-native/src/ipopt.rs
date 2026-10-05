// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Persistent shielded Ipopt TNLP/application execution on its owning worker.
mod runtime;
mod sequence;
mod settings;
pub use crate::settings::ipopt::{
    Linear, MuStrategy, MumpsOrdering, PardisoMatching, PardisoOrdering, Settings, SpralOrdering,
    SpralPivot, SpralScaling,
};
use crate::{
    ProblemError,
    solve::{Assurance, NativeTermination, Termination},
};
use pse_ipopt_sys as ffi;
pub use runtime::{Build, Runtime, build};
pub use sequence::Sequence as Session;
pub use settings::admit;
pub(crate) use settings::same_admission;
use std::ffi::CString;

const INFINITY: f64 = 1e19;
fn index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Unsupported("Ipopt index overflow".into()))
}
fn cstring(s: &str) -> Result<CString, ProblemError> {
    CString::new(s).map_err(|_| ProblemError::Contract("NUL in native option".into()))
}
fn native_bound(x: f64) -> Result<f64, ProblemError> {
    if x.is_nan() || x.is_finite() && x.abs() >= INFINITY {
        return Err(ProblemError::Unsupported(
            "finite bound reaches Ipopt infinity threshold".into(),
        ));
    }
    Ok(if x == f64::INFINITY {
        INFINITY
    } else if x == f64::NEG_INFINITY {
        -INFINITY
    } else {
        x
    })
}
/// Map every pinned Ipopt return status without claiming global NLP certificates.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category, assurance) = match code {
        ffi::ApplicationReturnStatus_Solve_Succeeded => {
            ("Solve_Succeeded", Termination::Success, Assurance::None)
        }
        ffi::ApplicationReturnStatus_Solved_To_Acceptable_Level => (
            "Solved_To_Acceptable_Level",
            Termination::Acceptable,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Infeasible_Problem_Detected => (
            "Infeasible_Problem_Detected",
            Termination::Infeasible,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Search_Direction_Becomes_Too_Small => (
            "Search_Direction_Becomes_Too_Small",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Diverging_Iterates => (
            "Diverging_Iterates",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_User_Requested_Stop => (
            "User_Requested_Stop",
            Termination::Cancelled,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Feasible_Point_Found => (
            "Feasible_Point_Found",
            Termination::FeasibleOnly,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_Iterations_Exceeded => (
            "Maximum_Iterations_Exceeded",
            Termination::IterationLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Restoration_Failed => (
            "Restoration_Failed",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Error_In_Step_Computation => (
            "Error_In_Step_Computation",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_CpuTime_Exceeded => (
            "Maximum_CpuTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_WallTime_Exceeded => (
            "Maximum_WallTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Not_Enough_Degrees_Of_Freedom => (
            "Not_Enough_Degrees_Of_Freedom",
            Termination::Invalid,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Invalid_Problem_Definition => (
            "Invalid_Problem_Definition",
            Termination::Invalid,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Invalid_Option => {
            ("Invalid_Option", Termination::Invalid, Assurance::None)
        }
        ffi::ApplicationReturnStatus_Invalid_Number_Detected => (
            "Invalid_Number_Detected",
            Termination::Evaluation,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Unrecoverable_Exception => (
            "Unrecoverable_Exception",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_NonIpopt_Exception_Thrown => (
            "NonIpopt_Exception_Thrown",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Insufficient_Memory => (
            "Insufficient_Memory",
            Termination::ResourceExhausted,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Internal_Error => {
            ("Internal_Error", Termination::Invalid, Assurance::None)
        }
        _ => (
            "Unknown_Ipopt_Status",
            Termination::Inconclusive,
            Assurance::None,
        ),
    };
    NativeTermination {
        code: i64::from(code),
        name: name.into(),
        message: None,
        category,
        assurance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_ipopt_preserves_tight_rows_with_independent_kkt_budgets() {
        use crate::{
            quality::Tolerances,
            solve::*,
            solver_tests::{Polynomial, stamp},
        };
        use std::sync::{Arc, atomic::AtomicBool};
        let mut oracle = Polynomial::new();
        oracle.c.variables[0].lower = 0.1;
        oracle.c.variables[0].upper = 2.0;
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy {
            feasibility: 1e-12,
            stationarity: 2e-8,
            complementarity: 3e-9,
            ..ResolvedAccuracy::verification()
        };
        let tolerances = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-12],
            integrality: 1e-8,
        };
        let report = Session::new()
            .solve(
                &mut oracle,
                &[1.5],
                pse_math::binding::ObjectiveSense::Minimize,
                &controls,
                &accuracy,
                &Settings::default(),
                Execution::new(Arc::new(AtomicBool::new(false)), &controls),
                &tolerances,
                None,
                stamp(Backend::Ipopt),
            )
            .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{:?}",
            report.termination
        );
        assert!(report.quality.as_ref().is_some_and(|q| q.feasible()));
        assert_eq!(report.options["tol"], OptionValue::Real(2e-8));
        assert_eq!(report.options["constr_viol_tol"], OptionValue::Real(1e-12));
        assert_eq!(report.options["dual_inf_tol"], OptionValue::Real(2e-8));
        assert_eq!(report.options["compl_inf_tol"], OptionValue::Real(3e-9));
        assert_eq!(report.options["acceptable_iter"], OptionValue::Integer(0));
        assert_eq!(report.options["bound_relax_factor"], OptionValue::Real(0.0));
        let candidate = report.candidate.as_ref().unwrap();
        assert!((candidate.primal[0].powi(3) - 1.0).abs() <= 1e-12);
        assert!((0.1..=2.0).contains(&candidate.primal[0]));
    }
    #[test]
    fn status_scope_and_finite_infinity_are_not_conflated() {
        assert_eq!(termination(2).assurance, Assurance::None);
        assert_eq!(termination(6).assurance, Assurance::None);
        assert_eq!(termination(6).category, Termination::FeasibleOnly);
        for code in [
            0, 1, 2, 3, 4, 5, 6, -1, -2, -3, -4, -5, -10, -11, -12, -13, -100, -101, -102, -199,
        ] {
            assert!(!termination(code).name.starts_with("Unknown"))
        }
        assert!(native_bound(INFINITY).is_err());
        assert_eq!(native_bound(f64::INFINITY).unwrap(), INFINITY);
        assert!(index(usize::MAX).is_err());
    }
}
