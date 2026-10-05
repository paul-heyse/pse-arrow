// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The solve settings document (ADR-0116 Outcomes 6 and 7; Plan 22 X13): the typed,
//! versioned form of a [`SolverProfile`] that Python constructs as a generated msgspec type
//! and a durable job carries. Every field is a pse-owned typed value; decoding refuses an
//! unknown version, field or vocabulary member and every single-value domain violation, and
//! [`SolveSettings::profile`] applies the rules that relate fields.
use super::{MathRuntimeError, solves::SolverProfile};
use pse_backend_native::{
    ProblemError,
    execution::BackendSettings,
    presolve::{Pass, Policy, PolicyKind},
    solve::{Backend, Controls, Options, SolveIntent, SolverSelection},
};
use pse_math::convexity::ConvexityPolicy;
use pse_model::{document::Version, numerics::NumericalPolicy, scalars::FiniteBound};
use std::collections::BTreeSet;

/// The settings of one solve. The version is required; every other field takes the
/// default of [`SolverProfile::default`] when absent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SolveSettings {
    /// Document version.
    pub version: Version<3>,
    /// Requested composition; omission chooses automatic resolution without replacement starts.
    #[serde(default)]
    pub composition: pse_model::strategy::CompositionRequest,
    /// Explicit reconstruction evidence class and finite refinement allowances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconstruction: Option<pse_backend_native::derived::ReconstructionAccuracy>,
    /// Mathematical purpose.
    #[serde(default = "defaults::intent")]
    pub intent: SolveIntent,
    /// An explicitly selected eligible backend; absent is deterministic routing.
    #[serde(default)]
    pub backend: Option<Backend>,
    /// Typed settings of one backend; absent is the routed backend's native defaults.
    #[serde(default)]
    pub settings: Option<BackendSettings>,
    /// Library presolve policy.
    #[serde(default = "defaults::presolve")]
    pub presolve: PolicyKind,
    /// Native presolve options; an explicit policy only.
    #[serde(default)]
    pub presolve_options: Options,
    /// Passes that may not silently become unavailable; an explicit policy only.
    #[serde(default)]
    pub required_passes: BTreeSet<Pass>,
    /// Shared finite attempt controls.
    #[serde(default = "defaults::controls")]
    pub controls: Controls,
    /// ID-keyed numerical requirements and acceptance budgets.
    #[serde(default)]
    pub numerics: NumericalPolicy,
    /// Absolute eigenvalue budget of a numerical convexity assessment; with the relative
    /// budget, it permits numerical assessment, and absent both, only exact certification.
    #[serde(default)]
    pub convexity_absolute: Option<f64>,
    /// Relative eigenvalue budget of a numerical convexity assessment.
    #[serde(default)]
    pub convexity_relative: Option<f64>,
    /// Physical parameter sensitivities at a regular square Root or optimizing candidate;
    /// absent computes none. An absent request is not encoded, so a document without
    /// one keeps its identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<SensitivityRequest>,
}

