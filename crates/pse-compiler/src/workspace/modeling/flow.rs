// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Declared topology is independent of residual incidence and numerical solve order.
use super::*;
use pse_structural::flowsheet::{Connection, Declaration, Decision, FlowGraph, Node};
use std::collections::BTreeSet;
/// Explicit selected nodes, including isolates, and a policy for every selected connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingFlowSelection {
    /// Instantiated owners. There is no inference from equation dependencies.
    pub nodes: BTreeSet<InstanceId>,
    /// Connection occurrence to its tear decision. Grouping is explicit.
    pub connections: BTreeMap<SemanticId, Decision>,
}
impl PreparedModeling {
    /// Project declared port ownership and directed connections into the existing graph library.
    pub fn flow_graph(&self, selection: &ModelingFlowSelection, quantities: &QuantityRegistry) -> Result<FlowGraph> {
        if selection.nodes.is_empty() || selection.nodes.iter().any(|n| !self.model.instances.contains_key(n)) {
            return Err(CompileError::Missing("flow selection needs known instance nodes".into()));
        }
        let mut nodes = Vec::new();
        for id in &selection.nodes {
            let mut ports=Vec::new();
            for port in self.model.ports.values().filter(|p|p.lineage.instance==*id) {
                let row=ModelingOutput::Member(port.symbol).row_id();
                let quantity=self.admitted.case.rows().iter().find(|r|r.id==row)
                    .ok_or_else(||CompileError::Missing("admitted physical flow coordinate".into()))?.quantity;
                let unit=quantities.quantity_type(quantity).map_err(pse_math::MathError::from)?.canonical_unit;
                ports.push(pse_kernels::Port{id:port.id,quantity,unit});
            }
            nodes.push(Node{id:id.as_id(),ports});
        }
        let mut connections=Vec::new();
        let mut decisions=BTreeMap::new();
        for c in self.model.connections.values() {
            let source=self.model.ports.get(&c.from).ok_or_else(||CompileError::Missing("source port topology".into()))?;
            let target=self.model.ports.get(&c.to).ok_or_else(||CompileError::Missing("destination port topology".into()))?;
            let from=source.lineage.instance;
            let to=target.lineage.instance;
            if !selection.nodes.contains(&from) && !selection.nodes.contains(&to) {continue;}
            if !selection.nodes.contains(&from) || !selection.nodes.contains(&to) {
                return Err(CompileError::Missing("flow selection cuts a declared connection".into()));
            }
            let decision=selection.connections.get(&c.id).ok_or_else(||CompileError::Missing("every selected connection needs an explicit tear policy".into()))?;
            if let Some(old)=decisions.insert(decision.id,decision.clone()) && old!=*decision {
                return Err(CompileError::Missing("conflicting tear group policy".into()));
            }
            connections.push(Connection{id:c.id,from:from.as_id(),to:to.as_id(),decision:decision.id,bindings:vec![(c.from,c.to)]});
        }
        if connections.len()!=selection.connections.len() {
            return Err(CompileError::Missing("tear policy names a connection outside the selected topology".into()));
        }
        Ok(FlowGraph::admit(Declaration{nodes,connections,decisions:decisions.into_values().collect()},quantities,GraphLimits{nodes:100_000,edges:1_000_000})?)
    }
}
