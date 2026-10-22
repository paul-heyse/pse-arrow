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
    structural: crate::structural::Policy::Equalities,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SmoothNlp],
    automatic_classes: &[ProblemClass::SmoothNlp],
    derivatives: DerivativeCapability::ExactHessianOrLimitedMemory,
    warm: WarmCapability::PrimalDualAndWorkingSet,
    general_bounds: true,
    sign_bounds: true,
    parallel: true,
    certifies: false,
    native_forms: &[],
    // The l1 exact penalty an authored `penalty(l1)` realization states, with the
    // `L1ExactPenalty` method (ADR-0104 §5).
    requirements: &[pse_model::generated::enums::ModelingStructuralRequirement::L1ExactPenalty],
    lexicographic: &[],
    batch: false,
    sensitivities: true,
    reuse: "compatible native application within a sequence; neutral role-specific FERAL allocation/symbolics with application-policy reset; explicit starts only",
    cancellation: "TNLP intermediate/evaluation checkpoints",
    diagnostics: "complete SolveStatistics, phase timing, FERAL inertia/pivots/fill, restoration and crossover",
};
#[cfg(feature = "pounce")]
fn recovery_settings(
    controls: &crate::solve::Controls,
    settings: &BackendSettings,
) -> Result<crate::settings::pounce::Settings, ProblemError> {
    let settings = match settings {
        BackendSettings::Default => crate::settings::pounce::Settings::default(),
        BackendSettings::Pounce(settings) => settings.clone(),
        _ => return Err(super::foreign(Backend::Pounce)),
    };
    controls.validate()?;
    settings.restart.validate()?;
    settings.curvature_options(controls.hessian)?;
    Ok(settings)
}
impl BackendExecution for Pounce {
    fn backend(&self) -> Backend {
        Backend::Pounce
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn recovery_capacity(
        &self,
        controls: &crate::solve::Controls,
        settings: &BackendSettings,
        permission: super::RecoveryPermission,
    ) -> Result<usize, ProblemError> {
        #[cfg(feature = "pounce")]
        {
            let settings = recovery_settings(controls, settings)?;
            Ok(crate::pounce::second_opinion_capacity(
                controls,
                &settings,
                permission.replacement_start,
            ))
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = (controls, settings, permission);
            Err(super::unlinked(Backend::Pounce))
        }
    }
    fn recovery_profiles(
        &self,
        controls: &crate::solve::Controls,
        settings: &BackendSettings,
        report: &SolveReport,
        permission: super::RecoveryPermission,
    ) -> Result<Vec<super::RecoveryProfile>, ProblemError> {
        #[cfg(feature = "pounce")]
        {
            let settings = recovery_settings(controls, settings)?;
            if report.backend != Backend::Pounce {
                return Err(ProblemError::Contract(
                    "recovery observation belongs to another backend".into(),
                ));
            }
            crate::pounce::second_opinion_profiles(
                controls,
                &settings,
                report,
                permission.replacement_start,
            )
            .map(|profiles| {
                profiles
                    .into_iter()
                    .map(|profile| super::RecoveryProfile {
                        label: profile.label,
                        controls: profile.controls,
                        settings: BackendSettings::Pounce(profile.settings),
                        replaces_start: profile.replaces_start,
                        perturbation: profile.perturbation,
                    })
                    .collect()
            })
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = (controls, settings, report, permission);
            Err(super::unlinked(Backend::Pounce))
        }
    }
    fn consumes_separator(&self, _settings: &BackendSettings) -> bool {
        true
    }
    fn representation(&self) -> Representation {
        Representation::Nlp
    }
    fn linked(&self) -> bool {
        cfg!(feature = "pounce")
    }
    fn work_coverage(&self, execution: &crate::solve::Execution) -> crate::solve::WorkCoverage {
        crate::solve::WorkCoverage {
            evaluations: self.linked()
                && execution.work_admission.is_some()
                && execution.callback_work_owner,
            ..Default::default()
        }
    }
    fn automatic(&self) -> Option<u8> {
        Some(3)
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &crate::solve::Controls,
        _: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        match settings {
            BackendSettings::Default => {
                let settings = crate::settings::pounce::Settings::default();
                settings.restart.validate()?;
                settings.curvature_options(controls.hessian).map(|_| ())
            }
            BackendSettings::Pounce(settings) => {
                settings.restart.validate()?;
                settings.curvature_options(controls.hessian).map(|_| ())
            }
            _ => Err(super::foreign(Backend::Pounce)),
        }
    }
    fn assess_representation(
        &self,
        requirements: &crate::routing::Requirements<'_>,
        assessment: &mut crate::routing::Eligibility,
    ) {
        super::assess_representation(self, requirements, assessment);
        let context = &requirements.context;
        let (Some(contract), Some(structure), Some(budgets)) =
            (context.oracle, context.structure.as_ref(), context.budgets)
        else {
            assessment
                .evidence
                .push(crate::routing::EvidenceDemand::CallbackContract);
            return;
        };
        let check = || -> Result<(), ProblemError> {
            budgets
                .normalization
                .validate(contract.variables.len(), contract.rows.len())?;
            for (variable, scale) in contract
                .variables
                .iter()
                .zip(&budgets.normalization.variables)
            {
                for value in [variable.lower, variable.upper] {
                    crate::settings::pounce::admit_bound(crate::transport::bound(value, *scale)?)?;
                }
            }
            for (id, scale) in contract.rows.iter().zip(&budgets.normalization.rows) {
                let row = structure
                    .equations
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| {
                        ProblemError::Contract("POUNCE original row bounds missing".into())
                    })?;
                for value in [
                    row.lower.unwrap_or(f64::NEG_INFINITY),
                    row.upper.unwrap_or(f64::INFINITY),
                ] {
                    crate::settings::pounce::admit_bound(crate::transport::bound(value, *scale)?)?;
                }
            }
            Ok(())
        };
        if let Err(cause) = check() {
            assessment.refuse(cause);
        }
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::primal(primal))
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(payload, WarmPayload::Nlp { .. })
    }
    fn scope_threads(&self, threads: usize) -> usize {
        if cfg!(feature = "pounce") && threads > 1 {
            threads
        } else {
            0
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
                |_: &mut crate::pounce::Session| Ok(super::SessionUpdate::Compatible),
                || Ok(crate::pounce::Session::new()),
            )?;
            session.solve(
                oracle,
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

#[cfg(all(test, feature = "pounce"))]
mod recovery_capacity_tests {
    use super::*;

    #[test]
    fn recovery_capacity_consumes_admitted_settings_and_explicit_start_permission() {
        let controls = crate::solve::Controls::default();
        let settings = crate::settings::pounce::Settings::default();
        for replacement_start in [false, true] {
            let permission = crate::execution::RecoveryPermission { replacement_start };
            let expected =
                crate::pounce::second_opinion_capacity(&controls, &settings, replacement_start);
            assert_eq!(
                ADAPTER
                    .recovery_capacity(
                        &controls,
                        &BackendSettings::Pounce(settings.clone()),
                        permission
                    )
                    .unwrap(),
                expected
            );
            assert_eq!(
                ADAPTER
                    .recovery_capacity(&controls, &BackendSettings::Default, permission)
                    .unwrap(),
                expected
            );
        }
        let permission = crate::execution::RecoveryPermission::default();
        assert!(
            ADAPTER
                .recovery_capacity(
                    &controls,
                    &BackendSettings::Highs(crate::settings::highs::Settings::default()),
                    permission
                )
                .is_err()
        );
        let mut invalid = controls.clone();
        invalid.threads = 0;
        assert!(
            ADAPTER
                .recovery_capacity(&invalid, &BackendSettings::Default, permission)
                .is_err()
        );
        assert_eq!(
            crate::execution::adapter(Backend::Highs)
                .recovery_capacity(&controls, &BackendSettings::Default, permission)
                .unwrap(),
            0
        );
    }
}
