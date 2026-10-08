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
/// Checked finite process meaning, independently usable without numerical body admission.
#[derive(Clone, Debug)]
pub struct SemanticModeling {
    /// Instantiated members and original physical topology.
    pub model: pse_math::SharedAllocation<SpecializedModel>,
    port_quantities: Arc<BTreeMap<SemanticId, QuantityTypeId>>,
}
impl SemanticModeling {
    pub(super) fn admit(
        model: pse_math::SharedAllocation<SpecializedModel>,
        quantities: &QuantityRegistry,
        preconditions: &PhysicalPreconditions,
    ) -> Result<Self> {
        let port_quantities = model
            .ports
            .values()
            .map(|port| {
                let symbol = model
                    .symbols
                    .get(&port.symbol)
                    .ok_or_else(|| CompileError::Missing("physical flow symbol".into()))?;
                let scheme = symbol
                    .ty
                    .quantity_scheme()
                    .ok_or_else(|| CompileError::Missing("physical flow coordinate".into()))?;
                let quantity = scheme
                    .resolve_with_evidence(
                        quantities,
                        &pse_quantity::scheme::Substitution::new(),
                        preconditions,
                    )
                    .map_err(|error| CompileError::Missing(error.to_string()))?;
                Ok((port.id, quantity))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Self {
            model,
            port_quantities: Arc::new(port_quantities),
        })
    }
    /// Charge semantic allocation while aliases survive.
    pub fn retained_bytes(&self) -> usize {
        self.descriptor_bytes() + self.model.retained_bytes()
    }
    pub(super) fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + 256
            + self.port_quantities.len()
                * (size_of::<SemanticId>() + size_of::<QuantityTypeId>() + 64)
    }
    /// Attach the existing runtime product owner to escaping semantic aliases.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.model = self.model.with_owner(owner);
        self
    }
    /// Known selected source populations before declaration/physical graph allocation.
    /// Invalid selections still use bounded source subsets and refuse in `flow_graph`.
    pub fn flow_allocation_bound(
        &self,
        selection: &ModelingFlowSelection,
        quantities: &QuantityRegistry,
    ) -> Result<usize> {
        let overflow = || CompileError::from(MathError::Limit("flow construction extent"));
        let ports = self
            .model
            .ports
            .values()
            .filter(|port| selection.nodes.contains(&port.lineage.instance))
            .count();
        let mut connections = 0usize;
        let mut bindings = 0usize;
        let mut quantity_payload = 0usize;
        for connection in self.model.connections.values() {
            let owner = |id| {
                self.model
                    .material_ports
                    .get(&id)
                    .map(|port| port.lineage.instance)
                    .or_else(|| self.model.ports.get(&id).map(|port| port.lineage.instance))
            };
            if !owner(connection.from).is_some_and(|id| selection.nodes.contains(&id))
                && !owner(connection.to).is_some_and(|id| selection.nodes.contains(&id))
            {
                continue;
            }
            connections = connections.checked_add(1).ok_or_else(overflow)?;
            bindings = bindings
                .checked_add(connection.bindings.len())
                .ok_or_else(overflow)?;
            for (source, _) in &connection.bindings {
                let quantity = self
                    .port_quantities
                    .get(source)
                    .ok_or_else(|| CompileError::Missing("physical flow coordinate".into()))?;
                let quantity = quantities
                    .quantity_type(*quantity)
                    .map_err(MathError::from)?;
                quantity_payload = quantity
                    .key
                    .shape
                    .len()
                    .checked_mul(size_of::<pse_quantity::EntityKindId>())
                    .and_then(|n| n.checked_add(quantity.name.as_ref().map_or(0, String::len)))
                    .and_then(|n| quantity_payload.checked_add(n))
                    .ok_or_else(overflow)?;
            }
        }
        Ok(FlowGraph::construction_allocation_bound(
            selection.nodes.len(),
            ports,
            connections,
            selection.connections.len(),
            bindings,
            quantity_payload,
        )?)
    }
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
                let quantity = *self.port_quantities.get(&port.id).ok_or_else(|| {
                    CompileError::Missing("admitted physical flow coordinate".into())
                })?;
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
