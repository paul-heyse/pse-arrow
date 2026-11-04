// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

#[cfg(test)]
mod projection_controls {
    use super::*;
    use pse_model::boundary_projection::{ProjectionGraph, ProjectionRecord, ValidateProjection};

    #[test]
    fn actual_solve_settings_projection_preserves_original_default_bytes_and_identity() {
        let original = SolveSettings::default();
        let bytes = serde_json::to_vec(&original).unwrap();
        let identity = pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &original).unwrap();
        original.validate_projection().unwrap();
        let restored = SolveSettings::from_projection_fields(original.into_projection_fields()).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        assert_eq!(pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &restored).unwrap(), identity);
        let graph = ProjectionGraph::of::<SolveSettings>().unwrap();
        assert_eq!(graph.definitions()["SolveSettings"].version, 1);
        assert!(graph.definitions().contains_key("NativeBackendSettings"));
        assert!(graph.definitions().contains_key("SolveControls"));
    }

    #[test]
    fn convexity_numeric_inputs_keep_signed_zero_and_refuse_nonfinite_number_loss() {
        let mut settings = SolveSettings::default();
        settings.convexity_absolute = Some(-0.0);
        settings.convexity_relative = Some(0.0);
        let encoded = serde_json::to_vec(&settings).unwrap();
        let decoded: SolveSettings = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.convexity_absolute.unwrap().to_bits(), (-0.0_f64).to_bits());
        for nonfinite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            settings.convexity_absolute = Some(nonfinite);
            assert!(serde_json::to_vec(&settings).is_err());
            assert!(settings.validate_projection().is_err());
        }
    }
}

pse_model::projection_struct! {
/// Settings of one solve; original per-field defaults and admission stay authoritative.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SolveSettings {
    projection: { name: "SolveSettings", rust_path: crate::math::settings::SolveSettings, version: 1,
        direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Compiler stable block identities and admitted native realization overrides.
    /// Validated against the actual compiler scope even when direct execution wins;
    /// applied only if the governing policy selects the block mechanism.
    #[serde(default)]
    #[schemars(
        with = "pse_ids::document::KeyedMap<pse_structural::incidence::BlockId, BlockSolveOverride>"
    )]
    pub block_overrides: BTreeMap<pse_structural::incidence::BlockId, BlockSolveOverride> => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::Length { min: 0, max: 4096 }] };
    /// Maximum independent ready blocks; actual capacity is admitted by the existing owner.
    #[serde(default = "defaults::block_workers")]
    pub block_workers: usize => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 4096 }] };
    /// Document version.
    pub version: Version<5> => { tag: 3, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Requested composition; omission chooses automatic resolution without replacement starts.
    #[serde(default)]
    pub composition: pse_model::strategy::CompositionRequest => { tag: 4, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Explicit reconstruction evidence class and finite refinement allowances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconstruction: Option<pse_backend_native::derived::ReconstructionAccuracy> => { tag: 5, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Mathematical purpose.
    #[serde(default = "defaults::intent")]
    pub intent: SolveIntent => { tag: 6, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// An explicitly selected eligible backend; absent is deterministic routing.
    #[serde(default)]
    pub backend: Option<Backend> => { tag: 7, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Typed settings of one backend; absent is the routed backend's native defaults.
    #[serde(default)]
    pub settings: Option<BackendSettings> => { tag: 8, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Library presolve policy.
    #[serde(default = "defaults::presolve")]
    pub presolve: PolicyKind => { tag: 9, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Native presolve options; an explicit policy only.
    #[serde(default)]
    pub presolve_options: Options => { tag: 10, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Passes that may not silently become unavailable; an explicit policy only.
    #[serde(default)]
    pub required_passes: BTreeSet<Pass> => { tag: 11, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Shared finite attempt controls.
    #[serde(default = "defaults::controls")]
    pub controls: Controls => { tag: 12, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// ID-keyed numerical requirements and acceptance budgets.
    #[serde(default)]
    pub numerics: NumericalPolicy => { tag: 13, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Absolute eigenvalue budget of a numerical convexity assessment; with the relative
    /// budget, it permits numerical assessment, and absent both, only exact certification.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub convexity_absolute: Option<f64> => { tag: 14, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::NonnegativeFinite)] };
    /// Relative eigenvalue budget of a numerical convexity assessment.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub convexity_relative: Option<f64> => { tag: 15, direction: pse_model::boundary_projection::Direction::Both, rules: [pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::NonnegativeFinite)] };
    /// Physical parameter sensitivities at a regular square Root or optimizing candidate;
    /// absent computes none. An absent request is not encoded, so a document without
    /// one keeps its identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<SensitivityRequest> => { tag: 16, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
}
}

pse_model::projection_struct! {
/// Native realization and finite work controls of one compiler-issued block.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockSolveOverride {
    projection: { name: "BlockSolveOverride", rust_path: crate::math::settings::BlockSolveOverride, version: 1,
        direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Explicit eligible backend; omission uses contextual Auto selection.
    #[serde(default)]
    pub backend: Option<Backend> => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Typed settings admitted against this actual block.
    #[serde(default)]
    pub settings: Option<BackendSettings> => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    /// Optional finite controls within the original aggregate allowance.
    #[serde(default)]
    pub controls: Option<Controls> => { tag: 3, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
}
}
