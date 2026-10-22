// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical host controls become normalized integration controls only after preparation.
use super::{WorkflowError, contract, physical_values::admit_tolerances};
use pse_backend_native::dynamics::{IntegrationControls, Profile};
use pse_kernels::Port;
use pse_model::{document::Version, generated::structures::ScientificCoordinateValue};

/// Pending host simulation controls. Physical time stays signed binary64 seconds;
/// all coordinate-bearing tolerances and characteristic scales carry keys and units.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalSimulationRequest {
    /// Required physical-input interpretation.
    pub version: Version<1>,
    /// One controls declaration with physically typed Delta coordinate payloads.
    pub controls: IntegrationControls<Vec<ScientificCoordinateValue>>,
    #[serde(skip)]
    custody: Option<std::sync::Arc<dyn pse_columnar::PayloadOwner>>,
}
/// Host fitting settings whose experiment controls still carry physical coordinates.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalFitRequest {
    /// Required physical experiment-input interpretation.
    pub version: Version<1>,
    /// One shared fitting-settings declaration, with pending experiment controls.
    pub settings: super::FitPreparationDocument<PhysicalSimulationRequest>,
}
impl PhysicalFitRequest {
    /// Wrap the sole fitting-settings declaration in its required physical interpretation.
    pub fn new(settings: super::FitPreparationDocument<PhysicalSimulationRequest>) -> Self {
        Self {
            version: Version,
            settings,
        }
    }
    /// Own each decoded physical experiment request under the receiving deployment.
    pub fn retain(mut self, runtime: &super::Runtime) -> Result<Self, WorkflowError> {
        for request in self.settings.simulations.values_mut() {
            *request = request.clone().retain(runtime)?;
        }
        Ok(self)
    }
}
impl PhysicalSimulationRequest {
    /// Construct pending controls from owned values, with no external buffers retained.
    pub fn new(controls: IntegrationControls<Vec<ScientificCoordinateValue>>) -> Self {
        Self {
            version: Version,
            controls,
            custody: None,
        }
    }
    /// Retain decoded/request scalar storage under the existing deployment budget.
    pub fn retain(mut self, runtime: &super::Runtime) -> Result<Self, WorkflowError> {
        let extent =
            super::replay::encoded_extent(&self, runtime.shared.budget().math.workspace_bytes)?
                .checked_mul(8)
                .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 512))
                .ok_or_else(|| contract("physical simulation request extent"))?;
        self.custody = Some(
            runtime
                .shared
                .math()
                .reserve("workflow:physical-simulation-request", extent)?,
        );
        Ok(self)
    }
    pub(in crate::workflow) fn admit(
        &self,
        registry: &pse_quantity::QuantityRegistry,
        states: &[Port],
        integrals: &[Port],
        parameters: &[Port],
        state_scales: &[f64],
    ) -> Result<Profile, WorkflowError> {
        if states.len() != state_scales.len() {
            return Err(contract("physical state scale inventory differs"));
        }
        let state = admit_tolerances(registry, states, &self.controls.atol)?;
        let output = admit_tolerances(registry, integrals, &self.controls.out_atol)?;
        let parameter = admit_tolerances(registry, parameters, &self.controls.parameter_scales)?;
        let atol = states
            .iter()
            .zip(state_scales)
            .map(|(port, scale)| {
                let tolerance = state[&port.id] / scale.abs();
                if !scale.is_finite() || *scale <= 0.0 || !tolerance.is_finite() || tolerance <= 0.0
                {
                    return Err(contract(
                        "normalized physical state tolerance must be finite and positive",
                    ));
                }
                Ok(tolerance)
            })
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        let out_atol = integrals.iter().map(|port| output[&port.id]).collect();
        let parameter_scales = parameters.iter().map(|port| parameter[&port.id]).collect();
        Ok(self
            .controls
            .clone()
            .with_coordinates(atol, out_atol, parameter_scales))
    }
}
