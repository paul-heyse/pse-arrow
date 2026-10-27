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
    structural: crate::structural::Policy::NativeFeasibility,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
    automatic_classes: &[],
    // Forward sensitivities and adjoint gradients (Plan 22 Y3a).
    derivatives: DerivativeCapability::ForwardAndAdjointSensitivities,
    warm: WarmCapability::None,
    general_bounds: false,
    sign_bounds: false,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: false,
    reuse: "worker-local BDF, SDIRK or explicit Runge-Kutta state with faer LU or KLU factors",
    cancellation: "cooperative callbacks and step boundaries",
    diagnostics: "native statistics per segment and scheme, consistent starts, partial samples, root transitions and reset sensitivities",
};
static IDAS_CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::NativeFeasibility,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::Ode, ProblemClass::SemiExplicitIndex1],
    automatic_classes: &[],
    // Forward sensitivities, adjoint gradients and the second-order adjoint of an exact
    // transient Hessian (Plan 22 Y3b, Y4b).
    derivatives: DerivativeCapability::SecondOrderAdjointSensitivities,
    warm: WarmCapability::None,
    general_bounds: false,
    sign_bounds: false,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: false,
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
    fn build(&self) -> Option<pse_ids::ContentHash> {
        #[cfg(feature = "diffsol")]
        {
            diffsol_profile(
                include_str!(concat!(env!("OUT_DIR"), "/diffsol-source.txt")),
                include_bytes!(concat!(env!("OUT_DIR"), "/kinsol-klu-provider.json")),
            )
        }
        #[cfg(not(feature = "diffsol"))]
        {
            None
        }
    }
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

#[cfg(any(feature = "diffsol", test))]
fn diffsol_profile(source: &str, provider: &[u8]) -> Option<pse_ids::ContentHash> {
    if source.trim().is_empty() {
        return None;
    }
    // Native admission owns current provider/file qualification. An absent
    // embedded receipt keeps direct calls available but refuses portable identity.
    let document: serde_json::Value = serde_json::from_slice(provider).ok()?;
    for field in ["identity", "files"] {
        if document
            .get(field)?
            .as_object()
            .is_none_or(|values| values.is_empty())
        {
            return None;
        }
    }
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NativeBuildV1);
    h.str("diffsol-profile.v1").str(source).part(provider);
    Some(h.finish_hash())
}

impl BackendExecution for Idas {
    fn build(&self) -> Option<pse_ids::ContentHash> {
        #[cfg(feature = "idas")]
        {
            crate::sundials_profile::build()
        }
        #[cfg(not(feature = "idas"))]
        {
            None
        }
    }
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

#[cfg(test)]
mod build_tests {
    use super::*;
    const PROVIDER: &[u8] = br#"{"identity":{"provider":"KLU"},"files":{"lib/libklu.a":"digest"}}"#;
    #[test]
    fn diffsol_profile_binds_same_version_source_and_selected_klu_receipt() {
        let original = diffsol_profile("checked diffsol source", PROVIDER).unwrap();
        assert_eq!(
            diffsol_profile("checked diffsol source", PROVIDER),
            Some(original)
        );
        assert_ne!(
            diffsol_profile("same version changed numerical delta", PROVIDER),
            Some(original)
        );
        assert_ne!(
            diffsol_profile(
                "checked diffsol source",
                br#"{"identity":{"provider":"KLU"},"files":{"lib/libklu.a":"changed digest"}}"#
            ),
            Some(original)
        );
        for missing in [
            b"".as_slice(),
            b"{}",
            b"not json",
            br#"{"identity":{},"files":{"lib":1}}"#,
        ] {
            assert!(diffsol_profile("checked diffsol source", missing).is_none());
        }
        assert!(diffsol_profile("", PROVIDER).is_none());
    }
    #[cfg(feature = "diffsol")]
    #[test]
    fn linked_diffsol_exposes_selected_provider_build() {
        let actual = diffsol_profile(
            include_str!(concat!(env!("OUT_DIR"), "/diffsol-source.txt")),
            include_bytes!(concat!(env!("OUT_DIR"), "/kinsol-klu-provider.json")),
        )
        .expect("selected native test requires qualified KLU provider evidence");
        assert_eq!(
            crate::execution::adapter(Backend::Diffsol).build(),
            Some(actual)
        );
    }
}
