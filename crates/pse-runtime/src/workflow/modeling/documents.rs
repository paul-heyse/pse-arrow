// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned transport of source editing and nonexecuting selected-revision inspection.
use pse_ids::SemanticId;
use pse_model::generated::{
    enums::*,
    identities::{DeclarationId, InstanceId},
};
use std::collections::BTreeMap;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Authored declarations submitted for contextual revision admission."]
pub struct DeclarationEdit {
    #[doc = "Declarations from the selected authored revision."]
    pub declarations: Vec<pse_authoring::language::Declaration>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Declarations retained by the selected revision."]
pub struct DeclarationInventory {
    #[doc = "Declarations from the selected authored revision."]
    pub declarations: Vec<pse_authoring::language::Declaration>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Nonexecuting observation of the selected model."]
pub struct ModelingInspection {
    #[doc = "Selected route and procedure before execution."]
    pub execution: InspectionExecution,
    #[doc = "Instantiated authored definitions."]
    pub instances: Vec<InspectionInstance>,
    #[doc = "Instantiated model members and their provenance."]
    pub members: Vec<InspectionMember>,
    #[doc = "Selected authored ports."]
    pub ports: Vec<InspectionPort>,
    #[doc = "Selected authored connections."]
    pub connections: Vec<InspectionConnection>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Selected declared execution before native preparation."]
pub struct InspectionExecution {
    #[doc = "Native analysis route."]
    pub route: ModelingAnalysisRoute,
    #[doc = "Declared execution procedure."]
    pub procedure: ModelingProcedure,
    #[doc = "Requested retained-start policy."]
    pub requested_start: pse_backend_native::solve::StartPolicy,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "Retained authored provenance of an instantiated member."]
pub struct InspectionLineage {
    #[doc = "Originating authored declaration."]
    pub declaration: DeclarationId,
    #[doc = "Selected instance identity."]
    pub instance: InstanceId,
    #[doc = "Authored qualified path."]
    pub path: String,
    #[doc = "Declarations that demanded this specialization."]
    pub demand: Vec<DeclarationId>,
    #[doc = "Declaration supplying the selected default."]
    pub default_owner: Option<DeclarationId>,
    #[doc = "Whether this member overrides an inherited default."]
    pub is_override: bool,
    #[doc = "Applied authored presets."]
    pub presets: Vec<DeclarationId>,
}
impl From<&pse_modeling::specialize::Lineage> for InspectionLineage {
    fn from(lineage: &pse_modeling::specialize::Lineage) -> Self {
        Self {
            declaration: lineage.declaration,
            instance: lineage.instance,
            path: lineage.path.clone(),
            demand: lineage.demand.clone(),
            default_owner: lineage.default_owner,
            is_override: lineage.is_override,
            presets: lineage.presets.clone(),
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One selected instantiated definition."]
pub struct InspectionInstance {
    #[doc = "Identity retained from the admitted source."]
    pub id: InstanceId,
    #[doc = "Originating definition identity."]
    pub definition: DeclarationId,
    #[doc = "Containing instance, when present."]
    pub parent: Option<InstanceId>,
    #[doc = "Authored qualified path."]
    pub path: String,
    #[doc = "Instantiated model members and their provenance."]
    pub members: BTreeMap<String, DeclarationId>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One selected model member."]
pub struct InspectionMember {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Authored declaration role."]
    pub role: ModelingDeclarationKind,
    #[doc = "Provenance of this selected member."]
    pub lineage: InspectionLineage,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One selected declared port."]
pub struct InspectionPort {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Selected symbol carried by this port."]
    pub symbol: SemanticId,
    #[doc = "Provenance of this selected member."]
    pub lineage: InspectionLineage,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One selected declared connection."]
pub struct InspectionConnection {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Source endpoint identity."]
    pub from: SemanticId,
    #[doc = "Destination endpoint identity."]
    pub to: SemanticId,
    #[doc = "Provenance of this selected member."]
    pub lineage: InspectionLineage,
}
impl super::DeclaredExecution {
    /// Observe the selected immutable revision and declaration, without solver execution.
    pub fn inspection(&self) -> ModelingInspection {
        let source = &self.model.compiled().model;
        ModelingInspection {
            execution: InspectionExecution {
                route: self.route,
                procedure: self.procedure.kind(),
                requested_start: self.requested_start,
            },
            instances: source
                .instances
                .values()
                .map(|instance| InspectionInstance {
                    id: instance.id,
                    definition: instance.definition,
                    parent: instance.parent,
                    path: instance.path.clone(),
                    members: instance.members.clone(),
                })
                .collect(),
            members: source
                .symbols
                .values()
                .map(|symbol| InspectionMember {
                    id: symbol.id,
                    role: symbol.role,
                    lineage: (&symbol.lineage).into(),
                })
                .collect(),
            ports: source
                .ports
                .values()
                .map(|port| InspectionPort {
                    id: port.id,
                    symbol: port.symbol,
                    lineage: (&port.lineage).into(),
                })
                .collect(),
            connections: source
                .connections
                .values()
                .map(|connection| InspectionConnection {
                    id: connection.id,
                    from: connection.from,
                    to: connection.to,
                    lineage: (&connection.lineage).into(),
                })
                .collect(),
        }
    }
}

/// Native limits for retained knowledge inspection.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct KnowledgeControls {
    #[doc = "Maximum inspected scalar cells."]
    pub maximum_cells: usize,
    #[doc = "Maximum retained document bytes."]
    pub maximum_bytes: usize,
}
impl Default for KnowledgeControls {
    fn default() -> Self {
        Self {
            maximum_cells: 100_000,
            maximum_bytes: 64 << 20,
        }
    }
}
/// Native limits for supplied diagnostic points.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct DiagnosticSamplesControls {
    #[doc = "Maximum supplied diagnostic samples."]
    pub maximum_samples: usize,
    #[doc = "Wall-clock allowance in seconds."]
    pub time_limit: f64,
}
impl Default for DiagnosticSamplesControls {
    fn default() -> Self {
        Self {
            maximum_samples: 128,
            time_limit: 60.,
        }
    }
}
/// Local Jacobian diagnostic work, independent of nonlinear feasibility.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct JacobianDiagnosticControls {
    #[doc = "Maximum rows considered by the search."]
    pub maximum_rows: usize,
    #[doc = "Maximum retained matrix entries."]
    pub maximum_entries: usize,
    #[doc = "Maximum native search attempts."]
    pub maximum_attempts: usize,
    #[doc = "Bound on local diagnostic multipliers."]
    pub multiplier_bound: f64,
    #[doc = "Native search feasibility tolerance."]
    pub tolerance: f64,
    #[doc = "Relative numerical rank tolerance."]
    pub rank_relative: f64,
    #[doc = "Optional native discrete-search node limit."]
    pub maximum_nodes: Option<u32>,
}
impl Default for JacobianDiagnosticControls {
    fn default() -> Self {
        Self {
            maximum_rows: 32,
            maximum_entries: 100_000,
            maximum_attempts: 64,
            multiplier_bound: 10.,
            tolerance: 1e-7,
            rank_relative: 1e-8,
            maximum_nodes: None,
        }
    }
}
#[cfg(feature = "solver-highs")]
impl JacobianDiagnosticControls {
    /// The actual native search policy; semantic checks remain in its native operation.
    pub fn policy(self) -> pse_backend_native::jacobian_diagnostics::Policy {
        pse_backend_native::jacobian_diagnostics::Policy {
            maximum_rows: self.maximum_rows,
            maximum_entries: self.maximum_entries,
            maximum_attempts: self.maximum_attempts,
            multiplier_bound: self.multiplier_bound,
            tolerance: self.tolerance,
            rank_relative: self.rank_relative,
            maximum_nodes: self.maximum_nodes,
        }
    }
}
/// Source-coordinate declarations for bounded affine-model diagnostics.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LinearDiagnosticControls {
    #[doc = "Request native infeasibility or unboundedness rays."]
    pub rays: bool,
    #[doc = "Request native irreducible infeasibility evidence."]
    pub iis: bool,
    #[doc = "Request native sensitivity ranges."]
    pub ranging: bool,
    #[doc = "Explicit lower, upper and row relaxation penalties."]
    pub relaxation: Option<(f64, f64, f64)>,
    #[doc = "Complete lower-bound penalties keyed by source coordinate."]
    pub lower_penalties: Option<BTreeMap<SemanticId, f64>>,
    #[doc = "Complete upper-bound penalties keyed by source coordinate."]
    pub upper_penalties: Option<BTreeMap<SemanticId, f64>>,
    #[doc = "Complete row penalties keyed by source coordinate."]
    pub row_penalties: Option<BTreeMap<SemanticId, f64>>,
    #[doc = "Maximum retained matrix entries."]
    pub maximum_entries: usize,
}
impl Default for LinearDiagnosticControls {
    fn default() -> Self {
        Self {
            rays: false,
            iis: false,
            ranging: false,
            relaxation: None,
            lower_penalties: None,
            upper_penalties: None,
            row_penalties: None,
            maximum_entries: 100_000,
        }
    }
}
impl LinearDiagnosticControls {
    /// Bind optional penalty maps to the exact selected source coordinates.
    pub fn request(
        self,
        columns: &[SemanticId],
        rows: &[SemanticId],
    ) -> Result<pse_backend_native::settings::highs::Request, super::WorkflowError> {
        use pse_backend_native::settings::highs::{Penalties, Request};
        let align = |values: Option<BTreeMap<SemanticId, f64>>, ids: &[SemanticId]| {
            values
                .map(|values| {
                    if values.len() != ids.len() {
                        return Err(super::contract(
                            "local penalties require every source coordinate exactly once",
                        ));
                    }
                    ids.iter()
                        .map(|id| {
                            values.get(id).copied().ok_or_else(|| {
                                super::contract("local penalty source coordinate absent")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
        };
        if self.relaxation.is_none()
            && (self.lower_penalties.is_some()
                || self.upper_penalties.is_some()
                || self.row_penalties.is_some())
        {
            return Err(super::contract(
                "local penalties require explicit global relaxation penalties",
            ));
        }
        let relaxation = self
            .relaxation
            .map(|(lower, upper, row)| {
                let finite = |value| {
                    pse_model::scalars::FiniteBound::try_new(value).map_err(|error| {
                        super::contract(format!("global relaxation penalty: {error}"))
                    })
                };
                Ok::<_, super::WorkflowError>(Penalties {
                    global: [finite(lower)?, finite(upper)?, finite(row)?],
                    lower: align(self.lower_penalties, columns)?,
                    upper: align(self.upper_penalties, columns)?,
                    rows: align(self.row_penalties, rows)?,
                })
            })
            .transpose()?;
        Ok(Request {
            rays: self.rays,
            iis: self.iis,
            ranging: self.ranging,
            relaxation,
            ..Request::default()
        })
    }
}
/// Source-owned conformance limits and selected fixtures.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ConformanceControls {
    #[doc = "Maximum fixtures admitted by this operation."]
    pub maximum_fixtures: usize,
    #[doc = "Maximum conformance checks performed."]
    pub maximum_checks: usize,
    #[doc = "Maximum derivative comparison cells."]
    pub derivative_cells: usize,
    #[doc = "Finite-difference comparison step."]
    pub derivative_step: f64,
    #[doc = "Derivative comparison tolerance."]
    pub derivative_tolerance: f64,
    #[doc = "Explicit fixture occurrences, or the package selection when omitted."]
    pub fixtures: Option<Vec<DeclarationId>>,
    #[doc = "Optional native diagnostic policy."]
    pub diagnostics: Option<super::ModelingDiagnosticPolicy>,
}
impl Default for ConformanceControls {
    fn default() -> Self {
        Self {
            maximum_fixtures: 1024,
            maximum_checks: 16_384,
            derivative_cells: 100_000,
            derivative_step: 1e-6,
            derivative_tolerance: 1e-4,
            fixtures: None,
            diagnostics: None,
        }
    }
}
impl ConformanceControls {
    /// Resolve fixture occurrence uniqueness before contextual fixture membership checks.
    pub fn selection(&self) -> Result<super::ModelingFixtureSelection, super::WorkflowError> {
        match &self.fixtures {
            None => Ok(super::ModelingFixtureSelection::Package),
            Some(fixtures) => {
                let selected: std::collections::BTreeSet<_> = fixtures.iter().copied().collect();
                if selected.len() != fixtures.len() {
                    return Err(super::contract("duplicate selected conformance fixture"));
                }
                Ok(super::ModelingFixtureSelection::Selected(selected))
            }
        }
    }
}

/// Limits and selection for pure document conformance, with no solver work.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PureConformanceControls {
    #[doc = "Maximum fixtures admitted by this operation."]
    pub maximum_fixtures: usize,
    #[doc = "Maximum conformance checks performed."]
    pub maximum_checks: usize,
    #[doc = "Explicit fixture occurrences, or the package selection when omitted."]
    pub fixtures: Option<Vec<DeclarationId>>,
}
impl Default for PureConformanceControls {
    fn default() -> Self {
        let shared = ConformanceControls::default();
        Self {
            maximum_fixtures: shared.maximum_fixtures,
            maximum_checks: shared.maximum_checks,
            fixtures: shared.fixtures,
        }
    }
}
impl PureConformanceControls {
    /// Resolve selection using the same native occurrence rule.
    pub fn selection(&self) -> Result<super::ModelingFixtureSelection, super::WorkflowError> {
        ConformanceControls {
            fixtures: self.fixtures.clone(),
            ..Default::default()
        }
        .selection()
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    #[test]
    fn initialization_and_diagnostic_durations_keep_closed_standard_encoding() {
        use super::super::{InitializationOverrides, ModelingNonlinearPolicy};
        let duration = std::time::Duration::new(7, 123);
        let overrides = InitializationOverrides {
            time_limit: Some(duration),
            ..Default::default()
        };
        let encoded = serde_json::to_value(&overrides).unwrap();
        assert_eq!(
            encoded["time_limit"],
            serde_json::json!({"secs": 7, "nanos": 123})
        );
        assert_eq!(
            serde_json::from_value::<InitializationOverrides>(encoded.clone())
                .unwrap()
                .time_limit,
            Some(duration)
        );
        let mut extra = encoded;
        extra["time_limit"]["seconds"] = serde_json::json!(8);
        assert!(serde_json::from_value::<InitializationOverrides>(extra).is_err());
        let policy = ModelingNonlinearPolicy {
            nominals: Default::default(),
            penalty_tolerance: 1.0,
            maximum_attempts: 1,
            time_limit: duration,
        };
        let mut encoded = serde_json::to_value(&policy).unwrap();
        assert_eq!(
            serde_json::from_value::<ModelingNonlinearPolicy>(encoded.clone())
                .unwrap()
                .time_limit,
            duration
        );
        encoded["time_limit"]["secs"] = serde_json::json!(u64::MAX);
        encoded["time_limit"]["nanos"] = serde_json::json!(u32::MAX);
        assert!(serde_json::from_value::<ModelingNonlinearPolicy>(encoded).is_err());
        for schema in [
            schemars::schema_for!(InitializationOverrides).to_value(),
            schemars::schema_for!(ModelingNonlinearPolicy).to_value(),
        ] {
            assert_eq!(
                schema["$defs"]["ClosedDuration"]["additionalProperties"],
                false
            );
            assert_eq!(
                schema["$defs"]["ClosedDuration"]["required"],
                serde_json::json!(["secs", "nanos"])
            );
        }
    }
    #[test]
    fn fixture_occurrences_refuse_duplicates_before_context_admission() {
        let declaration = DeclarationId::from_id(SemanticId::from_bytes([1; 16]));
        let controls = PureConformanceControls {
            fixtures: Some(vec![declaration, declaration]),
            ..Default::default()
        };
        assert!(
            controls
                .selection()
                .unwrap_err()
                .to_string()
                .contains("duplicate selected conformance fixture")
        );
    }
    #[test]
    fn linear_penalties_bind_exact_selected_source_coordinates() {
        let column = SemanticId::from_bytes([1; 16]);
        let other = SemanticId::from_bytes([2; 16]);
        let request = LinearDiagnosticControls {
            relaxation: Some((1., 2., 3.)),
            lower_penalties: Some([(column, 4.)].into()),
            ..Default::default()
        }
        .request(&[column], &[])
        .unwrap();
        assert_eq!(request.relaxation.unwrap().lower, Some(vec![4.]));
        assert!(
            LinearDiagnosticControls {
                relaxation: Some((1., 2., 3.)),
                lower_penalties: Some([(other, 4.)].into()),
                ..Default::default()
            }
            .request(&[column], &[])
            .is_err()
        );
        assert!(
            LinearDiagnosticControls {
                lower_penalties: Some([(column, 4.)].into()),
                ..Default::default()
            }
            .request(&[column], &[])
            .is_err()
        );
    }
    #[test]
    fn omitted_conformance_policy_matches_explicit_owner_defaults() {
        let omitted: ConformanceControls = serde_json::from_str("{}").unwrap();
        let explicit = ConformanceControls::default();
        assert_eq!(omitted.maximum_checks, explicit.maximum_checks);
        assert_eq!(
            omitted.derivative_step.to_bits(),
            explicit.derivative_step.to_bits()
        );
        assert!(matches!(
            omitted.selection().unwrap(),
            super::super::ModelingFixtureSelection::Package
        ));
        assert!(
            serde_json::from_str::<PureConformanceControls>(r#"{"derivative_cells":1}"#).is_err()
        );
    }
}
