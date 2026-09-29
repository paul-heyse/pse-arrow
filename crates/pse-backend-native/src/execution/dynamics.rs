// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dynamics adapters publish their capability records; integration runs through the
//! integrator workflows, so the algebraic router never selects them.
use super::{BackendExecution, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{Backend, DerivativeCapability, ProblemClass, SolveReport, WarmCapability},
};

#[derive(Debug)]
pub(super) struct Diffsol;
pub(super) static DIFFSOL: Diffsol = Diffsol;
#[derive(Debug)]
pub(super) struct Idas;
pub(super) static IDAS: Idas = Idas;
static DIFFSOL_CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
    automatic_classes: &[],
    derivatives: DerivativeCapability::FirstWithSmoothSensitivities,
    warm: WarmCapability::None,
    general_bounds: false,
    sign_bounds: false,
    parallel: false,
    certifies: false,
    native_forms: &[],
    reuse: "worker-local BDF, SDIRK or explicit Runge-Kutta state with faer LU or KLU factors",
    cancellation: "cooperative callbacks and step boundaries",
    diagnostics: "native statistics per segment and scheme, consistent starts, partial samples, root transitions and reset sensitivities",
};
static IDAS_CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
    automatic_classes: &[],
    derivatives: DerivativeCapability::FirstWithSmoothSensitivities,
    warm: WarmCapability::None,
    general_bounds: false,
    sign_bounds: false,
    parallel: false,
    certifies: false,
    native_forms: &[],
    reuse: "worker-local IDAS memory, restarted in place at scheduled changes and resets",
    cancellation: "residual callbacks and native step boundaries",
    diagnostics: "native statuses and per-segment counters, consistent and steady starts, recoverable residual trials, roots, sign constraints, Krylov iterations and forward sensitivities",
};
fn trajectory(backend: Backend) -> ProblemError {
    ProblemError::Unsupported(format!(
        "{} integrates trajectories through the dynamics workflow",
        backend.as_str()
    ))
}
impl BackendExecution for Diffsol {
    fn backend(&self) -> Backend {
        Backend::Diffsol
    }
    fn capability(&self) -> &'static Capability {
        &DIFFSOL_CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Trajectory
    }
    fn linked(&self) -> bool {
        cfg!(feature = "diffsol")
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn execute(&self, _: &mut Retained, _: Input<'_>) -> Result<SolveReport, ProblemError> {
        Err(trajectory(Backend::Diffsol))
    }
}
impl BackendExecution for Idas {
    fn backend(&self) -> Backend {
        Backend::Idas
    }
    fn capability(&self) -> &'static Capability {
        &IDAS_CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Trajectory
    }
    fn linked(&self) -> bool {
        cfg!(feature = "idas")
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn execute(&self, _: &mut Retained, _: Input<'_>) -> Result<SolveReport, ProblemError> {
        Err(trajectory(Backend::Idas))
    }
}
