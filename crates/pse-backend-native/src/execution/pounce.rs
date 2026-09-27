// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! POUNCE adapter: typed method and FERAL settings, an admitted local pool, NLP and
//! active-set working-set starts.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, DerivativeCapability, ProblemClass, SolveReport, WarmCapability, WarmPayload,
    },
};

#[derive(Debug)]
pub(super) struct Pounce;
pub(super) static ADAPTER: Pounce = Pounce;
static CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::SmoothNlp],
    derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
    warm: WarmCapability::PrimalDualAndWorkingSet,
    general_bounds: true,
    sign_bounds: true,
    parallel: true,
    certifies: false,
    reuse: "native application and compatible starts; iteration factors are library-owned",
    cancellation: "TNLP intermediate/evaluation checkpoints",
    diagnostics: "complete SolveStatistics, phase timing, FERAL inertia/pivots/fill, restoration and crossover",
};
impl BackendExecution for Pounce {
    fn backend(&self) -> Backend {
        Backend::Pounce
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Nlp
    }
    fn linked(&self) -> bool {
        cfg!(feature = "pounce")
    }
    fn automatic(&self) -> Option<u8> {
        Some(3)
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::Nlp {
            primal,
            bounds: None,
            rows: None,
        })
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        match payload {
            WarmPayload::Nlp { .. } => true,
            #[cfg(feature = "pounce")]
            WarmPayload::PounceSqp(_) => true,
            WarmPayload::Root(_) | WarmPayload::Highs { .. } => false,
        }
    }
    fn scope(
        &self,
        threads: usize,
        stack: usize,
        work: &mut (dyn FnMut() + Send),
    ) -> Result<(), ProblemError> {
        #[cfg(feature = "pounce")]
        {
            crate::pounce::with_threads(threads, stack, move || {
                work();
                Ok::<(), ProblemError>(())
            })
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = (threads, stack);
            work();
            Ok(())
        }
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        let super::Problem::Nlp {
            oracle,
            initial,
            sense,
        } = input.problem
        else {
            return Err(super::representation(Backend::Pounce));
        };
        #[cfg(feature = "pounce")]
        {
            let defaults = crate::pounce::Settings::default();
            let settings = match input.settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Pounce(settings) => settings,
                _ => return Err(super::foreign(Backend::Pounce)),
            };
            let (session, _) = retained.session(
                Backend::Pounce,
                input.controls.reuse,
                |_: &mut crate::pounce::Session| Ok(true),
                || Ok(crate::pounce::Session::new()),
            )?;
            session.solve(
                oracle,
                initial,
                sense,
                input.controls,
                settings.method,
                settings.linear.clone(),
                input.execution,
                input.tolerances,
                input.warm,
                input.compatibility,
            )
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = (retained, oracle, initial, sense);
            if !matches!(input.settings, BackendSettings::Default) {
                return Err(super::foreign(Backend::Pounce));
            }
            Err(super::unlinked(Backend::Pounce))
        }
    }
}
