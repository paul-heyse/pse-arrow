// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ipopt adapter: typed linear-solver settings admitted against the linked library and the
//! process environment, SPRAL and oneMKL threads, a retained worker-local C problem and NLP
//! starts.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, Controls, DerivativeCapability, ProblemClass, SolveReport, WarmCapability,
        WarmPayload,
    },
};
use pse_ids::ContentHash;

#[derive(Debug)]
pub(super) struct Ipopt;
pub(super) static ADAPTER: Ipopt = Ipopt;
static CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::Equalities,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SmoothNlp],
    automatic_classes: &[ProblemClass::SmoothNlp],
    derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
    warm: WarmCapability::PrimalDual,
    general_bounds: true,
    sign_bounds: true,
    // SPRAL (OpenMP) and oneMKL Pardiso (MKL threads) consume admitted threads; MUMPS is
    // sequential and its settings refuse more than one.
    parallel: true,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: true,
    reuse: "same sparse layout and bounds: retained C problem",
    cancellation: "intermediate/evaluation checkpoints",
    diagnostics: "native current iterate, violations, callback counts and timing",
};
impl BackendExecution for Ipopt {
    fn backend(&self) -> Backend {
        Backend::Ipopt
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Nlp
    }
    fn linked(&self) -> bool {
        cfg!(feature = "ipopt")
    }
    fn automatic(&self) -> Option<u8> {
        Some(2)
    }
    fn build(&self) -> Option<ContentHash> {
        #[cfg(feature = "ipopt")]
        {
            Some(crate::ipopt::build().identity)
        }
        #[cfg(not(feature = "ipopt"))]
        {
            None
        }
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
        snapshot: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        if matches!(
            controls.hessian,
            crate::solve::HessianMode::Auto
                | crate::solve::HessianMode::Partitioned
                | crate::solve::HessianMode::FiniteDifference
        ) {
            return Err(ProblemError::Unsupported(
                "Ipopt does not implement this typed curvature mode".into(),
            ));
        }
        #[cfg(feature = "ipopt")]
        {
            let defaults = crate::ipopt::Settings::default();
            let settings = match settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Ipopt(settings) => settings,
                _ => return Err(super::foreign(Backend::Ipopt)),
            };
            crate::ipopt::admit(
                settings,
                controls.threads,
                snapshot.ipopt.as_ref().ok_or_else(|| {
                    ProblemError::Unsupported("Ipopt runtime observation missing".into())
                })?,
            )
        }
        #[cfg(not(feature = "ipopt"))]
        {
            let _ = (controls, snapshot);
            if !matches!(settings, BackendSettings::Default) {
                return Err(super::foreign(Backend::Ipopt));
            }
            Err(super::unlinked(Backend::Ipopt))
        }
    }
    fn validate_snapshot(
        &self,
        settings: &BackendSettings,
        snapshot: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        snapshot.validate_for(self)?;
        #[cfg(feature = "ipopt")]
        {
            let defaults = crate::ipopt::Settings::default();
            let settings = match settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Ipopt(settings) => settings,
                _ => return Err(super::foreign(Backend::Ipopt)),
            };
            let before = snapshot.ipopt.as_ref().ok_or_else(|| {
                ProblemError::Unsupported("Ipopt runtime observation missing".into())
            })?;
            if !crate::ipopt::same_admission(settings, before, &crate::ipopt::Runtime::observe()) {
                return Err(ProblemError::Unsupported(
                    "selected Ipopt runtime observation changed".into(),
                ));
            }
        }
        #[cfg(not(feature = "ipopt"))]
        let _ = settings;
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
        let super::Problem::Nlp {
            mut oracle,
            initial,
            sense,
        } = input.problem
        else {
            return Err(super::representation(Backend::Ipopt));
        };
        #[cfg(feature = "ipopt")]
        {
            let defaults = crate::ipopt::Settings::default();
            let settings = match input.settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Ipopt(settings) => settings,
                _ => return Err(super::foreign(Backend::Ipopt)),
            };
            let (session, _) = retained.session(
                Backend::Ipopt,
                input.controls.reuse,
                |_: &mut crate::ipopt::Session| Ok(true),
                || Ok(crate::ipopt::Session::new()),
            )?;
            session.solve(
                oracle.as_mut(),
                initial,
                sense,
                input.controls,
                input.accuracy,
                settings,
                input.execution,
                input.tolerances,
                input.warm,
                input.compatibility,
            )
        }
        #[cfg(not(feature = "ipopt"))]
        {
            let _ = (retained, &mut oracle, initial, sense);
            if !matches!(input.settings, BackendSettings::Default) {
                return Err(super::foreign(Backend::Ipopt));
            }
            Err(super::unlinked(Backend::Ipopt))
        }
    }
}
