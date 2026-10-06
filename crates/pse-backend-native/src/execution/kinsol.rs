// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! KINSOL adapter: typed method controls with policy-derived scales, retained SUNDIALS/KLU
//! allocations for compatible layouts, root starts. One-sided bounds reach KINSOL as sign
//! constraints on shifted coordinates.
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
    structural: crate::structural::Policy::Roots,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SquareRoot, ProblemClass::DeclaredFixedPoint],
    automatic_classes: &[ProblemClass::SquareRoot, ProblemClass::DeclaredFixedPoint],
    derivatives: DerivativeCapability::JacobianOrProduct,
    warm: WarmCapability::Primal,
    general_bounds: false,
    sign_bounds: true,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: true,
    reuse: "same sparse layout: retained SUNDIALS/KLU allocations",
    cancellation: "evaluation checkpoints; native factorization completes before teardown",
    diagnostics: "native nonlinear/linear iterations, setups, failures, norms, Krylov and preconditioner counters and callback timing",
};
/// Policy-derived native settings of this request's method controls.
#[cfg(feature = "kinsol")]
fn settings(
    settings: &BackendSettings,
    budgets: Budgets<'_>,
) -> Result<crate::kinsol::Settings, ProblemError> {
    let method = match settings {
        BackendSettings::Default => crate::settings::kinsol::Method::default(),
        BackendSettings::Kinsol(method) => *method,
        _ => return Err(super::foreign(Backend::Kinsol)),
    };
    // Scaling belongs to the frozen physical acceptance. Using the attempt's
    // tighter work tolerance here would cancel that tolerance in KINSOL's scaled
    // residual test, leaving the actual physical stopping threshold unchanged.
    let original = budgets.tolerances.normalized(budgets.normalization)?;
    let scaling_tolerance = original
        .variables
        .iter()
        .chain(&original.rows)
        .copied()
        .reduce(f64::min)
        .unwrap_or(budgets.accuracy.feasibility);
    let mut resolved = crate::kinsol::Settings::from_policy(
        method,
        budgets.tolerances,
        budgets.normalization,
        scaling_tolerance,
    );
    resolved.step_tolerance = budgets.accuracy.feasibility;
    Ok(resolved)
}

#[cfg(all(test, feature = "kinsol"))]
mod accuracy_tests {
    use super::*;

    #[test]
    fn engineering_accuracy_kinsol_work_tolerance_does_not_cancel_residual_scaling() {
        let tolerances = crate::quality::Tolerances {
            variables: vec![0.001],
            rows: vec![0.001],
            integrality: 0.001,
        };
        let normalization = pse_math::normalization::Normalization::identity(1, 1);
        let original = crate::solve::ResolvedAccuracy::resolve(
            &Default::default(),
            &tolerances,
            &normalization,
        )
        .unwrap();
        let mut tighter = original.clone();
        tighter.feasibility = 0.00002;
        let settings_for = |accuracy| {
            settings(
                &BackendSettings::Default,
                Budgets {
                    accuracy,
                    tolerances: &tolerances,
                    normalization: &normalization,
                },
            )
            .unwrap()
        };
        let base = settings_for(&original);
        let refined = settings_for(&tighter);
        assert_eq!(base.variable_scales, refined.variable_scales);
        assert_eq!(base.residual_scales, refined.residual_scales);
        // This independently selected physical residual meets ordinary stopping,
        // but must fail the tighter goal-derived request.
        let residual = 0.000225;
        assert!(residual * base.residual_scales[0] <= original.feasibility);
        assert!(residual * refined.residual_scales[0] > tighter.feasibility);
        assert_eq!(refined.step_tolerance, tighter.feasibility);
        assert_eq!(tolerances.rows, vec![0.001]);
    }

