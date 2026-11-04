// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_model::projection_struct! {
/// Finite integration policy; physical coordinate admission stays with the original owner.
#[derive(Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "IntegrationControlsFor{Coordinates}")]
#[schemars(bound = "Coordinates: schemars::JsonSchema + serde::Serialize")]
pub struct IntegrationControls<Coordinates = Vec<f64>> {
    projection:{name:"IntegrationControlsFor",rust_path:crate::dynamics::IntegrationControls,version:3,direction:pse_model::boundary_projection::Direction::Both,reserved_tags:[],reserved_names:[]};
    /// Required execution interpretation version; legacy unversioned requests need readmission.
    pub version: Version<3> => {tag:1,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Native method selected from the required trial semantics; automatic when absent.
    #[serde(default = "automatic")]
    pub method: Method => {tag:2,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Behavior required for internal domain failures; terminal when absent.
    #[serde(default = "terminal")]
    pub trial_failures: TrialPolicy => {tag:3,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Physical acceptance and ID-keyed nominal requests, distinct from integration error controls.
    #[serde(default)]
    pub numerics: pse_model::numerics::NumericalPolicy => {tag:4,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Declared completion event; omitted means the full fixed horizon.
    #[serde(default)]
    pub endpoint: EndpointRequirement => {tag:5,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Initial physical time in seconds.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub start: f64 => {tag:6,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::Finite)]};
    /// Requested final physical time in seconds.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub end: f64 => {tag:7,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::Finite)]};
    /// Strictly increasing finite output times, within the horizon.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub samples: Vec<f64> => {tag:8,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Length {min:1,max:u64::MAX}, pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::Finite)]};
    /// Positive relative integration tolerance.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub rtol: f64 => {tag:9,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::PositiveFinite)]};
    /// Explicit integrated-flux relative tolerance; required when conservation is declared.
    #[serde(default)]
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub out_rtol: Option<f64> => {tag:10,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::PositiveFinite)]};
    /// Absolute integrated-flux tolerances in canonical conserved quantities.
    pub out_atol: Coordinates => {tag:11,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Positive absolute tolerances in normalized state order.
    pub atol: Coordinates => {tag:12,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Positive initialization orientation scale in seconds. IDAS also attempts this
    /// physical step only under [`IdasInitialStep::Configured`]; its default uses the
    /// native estimate. Diffsol's initial-step meaning is unchanged.
    #[serde(serialize_with = "pse_model::document::finite::serialize", deserialize_with = "pse_model::document::finite::deserialize")]
    pub initial_step: f64 => {tag:13,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::Float64Domain(pse_model::boundary_projection::Float64Domain::PositiveFinite)]};
    /// Maximum accepted advancement steps in each complete trajectory: forward/recording,
    /// all forward replay of one derivative operation, and each backward direction.
    /// Physical segments, checkpoint restores and sample jumps do not renew this cap.
    /// Interpolation consumes no steps; failed calls and rejected trials are separate work.
    pub max_steps: usize => {tag:14,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::UnsignedRange {min:1,max:usize::MAX as u64}]};
    /// Maximum retained event records.
    pub max_events: usize => {tag:15,direction:pse_model::boundary_projection::Direction::Both,rules:[pse_model::boundary_projection::LocalRule::UnsignedRange {min:1,max:usize::MAX as u64}]};
    /// Cooperative outer time allowance, including callbacks and resets.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub time_limit: Duration => {tag:16,direction:pse_model::boundary_projection::Direction::Both,rules:[crate::solve::POSITIVE_TIME_LIMIT_RULE]};
    /// Maximum retained scalar cells, including sensitivities and event states.
    pub max_cells: usize => {tag:17,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Parameter derivatives: none, forward sensitivities of every sample, or adjoint
    /// gradients of one functional of the sampled outputs through `gradient`; none when
    /// absent.
    #[serde(default = "no_sensitivity")]
    pub sensitivity: DynamicSensitivity => {tag:18,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Checkpointing of the adjoint backward pass; used only by adjoint gradients.
    #[serde(default)]
    pub adjoint: AdjointSettings => {tag:19,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Positive characteristic scales in the contract's parameter order, also used for
    /// sensitivity tolerances; every interval of a scheduled input takes its parameter's.
    pub parameter_scales: Coordinates => {tag:20,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Scheduled inputs: a contract parameter that takes one integration value per
    /// schedule interval, each with its own live sensitivity (I6).
    #[serde(default)]
    pub schedule: Vec<ScheduledInput> => {tag:21,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Typed Diffsol scheme and linear solver.
    #[serde(default)]
    pub diffsol: DiffsolSettings => {tag:22,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Typed IDAS linear solver, sensitivity corrector and start.
    #[serde(default)]
    pub idas: IdasSettings => {tag:23,direction:pse_model::boundary_projection::Direction::Both,rules:[]};
    /// Library-owned initialization controls, transported in every profile.
    #[serde(with = "initial_options")]
    #[schemars(with = "initial_options::Remote")]
    pub initialization: Arc<diffsol::InitialConditionSolverOptions<f64>> => {tag:24,direction:pse_model::boundary_projection::Direction::Both,rules:[],adapter:pse_model::boundary_projection::SharedAdapter<InitialConditionProjection>};
    /// Native BDF/nonlinear/error-control settings.
    #[serde(with = "ode_options")]
    #[schemars(with = "ode_options::Remote")]
    pub native: Arc<diffsol::OdeSolverOptions<f64>> => {tag:25,direction:pse_model::boundary_projection::Direction::Both,rules:[],adapter:pse_model::boundary_projection::SharedAdapter<OdeOptionsProjection>};
    // Original Serde omissions do not grant remote derivative-demand authority.
    @local {
        /// Process-local selected forward axes; ordinary documents reconstruct all axes.
        #[serde(skip)]
        pub forward_axes: Option<Vec<usize>> = None => check: no_forward_axes;
        /// Process-local adjoint demand; ordinary documents reconstruct All.
        #[serde(skip)]
        pub adjoint_parameters: AdjointParameters = AdjointParameters::All => check: no_selected_adjoint;
    }
}
}

fn no_forward_axes(
    axes: &Option<Vec<usize>>,
    path: &str,
) -> Result<(), pse_model::boundary_projection::ValidationError> {
    if axes.is_none() {
        Ok(())
    } else {
        Err(pse_model::boundary_projection::ValidationError::Rule {
            path: path.into(),
            rule: "local.derivative_demand",
            reason: "process-local selected derivative axes require native contextual admission"
                .into(),
        })
    }
}

fn no_selected_adjoint(
    axes: &AdjointParameters,
    path: &str,
) -> Result<(), pse_model::boundary_projection::ValidationError> {
    if *axes == AdjointParameters::All {
        Ok(())
    } else {
        Err(pse_model::boundary_projection::ValidationError::Rule {
            path: path.into(),
            rule: "local.derivative_demand",
            reason: "process-local selected adjoint axes require native contextual admission"
                .into(),
        })
    }
}

#[cfg(test)]
mod integration_projection_controls {
    use super::*;
    use pse_model::boundary_projection::{ProjectionGraph, ProjectionRecord, ValidateProjection};

    #[test]
    fn original_integration_projection_preserves_control_bytes_and_refuses_local_axis_demand() {
        let original = Profile::default();
        let bytes = serde_json::to_vec(&original).unwrap();
        let restored = Profile::from_projection_fields(original.into_projection_fields()).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        assert!(restored.forward_axes.is_none());
        assert_eq!(restored.adjoint_parameters, AdjointParameters::All);
        restored.validate_projection().unwrap();
        let graph = ProjectionGraph::of::<Profile>().unwrap();
        assert!(
            graph
                .definitions()
                .contains_key("SharedDiffsolInitialConditionOptions")
        );
        assert!(
            graph
                .definitions()
                .contains_key("SharedDiffsolOdeSolverOptions")
        );
        let mut local = restored;
        local.forward_axes = Some(vec![0]);
        assert!(local.validate_projection().is_err());
        local.forward_axes = None;
        local.adjoint_parameters = AdjointParameters::Selected(vec![]);
        assert!(local.validate_projection().is_err());
    }
}
