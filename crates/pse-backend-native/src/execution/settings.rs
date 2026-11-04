// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

#[cfg(test)]
mod projection_controls {
    use super::*;
    use pse_contract::boundary_projection::{ProjectionGraph, ValidateProjection};

    #[test]
    fn backend_settings_projects_every_serializable_family_and_refuses_local_routed_default() {
        assert!(BackendSettings::Default.validate_projection().is_err());
        let graph = ProjectionGraph::of::<BackendSettings>().unwrap();
        let pse_contract::boundary_projection::Shape::Alternatives(alternatives) =
            &graph.definitions()["NativeBackendSettings"].shape
        else {
            panic!("actual backend alternatives")
        };
        assert_eq!(
            alternatives
                .iter()
                .map(|variant| (variant.name.as_str(), variant.tag))
                .collect::<Vec<_>>(),
            vec![
                ("Ipopt", 1),
                ("Pounce", 2),
                ("Kinsol", 3),
                ("Highs", 4),
                ("Clarabel", 5),
                ("Scip", 6),
                ("PounceConvex", 7),
                ("Uno", 8),
                ("Petsc", 9)
            ]
        );
        let choices = [
            BackendSettings::Ipopt(crate::settings::ipopt::Settings::default()),
            BackendSettings::Pounce(crate::settings::pounce::Settings::default()),
            BackendSettings::Kinsol(crate::settings::kinsol::Method::default()),
            BackendSettings::Highs(crate::settings::highs::Settings::default()),
            BackendSettings::Clarabel(crate::conic::Settings::default()),
            BackendSettings::Scip(ScipSettings::default()),
            BackendSettings::PounceConvex(crate::settings::pounce_convex::Settings::default()),
            BackendSettings::Uno(crate::settings::uno::Settings::default()),
            BackendSettings::Petsc(crate::settings::petsc::DirectSettings::default()),
        ];
        for original in choices {
            original.validate_projection().unwrap();
            let document = serde_json::to_vec(&original).unwrap();
            let restored: BackendSettings = serde_json::from_slice(&document).unwrap();
            assert_eq!(serde_json::to_vec(&restored).unwrap(), document);
            assert_eq!(original.backend(), restored.backend());
        }
    }
}

pse_contract::projection_enum! {
/// Explicit serializable native settings; routed defaults remain local to native dispatch.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum BackendSettings {
    projection: { name: "NativeBackendSettings", rust_path: crate::execution::BackendSettings, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    @unprojected {
    /// Native defaults and the common semantic controls on the routed backend.
    #[default]
    #[serde(skip)]
    Default;
    }
    /// Ipopt linear solver with its parameters, barrier strategy and initial-point push.
    #[schemars(title = "IpoptSettings")]
    Ipopt(crate::settings::ipopt::Settings) => 1;
    /// POUNCE method and complete native FERAL configuration.
    #[schemars(title = "PounceSettings")]
    Pounce(crate::settings::pounce::Settings) => 2;
    /// KINSOL method controls; scales and the step tolerance derive from policy.
    #[schemars(title = "KinsolSettings")]
    Kinsol(crate::settings::kinsol::Method) => 3;
    /// HiGHS method, opt-in diagnostics and partial MIP start.
    #[schemars(title = "HighsSettings")]
    Highs(crate::settings::highs::Settings) => 4;
    /// The pse-owned Clarabel settings: mode and every admitted native control.
    #[schemars(title = "ClarabelSettings")]
    Clarabel(crate::conic::Settings) => 5;
    /// SCIP's nested Ipopt linear solver, seed and node budget.
    #[schemars(title = "ScipSettings")]
    Scip(ScipSettings) => 6;
    /// The POUNCE-convex interior-point method's choices and FERAL configuration.
    #[schemars(title = "PounceConvexSettings")]
    PounceConvex(crate::settings::pounce_convex::Settings) => 7;
    /// First-order Uno filter trust-region SQP or SLP.
    #[schemars(title = "UnoSettings")]
    Uno(crate::settings::uno::Settings) => 8;
    /// Serial PETSc direct trust-region root profile.
    #[schemars(title = "PetscSettings")]
    Petsc(crate::settings::petsc::DirectSettings) => 9;
}
}
