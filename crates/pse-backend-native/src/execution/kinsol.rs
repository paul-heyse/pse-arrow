// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! KINSOL adapter: typed method controls with policy-derived scales, retained SUNDIALS/KLU
//! allocations for compatible layouts, root starts.
use super::{
    BackendExecution, BackendSettings, Budgets, Capability, Input, Representation, Retained,
};
use crate::{
    OracleContract, ProblemError,
    solve::{
        Backend, DerivativeCapability, ProblemClass, SolveReport, WarmCapability, WarmPayload,
    },
};
use pse_ids::SemanticId;
use pse_math::presolve::GuardSign;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct Kinsol;
pub(super) static ADAPTER: Kinsol = Kinsol;
static CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::SquareRoot, ProblemClass::DeclaredFixedPoint],
    derivatives: DerivativeCapability::JacobianOrProduct,
    warm: WarmCapability::Primal,
    general_bounds: false,
    sign_bounds: true,
    parallel: false,
    reuse: "same sparse layout: retained SUNDIALS/KLU allocations",
    cancellation: "evaluation checkpoints; native factorization completes before teardown",
    diagnostics: "native nonlinear/linear iterations, setups, failures, norms and callback timing",
};
/// Policy-derived native settings of this request's method controls.
#[cfg(feature = "kinsol")]
fn settings(
    settings: &BackendSettings,
    budgets: Budgets<'_>,
) -> Result<crate::kinsol::Settings, ProblemError> {
    let method = match settings {
        BackendSettings::Default => crate::kinsol::Method::default(),
        BackendSettings::Kinsol(method) => *method,
        _ => return Err(super::foreign(Backend::Kinsol)),
    };
    Ok(crate::kinsol::Settings::from_policy(
        method,
        budgets.tolerances,
        budgets.normalization,
        budgets.feasibility,
    ))
}
impl BackendExecution for Kinsol {
    fn backend(&self) -> Backend {
        Backend::Kinsol
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Roots
    }
    fn linked(&self) -> bool {
        cfg!(feature = "kinsol")
    }
    fn automatic(&self) -> Option<u8> {
        Some(0)
    }
    fn admit_contract(
        &self,
        contract: &OracleContract,
        guards: &BTreeMap<SemanticId, GuardSign>,
        backend: &BackendSettings,
        budgets: Budgets<'_>,
    ) -> Result<(), ProblemError> {
        #[cfg(feature = "kinsol")]
        {
            settings(backend, budgets)?.validate_contract(
                contract,
                crate::kinsol::Strategy::LineSearch,
                guards,
            )?;
        }
        #[cfg(not(feature = "kinsol"))]
        let _ = (contract, guards, backend, budgets);
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
        let super::Problem::Roots {
            oracle,
            initial,
            budgets,
            owner,
        } = input.problem
        else {
            return Err(super::representation(Backend::Kinsol));
        };
        #[cfg(feature = "kinsol")]
        {
            use crate::kinsol::{Function, Session};
            use std::{any::Any, cell::Cell};
            let settings = settings(input.settings, budgets)?;
            let stamp = input.compatibility;
            let function = Cell::new(Some(Function::Equations(oracle)));
            let owner = Cell::new(owner);
            let take = || {
                function
                    .take()
                    .ok_or_else(|| ProblemError::Internal("KINSOL function consumed".into()))
            };
            // The session retains the function, so the function's owner lives with it.
            let (state, reused) = retained.session(
                Backend::Kinsol,
                input.controls.reuse,
                |(session, held): &mut (Session, Option<Box<dyn Any>>)| {
                    if !(session.matches_layout(&stamp) && session.matches_settings(&settings)) {
                        return Ok(false);
                    }
                    session.replace(take()?, settings.clone(), stamp.clone())?;
                    *held = owner.take();
                    Ok(true)
                },
                || {
                    let session = Session::new(
                        take()?,
                        settings.clone(),
                        input.execution.clone(),
                        stamp.clone(),
                    )?;
                    Ok((session, owner.take()))
                },
            )?;
            let mut report = state.0.solve(
                initial,
                input.controls,
                input.accuracy,
                input.execution,
                input.tolerances,
                input.warm,
            )?;
            report.metrics.insert(
                "reuse.native_model".into(),
                crate::solve::Metric::Bool(reused),
            );
            report.evidence.reused_native_state = reused;
            Ok(report)
        }
        #[cfg(not(feature = "kinsol"))]
        {
            let _ = (retained, oracle, initial, budgets, owner);
            Err(super::unlinked(Backend::Kinsol))
        }
    }
}
