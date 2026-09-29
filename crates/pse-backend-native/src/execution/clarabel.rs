// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Clarabel adapter: explicit cones, and linear and convex quadratic coefficient programs the
//! coefficient runner lowers to cone form; complete native settings, a preprocessing or
//! data-update mode over a retained native solver, and a QDLDL or MKL Pardiso KKT solver.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError, conic,
    solve::{
        Backend, Controls, DerivativeCapability, Metric, ProblemClass, SolveReport,
        WarmCapability,
    },
};

#[derive(Debug)]
pub(super) struct Clarabel;
pub(super) static ADAPTER: Clarabel = Clarabel;
static CAPABILITY: Capability = Capability {
    classes: &[
        ProblemClass::Linear,
        ProblemClass::ConvexQuadratic,
        ProblemClass::ContinuousCone,
    ],
    // Linear and convex quadratic programs belong to HiGHS automatically; Clarabel serves
    // them when selected explicitly (ADR-0121).
    automatic_classes: &[ProblemClass::ContinuousCone],
    derivatives: DerivativeCapability::Coefficients,
    warm: WarmCapability::None,
    general_bounds: true,
    sign_bounds: true,
    // MKL Pardiso admits the worker's native threads; QDLDL refuses more than one at
    // settings admission.
    parallel: cfg!(feature = "clarabel-pardiso"),
    certifies: false,
    native_forms: &[],
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
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
    ) -> Result<(), ProblemError> {
        let direct = match settings {
            BackendSettings::Default => conic::Settings::default().direct,
            BackendSettings::Clarabel(settings) => settings.direct,
            _ => return Err(super::foreign(Backend::Clarabel)),
        };
        conic::admit_threads(direct, controls.threads)
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
        let defaults = conic::Settings::default();
        let settings = match input.settings {
            BackendSettings::Default => &defaults,
            BackendSettings::Clarabel(settings) => settings,
            #[allow(
                unreachable_patterns,
                reason = "other adapters' variants exist only when their features are linked"
            )]
            _ => return Err(super::foreign(Backend::Clarabel)),
        };
        // MKL Pardiso runs on the owning worker's oneMKL-local thread count (ADR-0108
        // item 12); QDLDL is serial.
        #[cfg(feature = "clarabel-pardiso")]
        let _threads = (settings.direct == conic::Direct::MklPardiso)
            .then(|| crate::mkl::Threads::enter(input.controls.threads))
            .transpose()?;
        let stamp = input.compatibility;
        let (session, reused) = retained.session(
            Backend::Clarabel,
            input.controls.reuse,
            |session: &mut conic::Session| {
                Ok(session
                    .update(problem, certificate, settings, stamp.clone())
                    .is_ok())
            },
            || {
                conic::Session::new(
                    problem,
                    certificate,
                    input.controls,
                    input.accuracy,
                    settings,
                    stamp.clone(),
                )
            },
        )?;
        let mut report = session.solve(
            problem,
            input.controls,
            input.accuracy,
            settings,
            input.execution,
            input.tolerances,
        )?;
        report
            .metrics
            .insert("reuse.native_model".into(), Metric::Bool(reused));
        report.evidence.reused_native_state = reused;
        Ok(report)
    }
}
