// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared topology is independent of residual incidence and numerical solve order.
use super::*;
use pse_structural::flowsheet::{Connection, Decision, Declaration, FlowGraph, Node};
use std::collections::BTreeSet;
/// Explicit selected nodes, including isolates, and a policy for every selected connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingFlowSelection {
    /// Instantiated owners. There is no inference from equation dependencies.
    pub nodes: BTreeSet<InstanceId>,
    /// Connection occurrence to its tear decision. Grouping is explicit.
    pub connections: BTreeMap<SemanticId, Decision>,
}
/// Decoded flow selection. Identity membership and physical closure are admitted by `flow_graph`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlowSelectionDocument {
    /// Explicit selected instances; repeated identities refuse before set construction.
    pub nodes: Vec<InstanceId>,
    /// One decision for every selected connection occurrence.
    pub connections: Vec<FlowConnectionDocument>,
}
/// Decision transport owned by the compiler's flow operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlowConnectionDocument {
    /// Selected declared connection occurrence.
    pub connection: SemanticId,
    /// Explicit shared tear-decision identity.
    pub group: SemanticId,
    /// Finite nonnegative authored tear cost.
    pub cost: f64,
    /// Mandatory, forbidden or freely selected tear policy.
    pub policy: pse_structural::flowsheet::Policy,
}
impl TryFrom<FlowSelectionDocument> for ModelingFlowSelection {
    type Error = CompileError;
    fn try_from(document: FlowSelectionDocument) -> Result<Self> {
        let mut nodes = BTreeSet::new();
        for node in document.nodes {
            if !nodes.insert(node) {
                return Err(CompileError::Missing("duplicate selected flow node".into()));
            }
        }
        let mut connections = BTreeMap::new();
        for connection in document.connections {
            if !connection.cost.is_finite() || connection.cost < 0.0 {
                return Err(CompileError::Missing(
                    "tear cost must be finite and nonnegative".into(),
                ));
            }
            if connections
                .insert(
                    connection.connection,
                    Decision {
                        id: connection.group,
                        cost: connection.cost,
                        policy: connection.policy,
                    },
                )
                .is_some()
            {
                return Err(CompileError::Missing(
                    "duplicate selected connection".into(),
                ));
            }
        }
        Ok(Self { nodes, connections })
    }
}
impl PreparedModeling {
    /// Project declared port ownership and directed connections into the existing graph library.
    pub fn flow_graph(
        &self,
        selection: &ModelingFlowSelection,
        quantities: &QuantityRegistry,
    ) -> Result<FlowGraph> {
        if selection.nodes.is_empty()
            || selection
                .nodes
                .iter()
                .any(|n| !self.model.instances.contains_key(n))
        {
            return Err(CompileError::Missing(
                "flow selection needs known instance nodes".into(),
            ));
        }
        let mut nodes = Vec::new();
        for id in &selection.nodes {
            let mut ports = Vec::new();
            for port in self
                .model
                .ports
                .values()
                .filter(|p| p.lineage.instance == *id)
            {
                let row = ModelingOutput::Member(port.symbol).row_id();
                let quantity = self
                    .admitted
                    .case
                    .rows()
                    .iter()
                    .find(|r| r.id == row)
                    .ok_or_else(|| {
                        CompileError::Missing("admitted physical flow coordinate".into())
                    })?
                    .quantity;
                let unit = quantities
                    .quantity_type(quantity)
                    .map_err(MathError::from)?
                    .canonical_unit;
                ports.push(pse_kernels::Port {
                    id: port.id,
                    quantity,
                    unit,
                });
            }
            nodes.push(Node {
                id: id.as_id(),
                ports,
            });
        }
        let mut connections = Vec::new();
        let mut decisions = BTreeMap::new();
        for c in self.model.connections.values() {
            let owner = |id: SemanticId| {
                self.model
                    .material_ports
                    .get(&id)
                    .map(|p| p.lineage.instance)
                    .or_else(|| self.model.ports.get(&id).map(|p| p.lineage.instance))
                    .ok_or_else(|| CompileError::Missing("connection endpoint topology".into()))
            };
            let from = owner(c.from)?;
            let to = owner(c.to)?;
            if !selection.nodes.contains(&from) && !selection.nodes.contains(&to) {
                continue;
            }
            if !selection.nodes.contains(&from) || !selection.nodes.contains(&to) {
                return Err(CompileError::Missing(
                    "flow selection cuts a declared connection".into(),
                ));
            }
            let decision = selection.connections.get(&c.id).ok_or_else(|| {
                CompileError::Missing(
                    "every selected connection needs an explicit tear policy".into(),
                )
            })?;
            if let Some(old) = decisions.insert(decision.id, decision.clone())
                && old != *decision
            {
                return Err(CompileError::Missing(
                    "conflicting tear group policy".into(),
                ));
            }
            connections.push(Connection {
                id: c.id,
                from: from.as_id(),
                to: to.as_id(),
                decision: decision.id,
                bindings: c.bindings.clone(),
            });
        }
        if connections.len() != selection.connections.len() {
            return Err(CompileError::Missing(
                "tear policy names a connection outside the selected topology".into(),
            ));
        }
        Ok(FlowGraph::admit(
            Declaration {
                nodes,
                connections,
                decisions: decisions.into_values().collect(),
            },
            quantities,
            GraphLimits {
                nodes: 100_000,
                edges: 1_000_000,
            },
        )?)
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    use pse_structural::flowsheet::Policy;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn connection(cost: f64) -> FlowConnectionDocument {
        FlowConnectionDocument {
            connection: id(2),
            group: id(3),
            cost,
            policy: Policy::Mandatory,
        }
    }
    #[test]
    fn flow_document_refuses_duplicate_occurrences_before_set_construction() {
        let node = InstanceId::from_id(id(1));
        assert!(
            ModelingFlowSelection::try_from(FlowSelectionDocument {
                nodes: vec![node, node],
                connections: vec![],
            })
            .unwrap_err()
            .to_string()
            .contains("duplicate selected flow node")
        );
        assert!(
            ModelingFlowSelection::try_from(FlowSelectionDocument {
                nodes: vec![node],
                connections: vec![connection(1.), connection(1.)],
            })
            .unwrap_err()
            .to_string()
            .contains("duplicate selected connection")
        );
    }
    #[test]
    fn flow_document_keeps_native_cost_admission_after_shape_decode() {
        for cost in [-1., f64::NAN, f64::INFINITY] {
            assert!(
                ModelingFlowSelection::try_from(FlowSelectionDocument {
                    nodes: vec![InstanceId::from_id(id(1))],
                    connections: vec![connection(cost)],
                })
                .is_err()
            );
        }
        let admitted = ModelingFlowSelection::try_from(FlowSelectionDocument {
            nodes: vec![InstanceId::from_id(id(1))],
            connections: vec![connection(-0.)],
        })
        .unwrap();
        assert_eq!(
            admitted.connections[&id(2)].cost.to_bits(),
            (-0f64).to_bits()
        );
        assert_eq!(admitted.connections[&id(2)].policy, Policy::Mandatory);
    }
    #[test]
    fn flow_document_refuses_unknown_policy_and_fields() {
        let source = format!(
            r#"{{"nodes":["{}"],"connections":[{{"connection":"{}","group":"{}","cost":1.0,"policy":"automatic"}}]}}"#,
            id(1).to_hex(),
            id(2).to_hex(),
            id(3).to_hex()
        );
        assert!(serde_json::from_str::<FlowSelectionDocument>(&source).is_err());
        assert!(
            serde_json::from_str::<FlowSelectionDocument>(
                r#"{"nodes":[],"connections":[],"infer":true}"#
            )
            .is_err()
        );
    }
}
