// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit serial PETSc root profiles; bounds remain an original routing obligation.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, Controls, DerivativeCapability, ProblemClass, SolveReport, WarmCapability,
        WarmPayload,
    },
};
#[derive(Debug)]
pub(super) struct Petsc;
pub(super) static ADAPTER: Petsc = Petsc;
static CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::Roots,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SquareRoot],
    automatic_classes: &[],
    derivatives: DerivativeCapability::JacobianOrProduct,
    warm: WarmCapability::Primal,
    general_bounds: false,
    sign_bounds: false,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: false,
    reuse: "fresh scoped SNES handles; retained-state requests require an admitted reuse owner",
    cancellation: "residual, Jacobian, nonlinear convergence and pseudo-time stage checkpoints",
    diagnostics: "native nonlinear/linear iterations, pseudo-time rejections and callback causes",
};
impl BackendExecution for Petsc {
    fn backend(&self) -> Backend {
        Backend::Petsc
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Roots
    }
    fn linked(&self) -> bool {
        cfg!(feature = "petsc")
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn build(&self) -> Option<pse_ids::ContentHash> {
        #[cfg(feature = "petsc")]
        {
            Some(crate::petsc::build())
        }
        #[cfg(not(feature = "petsc"))]
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
        let defaults = crate::settings::petsc::Settings::default();
        let selected = match settings {
            BackendSettings::Default => &defaults,
            BackendSettings::Petsc(s) => s,
            _ => return Err(super::foreign(Backend::Petsc)),
        };
        selected.validate()?;
        if selected.method != crate::settings::petsc::Method::NewtonTrustRegion {
            return Err(ProblemError::Unsupported("PETSc pseudo-time and Schwarz require their declared mathematical composition owner".into()));
        }
        if controls.threads != 1 || !controls.options.is_empty() {
            return Err(ProblemError::Unsupported(
                "PETSc MPIUNI profiles require one thread and typed settings".into(),
            ));
        }
        Ok(())
    }
    fn admit_contract(
        &self,
        contract: &crate::OracleContract,
        _: &std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign>,
        _: &BackendSettings,
        budgets: super::Budgets<'_>,
    ) -> Result<(), ProblemError> {
        if contract
            .variables
            .iter()
            .any(|v| v.lower.is_finite() || v.upper.is_finite())
        {
            return Err(ProblemError::Unsupported(
                "PETSc root profiles do not preserve arbitrary variable bounds".into(),
            ));
        }
        if budgets.accuracy.native_scaling || budgets.accuracy.acceptable.is_some() {
            return Err(ProblemError::Unsupported(
                "PETSc root profiles require resolved original scaling and strict acceptance"
                    .into(),
            ));
        }
        Ok(())
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::Root(primal))
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(payload, WarmPayload::Root(_))
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        #[cfg(feature = "petsc")]
        {
            let super::Problem::Roots {
                mut oracle,
                initial,
                ..
            } = input.problem
            else {
                return Err(super::representation(Backend::Petsc));
            };
            let defaults = crate::settings::petsc::Settings::default();
            let settings = match input.settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Petsc(s) => s,
                _ => return Err(super::foreign(Backend::Petsc)),
            };
            retained.clear();
            crate::petsc::solve(
                oracle.as_mut(),
                initial,
                settings,
                crate::petsc::SolveRequest {
                    controls: input.controls,
                    accuracy: input.accuracy,
                    execution: input.execution,
                    tolerances: input.tolerances,
                    warm: input.warm,
                    compatibility: &input.compatibility,
                },
            )
        }
        #[cfg(not(feature = "petsc"))]
        {
            let _ = (retained, input);
            Err(super::unlinked(Backend::Petsc))
        }
    }
}
