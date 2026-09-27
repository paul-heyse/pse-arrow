// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Clarabel adapter: explicit cones, complete native settings and a preprocessing or
//! data-update mode over a retained native solver.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError, conic,
    solve::{Backend, DerivativeCapability, Metric, ProblemClass, SolveReport, WarmCapability},
};

#[derive(Debug)]
pub(super) struct Clarabel;
pub(super) static ADAPTER: Clarabel = Clarabel;
static CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::ContinuousCone],
    derivatives: DerivativeCapability::Coefficients,
    warm: WarmCapability::None,
    general_bounds: true,
    sign_bounds: true,
    parallel: false,
    reuse: "native data-update eligibility; reusable mode disables preprocessing",
    cancellation: "native iteration termination callback",
    diagnostics: "complete native info/settings, cone slacks/duals and certificates",
};
impl BackendExecution for Clarabel {
    fn backend(&self) -> Backend {
        Backend::Clarabel
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Cone
    }
    fn linked(&self) -> bool {
        true
    }
    fn automatic(&self) -> Option<u8> {
        Some(4)
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        let super::Problem::Cone {
            problem,
            certificate,
        } = input.problem
        else {
            return Err(super::representation(Backend::Clarabel));
        };
        let (native, mode) = match input.settings {
            BackendSettings::Default => (conic::Settings::default(), conic::Mode::SingleSolve),
            BackendSettings::Clarabel { native, mode } => ((**native).clone(), *mode),
            #[allow(
                unreachable_patterns,
                reason = "other adapters' variants exist only when their features are linked"
            )]
            _ => return Err(super::foreign(Backend::Clarabel)),
        };
        let stamp = input.compatibility;
        let (session, reused) = retained.session(
            Backend::Clarabel,
            input.controls.reuse,
            |session: &mut conic::Session| {
                Ok(session.update(problem, certificate, stamp.clone()).is_ok())
            },
            || {
                conic::Session::new(
                    problem,
                    certificate,
                    input.controls,
                    native.clone(),
                    mode,
                    stamp.clone(),
                )
            },
        )?;
        let mut report = session.solve(
            problem,
            input.controls,
            native,
            input.execution,
            input.tolerances,
        )?;
        report
            .metrics
            .insert("reuse.native_model".into(), Metric::Bool(reused));
        Ok(report)
    }
}
