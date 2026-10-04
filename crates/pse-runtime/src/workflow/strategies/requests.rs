// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Serializable causal requests remain available independently of native backend selection.
use pse_ids::SemanticId;
use std::collections::BTreeSet;

/// Preserve unique-items refusal before an operation receives its set product.
fn distinct_ids<'de, D>(deserializer: D) -> Result<BTreeSet<SemanticId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::{Deserialize, de::Error};
    let occurrences = Vec::<SemanticId>::deserialize(deserializer)?;
    let mut admitted = BTreeSet::new();
    for (position, identity) in occurrences.into_iter().enumerate() {
        if !admitted.insert(identity) {
            return Err(D::Error::custom(format!(
                "duplicate selected identity {identity} at occurrence {position}"
            )));
        }
    }
    Ok(admitted)
}
/// One declared mathematical realization of a causal unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CausalUnitRealization {
    /// Authored output functions with every free dependency declared at the boundary.
    ExplicitMap,
    /// Original owned residuals, local solved coordinates and one declared root procedure.
    Conditional {
        /// Complete selected original unit equality identities.
        #[serde(deserialize_with = "distinct_ids")]
        residuals: BTreeSet<SemanticId>,
        /// Remaining local free coordinates after boundary inputs are fixed.
        #[serde(deserialize_with = "distinct_ids")]
        unknowns: BTreeSet<SemanticId>,
        /// The canonical solver settings document; defaults resolve in its existing owner.
        solver: Box<crate::math::settings::SolveSettings>,
    },
}
/// Explicit causal direction and admitted mathematical realization for one unit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CausalUnitRequest {
    /// Owning selected flow node.
    pub node: SemanticId,
    /// Input ports; their coordinates are owned by the authored port declarations.
    #[serde(deserialize_with = "distinct_ids")]
    pub inputs: BTreeSet<SemanticId>,
    /// Output ports; their expressions are owned by the authored port declarations.
    #[serde(deserialize_with = "distinct_ids")]
    pub outputs: BTreeSet<SemanticId>,
    /// A unit equation solve is declared independently of an explicit function map.
    pub realization: CausalUnitRealization,
}
/// Concrete tear witness and causal directions; native KINSOL owns all recycle iteration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecycleRequest {
    /// Exact selected tear decision groups, obtainable through select_tears.
    #[serde(deserialize_with = "distinct_ids")]
    pub tears: BTreeSet<SemanticId>,
    /// Complete admitted causal unit inventory.
    pub units: Vec<CausalUnitRequest>,
    /// Native Anderson history; zero means unaccelerated fixed point.
    pub anderson: usize,
    /// Native fixed-point damping in (0,1].
    pub damping: pse_model::scalars::Fraction,
}
impl RecycleRequest {
    /// Bind a complete compiler-issued causal inventory and a checked tear witness.
    /// Explicit tears constrain selection; absent tears use the existing graph
    /// heuristic, which still validates mandatory/forbidden policies and the DAG.
    pub fn from_model(
        model: &pse_compiler::workspace::PreparedModeling,
        source: &pse_compiler::workspace::PreparedCase,
        selection: &pse_compiler::workspace::ModelingFlowSelection,
        quantities: &pse_quantity::QuantityRegistry,
        solver: crate::math::settings::SolveSettings,
        tears: Option<BTreeSet<SemanticId>>,
    ) -> Result<Self, crate::workflow::WorkflowError> {
        let graph = model
            .semantic()
            .flow_graph(selection, quantities)
            .map_err(crate::math::MathRuntimeError::from)?;
        let tears = match tears {
            Some(tears) => {
                graph
                    .witness(&tears)
                    .map_err(|cause| super::contract(cause.to_string()))?;
                tears
            }
            None => graph
                .unweighted_heuristic()
                .map_err(|cause| super::contract(cause.to_string()))?,
        };
        let units = model
            .automatic_causal_units(source, &selection.nodes)
            .map_err(crate::math::MathRuntimeError::from)?
            .into_iter()
            .map(|unit| {
                let realization =
                    if unit.local.residuals.is_empty() && unit.local.unknowns.is_empty() {
                        CausalUnitRealization::ExplicitMap
                    } else {
                        CausalUnitRealization::Conditional {
                            residuals: unit.local.residuals,
                            unknowns: unit.local.unknowns,
                            solver: Box::new(solver.clone()),
                        }
                    };
                CausalUnitRequest {
                    node: unit.node,
                    inputs: unit.inputs,
                    outputs: unit.outputs,
                    realization,
                }
            })
            .collect();
        Ok(Self {
            tears,
            units,
            anderson: 0,
            damping: pse_model::scalars::Fraction::try_new(1.0)
                .map_err(|cause| super::contract(cause.to_string()))?,
        })
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    #[test]
    fn selected_strategy_sets_refuse_duplicate_wire_occurrences() {
        let identity = "01010101010101010101010101010101";
        let duplicate = format!(r#"["{identity}","{identity}"]"#);
        let request = format!(r#"{{"tears":{duplicate},"units":[],"anderson":0,"damping":1.0}}"#);
        assert!(
            serde_json::from_str::<RecycleRequest>(&request)
                .unwrap_err()
                .to_string()
                .contains("duplicate selected identity")
        );
        for field in ["inputs", "outputs"] {
            let mut document = serde_json::json!({"node": identity, "inputs": [], "outputs": [], "realization": {"kind": "explicit_map"}});
            document[field] = serde_json::from_str(&duplicate).unwrap();
            assert!(
                serde_json::from_value::<CausalUnitRequest>(document)
                    .unwrap_err()
                    .to_string()
                    .contains("duplicate selected identity")
            );
        }
        for field in ["residuals", "unknowns"] {
            let mut document = serde_json::json!({"kind": "conditional", "residuals": [], "unknowns": [], "solver": {"version": 1}});
            document[field] = serde_json::from_str(&duplicate).unwrap();
            assert!(
                serde_json::from_value::<CausalUnitRealization>(document)
                    .unwrap_err()
                    .to_string()
                    .contains("duplicate selected identity")
            );
        }
        let single = format!(r#"{{"tears":["{identity}"],"units":[],"anderson":0,"damping":1.0}}"#);
        assert_eq!(
            serde_json::from_str::<RecycleRequest>(&single)
                .unwrap()
                .tears
                .len(),
            1
        );
    }
}
