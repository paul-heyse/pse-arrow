// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit first-order Uno SQP/SLP profiles; routing never selects them implicitly.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, Controls, DerivativeCapability, HessianMode, ProblemClass, SolveReport,
        WarmCapability, WarmPayload,
    },
};
#[derive(Debug)]
pub(super) struct Uno;
pub(super) static ADAPTER: Uno = Uno;
static CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::Equalities,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SmoothNlp],
    automatic_classes: &[],
    derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
    warm: WarmCapability::Primal,
    general_bounds: true,
    sign_bounds: true,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: false,
    reuse: "fresh Uno attempt; shared HiGHS scheduler provider",
    cancellation: "callback and QP/LP boundaries; indivisible HiGHS call has remaining time limit",
    diagnostics: "native stop, iterate status, objective/residuals, iterations and callback counts",
};
impl BackendExecution for Uno {
    fn backend(&self) -> Backend {
        Backend::Uno
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Nlp
    }
    fn linked(&self) -> bool {
        cfg!(feature = "uno")
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn build(&self) -> Option<pse_ids::ContentHash> {
        #[cfg(feature = "uno")]
        {
            Some(crate::uno::build())
        }
        #[cfg(not(feature = "uno"))]
        {
            None
        }
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
        _: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        let defaults = crate::settings::uno::Settings::default();
        let selected = match settings {
            BackendSettings::Default => &defaults,
            BackendSettings::Uno(s) => s,
            _ => return Err(super::foreign(Backend::Uno)),
        };
        selected.validate()?;
        if controls.threads != 1
            || !controls.options.is_empty()
            || controls.hessian != HessianMode::LimitedMemory
        {
            return Err(ProblemError::Unsupported(
                "Uno profiles require one thread, typed settings and first-order limited memory"
                    .into(),
            ));
        }
        Ok(())
    }
    fn admit_contract(
        &self,
        _: &crate::OracleContract,
        _: &std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
        settings: &BackendSettings,
        budgets: super::Budgets<'_>,
    ) -> Result<(), ProblemError> {
        let defaults = crate::settings::uno::Settings::default();
        match settings {
            BackendSettings::Default => defaults.validate()?,
            BackendSettings::Uno(s) => s.validate()?,
            _ => return Err(super::foreign(Backend::Uno)),
        };
        if budgets.accuracy.native_scaling || budgets.accuracy.acceptable.is_some() {
            return Err(ProblemError::Unsupported(
                "Uno profile does not consume native scaling or relaxed KKT acceptance".into(),
            ));
        }
        Ok(())
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::primal(primal))
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(
            payload,
            WarmPayload::Nlp {
                bounds: None,
                rows: None,
                barrier: None,
                working: None,
                ..
            }
        )
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        #[cfg(feature = "uno")]
        {
            let super::Problem::Nlp {
                mut oracle,
                initial,
                sense,
            } = input.problem
            else {
                return Err(super::representation(Backend::Uno));
            };
            let defaults = crate::settings::uno::Settings::default();
            let settings = match input.settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Uno(s) => s,
                _ => return Err(super::foreign(Backend::Uno)),
            };
            // No direct HiGHS session may remain on this worker before Uno takes its
            // exclusive serial scheduler owner. Semantic response products are separate.
            retained.clear();
            crate::uno::solve(
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
        #[cfg(not(feature = "uno"))]
        {
            let _ = (retained, input);
            Err(super::unlinked(Backend::Uno))
        }
    }
}
