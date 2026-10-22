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
use std::collections::{BTreeMap, BTreeSet};

/// The settings of one solve. The version is required; every other field takes the
/// default of [`SolverProfile::default`] when absent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SolveSettings {
    /// Compiler stable block identities and admitted native realization overrides.
    /// Validated against the actual compiler scope even when direct execution wins;
    /// applied only if the governing policy selects the block mechanism.
    #[serde(default)]
    #[schemars(with = "pse_ids::document::KeyedMap<pse_structural::incidence::BlockId, BlockSolveOverride>")]
    pub block_overrides: BTreeMap<pse_structural::incidence::BlockId, BlockSolveOverride>,
    /// Maximum independent ready blocks; actual capacity is admitted by the existing owner.
    #[serde(default = "defaults::block_workers")]
    pub block_workers: usize,
    /// Document version.
    pub version: Version<5>,
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParameterCovariance {
    /// Covariance input interpretation; required before decoding its body.
    pub version: Version<1>,
    /// A qualified fit reference or explicit external statistical assumptions.
    pub origin: CovarianceOrigin,
    /// The fitted parameters by identity, in the order of `values`.
    pub parameters: Vec<pse_ids::SemanticId>,
    /// `Σ_θ` row-major, in the parameters' units.
    pub values: Vec<FiniteBound>,
    /// Positive coordinate scales in those same units, in parameter order.
    /// Fit inputs use their recorded scales; external inputs declare their scales.
    pub scales: Vec<FiniteBound>,
    /// Complete quantity contracts and representation units, in parameter order.
    pub parameter_quantities: Vec<pse_ids::SemanticId>,
    /// Representation unit of each matrix row/column and its coordinate scale.
    pub parameter_units: Vec<pse_ids::SemanticId>,
    // Deserialization cannot create evidence. A fit payload is reconciled against
    // actual immutable results before this process-local authority is issued.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) verified: Option<std::sync::Arc<VerifiedFitCovariance>>,
}

/// Exact canonical selection of the fit that owns the covariance.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitCovarianceRecord {
    /// Native canonical run key.
    pub run: String,
    /// Exact native attempt key; latest-result substitution is not allowed.
    pub attempt: String,
}
/// Statistical provenance is independent of matrix admissibility.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CovarianceOrigin {
    /// Resolve or reconcile actual qualified fit content and its local validity.
    QualifiedFit {
        /// Scientific run identity, checked against the selected actual fit.
        run_id: pse_model::generated::identities::RunId,
        /// Absent only for a still-owned in-process fit; a decoded reference then refuses.
        record: Option<FitCovarianceRecord>,
    },
    /// Caller-declared covariance with uncertified statistical provenance.
    External {
        /// Attribution label; it never identifies a certified fit.
        label: String,
        /// Explicit statistical assumptions of this external input.
        assumptions: Vec<String>,
    },
}

