// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Profile evidence preserves execution failures separately from scientific intervals.
use super::{FitReport, ProfileChain};
use pse_model::generated::enums::WithheldReason;

/// Decoded fit preparation. Native fitting resolves its selected model and experiments.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitPreparationDocument {
    /// Canonical solve settings; their own owner admits cross-field rules.
    pub solver: crate::math::settings::SolveSettings,
    /// Native dynamic settings by experiment identity.
    #[serde(default)]
    pub simulations: std::collections::BTreeMap<
        pse_model::generated::identities::InstanceId,
        crate::workflow::SimulationProfile,
    >,
    #[serde(default = "FitPreparationDocument::default_rank_tolerance")]
    #[doc = "Rank tolerance for fit admission."]
    pub rank_tolerance: f64,
    #[serde(default = "FitPreparationDocument::default_max_cells")]
    #[doc = "Maximum native fit workspace cells."]
    pub max_cells: usize,
    #[serde(default = "FitPreparationDocument::default_derivatives")]
    #[doc = "Requested derivative evidence policy."]
    pub derivatives: super::FitDerivatives,
    #[serde(default)]
    #[doc = "Optional fit uncertainty request."]
    pub uncertainty: Option<super::FitUncertainty>,
}
impl FitPreparationDocument {
    const fn default_rank_tolerance() -> f64 {
        1e-8
    }
    const fn default_max_cells() -> usize {
        1_000_000
    }
    const fn default_derivatives() -> super::FitDerivatives {
        super::FitDerivatives::Responses
    }
    /// Resolve solver policy; contextual experiment admission stays in `prepare_fit`.
    pub fn profile(self) -> Result<super::FitProfile, crate::math::MathRuntimeError> {
        Ok(super::FitProfile {
            solver: self.solver.profile()?,
            simulations: self.simulations,
            rank_tolerance: self.rank_tolerance,
            max_cells: self.max_cells,
            derivatives: self.derivatives,
            uncertainty: self.uncertainty,
        })
    }
}
/// Whether profile intervals were requested, available, or withheld by native admission.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum FitProfileDocument {
    /// No profile request was made.
    NotRequested,
    /// Every chain, including worker failures and its actual parallelism.
    Available {
        #[doc = "Every retained chain, including distinct worker failures."]
        chains: Vec<ProfileChain>,
    },
    /// The native reason and retained diagnostic explanation.
    Withheld {
        #[doc = "Canonical native reason for withholding."]
        reason: WithheldReason,
        #[doc = "Retained explanation of the refusal."]
        detail: String,
    },
}
impl FitReport {
    /// Project retained evidence without starting work or revising its outcome.
    pub fn profile_document(&self) -> FitProfileDocument {
        match &self.profiles {
            None => FitProfileDocument::NotRequested,
            Some(Ok(chains)) => FitProfileDocument::Available {
                chains: chains.clone(),
            },
            Some(Err(reason)) => FitProfileDocument::Withheld {
                reason: reason.reason(),
                detail: reason.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    use crate::workflow::{IntervalBound, ProfileWorkerFailure};
    use pse_model::generated::enums::{IntervalEnd, IntervalOutcome};
    #[test]
    fn fit_preparation_defaults_are_owned_and_resolved_in_rust() {
        let omitted: FitPreparationDocument =
            serde_json::from_str(r#"{"solver":{"version":3}}"#).unwrap();
        let explicit: FitPreparationDocument = serde_json::from_str(r#"{"solver":{"version":3},"simulations":{},"rank_tolerance":1e-8,"max_cells":1000000,"derivatives":"responses","uncertainty":null}"#).unwrap();
        let omitted = omitted.profile().unwrap();
        let explicit = explicit.profile().unwrap();
        assert_eq!(omitted.rank_tolerance, explicit.rank_tolerance);
        assert_eq!(omitted.max_cells, explicit.max_cells);
        assert_eq!(omitted.derivatives, explicit.derivatives);
        assert!(omitted.simulations.is_empty());
    }
    #[test]
    fn profile_transport_preserves_worker_failure_and_scientific_state() {
        let document = FitProfileDocument::Available {
            chains: vec![ProfileChain {
                scheduling_failures: Vec::new(),
                worker_failure: Some(ProfileWorkerFailure::Scheduling("pool unavailable".into())),
                actual_parallelism: 0,
                parameter: pse_ids::SemanticId::from_bytes([7; 16]),
                end: IntervalEnd::Lower,
                estimate: 2.,
                bound: IntervalBound {
                    value: None,
                    outcome: IntervalOutcome::Stopped,
                },
                detail: Some("worker did not begin".into()),
                points: vec![],
            }],
        };
        let value = serde_json::to_value(document).unwrap();
        assert_eq!(value["status"], "available");
        assert_eq!(value["chains"][0]["worker_failure"]["kind"], "scheduling");
        assert_eq!(
            value["chains"][0]["worker_failure"]["detail"],
            "pool unavailable"
        );
        assert_eq!(value["chains"][0]["actual_parallelism"], 0);
        assert_eq!(value["chains"][0]["bound"]["outcome"], "stopped");
        assert!(value["chains"][0]["bound"]["value"].is_null());
        assert_eq!(
            serde_json::to_value(FitProfileDocument::NotRequested).unwrap()["status"],
            "not_requested"
        );
    }
    #[test]
    fn profile_transport_preserves_typed_trial_terminal_and_environment_causes() {
        use super::super::{ProfileFailure, ProfilePoint};
        use pse_backend_native::{ProblemError, solve::Termination};
        let failure = |cause| ProfileFailure::from(cause);
        let chain = ProfileChain {
            scheduling_failures: Vec::new(),
            worker_failure: Some(ProfileWorkerFailure::Environment(failure(
                ProblemError::memory("worker native scope"),
            ))),
            actual_parallelism: 0,
            parameter: pse_ids::SemanticId::from_bytes([7; 16]),
            end: IntervalEnd::Lower,
            estimate: 2.,
            bound: IntervalBound {
                value: None,
                outcome: IntervalOutcome::Stopped,
            },
            detail: Some("stopped after terminal failure".into()),
            points: vec![
                ProfilePoint {
                    value: 1.,
                    seed: None,
                    qualification: None,
                    termination: None,
                    failures: vec![failure(ProblemError::numerical("trial trajectory"))],
                    callback_terminal_failure: false,
                    objective: None,
                    statistic: None,
                    accepted: false,
                    detail: Some("trial trajectory".into()),
                },
                ProfilePoint {
                    value: 1.5,
                    seed: None,
                    qualification: None,
                    termination: Some(Termination::Evaluation),
                    failures: vec![failure(ProblemError::Provider(
                        pse_kernels::ProviderError::Terminal("provider stopped".into()),
                    ))],
                    callback_terminal_failure: true,
                    objective: None,
                    statistic: None,
                    accepted: false,
                    detail: Some("provider stopped".into()),
                },
            ],
        };
        let value = serde_json::to_value(FitProfileDocument::Available {
            chains: vec![chain],
        })
        .unwrap();
        let chain = &value["chains"][0];
        assert_eq!(chain["worker_failure"]["kind"], "environment");
        assert_eq!(chain["worker_failure"]["detail"]["class"], "resource_limit");
        assert_eq!(chain["points"][0]["failures"][0]["class"], "numerical");
        assert_eq!(chain["points"][1]["failures"][0]["class"], "infrastructure");
        assert_eq!(chain["points"][1]["termination"], "evaluation");
        assert_eq!(chain["points"][1]["callback_terminal_failure"], true);
        assert_eq!(chain["bound"]["outcome"], "stopped");
    }
}
