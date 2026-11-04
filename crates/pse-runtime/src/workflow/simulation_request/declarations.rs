// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_model::projection_struct! {
    /// Pending host simulation controls; coordinates retain physical keys and units.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct PhysicalSimulationRequest {
        projection: {name:"PhysicalSimulationRequest",rust_path:crate::workflow::PhysicalSimulationRequest,
            version:1,direction:pse_model::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
        /// Required physical-input interpretation.
        pub version: Version<1> => {tag:1,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
        /// One controls declaration with physically typed Delta coordinate payloads.
        pub controls: IntegrationControls<Vec<ScientificCoordinateValue>> => {
            tag:2,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
        @local {
            // Original Serde omits this accounting grant; the receiving retain() method
            // reserves its own actual request extent. It conveys no scientific evidence.
            #[serde(skip)]
            custody: Option<std::sync::Arc<dyn pse_columnar::PayloadOwner>> = None =>
                check: |_: &Option<std::sync::Arc<dyn pse_columnar::PayloadOwner>>, _: &str| Ok(());
        }
    }
}

pse_model::projection_struct! {
    /// Host fit settings whose experiment controls retain physical coordinates.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct PhysicalFitRequest {
        projection: {name:"PhysicalFitRequest",rust_path:crate::workflow::PhysicalFitRequest,
            version:1,direction:pse_model::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
        /// Required physical experiment-input interpretation.
        pub version: Version<1> => {tag:1,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
        /// One shared fitting declaration with pending experiment controls.
        pub settings: super::FitPreparationDocument<PhysicalSimulationRequest> => {
            tag:2,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    }
}
