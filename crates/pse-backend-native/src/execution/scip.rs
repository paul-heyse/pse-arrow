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
use pse_model::generated::enums::NativeConstraintForm;

/// Typed SCIP settings. Reserved native options derive from these and the shared controls;
/// identity derives from serde.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "ScipSettings")]
pub struct Settings {
    /// Linear solver of the nested Ipopt (`nlpi/ipopt/linear_solver`): the image's one
    /// Ipopt, so the same type the Ipopt adapter's settings select.
    pub nlp_linear_solver: IpoptLinearSolver,
    /// Random seed shift (`randomization/randomseedshift`), recorded with the result.
    pub seed: u16,
    /// Total branch-and-bound node budget including restarts (`limits/totalnodes`);
    /// absent leaves the deadline as the only budget.
    pub nodes: Option<u32>,
    /// Number of ranked stored solutions reported beside the candidate (0: none).
    pub pool: u16,
    /// After a proof of infeasibility, compute an irreducible infeasible subsystem of the
    /// true exported program (`SCIPgenerateIIS`).
    pub iis: bool,
    /// Exact rational MILP (`SCIPenableExactSolving`); admitted for linear programs
    /// without native forms, and never with reoptimization, IIS generation (SCIP's IIS
    /// finders do not support exact solving) or concurrency.
    pub exact: bool,
    /// Retain the native search tree across a finite MIP sequence whose constraint system
    /// is unchanged and whose linear objective changes (`SCIPenableReoptimization`).
    pub reoptimize: bool,
}
impl Default for Settings {
    /// Sequential MUMPS in the nested Ipopt, seed shift 0, no node budget, no pool, and
    /// neither IIS, exact solving nor reoptimization.
    fn default() -> Self {
        Self {
            nlp_linear_solver: IpoptLinearSolver::Mumps,
            seed: 0,
            nodes: None,
            pool: 0,
            iis: false,
            exact: false,
            reoptimize: false,
        }
    }
}
impl Settings {
    /// Refuse a nested linear solver whose process preconditions are unmet and every
    /// mutually exclusive mode combination.
    ///
    /// # Errors
    /// SPRAL without `OMP_CANCELLATION=TRUE`; exact mode with reoptimization, IIS
    /// generation or concurrency; reoptimization with concurrency.
    pub fn admit(&self, threads: usize) -> Result<(), ProblemError> {
        if self.nlp_linear_solver == IpoptLinearSolver::Spral
            && !std::env::var("OMP_CANCELLATION").is_ok_and(|v| v.eq_ignore_ascii_case("true"))
        {
            return Err(ProblemError::Unsupported(
                "SPRAL in SCIP's nested Ipopt needs OMP_CANCELLATION=TRUE".into(),
            ));
        }
        let refuse = |why: &str| Err(ProblemError::Unsupported(why.into()));
        if self.exact && self.reoptimize {
            return refuse("SCIP exact solving excludes reoptimization");
        }
        if self.exact && self.iis {
            return refuse("SCIP IIS generation does not support exact solving");
        }
        if threads > 1 && (self.exact || self.reoptimize) {
            return refuse("SCIP concurrent solving excludes exact solving and reoptimization");
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
    // Automatic only where HiGHS, Ipopt or POUNCE do not own the class: mixed-integer
    // programs, and smooth NLP behind the local NLP adapters. Continuous linear and
    // quadratic programs select SCIP explicitly.
    automatic_classes: &[
        ProblemClass::MixedLinear,
        ProblemClass::MixedIntegerQuadratic,
        ProblemClass::MixedIntegerNonlinear,
        ProblemClass::SmoothNlp,
    ],
    derivatives: DerivativeCapability::Factorable,
    warm: WarmCapability::Primal,
    general_bounds: true,
    sign_bounds: true,
    // Concurrent solving in deterministic mode, with the admitted permits as threads; the
    // concurrent solvers' improving incumbents stream while the search runs.
    parallel: true,
    certifies: true,
    // Indicator (nonlinear rows lifted exactly through slacks), SOS1/SOS2, and/or/xor and
    // cardinality handlers; an asserted `or` is upgraded to logicor by SCIP presolve.
    native_forms: &[
        NativeConstraintForm::Indicator,
        NativeConstraintForm::Sos1,
        NativeConstraintForm::Sos2,
        NativeConstraintForm::And,
        NativeConstraintForm::Or,
        NativeConstraintForm::Xor,
        NativeConstraintForm::Cardinality,
    ],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: true,
    reuse: "one SCIP instance per attempt, created and freed on the owning worker; with reoptimization, one instance and its search tree across a finite MIP sequence whose constraint system is unchanged",
    cancellation: "event handler on presolve rounds, node focus and solve, and LP solves calls SCIPinterruptSolve; copied into sub-SCIPs, concurrent solvers and the IIS sub-problem",
    diagnostics: "raw status, primal and dual bound, gap, node count, export fidelity and readback, effective reserved options, ranked solution pool, IIS with its irreducibility flag, exact rational objective",
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
        // After HiGHS, Ipopt and POUNCE within the classes they share.
        Some(5)
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
    ) -> Result<(), ProblemError> {
        match settings {
            BackendSettings::Default => Settings::default().admit(controls.threads),
            BackendSettings::Scip(s) => s.admit(controls.threads),
            _ => Err(super::foreign(Backend::Scip)),
        }
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
        #[cfg(feature = "scip")]
        {
            // Only a reoptimization session retains native state across attempts.
            crate::scip::solve(
                &crate::scip::Request {
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
                },
                retained,
            )
        }
        #[cfg(not(feature = "scip"))]
        {
            let _ = (program, initial, intent, normalization, settings);
            retained.clear();
            Err(super::unlinked(Backend::Scip))
        }
    }
}
