// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ipopt adapter: native defaults only, a retained worker-local C problem, NLP starts.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, DerivativeCapability, ProblemClass, SolveReport, WarmCapability, WarmPayload,
    },
};

#[derive(Debug)]
pub(super) struct Ipopt;
pub(super) static ADAPTER: Ipopt = Ipopt;
static CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::SmoothNlp],
    derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
    warm: WarmCapability::PrimalDual,
    general_bounds: true,
    sign_bounds: true,
    parallel: false,
    certifies: false,
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
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::Nlp {
            primal,
            bounds: None,
            rows: None,
        })
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
        if !matches!(input.settings, BackendSettings::Default) {
            return Err(super::foreign(Backend::Ipopt));
        }
        #[cfg(feature = "ipopt")]
        {
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
                input.execution,
                input.tolerances,
                input.warm,
                input.compatibility,
            )
        }
        #[cfg(not(feature = "ipopt"))]
        {
            let _ = (retained, &mut oracle, initial, sense);
            Err(super::unlinked(Backend::Ipopt))
        }
    }
}