#[derive(Debug)]
pub(crate) struct VerifiedFitCovariance {
    pub(crate) content: pse_ids::ContentHash,
    pub(crate) source: pse_ids::ContentHash,
    pub(crate) physical: pse_ids::ContentHash,
    pub(crate) validity: pse_model::generated::structures::LocalValidity,
    pub(crate) _owner: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>,
}
impl PartialEq for ParameterCovariance {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin && self.parameters == other.parameters
            && self.values == other.values && self.scales == other.scales
            && self.parameter_quantities == other.parameter_quantities
            && self.parameter_units == other.parameter_units
    }
}
impl ParameterCovariance {
    /// A numerical covariance supplied under explicit external assumptions.
    pub fn external(
        parameters: Vec<pse_ids::SemanticId>, parameter_quantities: Vec<pse_ids::SemanticId>,
        parameter_units: Vec<pse_ids::SemanticId>, values: Vec<FiniteBound>,
        scales: Vec<FiniteBound>, label: String, assumptions: Vec<String>,
    ) -> Self {
        Self { version: Version, origin: CovarianceOrigin::External { label, assumptions }, parameters,
            parameter_quantities, parameter_units, values, scales, verified: None }
    }
    pub(crate) fn content(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        pse_ids::document::of(pse_ids::Frame::CovarianceInputV1, self)
            .map_err(|error| ProblemError::Contract(error.to_string()))
    }
    pub(crate) fn verified_fit(&self) -> Result<Option<&VerifiedFitCovariance>, ProblemError> {
        match &self.origin {
            CovarianceOrigin::External { .. } => Ok(None),
            CovarianceOrigin::QualifiedFit { .. } => self.verified.as_deref()
                .filter(|proof| proof.validity.certified)
                .filter(|proof| self.content().is_ok_and(|content| content == proof.content))
                .map(Some)
                .ok_or_else(|| ProblemError::Contract("fit covariance payload has not been reconciled with its actual qualified result".into())),
        }
    }
    /// Structural input admission. Mathematical PSD admission also requires the
    /// caller's resource owner and is performed before propagation.
    ///
    /// # Errors
    /// A contract error naming the violated rule.
    pub fn admit(&self) -> Result<(), ProblemError> {
        let n = self.parameters.len();
        let distinct: BTreeSet<_> = self.parameters.iter().collect();
        if n == 0
            || distinct.len() != n
            || n.checked_mul(n) != Some(self.values.len())
            || self.scales.len() != n
            || self.parameter_quantities.len() != n
            || self.parameter_units.len() != n
            || self.scales.iter().any(|scale| scale.into_inner() <= 0.0)
        {
            return Err(ProblemError::Contract(
                "a parameter covariance names distinct parameters, a square finite matrix and one positive scale per parameter".into(),
            ));
        }
        if let CovarianceOrigin::External { label, assumptions } = &self.origin
            && (label.trim().is_empty() || assumptions.is_empty() || assumptions.iter().any(|value| value.trim().is_empty()))
        {
            return Err(ProblemError::Contract("external covariance requires an attribution label and explicit nonempty assumptions".into()));
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

/// Native realization and finite work controls of one compiler-issued block.
/// Physical numerical meaning, intent, branch permission and authored start stay inherited.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockSolveOverride {
    /// Explicit eligible backend; omission uses contextual Auto selection.
    #[serde(default)]
    pub backend: Option<Backend>,
    /// Typed settings admitted against this actual block.
    #[serde(default)]
    pub settings: Option<BackendSettings>,
    /// Optional finite controls within the original aggregate allowance.
    #[serde(default)]
    pub controls: Option<Controls>,
}
impl BlockSolveOverride {
    #[cfg(any(feature = "solver-kinsol", test))]
    pub(crate) fn apply(&self, original: &SolverProfile) -> Result<SolverProfile, ProblemError> {
        let mut profile = original.clone();
        profile.block_overrides.clear();
        profile.block_workers = 1;
        profile.selection = self
            .backend
            .map_or(SolverSelection::Auto, SolverSelection::Explicit);
        profile.backend = BackendSettings::from_document(self.settings.clone());
        if let Some(controls) = &self.controls {
            controls.validate()?;
            if controls.start != original.controls.start
                || controls.reuse != original.controls.reuse
                || controls.hessian != original.controls.hessian
                || controls.time_limit > original.controls.time_limit
                || controls.iterations > original.controls.iterations
                || controls.history > original.controls.history
                || controls.threads > original.controls.threads
                || original
                    .controls
                    .foreign_bytes
                    .is_some_and(|cap| controls.foreign_bytes.is_none_or(|bytes| bytes > cap))
            {
                return Err(ProblemError::Contract("block override changes original start/reuse/derivative permission or expands aggregate controls".into()));
            }
            profile.controls = controls.clone();
        }
        Ok(profile)
    }
}
mod defaults {
    use super::{PolicyKind, SolveIntent, SolverProfile};

    pub(super) fn block_workers() -> usize {
        SolverProfile::default().block_workers
    }
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
            block_overrides: BTreeMap::new(),
            block_workers: defaults::block_workers(),
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
        if self.block_workers == 0 || self.block_workers > 4096 || self.block_overrides.len() > 4096
        {
            return Err(ProblemError::Contract("finite block execution extent".into()).into());
        }
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
            block_overrides: self.block_overrides,
            block_workers: self.block_workers,
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
    fn block_overrides_refuse_unknown_identity_and_expanded_or_changed_start_controls() {
        let id = pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([61; 32]));
        let mut profile = SolverProfile::default();
        profile.block_overrides.insert(
            id,
            BlockSolveOverride {
                backend: Some(Backend::Kinsol),
                settings: None,
                controls: None,
            },
        );
        assert!(profile.validate_block_ids(std::iter::empty()).is_err());
        profile.validate_block_ids(std::iter::once(id)).unwrap();
        let document = SolveSettings {
            block_overrides: profile.block_overrides.clone(),
            block_workers: 2,
            ..Default::default()
        };
        let wire = serde_json::to_string(&document).unwrap();
        let roundtrip: SolveSettings = serde_json::from_str(&wire).unwrap();
        assert!(roundtrip.block_overrides.contains_key(&id));
        assert_eq!(roundtrip.block_workers, 2);
        let effective = profile.block_profile(id).unwrap();
        assert_eq!(
            effective.selection,
            SolverSelection::Explicit(Backend::Kinsol)
        );
        assert_eq!(effective.numerics.key(), profile.numerics.key());
        assert!(effective.block_overrides.is_empty());
        for controls in [
            Controls {
                iterations: profile.controls.iterations + 1,
                ..profile.controls.clone()
            },
            Controls {
                start: pse_backend_native::solve::StartPolicy::PreviousAccepted,
                ..profile.controls.clone()
            },
            Controls {
                time_limit: profile.controls.time_limit * 2,
                ..profile.controls.clone()
            },
        ] {
            profile.block_overrides.get_mut(&id).unwrap().controls = Some(controls);
            assert!(profile.validate_block_ids(std::iter::once(id)).is_err());
        }
    }
    #[test]
    fn current_request_admits_version_before_body_and_preserves_start_grants() {
        let old = br#"{"settings":{"backend":"retired"},"version":1}"#;
        let error = pse_model::document::decode_versioned::<SolveSettings, 5>(old).unwrap_err();
        assert!(error.to_string().contains("explicit readmission"));
        let current =
            pse_model::document::decode_versioned::<SolveSettings, 5>(br#"{"version":5}"#).unwrap();
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
            "version": 5,
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
            json!({"version": 5, "intent": "rooot"}),
            json!({"version": 5, "presolve": "magic"}),
            json!({"version": 5, "controls": {"reuse": "sometimes"}}),
            json!({"version": 5, "settings": {"backend": "gurobi"}}),
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
            "version": 5,
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
            json!({"version":5,"intent":"root","sensitivity":{"parameters":[a]}}),
        )
        .unwrap();
        assert!(root.profile().is_ok());
        for refused in [
            json!({"version": 5, "intent": "root", "sensitivity": {"parameters": [a], "reduced_hessian": true}}),
            json!({"version": 5, "sensitivity": {"parameters": [a, a]}}),
            json!({"version": 5, "sensitivity": {"parameters": []}}),
        ] {
            let settings: SolveSettings = serde_json::from_value(refused.clone()).unwrap();
            assert!(settings.profile().is_err(), "{refused}");
        }
        assert!(
            serde_json::from_value::<SolveSettings>(json!({
                "version": 5,
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
            let count = parameters.as_array().unwrap().len();
            serde_json::from_value::<SolveSettings>(json!({
                "version": 5,
                "sensitivity": {"parameters": [a, b], "propagation": {
                    "covariance": {"version": 1, "origin": {"kind": "external", "label": "admission control", "assumptions": ["declared model"]}, "parameters": parameters, "values": values,
                        "parameter_quantities": vec![a; count], "parameter_units": vec![run; count], "scales": vec![1.0; count]},
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
            (json!([a, a]), json!([1.0, 0.0, 0.0, 1.0]), json!([x])),
            (json!([a, b]), json!([1.0, 0.0]), json!([x])),
            (json!([a]), json!([1.0]), json!([])),
        ] {
            assert!(
                propagating(parameters.clone(), values, outputs).is_err(),
                "{parameters}"
            );
        }
        // Symmetry and definiteness belong to the prepaid faer admission, not
        // this structural document/profile decoder.
    }

    /// A value outside its single-value domain is refused where the document is decoded,
    /// with the domain's typed cause in the message (ADR-0116 Outcome 8).
    #[test]
    fn invalid_tolerance_refused_at_decode() {
        for (document, cause) in [
            (
                json!({"version": 5, "settings": {"backend": "ipopt", "bound_push": -1.0}}),
                "Tolerance",
            ),
            (
                json!({"version": 5, "settings": {"backend": "kinsol", "damping": 1.5}}),
                "Fraction",
            ),
            (
                json!({"version": 5, "settings": {"backend": "kinsol", "linear": {"kind": "spgmr", "dimension": 0}}}),
                "PositiveCount",
            ),
            (
                json!({"version": 5, "controls": {"time_limit": -3.0}}),
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
            "version": 5,
            "presolve_options": {"max_passes": 3},
        }))
        .unwrap();
        assert!(matches!(
            options.profile(),
            Err(MathRuntimeError::Solve(ProblemError::Contract(_)))
        ));
        let one_budget = serde_json::from_value::<SolveSettings>(json!({
            "version": 5,
            "convexity_absolute": 1e-9,
        }))
        .unwrap();
        assert!(one_budget.profile().is_err());
    }
}
