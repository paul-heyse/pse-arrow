// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! HiGHS adapter: typed method, opt-in native diagnostics and partial MIP starts over a
//! retained native model updated in place for compatible layouts.
use super::{BackendExecution, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{
        Backend, DerivativeCapability, ProblemClass, SolveReport, WarmCapability, WarmPayload,
    },
};

#[derive(Debug)]
pub(super) struct Highs;
pub(super) static ADAPTER: Highs = Highs;
static CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::NativeFeasibility,
    lexicographic_degradation: crate::routing::DegradationSupport::SingleNonzero,
    classes: &[
        ProblemClass::Linear,
        ProblemClass::MixedLinear,
        ProblemClass::ConvexQuadratic,
    ],
    automatic_classes: &[
        ProblemClass::Linear,
        ProblemClass::MixedLinear,
        ProblemClass::ConvexQuadratic,
    ],
    derivatives: DerivativeCapability::Coefficients,
    warm: WarmCapability::PrimalDualAndBasis,
    general_bounds: true,
    sign_bounds: true,
    parallel: true,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[ProblemClass::Linear, ProblemClass::MixedLinear],
    batch: false,
    sensitivities: false,
    reuse: "native coefficient/bound updates with compatible layout",
    // PDLP never polls the interrupt callback (HiGHS `pdlp/*Wrapper.cpp`), so, like the
    // QP solver, it stops only at the native time limit (F08c).
    cancellation: "simplex/IPM/MIP interrupt callbacks; QP and PDLP native time limit only",
    diagnostics: "native information, rays, IIS, ranging and explicit relaxation",
};
impl BackendExecution for Highs {
    fn backend(&self) -> Backend {
        Backend::Highs
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Coefficients
    }
    fn linked(&self) -> bool {
        cfg!(feature = "highs")
    }
    fn automatic(&self) -> Option<u8> {
        Some(1)
    }
    fn admit_settings(
        &self,
        settings: &super::BackendSettings,
        controls: &crate::solve::Controls,
        _: &super::Snapshot,
    ) -> Result<(), ProblemError> {
        let defaults = crate::settings::highs::Settings::default();
        let settings = match settings {
            super::BackendSettings::Default => &defaults,
            super::BackendSettings::Highs(settings) => settings,
            _ => return Err(super::foreign(Backend::Highs)),
        };
        settings.admit_controls(controls)
    }
    fn assess_representation(
        &self,
        requirements: &crate::routing::Requirements<'_>,
        assessment: &mut crate::routing::Eligibility,
    ) {
        super::assess_representation(self, requirements, assessment);
        if let Some(problem) = requirements.context.coefficients {
            let defaults = crate::settings::highs::Settings::default();
            let settings = match requirements.settings {
                super::BackendSettings::Default => Some(&defaults),
                super::BackendSettings::Highs(settings) => Some(settings),
                _ => None,
            };
            if let Some(settings) = settings {
                if let Some(budgets) = requirements.context.budgets {
                    if let Err(cause) = settings.admit_model(problem, budgets.accuracy) {
                        assessment.refuse(cause);
                    }
                } else {
                    assessment
                        .evidence
                        .push(crate::routing::EvidenceDemand::CallbackContract);
                }
            }
        }
        #[cfg(feature = "highs")]
        if let Some(problem) = requirements.context.coefficients
            && !assessment
                .evidence
                .iter()
                .any(|demand| matches!(demand, crate::routing::EvidenceDemand::Class(_)))
            && let Err(cause) = crate::highs::admit_in_coordinates(
                problem,
                requirements.context.certificate,
                requirements
                    .context
                    .budgets
                    .map(|budgets| budgets.normalization),
            )
        {
            assessment.refuse(cause);
        }
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::Highs {
            primal: Some(primal),
            dual: None,
            basis: None,
        })
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(payload, WarmPayload::Highs { .. })
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        let super::Problem::Coefficients {
            problem,
            certificate,
            normalization,
            row_constants,
        } = input.problem
        else {
            return Err(super::representation(Backend::Highs));
        };
        #[cfg(feature = "highs")]
        {
            use super::BackendSettings;
            use crate::{highs, solve::Metric, transport};
            use std::collections::BTreeMap;
            let defaults = highs::Settings::default();
            let settings = match input.settings {
                BackendSettings::Default => &defaults,
                BackendSettings::Highs(settings) => settings,
                _ => return Err(super::foreign(Backend::Highs)),
            };
            let stamp = input.compatibility;
            let (session, reused) = retained.session(
                Backend::Highs,
                input.controls.reuse,
                |session: &mut highs::Session| {
                    Ok(session.update(problem, certificate, stamp.clone()).is_ok())
                },
                || highs::Session::new(problem, certificate, stamp.clone(), &input.execution),
            )?;
            if let Some(start) = &settings.sparse_start {
                let start = start
                    .iter()
                    .map(|(id, v)| {
                        let i = problem
                            .contract
                            .variables
                            .iter()
                            .position(|c| c.id == *id)
                            .ok_or_else(|| {
                                ProblemError::Contract("unknown sparse start coordinate".into())
                            })?;
                        Ok((
                            *id,
                            pse_math::normalization::checked_ratio(*v, normalization.variables[i])?,
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, ProblemError>>()?;
                session.sparse_start(problem, &start)?;
            }
            let mut report = session.solve(
                problem,
                normalization,
                input.controls,
                input.accuracy,
                settings,
                input.execution.clone(),
                input.tolerances,
                input.warm,
            )?;
            let requested = &settings.diagnostics;
            if requested.any() {
                let request = transport::diagnostic_request(requested, normalization)?;
                let mut diagnostics = session
                    .diagnose(problem, &request, &input.execution)
                    .unwrap_or_else(|e| highs::diagnostics::Report {
                        unavailable: BTreeMap::from([("operation".into(), e.to_string())]),
                        ..Default::default()
                    });
                // The fixed-commitment LP prices the MIP candidate, conditional on its
                // commitment (ADR-0118 item 9); both are still native coordinates.
                if let Some(fixed) = &diagnostics.fixed_lp {
                    let priced = report.candidate.as_mut().map_or_else(
                        || Err("no MIP candidate to price".to_owned()),
                        |candidate| fixed.price(candidate, input.accuracy),
                    );
                    if let Err(reason) = priced {
                        diagnostics
                            .unavailable
                            .insert("fixed_lp.candidate".into(), reason);
                    }
                }
                // An exported ray is the typed certificate of an infeasible or unbounded LP,
                // in native coordinates like every adapter's; the runner verifies it.
                report.certificate = crate::certificate::from_highs_rays(
                    problem,
                    report.termination.category,
                    diagnostics.dual_ray.as_deref(),
                    diagnostics.primal_ray.as_deref(),
                );
                transport::recover_diagnostics(
                    &mut diagnostics,
                    normalization,
                    row_constants,
                    &problem.contract,
                )?;
                report.highs_diagnostics = Some(Box::new(diagnostics));
            }
            report
                .metrics
                .insert("reuse.native_model".into(), Metric::Bool(reused));
            report.evidence.reused_native_state = reused;
            Ok(report)
        }
        #[cfg(not(feature = "highs"))]
        {
            let _ = (retained, problem, certificate, normalization, row_constants);
            Err(super::unlinked(Backend::Highs))
        }
    }
}