    #[test]
    fn engineering_accuracy_kinsol_projects_heterogeneous_original_row_budgets() {
        let tolerances = crate::quality::Tolerances {
            variables: vec![1., 0.001],
            rows: vec![1., 0.001],
            integrality: 0.001,
        };
        let normalization = pse_math::normalization::Normalization::identity(2, 2);
        let accuracy = crate::solve::ResolvedAccuracy::resolve(
            &Default::default(),
            &tolerances,
            &normalization,
        )
        .unwrap();
        let budgets = Budgets {
            accuracy: &accuracy,
            tolerances: &tolerances,
            normalization: &normalization,
        };
        let frozen = settings(&BackendSettings::Default, budgets).unwrap();
        let request = ADAPTER
            .residual_work_tolerance(&BackendSettings::Default, budgets, &[(0, 0.01)])
            .unwrap()
            .unwrap();
        assert!((request - 0.00001).abs() <= 2. * f64::EPSILON * 0.00001);
        assert!(request < accuracy.feasibility);
        assert!((request / frozen.residual_scales[0] - 0.01).abs() <= 2. * f64::EPSILON * 0.01);
        assert_eq!(tolerances.rows, vec![1., 0.001]);
        assert!(
            ADAPTER
                .residual_work_tolerance(
                    &BackendSettings::Default,
                    budgets,
                    &[(0, f64::from_bits(1))]
                )
                .is_err(),
            "positive underflow must not become a zero work tolerance"
        );
    }
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
    fn residual_work_tolerance(
        &self,
        backend: &BackendSettings,
        budgets: Budgets<'_>,
        demands: &[(usize, f64)],
    ) -> Result<Option<f64>, ProblemError> {
        #[cfg(feature = "kinsol")]
        {
            // Construct the frozen scales once for the complete demand group,
            // using exactly the owner consumed by native execution.
            let frozen = settings(backend, budgets)?;
            let mut requested: Option<f64> = None;
            for &(row, allowance) in demands {
                let coordinate = budgets.normalization.rows.get(row).ok_or_else(|| {
                    ProblemError::Contract("KINSOL work demand row is absent".into())
                })?;
                let scale = frozen.residual_scales.get(row).ok_or_else(|| {
                    ProblemError::Contract("KINSOL work demand scale is absent".into())
                })?;
                if !allowance.is_finite() || allowance <= 0. {
                    return Err(ProblemError::Contract(
                        "invalid KINSOL residual work allowance".into(),
                    ));
                }
                let normalized = pse_math::normalization::checked_ratio(allowance, *coordinate)
                    .map_err(|_| {
                        ProblemError::numerical("KINSOL row work tolerance is not representable")
                    })?;
                let value = normalized * scale;
                if !value.is_finite() || value <= 0. {
                    return Err(ProblemError::numerical(
                        "KINSOL row work tolerance is not representable",
                    ));
                }
                requested = Some(requested.map_or(value, |held| held.min(value)));
            }
            Ok(requested)
        }
        #[cfg(not(feature = "kinsol"))]
        {
            let _ = (backend, budgets, demands);
            Err(super::unlinked(Backend::Kinsol))
        }
    }
    fn required_artifact(
        &self,
        r: &crate::routing::Requirements<'_>,
    ) -> Option<crate::routing::ArtifactDemand> {
        let method = match r.settings {
            BackendSettings::Kinsol(method) => *method,
            _ => crate::settings::kinsol::Method::default(),
        };
        if method.consumes_jvp() {
            // An assembled preconditioner consumes both programs. Establish its
            // ordinary derivative order first, then the independent residual action.
            if !method.consumes_directional_only()
                && let Some(order) = self.required_order(r)
                && r.facts.prepared_derivatives < order
            {
                return Some(crate::routing::ArtifactDemand::Derivatives(order));
            }
            Some(crate::routing::ArtifactDemand::JacobianProduct)
        } else {
            self.required_order(r)
                .map(crate::routing::ArtifactDemand::Derivatives)
        }
    }
    fn linked(&self) -> bool {
        cfg!(feature = "kinsol")
    }
    fn work_coverage(&self, execution: &crate::solve::Execution) -> crate::solve::WorkCoverage {
        crate::solve::WorkCoverage {
            evaluations: self.linked()
                && execution.work_admission.is_some()
                && execution.callback_work_owner,
            // SUNDIALS/KLU iterations and factors have no primitive admission
            // observer. Evaluation callbacks, including final validation, do.
            ..Default::default()
        }
    }
    fn automatic(&self) -> Option<u8> {
        Some(0)
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &crate::solve::Controls,
        _: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        if !matches!(
            settings,
            BackendSettings::Default | BackendSettings::Kinsol(_)
        ) {
            return Err(super::foreign(Backend::Kinsol));
        }
        if controls.threads != 1 || !controls.options.is_empty() {
            return Err(ProblemError::Unsupported(
                "serial KINSOL uses typed Settings and one core".into(),
            ));
        }
        Ok(())
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