/// Physical parameter sensitivities at a regular square Root or an optimization's local
/// solution (ADR-0144; ADR-0118). Roots use qualified equality Jacobian response;
/// optimization uses KKT-point analysis. Results use original physical coordinates and
/// units, with certified validity or the condition that withheld them.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SensitivityRequest {
    /// Declared parameters of the solved case by identity, in the order results report
    /// them; at least one, none repeated.
    pub parameters: Vec<pse_ids::SemanticId>,
    /// Also compute the reduced Hessian over the parameters: the second derivative of the
    /// optimal value.
    #[serde(default)]
    pub reduced_hessian: bool,
    /// Also propagate a fit's parameter covariance through the sensitivities to named
    /// variables (Plan 22 S4); absent propagates none and is not encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub propagation: Option<Propagation>,
}
/// Propagation of a parameter covariance to solved variables, `Σ_y = J·Σ_θ·Jᵀ` with `J` the
/// step's parametric sensitivities (Plan 22 S4; ADR-0118 items 1 and 11). The result holds
/// while the sensitivities do: its validity is theirs, and the covariance's, which a fit
/// publishes only when certified.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Propagation {
    /// The covariance to propagate, as a fit run published it.
    pub covariance: ParameterCovariance,
    /// Solved variables of the case by identity, in the order of the result; at least one,
    /// none repeated.
    pub outputs: Vec<pse_ids::SemanticId>,
}
/// A parameter covariance as `runtime.parameter_covariances` publishes it.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParameterCovariance {
    /// The fit run that derived it.
    pub run_id: pse_model::generated::identities::RunId,
    /// The fitted parameters by identity, in the order of `values`.
    pub parameters: Vec<pse_ids::SemanticId>,
    /// `Σ_θ` row-major, in the parameters' units.
    pub values: Vec<FiniteBound>,
}
impl ParameterCovariance {
    /// Distinct parameters and a square, symmetric `values` over them with a nonnegative
    /// diagonal.
    ///
    /// # Errors
    /// A contract error naming the violated rule.
    pub fn admit(&self) -> Result<(), ProblemError> {
        let n = self.parameters.len();
        let distinct: BTreeSet<_> = self.parameters.iter().collect();
        let value = |i: usize, j: usize| self.values[i * n + j].into_inner();
        let scale = self
            .values
            .iter()
            .fold(0.0_f64, |m, v| m.max(v.into_inner().abs()));
        if n == 0
            || distinct.len() != n
            || self.values.len() != n * n
            || (0..n).any(|i| {
                value(i, i) < 0.0
                    || (0..i).any(|j| (value(i, j) - value(j, i)).abs() > 1e-12 * scale)
            })
        {
            return Err(ProblemError::Contract(
                "a parameter covariance names distinct parameters and a symmetric square matrix over them with a nonnegative diagonal".into(),
            ));
        }
        Ok(())
    }
}
impl SensitivityRequest {
    /// Admit the request for Root or Optimize: distinct parameters, optimization-only
    /// and a propagation over a valid covariance of requested parameters to distinct
    /// outputs.
    ///
    /// # Errors
    /// A contract error naming the violated rule.
    pub fn admit(&self, intent: SolveIntent) -> Result<(), ProblemError> {
        if !matches!(intent, SolveIntent::Optimize | SolveIntent::Root) {
            return Err(ProblemError::Contract(format!(
                "parametric sensitivities differentiate an optimum, not a {} solve",
                intent.as_str()
            )));
        }
        if intent == SolveIntent::Root && (self.reduced_hessian || self.propagation.is_some()) {
            return Err(ProblemError::Contract("a root response supports parameter sensitivities; reduced Hessians and covariance propagation require optimization".into()));
        }
        let distinct: BTreeSet<_> = self.parameters.iter().collect();
        if self.parameters.is_empty() || distinct.len() != self.parameters.len() {
            return Err(ProblemError::Contract(
                "a sensitivity request names at least one parameter, none repeated".into(),
            ));
        }
        if let Some(propagation) = &self.propagation {
            propagation.covariance.admit()?;
            let outputs: BTreeSet<_> = propagation.outputs.iter().collect();
            if propagation
                .covariance
                .parameters
                .iter()
                .any(|p| !distinct.contains(p))
                || outputs.is_empty()
                || outputs.len() != propagation.outputs.len()
            {
                return Err(ProblemError::Contract(
                    "a propagation's covariance parameters are requested parameters, and its outputs are at least one, none repeated".into(),
                ));
            }
        }
        Ok(())
    }
}

mod defaults {
    use super::{PolicyKind, SolveIntent, SolverProfile};

    pub(super) fn controls() -> super::Controls {
        SolverProfile::default().controls
    }
    pub(super) fn intent() -> SolveIntent {
        SolverProfile::default().intent
    }
    pub(super) fn presolve() -> PolicyKind {
        SolverProfile::default().presolve.kind()
    }
}

impl Default for SolveSettings {
    /// The document of [`SolverProfile::default`].
    fn default() -> Self {
        Self {
            version: Version,
            composition: Default::default(),
            reconstruction: None,
            intent: defaults::intent(),
            backend: None,
            settings: None,
            presolve: defaults::presolve(),
            presolve_options: Options::new(),
            required_passes: BTreeSet::new(),
            controls: defaults::controls(),
            numerics: NumericalPolicy::default(),
            convexity_absolute: None,
            convexity_relative: None,
            sensitivity: None,
        }
    }
}

