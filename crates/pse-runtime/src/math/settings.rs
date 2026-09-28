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
use pse_model::{document::Version, numerics::NumericalPolicy};
use std::collections::BTreeSet;

/// The settings of one solve. The version is required; every other field takes the
/// default of [`SolverProfile::default`] when absent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SolveSettings {
    /// Document version.
    pub version: Version<1>,
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
    #[serde(default)]
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
}

mod defaults {
    use super::{PolicyKind, SolveIntent, SolverProfile};

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
            intent: defaults::intent(),
            backend: None,
            settings: None,
            presolve: defaults::presolve(),
            presolve_options: Options::new(),
            required_passes: BTreeSet::new(),
            controls: Controls::default(),
            numerics: NumericalPolicy::default(),
            convexity_absolute: None,
            convexity_relative: None,
        }
    }
}

impl SolveSettings {
    /// The solver profile these settings state, after the rules that relate fields:
    /// presolve options and required passes only with an explicit policy, finite controls,
    /// independent acceptance budgets, and both or neither convexity budget.
    ///
    /// # Errors
    /// A typed refusal naming the violated rule.
    pub fn profile(self) -> Result<SolverProfile, MathRuntimeError> {
        let presolve = Policy::new(self.presolve, &self.presolve_options, self.required_passes)?;
        self.controls.validate()?;
        self.numerics
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let convexity =
            ConvexityPolicy::from_tolerances(self.convexity_absolute, self.convexity_relative)?;
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Registry vocabularies are the document's enum types: every enumeration decodes from
    /// its registry spelling and a misspelled member is refused at decoding (Plan 22 B4, B5).
    #[test]
    fn solve_settings_enum_types() {
        let settings: SolveSettings = serde_json::from_value(json!({
            "version": 1,
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
            json!({"version": 1, "intent": "rooot"}),
            json!({"version": 1, "presolve": "magic"}),
            json!({"version": 1, "controls": {"reuse": "sometimes"}}),
            json!({"version": 1, "settings": {"backend": "gurobi"}}),
            json!({"version": 2}),
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
        assert_eq!(profile.controls, Controls::default());
    }

    /// A value outside its single-value domain is refused where the document is decoded,
    /// with the domain's typed cause in the message (ADR-0116 Outcome 8).
    #[test]
    fn invalid_tolerance_refused_at_decode() {
        for (document, cause) in [
            (
                json!({"version": 1, "settings": {"backend": "ipopt", "bound_push": -1.0}}),
                "Tolerance",
            ),
            (
                json!({"version": 1, "settings": {"backend": "kinsol", "damping": 1.5}}),
                "Fraction",
            ),
            (
                json!({"version": 1, "settings": {"backend": "kinsol", "linear": {"kind": "spgmr", "dimension": 0}}}),
                "PositiveCount",
            ),
            (
                json!({"version": 1, "controls": {"time_limit": -3.0}}),
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
            "version": 1,
            "presolve_options": {"max_passes": 3},
        }))
        .unwrap();
        assert!(matches!(
            options.profile(),
            Err(MathRuntimeError::Solve(ProblemError::Contract(_)))
        ));
        let one_budget = serde_json::from_value::<SolveSettings>(json!({
            "version": 1,
            "convexity_absolute": 1e-9,
        }))
        .unwrap();
        assert!(one_budget.profile().is_err());
    }
}
