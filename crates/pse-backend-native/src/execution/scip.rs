// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! SCIP adapter (ADR-0105): the factorable representation, one native instance per
//! attempt, primal incumbents and the certify intent. The FFI lives in `crate::scip`.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, Controls, DerivativeCapability, IpoptLinearSolver, ProblemClass, SolveReport,
        WarmCapability, WarmPayload,
    },
};

/// Typed SCIP settings. Reserved native options derive from these and the shared controls;
/// identity derives from serde.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Linear solver of the nested Ipopt (`nlpi/ipopt/linear_solver`): the image's one
    /// Ipopt, so the same type the Ipopt adapter's settings select.
    pub nlp_linear_solver: IpoptLinearSolver,
    /// Random seed shift (`randomization/randomseedshift`), recorded with the result.
    pub seed: u16,
    /// Total branch-and-bound node budget including restarts (`limits/totalnodes`);
    /// absent leaves the deadline as the only budget.
    pub nodes: Option<u32>,
}
impl Settings {
    /// Refuse a nested linear solver whose process preconditions are unmet.
    ///
    /// # Errors
    /// SPRAL without `OMP_CANCELLATION=TRUE`.
    pub fn admit(&self) -> Result<(), ProblemError> {
        if self.nlp_linear_solver == IpoptLinearSolver::Spral
            && !std::env::var("OMP_CANCELLATION").is_ok_and(|v| v.eq_ignore_ascii_case("true"))
        {
            return Err(ProblemError::Unsupported(
                "SPRAL in SCIP's nested Ipopt needs OMP_CANCELLATION=TRUE".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct Scip;
pub(super) static ADAPTER: Scip = Scip;
static CAPABILITY: Capability = Capability {
    classes: &[
        ProblemClass::Linear,
        ProblemClass::MixedLinear,
        ProblemClass::ConvexQuadratic,
        ProblemClass::NonconvexQuadratic,
        ProblemClass::MixedIntegerQuadratic,
        ProblemClass::MixedIntegerNonlinear,
        ProblemClass::SmoothNlp,
    ],
    derivatives: DerivativeCapability::Factorable,
    warm: WarmCapability::Primal,
    general_bounds: true,
    sign_bounds: true,
    parallel: false,
    certifies: true,
    // The indicator, SOS, logic and cardinality handlers are consumed from Plan 22 G7.
    native_forms: &[],
    reuse: "none: one SCIP instance per attempt, created and freed on the owning worker",
    cancellation: "event handler on presolve rounds, node focus and solve, and LP solves calls SCIPinterruptSolve",
    diagnostics: "raw status, primal and dual bound, gap, node count, export fidelity and readback, effective reserved options",
};
impl BackendExecution for Scip {
    fn backend(&self) -> Backend {
        Backend::Scip
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Factorable
    }
    fn linked(&self) -> bool {
        cfg!(feature = "scip")
    }
    fn automatic(&self) -> Option<u8> {
        // After HiGHS, Ipopt and POUNCE, so it is automatic only where no other adapter
        // represents the class (mixed-integer quadratic and nonlinear programs).
        Some(5)
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
    ) -> Result<(), ProblemError> {
        match settings {
            BackendSettings::Default => {}
            BackendSettings::Scip(s) => s.admit()?,
            _ => return Err(super::foreign(Backend::Scip)),
        }
        if controls.threads != 1 {
            return Err(ProblemError::Unsupported(
                "SCIP runs serial until concurrent mode is admitted (Plan 22 G7)".into(),
            ));
        }
        Ok(())
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::primal(primal))
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(payload, WarmPayload::Nlp { .. })
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        let super::Problem::Factorable {
            program,
            initial,
            intent,
            normalization,
        } = input.problem
        else {
            return Err(super::representation(Backend::Scip));
        };
        let defaults = Settings::default();
        let settings = match input.settings {
            BackendSettings::Default => &defaults,
            BackendSettings::Scip(s) => s,
            _ => return Err(super::foreign(Backend::Scip)),
        };
        // Nothing native is retained across attempts.
        retained.clear();
        #[cfg(feature = "scip")]
        {
            crate::scip::solve(&crate::scip::Request {
                program,
                initial,
                intent,
                normalization,
                settings,
                controls: input.controls,
                accuracy: input.accuracy,
                execution: &input.execution,
                warm: input.warm,
                compatibility: &input.compatibility,
            })
        }
        #[cfg(not(feature = "scip"))]
        {
            let _ = (program, initial, intent, normalization, settings);
            Err(super::unlinked(Backend::Scip))
        }
    }
}
