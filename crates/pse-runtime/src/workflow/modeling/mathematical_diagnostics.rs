// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected exact/assurance operations over the original admitted modeling package.
use super::*;
use pse_math::mathematical_diagnostics::{DiagnosticRequest, DiagnosticResult};

pse_contract::projection_struct! {
/// An authored original-source mathematical demand with explicit fixed member values.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MathematicalDiagnosticOperation {
    projection: { name: "MathematicalDiagnosticOperation", rust_path: crate::workflow::MathematicalDiagnosticOperation, version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };

    /// Original modeling declaration.
    pub root: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original specialized instance.
    pub instance: InstanceId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original selected source demand and specialization.
    pub bindings: Bindings => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Fixed physical inputs in their admitted canonical representation.
    pub case_members: BTreeMap<SemanticId, f64> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordered exact/assurance mathematical demand.
    pub request: DiagnosticRequest => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

impl ModelingPackage {
    /// Execute the typed mathematical operation through the shared native job owner.
    pub async fn diagnose_selected_mathematics(
        &self,
        services: &crate::workflow::PreparationServices,
        operation: &MathematicalDiagnosticOperation,
        settings: &crate::workflow::PreparationSettings,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<DiagnosticResult>>, WorkflowError> {
        self.check_service_bindings(services)?;
        let model = self
            .prepare(
                services,
                operation.root,
                operation.instance,
                operation.bindings.clone(),
                settings.limits,
                cancel,
            )
            .await?;
        let values = model
            .compiled()
            .case_values(&pse_compiler::workspace::ModelingCaseBindings {
                members: operation.case_members.clone(),
                ..Default::default()
            })
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(services
            .math()
            .diagnose_modeling(model, values, operation.request.clone(), cancel)
            .await?)
    }
}

#[cfg(test)]
#[path = "mathematical_diagnostics_tests.rs"]
mod tests;
