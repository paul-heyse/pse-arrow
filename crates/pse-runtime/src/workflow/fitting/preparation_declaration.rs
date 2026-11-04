// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pub(in crate::workflow) const RANK_TOLERANCE_DOMAIN: pse_contract::boundary_projection::Float64Domain =
    pse_contract::boundary_projection::Float64Domain::Interval(
        pse_contract::boundary_projection::Float64Interval::new(0.0, false, 1.0, false));

pse_contract::projection_struct! {
/// Decoded fit preparation; native fitting resolves selected models and experiments.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields, bound(serialize = "Simulation: serde::Serialize", deserialize = "Simulation: serde::Deserialize<'de>"))]
#[schemars(rename = "FitPreparationDocumentFor{Simulation}")]
#[schemars(bound = "Simulation: schemars::JsonSchema + serde::Serialize")]
pub struct FitPreparationDocument<Simulation = crate::workflow::SimulationProfile> {
    projection:{name:"FitPreparationDocumentFor",rust_path:crate::workflow::FitPreparationDocument,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Fitting-owned method selection, resolved before scientific effects.
    #[serde(default)]
    pub optimizer: super::FitOptimizer => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Canonical solve settings; their own owner admits cross-field rules.
    pub solver: crate::math::settings::SolveSettings => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Native dynamic settings by experiment identity.
    #[serde(default)]
    #[schemars(
        with = "pse_ids::document::KeyedMap<pse_model::generated::identities::InstanceId, Simulation>"
    )]
    pub simulations:
        std::collections::BTreeMap<pse_model::generated::identities::InstanceId, Simulation> => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    #[serde(
        default = "FitPreparationDocument::<crate::workflow::SimulationProfile>::default_rank_tolerance"
    )]
    #[doc = "Rank tolerance for fit admission."]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub rank_tolerance: f64 => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::Float64Domain(RANK_TOLERANCE_DOMAIN)]};
    #[serde(
        default = "FitPreparationDocument::<crate::workflow::SimulationProfile>::default_max_cells"
    )]
    #[doc = "Maximum native fit workspace cells."]
    pub max_cells: usize => {tag:5,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    #[serde(
        default = "FitPreparationDocument::<crate::workflow::SimulationProfile>::default_derivatives"
    )]
    #[doc = "Requested derivative evidence policy."]
    pub derivatives: super::FitDerivatives => {tag:6,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    #[serde(
        default = "FitPreparationDocument::<crate::workflow::SimulationProfile>::default_experiment_workers"
    )]
    #[doc = "Finite independent experiment workers, separately from profile chains."]
    pub experiment_workers: PositiveCount => {tag:7,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    #[serde(default)]
    #[doc = "Optional fit uncertainty request."]
    pub uncertainty: Option<super::FitUncertainty> => {tag:8,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}