impl SolveSettings {
    /// The solver profile these settings state, after the rules that relate fields:
    /// presolve options and required passes only with an explicit policy, finite controls,
    /// independent acceptance budgets, both or neither convexity budget, and a sensitivity
    /// request for optimization KKT sensitivity or regular-square Root response.
    ///
    /// # Errors
    /// A typed refusal naming the violated rule.
    pub fn profile(self) -> Result<SolverProfile, MathRuntimeError> {
        let presolve = Policy::new(self.presolve, &self.presolve_options, self.required_passes)?;
        self.controls.validate()?;
        if let Some(accuracy) = self.reconstruction {
            accuracy.validate()?;
        }
        self.composition
            .validate()
            .map_err(|error| ProblemError::Contract(error.to_string()))?;
        self.numerics
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let convexity =
            ConvexityPolicy::from_tolerances(self.convexity_absolute, self.convexity_relative)?;
        if let Some(request) = &self.sensitivity {
            request.admit(self.intent)?;
        }
        Ok(SolverProfile {
            presolve,
            numerics: self.numerics,
            convexity,
            intent: self.intent,
            selection: self
                .backend
                .map_or(SolverSelection::Auto, SolverSelection::Explicit),
            controls: self.controls,
            backend: BackendSettings::from_document(self.settings),
            sensitivity: self.sensitivity,
            composition: self.composition,
            reconstruction: self.reconstruction,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn current_request_admits_version_before_body_and_preserves_start_grants() {
        let old = br#"{"settings":{"backend":"retired"},"version":1}"#;
        let error = pse_model::document::decode_versioned::<SolveSettings, 3>(old).unwrap_err();
        assert!(error.to_string().contains("explicit readmission"));
        let current =
            pse_model::document::decode_versioned::<SolveSettings, 3>(br#"{"version":3}"#).unwrap();
        assert_eq!(
            current.composition.policy,
            pse_model::strategy::CompositionPolicy::Auto
        );
        assert!(current.composition.recovery.is_empty());
        assert!(current.reconstruction.is_none());
        assert_eq!(
            current.controls.hessian,
            pse_backend_native::solve::HessianMode::Auto
        );
        let mut constrained = current.clone();
        constrained.composition.limits = Some(pse_model::strategy::WorkLimits {
            attempts: 1,
            evaluations: Some(0),
            iterations: None,
            factorizations: None,
            proof_steps: None,
        });
        assert_ne!(
            super::super::solves::profile_key(&current.profile().unwrap()).unwrap(),
            super::super::solves::profile_key(&constrained.profile().unwrap()).unwrap()
        );
    }

    /// Registry vocabularies are the document's enum types: every enumeration decodes from
    /// its registry spelling and a misspelled member is refused at decoding (Plan 22 B4, B5).
    #[test]
    fn solve_settings_enum_types() {
        let settings: SolveSettings = serde_json::from_value(json!({
            "version": 3,
            "intent": "root",
            "backend": "kinsol",
            "presolve": "explicit",
            "required_passes": ["fbbt"],
            "controls": {"hessian": "limited_memory", "reuse": "require_reuse", "start": "previous_accepted"},
            "settings": {"backend": "kinsol", "strategy": "newton"},
        }))
        .unwrap();
        assert_eq!(settings.intent, SolveIntent::Root);
        assert_eq!(settings.backend, Some(Backend::Kinsol));
        assert_eq!(settings.presolve, PolicyKind::Explicit);
        assert_eq!(settings.required_passes, BTreeSet::from([Pass::Fbbt]));
        for misspelled in [
            json!({"version": 3, "intent": "rooot"}),
            json!({"version": 3, "presolve": "magic"}),
            json!({"version": 3, "controls": {"reuse": "sometimes"}}),
            json!({"version": 3, "settings": {"backend": "gurobi"}}),
            json!({"version": 1}),
            json!({"intent": "root"}),
        ] {
            assert!(
                serde_json::from_value::<SolveSettings>(misspelled.clone()).is_err(),
                "{misspelled}"
            );
        }
        // The defaults are the default profile's, and a default document round-trips.
        let default = SolveSettings::default();
        let text = serde_json::to_string(&default).unwrap();
        let back: SolveSettings = serde_json::from_str(&text).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), text);
        let profile = back.profile().unwrap();
        assert_eq!(profile.intent, SolverProfile::default().intent);
        assert_eq!(profile.controls, SolverProfile::default().controls);
    }

    /// A sensitivity request is a typed field of the settings document (Plan 22 S1): it
    /// decodes with the parameter identities it names, enters the profile and its request
    /// identity, admits Root parameter responses, and refuses Root KKT-only quantities, repeated parameters or
    /// an unknown field.
    #[test]
    fn sensitivity_request_in_solve_settings() {
        let (a, b) = (
            pse_ids::SemanticId::from_bytes([1; 16]),
            pse_ids::SemanticId::from_bytes([2; 16]),
        );
        let settings: SolveSettings = serde_json::from_value(json!({
            "version": 3,
            "sensitivity": {"parameters": [a, b], "reduced_hessian": true},
        }))
        .unwrap();
        let request = SensitivityRequest {
            parameters: vec![a, b],
            reduced_hessian: true,
            propagation: None,
        };
        assert_eq!(settings.sensitivity, Some(request.clone()));
        let profile = settings.profile().unwrap();
        assert_eq!(profile.sensitivity, Some(request));
        let plain = SolveSettings::default().profile().unwrap();
        assert_ne!(
            super::super::solves::profile_key(&profile).unwrap(),
            super::super::solves::profile_key(&plain).unwrap()
        );
        let root: SolveSettings = serde_json::from_value(
            json!({"version":3,"intent":"root","sensitivity":{"parameters":[a]}}),
        )
        .unwrap();
        assert!(root.profile().is_ok());
        for refused in [
            json!({"version": 3, "intent": "root", "sensitivity": {"parameters": [a], "reduced_hessian": true}}),
            json!({"version": 3, "sensitivity": {"parameters": [a, a]}}),
            json!({"version": 3, "sensitivity": {"parameters": []}}),
        ] {
            let settings: SolveSettings = serde_json::from_value(refused.clone()).unwrap();
            assert!(settings.profile().is_err(), "{refused}");
        }
        assert!(
            serde_json::from_value::<SolveSettings>(json!({
                "version": 3,
                "sensitivity": {"parameters": [a], "gradient": true},
            }))
            .is_err()
        );
        // A propagation (Plan 22 S4) names a covariance over requested parameters and at
        // least one output; it enters the request identity.
        let x = pse_ids::SemanticId::from_bytes([3; 16]);
        let run = pse_ids::SemanticId::from_bytes([4; 16]);
        let propagating = |parameters: serde_json::Value,
                           values: serde_json::Value,
                           outputs: serde_json::Value| {
            serde_json::from_value::<SolveSettings>(json!({
                "version": 3,
                "sensitivity": {"parameters": [a, b], "propagation": {
                    "covariance": {"run_id": run, "parameters": parameters, "values": values},
                    "outputs": outputs,
                }},
            }))
            .unwrap()
            .profile()
        };
        let accepted = propagating(json!([b]), json!([2.0]), json!([x])).unwrap();
        assert_ne!(
            super::super::solves::profile_key(&accepted).unwrap(),
            super::super::solves::profile_key(&profile).unwrap()
        );
        for (parameters, values, outputs) in [
            (json!([x]), json!([2.0]), json!([x])),
            (json!([a, b]), json!([1.0, 0.5, 0.0, 1.0]), json!([x])),
            (json!([a]), json!([-1.0]), json!([x])),
            (json!([a]), json!([1.0]), json!([])),
        ] {
            assert!(
                propagating(parameters.clone(), values, outputs).is_err(),
                "{parameters}"
            );
        }
    }

    /// A value outside its single-value domain is refused where the document is decoded,
    /// with the domain's typed cause in the message (ADR-0116 Outcome 8).
    #[test]
    fn invalid_tolerance_refused_at_decode() {
        for (document, cause) in [
            (
                json!({"version": 3, "settings": {"backend": "ipopt", "bound_push": -1.0}}),
                "Tolerance",
            ),
            (
                json!({"version": 3, "settings": {"backend": "kinsol", "damping": 1.5}}),
                "Fraction",
            ),
            (
                json!({"version": 3, "settings": {"backend": "kinsol", "linear": {"kind": "spgmr", "dimension": 0}}}),
                "PositiveCount",
            ),
            (
                json!({"version": 3, "controls": {"time_limit": -3.0}}),
                "seconds",
            ),
        ] {
            let error = serde_json::from_value::<SolveSettings>(document)
                .unwrap_err()
                .to_string();
            assert!(error.contains(cause), "{error}");
        }
        assert_eq!(
            pse_model::scalars::Tolerance::try_new(-1.0),
            Err(pse_model::scalars::ToleranceError::GreaterViolated)
        );
        // Cross-field rules stay admission rules with typed reasons.
        let options = serde_json::from_value::<SolveSettings>(json!({
            "version": 3,
            "presolve_options": {"max_passes": 3},
        }))
        .unwrap();
        assert!(matches!(
            options.profile(),
            Err(MathRuntimeError::Solve(ProblemError::Contract(_)))
        ));
        let one_budget = serde_json::from_value::<SolveSettings>(json!({
            "version": 3,
            "convexity_absolute": 1e-9,
        }))
        .unwrap();
        assert!(one_budget.profile().is_err());
    }
}
