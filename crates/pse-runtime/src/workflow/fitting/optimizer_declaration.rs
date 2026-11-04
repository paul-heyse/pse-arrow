// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_enum! {
    /// Fitting owns selection between its residual method and native NLP methods.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum FitOptimizer {
        projection: {name:"FitOptimizer",rust_path:crate::workflow::FitOptimizer,version:1,
            direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
        /// Use residual LM only when native preparation establishes complete eligibility.
        #[default]
        Automatic => 1;
        /// Preserve the requested native NLP portfolio and settings.
        NativeNlp => 2;
        /// Require the supported unconstrained residual interpretation before effects.
        ResidualLm => 3;
    }
}
