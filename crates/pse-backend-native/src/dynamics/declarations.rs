// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_enum! {
/// Origin of the first physical IDAS step, independent of the IC orientation scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdasInitialStep {
    projection:{name:"NativeIdasInitialStep",rust_path:crate::dynamics::IdasInitialStep,version:1,
        direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Attempt the positive step in the shared dynamic profile.
    Configured => 1;
    /// Let the native integrator estimate its first physical step.
    #[default]
    Automatic => 2;
}
}

pse_contract::projection_enum! {
/// IDAS Newton linear solver; original band and Krylov admission remains contextual.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum IdasLinear {
    projection:{name:"NativeIdasLinear",rust_path:crate::dynamics::IdasLinear,version:1,
        direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// SuiteSparse KLU over the compiled analytic Jacobian.
    #[default]
    Klu => 1;
    /// Matrix-free GMRES over analytic Jacobian-vector products.
    Spgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Left preconditioner built from the compiled Jacobian.
        #[serde(default = "unpreconditioned")]
        preconditioner: crate::solve::Preconditioner => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 2;
    /// Exact matrix-free GMRES with the native library's band approximation.
    SpgmrBbd {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Envelope used by native grouped residual differences.
        difference: IdasBand => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Band of the approximation retained and factored by IDAS.
        retained: IdasBand => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 3;
    /// Matrix-free flexible GMRES over analytic Jacobian-vector products.
    Spfgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
        /// Left preconditioner built from the compiled Jacobian.
        #[serde(default = "unpreconditioned")]
        preconditioner: crate::solve::Preconditioner => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    } => 4;
}
}

pse_contract::projection_struct! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Typed Diffsol method document with its required original version.
pub struct DiffsolSettings {
    projection:{name:"NativeDiffsolSettings",rust_path:crate::dynamics::DiffsolSettings,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Document version.
    pub version: Version<1> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Time-stepping scheme.
    #[serde(default = "DiffsolSettings::default_method")]
    pub method: DiffsolMethod => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Newton linear solver of the implicit schemes.
    #[serde(default = "DiffsolSettings::default_linear")]
    pub linear: DiffsolLinear => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Finite original adjoint-checkpoint controls.
pub struct AdjointSettings {
    projection:{name:"NativeAdjointSettings",rust_path:crate::dynamics::AdjointSettings,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Document version.
    pub version: Version<1> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Native forward steps between stored checkpoints (IDAS `IDAAdjInit`'s `Nd`).
    #[serde(default = "AdjointSettings::default_steps")]
    pub steps_between_checkpoints: PositiveCount => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Largest number of forward checkpoints held at once.
    #[serde(default = "AdjointSettings::default_checkpoints")]
    pub max_checkpoints: PositiveCount => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Explicit half-bandwidths in the original admitted state ordering.
pub struct IdasBand {
    projection:{name:"NativeIdasBand",rust_path:crate::dynamics::IdasBand,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Number of original state columns retained above a row's diagonal.
    pub upper: usize => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Number of original state columns retained below a row's diagonal.
    pub lower: usize => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Typed original IDAS method and consistent-initialization controls.
pub struct IdasSettings {
    projection:{name:"NativeIdasSettings",rust_path:crate::dynamics::IdasSettings,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Document version.
    pub version: Version<3> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Newton linear solver.
    #[serde(default)]
    pub linear: IdasLinear => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Forward-sensitivity corrector.
    #[serde(default = "IdasSettings::default_sensitivity")]
    pub sensitivity: SensitivityCorrector => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Native first-step policy, independent of the positive IC orientation scale.
    #[serde(default)]
    pub native_initial_step: IdasInitialStep => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Consistent initialization at the start of the horizon; scheduled changes and resets
    /// always keep their differential states.
    #[serde(default = "IdasSettings::default_initialization")]
    pub initialization: IdasInitialization => {tag:5,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Public IDACalcIC phase controls, also applied to scheduled consistent restarts.
    #[serde(default)]
    pub initial_conditions: IdasInitialConditions => {tag:6,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
/// Exact native IDAS consistent-initialization work controls.
pub struct IdasInitialConditions {
    projection:{name:"NativeIdasInitialConditions",rust_path:crate::dynamics::IdasInitialConditions,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Maximum attempted artificial step scales; applies only to AlgebraicAndRates.
    pub step_trials: Option<u32> => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::UnsignedRange {min:1,max:i32::MAX as u64}]};
    /// Maximum Jacobian setup attempts at each artificial scale.
    pub jacobian_attempts: u32 => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::UnsignedRange {min:1,max:i32::MAX as u64}]};
    /// Maximum Newton iterations at each setup attempt.
    pub newton_iterations: u32 => {tag:3,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::UnsignedRange {min:1,max:i32::MAX as u64}]};
    /// Positive IDASetNonlinConvCoefIC coefficient.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub convergence_coefficient: f64 => {tag:4,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::PositiveFinite)]};
    /// Enable IDAS's IC line search.
    pub line_search: bool => {tag:5,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Explicit maximum backtracks per Newton step, when line search is enabled.
    pub backtracks: Option<u32> => {tag:6,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::UnsignedRange {min:1,max:i32::MAX as u64}]};
    /// Positive scaled Newton-step floor for constraints and the enabled line search.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub step_tolerance: Option<f64> => {tag:7,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::PositiveFinite)]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Declared completion obligation, independent of requested output times.
pub struct EndpointRequirement {
    projection:{name:"NativeEndpointRequirement",rust_path:crate::dynamics::EndpointRequirement,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Registry-owned completion policy.
    pub kind: pse_model::generated::enums::EndpointPolicy => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Allowed terminal guard identity, required only for declared-event completion.
    pub event: Option<SemanticId> => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One original parameter's strictly ordered piecewise-constant change times.
pub struct ScheduledInput {
    projection:{name:"NativeScheduledInput",rust_path:crate::dynamics::ScheduledInput,version:1,direction:pse_contract::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Position of the input in the contract's parameter order.
    pub parameter: usize => {tag:1,direction:pse_contract::boundary_projection::Direction::Both,rules:[]};
    /// Strictly increasing change times after the start and up to the end; a change at
    /// the end is observed by the final sample only.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub times: Vec<f64> => {tag:2,direction:pse_contract::boundary_projection::Direction::Both,rules:[pse_contract::boundary_projection::LocalRule::Length {min:1,max:u64::MAX}, pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)]};
}
}
